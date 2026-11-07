//! HTTP/1.x wire handling shared by `http.rs` and `router.rs`.
//!
//! Limits, strict request parsing and response writing live here so both servers
//! behave the same. Unsupported features (chunked request bodies, `Expect`) are
//! rejected explicitly instead of being half-handled.

#[cfg(test)]
use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
#[cfg(test)]
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

use crate::error::fatal;

pub(crate) const MAX_REQUEST_LINE: usize = 8 * 1024;
pub(crate) const MAX_HEADER_BYTES: usize = 32 * 1024;
pub(crate) const MAX_HEADERS: usize = 100;
pub(crate) const MAX_BODY: usize = 8 * 1024 * 1024;
const MAX_ACTIVE_CONNECTIONS: usize = 1024;
const MAX_REQUESTS_PER_CONNECTION: usize = 10_000;
const METHODS: [&str; 7] = ["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];

pub(crate) struct Request {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub keep_alive: bool,
}

pub(crate) enum ReadError {
    /// Clean close, reset or timeout: just drop the connection.
    Closed,
    /// Answer with this status and a short message, then close.
    Reject(u16, &'static str),
}

pub(crate) fn checked_port(port: i64) -> u16 {
    u16::try_from(port)
        .unwrap_or_else(|_| fatal(&format!("port {port} is out of range (0 to 65535)")))
}

#[cfg(test)]
pub(crate) fn worker_count() -> usize {
    worker_count_with(
        std::env::var("KODE_HTTP_WORKERS").ok(),
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    )
}

#[cfg(test)]
fn worker_count_with(raw: Option<String>, available_parallelism: usize) -> usize {
    raw.and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or_else(|| available_parallelism.max(4))
}

/// Accept forever. A failed accept (for example "too many open files") backs off
/// instead of spinning at 100% CPU.
#[cfg(test)]
pub(crate) fn accept_loop<F>(listener: TcpListener, handle: F)
where
    F: Fn(TcpStream) + Send + Sync + 'static,
{
    let handle = Arc::new(handle);
    let workers = worker_count().max(1);
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let cv = Arc::new(Condvar::new());

    for _ in 0..workers {
        let h = Arc::clone(&handle);
        let q = Arc::clone(&queue);
        let c = Arc::clone(&cv);
        std::thread::spawn(move || loop {
            let stream = {
                let mut guard = q.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if let Some(s) = guard.pop_front() {
                        // Wake an acceptor that is waiting for room in the queue.
                        c.notify_all();
                        break Some(s);
                    }
                    guard = c.wait(guard).unwrap_or_else(|e| e.into_inner());
                }
            };
            let Some(stream) = stream else {
                continue;
            };
            h(stream);
        });
    }

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                let mut guard = queue.lock().unwrap_or_else(|e| e.into_inner());
                while guard.len() >= workers.saturating_mul(2).max(4) {
                    guard = cv.wait(guard).unwrap_or_else(|e| e.into_inner());
                }
                guard.push_back(stream);
                cv.notify_all();
            }
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// `TcpListener::bind` leaves a small accept backlog (128). A burst of 1000
/// connections overflows it, the kernel drops SYNs, and clients wait 1s to retry.
/// Calling `listen` again on the same socket raises the limit (the kernel caps it
/// at `net.core.somaxconn`).
#[cfg(unix)]
fn raise_backlog(listener: &TcpListener) {
    use std::os::unix::io::AsRawFd;
    unsafe extern "C" {
        fn listen(fd: i32, backlog: i32) -> i32;
    }
    unsafe {
        listen(listener.as_raw_fd(), 4096);
    }
}

#[cfg(not(unix))]
fn raise_backlog(_listener: &TcpListener) {}

struct ActiveGuard(Arc<AtomicUsize>);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Portable fallback: one thread per connection, capped at `MAX_ACTIVE_CONNECTIONS`.
fn threaded_loop<F>(listener: TcpListener, dispatch: Arc<F>)
where
    F: Fn(&Request) -> (i64, Vec<u8>, Vec<u8>) + Send + Sync + 'static,
{
    let active = Arc::new(AtomicUsize::new(0));

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if active.fetch_add(1, Ordering::AcqRel) >= MAX_ACTIVE_CONNECTIONS {
                    active.fetch_sub(1, Ordering::AcqRel);
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    continue;
                }
                let d = Arc::clone(&dispatch);
                let guard = ActiveGuard(Arc::clone(&active));
                let spawned = thread::Builder::new().spawn(move || {
                    let _guard = guard;
                    serve_connection(stream, |req| d(req));
                });
                if spawned.is_err() {
                    // The closure and its guard were dropped, so the count is fixed.
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// Serve forever. `dispatch` is shared and may run on several threads at once.
///
/// Linux: epoll event loop, one thread per CPU (override with `KODE_HTTP_WORKERS`;
/// `1` runs every handler on a single thread). Other systems: one thread per
/// connection.
pub(crate) fn event_loop<F>(listener: TcpListener, dispatch: F)
where
    F: Fn(&Request) -> (i64, Vec<u8>, Vec<u8>) + Send + Sync + 'static,
{
    raise_backlog(&listener);
    let dispatch = Arc::new(dispatch);

    #[cfg(target_os = "linux")]
    {
        if let Some(ep) = epoll::Epoll::new() {
            return epoll::run(listener, dispatch, ep);
        }
    }

    threaded_loop(listener, dispatch)
}

#[cfg(target_os = "linux")]
mod epoll {
    //! Readiness-based server loop. Sockets are non-blocking; each worker thread owns
    //! an epoll instance and a slab of connections, reads whatever arrived, answers
    //! every complete request in one `write`, and never blocks on a slow client.

    use super::*;
    use std::io::ErrorKind;
    use std::os::unix::io::AsRawFd;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::time::Instant;

    const EPOLLIN: u32 = 0x001;
    const EPOLLOUT: u32 = 0x004;
    const EPOLLERR: u32 = 0x008;
    const EPOLLHUP: u32 = 0x010;
    const EPOLLEXCLUSIVE: u32 = 1 << 28;
    const EPOLL_CTL_ADD: i32 = 1;
    const EPOLL_CTL_MOD: i32 = 3;
    const EPOLL_CLOEXEC: i32 = 0o2_000_000;

    const LISTENER: u64 = u64::MAX;
    const READ_BUF: usize = 16 * 1024;
    const MAX_EVENTS: usize = 256;
    const ACCEPTS_PER_WAKE: usize = 32;
    /// A keep-alive connection with nothing in flight.
    const IDLE: Duration = Duration::from_secs(60);
    /// A request that started but stopped making progress.
    const PARTIAL: Duration = Duration::from_secs(5);
    const SWEEP_EVERY: Duration = Duration::from_secs(1);

    // The kernel packs this struct on x86_64 only.
    #[cfg_attr(target_arch = "x86_64", repr(C, packed))]
    #[cfg_attr(not(target_arch = "x86_64"), repr(C))]
    #[derive(Clone, Copy)]
    struct EpollEvent {
        events: u32,
        data: u64,
    }

    unsafe extern "C" {
        fn epoll_create1(flags: i32) -> i32;
        fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut EpollEvent) -> i32;
        fn epoll_wait(epfd: i32, events: *mut EpollEvent, maxevents: i32, timeout: i32) -> i32;
        fn close(fd: i32) -> i32;
    }

    pub(super) struct Epoll(i32);

    impl Epoll {
        pub(super) fn new() -> Option<Epoll> {
            let fd = unsafe { epoll_create1(EPOLL_CLOEXEC) };
            (fd >= 0).then_some(Epoll(fd))
        }

        fn ctl(&self, op: i32, fd: i32, events: u32, token: u64) -> bool {
            let mut ev = EpollEvent { events, data: token };
            unsafe { epoll_ctl(self.0, op, fd, &mut ev) == 0 }
        }

        fn wait(&self, out: &mut [EpollEvent], timeout_ms: i32) -> usize {
            let n = unsafe { epoll_wait(self.0, out.as_mut_ptr(), out.len() as i32, timeout_ms) };
            if n < 0 {
                0
            } else {
                n as usize
            }
        }
    }

    impl Drop for Epoll {
        fn drop(&mut self) {
            unsafe {
                close(self.0);
            }
        }
    }

    struct Conn {
        stream: TcpStream,
        /// Bytes of an unfinished request. Empty on the fast path.
        buf: Vec<u8>,
        /// Headers already parsed while the body is still arriving, with their length.
        head: Option<(Head, usize)>,
        /// Response bytes the socket could not take yet.
        out: Vec<u8>,
        out_pos: usize,
        /// Close once `out` is flushed.
        closing: bool,
        served: usize,
        last: Instant,
    }

    impl Conn {
        fn pending_write(&self) -> bool {
            self.out_pos < self.out.len()
        }
    }

    fn worker_threads() -> usize {
        std::env::var("KODE_HTTP_WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            })
            .min(256)
    }

    pub(super) fn run<F>(listener: TcpListener, dispatch: Arc<F>, first: Epoll)
    where
        F: Fn(&Request) -> (i64, Vec<u8>, Vec<u8>) + Send + Sync + 'static,
    {
        if listener.set_nonblocking(true).is_err() {
            fatal("cannot make the listening socket non-blocking");
        }
        let listener = Arc::new(listener);
        let active = Arc::new(AtomicUsize::new(0));

        for _ in 1..worker_threads() {
            let Some(ep) = Epoll::new() else { break };
            let l = Arc::clone(&listener);
            let d = Arc::clone(&dispatch);
            let a = Arc::clone(&active);
            let spawned = thread::Builder::new().spawn(move || Worker::new(l, d, a, ep).run());
            if spawned.is_err() {
                break;
            }
        }
        Worker::new(listener, dispatch, active, first).run();
    }

    struct Worker<F> {
        ep: Epoll,
        listener: Arc<TcpListener>,
        dispatch: Arc<F>,
        active: Arc<AtomicUsize>,
        slab: Vec<Option<Conn>>,
        free: Vec<usize>,
        rbuf: Vec<u8>,
        scratch: Vec<u8>,
        now: Instant,
    }

    impl<F> Worker<F>
    where
        F: Fn(&Request) -> (i64, Vec<u8>, Vec<u8>),
    {
        fn new(listener: Arc<TcpListener>, dispatch: Arc<F>, active: Arc<AtomicUsize>, ep: Epoll) -> Self {
            let fd = listener.as_raw_fd();
            // EPOLLEXCLUSIVE wakes one worker per connection instead of all of them.
            if !ep.ctl(EPOLL_CTL_ADD, fd, EPOLLIN | EPOLLEXCLUSIVE, LISTENER)
                && !ep.ctl(EPOLL_CTL_ADD, fd, EPOLLIN, LISTENER)
            {
                fatal("cannot watch the listening socket");
            }
            Worker {
                ep,
                listener,
                dispatch,
                active,
                slab: Vec::new(),
                free: Vec::new(),
                rbuf: vec![0u8; READ_BUF],
                scratch: Vec::with_capacity(4 * 1024),
                now: Instant::now(),
            }
        }

        fn run(mut self) {
            let mut events = [EpollEvent { events: 0, data: 0 }; MAX_EVENTS];
            let mut last_sweep = Instant::now();
            loop {
                let n = self.ep.wait(&mut events, 1000);
                self.now = Instant::now();
                for ev in events.iter().take(n) {
                    let flags = ev.events;
                    let token = ev.data;
                    if token == LISTENER {
                        self.accept_ready();
                    } else {
                        self.conn_event(token as usize, flags);
                    }
                }
                if self.now.duration_since(last_sweep) >= SWEEP_EVERY {
                    last_sweep = self.now;
                    self.sweep();
                }
            }
        }

        fn release(&mut self, idx: usize) {
            self.free.push(idx);
            self.active.fetch_sub(1, Ordering::AcqRel);
        }

        fn accept_ready(&mut self) {
            for _ in 0..ACCEPTS_PER_WAKE {
                match self.listener.accept() {
                    Ok((stream, _)) => {
                        if self.active.fetch_add(1, Ordering::AcqRel) >= MAX_ACTIVE_CONNECTIONS {
                            self.active.fetch_sub(1, Ordering::AcqRel);
                            continue; // dropping the stream closes it
                        }
                        let _ = stream.set_nodelay(true);
                        if stream.set_nonblocking(true).is_err() {
                            self.active.fetch_sub(1, Ordering::AcqRel);
                            continue;
                        }
                        let fd = stream.as_raw_fd();
                        let idx = match self.free.pop() {
                            Some(i) => i,
                            None => {
                                self.slab.push(None);
                                self.slab.len() - 1
                            }
                        };
                        if !self.ep.ctl(EPOLL_CTL_ADD, fd, EPOLLIN, idx as u64) {
                            self.release(idx);
                            continue;
                        }
                        self.slab[idx] = Some(Conn {
                            stream,
                            buf: Vec::new(),
                            head: None,
                            out: Vec::new(),
                            out_pos: 0,
                            closing: false,
                            served: 0,
                            last: self.now,
                        });
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::WouldBlock => return,
                        ErrorKind::Interrupted | ErrorKind::ConnectionAborted => continue,
                        // For example "too many open files": back off, do not spin.
                        _ => {
                            thread::sleep(Duration::from_millis(50));
                            return;
                        }
                    },
                }
            }
        }

        fn conn_event(&mut self, idx: usize, events: u32) {
            let Some(mut c) = self.slab.get_mut(idx).and_then(|slot| slot.take()) else {
                return;
            };
            let alive = if c.pending_write() {
                self.on_writable(idx, &mut c, events)
            } else {
                self.on_readable(idx, &mut c)
            };
            if alive {
                self.slab[idx] = Some(c);
            } else {
                self.release(idx); // `c` drops here and closes the socket
            }
        }

        fn on_readable(&mut self, idx: usize, c: &mut Conn) -> bool {
            let n = match c.stream.read(&mut self.rbuf) {
                Ok(0) => return false,
                Ok(n) => n,
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {
                    return true
                }
                Err(_) => return false,
            };
            c.last = self.now;
            self.scratch.clear();

            if c.buf.is_empty() {
                // Fast path: parse straight from the read buffer, copy only leftovers.
                let used = handle_input(&*self.dispatch, &mut self.scratch, c, &self.rbuf[..n]);
                if !c.closing && used < n {
                    c.buf.extend_from_slice(&self.rbuf[used..n]);
                }
            } else {
                c.buf.extend_from_slice(&self.rbuf[..n]);
                let mut data = std::mem::take(&mut c.buf);
                let used = handle_input(&*self.dispatch, &mut self.scratch, c, &data);
                if !c.closing && used < data.len() {
                    data.drain(..used);
                    c.buf = data;
                }
            }
            self.flush(idx, c)
        }

        /// Send `self.scratch`. Returns false when the connection should close.
        fn flush(&mut self, idx: usize, c: &mut Conn) -> bool {
            let mut off = 0;
            while off < self.scratch.len() {
                match c.stream.write(&self.scratch[off..]) {
                    Ok(0) => return false,
                    Ok(k) => off += k,
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => return false,
                }
            }
            if off < self.scratch.len() {
                // The client is slow: keep the rest, stop reading, wait for room.
                c.out.clear();
                c.out.extend_from_slice(&self.scratch[off..]);
                c.out_pos = 0;
                return self.ep.ctl(EPOLL_CTL_MOD, c.stream.as_raw_fd(), EPOLLOUT, idx as u64);
            }
            !c.closing
        }

        fn on_writable(&mut self, idx: usize, c: &mut Conn, events: u32) -> bool {
            while c.out_pos < c.out.len() {
                match c.stream.write(&c.out[c.out_pos..]) {
                    Ok(0) => return false,
                    Ok(k) => {
                        c.out_pos += k;
                        c.last = self.now;
                    }
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        return (events & (EPOLLERR | EPOLLHUP)) == 0;
                    }
                    Err(_) => return false,
                }
            }
            c.out = Vec::new();
            c.out_pos = 0;
            if c.closing {
                return false;
            }
            self.ep.ctl(EPOLL_CTL_MOD, c.stream.as_raw_fd(), EPOLLIN, idx as u64)
        }

        /// Drop connections that went quiet.
        fn sweep(&mut self) {
            for idx in 0..self.slab.len() {
                let stale = match &self.slab[idx] {
                    Some(c) => {
                        let quiet = self.now.duration_since(c.last);
                        if c.pending_write() {
                            quiet > WRITE_TIMEOUT
                        } else if c.buf.is_empty() && c.head.is_none() {
                            quiet > IDLE
                        } else {
                            quiet > PARTIAL
                        }
                    }
                    None => false,
                };
                if stale {
                    self.slab[idx] = None;
                    self.release(idx);
                }
            }
        }
    }

    /// Answer every complete request in `data`, appending the responses to `scratch`.
    /// Returns how many bytes were consumed; the rest is an unfinished request.
    fn handle_input<F>(dispatch: &F, scratch: &mut Vec<u8>, c: &mut Conn, data: &[u8]) -> usize
    where
        F: Fn(&Request) -> (i64, Vec<u8>, Vec<u8>),
    {
        let mut pos = 0;
        while pos < data.len() && !c.closing {
            let rest = &data[pos..];
            let (head, head_len) = match c.head.take() {
                Some(h) => h,
                None => {
                    let mut cur = rest;
                    match read_head(&mut cur) {
                        Ok(h) => (h, rest.len() - cur.len()),
                        // Reading from memory only fails by running out: wait for more.
                        Err(ReadError::Closed) => break,
                        Err(ReadError::Reject(code, msg)) => {
                            let _ = write_response(
                                scratch,
                                i64::from(code),
                                b"text/plain; charset=utf-8",
                                msg.as_bytes(),
                                false,
                                false,
                            );
                            c.closing = true;
                            return data.len();
                        }
                    }
                }
            };
            let total = head_len + head.content_length;
            if rest.len() < total {
                c.head = Some((head, head_len));
                break;
            }
            let body = rest[head_len..total].to_vec();
            pos += total;
            c.served += 1;
            let keep = head.keep_alive && c.served < MAX_REQUESTS_PER_CONNECTION;
            let req = head.into_request(body);
            let (status, body, ctype) = match catch_unwind(AssertUnwindSafe(|| dispatch(&req))) {
                Ok(r) => r,
                Err(_) => {
                    c.closing = true;
                    (
                        500,
                        b"internal error".to_vec(),
                        b"text/plain; charset=utf-8".to_vec(),
                    )
                }
            };
            let keep = keep && !c.closing;
            if !keep {
                c.closing = true;
            }
            let _ = write_response(scratch, status, &ctype, &body, keep, req.method == "HEAD");
        }
        pos
    }
}

pub(crate) fn serve_connection<F>(stream: TcpStream, mut dispatch: F)
where
    F: FnMut(&Request) -> (i64, Vec<u8>, Vec<u8>),
{
    let _ = stream.set_nodelay(true);
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
    let Ok(read_half) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::with_capacity(8 * 1024, read_half);
    let mut out = stream;

    for served in 1..=MAX_REQUESTS_PER_CONNECTION {
        let _ = reader.get_ref().set_read_timeout(Some(IDLE_TIMEOUT));
        match reader.fill_buf() {
            Ok(buf) if !buf.is_empty() => {}
            _ => return,
        }
        let _ = reader.get_ref().set_read_timeout(Some(READ_TIMEOUT));

        let req = match read_request(&mut reader) {
            Ok(r) => r,
            Err(ReadError::Closed) => return,
            Err(ReadError::Reject(code, msg)) => {
                let _ = write_response(
                    &mut out,
                    i64::from(code),
                    b"text/plain; charset=utf-8",
                    msg.as_bytes(),
                    false,
                    false,
                );
                return;
            }
        };
        let keep = req.keep_alive && served < MAX_REQUESTS_PER_CONNECTION;
        let (status, body, ctype) = dispatch(&req);
        if write_response(&mut out, status, &ctype, &body, keep, req.method == "HEAD").is_err()
            || !keep
        {
            return;
        }
    }
}

// Tests use a short idle timeout; real servers keep keep-alive connections open.
#[cfg(test)]
pub(crate) const IDLE_TIMEOUT: Duration = Duration::from_millis(300);
#[cfg(not(test))]
pub(crate) const IDLE_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const READ_TIMEOUT: Duration = Duration::from_millis(500);
pub(crate) const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

fn trim_eol(line: &mut Vec<u8>) {
    if line.last() == Some(&b'\n') {
        line.pop();
    }
    if line.last() == Some(&b'\r') {
        line.pop();
    }
}

fn is_tchar(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
}

/// Read one line without ever buffering more than `limit` bytes.
fn read_line<R: BufRead>(
    r: &mut R,
    limit: usize,
    too_long: u16,
) -> Result<Option<Vec<u8>>, ReadError> {
    let mut buf = Vec::new();
    let n = r
        .by_ref()
        .take(limit as u64 + 1)
        .read_until(b'\n', &mut buf)
        .map_err(|_| ReadError::Closed)?;
    if n == 0 {
        return Ok(None);
    }
    if buf.len() > limit {
        return Err(ReadError::Reject(
            too_long,
            "request line or header is too long",
        ));
    }
    if buf.last() != Some(&b'\n') {
        return Err(ReadError::Closed); // the peer stopped mid-line
    }
    Ok(Some(buf))
}

/// A parsed request line and headers, before the body is read.
struct Head {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    content_length: usize,
    keep_alive: bool,
}

impl Head {
    fn into_request(self, body: Vec<u8>) -> Request {
        Request {
            method: self.method,
            target: self.target,
            headers: self.headers,
            body,
            keep_alive: self.keep_alive,
        }
    }
}

/// Parse the request line and headers. Stops before the body.
fn read_head<R: BufRead>(r: &mut R) -> Result<Head, ReadError> {
    use ReadError::{Closed, Reject};

    let mut line = read_line(r, MAX_REQUEST_LINE, 414)?.ok_or(Closed)?;
    trim_eol(&mut line);
    if line.is_empty() {
        // One empty line before the request line is allowed (RFC 9112, 2.2).
        line = read_line(r, MAX_REQUEST_LINE, 414)?.ok_or(Closed)?;
        trim_eol(&mut line);
    }
    let text = std::str::from_utf8(&line).map_err(|_| Reject(400, "malformed request line"))?;
    let mut parts = text.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Reject(400, "malformed request line"));
    };
    if method.is_empty() || !method.bytes().all(is_tchar) {
        return Err(Reject(400, "malformed request method"));
    }
    if !METHODS.contains(&method) {
        return Err(Reject(501, "this HTTP method is not supported"));
    }
    if !(target.starts_with('/') || target == "*") || target.bytes().any(|b| b < 0x21 || b == 0x7f)
    {
        return Err(Reject(400, "malformed request target"));
    }
    let http11 = match version {
        "HTTP/1.1" => true,
        "HTTP/1.0" => false,
        _ => return Err(Reject(505, "only HTTP/1.0 and HTTP/1.1 are supported")),
    };

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut total = 0usize;
    let mut content_length: Option<usize> = None;
    let (mut close, mut keep) = (false, false);
    loop {
        let mut raw = read_line(r, MAX_HEADER_BYTES, 431)?.ok_or(Closed)?;
        trim_eol(&mut raw);
        if raw.is_empty() {
            break;
        }
        total += raw.len() + 2;
        if total > MAX_HEADER_BYTES || headers.len() >= MAX_HEADERS {
            return Err(Reject(431, "the request headers are too large"));
        }
        let text = std::str::from_utf8(&raw).map_err(|_| Reject(400, "malformed header"))?;
        let Some((name, value)) = text.split_once(':') else {
            return Err(Reject(400, "malformed header"));
        };
        // A name with spaces, or a continuation line, is a classic smuggling vector.
        if name.is_empty() || !name.bytes().all(is_tchar) {
            return Err(Reject(400, "malformed header name"));
        }
        let value = value.trim_matches(|c| c == ' ' || c == '\t');
        if value.bytes().any(|b| (b < 0x20 && b != b'\t') || b == 0x7f) {
            return Err(Reject(400, "malformed header value"));
        }
        let lname = name.to_ascii_lowercase();
        match lname.as_str() {
            "content-length" => {
                if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(Reject(400, "invalid Content-Length"));
                }
                // Digits that do not fit in usize are certainly over the limit.
                let n: usize = value
                    .parse()
                    .map_err(|_| Reject(413, "the request body is too large"))?;
                if content_length.is_some_and(|prev| prev != n) {
                    return Err(Reject(400, "conflicting Content-Length headers"));
                }
                content_length = Some(n);
            }
            "transfer-encoding" => {
                // Not implemented. Rejecting also blocks Content-Length + chunked smuggling.
                return Err(Reject(501, "Transfer-Encoding is not supported"));
            }
            "connection" => {
                for token in value.split(',') {
                    match token.trim().to_ascii_lowercase().as_str() {
                        "close" => close = true,
                        "keep-alive" => keep = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        headers.push((lname, value.to_string()));
    }

    let len = content_length.unwrap_or(0);
    if len > MAX_BODY {
        return Err(Reject(413, "the request body is too large"));
    }
    Ok(Head {
        method: method.to_string(),
        target: target.to_string(),
        headers,
        content_length: len,
        keep_alive: if http11 { !close } else { keep && !close },
    })
}

pub(crate) fn read_request<R: BufRead>(r: &mut R) -> Result<Request, ReadError> {
    let head = read_head(r)?;
    let len = head.content_length;
    // Read incrementally: a lying Content-Length must not allocate memory up front.
    let mut body = Vec::with_capacity(len.min(64 * 1024));
    let got = r
        .by_ref()
        .take(len as u64)
        .read_to_end(&mut body)
        .map_err(|_| ReadError::Closed)?;
    if got != len {
        return Err(ReadError::Closed); // the client disconnected mid-body
    }
    Ok(head.into_request(body))
}

pub(crate) fn status_text(code: i64) -> &'static str {
    match code {
        100 => "Continue",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        505 => "HTTP Version Not Supported",
        _ => match code / 100 {
            1 => "Informational",
            2 => "Success",
            3 => "Redirect",
            4 => "Client Error",
            _ => "Server Error",
        },
    }
}

/// A handler-controlled content type must not be able to inject header lines.
fn safe_content_type(raw: &[u8]) -> &str {
    match std::str::from_utf8(raw) {
        Ok(s) if !s.is_empty() && s.bytes().all(|b| (0x20..0x7f).contains(&b)) => s,
        _ => "text/plain; charset=utf-8",
    }
}

/// Write one response with a single `write_all` for small bodies (one packet).
pub(crate) fn write_response<W: Write>(
    out: &mut W,
    status: i64,
    content_type: &[u8],
    body: &[u8],
    keep_alive: bool,
    head_only: bool,
) -> io::Result<()> {
    // A handler can return any int; an invalid status line would break the client.
    let status = if (100..=599).contains(&status) {
        status
    } else {
        500
    };
    let no_body = status < 200 || status == 204 || status == 304;
    let mut head = Vec::with_capacity(160);
    write!(head, "HTTP/1.1 {status} {}\r\n", status_text(status))?;
    if !no_body {
        write!(
            head,
            "Content-Type: {}\r\n",
            safe_content_type(content_type)
        )?;
        write!(head, "Content-Length: {}\r\n", body.len())?;
    }
    write!(
        head,
        "Connection: {}\r\n\r\n",
        if keep_alive { "keep-alive" } else { "close" }
    )?;
    if no_body || head_only {
        return out.write_all(&head);
    }
    if body.len() <= 16 * 1024 {
        head.extend_from_slice(body);
        out.write_all(&head)
    } else {
        out.write_all(&head)?;
        out.write_all(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &[u8]) -> Result<Request, ReadError> {
        read_request(&mut BufReader::new(raw))
    }

    fn rejected(raw: &[u8]) -> u16 {
        match parse(raw) {
            Err(ReadError::Reject(code, _)) => code,
            Err(ReadError::Closed) => panic!("connection was closed instead of rejected"),
            Ok(_) => panic!("request was accepted"),
        }
    }

    #[test]
    fn basic_get_keeps_alive() {
        let req = parse(b"GET /a?b=1 HTTP/1.1\r\nHost: x\r\n\r\n")
            .ok()
            .expect("parse");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/a?b=1");
        assert!(req.keep_alive);
    }

    #[test]
    fn http10_closes_unless_keep_alive() {
        let a = parse(b"GET / HTTP/1.0\r\n\r\n").ok().expect("parse");
        assert!(!a.keep_alive);
        let b = parse(b"GET / HTTP/1.0\r\nConnection: keep-alive\r\n\r\n")
            .ok()
            .expect("parse");
        assert!(b.keep_alive);
    }

    #[test]
    fn body_is_read_exactly() {
        let req = parse(b"POST / HTTP/1.1\r\nContent-Length: 3\r\n\r\nabcGET")
            .ok()
            .expect("parse");
        assert_eq!(req.body, b"abc");
    }

    #[test]
    fn bad_content_length_is_rejected() {
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: abc\r\n\r\n"),
            400
        );
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: -1\r\n\r\n"),
            400
        );
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: 99999999999999999999999\r\n\r\n"),
            413
        );
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: 8388609\r\n\r\n"),
            413
        );
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\n"),
            400
        );
    }

    #[test]
    fn smuggling_shapes_are_rejected() {
        assert_eq!(
            rejected(b"POST / HTTP/1.1\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n"),
            501
        );
        assert_eq!(rejected(b"GET / HTTP/1.1\r\nBad Name: x\r\n\r\n"), 400);
        assert_eq!(rejected(b"GET / HTTP/1.1\r\nX: a\x00b\r\n\r\n"), 400);
    }

    #[test]
    fn malformed_request_lines_are_rejected() {
        assert_eq!(rejected(b"GET /\r\n\r\n"), 400);
        assert_eq!(rejected(b"GET / HTTP/2.0\r\n\r\n"), 505);
        assert_eq!(rejected(b"BREW / HTTP/1.1\r\n\r\n"), 501);
        assert_eq!(rejected(b"GET nothing HTTP/1.1\r\n\r\n"), 400);
    }

    #[test]
    fn oversized_lines_and_headers_are_rejected() {
        let mut long = b"GET /".to_vec();
        long.extend(std::iter::repeat_n(b'a', MAX_REQUEST_LINE + 10));
        long.extend_from_slice(b" HTTP/1.1\r\n\r\n");
        assert_eq!(rejected(&long), 414);

        let mut big = b"GET / HTTP/1.1\r\n".to_vec();
        for i in 0..=MAX_HEADERS {
            big.extend_from_slice(format!("X-{i}: v\r\n").as_bytes());
        }
        big.extend_from_slice(b"\r\n");
        assert_eq!(rejected(&big), 431);
    }

    #[test]
    fn truncated_body_closes() {
        assert!(matches!(
            parse(b"POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\nabc"),
            Err(ReadError::Closed)
        ));
    }

    #[test]
    fn two_requests_on_one_connection() {
        let mut r = BufReader::new(&b"GET /1 HTTP/1.1\r\n\r\nGET /2 HTTP/1.1\r\n\r\n"[..]);
        assert_eq!(read_request(&mut r).ok().expect("first").target, "/1");
        assert_eq!(read_request(&mut r).ok().expect("second").target, "/2");
    }

    #[test]
    fn response_head_cannot_be_injected() {
        let mut out = Vec::new();
        write_response(
            &mut out,
            99999,
            b"text/plain\r\nX-Evil: 1",
            b"hi",
            true,
            false,
        )
        .expect("write");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.starts_with("HTTP/1.1 500 "));
        assert!(!text.contains("X-Evil"));
        let mut empty = Vec::new();
        write_response(&mut empty, 204, b"", b"ignored", true, false).expect("write");
        assert!(!String::from_utf8(empty)
            .expect("utf8")
            .contains("Content-Length"));
    }
}

#[cfg(test)]
mod socket_tests {
    //! Real sockets, tiny timeouts. Assertions use generous upper bounds and never
    //! depend on two threads winning a race.
    use super::*;
    use std::time::Instant;

    fn start_server(workers: usize) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        for _ in 0..workers {
            let l = listener.try_clone().expect("clone");
            std::thread::spawn(move || {
                accept_loop(l, |s| {
                    serve_connection(s, |req| {
                        (
                            200,
                            format!("hi {} {}", req.target, req.body.len()).into_bytes(),
                            b"text/plain".to_vec(),
                        )
                    })
                })
            });
        }
        port
    }

    fn connect(port: u16) -> TcpStream {
        let s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        s.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("timeout");
        s
    }

    /// (status, body); `None` when the server closed the connection first.
    fn read_response(r: &mut BufReader<TcpStream>) -> Option<(u16, Vec<u8>)> {
        let mut status_line = String::new();
        if r.read_line(&mut status_line).ok()? == 0 {
            return None;
        }
        let status: u16 = status_line.split(' ').nth(1)?.parse().ok()?;
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            r.read_line(&mut h).ok()?;
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length:") {
                len = v.trim().parse().ok()?;
            }
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).ok()?;
        Some((status, body))
    }

    fn is_closed(r: &mut BufReader<TcpStream>) -> bool {
        let mut b = [0u8; 1];
        matches!(r.read(&mut b), Ok(0) | Err(_))
    }

    #[test]
    fn keep_alive_serves_many_requests_on_one_connection() {
        let port = start_server(2);
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        for i in 0..3 {
            write!(c, "GET /{i} HTTP/1.1\r\nHost: x\r\n\r\n").expect("write");
            let (status, body) = read_response(&mut r).expect("response");
            assert_eq!(status, 200);
            assert_eq!(body, format!("hi /{i} 0").into_bytes());
        }
    }

    #[test]
    fn connection_close_and_http10_close_the_socket() {
        let port = start_server(2);
        for raw in [
            "GET /a HTTP/1.1\r\nConnection: close\r\n\r\n",
            "GET /a HTTP/1.0\r\n\r\n",
        ] {
            let mut c = connect(port);
            let mut r = BufReader::new(c.try_clone().expect("clone"));
            c.write_all(raw.as_bytes()).expect("write");
            assert_eq!(read_response(&mut r).expect("response").0, 200);
            assert!(is_closed(&mut r), "server must close after: {raw:?}");
        }
    }

    #[test]
    fn head_has_headers_but_no_body() {
        let port = start_server(1);
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"HEAD /x HTTP/1.1\r\n\r\nGET /y HTTP/1.1\r\n\r\n")
            .expect("write");
        let mut line = String::new();
        r.read_line(&mut line).expect("status");
        assert!(line.starts_with("HTTP/1.1 200"));
        loop {
            line.clear();
            r.read_line(&mut line).expect("header");
            if line == "\r\n" {
                break;
            }
        }
        // The next bytes must be the second response, not a body for HEAD.
        let (status, body) = read_response(&mut r).expect("second response");
        assert_eq!(status, 200);
        assert_eq!(body, b"hi /y 0");
    }

    #[test]
    fn oversized_body_gets_413_and_close() {
        let port = start_server(1);
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"POST / HTTP/1.1\r\nContent-Length: 99999999\r\n\r\n")
            .expect("write");
        assert_eq!(read_response(&mut r).expect("response").0, 413);
        assert!(is_closed(&mut r));
    }

    #[test]
    fn malformed_request_gets_400() {
        let port = start_server(1);
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"this is not http\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("response").0, 400);
    }

    #[test]
    fn client_disconnect_mid_body_does_not_wedge_the_worker() {
        let port = start_server(1);
        {
            let mut c = connect(port);
            c.write_all(b"POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\nabc")
                .expect("write");
        } // dropped: the server sees EOF in the middle of the body
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /ok HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("served").0, 200);
    }

    #[test]
    fn idle_connections_release_their_workers() {
        // Two workers, both occupied by idle clients. A third client must still be
        // served once the idle timeout frees a worker, not wait forever.
        let port = start_server(2);
        let _idle1 = connect(port);
        let _idle2 = connect(port);
        let started = Instant::now();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /late HTTP/1.1\r\n\r\n").expect("write");
        let (status, _) = read_response(&mut r).expect("served after idle timeout");
        assert_eq!(status, 200);
        assert!(started.elapsed() < Duration::from_secs(4));
    }

    #[test]
    fn slow_headers_are_cut_off() {
        let port = start_server(1);
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        let started = Instant::now();
        c.write_all(b"GET / HTTP/1.1\r\nX-Slow: ").expect("write"); // never finished
        assert!(
            is_closed(&mut r),
            "the server must give up on a stalled request"
        );
        assert!(started.elapsed() < Duration::from_secs(4));
    }

    #[test]
    fn many_concurrent_clients_are_all_served() {
        let port = start_server(4);
        let handles: Vec<_> = (0..16)
            .map(|i| {
                std::thread::spawn(move || {
                    let mut c = connect(port);
                    let mut r = BufReader::new(c.try_clone().expect("clone"));
                    for j in 0..5 {
                        write!(c, "GET /{i}-{j} HTTP/1.1\r\n\r\n").expect("write");
                        let (status, body) = read_response(&mut r).expect("response");
                        assert_eq!(status, 200);
                        assert_eq!(body, format!("hi /{i}-{j} 0").into_bytes());
                    }
                    // Close politely so the worker is freed at once.
                    write!(c, "GET /bye HTTP/1.1\r\nConnection: close\r\n\r\n").expect("write");
                    read_response(&mut r).expect("last");
                })
            })
            .collect();
        for h in handles {
            h.join().expect("client thread");
        }
    }
}

#[cfg(test)]
mod worker_count_tests {
    use super::*;

    #[test]
    fn worker_count_uses_a_safe_default_for_io_bound_servers() {
        assert_eq!(worker_count_with(None, 1), 4);
        assert_eq!(worker_count_with(None, 2), 4);
        assert_eq!(worker_count_with(Some("3".to_string()), 1), 3);
        assert_eq!(worker_count_with(Some("0".to_string()), 8), 8);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod epoll_tests {
    //! The real `event_loop` over real sockets.
    use super::*;
    use std::time::Instant;

    fn start() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        std::thread::spawn(move || {
            event_loop(listener, |req| {
                if req.target == "/panic" {
                    panic!("handler exploded");
                }
                if req.target == "/large" {
                    return (200, vec![b'x'; 4 * 1024 * 1024], b"text/plain".to_vec());
                }
                (
                    200,
                    format!("hi {} {}", req.target, req.body.len()).into_bytes(),
                    b"text/plain".to_vec(),
                )
            })
        });
        port
    }

    fn connect(port: u16) -> TcpStream {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(s) => {
                    s.set_read_timeout(Some(Duration::from_secs(5)))
                        .expect("timeout");
                    return s;
                }
                Err(e) if Instant::now() > deadline => panic!("server did not start: {e}"),
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    fn read_response(r: &mut BufReader<TcpStream>) -> Option<(u16, Vec<u8>)> {
        let mut status_line = String::new();
        if r.read_line(&mut status_line).ok()? == 0 {
            return None;
        }
        let status: u16 = status_line.split(' ').nth(1)?.parse().ok()?;
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            r.read_line(&mut h).ok()?;
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length:") {
                len = v.trim().parse().ok()?;
            }
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).ok()?;
        Some((status, body))
    }

    fn is_closed(r: &mut BufReader<TcpStream>) -> bool {
        let mut b = [0u8; 1];
        matches!(r.read(&mut b), Ok(0) | Err(_))
    }

    #[test]
    fn keep_alive_and_pipelining() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        write!(c, "GET /a HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("a").1, b"hi /a 0");
        // Three requests in one packet are answered in order.
        c.write_all(b"GET /1 HTTP/1.1\r\n\r\nGET /2 HTTP/1.1\r\n\r\nGET /3 HTTP/1.1\r\n\r\n")
            .expect("write");
        for i in 1..=3 {
            let (status, body) = read_response(&mut r).expect("pipelined");
            assert_eq!(status, 200);
            assert_eq!(body, format!("hi /{i} 0").into_bytes());
        }
    }

    #[test]
    fn request_split_across_packets() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /sp").expect("write");
        std::thread::sleep(Duration::from_millis(50));
        c.write_all(b"lit HTTP/1.1\r\nHo").expect("write");
        std::thread::sleep(Duration::from_millis(50));
        c.write_all(b"st: x\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("split").1, b"hi /split 0");
    }

    #[test]
    fn body_arriving_in_chunks() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"POST /big HTTP/1.1\r\nContent-Length: 100000\r\n\r\n")
            .expect("write");
        for _ in 0..10 {
            c.write_all(&[b'z'; 10_000]).expect("write");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(read_response(&mut r).expect("big").1, b"hi /big 100000");
        // The connection is still usable afterwards.
        c.write_all(b"GET /after HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("after").1, b"hi /after 0");
    }

    #[test]
    fn head_has_no_body_and_close_is_honoured() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"HEAD /x HTTP/1.1\r\n\r\nGET /y HTTP/1.1\r\nConnection: close\r\n\r\n")
            .expect("write");
        let mut line = String::new();
        r.read_line(&mut line).expect("status");
        assert!(line.starts_with("HTTP/1.1 200"));
        loop {
            line.clear();
            r.read_line(&mut line).expect("header");
            if line == "\r\n" {
                break;
            }
        }
        let (status, body) = read_response(&mut r).expect("second");
        assert_eq!(status, 200);
        assert_eq!(body, b"hi /y 0");
        assert!(is_closed(&mut r));
    }

    #[test]
    fn bad_requests_are_rejected_then_closed() {
        let port = start();
        for (raw, code) in [
            (&b"this is not http\r\n\r\n"[..], 400),
            (&b"POST / HTTP/1.1\r\nContent-Length: 99999999\r\n\r\n"[..], 413),
            (&b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"[..], 501),
        ] {
            let mut c = connect(port);
            let mut r = BufReader::new(c.try_clone().expect("clone"));
            c.write_all(raw).expect("write");
            assert_eq!(read_response(&mut r).expect("response").0, code);
            assert!(is_closed(&mut r));
        }
    }

    #[test]
    fn endless_request_line_is_rejected() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        let junk = vec![b'a'; MAX_REQUEST_LINE + 100];
        c.write_all(b"GET /").expect("write");
        c.write_all(&junk).expect("write");
        assert_eq!(read_response(&mut r).expect("response").0, 414);
    }

    #[test]
    fn idle_connections_do_not_block_others() {
        let port = start();
        let _idle: Vec<_> = (0..200).map(|_| connect(port)).collect();
        let started = Instant::now();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /late HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("served").0, 200);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn client_disconnect_mid_body_is_harmless() {
        let port = start();
        {
            let mut c = connect(port);
            c.write_all(b"POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\nabc")
                .expect("write");
        }
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /ok HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("served").0, 200);
    }

    #[test]
    fn handler_panic_costs_one_connection_not_the_server() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /panic HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("500").0, 500);
        assert!(is_closed(&mut r));
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /ok HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("served").0, 200);
    }

    #[test]
    fn slow_reader_gets_a_large_response_intact() {
        let port = start();
        let mut c = connect(port);
        let mut r = BufReader::new(c.try_clone().expect("clone"));
        c.write_all(b"GET /large HTTP/1.1\r\n\r\n").expect("write");
        // Do not read for a while: the server must park the rest and keep going.
        let mut other = connect(port);
        let mut other_r = BufReader::new(other.try_clone().expect("clone"));
        other.write_all(b"GET /fast HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut other_r).expect("fast").1, b"hi /fast 0");
        std::thread::sleep(Duration::from_millis(300));
        let (status, body) = read_response(&mut r).expect("large");
        assert_eq!(status, 200);
        assert_eq!(body.len(), 4 * 1024 * 1024);
        assert!(body.iter().all(|&b| b == b'x'));
        // And the same connection keeps working after the flush.
        c.write_all(b"GET /next HTTP/1.1\r\n\r\n").expect("write");
        assert_eq!(read_response(&mut r).expect("next").1, b"hi /next 0");
    }

    #[test]
    fn many_concurrent_clients_are_all_served() {
        let port = start();
        let handles: Vec<_> = (0..64)
            .map(|i| {
                std::thread::spawn(move || {
                    let mut c = connect(port);
                    let mut r = BufReader::new(c.try_clone().expect("clone"));
                    for j in 0..20 {
                        write!(c, "GET /{i}-{j} HTTP/1.1\r\n\r\n").expect("write");
                        let (status, body) = read_response(&mut r).expect("response");
                        assert_eq!(status, 200);
                        assert_eq!(body, format!("hi /{i}-{j} 0").into_bytes());
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("client thread");
        }
    }
}
//! HTTP/1.x wire handling shared by `http.rs` and `router.rs`.
//!
//! Limits, strict request parsing and response writing live here so both servers
//! behave the same. Unsupported features (chunked request bodies, `Expect`) are
//! rejected explicitly instead of being half-handled.

use std::collections::HashMap;
use std::io::{self, BufRead, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};
use crate::error::fatal;

#[cfg(test)]
use std::io::BufReader;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::time::Duration;

#[cfg(unix)]
mod sys {
    use std::os::fd::RawFd;

    #[repr(C)]
    pub struct pollfd {
        pub fd: RawFd,
        pub events: i16,
        pub revents: i16,
    }

    extern "C" {
        pub fn poll(fds: *mut pollfd, nfds: usize, timeout: i32) -> i32;
    }

    pub const POLLIN: i16 = 0x001;
    pub const POLLOUT: i16 = 0x004;
    pub const POLLERR: i16 = 0x008;
    pub const POLLHUP: i16 = 0x010;
    pub const POLLNVAL: i16 = 0x020;
}

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
    let queue = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    let cv = Arc::new(std::sync::Condvar::new());

    for _ in 0..workers {
        let h = Arc::clone(&handle);
        let q = Arc::clone(&queue);
        let c = Arc::clone(&cv);
        std::thread::spawn(move || loop {
            let stream = {
                let mut guard = q.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if let Some(s) = guard.pop_front() {
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
                cv.notify_one();
            }
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

struct ActiveConn {
    stream: TcpStream,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    requests_served: usize,
    close_after_write: bool,
}

impl ActiveConn {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_buf: Vec::new(),
            write_buf: Vec::new(),
            requests_served: 0,
            close_after_write: false,
        }
    }
}

pub(crate) fn event_loop<F>(listener: TcpListener, mut dispatch: F)
where
    F: FnMut(&Request) -> (i64, Vec<u8>, Vec<u8>) + Send + Sync + 'static,
{
    let _ = listener.set_nonblocking(true);
    let mut clients: HashMap<RawFd, ActiveConn> = HashMap::new();
    let listener_fd = listener.as_raw_fd();

    loop {
        let mut fds = Vec::with_capacity(clients.len() + 1);
        fds.push(sys::pollfd {
            fd: listener_fd,
            events: sys::POLLIN,
            revents: 0,
        });

        for (&fd, client) in &clients {
            let mut events = sys::POLLIN;
            if !client.write_buf.is_empty() {
                events |= sys::POLLOUT;
            }
            fds.push(sys::pollfd {
                fd,
                events,
                revents: 0,
            });
        }

        let ready = unsafe { sys::poll(fds.as_mut_ptr(), fds.len(), 50) };
        if ready <= 0 {
            continue;
        }

        if fds[0].revents & sys::POLLIN != 0 {
            loop {
                if clients.len() >= MAX_ACTIVE_CONNECTIONS {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                        Err(_) => break,
                    }
                    continue;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nonblocking(true);
                        let fd = stream.as_raw_fd();
                        clients.insert(fd, ActiveConn::new(stream));
                    }
                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }

        let mut closed = Vec::new();
        for client_fd in fds.iter().skip(1) {
            let Some(client) = clients.get_mut(&client_fd.fd) else {
                continue;
            };
            let revents = client_fd.revents;
            if revents & (sys::POLLERR | sys::POLLHUP | sys::POLLNVAL) != 0 {
                closed.push(client_fd.fd);
                continue;
            }

            if revents & sys::POLLIN != 0 {
                let mut chunk = [0u8; 8 * 1024];
                loop {
                    match client.stream.read(&mut chunk) {
                        Ok(0) => {
                            closed.push(client_fd.fd);
                            break;
                        }
                        Ok(n) => {
                            client.read_buf.extend_from_slice(&chunk[..n]);
                            loop {
                                let mut cursor = Cursor::new(client.read_buf.clone());
                                match read_request(&mut cursor) {
                                    Ok(req) => {
                                        let consumed = cursor.position() as usize;
                                        if consumed == 0 {
                                            break;
                                        }
                                        let keep_alive = req.keep_alive && client.requests_served + 1 < MAX_REQUESTS_PER_CONNECTION;
                                        client.requests_served += 1;
                                        let (status, body, ctype) = dispatch(&req);
                                        let mut response = Vec::new();
                                        let _ = write_response(
                                            &mut response,
                                            status,
                                            &ctype,
                                            &body,
                                            keep_alive,
                                            req.method == "HEAD",
                                        );
                                        client.write_buf.extend_from_slice(&response);
                                        if !keep_alive {
                                            client.close_after_write = true;
                                        }
                                        client.read_buf.drain(..consumed);
                                        if client.close_after_write {
                                            break;
                                        }
                                    }
                                    Err(ReadError::Closed) => break,
                                    Err(ReadError::Reject(code, msg)) => {
                                        let mut response = Vec::new();
                                        let _ = write_response(
                                            &mut response,
                                            i64::from(code),
                                            b"text/plain; charset=utf-8",
                                            msg.as_bytes(),
                                            false,
                                            false,
                                        );
                                        client.write_buf.extend_from_slice(&response);
                                        client.close_after_write = true;
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                        Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => {
                            closed.push(client_fd.fd);
                            break;
                        }
                    }
                }
            }

            if revents & sys::POLLOUT != 0 && !client.write_buf.is_empty() {
                let before = client.write_buf.len();
                let written = client.stream.write(&client.write_buf).unwrap_or(0);
                if written > 0 {
                    client.write_buf.drain(..written);
                }
                if written == 0 || client.write_buf.len() == before {
                    closed.push(client_fd.fd);
                }
            }

            if client.close_after_write && client.write_buf.is_empty() {
                closed.push(client_fd.fd);
            }
        }

        for fd in closed {
            clients.remove(&fd);
        }
    }
}

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

pub(crate) fn read_request<R: BufRead>(r: &mut R) -> Result<Request, ReadError> {
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
    // Read incrementally: a lying Content-Length must not allocate memory up front.
    let mut body = Vec::with_capacity(len.min(64 * 1024));
    let got = r
        .by_ref()
        .take(len as u64)
        .read_to_end(&mut body)
        .map_err(|_| Closed)?;
    if got != len {
        return Err(Closed); // the client disconnected mid-body
    }

    Ok(Request {
        method: method.to_string(),
        target: target.to_string(),
        headers,
        body,
        keep_alive: if http11 { !close } else { keep && !close },
    })
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

#[cfg(test)]
pub(crate) fn serve_connection(
    stream: TcpStream,
    mut dispatch: impl FnMut(&Request) -> (i64, Vec<u8>, Vec<u8>),
) {
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

#[cfg(test)]
const IDLE_TIMEOUT: Duration = Duration::from_millis(300);
#[cfg(test)]
const READ_TIMEOUT: Duration = Duration::from_millis(500);
#[cfg(test)]
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

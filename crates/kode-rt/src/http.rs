//! A small HTTP/1.1 server for `serve(port, handler)`.
//! The handler is a Kode function `(method, path, body) -> string`. A handler that
//! throws an `Error` answers with the error's `code` (400 to 599) or 500.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::Ordering;

use crate::error::{fatal, take_err};
use crate::object::{kode_release, Header, THREADED};
use crate::string::{bytes, from_bytes};

type Handler = extern "C" fn(*mut Header, *mut Header, *mut Header) -> *mut Header;

pub(crate) fn status_text(code: i64) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        _ if code >= 500 => "Internal Server Error",
        _ => "Status",
    }
}

fn handle_connection(stream: TcpStream, handler: usize) {
    let _ = stream.set_nodelay(true);
    let Ok(read_half) = stream.try_clone() else { return };
    let mut reader = BufReader::new(read_half);
    let mut out = stream;
    let handler: Handler = unsafe { std::mem::transmute(handler) };

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let mut parts = line.split_whitespace();
        let (Some(method), Some(path)) = (parts.next(), parts.next()) else { return };
        let (method, path) = (method.to_string(), path.to_string());

        let mut content_length = 0usize;
        let mut close = false;
        loop {
            let mut h = String::new();
            if reader.read_line(&mut h).unwrap_or(0) == 0 {
                return;
            }
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            let lower = h.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            } else if lower.starts_with("connection:") && lower.contains("close") {
                close = true;
            }
        }
        let mut body = vec![0u8; content_length.min(64 << 20)];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        let (status, text) = unsafe {
            let m = from_bytes(method.as_bytes());
            let p = from_bytes(path.as_bytes());
            let b = from_bytes(&body);
            let r = handler(m, p, b);
            kode_release(m);
            kode_release(p);
            kode_release(b);
            let err = take_err();
            if !err.is_null() {
                kode_release(r);
                let msg_ptr = *((err as *const *const Header).add(3));
                let code = *((err as *const i64).add(4));
                let msg = if msg_ptr.is_null() { Vec::new() } else { bytes(msg_ptr).to_vec() };
                kode_release(err);
                (if (400..600).contains(&code) { code } else { 500 }, msg)
            } else {
                let t = if r.is_null() { Vec::new() } else { bytes(r).to_vec() };
                kode_release(r);
                (200, t)
            }
        };

        let trimmed = text.iter().find(|c| !c.is_ascii_whitespace());
        let ctype = if matches!(trimmed, Some(b'{') | Some(b'[')) {
            "application/json"
        } else {
            "text/plain; charset=utf-8"
        };
        let head = format!(
            "HTTP/1.1 {status} {}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
            status_text(status),
            text.len(),
            if close { "close" } else { "keep-alive" }
        );
        if out.write_all(head.as_bytes()).is_err() || out.write_all(&text).is_err() {
            return;
        }
        if close {
            return;
        }
    }
}

/// Run a web server on `port` until the program is stopped.
///
/// # Safety
/// `handler` must be a compiled Kode function with the handler signature.
#[no_mangle]
pub unsafe extern "C" fn kode_http_serve(port: i64, handler: Handler) {
    THREADED.store(true, Ordering::SeqCst);
    let listener = TcpListener::bind(("0.0.0.0", port as u16))
        .unwrap_or_else(|e| fatal(&format!("cannot listen on port {port}: {e}")));
    eprintln!("kode: listening on http://localhost:{port}");
    let workers = std::env::var("KODE_HTTP_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).max(4));
    let h = handler as usize;
    let mut threads = Vec::new();
    for _ in 0..workers {
        let l = listener.try_clone().unwrap_or_else(|e| fatal(&format!("cannot share the socket: {e}")));
        threads.push(std::thread::spawn(move || loop {
            if let Ok((stream, _)) = l.accept() {
                handle_connection(stream, h);
            }
        }));
    }
    for t in threads {
        let _ = t.join();
    }
}
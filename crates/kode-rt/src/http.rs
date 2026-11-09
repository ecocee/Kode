//! A small HTTP/1.1 server for `serve(port, handler)`.
//! The handler is a Kode function `(method, path, body) -> string`. A handler that
//! throws an `Error` answers with the error's `code` (400 to 599) or 500.
//!
//! Parsing, limits, timeouts and response writing are in `http_wire.rs`

use std::net::TcpListener;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::error::{fatal, take_err};
use crate::http_wire::{checked_port, event_loop, tls_event_loop, Request};
use crate::object::{kode_release, Header, THREADED};
use crate::string::{bytes, from_bytes, from_external};

// `router.rs` and tests import it from here.
#[allow(unused_imports)]
pub(crate) use crate::http_wire::status_text;

type Handler = extern "C" fn(*mut Header, *mut Header, *mut Header) -> *mut Header;

fn call_handler(handler: Handler, req: &Request) -> (i64, Vec<u8>, Vec<u8>) {
    unsafe {
        // Isolation: a request never inherits an error left on this worker thread.
        kode_release(take_err());
        let m = from_bytes(req.method.as_bytes());
        let p = from_bytes(req.target.as_bytes());
        let b = from_external(&req.body);
        let r = handler(m, p, b);
        kode_release(m);
        kode_release(p);
        kode_release(b);
        let err = take_err();
        let (status, text) = if !err.is_null() {
            kode_release(r);
            let msg_ptr = *((err as *const *const Header).add(3));
            let code = *((err as *const i64).add(4));
            let msg = if msg_ptr.is_null() {
                Vec::new()
            } else {
                bytes(msg_ptr).to_vec()
            };
            kode_release(err);
            (
                if (400..600).contains(&code) {
                    code
                } else {
                    500
                },
                msg,
            )
        } else {
            let t = if r.is_null() {
                Vec::new()
            } else {
                bytes(r).to_vec()
            };
            kode_release(r);
            (200, t)
        };
        let first = text.iter().find(|c| !c.is_ascii_whitespace());
        let ctype: &[u8] = if matches!(first, Some(b'{') | Some(b'[')) {
            b"application/json"
        } else {
            b"text/plain; charset=utf-8"
        };
        (status, text, ctype.to_vec())
    }
}

/// Run a web server on `port` until the program is stopped.
///
/// # Safety
/// `handler` must be a compiled Kode function with the handler signature.
#[no_mangle]
pub unsafe extern "C" fn kode_http_serve(port: i64, handler: Handler) {
    THREADED.store(true, Ordering::SeqCst);
    let port = checked_port(port);
    let listener = TcpListener::bind(("0.0.0.0", port))
        .unwrap_or_else(|e| fatal(&format!("cannot listen on port {port}: {e}")));
    eprintln!("kode: listening on http://localhost:{port}");
    let h = handler as usize;
    event_loop(listener, move |req| {
        let handler: Handler = unsafe { std::mem::transmute::<usize, Handler>(h) };
        call_handler(handler, req)
    });
}

/// Read a TLS file path passed to `serve_tls` from a Kode string.
pub(crate) unsafe fn tls_path(path: *mut Header, what: &str) -> String {
    match std::str::from_utf8(bytes(path)) {
        Ok(text) => text.to_string(),
        Err(_) => fatal(&format!("the {what} path is not valid UTF-8")),
    }
}

/// Build the TLS server configuration from DER files.
///
/// DER rather than PEM keeps the runtime free of a PEM parser; the error says how
/// to convert. See ADR 0034.
pub(crate) fn server_config(cert_path: &str, key_path: &str) -> Arc<rustls::ServerConfig> {
    let read = |path: &str, what: &str| {
        std::fs::read(path)
            .unwrap_or_else(|e| fatal(&format!("cannot read the {what} `{path}`: {e}")))
    };
    let certs = vec![rustls::pki_types::CertificateDer::from(read(
        cert_path,
        "certificate",
    ))];
    let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(read(key_path, "private key")),
    );
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap_or_else(|e| {
            fatal(&format!(
                "`{cert_path}` is not a usable DER certificate for the key `{key_path}`: {e}; \
                 convert PEM with `openssl x509 -outform der` and \
                 `openssl pkcs8 -topk8 -nocrypt -outform der`"
            ))
        });
    Arc::new(config)
}

/// Bind a TLS listener and print the scheme it serves.
pub(crate) fn tls_listener(
    port: i64,
    cert_path: &str,
    key_path: &str,
) -> (TcpListener, Arc<rustls::ServerConfig>) {
    let port = checked_port(port);
    let config = server_config(cert_path, key_path);
    let listener = TcpListener::bind(("0.0.0.0", port))
        .unwrap_or_else(|e| fatal(&format!("cannot listen on port {port}: {e}")));
    eprintln!("kode: listening on https://localhost:{port}");
    (listener, config)
}

/// Run an HTTPS server with a plain handler function.
///
/// # Safety
/// `cert` and `key` must be string paths to DER files, and `handler` must be a
/// compiled Kode function with the handler signature.
#[no_mangle]
pub unsafe extern "C" fn kode_http_serve_tls(
    port: i64,
    cert: *mut Header,
    key: *mut Header,
    handler: Handler,
) {
    let cert = tls_path(cert, "certificate");
    let key = tls_path(key, "private key");
    THREADED.store(true, Ordering::SeqCst);
    let (listener, config) = tls_listener(port, &cert, &key);
    let h = handler as usize;
    tls_event_loop(listener, config, move |req| {
        let handler: Handler = unsafe { std::mem::transmute::<usize, Handler>(h) };
        call_handler(handler, req)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{kode_err_throw, make_error};
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::{Duration, Instant};

    extern "C" fn handler(_m: *mut Header, p: *mut Header, b: *mut Header) -> *mut Header {
        unsafe {
            match bytes(p) {
                b"/json" => from_bytes(br#"{"ok":true}"#),
                b"/err" => {
                    kode_err_throw(make_error("teapot", 418));
                    from_bytes(b"")
                }
                b"/status" => from_bytes(b"plain"),
                _ => from_bytes(bytes(b)), // echo the request body
            }
        }
    }

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind")
            .local_addr()
            .expect("addr")
            .port()
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
                Err(_) => std::thread::sleep(Duration::from_millis(20)),
            }
        }
    }

    /// Send a request and read one response (head text, body bytes).
    fn exchange(c: &mut TcpStream, raw: &[u8]) -> (String, Vec<u8>) {
        c.write_all(raw).expect("write");
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            c.read_exact(&mut byte).expect("head");
            head.push(byte[0]);
        }
        let head = String::from_utf8(head).expect("utf8 head");
        let len = head
            .lines()
            .find_map(|l| {
                l.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(|v| v.trim().parse::<usize>().expect("len"))
            })
            .expect("Content-Length is always sent");
        let mut body = vec![0u8; len];
        c.read_exact(&mut body).expect("body");
        (head, body)
    }

    #[test]
    fn serve_end_to_end() {
        let port = free_port();
        std::thread::spawn(move || unsafe { kode_http_serve(i64::from(port), handler) });
        let mut c = connect(port);

        let (head, body) = exchange(&mut c, b"GET /json HTTP/1.1\r\n\r\n");
        assert!(head.starts_with("HTTP/1.1 200 OK"));
        assert!(head.contains("Content-Type: application/json"));
        assert_eq!(body, br#"{"ok":true}"#);

        // A thrown error becomes its status code, and the NEXT request on the same
        // connection must not inherit it.
        let (head, body) = exchange(&mut c, b"GET /err HTTP/1.1\r\n\r\n");
        assert!(head.starts_with("HTTP/1.1 418 "));
        assert_eq!(body, b"teapot");
        let (head, body) = exchange(&mut c, b"GET /status HTTP/1.1\r\n\r\n");
        assert!(head.starts_with("HTTP/1.1 200 OK"));
        assert_eq!(body, b"plain");

        // Invalid UTF-8 from the client must come back as valid UTF-8 text.
        let mut post = b"POST /echo HTTP/1.1\r\nContent-Length: 4\r\n\r\n".to_vec();
        post.extend_from_slice(&[b'a', 0xff, 0xfe, b'z']);
        let (head, body) = exchange(&mut c, &post);
        assert!(head.starts_with("HTTP/1.1 200 OK"));
        let text = String::from_utf8(body).expect("the echoed body is valid UTF-8");
        assert!(text.starts_with('a') && text.ends_with('z') && text.contains('\u{fffd}'));

        // Unsupported framing is refused, not guessed at.
        let mut c2 = connect(port);
        let (head, _) = exchange(
            &mut c2,
            b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n",
        );
        assert!(head.starts_with("HTTP/1.1 501 "));
    }
}

//! The Express/FastAPI-style web API layer: `Router()`, `router.get(path, handler)`
//! and `serve(port, router)`.
//!
//! A route stores the handler's function pointer, the HTTP method and the path
//! pattern. Handlers are Kode functions `fn handler(req: Request) -> Response`.
//! `Request` and `Response` are the built-in classes declared in
//! `kode-sema/src/check/mod.rs`; the field order below must match that file:
//!
//! ```text
//! Request:  0 method  1 path  2 params  3 query  4 headers  5 body
//! Response: 0 status  1 body  2 content_type
//! Router:   0 id
//! ```
//!
//! The registry copies every pattern and method into Rust strings, so it never
//! holds a Kode object alive.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::error::{fatal, take_err, throw_msg};
use crate::http::status_text;
use crate::map::{kode_map_new, kode_map_set};
use crate::object::{kode_obj_new, kode_release, kode_retain, Header, THREADED};
use crate::string::{bytes, from_bytes};

type Handler = extern "C" fn(*mut Header) -> *mut Header;

const REQUEST_FIELDS: i64 = 6;
const REQUEST_MASK: i64 = 0b111111;
const RESPONSE_FIELDS: i64 = 3;
const RESPONSE_MASK: i64 = 0b110;

enum Seg {
    Lit(String),
    Param(String),
    Rest(String),
}

struct Route {
    router: i64,
    method: String,
    segs: Vec<Seg>,
    handler: usize,
}

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();
static NEXT_ROUTER: AtomicI64 = AtomicI64::new(1);

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

/// # Safety
/// `router` must be a valid `Router`, and the strings must be valid Kode strings.
#[no_mangle]
pub unsafe extern "C" fn kode_router_add(router: *mut Header, method: *mut Header, pattern: *mut Header, handler: usize) {
    let id = *((router as *const i64).add(3));
    let method = String::from_utf8_lossy(bytes(method)).to_ascii_uppercase();
    let pattern = String::from_utf8_lossy(bytes(pattern)).to_string();
    let segs = pattern_segments(&pattern);
    routes().lock().unwrap_or_else(|e| e.into_inner()).push(Route {
        router: id,
        method,
        segs,
        handler,
    });
}

#[no_mangle]
pub extern "C" fn kode_router_new() -> i64 {
    NEXT_ROUTER.fetch_add(1, Ordering::Relaxed)
}

fn split_path(path: &str) -> Vec<&str> {
    let rest = path.strip_prefix('/').unwrap_or(path);
    if rest.is_empty() {
        Vec::new()
    } else {
        rest.split('/').collect()
    }
}

fn pattern_segments(pattern: &str) -> Vec<Seg> {
    split_path(pattern)
        .into_iter()
        .map(|s| {
            if let Some(name) = s.strip_prefix(':') {
                Seg::Param(name.to_string())
            } else if let Some(name) = s.strip_prefix('*') {
                Seg::Rest(name.to_string())
            } else {
                Seg::Lit(s.to_string())
            }
        })
        .collect()
}

fn match_segments(pattern: &[Seg], path: &[&str]) -> Option<Vec<(String, String)>> {
    let mut params = Vec::new();
    let mut i = 0;
    for seg in pattern {
        match seg {
            Seg::Lit(text) => {
                if i >= path.len() || path[i] != text {
                    return None;
                }
                i += 1;
            }
            Seg::Param(name) => {
                if i >= path.len() {
                    return None;
                }
                params.push((name.clone(), path[i].to_string()));
                i += 1;
            }
            Seg::Rest(name) => {
                params.push((name.clone(), path[i..].join("/")));
                i = path.len();
            }
        }
    }
    (i == path.len()).then_some(params)
}

/// Find a route for `method`/`path`. The second value is `true` when the path
/// exists for another method (the caller answers 405 instead of 404).
fn lookup(router: i64, method: &str, path: &str) -> (Option<usize>, Vec<(String, String)>, bool) {
    let segs = split_path(path);
    let list = routes().lock().unwrap_or_else(|e| e.into_inner());
    let mut path_matched = false;
    for route in list.iter().filter(|r| r.router == router) {
        if let Some(params) = match_segments(&route.segs, &segs) {
            if route.method == method {
                return (Some(route.handler), params, true);
            }
            path_matched = true;
        }
    }
    (None, Vec::new(), path_matched)
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    if query.is_empty() {
        return Vec::new();
    }
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (k.to_string(), v.to_string()),
            None => (pair.to_string(), String::new()),
        })
        .collect()
}

unsafe fn put(map: *mut Header, key: &str, value: &str) {
    let k = from_bytes(key.as_bytes());
    let v = from_bytes(value.as_bytes());
    kode_map_set(map, k, v as i64);
    kode_release(k);
}

unsafe fn set_field(obj: *mut Header, index: usize, value: *mut Header) {
    *((obj as *mut *mut Header).add(3 + index)) = value;
}

unsafe fn build_request(
    method: &str,
    path: &str,
    params: Vec<(String, String)>,
    query: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> *mut Header {
    let req = kode_obj_new(REQUEST_FIELDS, REQUEST_MASK);
    set_field(req, 0, from_bytes(method.as_bytes()));
    set_field(req, 1, from_bytes(path.as_bytes()));

    let params_map = kode_map_new(1);
    for (k, v) in params {
        put(params_map, &k, &v);
    }
    set_field(req, 2, params_map);

    let query_map = kode_map_new(1);
    for (k, v) in parse_query(query) {
        put(query_map, &k, &v);
    }
    set_field(req, 3, query_map);

    let header_map = kode_map_new(1);
    for (k, v) in headers {
        put(header_map, k, v);
    }
    set_field(req, 4, header_map);

    set_field(req, 5, from_bytes(body));
    req
}

unsafe fn read_response(resp: *mut Header) -> (i64, Vec<u8>, Vec<u8>) {
    let status = *((resp as *const i64).add(3));
    let body = *((resp as *const *mut Header).add(4));
    let ctype = *((resp as *const *mut Header).add(5));
    let body = if body.is_null() { Vec::new() } else { bytes(body).to_vec() };
    let ctype = if ctype.is_null() { Vec::new() } else { bytes(ctype).to_vec() };
    (status, body, ctype)
}

unsafe fn read_error(err: *mut Header) -> (Vec<u8>, i64) {
    let message = *((err as *const *mut Header).add(3));
    let code = *((err as *const i64).add(4));
    (if message.is_null() { Vec::new() } else { bytes(message).to_vec() }, code)
}

fn content_type_for(body: &[u8]) -> Vec<u8> {
    match body.iter().find(|b| !b.is_ascii_whitespace()) {
        Some(b'{') | Some(b'[') => b"application/json".to_vec(),
        _ => b"text/plain; charset=utf-8".to_vec(),
    }
}

const NOT_FOUND: &[u8] = b"not found";
const METHOD_NOT_ALLOWED: &[u8] = b"method not allowed";

fn dispatch(router: i64, method: &str, target: &str, headers: &[(String, String)], body: &[u8]) -> (i64, Vec<u8>, Vec<u8>) {
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (target, ""),
    };
    let (handler, params, path_matched) = lookup(router, method, path);
    let Some(handler) = handler else {
        return if path_matched {
            (405, METHOD_NOT_ALLOWED.to_vec(), b"text/plain; charset=utf-8".to_vec())
        } else {
            (404, NOT_FOUND.to_vec(), b"text/plain; charset=utf-8".to_vec())
        };
    };

    let handler: Handler = unsafe { std::mem::transmute(handler) };
    let req = unsafe { build_request(method, path, params, query, headers, body) };
    let resp = handler(req);
    let err = take_err();
    if !err.is_null() {
        let (message, code) = unsafe { read_error(err) };
        unsafe {
            kode_release(req);
            kode_release(err);
        }
        let status = if (400..600).contains(&code) { code } else { 500 };
        return (status, message, b"text/plain; charset=utf-8".to_vec());
    }
    if resp.is_null() {
        unsafe { kode_release(req) };
        return (500, b"the handler returned no response".to_vec(), b"text/plain; charset=utf-8".to_vec());
    }

    let (status, body, ctype) = unsafe { read_response(resp) };
    unsafe {
        kode_release(resp);
        kode_release(req);
    }
    let ctype = if ctype.is_empty() { content_type_for(&body) } else { ctype };
    (status, body, ctype)
}

fn handle_connection(stream: TcpStream, router: i64) {
    let _ = stream.set_nodelay(true);
    let Ok(read_half) = stream.try_clone() else { return };
    let mut reader = BufReader::new(read_half);
    let mut out = stream;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let mut parts = line.split_whitespace();
        let (Some(method), Some(target)) = (parts.next(), parts.next()) else { return };
        let (method, target) = (method.to_ascii_uppercase(), target.to_string());

        let mut content_length = 0usize;
        let mut close = false;
        let mut headers = Vec::new();
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).unwrap_or(0) == 0 {
                return;
            }
            let header = header.trim_end_matches(|c| c == '\r' || c == '\n');
            if header.is_empty() {
                break;
            }
            if let Some((name, value)) = header.split_once(':') {
                let name = name.trim().to_ascii_lowercase();
                let value = value.trim().to_string();
                if name == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                }
                if name == "connection" && value.to_ascii_lowercase().contains("close") {
                    close = true;
                }
                headers.push((name, value));
            }
        }
        let mut body = vec![0u8; content_length.min(64 << 20)];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        let (status, body, ctype) = dispatch(router, &method, &target, &headers, &body);
        let head = format!(
            "HTTP/1.1 {status} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
            status_text(status),
            String::from_utf8_lossy(&ctype),
            body.len(),
            if close { "close" } else { "keep-alive" }
        );
        if out.write_all(head.as_bytes()).is_err() || out.write_all(&body).is_err() {
            return;
        }
        if close {
            return;
        }
    }
}

/// Run a router until the program is stopped.
///
/// # Safety
/// `router` must be a valid `Router` object.
#[no_mangle]
pub unsafe extern "C" fn kode_http_serve_router(port: i64, router: *mut Header) {
    let id = *((router as *const i64).add(3));
    THREADED.store(true, Ordering::SeqCst);
    let listener = TcpListener::bind(("0.0.0.0", port as u16))
        .unwrap_or_else(|e| fatal(&format!("cannot listen on port {port}: {e}")));
    eprintln!("kode: listening on http://localhost:{port}");
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).max(4);
    let mut threads = Vec::new();
    for _ in 0..workers {
        let l = listener.try_clone().unwrap_or_else(|e| fatal(&format!("cannot share the socket: {e}")));
        threads.push(std::thread::spawn(move || loop {
            if let Ok((stream, _)) = l.accept() {
                handle_connection(stream, id);
            }
        }));
    }
    for t in threads {
        let _ = t.join();
    }
}

/// `req.json()`: parse the request body, raising a catchable error on bad JSON.
///
/// # Safety
/// `req` must be a valid `Request`.
#[no_mangle]
pub unsafe extern "C" fn kode_request_json(req: *mut Header) -> *mut Header {
    let body = *((req as *const *mut Header).add(8));
    let text = match std::str::from_utf8(bytes(body)) {
        Ok(text) => text,
        Err(_) => {
            throw_msg("the request body is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };
    match crate::json_parse::parse(text) {
        Ok(value) => value,
        Err(msg) => {
            throw_msg(&format!("the request body is not valid JSON: {msg}"));
            std::ptr::null_mut()
        }
    }
}

/// `json_obj()`: an empty `{string: Json}` for building an object key by key.
#[no_mangle]
pub extern "C" fn kode_json_obj() -> *mut Header {
    kode_map_new(1)
}

/// `response(status, body, content_type)`.
///
/// # Safety
/// `body` and `ctype` must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_response_new(status: i64, body: *mut Header, ctype: *mut Header) -> *mut Header {
    let p = kode_obj_new(RESPONSE_FIELDS, RESPONSE_MASK);
    *((p as *mut i64).add(3)) = status;
    kode_retain(body);
    *((p as *mut *mut Header).add(4)) = body;
    kode_retain(ctype);
    *((p as *mut *mut Header).add(5)) = ctype;
    p
}

/// `Response.text(text)`: 200 with a plain text content type.
///
/// # Safety
/// `body` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_response_text(body: *mut Header) -> *mut Header {
    let ctype = from_bytes(b"text/plain; charset=utf-8");
    let p = kode_obj_new(RESPONSE_FIELDS, RESPONSE_MASK);
    *((p as *mut i64).add(3)) = 200;
    kode_retain(body);
    *((p as *mut *mut Header).add(4)) = body;
    *((p as *mut *mut Header).add(5)) = ctype;
    p
}

/// `Response.json(value)`: 200 with the encoded value and a JSON content type.
///
/// # Safety
/// `value` must be a valid `Json` value.
#[no_mangle]
pub unsafe extern "C" fn kode_response_json(value: *mut Header) -> *mut Header {
    let text = match crate::json::encode_value(value) {
        Ok(text) => text,
        Err(msg) => {
            throw_msg(&msg);
            return std::ptr::null_mut();
        }
    };
    let body = from_bytes(text.as_bytes());
    let ctype = from_bytes(b"application/json");
    let p = kode_obj_new(RESPONSE_FIELDS, RESPONSE_MASK);
    *((p as *mut i64).add(3)) = 200;
    *((p as *mut *mut Header).add(4)) = body;
    *((p as *mut *mut Header).add(5)) = ctype;
    p
}

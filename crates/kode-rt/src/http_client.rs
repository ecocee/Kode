//! A small HTTP/1.1 client for `http_get`, `http_post` and `http_request`.
//!
//! Plain `http://` only: the runtime has no TLS library yet, so `https://` fails
//! with a clear message instead of sending an unencrypted request. Responses are
//! read with `Content-Length`, chunked transfer encoding, or up to end of stream.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::error::throw_msg;
use crate::map::{entries, kode_map_new, kode_map_set};
use crate::object::{kode_obj_new, kode_release, Header};
use crate::string::{bytes, from_bytes};

const MAX_BODY: usize = 64 << 20;
const MAX_LINE: usize = 8 << 10;
const MAX_HEADER_BYTES: usize = 64 << 10;
const MAX_HEADERS: usize = 200;
const TIMEOUT: Duration = Duration::from_secs(30);

struct Target {
    host: String,
    port: u16,
    path: String,
    host_header: String,
}

fn parse_port(text: &str) -> Result<u16, String> {
    let port: u32 = text
        .parse()
        .map_err(|_| format!("`{text}` is not a valid port"))?;
    if port == 0 || port > 65535 {
        return Err(format!("port {port} is out of range"));
    }
    Ok(port as u16)
}

fn parse_url(url: &str) -> Result<Target, String> {
    if url.chars().any(|c| c.is_control() || c == ' ') {
        return Err("the URL contains a space or control character".into());
    }
    let scheme_end = url
        .find("://")
        .ok_or("the URL must start with http://")?;
    let scheme = &url[..scheme_end];
    if scheme.eq_ignore_ascii_case("https") {
        return Err("https is not supported yet: the Kode HTTP client speaks plain http:// only".into());
    }
    if !scheme.eq_ignore_ascii_case("http") {
        return Err(format!("unsupported URL scheme `{scheme}`"));
    }
    let rest = &url[scheme_end + 3..];
    let split = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..split];
    if authority.is_empty() {
        return Err("the URL has no host".into());
    }
    if authority.contains('@') {
        return Err("user names in URLs are not supported".into());
    }
    let (host, port, bracketed) = if let Some(after) = authority.strip_prefix('[') {
        let end = after
            .find(']')
            .ok_or("the IPv6 host in the URL is not closed")?;
        let host = &after[..end];
        let tail = &after[end + 1..];
        let port = if tail.is_empty() {
            80
        } else if let Some(p) = tail.strip_prefix(':') {
            parse_port(p)?
        } else {
            return Err("unexpected text after the IPv6 host".into());
        };
        (host.to_string(), port, true)
    } else {
        if authority.matches(':').count() > 1 {
            return Err("put an IPv6 address in brackets, like http://[::1]:8080/".into());
        }
        match authority.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), parse_port(p)?, false),
            None => (authority.to_string(), 80, false),
        }
    };
    if host.is_empty() {
        return Err("the URL has no host".into());
    }
    let raw = rest[split..].split('#').next().unwrap_or("");
    let path = if raw.is_empty() {
        "/".to_string()
    } else if raw.starts_with('?') {
        format!("/{raw}")
    } else {
        raw.to_string()
    };
    let host_header = if bracketed {
        if port == 80 {
            format!("[{host}]")
        } else {
            format!("[{host}]:{port}")
        }
    } else if port == 80 {
        host.clone()
    } else {
        format!("{host}:{port}")
    };
    Ok(Target {
        host,
        port,
        path,
        host_header,
    })
}

fn valid_method(method: &str) -> bool {
    !method.is_empty()
        && method.len() <= 20
        && method
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
}

fn valid_header_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric() || b"-_.!#$%&'*+^`|~".contains(&b)
        })
}

fn valid_header_value(value: &str) -> bool {
    !value
        .chars()
        .any(|c| c == '\r' || c == '\n' || (c.is_control() && c != '\t'))
}

/// Read one CRLF (or LF) terminated line, without the line ending.
fn read_line(reader: &mut impl BufRead, max: usize) -> Result<String, String> {
    let mut buf = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        match reader.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                if buf.len() >= max {
                    return Err("a response line is too long".into());
                }
                if byte[0] != b'\r' {
                    buf.push(byte[0]);
                }
            }
            Err(e) => return Err(format!("cannot read the response: {e}")),
        }
    }
    String::from_utf8(buf).map_err(|_| "the response is not valid UTF-8".into())
}

fn read_chunked(reader: &mut impl BufRead) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    loop {
        let line = read_line(reader, 128)?;
        let size_text = line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| format!("`{size_text}` is not a chunk size"))?;
        if size == 0 {
            loop {
                if read_line(reader, MAX_LINE)?.is_empty() {
                    break;
                }
            }
            return Ok(body);
        }
        if body.len() + size > MAX_BODY {
            return Err("the response body is larger than the limit".into());
        }
        let start = body.len();
        body.resize(start + size, 0);
        reader
            .read_exact(&mut body[start..])
            .map_err(|e| format!("a chunk ended early: {e}"))?;
        let mut crlf = [0u8; 2];
        reader
            .read_exact(&mut crlf)
            .map_err(|e| format!("a chunk has no ending: {e}"))?;
        if &crlf != b"\r\n" {
            return Err("a chunk does not end with CRLF".into());
        }
    }
}

unsafe fn response(status: i64, body: &[u8], headers: *mut Header) -> *mut Header {
    let p = kode_obj_new(3, 0b110);
    *((p as *mut i64).add(3)) = status;
    *((p as *mut *mut Header).add(4)) = from_bytes(body);
    *((p as *mut *mut Header).add(5)) = headers;
    p
}

unsafe fn request_inner(
    method: &str,
    url: &str,
    extra: &[(String, String)],
    body: &[u8],
) -> Result<*mut Header, String> {
    if !valid_method(method) {
        return Err(format!("`{method}` is not a valid HTTP method"));
    }
    let target = parse_url(url)?;

    let mut head = String::new();
    head.push_str(&format!("{method} {} HTTP/1.1\r\n", target.path));
    head.push_str(&format!("Host: {}\r\n", target.host_header));
    head.push_str("Connection: close\r\nAccept: */*\r\n");
    for (name, value) in extra {
        if name.eq_ignore_ascii_case("host")
            || name.eq_ignore_ascii_case("connection")
            || name.eq_ignore_ascii_case("content-length")
        {
            continue;
        }
        if !valid_header_name(name) {
            return Err(format!("`{name}` is not a valid HTTP header name"));
        }
        if !valid_header_value(value) {
            return Err(format!("the value of header `{name}` contains a control character"));
        }
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    let sends_body = !body.is_empty() || matches!(method, "POST" | "PUT" | "PATCH");
    if sends_body {
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("\r\n");

    let mut stream = TcpStream::connect((target.host.as_str(), target.port))
        .map_err(|e| format!("cannot connect to {}: {e}", target.host_header))?;
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    let _ = stream.set_write_timeout(Some(TIMEOUT));
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|e| format!("cannot send the request: {e}"))?;

    let mut reader = BufReader::new(stream);
    let status_line = read_line(&mut reader, MAX_LINE)?;
    if status_line.is_empty() {
        return Err("the server closed the connection without a response".into());
    }
    let mut parts = status_line.split_whitespace();
    let version = parts.next().unwrap_or("");
    if !version.starts_with("HTTP/") {
        return Err(format!("this is not an HTTP response: `{status_line}`"));
    }
    let code = parts.next().ok_or("the response has no status code")?;
    let status: u16 = code
        .parse()
        .map_err(|_| format!("`{code}` is not a status code"))?;

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut header_bytes = 0usize;
    loop {
        let line = read_line(&mut reader, MAX_LINE)?;
        if line.is_empty() {
            break;
        }
        header_bytes += line.len();
        if header_bytes > MAX_HEADER_BYTES {
            return Err("the response headers are too large".into());
        }
        if headers.len() >= MAX_HEADERS {
            return Err("the response has too many headers".into());
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| format!("malformed response header: `{line}`"))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
    }
    let find = |name: &str| {
        headers
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };

    let mut body_out = Vec::new();
    let no_body = method == "HEAD" || status == 204 || status == 304;
    if !no_body {
        let chunked = find("transfer-encoding")
            .map(|v| v.to_ascii_lowercase().contains("chunked"))
            .unwrap_or(false);
        if chunked {
            body_out = read_chunked(&mut reader)?;
        } else if let Some(length) = find("content-length") {
            let n: usize = length
                .parse()
                .map_err(|_| format!("`{length}` is not a Content-Length"))?;
            if n > MAX_BODY {
                return Err("the response body is larger than the limit".into());
            }
            body_out.resize(n, 0);
            reader
                .read_exact(&mut body_out)
                .map_err(|e| format!("the response body ended early: {e}"))?;
        } else {
            reader
                .by_ref()
                .take((MAX_BODY + 1) as u64)
                .read_to_end(&mut body_out)
                .map_err(|e| format!("cannot read the response body: {e}"))?;
            if body_out.len() > MAX_BODY {
                return Err("the response body is larger than the limit".into());
            }
        }
    }

    let map = kode_map_new(1);
    for (name, value) in &headers {
        let key = from_bytes(name.as_bytes());
        kode_map_set(map, key, from_bytes(value.as_bytes()) as i64);
        kode_release(key);
    }
    Ok(response(status as i64, &body_out, map))
}

unsafe fn finish(
    method: &str,
    url: *const Header,
    extra: &[(String, String)],
    body: &[u8],
) -> *mut Header {
    let url = std::str::from_utf8_unchecked(bytes(url));
    match request_inner(method, url, extra, body) {
        Ok(r) => r,
        Err(msg) => {
            throw_msg(&msg);
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `url` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_http_get(url: *const Header) -> *mut Header {
    finish("GET", url, &[], &[])
}

/// # Safety
/// All arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_http_post(
    url: *const Header,
    body: *const Header,
    content_type: *const Header,
) -> *mut Header {
    let mut extra = Vec::new();
    let ctype = std::str::from_utf8_unchecked(bytes(content_type));
    if !ctype.is_empty() {
        extra.push(("Content-Type".to_string(), ctype.to_string()));
    }
    finish("POST", url, &extra, bytes(body))
}

/// # Safety
/// All arguments must be valid (`method`, `url`, `body` strings; `headers` a
/// map of strings).
#[no_mangle]
pub unsafe extern "C" fn kode_http_request(
    method: *const Header,
    url: *const Header,
    headers: *const Header,
    body: *const Header,
) -> *mut Header {
    let mut extra = Vec::new();
    for (key, value) in entries(headers) {
        extra.push((
            String::from_utf8_lossy(bytes(key)).to_string(),
            String::from_utf8_lossy(bytes(value as *const Header)).to_string(),
        ));
    }
    let method = std::str::from_utf8_unchecked(bytes(method));
    finish(method, url, &extra, bytes(body))
}

//! End-to-end test of the Router API: start `examples/api.kode` on a free port
//! and talk to it over a real socket.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a free port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

fn request(port: u16, raw: &str) -> (u16, String, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to the server");
    stream.write_all(raw.as_bytes()).expect("write the request");
    let mut out = Vec::new();
    stream.read_to_end(&mut out).expect("read the response");
    let text = String::from_utf8_lossy(&out).to_string();
    let (head, body) = text.split_once("\r\n\r\n").expect("a response head");
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .expect("a status code");
    let ctype = head
        .lines()
        .find_map(|line| {
            let lower = line.to_ascii_lowercase();
            lower
                .strip_prefix("content-type:")
                .map(|_| line.split_once(':').unwrap().1.trim().to_string())
        })
        .unwrap_or_default();
    (status, ctype, body.to_string())
}

fn get(port: u16, path: &str) -> (u16, String, String) {
    request(
        port,
        &format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
    )
}

#[test]
fn router_serves_routes_params_query_and_errors() {
    let port = free_port();
    let program = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/api.kode");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kode"))
        .arg("run")
        .arg(&program)
        .arg("--")
        .arg(port.to_string())
        .env("KODE_LEAK_CHECK", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the API");

    let deadline = Instant::now() + Duration::from_secs(20);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        if Instant::now() > deadline {
            let _ = child.kill();
            let out = child.wait_with_output().expect("collect the server output");
            panic!(
                "the server did not start\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let (status, ctype, body) = get(port, "/users/7");
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    assert_eq!(body, r#"{"id":7,"name":"Asha"}"#);

    let (status, _, body) = get(port, "/users?limit=5");
    assert_eq!(status, 200);
    assert_eq!(body, r#"{"count":2,"limit":"5"}"#);

    let (status, _, body) = request(
        port,
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"name\":\"Ravi\"}",
    );
    assert_eq!(status, 201);
    assert_eq!(body, r#"{"name":"Ravi"}"#);

    let (status, _, _) = request(
        port,
        "PUT /users/7 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 405);

    let (status, _, body) = get(port, "/missing");
    assert_eq!(status, 404);
    assert_eq!(body, "not found");

    let _ = child.kill();
    let _ = child.wait();
}

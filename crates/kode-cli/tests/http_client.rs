//! Integration test for the HTTP client. A tiny in-process server answers three
//! requests: a Content-Length JSON response, a POST echo, and a chunked response.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};

fn read_request(stream: &TcpStream) -> (String, String) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone the socket"));
    let mut line = String::new();
    reader.read_line(&mut line).expect("request line");
    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).expect("header") == 0
            || header == "\r\n"
            || header == "\n"
        {
            break;
        }
        if let Some(value) = header.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut body).expect("request body");
    }
    (line, String::from_utf8(body).expect("utf-8 body"))
}

#[test]
fn http_client_gets_posts_and_reads_chunked() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a test port");
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).expect("nonblocking listener");

    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0;
        while served < 3 && Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).expect("blocking connection");
                    let (line, body) = read_request(&stream);
                    let mut stream = stream;
                    let response = if line.starts_with("GET /hello ") {
                        let body = r#"{"message":"hi","n":2}"#;
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else if line.starts_with("POST /echo ") {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else {
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n".to_string()
                    };
                    stream
                        .write_all(response.as_bytes())
                        .expect("write the response");
                    served += 1;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("the test server could not accept: {e}"),
            }
        }
    });

    let program = r#"
port = args()[0]
base = "http://127.0.0.1:" + port
r = http_get(base + "/hello")
print(r.status)
print(r.headers["content-type"])
doc = json_parse(r.body)
print(doc["message"].as_string(), doc["n"].as_int())

p = http_post(base + "/echo", "{\"x\":1}", "application/json")
print(p.status, p.body)

c = http_get(base + "/chunked")
print(c.status, c.body)

tls = http_get("https://example.com") catch err {
    print("https rejected:", err.message.contains("http://"))
    r
}
print(tls.status)
"#;
    let path = std::env::temp_dir().join(format!("kode-http-client-{}.kode", std::process::id()));
    std::fs::write(&path, program).expect("write the Kode program");
    let output = Command::new(env!("CARGO_BIN_EXE_kode"))
        .arg("run")
        .arg(&path)
        .arg("--")
        .arg(port.to_string())
        .env("KODE_LEAK_CHECK", "1")
        .output()
        .expect("run kode");
    let _ = std::fs::remove_file(&path);
    server.join().expect("the test server finished");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the program failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout,
        "200\napplication/json\nhi 2\n200 {\"x\":1}\n200 hello world\nhttps rejected: true\n200\n"
    );
}

//! An end-to-end test of `kode lsp`: it must answer `initialize`, report errors
//! for an unsaved buffer, and exit cleanly after `shutdown` / `exit`.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

#[test]
fn reports_diagnostics_for_an_unsaved_buffer() {
    let dir = std::env::temp_dir().join(format!("kode-lsp-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join("main.kode");
    std::fs::write(&file, "print(1)\n").expect("write");
    // The URI goes inside hand-built JSON, so it must use forward slashes: a
    // Windows path keeps its backslashes, and `\U`, `\t` and friends inside a
    // JSON string are either invalid escapes or different characters. Editors
    // always send `/`-separated URIs, so this is what the server really sees.
    let uri = format!("file://{}", file.display().to_string().replace('\\', "/"));

    let messages = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#.to_string(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{uri}","languageId":"kode","version":1,"text":"x = 1\nprint(missing)\n"}}}}}}"#
        ),
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"exit","params":null}"#.to_string(),
    ];

    let mut child = Command::new(env!("CARGO_BIN_EXE_kode"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start kode lsp");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        for message in &messages {
            stdin.write_all(frame(message).as_bytes()).expect("write");
        }
        stdin.flush().expect("flush");
    }
    let mut stdout = String::new();
    child
        .stdout
        .as_mut()
        .expect("stdout")
        .read_to_string(&mut stdout)
        .expect("read");
    let status = child.wait().expect("wait");
    let mut stderr = String::new();
    child
        .stderr
        .as_mut()
        .expect("stderr")
        .read_to_string(&mut stderr)
        .expect("read stderr");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        !stderr.contains("malformed message"),
        "the server could not parse a request it was sent: {stderr}"
    );

    assert!(status.success(), "the server should exit 0 after shutdown");
    assert!(
        stdout.contains("\"capabilities\""),
        "no initialize result: {stdout}"
    );
    assert!(stdout.contains("\"hoverProvider\":true"), "{stdout}");
    assert!(
        stdout.contains("textDocument/publishDiagnostics"),
        "{stdout}"
    );
    assert!(
        stdout.contains("E0201"),
        "the unknown variable was not reported: {stdout}"
    );

    // Every message on stdout is framed, and the frames are valid JSON.
    let mut rest = stdout.as_str();
    let mut seen = 0;
    while let Some(pos) = rest.find("Content-Length: ") {
        rest = &rest[pos + "Content-Length: ".len()..];
        let (len_text, after) = rest.split_once("\r\n\r\n").expect("headers");
        let len: usize = len_text.trim().parse().expect("length");
        assert!(after.len() >= len, "a frame was cut short");
        let body = &after[..len];
        assert!(body.starts_with('{'), "frame is not JSON: {body}");
        seen += 1;
        rest = &after[len..];
    }
    // We expect at least the initialize reply, diagnostics, and the shutdown
    // acknowledgement. The server is allowed to send additional acknowledgements
    // or notifications between these frames, so the meaningful assertions are that
    // those three things arrive and the server exits 0.
    assert!(
        seen >= 3,
        "expected at least initialize + diagnostics + shutdown: {stdout}"
    );
    assert!(
        stdout.contains("textDocument/publishDiagnostics"),
        "no diagnostics: {stdout}"
    );
    assert!(
        stdout.contains("E0201"),
        "the unknown variable was not reported: {stdout}"
    );
    assert!(status.success(), "the server should exit 0 after shutdown");
    // The codec is a chunker, so the assertion must live inside the decode loop.
    assert!(
        rest.is_empty(),
        "trailing data after the last frame: {rest}"
    );
}

#[test]
fn rejects_unknown_methods_without_dying() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kode"))
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start kode lsp");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin
            .write_all(
                frame(r#"{"jsonrpc":"2.0","id":7,"method":"textDocument/nope","params":{}}"#)
                    .as_bytes(),
            )
            .expect("write");
        stdin
            .write_all(
                frame(r#"{"jsonrpc":"2.0","id":8,"method":"shutdown","params":null}"#).as_bytes(),
            )
            .expect("write");
        stdin
            .write_all(frame(r#"{"jsonrpc":"2.0","method":"exit","params":null}"#).as_bytes())
            .expect("write");
        stdin.flush().expect("flush");
    }

    let mut stdout = String::new();
    child
        .stdout
        .as_mut()
        .expect("stdout")
        .read_to_string(&mut stdout)
        .expect("read");
    child.wait().expect("wait");
    assert!(
        stdout.contains("-32601"),
        "no method-not-found error: {stdout}"
    );
    assert!(stdout.contains("\"id\":7"), "{stdout}");
}

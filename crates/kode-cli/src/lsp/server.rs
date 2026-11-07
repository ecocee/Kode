//! The language server loop: `Content-Length` framed JSON-RPC on stdio.
//! Requests are answered on stdout; only protocol messages are printed there,
//! so `kode check --format=json` style output never mixes with it.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use kode_diagnostics::Diagnostic;
use kode_sema::Checked;
use kode_syntax::ast::Program;

use super::features;
use super::json::{self, Json};
use super::nav;
use super::position;
use crate::driver;
use crate::source::SourceMap;

/// Run the server until the client sends `exit` or closes stdin.
/// Exit code 0 means a clean `shutdown`, 1 means the client vanished.
pub fn serve() -> Result<u8, String> {
    let stdin = std::io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut state = State::default();
    while let Some(body) = read_message(&mut input)? {
        let msg = match json::parse(&body) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("kode lsp: ignoring a malformed message: {e}");
                continue;
            }
        };
        if !state.dispatch(&msg, &mut out)? {
            break;
        }
    }
    Ok(if state.shutdown { 0 } else { 1 })
}

struct Doc {
    path: PathBuf,
    text: String,
    map: SourceMap,
    errors: Vec<Diagnostic>,
    program: Option<Program>,
    checked: Option<Checked>,
}

#[derive(Default)]
struct State {
    docs: HashMap<String, Doc>,
    shutdown: bool,
}

fn analyze(path: &Path, text: &str) -> Doc {
    let wanted = path.to_path_buf();
    let source = text.to_string();
    let overlay = move |p: &Path| {
        if p == wanted {
            Some(source.clone())
        } else {
            None
        }
    };
    match driver::load_with(path, &overlay) {
        Err((map, errors)) => Doc {
            path: path.to_path_buf(),
            text: text.to_string(),
            map,
            errors,
            program: None,
            checked: None,
        },
        Ok(loaded) => match kode_sema::check(&loaded.program, &loaded.modules) {
            Err(errors) => Doc {
                path: path.to_path_buf(),
                text: text.to_string(),
                map: loaded.map,
                errors,
                program: Some(loaded.program),
                checked: None,
            },
            Ok(checked) => Doc {
                path: path.to_path_buf(),
                text: text.to_string(),
                map: loaded.map,
                errors: checked.warnings.clone(),
                program: Some(loaded.program),
                checked: Some(checked),
            },
        },
    }
}

impl State {
    fn dispatch(&mut self, msg: &Json, out: &mut impl Write) -> Result<bool, String> {
        let method = msg.str_of("method").unwrap_or("");
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Json::Null);
        match method {
            "initialize" => self.respond(out, id, capabilities()),
            "initialized"
            | "workspace/didChangeConfiguration"
            | "$/setTrace"
            | "$/cancelRequest" => {}
            "shutdown" => {
                self.shutdown = true;
                self.respond(out, id, Json::Null);
            }
            "exit" => return Ok(false),
            "textDocument/didOpen" => self.did_open(&params, out),
            "textDocument/didChange" => self.did_change(&params, out),
            "textDocument/didSave" => self.did_save(&params, out),
            "textDocument/didClose" => self.did_close(&params, out),
            "textDocument/hover" => self.hover(&params, out, id),
            "textDocument/completion" => self.completion(&params, out, id),
            "textDocument/documentSymbol" => self.document_symbols(&params, out, id),
            "textDocument/formatting" => self.formatting(&params, out, id),
            "textDocument/definition" => self.definition(&params, out, id),
            "textDocument/references" => self.references(&params, out, id),
            "textDocument/rename" => self.rename(&params, out, id),
            "textDocument/signatureHelp" => self.signature_help(&params, out, id),
            "" => {} // a response from the client: nothing to do
            other => {
                if let Some(id) = id {
                    self.respond_error(out, id, -32601, &format!("unknown method `{other}`"));
                }
            }
        }
        Ok(true)
    }

    fn did_open(&mut self, params: &Json, out: &mut impl Write) {
        let Some(doc) = params.get("textDocument") else {
            return;
        };
        let (Some(uri), Some(text)) = (doc.str_of("uri"), doc.str_of("text")) else {
            return;
        };
        let Some(path) = position::path_from_uri(uri) else {
            return;
        };
        self.docs.insert(uri.to_string(), analyze(&path, text));
        self.publish(uri, out);
    }

    fn did_change(&mut self, params: &Json, out: &mut impl Write) {
        let Some(doc) = params.get("textDocument") else {
            return;
        };
        let Some(uri) = doc.str_of("uri") else { return };
        let Some(change) = params.arr_of("contentChanges").last() else {
            return;
        };
        let Some(text) = change.str_of("text") else {
            return;
        };
        let Some(current) = self.docs.get(uri) else {
            return;
        };
        let path = current.path.clone();
        self.docs.insert(uri.to_string(), analyze(&path, text));
        self.publish(uri, out);
    }

    fn did_save(&mut self, params: &Json, out: &mut impl Write) {
        let Some(doc) = params.get("textDocument") else {
            return;
        };
        let Some(uri) = doc.str_of("uri") else { return };
        self.publish(uri, out);
    }

    fn did_close(&mut self, params: &Json, out: &mut impl Write) {
        let Some(doc) = params.get("textDocument") else {
            return;
        };
        let Some(uri) = doc.str_of("uri") else { return };
        self.docs.remove(uri);
        self.notify(
            out,
            "textDocument/publishDiagnostics",
            Json::obj(vec![
                ("uri", Json::str(uri)),
                ("diagnostics", Json::Arr(Vec::new())),
            ]),
        );
    }

    fn hover(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(doc) = params.get("textDocument") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(uri) = doc.str_of("uri") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(d) = self.docs.get(uri) else {
            return self.respond(out, id, Json::Null);
        };
        let (Some(program), Some(checked)) = (&d.program, &d.checked) else {
            return self.respond(out, id, Json::Null);
        };
        let line = params
            .get("position")
            .and_then(|p| p.int_of("line"))
            .unwrap_or(0) as u32;
        let character = params
            .get("position")
            .and_then(|p| p.int_of("character"))
            .unwrap_or(0) as u32;
        let local = position::offset(&d.text, line, character);
        let name = d.path.display().to_string();
        let result = features::hover(program, checked, &d.map, &name, local).unwrap_or(Json::Null);
        self.respond(out, id, result);
    }

    fn completion(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(doc) = params.get("textDocument") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(uri) = doc.str_of("uri") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(d) = self.docs.get(uri) else {
            return self.respond(out, id, Json::Null);
        };
        let line = params
            .get("position")
            .and_then(|p| p.int_of("line"))
            .unwrap_or(0) as u32;
        let character = params
            .get("position")
            .and_then(|p| p.int_of("character"))
            .unwrap_or(0) as u32;
        let at = position::offset(&d.text, line, character);
        // A buffer that does not parse is still worth completing: fall back to
        // whatever the last good parse of the text gives.
        let fallback = if d.program.is_none() {
            kode_syntax::parse_source(&d.text).ok()
        } else {
            None
        };
        let program = d.program.as_ref().or(fallback.as_ref());
        self.respond(out, id, features::completion(program, &d.text, at));
    }

    fn document_symbols(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(doc) = params.get("textDocument") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(uri) = doc.str_of("uri") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(d) = self.docs.get(uri) else {
            return self.respond(out, id, Json::Null);
        };
        self.respond(out, id, features::symbols(&d.text));
    }

    fn formatting(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(doc) = params.get("textDocument") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(uri) = doc.str_of("uri") else {
            return self.respond(out, id, Json::Null);
        };
        let Some(d) = self.docs.get(uri) else {
            return self.respond(out, id, Json::Null);
        };
        self.respond(out, id, features::formatting(&d.text));
    }

    fn doc(&self, params: &Json) -> Option<&Doc> {
        let uri = params.get("textDocument")?.str_of("uri")?;
        self.docs.get(uri)
    }

    fn definition(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(d) = self.doc(params) else {
            return self.respond(out, id, Json::Null);
        };
        let at = cursor(d, params);
        let uri = position::uri_from_path(&d.path);
        let result = nav::definition(&d.text, &uri, at).unwrap_or(Json::Null);
        self.respond(out, id, result);
    }

    fn references(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(d) = self.doc(params) else {
            return self.respond(out, id, Json::Null);
        };
        let at = cursor(d, params);
        let uri = position::uri_from_path(&d.path);
        let result = Json::Arr(nav::references(&d.text, &uri, at));
        self.respond(out, id, result);
    }

    fn rename(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(d) = self.doc(params) else {
            return self.respond(out, id, Json::Null);
        };
        let at = cursor(d, params);
        let uri = position::uri_from_path(&d.path);
        let new_name = params.str_of("newName").unwrap_or("");
        let result = nav::rename(&d.text, &uri, at, new_name).unwrap_or(Json::Null);
        self.respond(out, id, result);
    }

    fn signature_help(&mut self, params: &Json, out: &mut impl Write, id: Option<Json>) {
        let Some(d) = self.doc(params) else {
            return self.respond(out, id, Json::Null);
        };
        let at = cursor(d, params);
        let result = nav::signature_help(&d.text, at).unwrap_or(Json::Null);
        self.respond(out, id, result);
    }

    fn publish(&self, uri: &str, out: &mut impl Write) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        let name = doc.path.display().to_string();
        let diags = features::diagnostics_for(&doc.map, &name, &doc.errors);
        self.notify(
            out,
            "textDocument/publishDiagnostics",
            Json::obj(vec![
                ("uri", Json::str(uri)),
                ("diagnostics", Json::Arr(diags)),
            ]),
        );
    }

    fn respond(&self, out: &mut impl Write, id: Option<Json>, result: Json) {
        let Some(id) = id else { return };
        let msg = Json::obj(vec![
            ("jsonrpc", Json::str("2.0")),
            ("id", id),
            ("result", result),
        ]);
        let _ = write_message(out, &msg);
    }

    fn respond_error(&self, out: &mut impl Write, id: Json, code: i64, message: &str) {
        let msg = Json::obj(vec![
            ("jsonrpc", Json::str("2.0")),
            ("id", id),
            (
                "error",
                Json::obj(vec![
                    ("code", Json::int(code)),
                    ("message", Json::str(message)),
                ]),
            ),
        ]);
        let _ = write_message(out, &msg);
    }

    fn notify(&self, out: &mut impl Write, method: &str, params: Json) {
        let msg = Json::obj(vec![
            ("jsonrpc", Json::str("2.0")),
            ("method", Json::str(method)),
            ("params", params),
        ]);
        let _ = write_message(out, &msg);
    }
}

fn cursor(d: &Doc, params: &Json) -> usize {
    let Some(p) = params.get("position") else {
        return 0;
    };
    let line = p.int_of("line").unwrap_or(0) as u32;
    let character = p.int_of("character").unwrap_or(0) as u32;
    position::offset(&d.text, line, character)
}

fn capabilities() -> Json {
    Json::obj(vec![
        (
            "capabilities",
            Json::obj(vec![
                ("textDocumentSync", Json::int(1)),
                ("hoverProvider", Json::Bool(true)),
                (
                    "completionProvider",
                    Json::obj(vec![("triggerCharacters", Json::Arr(vec![Json::str(".")]))]),
                ),
                ("documentSymbolProvider", Json::Bool(true)),
                ("documentFormattingProvider", Json::Bool(true)),
                ("definitionProvider", Json::Bool(true)),
                ("referencesProvider", Json::Bool(true)),
                ("renameProvider", Json::Bool(true)),
                (
                    "signatureHelpProvider",
                    Json::obj(vec![(
                        "triggerCharacters",
                        Json::Arr(vec![Json::str("("), Json::str(",")]),
                    )]),
                ),
            ]),
        ),
        (
            "serverInfo",
            Json::obj(vec![
                ("name", Json::str("kode")),
                ("version", Json::str(env!("CARGO_PKG_VERSION"))),
            ]),
        ),
    ])
}

fn write_message(out: &mut impl Write, msg: &Json) -> Result<(), String> {
    let body = msg.to_json();
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())
}

/// Read one framed message. `Ok(None)` means stdin closed.
fn read_message(input: &mut impl BufRead) -> Result<Option<String>, String> {
    let mut length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = input.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(None);
        }
        let text = line.trim_end_matches(['\r', '\n']);
        if text.is_empty() {
            match length {
                Some(_) => break,
                None => continue,
            }
        }
        if let Some(rest) = text.strip_prefix("Content-Length:") {
            length = rest.trim().parse().ok();
        }
    }
    let length = length.ok_or("a message has no `Content-Length` header")?;
    let mut buf = vec![0u8; length];
    input.read_exact(&mut buf).map_err(|e| e.to_string())?;
    String::from_utf8(buf).map(Some).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_a_framed_message() {
        let raw = "Content-Length: 7\r\n\r\n{\"a\":1}";
        let mut input = Cursor::new(raw.as_bytes());
        assert_eq!(
            read_message(&mut input).unwrap().as_deref(),
            Some("{\"a\":1}")
        );
        assert_eq!(read_message(&mut input).unwrap(), None);
    }

    #[test]
    fn skips_unknown_headers() {
        let raw = "Content-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        let mut input = Cursor::new(raw.as_bytes());
        assert_eq!(read_message(&mut input).unwrap().as_deref(), Some("{}"));
    }

    #[test]
    fn writes_a_framed_message() {
        let mut out: Vec<u8> = Vec::new();
        write_message(&mut out, &Json::obj(vec![("x", Json::int(1))])).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("Content-Length: 7\r\n\r\n"));
        assert!(text.ends_with("{\"x\":1}"));
    }

    #[test]
    fn analyses_an_unsaved_buffer() {
        let dir = std::env::temp_dir().join(format!("kode-lsp-analyze-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("main.kode");
        std::fs::write(&file, "print(1)\n").unwrap();
        let doc = analyze(&file, "x = 1\nprint(missing)\n");
        assert!(doc.checked.is_none());
        assert!(
            doc.errors.iter().any(|e| e.code == "E0201"),
            "{:?}",
            doc.errors
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

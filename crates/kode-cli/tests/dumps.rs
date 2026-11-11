//! `kode tokens` and `kode ast` are the oracles the self-hosted compiler is
//! compared against (`docs/PLAN-self-hosting.md` §5), so the output is checked
//! byte for byte here: a change to it is a change to that contract.

use std::path::PathBuf;
use std::process::Command;

fn kode() -> &'static str {
    env!("CARGO_BIN_EXE_kode")
}

fn write_temp(name: &str, source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("kode-dump-{}-{name}.kode", std::process::id()));
    std::fs::write(&path, source).expect("write the source");
    path
}

fn run(args: &[&str]) -> (i32, String) {
    let result = Command::new(kode()).args(args).output().expect("run kode");
    (
        result.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&result.stdout).to_string(),
    )
}

#[test]
fn tokens_have_spans_and_places() {
    let file = write_temp("tokens", "x = 1\n");
    let (code, out) = run(&["tokens", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        "0..1\t1:1\tIdent\tx\n\
         2..3\t1:3\tAssign\t=\n\
         4..5\t1:5\tInt\t1\n\
         5..6\t1:6\tNewline\t\\n\n\
         6..6\t2:1\tEof\t\n"
    );
    let _ = std::fs::remove_file(&file);
}

#[test]
fn tokens_json_uses_bytes_for_spans_and_characters_for_columns() {
    let file = write_temp("tokens-json", "x = \"é\"\n");
    let (code, out) = run(&["tokens", file.to_str().unwrap(), "--format=json"]);
    assert_eq!(code, 0);
    assert!(
        out.starts_with("[\n  {\"kind\":\"Ident\",\"text\":\"x\",\"span\":{\"start\":0,\"end\":1},\"line\":1,\"column\":1},"),
        "{out}"
    );
    // `é` is two bytes and one column wide, so the span is 4..8 and the
    // remaining tokens still start at column 8.
    assert!(
        out.contains("{\"kind\":\"Str\",\"text\":\"\\\"é\\\"\",\"span\":{\"start\":4,\"end\":8},\"line\":1,\"column\":5}"),
        "{out}"
    );
    let _ = std::fs::remove_file(&file);
}

#[test]
fn the_tree_keeps_shape_and_spans() {
    let file = write_temp("ast", "x = [1, 2]\n");
    let (code, out) = run(&["ast", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Program "), "{out}");
    assert!(out.contains("TargetVar name=\"x\" span=0..1"), "{out}");
    assert!(out.contains("List span="), "{out}");
    assert!(out.contains("Int value=1"), "{out}");
    // Allocation order is not part of the contract.
    assert!(!out.contains("id="), "{out}");
    let _ = std::fs::remove_file(&file);
}

#[test]
fn a_lexer_error_is_a_diagnostic_not_a_dump() {
    let file = write_temp("bad-token", "x = \"unterminated\n");
    let (code, out) = run(&["tokens", file.to_str().unwrap(), "--format=json"]);
    assert_eq!(code, 1);
    assert!(out.contains("\"code\":\"E"), "{out}");
    // The diagnostics are the JSON report, not a token dump.
    assert!(!out.contains("\"kind\":"), "{out}");
    let _ = std::fs::remove_file(&file);
}

#[test]
fn a_syntax_error_is_a_diagnostic_not_a_dump() {
    let file = write_temp("bad-ast", "fn (\n");
    let (code, out) = run(&["ast", file.to_str().unwrap(), "--format=json"]);
    assert_eq!(code, 1);
    assert!(out.contains("\"code\":\"E"), "{out}");
    let _ = std::fs::remove_file(&file);
}

//! Conformance tests. Every `.kode` file under `tests/` is compiled by the
//! real `kode` binary.
//!
//! * `tests/run-pass/name.kode` + `name.out`  -> must run and print exactly `name.out`,
//!   and must not leak memory (`KODE_LEAK_CHECK=1`).
//! * `tests/compile-fail/name.kode` + `name.err` -> `kode check --strict --format=json`
//!   must fail and report the diagnostic code written in `name.err`.
//!
//! These files, not the Rust code, are the source of truth for the language.
//! The runtime library is rebuilt automatically before the tests run.

use std::path::{Path, PathBuf};
use std::process::Command;

fn kode() -> &'static str {
    env!("CARGO_BIN_EXE_kode")
}

fn tests_dir(sub: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests")
        .join(sub)
}

fn kode_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map(|e| e == "kode").unwrap_or(false))
        .collect();
    files.sort();
    files
}

#[test]
fn run_pass() {
    let files = kode_files(&tests_dir("run-pass"));
    assert!(!files.is_empty(), "no run-pass programs found");
    let mut failures = Vec::new();
    for file in &files {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        // A test that can only run on one family of systems says so on its first
        // line: `// skip-os: windows` or `// skip-os: unix`.
        let source = std::fs::read_to_string(file).unwrap_or_default();
        if let Some(os) = source.lines().next().and_then(|l| l.strip_prefix("// skip-os: ")) {
            let skip = match os.trim() {
                "windows" => cfg!(windows),
                "unix" => !cfg!(windows),
                _ => false,
            };
            if skip {
                continue;
            }
            if skip {
                continue;
            }
        }
        // A checkout made with `core.autocrlf` on Windows can still hand us CRLF;
        // the programs themselves always print LF.
        let expected = std::fs::read_to_string(file.with_extension("out"))
            .unwrap_or_else(|_| panic!("{name}: missing .out file"))
            .replace("\r\n", "\n");
        let result = Command::new(kode())
            .arg("run")
            .arg(file)
            .env("KODE_LEAK_CHECK", "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        if !result.status.success() {
            failures.push(format!("{name}: failed to run\n{stderr}"));
        } else if stdout != expected {
            failures.push(format!(
                "{name}: wrong output\n--- expected\n{expected:?}\n--- got\n{stdout:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        files.len(),
        failures.join("\n\n")
    );
}

#[test]
fn compile_fail() {
    let files = kode_files(&tests_dir("compile-fail"));
    assert!(!files.is_empty(), "no compile-fail programs found");
    let mut failures = Vec::new();
    for file in &files {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        let code = std::fs::read_to_string(file.with_extension("err"))
            .unwrap_or_else(|_| panic!("{name}: missing .err file"))
            .trim()
            .to_string();
        let result = Command::new(kode())
            .args(["check", "--strict", "--format=json"])
            .arg(file)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        if result.status.success() {
            failures.push(format!("{name}: expected {code} but it compiled"));
        } else if !stdout.contains(&format!("\"code\":\"{code}\"")) {
            failures.push(format!("{name}: expected {code}, got\n{stdout}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        files.len(),
        failures.join("\n\n")
    );
}

fn run_kode(args: &[&str]) -> (i32, String, String) {
    let r = Command::new(kode()).args(args).output().unwrap();
    (
        r.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&r.stdout).to_string(),
        String::from_utf8_lossy(&r.stderr).to_string(),
    )
}

#[test]
fn kode_test_command() {
    let dir = tests_dir("kode-test");
    let pass = dir.join("passing.kode");
    let (code, out, _) = run_kode(&["test", pass.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(out, "PASS add works\nPASS strings\n2 passed, 0 failed\n");

    let fail = dir.join("failing.kode");
    let (code, out, _) = run_kode(&["test", fail.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert_eq!(
        out,
        "PASS one\nFAIL two: assertion failed at line 6\nPASS three\n2 passed, 1 failed\n"
    );
}

#[test]
fn explain_command() {
    let (code, out, _) = run_kode(&["explain", "E0201"]);
    assert_eq!(code, 0);
    assert!(out.contains("unknown variable"), "{out}");
    let (code, _, err) = run_kode(&["explain", "E9999"]);
    assert_eq!(code, 1);
    assert!(err.contains("unknown error code"), "{err}");
}

#[test]
fn fmt_command() {
    // Our own examples and tests are already formatted.
    let run_pass = tests_dir("run-pass");
    let kode_test = tests_dir("kode-test");
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let (code, out, _) = run_kode(&[
        "fmt",
        "--check",
        run_pass.to_str().unwrap(),
        kode_test.to_str().unwrap(),
        examples.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "these files should already be formatted:\n{out}");

    // Messy input gets fixed, and the result is stable.
    let tmp = std::env::temp_dir().join(format!("kode-fmt-test-{}.kode", std::process::id()));
    std::fs::write(&tmp, "fn f(a: int) -> int {\nreturn a+1\n}\n\n\n\nif true {\nprint( \"x\" )\n  }\n").unwrap();
    let (code, _, _) = run_kode(&["fmt", tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    let formatted = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(
        formatted,
        "fn f(a: int) -> int {\n    return a+1\n}\n\nif true {\n    print( \"x\" )\n}\n"
    );
    let (code, _, _) = run_kode(&["fmt", "--check", tmp.to_str().unwrap()]);
    assert_eq!(code, 0);
    let _ = std::fs::remove_file(&tmp);
}
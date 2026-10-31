//! Conformance tests. Every `.kode` file under `tests/` is compiled by the
//! real `kode` binary.
//!
//! * `tests/run-pass/name.kode` + `name.out`  -> must run and print exactly `name.out`,
//!   and must not leak memory (`KODE_LEAK_CHECK=1`).
//! * `tests/compile-fail/name.kode` + `name.err` -> `kode check --strict --format=json`
//!   must fail and report the diagnostic code written in `name.err`.
//!
//! These files, not the Rust code, are the source of truth for the language.
//! The runtime library is built automatically if it is missing.

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

/// `cargo test` does not build the runtime library by itself, so build it here if needed.
fn require_runtime() {
    let rt = Path::new(kode()).parent().unwrap().join("libkode_rt.a");
    if !rt.exists() {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args(["build", "-p", "kode-rt"])
            .status()
            .expect("could not run cargo to build the runtime library");
        assert!(status.success(), "building the runtime library failed");
    }
    assert!(
        rt.exists(),
        "runtime library missing at {}: run `cargo build --workspace`",
        rt.display()
    );
}

#[test]
fn run_pass() {
    require_runtime();
    let files = kode_files(&tests_dir("run-pass"));
    assert!(!files.is_empty(), "no run-pass programs found");
    let mut failures = Vec::new();
    for file in &files {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        let expected = std::fs::read_to_string(file.with_extension("out"))
            .unwrap_or_else(|_| panic!("{name}: missing .out file"));
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

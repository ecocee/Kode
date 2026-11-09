//! End-to-end checks for the B1 backend: emit IR, build with clang, link with
//! the Kode runtime, run, and compare the program's output.
//!
//! These tests drive the `kode-llvm` binary, so they cover the whole path the
//! prototype offers. They skip, loudly, when the machine has no `clang`; the
//! error path for that case has its own test below.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_kode-llvm");

fn have_clang() -> bool {
    Command::new(kode_llvm::clang_program())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_file(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kode-llvm-test-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(format!("{name}.kode"));
    std::fs::write(&path, src).expect("write program");
    path
}

struct Run {
    stdout: String,
    stderr: String,
    ok: bool,
}

fn run(name: &str, src: &str) -> Run {
    let path = temp_file(name, src);
    let out = Command::new(BIN)
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kode-llvm");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        ok: out.status.success(),
    }
}

fn assert_output(name: &str, src: &str, expected: &str) {
    if !have_clang() {
        eprintln!("skipping `{name}`: no clang on PATH");
        return;
    }
    let r = run(name, src);
    assert!(r.ok, "`{name}` failed:\n{}", r.stderr);
    assert_eq!(r.stdout.trim(), expected.trim(), "`{name}` output");
}

#[test]
fn recursion_and_returns() {
    let src = "\
fn fib(n: int) -> int {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}
print(fib(20))
";
    assert_output("fib", src, "6765");
}

#[test]
fn loops_remainder_and_bit_operations() {
    let mut total = 0i64;
    for i in 0..1000 {
        total += (i * 7 + 3) % 1000;
    }
    let src = "\
total = 0
for i in 0..1000 {
    total += (i * 7 + 3) % 1000
}
print(total)
print(1 << 64)
print(-1 >> 1)
print(6 & 3)
print(6 | 1)
print(6 ^ 3)
";
    assert_output("loops", src, &format!("{total}\n0\n-1\n2\n7\n5"));
}

#[test]
fn floats_and_comparisons() {
    let src = "\
x = 1.5 * 2.0
print(x < 3.5)
print(x == 3.0)
print(x != 3.0)
print(0.1 + 0.2 > 0.3)
";
    assert_output("floats", src, "true\ntrue\nfalse\ntrue");
}

#[test]
fn bools_and_conditionals() {
    let src = "\
a = true
b = false
if a and not b {
    print(1)
} else {
    print(2)
}
print(not a)
print(a == b)
";
    assert_output("bools", src, "1\nfalse\nfalse");
}

#[test]
fn division_by_zero_stops_with_the_runtime_message() {
    if !have_clang() {
        eprintln!("skipping `divzero`: no clang on PATH");
        return;
    }
    let r = run("divzero", "print(1 / 0)\n");
    assert!(!r.ok, "division by zero must not succeed");
    assert!(
        r.stderr.contains("division by zero"),
        "expected the runtime's message, got: {}",
        r.stderr
    );
}

#[test]
fn missing_clang_is_a_clear_error() {
    let path = temp_file("nocl", "print(1)\n");
    let out = Command::new(BIN)
        .arg("run")
        .arg(&path)
        .env("KODE_CLANG", "/definitely/not/a/compiler")
        .output()
        .expect("run kode-llvm");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    assert!(!out.status.success(), "must fail without clang");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("needs") && stderr.contains("KODE_CLANG"),
        "the error should say what is missing and how to point at it: {stderr}"
    );
}

//! B2: strings, reference counting, optionals and the error slot.
//!
//! Every program here runs with `KODE_LEAK_CHECK=1` and must leave the runtime
//! with nothing to report: a leak, a double free or a use-after-free in the
//! generated retain/release placement would show up on stderr. Outputs are the
//! values the Cranelift backend produces for the same source (checked against
//! `kode run` while writing these tests; the run-pass sweep covers the rest).

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
    let dir = std::env::temp_dir().join(format!(
        "kode-llvm-rt-{}-{}",
        std::process::id(),
        name
    ));
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

/// Build, then run the binary with leak checking on so that a bad
/// retain/release fails the test. Building is a separate process so the
/// linker's own warnings cannot be mistaken for the program's stderr.
fn run(name: &str, src: &str) -> Run {
    let path = temp_file(name, src);
    let dir = path.parent().unwrap().to_path_buf();
    let exe = dir.join("program");
    let built = Command::new(BIN)
        .arg("build")
        .arg(&path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("build with kode-llvm");
    if !built.status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        panic!(
            "`{name}` did not build:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
    }
    let out = Command::new(&exe)
        .env("KODE_LEAK_CHECK", "1")
        .output()
        .expect("run the built program");
    let _ = std::fs::remove_dir_all(&dir);
    Run {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        ok: out.status.success(),
    }
}

fn check(name: &str, src: &str, expected: &str) {
    if !have_clang() {
        eprintln!("skipping `{name}`: no clang on PATH");
        return;
    }
    let r = run(name, src);
    assert!(r.ok, "`{name}` failed:\n{}", r.stderr);
    assert_eq!(r.stdout.trim(), expected.trim(), "`{name}` output");
    assert!(
        r.stderr.trim().is_empty(),
        "`{name}` reported runtime problems (leaks?):\n{}",
        r.stderr
    );
}

#[test]
fn string_concat_length_and_equality() {
    let src = "\
a = \"hello\"
b = \" world\"
c = a + b
print(c)
print(len(c))
print(c == \"hello world\")
print(c != a)
print(len(\"\"))
";
    check("strings", src, "hello world\n11\ntrue\ntrue\n0");
}

#[test]
fn string_conversion_through_interpolation() {
    let src = "\
n = 42
t = true
f = 2.5
print(\"n=${n}\")
print(\"t=${t}\")
print(\"f=${f}\")
";
    check("convert", src, "n=42\nt=true\nf=2.5");
}

#[test]
fn allocations_in_a_loop_are_released() {
    // 5,000 strings, each built by concatenation and dropped at the next
    // iteration; the leak checker sees the whole run.
    let mut total = 0usize;
    for i in 0..5000 {
        total += 1 + i.to_string().len();
    }
    let src = "\
total = 0
i = 0
while i < 5000 {
    s = \"x\" + \"${i}\"
    total += len(s)
    i += 1
}
print(total)
";
    check("allocloop", src, &total.to_string());
}

#[test]
fn optionals_are_none_or_a_value() {
    let src = "\
s: string? = none
print(s ?? \"empty\")
s = \"set\"
print(s ?? \"empty\")
n: int? = none
print(n ?? 7)
n = 3
print(n ?? 7)
print(s == none)
print(n != none)
";
    check("optionals", src, "empty\nset\n7\n3\nfalse\ntrue");
}

#[test]
fn errors_are_caught_and_propagate() {
    let src = "\
fn parse(s: string) -> int {
    if s == \"bad\" {
        throw \"not a number: ${s}\"
    }
    return 42
}

fn wrap(s: string) -> int {
    return parse(s) + 1
}

print(parse(\"ok\"))
v = parse(\"bad\") catch e {
    print(\"caught:\", e.message)
    -1
}
print(v)
w = wrap(\"bad\") catch e {
    print(\"wrapped:\", e.message)
    0
}
print(w)
";
    check(
        "errors",
        src,
        "42\ncaught: not a number: bad\n-1\nwrapped: not a number: bad\n0",
    );
}

#[test]
fn early_returns_release_their_strings() {
    let src = "\
fn label(n: int) -> string {
    if n % 2 == 0 {
        return \"even ${n}\"
    }
    s = \"odd ${n}\"
    return s
}

i = 0
while i < 2000 {
    t = label(i)
    if len(t) == 0 {
        print(\"impossible\")
    }
    i += 1
}
print(label(7), label(8))
";
    check("earlyreturn", src, "odd 7 even 8");
}

#[test]
fn uncaught_errors_stop_with_the_message() {
    if !have_clang() {
        eprintln!("skipping `uncaught`: no clang on PATH");
        return;
    }
    let r = run(
        "uncaught",
        "\
fn boom() -> int {
    throw \"kaboom\"
}
print(boom())
",
    );
    assert!(!r.ok, "an uncaught error must fail the program");
    assert!(
        r.stderr.contains("kaboom"),
        "the runtime's error message should reach stderr: {}",
        r.stderr
    );
}

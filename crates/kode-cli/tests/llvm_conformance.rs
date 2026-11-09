//! Conformance of the shipped compiler (LLVM is the only backend).
//!
//! Every program is built through the real `kode` binary and run with
//! `KODE_LEAK_CHECK=1` under a timeout: run-pass programs must match their
//! checked-in `.out`, examples must run, and compile-fail programs must report
//! the code in their `.err` file. The backend flag is covered too: `--backend=llvm`
//! is an accepted no-op and `--backend=cranelift` fails with a clear message
//! rather than silently choosing something else.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_kode");
const TIMEOUT: Duration = Duration::from_secs(30);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn have_clang() -> bool {
    Command::new(
        std::env::var("KODE_CLANG")
            .or_else(|_| std::env::var("CLANG"))
            .unwrap_or_else(|_| "clang".into()),
    )
    .arg("--version")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

struct Ran {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_with_timeout(exe: &Path, args: &[String], cwd: &Path) -> Ran {
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .env("KODE_LEAK_CHECK", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the built program");
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait().expect("poll the program") {
            Some(_) => break,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("program {:?} did not finish within {TIMEOUT:?}", exe);
            }
            None => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    let out: Output = child.wait_with_output().expect("collect output");
    Ran {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn build(src: &Path, exe: &Path, extra: &[&str], tag: &str) -> Output {
    let mut cmd = Command::new(BIN);
    cmd.arg("build")
        .arg(src)
        .arg("-o")
        .arg(exe)
        .args(extra)
        .current_dir(root());
    let out = cmd.output().expect("run kode");
    assert!(
        out.status.success(),
        "{tag}: build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn compare(src: &Path, expected: Option<&str>, args: &[String]) {
    let name = src.file_stem().unwrap().to_string_lossy().replace('.', "_");
    let dir = std::env::temp_dir().join(format!(
        "kode-cli-conformance-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let exe = dir.join("program");
    build(src, &exe, &[], &name);
    let r = run_with_timeout(&exe, args, &root());
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        r.code == Some(0),
        "{}: exit {:?}, stderr:\n{}",
        src.display(),
        r.code,
        r.stderr
    );
    assert!(
        r.stderr.trim().is_empty(),
        "{}: leaked or reported a runtime problem:\n{}",
        src.display(),
        r.stderr
    );
    if let Some(want) = expected {
        assert_eq!(
            r.stdout.trim_end(),
            want.trim_end(),
            "{}: output does not match the checked-in .out file",
            src.display()
        );
    }
}

fn run_pass() -> Vec<PathBuf> {
    let dir = root().join("tests/run-pass");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/run-pass")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "kode"))
        .filter(|p| p.with_extension("out").is_file())
        .collect();
    files.sort();
    files
}

fn batch(name: &str, batch_index: usize, count: usize) {
    if !have_clang() {
        eprintln!("skipping {name}: no clang on PATH");
        return;
    }
    let files = run_pass();
    let start = batch_index * count;
    let end = (start + count).min(files.len());
    assert!(
        start < files.len(),
        "batch {batch_index} has nothing to run ({} files)",
        files.len()
    );
    for src in &files[start..end] {
        let expected = std::fs::read_to_string(src.with_extension("out")).expect(".out file");
        compare(src, Some(&expected), &[]);
    }
}

#[test]
fn run_pass_matches_expected_output_1() {
    batch("run-pass 1", 0, 15);
}

#[test]
fn run_pass_matches_expected_output_2() {
    batch("run-pass 2", 1, 15);
}

#[test]
fn run_pass_matches_expected_output_3() {
    batch("run-pass 3", 2, 15);
}

#[test]
fn examples_run() {
    if !have_clang() {
        eprintln!("skipping examples: no clang on PATH");
        return;
    }
    let examples = [
        "bank",
        "config",
        "fib",
        "hello",
        "pipeline",
        "report",
        "tour",
        "wordcount",
    ];
    for name in examples {
        // wordcount reads the file named in its first argument.
        let args: Vec<String> = if name == "wordcount" {
            vec!["README.md".to_string()]
        } else {
            Vec::new()
        };
        compare(&root().join(format!("examples/{name}.kode")), None, &args);
    }
}

#[test]
fn compile_fail_diagnostics_match_the_checked_in_codes() {
    // Mirrors `crates/kode-cli/tests/conformance.rs`: `check --strict` so that
    // warning-only fixtures (W codes) are rejected too.
    let dir = root().join("tests/compile-fail");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/compile-fail")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "kode"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no compile-fail programs found");
    for src in files {
        let expected = std::fs::read_to_string(src.with_extension("err"))
            .unwrap_or_else(|_| panic!("{}: missing .err file", src.display()))
            .trim()
            .to_string();
        let out = Command::new(BIN)
            .args([
                "check",
                "--strict",
                "--format=json",
                &src.display().to_string(),
            ])
            .current_dir(root())
            .output()
            .expect("run kode");
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(
            !out.status.success(),
            "{}: expected {expected} but it compiled",
            src.display()
        );
        assert!(
            stdout.contains(&format!("\"code\":\"{expected}\"")),
            "{}: expected {expected}, got:\n{stdout}",
            src.display()
        );
    }
}

#[test]
fn the_only_backend_is_llvm() {
    let exe = std::env::temp_dir().join(format!("kode-cli-backend-{}.exe", std::process::id()));
    // `--backend=llvm` is an accepted compatibility alias.
    build(
        &root().join("examples/hello.kode"),
        &exe,
        &["--backend=llvm"],
        "llvm alias",
    );
    let out = Command::new(&exe)
        .env("KODE_LEAK_CHECK", "1")
        .output()
        .expect("run hello");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello world");
    let _ = std::fs::remove_file(&exe);

    // The removed backend is a clear error, not a fallback.
    let out = Command::new(BIN)
        .args(["build", "examples/hello.kode", "--backend=cranelift"])
        .current_dir(root())
        .output()
        .expect("run kode");
    assert_eq!(out.status.code(), Some(2));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("the Cranelift backend was removed"),
        "expected a clear removal message, got:\n{text}"
    );

    let out = Command::new(BIN)
        .args(["build", "examples/hello.kode", "--backend=wat"])
        .current_dir(root())
        .output()
        .expect("run kode");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("LLVM is the only backend"),
        "expected the unknown-backend message"
    );
}

/// Without clang the compiler must stop with an actionable message, leave no
/// output behind, and still say which program is missing.
#[test]
fn a_missing_clang_is_actionable_and_leaves_no_artifact() {
    let exe = std::env::temp_dir().join(format!("kode-cli-clang-{}.exe", std::process::id()));
    let missing = "/nonexistent/kode-cli-test-clang";
    let out = Command::new(BIN)
        .args([
            "build",
            "examples/hello.kode",
            "-o",
            &exe.display().to_string(),
        ])
        .env("KODE_CLANG", missing)
        .current_dir(root())
        .output()
        .expect("run kode");
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("the LLVM backend needs") && text.contains(missing),
        "got:\n{text}"
    );
    assert!(!exe.exists(), "a failed build must not leave an output");
}

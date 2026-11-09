//! B6: the LLVM backend must reproduce the Cranelift backend exactly.
//!
//! Every program is built twice through the real `kode` binary — once with
//! `--backend=cranelift` (the default) and once with `--backend=llvm` — then run
//! with `KODE_LEAK_CHECK=1` under a timeout. Exit status, stdout and stderr must
//! match, and the LLVM run must match the checked-in `.out` file.
//!
//! These tests are integration tests of the CLI on purpose: they exercise the
//! same import/package loading path users get, which is how `modules_alias.kode`
//! is covered.

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

fn build(backend: &str, src: &Path, exe: &Path, tag: &str) -> Result<(), String> {
    let out = Command::new(BIN)
        .args([
            "build",
            &src.display().to_string(),
            "-o",
            &exe.display().to_string(),
            &format!("--backend={backend}"),
        ])
        .current_dir(root())
        .output()
        .expect("run kode");
    if out.status.success() {
        return Ok(());
    }
    Err(format!(
        "{tag}: `{backend}` build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    ))
}

/// Build and run `src` on both backends; return the two runs for comparison.
fn both(src: &Path, name: &str) -> (Ran, Ran) {
    let dir = std::env::temp_dir().join(format!(
        "kode-cli-conformance-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let cr = dir.join("cranelift");
    let ll = dir.join("llvm");
    if let Err(e) = build("cranelift", src, &cr, name) {
        panic!("{e}");
    }
    if let Err(e) = build("llvm", src, &ll, name) {
        panic!("{e}");
    }
    let a = run_with_timeout(&cr, &[], &root());
    let b = run_with_timeout(&ll, &[], &root());
    let _ = std::fs::remove_dir_all(&dir);
    (a, b)
}

fn compare(src: &Path, expected: Option<&str>) {
    let name = src.file_stem().unwrap().to_string_lossy().replace('.', "_");
    let (cr, ll) = both(src, &name);
    assert_eq!(
        ll.code,
        cr.code,
        "{}: exit status differs (cranelift {:?}, llvm {:?})\nllvm stderr:\n{}",
        src.display(),
        cr.code,
        ll.code,
        ll.stderr
    );
    assert_eq!(
        ll.stdout,
        cr.stdout,
        "{}: stdout differs\n--- cranelift ---\n{}\n--- llvm ---\n{}",
        src.display(),
        cr.stdout,
        ll.stdout
    );
    assert_eq!(
        ll.stderr,
        cr.stderr,
        "{}: stderr differs\n--- cranelift ---\n{}\n--- llvm ---\n{}",
        src.display(),
        cr.stderr,
        ll.stderr
    );
    if let Some(want) = expected {
        assert_eq!(
            ll.stdout.trim_end(),
            want.trim_end(),
            "{}: LLVM output does not match the checked-in .out file",
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
        compare(src, Some(&expected));
    }
}

#[test]
fn run_pass_matches_cranelift_1() {
    batch("run-pass 1", 0, 15);
}

#[test]
fn run_pass_matches_cranelift_2() {
    batch("run-pass 2", 1, 15);
}

#[test]
fn run_pass_matches_cranelift_3() {
    batch("run-pass 3", 2, 15);
}

#[test]
fn examples_match_cranelift() {
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
        let src = root().join(format!("examples/{name}.kode"));
        let (cr, ll) = both(&src, &format!("example_{name}"));
        assert_eq!(ll.code, cr.code, "examples/{name}.kode: exit status");
        assert_eq!(
            ll.stdout, cr.stdout,
            "examples/{name}.kode: stdout\n--- llvm ---\n{}",
            ll.stdout
        );
        assert_eq!(
            ll.stderr, cr.stderr,
            "examples/{name}.kode: stderr\n--- llvm ---\n{}",
            ll.stderr
        );
    }
}

#[test]
fn compile_fail_diagnostics_are_identical() {
    // Mirrors `crates/kode-cli/tests/conformance.rs`: `check --strict` so that
    // warning-only fixtures (W codes) are rejected too, and the expected code
    // comes from the checked-in `.err` file.
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
        let mut outs = Vec::new();
        for backend in ["cranelift", "llvm"] {
            let out = Command::new(BIN)
                .args([
                    "check",
                    "--strict",
                    "--format=json",
                    &src.display().to_string(),
                    &format!("--backend={backend}"),
                ])
                .current_dir(root())
                .output()
                .expect("run kode");
            outs.push(out);
        }
        let (cr, ll) = (&outs[0], &outs[1]);
        let cr_out = String::from_utf8_lossy(&cr.stdout).to_string();
        let ll_out = String::from_utf8_lossy(&ll.stdout).to_string();
        assert_eq!(
            ll.status.code(),
            cr.status.code(),
            "{}: exit status differs between backends",
            src.display()
        );
        assert!(
            !cr.status.success(),
            "{}: expected {expected} but it compiled",
            src.display()
        );
        assert!(
            cr_out.contains(&format!("\"code\":\"{expected}\"")),
            "{}: expected {expected}, got:\n{cr_out}",
            src.display()
        );
        assert_eq!(
            ll_out,
            cr_out,
            "{}: diagnostics differ between backends",
            src.display()
        );
    }
}

#[test]
fn an_unknown_backend_is_rejected() {
    let out = Command::new(BIN)
        .args(["build", "examples/hello.kode", "--backend=wat"])
        .current_dir(root())
        .output()
        .expect("run kode");
    assert_eq!(out.status.code(), Some(2));
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("unknown backend `wat`"),
        "expected a clear message, got:\n{text}"
    );
}

/// The default is LLVM, proven by behaviour: with clang unreachable, a default
/// build fails with the LLVM message, while `--backend=cranelift` still builds.
#[test]
fn the_default_backend_is_llvm_and_cranelift_is_the_escape_hatch() {
    let exe = std::env::temp_dir().join(format!("kode-cli-default-{}.exe", std::process::id()));
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
    assert_eq!(out.status.code(), Some(1), "default build with no clang");
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("the LLVM backend needs") && text.contains(missing),
        "the default must be LLVM and say what is missing, got:\n{text}"
    );
    assert!(
        !exe.exists(),
        "a failed LLVM build must not leave an output behind"
    );
    let out = Command::new(BIN)
        .args([
            "build",
            "examples/hello.kode",
            "-o",
            &exe.display().to_string(),
            "--backend=cranelift",
        ])
        .env("KODE_CLANG", missing)
        .current_dir(root())
        .output()
        .expect("run kode");
    assert!(
        out.status.success(),
        "explicit cranelift must not need clang:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_file(&exe);
}

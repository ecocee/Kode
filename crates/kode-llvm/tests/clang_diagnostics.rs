//! What the clang and linker stage reports when it cannot do its job.

use std::path::PathBuf;

fn have_clang() -> bool {
    std::process::Command::new(kode_llvm::clang_program())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kode-llvm-diag-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Text that is not LLVM IR must be rejected, and neither the `.ll` text nor a
/// half-written object may be left behind.
#[test]
fn invalid_ir_is_reported_and_cleaned_up() {
    if !have_clang() {
        eprintln!("skipping: no clang on PATH");
        return;
    }
    let dir = scratch("invalid-ir");
    let object = dir.join("bad.o");
    match kode_llvm::compile_ir("this is not LLVM IR\n", &object) {
        Err(kode_llvm::ClangError::Failed { stderr, .. }) => {
            assert!(
                stderr.to_lowercase().contains("error"),
                "expected clang's diagnostic, got: {stderr}"
            );
        }
        other => panic!("expected ClangError::Failed, got {other:?}"),
    }
    assert!(
        !object.exists(),
        "a rejected compile must not leave an object"
    );
    assert!(
        !dir.join("bad.ll").exists(),
        "a rejected compile must not leave the IR text"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `KODE_CLANG` pointing nowhere is a start failure with an actionable message.
#[test]
fn a_missing_clang_is_reported_before_any_work() {
    let dir = scratch("missing-clang");
    let object = dir.join("out.o");
    // The variable is process-wide, so this test owns the process for its duration.
    std::env::set_var("KODE_CLANG", "/nonexistent/kode-llvm-test-clang");
    let result = kode_llvm::compile_ir("define void @f() { ret void }\n", &object);
    std::env::remove_var("KODE_CLANG");
    match result {
        Err(kode_llvm::ClangError::NotFound { program, .. }) => {
            assert!(program.contains("kode-llvm-test-clang"), "got {program}");
        }
        other => panic!("expected ClangError::NotFound, got {other:?}"),
    }
    assert!(!object.exists());
    assert!(!dir.join("out.ll").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

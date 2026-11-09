//! Object files from textual IR, through `clang`.
//!
//! No LLVM libraries are linked into the compiler: the `.ll` text is written
//! next to the object and `clang` lowers it. A missing `clang` is a clear,
//! actionable error, never a silent fallback to another backend.

use std::fmt;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum ClangError {
    /// The program could not be started at all.
    NotFound { program: String, error: String },
    /// The program ran and rejected the IR; the message is its stderr.
    Failed {
        program: String,
        status: String,
        stderr: String,
    },
    /// The IR or the object file could not be written.
    Io(String),
}

impl fmt::Display for ClangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClangError::NotFound { program, error } => write!(
                f,
                "the LLVM backend needs `{program}` on PATH, but it could not be run ({error}); \
                 install LLVM (clang 15 or newer) or set KODE_CLANG to the compiler's path"
            ),
            ClangError::Failed {
                program,
                status,
                stderr,
            } => write!(
                f,
                "`{program}` rejected the generated LLVM IR ({status}){}",
                if stderr.trim().is_empty() {
                    String::new()
                } else {
                    format!(":\n{}", stderr.trim_end())
                }
            ),
            ClangError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ClangError {}

/// `KODE_CLANG`, then `CLANG`, then `clang`.
pub fn clang_program() -> String {
    std::env::var("KODE_CLANG")
        .or_else(|_| std::env::var("CLANG"))
        .unwrap_or_else(|_| "clang".to_string())
}

/// Compile IR text into an object file at `object`.
pub fn compile_ir(ir: &str, object: &Path) -> Result<(), ClangError> {
    let ll = object.with_extension("ll");
    std::fs::write(&ll, ir)
        .map_err(|e| ClangError::Io(format!("cannot write {}: {e}", ll.display())))?;
    let program = clang_program();
    let output = Command::new(&program)
        .args(["-O2", "-fPIC", "-Wno-override-module", "-c"])
        .arg(&ll)
        .arg("-o")
        .arg(object)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            if std::env::var_os("KODE_LLVM_KEEP_IR").is_none() {
                let _ = std::fs::remove_file(&ll);
            }
            Ok(())
        }
        Ok(out) => {
            let _ = std::fs::remove_file(&ll);
            Err(ClangError::Failed {
                program,
                status: out.status.to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
        Err(e) => {
            let _ = std::fs::remove_file(&ll);
            Err(ClangError::NotFound {
                program,
                error: e.to_string(),
            })
        }
    }
}

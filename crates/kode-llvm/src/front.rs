//! Single-file front end: parse, check, lower to MIR, verify.
//!
//! The prototype deliberately does not duplicate `kode-cli`'s package loader:
//! `import`s and `kode.toml` resolution stay in the CLI, and programs that need
//! them are out of scope until this backend is integrated (ADR 0035, B6).

use std::path::Path;

use kode_mir::{LineMap, MirProgram};

pub struct Lines(String);

impl LineMap for Lines {
    fn line_of(&self, offset: usize) -> i64 {
        let bytes = self.0.as_bytes();
        let end = offset.min(bytes.len());
        bytes[..end].iter().filter(|b| **b == b'\n').count() as i64 + 1
    }
}

pub fn program_from_source(src: &str, name: &str) -> Result<MirProgram, String> {
    let program = kode_syntax::parse_source(src).map_err(|errs| {
        errs.iter()
            .map(|d| d.render(name, src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let checked = kode_sema::check(&program, &[]).map_err(|errs| {
        errs.iter()
            .map(|d| d.render(name, src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mir = kode_mir::lower(
        &program,
        &checked,
        &Lines(src.to_string()),
        kode_mir::Mode::Run,
    );
    kode_mir::verify::verify(&mir).map_err(|e| format!("internal compiler error: {e}"))?;
    Ok(mir)
}

pub fn load_program(path: &Path) -> Result<MirProgram, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    program_from_source(&src, &path.display().to_string())
}

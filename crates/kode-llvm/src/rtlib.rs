//! Finding the runtime archive the linker needs.
//!
//! The CLI embeds the runtime in the compiler binary; a standalone prototype
//! cannot, so it points at the archive Cargo already built. `KODE_RT_LIB`
//! wins, then the usual `target/<profile>` locations, newest first.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn find_runtime_archive(start: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("KODE_RT_LIB") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for profile in ["release", "debug"] {
        let dir = start.join("target").join(profile);
        let direct = dir.join("libkode_rt.a");
        if direct.is_file() {
            found.push((mtime(&direct), direct));
        }
        let build = dir.join("build");
        let Ok(entries) = std::fs::read_dir(&build) else {
            continue;
        };
        for e in entries.flatten() {
            if !e.file_name().to_string_lossy().starts_with("kode-rt-") {
                continue;
            }
            let lib = e.path().join("out").join("libkode_rt.a");
            if lib.is_file() {
                found.push((mtime(&lib), lib));
            }
        }
    }
    found.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    found.into_iter().next().map(|(_, p)| p)
}

fn mtime(p: &Path) -> SystemTime {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

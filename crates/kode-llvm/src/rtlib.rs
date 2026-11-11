//! Finding the runtime archive the linker needs.
//!
//! The CLI embeds the runtime in the compiler binary; a standalone prototype
//! cannot, so it points at the archive Cargo already built. `KODE_RT_LIB`
//! wins, then the newest archive under the usual `target/<profile>` locations.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn find_runtime_archive(start: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("KODE_RT_LIB") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    candidates(start).into_iter().next()
}

/// Every runtime archive under `target/`, newest first.
fn candidates(start: &Path) -> Vec<PathBuf> {
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for profile in ["release", "debug"] {
        let dir = start.join("target").join(profile);
        let direct = dir.join("libkode_rt.a");
        if direct.is_file() {
            found.push((mtime(&direct), direct));
        }
        if let Ok(entries) = std::fs::read_dir(dir.join("build")) {
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
        // `kode-rt` is a `staticlib` build dependency, and Cargo does not uplift
        // a build dependency's archive out of `deps/`. A tree that has only run
        // `cargo test` (CI's release jobs) therefore has no
        // `target/<profile>/libkode_rt.a` at all, only the hashed copy here.
        // Newest first, so a stale archive from an earlier hash cannot be
        // linked in place of the current runtime.
        if let Ok(entries) = std::fs::read_dir(dir.join("deps")) {
            for e in entries.flatten() {
                let name = e.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("libkode_rt-") && name.ends_with(".a") {
                    found.push((mtime(&e.path()), e.path()));
                }
            }
        }
    }
    found.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    found.into_iter().map(|(_, p)| p).collect()
}

fn mtime(p: &Path) -> SystemTime {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kode-rtlib-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// Writes an empty archive and gives it a fixed mtime, so "newest" is a fact
    /// about the test rather than about how fast the filesystem is.
    fn archive(root: &Path, rel: &str, secs: u64) -> PathBuf {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).expect("archive dir");
        let file = std::fs::File::create(&path).expect("archive");
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
            .expect("mtime");
        path
    }

    #[test]
    fn finds_the_copy_in_deps_when_nothing_was_uplifted() {
        // The clean-tree shape: `cargo test --workspace` only.
        let root = scratch("deps-only");
        let lib = archive(&root, "target/debug/deps/libkode_rt-2fef349a576e5779.a", 10);
        assert_eq!(candidates(&root), vec![lib.clone()]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn the_newest_archive_wins() {
        let root = scratch("newest");
        let old = archive(&root, "target/debug/deps/libkode_rt-aaaaaaaaaaaaaaaa.a", 10);
        let new = archive(&root, "target/debug/deps/libkode_rt-bbbbbbbbbbbbbbbb.a", 20);
        let lifted = archive(&root, "target/debug/libkode_rt.a", 5);
        assert_eq!(candidates(&root), vec![new, old, lifted]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn leaves_other_archives_alone() {
        let root = scratch("others");
        let lib = archive(
            &root,
            "target/release/deps/libkode_rt-1111111111111111.a",
            10,
        );
        archive(
            &root,
            "target/release/deps/libkode_json-2222222222222222.a",
            30,
        );
        archive(&root, "target/release/deps/kode_rt-3333333333333333.a", 40);
        assert_eq!(candidates(&root), vec![lib]);
        std::fs::remove_dir_all(&root).ok();
    }
}

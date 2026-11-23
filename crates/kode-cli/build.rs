//! Builds the Kode runtime library (with all of its dependencies) and embeds it in
//! the compiler binary, so the compiler and the runtime can never get out of sync and
//! `kode` works from any folder.
//!
//! `kode-rt` is a build dependency of this crate, so by the time this build script
//! runs `kode-rt` and all of its transitive dependencies (including `rustls` and
//! `webpki-roots`) have already been compiled by cargo. We locate the resulting
//! `libkode_rt.a` and copy it into `OUT_DIR` so it can be embedded with
//! `include_bytes!`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // `CARGO_WORKSPACE_WORKDIR` is the workspace root for crates that are members of a
    // workspace. Fall back to walking up from `CARGO_MANIFEST_DIR` only when that is not
    // set (for example when kode-cli is built outside the workspace).
    let workspace = env::var("CARGO_WORKSPACE_WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            manifest
                .parent()
                .expect("manifest dir has no parent")
                .parent()
                .expect("manifest dir has no grandparent")
                .to_path_buf()
        });

    let lib_name = "libkode_rt.a";
    let target_triple = env::var("TARGET").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin".to_string()
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin".to_string()
        } else {
            "host".to_string()
        }
    });

    // `CARGO_TARGET_DIR` overrides the workspace `target` folder, and the build
    // profile decides where the archive actually lands.
    let target_directory = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace.join("target"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let mut candidates = vec![
        target_directory
            .join(target_triple.as_str())
            .join(&profile)
            .join(lib_name),
        target_directory.join(&profile).join(lib_name),
    ];
    // A `staticlib` compiled as a build dependency is not uplifted out of
    // `deps/`, so `cargo build --release` on its own leaves only
    // `target/release/deps/libkode_rt-<hash>.a`. Look there too, newest first,
    // because a stale archive from an earlier hash must not win.
    for deps in [
        target_directory
            .join(target_triple.as_str())
            .join(&profile)
            .join("deps"),
        target_directory.join(&profile).join("deps"),
    ] {
        let mut found: Vec<PathBuf> = fs::read_dir(&deps)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.starts_with("libkode_rt") && n.ends_with(".a"))
                    })
                    .collect()
            })
            .unwrap_or_default();
        found.sort_by_key(|p| std::cmp::Reverse(fs::metadata(p).and_then(|m| m.modified()).ok()));
        candidates.extend(found);
    }

    // Newest wins, not first-found. `cargo build --release -p kode-rt` uplifts the
    // staticlib to `target/release/libkode_rt.a` and leaves it there; a later build
    // with a new hash writes `deps/libkode_rt-<hash>.a` and leaves the uplifted copy
    // behind. Picking the first candidate embedded that stale copy, so a program using
    // a function added since (`StringBuilder`) failed at *link* time with undefined
    // symbols while `kode -V` looked fine.
    let mut existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    existing.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());

    let lib_path = existing.pop().unwrap_or_else(|| {
        panic!(
            "could not find {} under {}",
            lib_name,
            target_directory.display()
        )
    });

    // An archive older than the runtime sources cannot contain what they declare.
    // Refuse to embed it instead of failing later with undefined symbols only when
    // a user builds a program that happens to call a new function.
    if let Some(newest_src) = newest_mtime(&workspace.join("kode-rt").join("src")) {
        if let Ok(archive_mtime) = fs::metadata(&lib_path).and_then(|m| m.modified()) {
            if archive_mtime < newest_src {
                panic!(
                    "{} is older than the runtime sources; run `cargo build -p kode-rt{}` first",
                    lib_path.display(),
                    if profile == "release" { " --release" } else { "" },
                );
            }
        }
    }

    fs::create_dir_all(&out_dir).expect("could not create OUT_DIR");
    fs::copy(lib_path, out_dir.join(lib_name)).expect("could not copy libkode_rt.a into OUT_DIR");

    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("kode-rt").join("src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("kode-rt").join("Cargo.toml").display()
    );
}

/// The newest modification time of any file under `dir`.
fn newest_mtime(dir: &Path) -> Option<std::time::SystemTime> {
    let mut newest = None;

    for entry in fs::read_dir(dir).ok()? {
        let path = entry.ok()?.path();
        let mtime = if path.is_dir() {
            newest_mtime(&path)
        } else {
            fs::metadata(&path).and_then(|m| m.modified()).ok()
        };

        if let Some(t) = mtime {
            newest = Some(match newest {
                Some(n) if n > t => n,
                _ => t,
            });
        }
    }

    newest
}

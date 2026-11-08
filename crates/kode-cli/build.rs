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
use std::path::PathBuf;

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

    let target_directory = workspace.join("target");
    let candidates = [
        target_directory.join(&target_triple).join("debug").join(&lib_name),
        target_directory.join("debug").join(&lib_name),
        target_directory.join(&target_triple).join("release").join(&lib_name),
        target_directory.join("release").join(&lib_name),
    ];

    let lib_path = candidates
        .iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| panic!("could not find {} under {}", lib_name, target_directory.display()));

    fs::create_dir_all(&out_dir).expect("could not create OUT_DIR");
    fs::copy(&lib_path, out_dir.join(&lib_name)).expect("could not copy libkode_rt.a into OUT_DIR");

    println!("cargo:rerun-if-changed={}", workspace.join("kode-rt").join("src").display());
    println!("cargo:rerun-if-changed={}", workspace.join("kode-rt").join("Cargo.toml").display());
}

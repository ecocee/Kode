//! Builds the Kode runtime library and embeds it in the compiler binary,
//! so the compiler and the runtime can never get out of sync and `kode`
//! works from any folder.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let rt_src = manifest.join("../kode-rt/src");
    println!("cargo:rerun-if-changed={}", rt_src.display());
    println!("cargo:rerun-if-changed=build.rs");

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let lib = out.join("libkode_rt.a");
    let status = Command::new(rustc)
        .args([
            "--crate-name",
            "kode_rt",
            "--crate-type",
            "staticlib",
            "--edition",
            "2021",
            "-C",
            "opt-level=3",
            "-C",
            "panic=abort",
            "-C",
            "codegen-units=1",
            "-C",
            "debuginfo=0",
        ])
        .arg(rt_src.join("lib.rs"))
        .arg("-o")
        .arg(&lib)
        .status()
        .expect("could not run rustc to build the Kode runtime");
    assert!(status.success(), "building the Kode runtime library failed");
}
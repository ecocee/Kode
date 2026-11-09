//! `kode-llvm`: compile a single-file Kode program through the experimental
//! LLVM backend.
//!
//! This is a prototype driver, not part of the `kode` CLI: nothing in the
//! production path changes at B1. It exists so the backend can be exercised
//! against real programs without touching `kode-cli`.
//!
//! ```text
//! kode-llvm emit  file.kode [-o file.ll]   # print or write the IR
//! kode-llvm build file.kode [-o program]   # IR -> object -> linked binary
//! kode-llvm run   file.kode [args...]      # build in a temp dir and run it
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("kode-llvm: {message}");
            1
        }
    };
    std::process::exit(code);
}

fn run(args: &[String]) -> Result<i32, String> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "" | "-h" | "--help" | "help" => {
            println!("usage: kode-llvm <emit|build|run> <file.kode> [-o path] [-- program args]");
            return Ok(0);
        }
        "emit" | "build" | "run" => {}
        other => return Err(format!("unknown command `{other}`; try `kode-llvm help`")),
    }
    let rest = &args[1..];
    let file = rest
        .iter()
        .find(|a| !a.starts_with('-'))
        .ok_or_else(|| format!("`kode-llvm {command}` needs a .kode file"))?;
    let output = value_of(rest, "-o").map(PathBuf::from);
    let program_args: Vec<String> = match rest.iter().position(|a| a == "--") {
        Some(i) => rest[i + 1..].to_vec(),
        None => Vec::new(),
    };

    let mir = kode_llvm::load_program(Path::new(file))?;
    let ir = kode_llvm::emit(&mir)?;

    match command {
        "emit" => {
            match output {
                Some(path) => std::fs::write(&path, &ir)
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?,
                None => print!("{ir}"),
            }
            Ok(0)
        }
        "build" => {
            let out = output.unwrap_or_else(|| {
                Path::new(file)
                    .file_stem()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("a.out"))
            });
            build(&ir, &out, true)?;
            eprintln!("built {}", out.display());
            Ok(0)
        }
        "run" => {
            let dir = std::env::temp_dir().join(format!("kode-llvm-{}", std::process::id()));
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let exe = dir.join("program");
            let result = build(&ir, &exe, false).and_then(|_| {
                Command::new(&exe)
                    .args(&program_args)
                    .status()
                    .map_err(|e| format!("cannot run {}: {e}", exe.display()))
            });
            let _ = std::fs::remove_dir_all(&dir);
            Ok(result?.code().unwrap_or(1))
        }
        _ => unreachable!(),
    }
}

fn value_of(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn build(ir: &str, out: &Path, strip: bool) -> Result<(), String> {
    let object = out.with_extension("kode.o");
    kode_llvm::compile_ir(ir, &object).map_err(|e| e.to_string())?;
    let bytes = std::fs::read(&object).map_err(|e| format!("cannot read the object: {e}"))?;
    let _ = std::fs::remove_file(&object);
    let root = workspace_root();
    let rt = kode_llvm::find_runtime_archive(&root).ok_or_else(|| {
        "cannot find libkode_rt.a; run `cargo build --workspace` first or set KODE_RT_LIB"
            .to_string()
    })?;
    kode_codegen::link(&bytes, out, &rt, strip)
}

fn workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("crates").is_dir() {
            return dir;
        }
        if !dir.pop() {
            return PathBuf::from(".");
        }
    }
}

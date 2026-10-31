//! The `kode` command line tool.

mod args;
mod report;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use args::{Options, HELP};
use report::report;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let opts = match args::parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("kode: {e}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    match opts.command.as_str() {
        "" | "help" => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        "version" => {
            println!("kode {VERSION}");
            ExitCode::SUCCESS
        }
        "run" | "build" | "check" | "tokens" | "ast" | "mir" => match run_command(&opts) {
            Ok(code) => ExitCode::from(code),
            Err(msg) => {
                eprintln!("kode: {msg}");
                ExitCode::from(1)
            }
        },
        other => {
            eprintln!("kode: unknown command `{other}`\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn run_command(opts: &Options) -> Result<u8, String> {
    let file = opts
        .file
        .as_deref()
        .ok_or_else(|| format!("`kode {}` needs a file name", opts.command))?;
    let src = std::fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?;

    if opts.command == "tokens" {
        return match kode_syntax::lexer::lex(&src) {
            Ok(tokens) => {
                for t in tokens {
                    println!("{:?}  [{}..{}]", t.kind, t.span.start, t.span.end);
                }
                Ok(0)
            }
            Err(errs) => {
                report(opts.json, file, &src, &errs);
                Ok(1)
            }
        };
    }

    let program = match kode_syntax::parse_source(&src) {
        Ok(p) => p,
        Err(errs) => {
            report(opts.json, file, &src, &errs);
            return Ok(1);
        }
    };
    if opts.command == "ast" {
        println!("{program:#?}");
        return Ok(0);
    }

    let checked = match kode_sema::check(&program) {
        Ok(c) => c,
        Err(errs) => {
            report(opts.json, file, &src, &errs);
            return Ok(1);
        }
    };
    report(opts.json, file, &src, &checked.warnings);
    if opts.strict && !checked.warnings.is_empty() {
        return Ok(1);
    }
    if opts.command == "check" {
        if !opts.json {
            eprintln!("ok: {file} has no errors");
        }
        return Ok(0);
    }

    let mir = kode_mir::lower(&program, &checked, &src);
    kode_mir::verify::verify(&mir).map_err(|e| format!("internal compiler error: {e}"))?;
    if opts.command == "mir" {
        print!("{}", kode_mir::dump::dump(&mir));
        return Ok(0);
    }

    let object = kode_codegen::compile_to_object(&mir, opts.release)?;
    let rt = runtime_lib()?;

    if opts.command == "build" {
        let out = match &opts.output {
            Some(o) => PathBuf::from(o),
            None => PathBuf::from(
                Path::new(file)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "a".to_string()),
            ),
        };
        kode_codegen::link(&object, &out, &rt, opts.release)?;
        eprintln!("built {}", out.display());
        return Ok(0);
    }

    // run: build into a temporary folder, execute, clean up
    let dir = std::env::temp_dir().join(format!("kode-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let exe = dir.join("program");
    let result = kode_codegen::link(&object, &exe, &rt, false).and_then(|_| {
        Command::new(&exe)
            .status()
            .map_err(|e| format!("cannot run the program: {e}"))
    });
    let _ = std::fs::remove_dir_all(&dir);
    Ok(result?.code().unwrap_or(1) as u8)
}

/// Find the runtime library (`libkode_rt.a`).
/// Order: the KODE_RT_LIB variable, next to the `kode` binary, then the
/// `target` folder of this project (so a copied `kode` binary still works).
fn runtime_lib() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("KODE_RT_LIB") {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let beside = exe.parent().unwrap().join("libkode_rt.a");
    if beside.exists() {
        return Ok(beside);
    }
    for profile in ["debug", "release"] {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(profile)
            .join("libkode_rt.a");
        if p.exists() {
            return Ok(p);
        }
    }
    Err(format!(
        "runtime library not found next to {}; run `cargo build --workspace` first",
        exe.display()
    ))
}

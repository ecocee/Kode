//! The `kode` command line tool.

mod args;
mod doc;
mod doctor;
mod driver;
mod explain;
mod fmt;
mod lsp;
mod manifest;
mod packages;
mod report;
mod resolve;
mod source;
mod style;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use args::{Options, HELP};
use kode_mir::Mode;
use kode_sema::Checked;
use kode_syntax::ast::{Item, Program};
use report::report;
use source::SourceMap;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The runtime library, built by `build.rs` and embedded in this binary.
static RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/libkode_rt.a"));

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
        "doctor" => finish(doctor::run()),
        "doc" => finish(doc::run(&opts)),
        "lsp" => finish(lsp::serve()),
        "init" => finish(packages::init(&opts)),
        "new" => finish(packages::new_package(&opts)),
        "add" => finish(packages::add(&opts)),
        "remove" => finish(packages::remove(&opts)),
        "install" => finish(packages::install(&opts)),
        "run" | "build" | "check" | "tokens" | "ast" | "mir" | "test" | "fmt" | "explain" => {
            match run_command(&opts) {
                Ok(code) => ExitCode::from(code),
                Err(msg) => {
                    eprintln!("kode: {msg}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!("kode: unknown command `{other}`\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn finish(result: Result<u8, String>) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code),
        Err(msg) => {
            eprintln!("kode: {msg}");
            ExitCode::from(1)
        }
    }
}

struct Front {
    program: Program,
    checked: Checked,
    map: SourceMap,
}

/// Load, parse and check a program (and its imports). `Err` carries the exit code.
fn front_end(opts: &Options, file: &str) -> Result<Front, u8> {
    let loaded = match driver::load(Path::new(file)) {
        Ok(l) => l,
        Err((map, errs)) => {
            report(opts.json, &map, &errs);
            return Err(1);
        }
    };
    let checked = match kode_sema::check(&loaded.program, &loaded.modules) {
        Ok(c) => c,
        Err(errs) => {
            report(opts.json, &loaded.map, &errs);
            return Err(1);
        }
    };
    report(opts.json, &loaded.map, &checked.warnings);
    if opts.strict && !checked.warnings.is_empty() {
        return Err(1);
    }
    Ok(Front {
        program: loaded.program,
        checked,
        map: loaded.map,
    })
}

fn object_for(opts: &Options, f: &Front, mode: Mode) -> Result<Vec<u8>, String> {
    let mut mir = kode_mir::lower(&f.program, &f.checked, &f.map, mode);
    if !opts.debug {
        kode_mir::opt::optimize(&mut mir);
    }
    kode_mir::verify::verify(&mir).map_err(|e| format!("internal compiler error: {e}"))?;
    llvm_object(&mir)
}

/// Lower MIR to textual LLVM IR, then ask clang for an object file.
/// A missing clang is an error, never a quiet fallback to another code path.
fn llvm_object(mir: &kode_mir::ir::MirProgram) -> Result<Vec<u8>, String> {
    let ir = kode_llvm::emit(mir)?;
    let dir = std::env::temp_dir().join(format!("kode-llvm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make a temporary folder: {e}"))?;
    let object = dir.join("program.o");
    let result = kode_llvm::compile_ir(&ir, &object)
        .map_err(|e| e.to_string())
        .and_then(|_| {
            std::fs::read(&object).map_err(|e| format!("cannot read the object file: {e}"))
        });
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Write the embedded runtime library to `dir` and return its path.
/// `KODE_RT_LIB` can point at another runtime (for compiler development).
fn runtime_lib(dir: &Path) -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("KODE_RT_LIB") {
        return Ok(PathBuf::from(p));
    }
    let path = dir.join("libkode_rt.a");
    std::fs::write(&path, RUNTIME).map_err(|e| format!("cannot write the runtime library: {e}"))?;
    Ok(path)
}

/// Link into a temporary folder, run with the program arguments, clean up.
/// Returns the program's exit code.
fn run_object(object: &[u8], tag: &str, prog_args: &[String]) -> Result<u8, String> {
    let dir = std::env::temp_dir().join(format!("kode-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let exe = dir.join("program");
    let result = runtime_lib(&dir)
        .and_then(|rt| kode_link::link(object, &exe, &rt, false))
        .and_then(|_| {
            Command::new(&exe)
                .args(prog_args)
                .status()
                .map_err(|e| format!("cannot run the program: {e}"))
        });
    let _ = std::fs::remove_dir_all(&dir);
    Ok(result?.code().unwrap_or(1) as u8)
}

fn run_command(opts: &Options) -> Result<u8, String> {
    match opts.command.as_str() {
        "explain" => return explain_command(opts),
        "fmt" => return fmt_command(opts),
        "test" => return test_command(opts),
        _ => {}
    }

    let file = opts
        .file()
        .ok_or_else(|| format!("`kode {}` needs a file name", opts.command))?;

    if opts.command == "tokens" || opts.command == "ast" {
        let src = std::fs::read_to_string(file).map_err(|e| format!("cannot read {file}: {e}"))?;
        let mut map = SourceMap::default();
        map.add(file.to_string(), src.clone());
        if opts.command == "tokens" {
            return match kode_syntax::lexer::lex(&src) {
                Ok(tokens) => {
                    for t in tokens {
                        println!("{:?}  [{}..{}]", t.kind, t.span.start, t.span.end);
                    }
                    Ok(0)
                }
                Err(errs) => {
                    report(opts.json, &map, &errs);
                    Ok(1)
                }
            };
        }
        return match kode_syntax::parse_source(&src) {
            Ok(p) => {
                println!("{p:#?}");
                Ok(0)
            }
            Err(errs) => {
                report(opts.json, &map, &errs);
                Ok(1)
            }
        };
    }

    let started = std::time::Instant::now();
    let front = match front_end(opts, file) {
        Ok(f) => f,
        Err(code) => return Ok(code),
    };
    let elapsed = started.elapsed();
    if opts.command == "check" {
        if !opts.json {
            let on = style::on_stderr();
            eprintln!("{} {file} has no errors", style::green(on, "ok:"));
            eprintln!(
                "{}",
                style::dim(on, &format!("checked in {}", style::millis(elapsed)))
            );
        }
        return Ok(0);
    }
    if opts.command == "mir" {
        let mir = kode_mir::lower(&front.program, &front.checked, &front.map, Mode::Run);
        kode_mir::verify::verify(&mir).map_err(|e| format!("internal compiler error: {e}"))?;
        print!("{}", kode_mir::dump::dump(&mir));
        return Ok(0);
    }

    let object = object_for(opts, &front, Mode::Run)?;
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
        let tmp = std::env::temp_dir().join(format!("kode-build-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
        let linked =
            runtime_lib(&tmp).and_then(|rt| kode_link::link(&object, &out, &rt, opts.release));
        let _ = std::fs::remove_dir_all(&tmp);
        linked?;
        let on = style::on_stderr();
        eprintln!("{} {}", style::green(on, "built"), out.display());
        eprintln!(
            "{}",
            style::dim(
                on,
                &format!("linked in {}", style::millis(started.elapsed()))
            )
        );
        return Ok(0);
    }
    run_object(&object, "run", &opts.program_args)
}

// ---- kode test ---------------------------------------------------------

fn test_command(opts: &Options) -> Result<u8, String> {
    let paths: Vec<String> = if opts.files.is_empty() {
        vec![".".into()]
    } else {
        opts.files.clone()
    };
    let explicit_file = paths.len() == 1 && Path::new(&paths[0]).is_file();
    let files = collect_kode_files(&paths)?;
    let multi = files.len() > 1;
    let mut failed = false;
    let mut ran = 0;

    for (n, file) in files.iter().enumerate() {
        let file_s = file.display().to_string();
        let front = match front_end(opts, &file_s) {
            Ok(f) => f,
            Err(_) => {
                failed = true;
                continue;
            }
        };
        if !front
            .program
            .items
            .iter()
            .any(|i| matches!(i, Item::Test(_)))
        {
            if explicit_file {
                eprintln!("kode: no `test` blocks in {file_s}");
            }
            continue;
        }
        if multi {
            println!("== {file_s}");
        }
        ran += 1;
        let object = object_for(opts, &front, Mode::Test)?;
        let code = run_object(&object, &format!("test{n}"), &[])?;
        if code != 0 {
            failed = true;
        }
    }
    if ran == 0 && !explicit_file {
        eprintln!("kode: no `test` blocks found");
    }
    Ok(u8::from(failed))
}

// ---- kode fmt ----------------------------------------------------------

fn fmt_command(opts: &Options) -> Result<u8, String> {
    if opts.files.is_empty() {
        return Err("`kode fmt` needs a file or folder".into());
    }
    let mut changed = 0;
    for file in collect_kode_files(&opts.files)? {
        let src = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let out = fmt::format_source(&src);
        if out != src {
            changed += 1;
            if opts.check {
                println!("would reformat {}", file.display());
            } else {
                std::fs::write(&file, out)
                    .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
                println!("formatted {}", file.display());
            }
        }
    }
    Ok(u8::from(opts.check && changed > 0))
}

// ---- kode explain ------------------------------------------------------

fn explain_command(opts: &Options) -> Result<u8, String> {
    let code = opts
        .file()
        .ok_or("`kode explain` needs an error code, for example `kode explain E0201`")?;
    match explain::explain(code) {
        Some(text) => {
            print!("{text}");
            Ok(0)
        }
        None => Err(format!("unknown error code `{code}`")),
    }
}

/// Files given on the command line, plus every `.kode` file inside given folders.
fn collect_kode_files(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        for p in entries {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if p.is_dir() {
                if name != "target" && !name.starts_with('.') {
                    walk(&p, out)?;
                }
            } else if p.extension().map(|e| e == "kode").unwrap_or(false) {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for p in paths {
        let path = Path::new(p);
        if path.is_dir() {
            walk(path, &mut out)?;
        } else if path.is_file() {
            out.push(path.to_path_buf());
        } else {
            return Err(format!("cannot find {p}"));
        }
    }
    Ok(out)
}

//! `kode doctor`: is this machine able to build Kode programs?

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::style;

/// The ABI marker in the embedded runtime (`kode-rt/src/lib.rs`).
const RT_ABI: &str = "0004";

pub fn run() -> Result<u8, String> {
    let on = style::on_stdout();
    println!(
        "{}",
        style::bold(on, &format!("kode {} — toolchain check", crate::VERSION))
    );

    let cc = cc_program();
    let (cc_ok, cc_text) = describe_cc(&cc);
    let (pkg_ok, pkg_text) = package_state();
    let rows: Vec<(bool, &str, String)> = vec![
        (true, "compiler", format!("kode {}", crate::VERSION)),
        (
            true,
            "target",
            format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        ),
        (
            true,
            "runtime",
            format!("{} KB embedded, ABI {RT_ABI}", crate::RUNTIME.len() / 1024),
        ),
        (cc_ok, "linker", format!("{cc}: {cc_text}")),
        (pkg_ok, "package", pkg_text),
    ];
    for (ok, label, value) in &rows {
        let mark = if *ok {
            style::green(on, "ok")
        } else {
            style::red(on, "no")
        };
        println!("{mark} {label:>9}  {value}");
    }
    if !cc_ok {
        println!(
            "{}",
            style::dim(
                on,
                "install a C compiler: `xcode-select --install` (macOS), `apt install build-essential`\n\
                 (Debian/Ubuntu), `dnf groupinstall \"Development Tools\"` (Fedora) or MinGW-w64 (Windows)"
            )
        );
        return Ok(1);
    }
    // Any failed row is a failed check: an unmet compiler requirement or a stale
    // lockfile must show up in the exit code, not only in the text.
    if rows.iter().any(|(ok, _, _)| !ok) {
        return Ok(1);
    }
    Ok(0)
}

/// The linker driver: `KODE_CC`, then `CC`, then the system `cc`.
fn cc_program() -> String {
    std::env::var("KODE_CC")
        .or_else(|_| std::env::var("CC"))
        .unwrap_or_else(|_| "cc".to_string())
}

fn describe_cc(cc: &str) -> (bool, String) {
    let mut cmd = Command::new(cc);
    cmd.arg("--version");
    // Report a missing linker driver as "did not run", not as exit status 127.
    kode_link::force_fork_exec(&mut cmd);
    match cmd.output() {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let first = text.lines().next().unwrap_or("").trim().to_string();
            (true, first)
        }
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stderr);
            (
                false,
                text.lines()
                    .next()
                    .unwrap_or("it did not run")
                    .trim()
                    .to_string(),
            )
        }
        Err(e) => (false, format!("not found ({e})")),
    }
}

/// A package is optional, so "no manifest" is a healthy result, not a failure.
fn package_state() -> (bool, String) {
    let Some(manifest_path) = find_up("kode.toml") else {
        return (
            true,
            "no kode.toml here (fine for a single file)".to_string(),
        );
    };
    // A manifest this old cannot be read at all, so say why instead of claiming
    // the package is fine.
    let manifest = match crate::manifest::read(&manifest_path) {
        Ok(m) => m,
        Err(e) => return (false, format!("{}: {}", manifest_path.display(), e.message)),
    };
    // A package can state the compiler it needs. Failing here, before a build,
    // is the point: an older compiler may parse the source and still be wrong.
    if let Some(required) = manifest.kode.as_deref() {
        if !crate::manifest::version_meets(required, crate::VERSION) {
            return (
                false,
                format!(
                    "{}: needs kode {required} or newer, this is {}; upgrade the compiler",
                    manifest_path.display(),
                    crate::VERSION
                ),
            );
        }
    }
    let lock = manifest_path.with_file_name("kode.lock");
    if !lock.exists() {
        return (
            true,
            format!("{} (no kode.lock yet)", manifest_path.display()),
        );
    }
    let stale = match (modified(&manifest_path), modified(&lock)) {
        (Some(m), Some(l)) => m > l,
        _ => false,
    };
    if stale {
        (
            false,
            format!(
                "{}: kode.lock is older than kode.toml, run `kode install`",
                manifest_path.display()
            ),
        )
    } else {
        (
            true,
            format!("{} with a current kode.lock", manifest_path.display()),
        )
    }
}

fn find_up(name: &str) -> Option<PathBuf> {
    let mut dir: Option<&Path> = Some(Path::new("."));
    for _ in 0..6 {
        let d = dir?;
        let candidate = d.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

fn modified(p: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(p).ok()?.modified().ok()
}

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
    Ok(0)
}

/// The linker driver: `KODE_CC`, then `CC`, then the system `cc`.
fn cc_program() -> String {
    std::env::var("KODE_CC")
        .or_else(|_| std::env::var("CC"))
        .unwrap_or_else(|_| "cc".to_string())
}

fn describe_cc(cc: &str) -> (bool, String) {
    match Command::new(cc).arg("--version").output() {
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
    let Some(manifest) = find_up("kode.toml") else {
        return (
            true,
            "no kode.toml here (fine for a single file)".to_string(),
        );
    };
    let lock = manifest.with_file_name("kode.lock");
    if !lock.exists() {
        return (true, format!("{} (no kode.lock yet)", manifest.display()));
    }
    let stale = match (modified(&manifest), modified(&lock)) {
        (Some(m), Some(l)) => m > l,
        _ => false,
    };
    if stale {
        (
            false,
            format!(
                "{}: kode.lock is older than kode.toml, run `kode install`",
                manifest.display()
            ),
        )
    } else {
        (
            true,
            format!("{} with a current kode.lock", manifest.display()),
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

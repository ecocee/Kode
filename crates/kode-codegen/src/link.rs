use std::path::Path;
use std::process::Command;

/// The linker driver: `KODE_CC`, then `CC`, then the platform default.
pub fn cc_program() -> String {
    std::env::var("KODE_CC")
        .or_else(|_| std::env::var("CC"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "gcc".to_string()
            } else {
                "cc".to_string()
            }
        })
}

/// Link an object file with the Kode runtime into an executable.
pub fn link(object: &[u8], output: &Path, runtime_lib: &Path, strip: bool) -> Result<(), String> {
    let obj_path = format!("{}.kode.o", output.display());
    std::fs::write(&obj_path, object).map_err(|e| format!("cannot write {obj_path}: {e}"))?;
    let cc = cc_program();
    let mut cmd = Command::new(&cc);
    cmd.arg("-o")
        .arg(output)
        .arg(&obj_path)
        .arg(runtime_lib)
        .args(platform_args(strip));
    let status = cmd.status();
    let _ = std::fs::remove_file(&obj_path);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("the linker `{cc}` failed with {s}")),
        Err(e) => Err(format!(
            "could not start the linker `{cc}` ({e}); {}",
            install_hint()
        )),
    }
}

/// What to install on this platform, said in the error itself.
fn install_hint() -> &'static str {
    if cfg!(target_os = "windows") {
        "install MinGW-w64 (for example `winget install BrechtSanders.WinLibs.POSIX.UCRT`), \
         put `gcc` on PATH, or point `KODE_CC` at it"
    } else if cfg!(target_os = "macos") {
        "install the command line tools with `xcode-select --install`"
    } else {
        "install a C compiler such as gcc or clang"
    }
}

/// Linker options differ between operating systems.
fn platform_args(strip: bool) -> Vec<&'static str> {
    if cfg!(target_os = "windows") {
        // MinGW-w64: one static binary, with the Windows libraries std needs.
        let mut args = vec![
            "-Wl,--gc-sections",
            "-static",
            "-lws2_32",
            "-lbcrypt",
            "-luserenv",
            "-lntdll",
            "-ladvapi32",
            "-lm",
            "-lc",
        ];
        if strip {
            args.push("-s");
        }
        args
    } else if cfg!(target_os = "macos") {
        let mut args = vec!["-Wl,-dead_strip", "-lSystem", "-lc", "-lm", "-liconv"];
        if strip {
            args.push("-Wl,-x");
        }
        args
    } else {
        let mut args = vec![
            "-Wl,--gc-sections",
            "-lgcc_s",
            "-lutil",
            "-lrt",
            "-lpthread",
            "-lm",
            "-ldl",
            "-lc",
        ];
        if strip {
            args.push("-s");
        }
        args
    }
}

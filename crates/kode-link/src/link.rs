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
pub fn link(
    object: &[u8],
    output: &Path,
    runtime_libs: &[&Path],
    strip: bool,
) -> Result<(), String> {
    let obj_path = format!("{}.kode.o", output.display());
    std::fs::write(&obj_path, object).map_err(|e| format!("cannot write {obj_path}: {e}"))?;
    let cc = cc_program();
    let mut cmd = Command::new(&cc);
    cmd.arg("-o")
        .arg(output)
        .arg(&obj_path)
        .args(runtime_libs)
        .args(platform_args(strip));
    // Opt-in linker map, for tests that need to know which archive members were
    // pulled in. Never on by default: the map is large and only a diagnostic.
    if let Ok(map) = std::env::var("KODE_LINK_MAP") {
        if map.is_empty() {
            return Err("KODE_LINK_MAP names an empty path".to_string());
        }
        if cfg!(target_os = "windows") {
            cmd.arg(format!("-Wl,-Map={map}"));
        } else if cfg!(target_os = "macos") {
            cmd.arg(format!("-Wl,-map,{map}"));
        } else {
            cmd.arg(format!("-Wl,-Map={map}"));
        }
    }
    // A missing or non-executable linker must be reported as such, never as a
    // child that "ran" and exited 127.
    crate::force_fork_exec(&mut cmd);
    let status = cmd.status();
    let _ = std::fs::remove_file(&obj_path);
    match status {
        Ok(s) if s.success() => Ok(()),
        // Never leave a stale or half-written executable behind: a caller that
        // ignores the error must not find a binary from an earlier build here.
        Ok(s) => {
            let _ = std::fs::remove_file(output);
            Err(format!("the linker `{cc}` failed with {s}"))
        }
        Err(e) => {
            let _ = std::fs::remove_file(output);
            Err(format!(
                "could not start the linker `{cc}` ({e}); {}",
                install_hint()
            ))
        }
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
        // `gcc` links the C runtime itself, and there is no `libc.a` to name:
        // asking for `-lc` makes ld stop with "cannot find -lc".
        let mut args = vec![
            "-Wl,--gc-sections",
            "-static",
            "-lws2_32",
            // std's Windows futex calls WakeByAddressSingle, which lives in
            // synchronization.lib; without it the runtime archive links with an
            // undefined reference as soon as a program touches a thread primitive.
            "-lsynchronization",
            "-lbcrypt",
            "-luserenv",
            "-lntdll",
            "-ladvapi32",
            "-lm",
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

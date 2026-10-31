use std::path::Path;
use std::process::Command;

/// Link an object file with the Kode runtime into an executable.
pub fn link(object: &[u8], output: &Path, runtime_lib: &Path, strip: bool) -> Result<(), String> {
    let obj_path = format!("{}.kode.o", output.display());
    std::fs::write(&obj_path, object).map_err(|e| format!("cannot write {obj_path}: {e}"))?;
    let mut cmd = Command::new("cc");
    cmd.arg("-o")
        .arg(output)
        .arg(&obj_path)
        .arg(runtime_lib)
        .args(platform_args(strip));
    let status = cmd.status();
    let _ = std::fs::remove_file(&obj_path);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("the linker (`cc`) failed with {s}")),
        Err(e) => Err(format!(
            "could not start the linker `cc` ({e}); install a C compiler such as gcc or clang"
        )),
    }
}

/// Linker options differ between operating systems.
fn platform_args(strip: bool) -> Vec<&'static str> {
    if cfg!(target_os = "macos") {
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

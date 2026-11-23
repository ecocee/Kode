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
use std::path::{Path, PathBuf};

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

    // `CARGO_TARGET_DIR` overrides the workspace `target` folder, and the build
    // profile decides where the archive actually lands.
    let target_directory = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace.join("target"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let mut candidates = vec![
        target_directory
            .join(target_triple.as_str())
            .join(&profile)
            .join(lib_name),
        target_directory.join(&profile).join(lib_name),
    ];
    // A `staticlib` compiled as a build dependency is not uplifted out of
    // `deps/`, so `cargo build --release` on its own leaves only
    // `target/release/deps/libkode_rt-<hash>.a`. Look there too, newest first,
    // because a stale archive from an earlier hash must not win.
    for deps in [
        target_directory
            .join(target_triple.as_str())
            .join(&profile)
            .join("deps"),
        target_directory.join(&profile).join("deps"),
    ] {
        let mut found: Vec<PathBuf> = fs::read_dir(&deps)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .is_some_and(|n| n.starts_with("libkode_rt") && n.ends_with(".a"))
                    })
                    .collect()
            })
            .unwrap_or_default();
        found.sort_by_key(|p| std::cmp::Reverse(fs::metadata(p).and_then(|m| m.modified()).ok()));
        candidates.extend(found);
    }

    // Newest wins, not first-found. `cargo build --release -p kode-rt` uplifts the
    // staticlib to `target/release/libkode_rt.a` and leaves it there; a later build
    // with a new hash writes `deps/libkode_rt-<hash>.a` and leaves the uplifted copy
    // behind. Picking the first candidate embedded that stale copy, so a program using
    // a function added since (`StringBuilder`) failed at *link* time with undefined
    // symbols while `kode -V` looked fine.
    let mut existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    existing.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());

    let lib_path = match existing.pop() {
        Some(path) => path,
        // A clean checkout can reach here: a `staticlib` built as a build
        // dependency is not always uplifted out of `deps/`, and CI's release build
        // has no earlier `cargo build -p kode-rt` to lean on. Build the runtime once
        // in our own target directory (a nested cargo would deadlock on the same
        // lock) instead of failing the whole build.
        None => build_runtime(&workspace, &profile).unwrap_or_else(|why| {
            panic!(
                "could not find {} under {} ({why}); run `cargo build -p kode-rt{}`",
                lib_name,
                target_directory.display(),
                if profile == "release" {
                    " --release"
                } else {
                    ""
                }
            )
        }),
    };

    // An archive older than the runtime sources cannot contain what they declare.
    // Refuse to embed it instead of failing later with undefined symbols only when
    // a user builds a program that happens to call a new function.
    if let Some(newest_src) = newest_mtime(&workspace.join("kode-rt").join("src")) {
        if let Ok(archive_mtime) = fs::metadata(&lib_path).and_then(|m| m.modified()) {
            if archive_mtime < newest_src {
                panic!(
                    "{} is older than the runtime sources; run `cargo build -p kode-rt{}` first",
                    lib_path.display(),
                    if profile == "release" {
                        " --release"
                    } else {
                        ""
                    },
                );
            }
        }
    }

    fs::create_dir_all(&out_dir).expect("could not create OUT_DIR");
    fs::copy(lib_path, out_dir.join(lib_name)).expect("could not copy libkode_rt.a into OUT_DIR");

    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("kode-rt").join("src").display()
    );
    // Split the archive so a program that never touches the network does not link
    // TLS, HTTP or PostgreSQL. `ar` and `nm` come with the C toolchain the compiler
    // already needs; when they are missing, core gets the whole archive and net a
    // copy, which links exactly as before (bigger, never broken).
    let core_lib = out_dir.join("libkode_rt_core.a");
    let net_lib = out_dir.join("libkode_rt_net.a");

    if let Err(why) = split_archive(&out_dir.join(lib_name), &core_lib, &net_lib) {
        println!("cargo:warning=could not split the kode runtime ({why}); linking it whole");
        fs::copy(out_dir.join(lib_name), &core_lib).expect("could not copy the runtime");
        fs::copy(out_dir.join(lib_name), &net_lib).expect("could not copy the runtime");
    }

    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("kode-rt").join("Cargo.toml").display()
    );
}

/// Symbols only a networked program can reach.
const NET_SYMBOLS: [&str; 12] = [
    "aws_lc_",
    "rustls",
    "webpki",
    "ring_core",
    "scram",
    "sha256",
    "base64",
    "kode_http",
    "kode_tls",
    "kode_pg",
    "kode_router",
    "kode_https",
];

/// Symbols the program's entry point always needs, so their objects stay in core
/// even when they also define network entry points (the crate-root object does).
const CORE_SYMBOLS: [&str; 4] = [
    "kode_rt_init",
    "kode_rt_finish",
    "kode_retain",
    "kode_release",
];

fn tool(name: &str, env: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| name.to_string())
}

fn run(cmd: &mut std::process::Command) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("cannot run it: {e}"))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Global symbols of one object file: the ones it defines and the ones it needs.
/// Works for Mach-O (`nm -g` prints `addr T _name`, undefined entries are a
/// two-column `U _name` line) and ELF (`addr T name`).
fn symbols(nm: &str, object: &Path) -> Result<(Vec<String>, Vec<String>), String> {
    let out = std::process::Command::new(nm)
        .arg("-g")
        .arg(object)
        .output()
        .map_err(|e| format!("cannot run `{nm}`: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let mut defined = Vec::new();
    let mut undefined = Vec::new();

    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        match fields.as_slice() {
            // ELF undefined: `                 U name`
            [kind, name] if *kind == "U" => undefined.push(name.to_string()),
            // Defined: `addr T name`, or an undefined symbol with an address slot.
            [_addr, kind, name] if *kind == "U" => undefined.push(name.to_string()),
            [_addr, _kind, name] => defined.push(name.to_string()),
            _ => {}
        }
    }

    Ok((defined, undefined))
}

/// Build `kode-rt` and return the archive it produced. Used only when no archive
/// from the current build exists (clean checkout, CI). Separate target dir: cargo
/// holds a lock on the outer one, so reusing it would deadlock.
fn build_runtime(workspace: &Path, profile: &str) -> Result<PathBuf, String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = workspace.join("target").join("kode-rt-embedded");

    let mut cmd = std::process::Command::new(cargo);
    cmd.arg("build")
        .arg("-p")
        .arg("kode-rt")
        .arg("--target-dir")
        .arg(&dir);

    if profile == "release" {
        cmd.arg("--release");
        // The workspace release profile turns LTO on. An LTO'd staticlib drops
        // monomorphisations that a generated program links against (the router
        // closure was the first casualty), so the embedded runtime is built the way
        // the build-dependency profile builds it: optimised, not LTO'd.
        cmd.arg("--config").arg("profile.release.lto=false");
    }

    let out = cmd.output().map_err(|e| format!("cannot run cargo: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let mut found: Vec<PathBuf> = ["deps", ""]
        .iter()
        .flat_map(|sub| {
            fs::read_dir(dir.join(profile).join(sub))
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.starts_with("libkode_rt") && n.ends_with(".a"))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    found.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());

    found
        .pop()
        .ok_or_else(|| format!("no archive in {}", dir.display()))
}

fn split_archive(lib: &Path, core_lib: &Path, net_lib: &Path) -> Result<(), String> {
    let ar = tool("ar", "KODE_AR");
    let nm = tool("nm", "KODE_NM");
    let work = lib.with_extension("split");

    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|e| e.to_string())?;

    run(std::process::Command::new(&ar)
        .arg("x")
        .arg(lib)
        .current_dir(&work))
    .map_err(|e| format!("`{ar} x` failed: {e}"))?;

    let mut objects: Vec<PathBuf> = fs::read_dir(&work)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "o"))
        .collect();

    objects.sort();

    if objects.is_empty() {
        return Err("the archive has no objects".to_string());
    }

    let mut core = Vec::new();
    let mut net = Vec::new();

    for obj in &objects {
        // Apple's `nm` cannot read objects LLVM 22 wrote (compiler_builtins carries
        // bitcode), so an unreadable object stays in core: present everywhere,
        // never missing.
        let Ok((defined, _)) = symbols(&nm, obj) else {
            core.push(obj.clone());
            continue;
        };

        let networked = defined
            .iter()
            .any(|s| NET_SYMBOLS.iter().any(|m| s.contains(m)));
        let essential = defined
            .iter()
            .any(|s| CORE_SYMBOLS.iter().any(|m| s.contains(m)));

        if networked && !essential {
            net.push(obj.clone());
        } else {
            core.push(obj.clone());
        }
    }

    if core.is_empty() || net.is_empty() {
        return Err("nothing to split".to_string());
    }

    // Whatever the platform's codegen-unit layout, core has to satisfy an ordinary
    // program on its own. If it does not, give up on the split for this build and
    // let the caller embed the whole archive: correct and larger beats fast and
    // broken. This is the check that would have caught the ELF split losing
    // `kode_print_str`.
    let must_define = [
        "kode_rt_init",
        "kode_rt_finish",
        "kode_print_str",
        "kode_print_nl",
        "kode_retain",
        "kode_release",
    ];

    for needle in must_define {
        let found = core.iter().any(|obj| {
            symbols(&nm, obj)
                .map(|(defined, _)| defined.iter().any(|d| d.contains(needle)))
                .unwrap_or(false)
        });

        if !found {
            return Err(format!("core would not define {needle}"));
        }
    }

    for (dest, members) in [(core_lib, &core), (net_lib, &net)] {
        // `ar r` replaces or adds but never deletes, so a stale archive would keep
        // every member. Start from nothing.
        let _ = fs::remove_file(dest);

        let names: Vec<String> = members
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();

        run(std::process::Command::new(&ar)
            .arg("rcs")
            .arg(dest)
            .args(&names)
            .current_dir(&work))
        .map_err(|e| format!("`{ar} rcs` failed: {e}"))?;
    }

    let _ = fs::remove_dir_all(&work);
    Ok(())
}

/// The newest modification time of any file under `dir`.
fn newest_mtime(dir: &Path) -> Option<std::time::SystemTime> {
    let mut newest = None;

    for entry in fs::read_dir(dir).ok()? {
        let path = entry.ok()?.path();
        let mtime = if path.is_dir() {
            newest_mtime(&path)
        } else {
            fs::metadata(&path).and_then(|m| m.modified()).ok()
        };

        if let Some(t) = mtime {
            newest = Some(match newest {
                Some(n) if n > t => n,
                _ => t,
            });
        }
    }

    newest
}

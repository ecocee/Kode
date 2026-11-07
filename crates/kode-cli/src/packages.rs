//! Package commands: `kode init`, `kode new`, `kode add`, `kode remove`,
//! `kode install`. A package is a folder with a `kode.toml`; dependencies are
//! other packages, resolved by path into one flat namespace.

use std::path::PathBuf;

use kode_diagnostics::Diagnostic;

use crate::args::Options;
use crate::manifest;
use crate::resolve;

fn cwd() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| format!("cannot read the current folder: {e}"))
}

fn describe(d: Diagnostic) -> String {
    match d.help {
        Some(help) => format!("{}: {} (help: {help})", d.code, d.message),
        None => format!("{}: {}", d.code, d.message),
    }
}

/// `kode init [name]`: a package in the current folder.
pub fn init(opts: &Options) -> Result<u8, String> {
    let dir = cwd()?;
    let name = match opts.file() {
        Some(n) => n.to_string(),
        None => dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string()),
    };
    let file = manifest::create(&dir, &name)?;
    println!("created package `{name}` -> {}", file.display());
    println!("run it with: kode run src/main.kode");
    Ok(0)
}

/// `kode new <name>`: a package in a new folder.
pub fn new_package(opts: &Options) -> Result<u8, String> {
    let name = opts
        .file()
        .ok_or("`kode new` needs a name, for example `kode new shop`")?;
    let dir = cwd()?.join(name);
    let file = manifest::create(&dir, name)?;
    println!("created package `{name}` -> {}", file.display());
    println!("next: cd {name} && kode run src/main.kode");
    Ok(0)
}

/// `kode add <name> --path <folder>`
pub fn add(opts: &Options) -> Result<u8, String> {
    let name = opts
        .file()
        .ok_or("`kode add` needs a name, for example `kode add utils --path ../utils`")?;
    let path = opts.path.as_deref().ok_or(
        "`kode add` needs --path, because there is no registry yet: `kode add utils --path ../utils`",
    )?;
    let dir = cwd()?;
    let m = manifest::find_in(&dir)
        .map_err(describe)?
        .ok_or("no `kode.toml` here; run `kode init` first")?;
    manifest::add_dependency(&m.dir, name, path, None)?;
    println!("added `{name}` from {path}");
    install(opts)?;
    Ok(0)
}

/// `kode remove <name>`
pub fn remove(opts: &Options) -> Result<u8, String> {
    let name = opts
        .file()
        .ok_or("`kode remove` needs a name, for example `kode remove utils`")?;
    let dir = cwd()?;
    let m = manifest::find_in(&dir)
        .map_err(describe)?
        .ok_or("no `kode.toml` here; run `kode init` first")?;
    if !manifest::remove_dependency(&m.dir, name)? {
        return Err(format!(
            "`{name}` is not a dependency in {}",
            m.dir.join("kode.toml").display()
        ));
    }
    println!("removed `{name}`");
    install(opts)?;
    Ok(0)
}

/// `kode install [--check]`: resolve dependencies and write `kode.lock`.
pub fn install(opts: &Options) -> Result<u8, String> {
    let dir = cwd()?;
    let m = manifest::find_in(&dir)
        .map_err(describe)?
        .ok_or("no `kode.toml` here; run `kode init` to start a package")?;
    let deps = resolve::dependencies(&m).map_err(describe)?;
    let lock = resolve::lock_text(&m, &deps);
    let file = m.dir.join("kode.lock");
    if opts.check {
        let current = std::fs::read_to_string(&file).unwrap_or_default();
        if current == lock {
            println!("kode.lock is up to date ({} package(s))", deps.len());
            return Ok(0);
        }
        eprintln!("kode.lock is out of date; run `kode install`");
        return Ok(1);
    }
    std::fs::write(&file, lock).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    println!("resolved {} package(s) -> {}", deps.len(), file.display());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_a_diagnostic_with_its_hint() {
        let d = Diagnostic::error(
            "E0412",
            "dependency `utils` is not a package",
            Default::default(),
        )
        .with_help("check the path");
        assert_eq!(
            describe(d),
            "E0412: dependency `utils` is not a package (help: check the path)"
        );
    }
}

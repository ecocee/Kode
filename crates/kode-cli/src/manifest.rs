//! A package is a folder with a `kode.toml` file:
//!
//! ```toml
//! [package]
//! name = "shop"
//! version = "0.1.0"
//! root = "src"
//! ```
//!
//! `root` is the package source folder: `import "models/user"` (no leading
//! `./`) looks there. Paths that start with `./` or `../` stay relative to the
//! file that imports them. A manifest is found by walking up from the entry
//! file, so every program in a package shares one root.

use std::path::{Component, Path, PathBuf};

use kode_diagnostics::Diagnostic;

#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
}

/// Find the nearest `kode.toml` at or above `entry`.
pub fn find(entry: &Path) -> Result<Option<Manifest>, Diagnostic> {
    let mut dir = match entry.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    loop {
        let file = dir.join("kode.toml");
        if file.is_file() {
            return parse(&file).map(Some);
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

fn parse(path: &Path) -> Result<Manifest, Diagnostic> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| at(format!("cannot read {}: {e}", path.display())))?;
    let mut section = String::new();
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut root: Option<String> = None;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(at(format!(
                    "{}:{}: a section header needs a closing `]`",
                    path.display(),
                    i + 1
                )));
            }
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(at(format!(
                "{}:{}: expected `key = \"value\"`",
                path.display(),
                i + 1
            )));
        };
        if section != "package" {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or(value);
        match key.trim() {
            "name" => name = Some(value.to_string()),
            "version" => version = Some(value.to_string()),
            "root" => root = Some(value.to_string()),
            _ => {}
        }
    }
    let name = name.ok_or_else(|| {
        at(format!(
            "{}: `kode.toml` needs `name` under `[package]`",
            path.display()
        ))
    })?;
    if !valid_name(&name) {
        return Err(at(format!(
            "{}: the package name `{name}` must start with a letter or `_` and use only letters, digits, `_` and `-`",
            path.display()
        )));
    }
    let root = root.unwrap_or_else(|| "src".to_string());
    let rel = Path::new(&root);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(at(format!(
            "{}: the package root `{root}` must be a relative folder inside the package",
            path.display()
        )));
    }
    let dir = path.parent().unwrap_or(Path::new("."));
    Ok(Manifest {
        name,
        version: version.unwrap_or_else(|| "0.1.0".to_string()),
        root: dir.join(rel),
    })
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn at(message: String) -> Diagnostic {
    Diagnostic::error("E0406", message, Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_package_keys() {
        let dir = std::env::temp_dir().join(format!("kode-pkg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("kode.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"shop\"\nversion = \"1.2.0\"\nroot = \"lib\"\n",
        )
        .unwrap();
        let m = parse(&file).unwrap();
        assert_eq!(m.name, "shop");
        assert_eq!(m.version, "1.2.0");
        assert_eq!(m.root, dir.join("lib"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_a_missing_name() {
        let dir = std::env::temp_dir().join(format!("kode-pkg-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("kode.toml");
        std::fs::write(&file, "[package]\nroot = \"src\"\n").unwrap();
        let err = parse(&file).unwrap_err();
        assert_eq!(err.code, "E0406");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

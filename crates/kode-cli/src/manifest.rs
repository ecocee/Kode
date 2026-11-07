//! A package is a folder with a `kode.toml` file:
//!
//! ```toml
//! [package]
//! name = "shop"
//! version = "0.1.0"
//! root = "src"
//!
//! [dependencies]
//! utils = { path = "../utils" }
//! pricing = { path = "../pricing", version = "0.2.0" }
//! ```
//!
//! `root` is the package source folder: `import "models/user"` (no leading
//! `./`) looks there. Paths that start with `./` or `../` stay relative to the
//! file that imports them. A manifest is found by walking up from the entry
//! file, so every program in a package shares one root.
//!
//! Dependencies are folders with their own `kode.toml`. There is no registry
//! yet, so a version is only a compatibility check against that folder.

use std::path::{Component, Path, PathBuf};

use kode_diagnostics::Diagnostic;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    /// The dependency's folder (the folder that holds its `kode.toml`).
    pub path: PathBuf,
    /// The version the importing package expects, when it names one.
    pub version: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
    /// The folder that holds this manifest.
    pub dir: PathBuf,
    pub deps: Vec<Dependency>,
}

/// Find the nearest `kode.toml` at or above `entry`.
pub fn find(entry: &Path) -> Result<Option<Manifest>, Diagnostic> {
    let dir = match entry.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    find_in(&dir)
}

/// Find the nearest `kode.toml` at or above a folder.
pub fn find_in(dir: &Path) -> Result<Option<Manifest>, Diagnostic> {
    let mut dir = dir.to_path_buf();
    loop {
        let file = dir.join("kode.toml");
        if file.is_file() {
            return read(&file).map(Some);
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

/// Read and validate one manifest.
pub fn read(path: &Path) -> Result<Manifest, Diagnostic> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| at(format!("cannot read {}: {e}", path.display())))?;
    let mut section = String::new();
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut root: Option<String> = None;
    let mut raw_deps: Vec<(String, Option<String>, Option<String>)> = Vec::new();
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
        let (key, value) = (key.trim(), value.trim());
        match section.as_str() {
            "package" => {
                let value = unquote(value).to_string();
                match key {
                    "name" => name = Some(value),
                    "version" => version = Some(value),
                    "root" => root = Some(value),
                    _ => {}
                }
            }
            "dependencies" => {
                if !valid_name(key) {
                    return Err(at(format!(
                        "{}:{}: `{key}` is not a valid dependency name",
                        path.display(),
                        i + 1
                    )));
                }
                if raw_deps.iter().any(|(n, _, _)| n == key) {
                    return Err(code(
                        "E0414",
                        format!(
                            "{}: `{key}` is listed twice under [dependencies]",
                            path.display()
                        ),
                    ));
                }
                let (dep_path, dep_version) = dep_value(value)
                    .map_err(|m| at(format!("{}:{}: {m}", path.display(), i + 1)))?;
                raw_deps.push((key.to_string(), dep_path, dep_version));
            }
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
    let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let mut deps = Vec::new();
    for (dep_name, dep_path, dep_version) in raw_deps {
        if dep_name == name {
            return Err(code(
                "E0414",
                format!(
                    "{}: a package cannot depend on itself (`{name}`)",
                    path.display()
                ),
            ));
        }
        let Some(relative) = dep_path.filter(|p| !p.is_empty()) else {
            return Err(code(
                "E0413",
                format!("{}: `{dep_name}` is pinned by version only", path.display()),
            )
            .with_help(
                "there is no registry yet, so a dependency needs a path: \
                 `utils = { path = \"../utils\" }`",
            ));
        };
        deps.push(Dependency {
            name: dep_name,
            path: dir.join(relative),
            version: dep_version,
        });
    }

    let root = root.unwrap_or_else(|| "src".to_string());
    let rel = Path::new(&root);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(at(format!(
            "{}: the package root `{root}` must be a relative folder inside the package",
            path.display()
        )));
    }
    Ok(Manifest {
        name,
        version: version.unwrap_or_else(|| "0.1.0".to_string()),
        root: dir.join(rel),
        dir,
        deps,
    })
}

/// `{ path = "../x", version = "0.1.0" }` or a bare version string.
fn dep_value(value: &str) -> Result<(Option<String>, Option<String>), String> {
    if let Some(inner) = value.strip_prefix('{') {
        let inner = inner
            .strip_suffix('}')
            .ok_or_else(|| format!("`{value}` needs a closing `}}`"))?;
        let (mut path, mut version) = (None, None);
        for part in inner.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let Some((key, val)) = part.split_once('=') else {
                return Err(format!("expected `key = \"value\"` in `{value}`"));
            };
            let val = unquote(val.trim()).to_string();
            match key.trim() {
                "path" => path = Some(val),
                "version" => version = Some(val),
                other => {
                    return Err(format!(
                        "unknown dependency key `{other}`; only `path` and `version` are supported"
                    ))
                }
            }
        }
        Ok((path, version))
    } else {
        Ok((None, Some(unquote(value).to_string())))
    }
}

/// Create a package: `kode.toml`, `src/` and a small `src/main.kode`.
pub fn create(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if !valid_name(name) {
        return Err(format!(
            "`{name}` is not a valid package name: start with a letter or `_`, then letters, digits, `_` or `-`"
        ));
    }
    let file = dir.join("kode.toml");
    if file.exists() {
        return Err(format!("{} already has a kode.toml", dir.display()));
    }
    let src = dir.join("src");
    std::fs::create_dir_all(&src).map_err(|e| format!("cannot create {}: {e}", src.display()))?;
    let manifest = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nroot = \"src\"\n");
    std::fs::write(&file, manifest).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    let main = src.join("main.kode");
    if !main.exists() {
        let mut text = String::new();
        text.push_str(&format!("// {name}: a Kode package\n"));
        text.push_str("fn greet(who: string) {\n    print(\"hello, ${who}\")\n}\n\n");
        text.push_str(&format!("greet(\"{name}\")\n"));
        std::fs::write(&main, text).map_err(|e| format!("cannot write {}: {e}", main.display()))?;
    }
    Ok(file)
}

/// Add or replace one `[dependencies]` entry.
pub fn add_dependency(
    dir: &Path,
    name: &str,
    dep_path: &str,
    version: Option<&str>,
) -> Result<(), String> {
    if !valid_name(name) {
        return Err(format!("`{name}` is not a valid dependency name"));
    }
    let file = dir.join("kode.toml");
    let text = std::fs::read_to_string(&file).map_err(|e| {
        format!(
            "cannot read {}: {e} (run `kode init` first)",
            file.display()
        )
    })?;
    let entry = match version {
        Some(v) => format!("{name} = {{ path = \"{dep_path}\", version = \"{v}\" }}"),
        None => format!("{name} = {{ path = \"{dep_path}\" }}"),
    };
    let updated = upsert_in_section(&text, "dependencies", name, &entry);
    std::fs::write(&file, updated).map_err(|e| format!("cannot write {}: {e}", file.display()))
}

/// Remove one `[dependencies]` entry. Returns false when it was not there.
pub fn remove_dependency(dir: &Path, name: &str) -> Result<bool, String> {
    let file = dir.join("kode.toml");
    let text = std::fs::read_to_string(&file).map_err(|e| {
        format!(
            "cannot read {}: {e} (run `kode init` first)",
            file.display()
        )
    })?;
    let (updated, removed) = remove_from_section(&text, "dependencies", name);
    if removed {
        std::fs::write(&file, updated)
            .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    }
    Ok(removed)
}

fn upsert_in_section(text: &str, section: &str, key: &str, entry: &str) -> String {
    let header = format!("[{section}]");
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let Some(start) = lines.iter().position(|l| l.trim() == header) else {
        let mut out = text.trim_end().to_string();
        if !out.is_empty() {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&header);
        out.push('\n');
        out.push_str(entry);
        out.push('\n');
        return out;
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .map(|i| start + 1 + i)
        .unwrap_or(lines.len());
    let existing = (start + 1..end).find(|&i| {
        let line = lines[i].trim_start();
        line.strip_prefix(key)
            .map(|rest| rest.starts_with(' ') || rest.starts_with('='))
            .unwrap_or(false)
    });
    match existing {
        Some(i) => lines[i] = entry.to_string(),
        None => lines.insert(end, entry.to_string()),
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn remove_from_section(text: &str, section: &str, key: &str) -> (String, bool) {
    let header = format!("[{section}]");
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let Some(start) = lines.iter().position(|l| l.trim() == header) else {
        return (text.to_string(), false);
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim_start().starts_with('['))
        .map(|i| start + 1 + i)
        .unwrap_or(lines.len());
    let found = (start + 1..end).find(|&i| {
        let line = lines[i].trim_start();
        line.strip_prefix(key)
            .map(|rest| rest.starts_with(' ') || rest.starts_with('='))
            .unwrap_or(false)
    });
    let Some(i) = found else {
        return (text.to_string(), false);
    };
    lines.remove(i);
    let mut out = lines.join("\n");
    out.push('\n');
    (out, true)
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
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

fn code(code: &'static str, message: String) -> Diagnostic {
    Diagnostic::error(code, message, Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kode-pkg-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reads_package_keys() {
        let dir = temp_dir("keys");
        let file = dir.join("kode.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"shop\"\nversion = \"1.2.0\"\nroot = \"lib\"\n",
        )
        .unwrap();
        let m = read(&file).unwrap();
        assert_eq!(m.name, "shop");
        assert_eq!(m.version, "1.2.0");
        assert_eq!(m.root, dir.join("lib"));
        assert!(m.deps.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_a_missing_name() {
        let dir = temp_dir("bad");
        let file = dir.join("kode.toml");
        std::fs::write(&file, "[package]\nroot = \"src\"\n").unwrap();
        assert_eq!(read(&file).unwrap_err().code, "E0406");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reads_path_dependencies() {
        let dir = temp_dir("deps");
        let file = dir.join("kode.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"app\"\n\n[dependencies]\nutils = { path = \"../utils\" }\npricing = { path = \"vendor/pricing\", version = \"0.2.0\" }\n",
        )
        .unwrap();
        let m = read(&file).unwrap();
        assert_eq!(m.deps.len(), 2);
        assert_eq!(m.deps[0].name, "utils");
        assert_eq!(m.deps[0].path, dir.join("../utils"));
        assert_eq!(m.deps[0].version, None);
        assert_eq!(m.deps[1].version.as_deref(), Some("0.2.0"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_version_only_dependency_has_a_useful_error() {
        let dir = temp_dir("version-only");
        let file = dir.join("kode.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"app\"\n\n[dependencies]\nutils = \"1.0\"\n",
        )
        .unwrap();
        let err = read(&file).unwrap_err();
        assert_eq!(err.code, "E0413");
        assert!(err.help.unwrap().contains("path"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_a_self_dependency_and_duplicates() {
        let dir = temp_dir("self");
        let file = dir.join("kode.toml");
        std::fs::write(
            &file,
            "[package]\nname = \"app\"\n\n[dependencies]\napp = { path = \".\" }\n",
        )
        .unwrap();
        assert_eq!(read(&file).unwrap_err().code, "E0414");
        std::fs::write(&file, "[package]\nname = \"app\"\n\n[dependencies]\nutils = { path = \"a\" }\nutils = { path = \"b\" }\n").unwrap();
        assert_eq!(read(&file).unwrap_err().code, "E0414");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn adds_and_removes_a_dependency() {
        let dir = temp_dir("edit");
        std::fs::write(
            dir.join("kode.toml"),
            "[package]\nname = \"app\"\nroot = \"src\"\n",
        )
        .unwrap();
        add_dependency(&dir, "utils", "../utils", None).unwrap();
        add_dependency(&dir, "pricing", "../pricing", Some("0.2.0")).unwrap();
        let m = read(&dir.join("kode.toml")).unwrap();
        assert_eq!(m.deps.len(), 2);

        // Replacing, not duplicating.
        add_dependency(&dir, "utils", "../tools", None).unwrap();
        let m = read(&dir.join("kode.toml")).unwrap();
        assert_eq!(m.deps.len(), 2);
        assert_eq!(m.deps[0].path, dir.join("../tools"));

        assert!(remove_dependency(&dir, "utils").unwrap());
        assert!(!remove_dependency(&dir, "utils").unwrap());
        let m = read(&dir.join("kode.toml")).unwrap();
        assert_eq!(m.deps.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn creates_a_package() {
        let dir = temp_dir("create");
        let pkg = dir.join("notes");
        create(&pkg, "notes").unwrap();
        assert!(pkg.join("kode.toml").is_file());
        assert!(pkg.join("src/main.kode").is_file());
        let m = read(&pkg.join("kode.toml")).unwrap();
        assert_eq!(m.name, "notes");
        assert_eq!(m.root, pkg.join("src"));
        assert!(create(&pkg, "notes")
            .unwrap_err()
            .contains("already has a kode.toml"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

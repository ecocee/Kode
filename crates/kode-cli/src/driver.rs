//! Loading a program: read the file, follow its `import`s, parse everything,
//! and merge it into one `Program`.
//!
//! All modules still share one namespace (`pub` gates module and `from`
//! imports, but not the shared namespace). A `kode.toml` found above the entry
//! file makes bare imports package-relative; `./` and `../` stay file-relative.
//! `import "x" as name` records a module alias for qualified access.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use kode_diagnostics::Diagnostic;
use kode_sema::ModuleInfo;
use kode_syntax::ast::{Item, Program};
use kode_syntax::{lexer, parser};

use crate::manifest::{self, Manifest};
use crate::source::SourceMap;

pub struct Loaded {
    pub program: Program,
    pub map: SourceMap,
    pub modules: Vec<ModuleInfo>,
}

struct State {
    map: SourceMap,
    next_id: u32,
    stack: Vec<PathBuf>,
    loaded: HashSet<PathBuf>,
    names: HashMap<PathBuf, Vec<String>>,
    public: HashMap<PathBuf, Vec<String>>,
    modules: Vec<ModuleInfo>,
    imported: Vec<Item>,
    errors: Vec<Diagnostic>,
    manifest: Option<Manifest>,
}

pub fn load(path: &Path) -> Result<Loaded, (SourceMap, Vec<Diagnostic>)> {
    let manifest = match manifest::find(path) {
        Ok(m) => m,
        Err(err) => {
            let mut map = SourceMap::default();
            map.add(path.display().to_string(), String::new());
            return Err((map, vec![err]));
        }
    };
    let mut st = State {
        map: SourceMap::default(),
        next_id: 0,
        stack: Vec::new(),
        loaded: HashSet::new(),
        names: HashMap::new(),
        public: HashMap::new(),
        modules: Vec::new(),
        imported: Vec::new(),
        errors: Vec::new(),
        manifest,
    };
    let items = match load_file(path, &mut st) {
        Ok(items) => items,
        Err(errs) => {
            st.errors.extend(errs);
            return Err((st.map, st.errors));
        }
    };
    if !st.errors.is_empty() {
        return Err((st.map, st.errors));
    }
    let mut all = std::mem::take(&mut st.imported);
    all.extend(items);
    Ok(Loaded {
        program: Program {
            items: all,
            expr_count: st.next_id,
        },
        map: st.map,
        modules: std::mem::take(&mut st.modules),
    })
}

fn defined_names(items: &[Item]) -> (Vec<String>, Vec<String>) {
    let mut all = Vec::new();
    let mut public = Vec::new();
    for item in items {
        let entry = match item {
            Item::Fn(f) => Some((&f.name, f.is_pub)),
            Item::Const(c) => Some((&c.name, c.is_pub)),
            Item::Class(c) => Some((&c.name, c.is_pub)),
            Item::Enum(e) => Some((&e.name, e.is_pub)),
            Item::Interface(i) => Some((&i.name, i.is_pub)),
            _ => None,
        };
        if let Some((name, is_pub)) = entry {
            all.push(name.clone());
            if is_pub {
                public.push(name.clone());
            }
        }
    }
    (all, public)
}

fn load_file(path: &Path, st: &mut State) -> Result<Vec<Item>, Vec<Diagnostic>> {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = std::fs::read_to_string(path).map_err(|e| {
        vec![Diagnostic::error(
            "E0401",
            format!("cannot read {}: {e}", path.display()),
            Default::default(),
        )]
    })?;
    let base = st.map.add(path.display().to_string(), text.clone());
    let tokens = lexer::lex_at(&text, base)?;
    let program = parser::parse_from(tokens, st.next_id)?;
    st.next_id = program.expr_count;
    let (names, public) = defined_names(&program.items);
    st.names.insert(canon.clone(), names);
    st.public.insert(canon.clone(), public);

    st.stack.push(canon.clone());
    let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    for item in &program.items {
        let Item::Import(imp) = item else { continue };
        let target = match resolve_import(&dir, st.manifest.as_ref(), &imp.path, imp.path_span) {
            Ok(t) => t,
            Err(e) => {
                st.errors.push(e);
                continue;
            }
        };
        if !target.exists() {
            let help = match st.manifest.as_ref() {
                Some(m) => format!(
                    "looked for {} in package `{}` {}",
                    target.display(),
                    m.name,
                    m.version
                ),
                None => format!("looked for {}", target.display()),
            };
            st.errors.push(
                Diagnostic::error(
                    "E0401",
                    format!("cannot find the file `{}`", imp.path),
                    imp.path_span,
                )
                .with_help(help),
            );
            continue;
        }
        let tcanon = target.canonicalize().unwrap_or_else(|_| target.clone());
        if st.stack.contains(&tcanon) {
            st.errors.push(Diagnostic::error(
                "E0402",
                format!(
                    "`{}` imports itself, directly or through other files",
                    imp.path
                ),
                imp.path_span,
            ));
            continue;
        }
        if !st.loaded.contains(&tcanon) {
            st.loaded.insert(tcanon.clone());
            match load_file(&target, st) {
                Ok(items) => {
                    let mut keep = Vec::new();
                    for it in items {
                        match it {
                            Item::Stmt(_) => st.errors.push(
                                Diagnostic::error(
                                    "E0403",
                                    format!("`{}` has top-level code, which a module cannot have", imp.path),
                                    imp.path_span,
                                )
                                .with_help("a module may only contain fn, const, class, interface and enum"),
                            ),
                            Item::Test(_) | Item::Import(_) => {}
                            other => keep.push(other),
                        }
                    }
                    st.imported.extend(keep);
                }
                Err(errs) => st.errors.extend(errs),
            }
        }
        if let Some(defined) = st.names.get(&tcanon) {
            let public = st.public.get(&tcanon).cloned().unwrap_or_default();
            for (n, span) in &imp.names {
                if !defined.contains(n) {
                    st.errors.push(Diagnostic::error(
                        "E0405",
                        format!("`{}` does not define `{n}`", imp.path),
                        *span,
                    ));
                } else if !public.contains(n) {
                    st.errors.push(
                        Diagnostic::error(
                            "E0411",
                            format!("`{}` does not export `{n}`", imp.path),
                            *span,
                        )
                        .with_help("mark it `pub` in that file to import it"),
                    );
                }
            }
            if let Some((alias, aspan)) = &imp.alias {
                st.modules.push(ModuleInfo {
                    alias: alias.clone(),
                    path: imp.path.clone(),
                    exports: public,
                    all: defined.clone(),
                    span: *aspan,
                });
            }
        }
    }
    st.stack.pop();
    Ok(program.items)
}

/// `./x` and `../x` are relative to the importing file; a bare `pkg/x` is
/// relative to the package root when a manifest exists.
fn resolve_import(
    dir: &Path,
    manifest: Option<&Manifest>,
    raw: &str,
    span: kode_diagnostics::Span,
) -> Result<PathBuf, Diagnostic> {
    let relative = raw.starts_with("./") || raw.starts_with("../") || Path::new(raw).is_absolute();
    let base = match manifest {
        Some(m) if !relative => {
            if !m.root.is_dir() {
                return Err(Diagnostic::error(
                    "E0407",
                    format!("the package root `{}` does not exist", m.root.display()),
                    span,
                )
                .with_help(format!(
                    "create that folder, or change `root` in the `{}` package",
                    m.name
                )));
            }
            m.root.clone()
        }
        _ => dir.to_path_buf(),
    };
    let mut target = base.join(raw);
    if target.extension().is_none() {
        target.set_extension("kode");
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kode_diagnostics::Span;

    #[test]
    fn bare_imports_use_the_package_root() {
        let dir = std::env::temp_dir().join(format!("kode-root-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let m = Manifest {
            name: "pkg".into(),
            version: "0.1.0".into(),
            root: dir.join("missing"),
        };
        let err = resolve_import(&dir, Some(&m), "models/user", Span::default()).unwrap_err();
        assert_eq!(err.code, "E0407");

        std::fs::create_dir_all(&m.root).unwrap();
        let path = resolve_import(&dir, Some(&m), "models/user", Span::default()).unwrap();
        assert_eq!(path, m.root.join("models/user.kode"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

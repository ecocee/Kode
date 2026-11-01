//! Loading a program: read the file, follow its `import`s, parse everything,
//! and merge it into one `Program`.
//!
//! In v0.1 all modules share one namespace (`pub` is accepted but not enforced),
//! and a module may only contain functions, constants, classes and enums.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use kode_diagnostics::Diagnostic;
use kode_syntax::ast::{Item, Program};
use kode_syntax::{lexer, parser};

use crate::source::SourceMap;

pub struct Loaded {
    pub program: Program,
    pub map: SourceMap,
}

struct State {
    map: SourceMap,
    next_id: u32,
    stack: Vec<PathBuf>,
    loaded: HashSet<PathBuf>,
    names: HashMap<PathBuf, Vec<String>>,
    imported: Vec<Item>,
    errors: Vec<Diagnostic>,
}

pub fn load(path: &Path) -> Result<Loaded, (SourceMap, Vec<Diagnostic>)> {
    let mut st = State {
        map: SourceMap::default(),
        next_id: 0,
        stack: Vec::new(),
        loaded: HashSet::new(),
        names: HashMap::new(),
        imported: Vec::new(),
        errors: Vec::new(),
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
    })
}

fn defined_names(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f.name.clone()),
            Item::Const(c) => Some(c.name.clone()),
            Item::Class(c) => Some(c.name.clone()),
            Item::Enum(e) => Some(e.name.clone()),
            _ => None,
        })
        .collect()
}

fn load_file(path: &Path, st: &mut State) -> Result<Vec<Item>, Vec<Diagnostic>> {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = std::fs::read_to_string(path).map_err(|e| {
        vec![Diagnostic::error("E0401", format!("cannot read {}: {e}", path.display()), Default::default())]
    })?;
    let base = st.map.add(path.display().to_string(), text.clone());
    let tokens = lexer::lex_at(&text, base)?;
    let program = parser::parse_from(tokens, st.next_id)?;
    st.next_id = program.expr_count;
    st.names.insert(canon.clone(), defined_names(&program.items));

    st.stack.push(canon.clone());
    let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    for item in &program.items {
        let Item::Import(imp) = item else { continue };
        let mut target = dir.join(&imp.path);
        if target.extension().is_none() {
            target.set_extension("kode");
        }
        if !target.exists() {
            st.errors.push(
                Diagnostic::error("E0401", format!("cannot find the file `{}`", imp.path), imp.path_span)
                    .with_help(format!("looked for {}", target.display())),
            );
            continue;
        }
        let tcanon = target.canonicalize().unwrap_or_else(|_| target.clone());
        if st.stack.contains(&tcanon) {
            st.errors.push(Diagnostic::error(
                "E0402",
                format!("`{}` imports itself, directly or through other files", imp.path),
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
                                .with_help("a module may only contain fn, const, class and enum"),
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
            for (n, span) in &imp.names {
                if !defined.contains(n) {
                    st.errors.push(Diagnostic::error(
                        "E0405",
                        format!("`{}` does not define `{n}`", imp.path),
                        *span,
                    ));
                }
            }
        }
    }
    st.stack.pop();
    Ok(program.items)
}
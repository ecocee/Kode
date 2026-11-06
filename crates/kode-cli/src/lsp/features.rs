//! What the language server answers: diagnostics, hover, completion, document
//! symbols and formatting. All of it comes from the compiler's own front end.

use std::collections::HashSet;

use kode_diagnostics::{Diagnostic, Severity};
use kode_sema::Checked;
use kode_syntax::ast::*;

use super::json::Json;
use super::position::{position, word_at};
use super::walk;
use crate::source::SourceMap;

pub fn range(start: (u32, u32), end: (u32, u32)) -> Json {
    Json::obj(vec![
        ("start", Json::obj(vec![("line", Json::int(start.0 as i64)), ("character", Json::int(start.1 as i64))])),
        ("end", Json::obj(vec![("line", Json::int(end.0 as i64)), ("character", Json::int(end.1 as i64))])),
    ])
}

/// Diagnostics for one file, with positions the editor understands.
pub fn diagnostics_for(map: &SourceMap, doc_name: &str, diags: &[Diagnostic]) -> Vec<Json> {
    let mut out = Vec::new();
    for d in diags {
        let (file, local) = map.file_at(d.span.start);
        if file.name != doc_name {
            continue;
        }
        let start = position(&file.text, local);
        let end = position(&file.text, local + d.span.end.saturating_sub(d.span.start));
        let severity = match d.severity {
            Severity::Error => 1,
            Severity::Warning => 2,
        };
        let mut message = d.message.clone();
        if let Some(help) = &d.help {
            message.push_str(&format!("\nhelp: {help}"));
        }
        out.push(Json::obj(vec![
            ("range", range(start, end)),
            ("severity", Json::int(severity)),
            ("code", Json::str(d.code)),
            ("source", Json::str("kode")),
            ("message", Json::str(message)),
        ]));
    }
    out
}

/// The type of the expression under the cursor.
pub fn hover(program: &Program, checked: &Checked, map: &SourceMap, doc_name: &str, local: usize) -> Option<Json> {
    let base = map.start_of(doc_name)?;
    let at = base + local;
    let (id, span) = walk::innermost_at(&program.items, at)?;
    let (file, _) = map.file_at(span.start);
    if file.name != doc_name || span.end < base || span.end > base + file.text.len() {
        return None;
    }
    let ty = checked.expr_types.get(id as usize)?;
    if *ty == kode_sema::Type::Unit {
        return None;
    }
    let start = span.start - base;
    let end = span.end - base;
    if !file.text.is_char_boundary(start) || !file.text.is_char_boundary(end) {
        return None;
    }
    let source = &file.text[start..end];
    let label = if source.is_empty() || source.len() > 60 {
        String::new()
    } else {
        source.replace('\n', " ")
    };
    let body = if label.is_empty() {
        ty.name()
    } else {
        format!("{label}: {}", ty.name())
    };
    Some(Json::obj(vec![
        (
            "contents",
            Json::obj(vec![
                ("kind", Json::str("markdown")),
                ("value", Json::str(format!("```kode\n{body}\n```"))),
            ]),
        ),
        ("range", range(position(&file.text, start), position(&file.text, end))),
    ]))
}

const KEYWORDS: &[&str] = &[
    "and", "as", "assert", "await", "break", "catch", "class", "const", "continue", "else", "enum", "false", "fn",
    "for", "from", "if", "implements", "import", "in", "interface", "match", "none", "not", "or", "pub", "return",
    "spawn", "test", "throw", "true", "try", "while",
];

const TYPE_NAMES: &[&str] = &[
    "int", "float", "bool", "string", "Task", "Error", "Json", "Request", "Response", "Router",
];

/// Completion at a byte offset in the open document. `program` may be missing
/// (the buffer does not parse yet); names are then read from the text itself,
/// because an editor asks for completion while code is still being typed.
pub fn completion(program: Option<&Program>, text: &str, at: usize) -> Json {
    let at = at.min(text.len());
    let before = &text[..at];
    let (start, _) = word_at(text, at);
    let prefix = &before[start..];
    let after_dot = before[..start].trim_end().ends_with('.');
    let mut seen: HashSet<String> = HashSet::new();
    let mut entries: Vec<(String, i64, String)> = Vec::new();
    let mut push = |name: &str, kind: i64, detail: &str, seen: &mut HashSet<String>| {
        if !name.starts_with(prefix) || !seen.insert(name.to_string()) {
            return;
        }
        entries.push((name.to_string(), kind, detail.to_string()));
    };

    if after_dot {
        for f in kode_sema::std_fns::STD.iter().filter(|f| f.recv != kode_sema::std_fns::Recv::Free) {
            push(f.name, 2, "stdlib method", &mut seen);
        }
    } else {
        for k in KEYWORDS {
            push(k, 14, "keyword", &mut seen);
        }
        for name in kode_sema::builtins::BUILTIN_NAMES {
            push(name, 3, "builtin", &mut seen);
        }
        for t in TYPE_NAMES {
            push(t, 25, "type", &mut seen);
        }
        for f in kode_sema::std_fns::STD.iter().filter(|f| f.recv == kode_sema::std_fns::Recv::Free) {
            push(f.name, 3, "stdlib function", &mut seen);
        }
        if let Some(program) = program {
            for item in &program.items {
                match item {
                    Item::Fn(d) => push(&d.name, 3, "fn", &mut seen),
                    Item::Class(c) => push(&c.name, 7, "class", &mut seen),
                    Item::Interface(i) => push(&i.name, 8, "interface", &mut seen),
                    Item::Enum(e) => push(&e.name, 13, "enum", &mut seen),
                    Item::Const(c) => push(&c.name, 21, "const", &mut seen),
                    _ => {}
                }
            }
        }
        for (name, kind, detail) in declared_items(text) {
            push(&name, kind, detail, &mut seen);
        }
        for name in variables_before(text, at) {
            push(&name, 6, "variable", &mut seen);
        }
        // Module namespaces: `json.parse`, `math.sqrt`, ...
        for alias in ["json", "math", "fs", "time", "http"] {
            push(alias, 9, "namespace", &mut seen);
        }
    }

    let items: Vec<Json> = entries
        .into_iter()
        .map(|(label, kind, detail)| {
            Json::obj(vec![
                ("label", Json::str(label)),
                ("kind", Json::int(kind)),
                ("detail", Json::str(detail)),
            ])
        })
        .collect();
    Json::obj(vec![("isIncomplete", Json::Bool(false)), ("items", Json::Arr(items))])
}

/// Items declared in the text, whether or not the file parses. This is what
/// makes completion useful while a line is half written.
fn declared_items(text: &str) -> Vec<(String, i64, &'static str)> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_start();
        let line = line.strip_prefix("pub ").unwrap_or(line);
        let (kind, detail, rest) = if let Some(r) = line.strip_prefix("fn ") {
            (3, "fn", r)
        } else if let Some(r) = line.strip_prefix("class ") {
            (7, "class", r)
        } else if let Some(r) = line.strip_prefix("interface ") {
            (8, "interface", r)
        } else if let Some(r) = line.strip_prefix("enum ") {
            (13, "enum", r)
        } else if let Some(r) = line.strip_prefix("const ") {
            (21, "const", r)
        } else {
            continue;
        };
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !name.is_empty() {
            out.push((name, kind, detail));
        }
    }
    out
}

/// Names assigned in the document before `at`, so completion offers them.
fn variables_before(text: &str, at: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for raw in text[..at.min(text.len())].lines() {
        let line = raw.trim_start();
        if line.is_empty() || line.starts_with("//") || line.starts_with('*') {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let Some(first) = tokens.next() else { continue };
        let candidate = if first == "for" {
            // `for x in list` / `for i, x in list`
            tokens.next().map(|t| t.trim_end_matches(',').to_string())
        } else if first.contains(['=', ':', '.', '(', '"', '[']) || KEYWORDS.contains(&first) {
            None
        } else {
            let rest = line[first.len()..].trim_start();
            if rest.starts_with('=') || rest.starts_with(':') {
                Some(first.to_string())
            } else {
                None
            }
        };
        if let Some(name) = candidate {
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') && seen.insert(name.clone()) {
                out.push(name);
            }
        }
    }
    out
}

fn symbol(name: &str, kind: i64, span: kode_diagnostics::Span, text: &str, children: Vec<Json>) -> Json {
    let start = position(text, span.start);
    let end = position(text, span.end);
    let mut pairs = vec![
        ("name", Json::str(name)),
        ("kind", Json::int(kind)),
        ("range", range(start, end)),
        ("selectionRange", range(start, end)),
    ];
    if !children.is_empty() {
        pairs.push(("children", Json::Arr(children)));
    }
    Json::obj(pairs)
}

/// Document outline for the outline view and breadcrumbs.
pub fn symbols(text: &str) -> Json {
    let Ok(program) = kode_syntax::parse_source(text) else {
        return Json::Arr(Vec::new());
    };
    let mut out = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(d) => out.push(symbol(&d.name, 12, d.name_span, text, Vec::new())),
            Item::Test(t) => out.push(symbol(&format!("test \"{}\"", t.name), 12, t.name_span, text, Vec::new())),
            Item::Const(c) => out.push(symbol(&c.name, 14, c.name_span, text, Vec::new())),
            Item::Class(c) => {
                let mut children: Vec<Json> = c.fields.iter().map(|f| symbol(&f.name, 8, f.name_span, text, Vec::new())).collect();
                children.extend(c.methods.iter().map(|m| symbol(&m.name, 6, m.name_span, text, Vec::new())));
                out.push(symbol(&c.name, 5, c.name_span, text, children));
            }
            Item::Interface(i) => {
                let children: Vec<Json> = i.methods.iter().map(|m| symbol(&m.name, 6, m.name_span, text, Vec::new())).collect();
                out.push(symbol(&i.name, 11, i.name_span, text, children));
            }
            Item::Enum(e) => {
                let children: Vec<Json> = e.variants.iter().map(|(v, span)| symbol(v, 22, *span, text, Vec::new())).collect();
                out.push(symbol(&e.name, 10, e.name_span, text, children));
            }
            Item::Import(_) | Item::Stmt(_) => {}
        }
    }
    Json::Arr(out)
}

/// One full-document edit from `kode fmt`, so the editor and the CLI agree.
pub fn formatting(text: &str) -> Json {
    let formatted = crate::fmt::format_source(text);
    if formatted == text {
        return Json::Arr(Vec::new());
    }
    let end = position(text, text.len());
    Json::Arr(vec![Json::obj(vec![
        ("range", range((0, 0), end)),
        ("newText", Json::str(formatted)),
    ])])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_local_variables_and_items() {
        let text = "fn helper() -> int => 1\ntotal = 2\npri";
        let program = kode_syntax::parse_source(text).unwrap();
        let at = text.len();
        let result = completion(Some(&program), text, at);
        let items = result.arr_of("items");
        assert!(items.iter().any(|i| i.str_of("label") == Some("print")));
        assert!(!items.iter().any(|i| i.str_of("label") == Some("total")));
        let text2 = "total = 2\ntot";
        let result2 = completion(None, text2, text2.len());
        assert!(result2.arr_of("items").iter().any(|i| i.str_of("label") == Some("total")));
    }

    #[test]
    fn lists_methods_after_a_dot() {
        // `x.` does not parse, which is the normal state while someone types.
        let text = "x = \"abc\"\nx.";
        assert!(kode_syntax::parse_source(text).is_err());
        let result = completion(None, text, text.len());
        assert!(result.arr_of("items").iter().any(|i| i.str_of("label") == Some("trim")));
    }

    #[test]
    fn outlines_a_file() {
        let text = "const LIMIT = 3\nfn work(a: int) -> int => a\nclass Shop {\n    name: string\n    fn total() -> int => 1\n}\n";
        let symbols = symbols(text);
        let names: Vec<&str> = symbols.items().iter().filter_map(|s| s.str_of("name")).collect();
        assert_eq!(names, vec!["LIMIT", "work", "Shop"]);
        assert_eq!(symbols.items()[2].arr_of("children").len(), 2);
    }

    #[test]
    fn formats_a_whole_document() {
        let bad = "fn f() -> int {\nreturn 1\n}\n";
        let edits = formatting(bad);
        assert_eq!(edits.items().len(), 1);
        assert!(edits.items()[0].str_of("newText").unwrap().contains("    return 1"));
        assert!(formatting(&crate::fmt::format_source(bad)).items().is_empty());
    }
}

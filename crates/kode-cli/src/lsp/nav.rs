//! Go-to-definition, references, rename and signature help.
//!
//! Declarations are read from the source text, so the answers stay useful while
//! the buffer does not parse yet. The rule is deliberately simple: a name
//! resolves to the first `fn`/`class`/`const`/`enum`/`interface` declaration
//! with that name, or to the first assignment or annotation of it.

use super::features::range;
use super::json::Json;
use super::position;

fn identifier(s: &str) -> Option<&str> {
    let end = s
        .char_indices()
        .find(|(_, c)| !(c.is_alphanumeric() || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    if end == 0 {
        None
    } else {
        Some(&s[..end])
    }
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "fn" | "class"
            | "const"
            | "enum"
            | "interface"
            | "if"
            | "else"
            | "while"
            | "for"
            | "in"
            | "match"
            | "return"
            | "break"
            | "continue"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "spawn"
            | "await"
            | "none"
            | "true"
            | "false"
            | "and"
            | "or"
            | "not"
            | "int"
            | "float"
            | "bool"
            | "string"
            | "test"
            | "import"
            | "from"
            | "as"
            | "pub"
    )
}

fn location(uri: &str, text: &str, start: usize, end: usize) -> Json {
    Json::obj(vec![
        ("uri", Json::str(uri)),
        (
            "range",
            range(
                position::position(text, start),
                position::position(text, end),
            ),
        ),
    ])
}

/// Byte span of the declaration of `name`, if the text has one.
pub fn declaration_span(text: &str, name: &str) -> Option<(usize, usize)> {
    let mut off = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        let (body, skip) = match trimmed.strip_prefix("pub ") {
            Some(rest) => (rest, 4),
            None => (trimmed, 0),
        };
        let base = off + indent + skip;
        for kw in ["fn ", "class ", "const ", "enum ", "interface "] {
            if let Some(rest) = body.strip_prefix(kw) {
                if identifier(rest) == Some(name) {
                    return Some((base + kw.len(), base + kw.len() + name.len()));
                }
            }
        }
        if identifier(body) == Some(name) {
            let after = body[name.len()..].trim_start();
            if after.starts_with('=') || after.starts_with(':') {
                return Some((base, base + name.len()));
            }
        }
        off += line.len();
    }
    None
}

/// Every whole-word occurrence of `name` in `text`.
fn occurrences(text: &str, name: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    for (i, _) in text.match_indices(name) {
        let before_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
        let j = i + name.len();
        let after_ok = j == text.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_');
        if before_ok && after_ok {
            out.push((i, j));
        }
    }
    out
}

pub fn definition(text: &str, uri: &str, at: usize) -> Option<Json> {
    let (start, end) = position::word_at(text, at);
    if start == end {
        return None;
    }
    let (s, e) = declaration_span(text, &text[start..end])?;
    Some(location(uri, text, s, e))
}

pub fn references(text: &str, uri: &str, at: usize) -> Vec<Json> {
    let (start, end) = position::word_at(text, at);
    if start == end {
        return Vec::new();
    }
    let name = &text[start..end];
    occurrences(text, name)
        .into_iter()
        .map(|(s, e)| location(uri, text, s, e))
        .collect()
}

pub fn rename(text: &str, uri: &str, at: usize, new_name: &str) -> Option<Json> {
    if !is_identifier(new_name) || is_keyword(new_name) {
        return None;
    }
    let (start, end) = position::word_at(text, at);
    if start == end {
        return None;
    }
    let name = &text[start..end];
    if is_keyword(name) || declaration_span(text, name).is_none() {
        return None;
    }
    let edits: Vec<Json> = occurrences(text, name)
        .into_iter()
        .map(|(s, e)| {
            Json::obj(vec![
                (
                    "range",
                    range(position::position(text, s), position::position(text, e)),
                ),
                ("newText", Json::str(new_name)),
            ])
        })
        .collect();
    if edits.is_empty() {
        return None;
    }
    Some(Json::obj(vec![(
        "changes",
        Json::obj(vec![(uri, Json::Arr(edits))]),
    )]))
}

/// The call whose argument list contains `at`, and how many arguments are
/// already complete before it.
fn enclosing_call(text: &str, at: usize) -> Option<(String, usize)> {
    let before = &text[..at.min(text.len())];
    let mut depth = 0i32;
    let mut commas = 0usize;
    for (idx, ch) in before.char_indices().rev() {
        match ch {
            ')' => depth += 1,
            '(' if depth == 0 => {
                let (s, e) = position::word_at(before, idx);
                if s == e {
                    return None;
                }
                return Some((before[s..e].to_string(), commas));
            }
            '(' => depth -= 1,
            ',' if depth == 0 => commas += 1,
            _ => {}
        }
    }
    None
}

/// The parameters of the first `fn name(...)` declaration in the text.
fn declared_params(text: &str, name: &str) -> Option<Vec<String>> {
    let needle = format!("fn {name}");
    let pos = text.find(&needle)?;
    let open = pos + text[pos..].find('(')?;
    let mut depth = 0i32;
    let mut close = None;
    for (i, ch) in text[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let inner = text[open + 1..close].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        split_top_level(inner)
            .into_iter()
            .map(|s| s.trim().to_string())
            .collect(),
    )
}

fn split_top_level(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[start..]);
    out
}

pub fn signature_help(text: &str, at: usize) -> Option<Json> {
    let (name, active) = enclosing_call(text, at)?;
    let params = declared_params(text, &name)?;
    let label = format!("fn {name}({})", params.join(", "));
    let parameters: Vec<Json> = params
        .iter()
        .map(|p| Json::obj(vec![("label", Json::str(p))]))
        .collect();
    Some(Json::obj(vec![
        (
            "signatures",
            Json::Arr(vec![Json::obj(vec![
                ("label", Json::str(label)),
                ("parameters", Json::Arr(parameters)),
            ])]),
        ),
        ("activeSignature", Json::int(0)),
        ("activeParameter", Json::int(active as i64)),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "const LIMIT = 3\n\nfn add(a: int, b: int) -> int {\n    return a + b\n}\n\nprint(add(1, 2))\n";

    #[test]
    fn finds_the_declaration_of_a_call() {
        let at = SRC.find("add(1").unwrap() + 1;
        let loc = definition(SRC, "file:///t.kode", at).unwrap();
        let start = loc.get("range").unwrap().get("start").unwrap();
        assert_eq!(start.int_of("line"), Some(2));
        assert_eq!(start.int_of("character"), Some(3));
    }

    #[test]
    fn lists_every_use() {
        let at = SRC.find("add(1").unwrap() + 1;
        let found = references(SRC, "file:///t.kode", at);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn renames_a_whole_document() {
        let at = SRC.find("add(1").unwrap() + 1;
        let edit = rename(SRC, "file:///t.kode", at, "sum").unwrap();
        let changes = edit.get("changes").unwrap().get("file:///t.kode").unwrap();
        assert_eq!(changes.items().len(), 2);
        assert_eq!(changes.items()[0].str_of("newText"), Some("sum"));
    }

    #[test]
    fn refuses_to_rename_a_keyword() {
        let at = SRC.find("fn add").unwrap() + 1;
        assert!(rename(SRC, "file:///t.kode", at, "let").is_none());
    }

    #[test]
    fn describes_the_call_being_typed() {
        let at = SRC.find("add(1,").unwrap() + "add(1,".len();
        let help = signature_help(SRC, at).unwrap();
        assert_eq!(help.int_of("activeParameter"), Some(1));
        let sig = &help.get("signatures").unwrap().items()[0];
        assert_eq!(sig.str_of("label"), Some("fn add(a: int, b: int)"));
        assert_eq!(sig.get("parameters").unwrap().items().len(), 2);
    }
}

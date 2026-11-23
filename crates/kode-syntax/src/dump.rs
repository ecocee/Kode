//! Stable dumps of the token stream and the syntax tree.
//!
//! `kode tokens` and `kode ast` are the oracles a self-hosted compiler is
//! compared against (`docs/PLAN-self-hosting.md` §5), so this format is a
//! compatibility surface: add fields, never rename or reorder them (AGENTS.md 9).
//!
//! Rules that the port depends on:
//!
//! * one token per line: `start..end`, `line:column`, kind, then its text, with
//!   `\t`, `\n`, `\r`, `"` and `\` escaped so a kind is always one line;
//! * one node per line in a tree: kind, then `attr=value` pairs, then children
//!   indented by two spaces;
//! * `--format=json` prints the same fields as JSON, in the same order;
//! * expression ids are *left out on purpose*: they are allocation order, which
//!   a port is not required to reproduce.

pub mod ast;

use std::fmt::Write;

use kode_diagnostics::Span;

use crate::ast::Program;
use crate::token::Token;

/// A value attached to a [`Node`].
#[derive(Clone, Debug)]
pub enum Value {
    /// Text: quoted in both formats, so spaces and quotes are unambiguous.
    Text(String),
    Num(i64),
    Flag(bool),
    /// Already canonical text that is never quoted: a span or an operator.
    Raw(String),
}

/// One node of a dump tree: a kind, its attributes in a fixed order, children.
#[derive(Clone, Debug)]
pub struct Node {
    pub kind: &'static str,
    pub attrs: Vec<(&'static str, Value)>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(kind: &'static str) -> Self {
        Node {
            kind,
            attrs: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn text(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.attrs.push((key, Value::Text(value.into())));
        self
    }

    pub fn num(mut self, key: &'static str, value: i64) -> Self {
        self.attrs.push((key, Value::Num(value)));
        self
    }

    pub fn flag(mut self, key: &'static str, value: bool) -> Self {
        self.attrs.push((key, Value::Flag(value)));
        self
    }

    pub fn raw(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.attrs.push((key, Value::Raw(value.into())));
        self
    }

    pub fn span(self, key: &'static str, span: Span) -> Self {
        self.raw(key, format!("{}..{}", span.start, span.end))
    }

    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub fn kids(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }
}

/// The token stream as text, one token per line.
pub fn tokens_text(src: &str, tokens: &[Token]) -> String {
    let mut out = String::new();
    for token in tokens {
        let (line, column) = line_column(src, token.span.start);
        let _ = writeln!(
            out,
            "{}..{}\t{line}:{column}\t{}\t{}",
            token.span.start,
            token.span.end,
            token.kind.name(),
            escape(text_of(src, token.span))
        );
    }
    out
}

/// The same token stream as a JSON array, one token per line.
pub fn tokens_json(src: &str, tokens: &[Token]) -> String {
    let mut out = String::from("[\n");
    for (i, token) in tokens.iter().enumerate() {
        let (line, column) = line_column(src, token.span.start);
        let _ = write!(
            out,
            "  {{\"kind\":\"{}\",\"text\":\"{}\",\"span\":{},\"line\":{line},\"column\":{column}}}",
            token.kind.name(),
            json_escape(text_of(src, token.span)),
            span_json(token.span)
        );
        out.push_str(if i + 1 < tokens.len() { ",\n" } else { "\n" });
    }
    out.push_str("]\n");
    out
}

/// The syntax tree as indented text.
pub fn ast_text(program: &Program) -> String {
    let mut out = String::new();
    write_text(&ast::program(program), 0, &mut out);
    out
}

/// The syntax tree as JSON.
pub fn ast_json(program: &Program) -> String {
    let mut out = String::new();
    write_json(&ast::program(program), 0, &mut out);
    out.push('\n');
    out
}

/// The source text a span covers, or `""` if it is out of range.
fn text_of(src: &str, span: Span) -> &str {
    src.get(span.start..span.end).unwrap_or("")
}

/// 1-based line and column of a byte offset. Columns count characters, not
/// bytes, so a program with a multi-byte character has the same columns
/// everywhere.
fn line_column(src: &str, offset: usize) -> (usize, usize) {
    let before = src.get(..offset).unwrap_or("");
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(1, |l| l.chars().count() + 1);
    (line, column)
}

fn span_json(span: Span) -> String {
    format!("{{\"start\":{},\"end\":{}}}", span.start, span.end)
}

/// Escape the characters that would break the one-line rule, in the text form.
fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// Escape for JSON. Control characters must be `\u00XX`, not `\u{X}`.
fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn write_text(node: &Node, depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(node.kind);
    for (key, value) in &node.attrs {
        out.push(' ');
        out.push_str(key);
        out.push('=');
        match value {
            Value::Text(s) => {
                out.push('"');
                out.push_str(&escape(s));
                out.push('"');
            }
            Value::Num(n) => {
                let _ = write!(out, "{n}");
            }
            Value::Flag(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Raw(s) => out.push_str(s),
        }
    }
    out.push('\n');
    for child in &node.children {
        write_text(child, depth + 1, out);
    }
}

fn write_json(node: &Node, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let _ = write!(out, "{pad}{{\"kind\":\"{}\"", node.kind);
    if !node.attrs.is_empty() {
        out.push_str(",\"attrs\":{");
        for (i, (key, value)) in node.attrs.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "\"{key}\":");
            match value {
                Value::Text(s) | Value::Raw(s) => {
                    let _ = write!(out, "\"{}\"", json_escape(s));
                }
                Value::Num(n) => {
                    let _ = write!(out, "{n}");
                }
                Value::Flag(b) => out.push_str(if *b { "true" } else { "false" }),
            }
        }
        out.push('}');
    }
    if node.children.is_empty() {
        out.push('}');
        return;
    }
    out.push_str(",\"children\":[\n");
    for (i, child) in node.children.iter().enumerate() {
        write_json(child, depth + 1, out);
        if i + 1 < node.children.len() {
            out.push(',');
        }
        out.push('\n');
    }
    let _ = write!(out, "{pad}]}}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn tokens_carry_their_text_and_place() {
        let src = "x = 1\n";
        let tokens = lex(src).expect("lexes");
        assert_eq!(
            tokens_text(src, &tokens),
            "0..1\t1:1\tIdent\tx\n\
             2..3\t1:3\tAssign\t=\n\
             4..5\t1:5\tInt\t1\n\
             5..6\t1:6\tNewline\t\\n\n\
             6..6\t2:1\tEof\t\n"
        );
    }

    #[test]
    fn a_tab_and_a_quote_do_not_break_the_line() {
        let src = "s = \"a\\tb\"\n";
        let tokens = lex(src).expect("lexes");
        let text = tokens_text(src, &tokens);
        assert_eq!(text.lines().count(), tokens.len(), "{text}");
        for line in text.lines() {
            assert_eq!(line.matches('\t').count(), 3, "{line}");
        }
        assert_eq!(escape("a\tb\nc\"d\\e"), "a\\tb\\nc\\\"d\\\\e");
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        // `é` is two bytes, `𝄞` is four: both are one column wide.
        let src = "a = \"é𝄞\"\nb = 1\n";
        let tokens = lex(src).expect("lexes");
        let text = tokens_text(src, &tokens);
        let second_line = text
            .lines()
            .find(|l| l.contains("Ident\tb"))
            .unwrap_or_default();
        assert!(second_line.starts_with("13..14\t2:1\t"), "{text}");
    }

    #[test]
    fn json_escapes_control_characters() {
        assert_eq!(
            json_escape("a\tb\n\"c\\d\u{1}"),
            "a\\tb\\n\\\"c\\\\d\\u0001"
        );
    }

    #[test]
    fn the_tree_keeps_spans_but_not_allocation_order() {
        let program =
            crate::parse_source("fn f(x: int) -> int {\n    return x\n}\n").expect("parses");
        let text = ast_text(&program);
        assert!(text.starts_with("Program expr_count="), "{text}");
        assert!(
            text.contains("Fn name=\"f\" name_span=3..4 pub=false span=0.."),
            "{text}"
        );
        assert!(text.contains("TypeName name=\"int\""), "{text}");
        assert!(!text.contains("id="), "{text}");
    }

    #[test]
    fn both_forms_describe_the_same_tree() {
        let program = crate::parse_source("x = [1, 2]\n").expect("parses");
        let text = ast_text(&program);
        let json = ast_json(&program);

        let text_kinds: Vec<&str> = text
            .lines()
            .filter_map(|line| line.trim_start().split(' ').next())
            .collect();
        let json_kinds: Vec<&str> = json
            .match_indices("\"kind\":\"")
            .map(|(i, _)| {
                let rest = &json[i + 8..];
                &rest[..rest.find('"').unwrap()]
            })
            .collect();

        assert_eq!(text_kinds, json_kinds, "text:\n{text}\njson:\n{json}");
        assert!(
            text_kinds.starts_with(&["Program", "Assign", "TargetVar"]),
            "{text}"
        );
    }
}

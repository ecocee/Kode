use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// A compiler message with a stable code such as `E0201`.
/// It can be shown to people or printed as JSON for editors and AI agents.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

struct Location<'a> {
    line_no: usize,
    column: usize,
    line_text: &'a str,
    width: usize,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Diagnostic {
            severity: Severity::Error,
            code,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            code,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    fn locate<'a>(&self, src: &'a str) -> Location<'a> {
        let mut offset = self.span.start.min(src.len());
        while !src.is_char_boundary(offset) {
            offset -= 1;
        }
        let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = src[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(src.len());
        let line_no = src[..offset].matches('\n').count() + 1;
        let column = src[line_start..offset].chars().count() + 1;
        let mut end = self.span.end.min(line_end).max(offset);
        while !src.is_char_boundary(end) {
            end -= 1;
        }
        Location {
            line_no,
            column,
            line_text: src[line_start..line_end].trim_end_matches('\r'),
            width: src[offset..end].chars().count().max(1),
        }
    }

    /// Human readable form, similar to rustc.
    pub fn render(&self, file: &str, src: &str) -> String {
        let loc = self.locate(src);
        let num = loc.line_no.to_string();
        let pad = " ".repeat(num.len());
        let mut out = format!(
            "{}[{}]: {}\n{pad}--> {file}:{}:{}\n{pad} |\n{num} | {}\n{pad} | {}{}\n",
            self.severity.as_str(),
            self.code,
            self.message,
            loc.line_no,
            loc.column,
            loc.line_text,
            " ".repeat(loc.column - 1),
            "^".repeat(loc.width)
        );
        if let Some(help) = &self.help {
            out.push_str(&format!("{pad} = help: {help}\n"));
        }
        out
    }

    /// Machine readable form for AI agents and editors.
    pub fn to_json(&self, file: &str, src: &str) -> String {
        let loc = self.locate(src);
        let help = match &self.help {
            Some(h) => format!("\"{}\"", json_escape(h)),
            None => "null".to_string(),
        };
        format!(
            "{{\"severity\":\"{}\",\"code\":\"{}\",\"message\":\"{}\",\"file\":\"{}\",\"line\":{},\"column\":{},\"end_column\":{},\"help\":{}}}",
            self.severity.as_str(),
            self.code,
            json_escape(&self.message),
            json_escape(file),
            loc.line_no,
            loc.column,
            loc.column + loc.width,
            help
        )
    }
}

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

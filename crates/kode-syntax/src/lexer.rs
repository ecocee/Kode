use kode_diagnostics::{Diagnostic, Span};

use crate::token::{keyword, StrPart, Token, TokenKind};

/// Turn source text into tokens. Collects every error it finds.
pub fn lex(src: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    lex_at(src, 0)
}

/// Like `lex`, but every span is shifted by `base`. Used for `${...}` holes,
/// which are lexed separately but must point at the right place in the file.
pub fn lex_at(src: &str, base: usize) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut lx = Lexer {
        src,
        i: 0,
        tokens: Vec::new(),
        errors: Vec::new(),
    };
    lx.run();
    lx.finish(base)
}

struct Lexer<'a> {
    src: &'a str,
    i: usize,
    tokens: Vec<Token>,
    errors: Vec<Diagnostic>,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_part(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    /// Character `n` bytes ahead (only used after ASCII characters).
    fn peek_at(&self, n: usize) -> Option<char> {
        self.src.get(self.i + n..).and_then(|s| s.chars().next())
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(start, end),
        });
    }

    fn error(&mut self, code: &'static str, msg: impl Into<String>, start: usize, end: usize) -> &mut Diagnostic {
        self.errors
            .push(Diagnostic::error(code, msg, Span::new(start, end)));
        self.errors.last_mut().unwrap()
    }

    /// One or two character operator: `second` after the first char picks `yes`.
    fn one_or_two(&mut self, second: char, yes: TokenKind, no: TokenKind) {
        let start = self.i;
        if self.peek_at(1) == Some(second) {
            self.push(yes, start, start + 2);
            self.i += 2;
        } else {
            self.push(no, start, start + 1);
            self.i += 1;
        }
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.i;
        self.push(kind, start, start + 1);
        self.i += 1;
    }

    fn run(&mut self) {
        while let Some(c) = self.peek() {
            let start = self.i;
            match c {
                ' ' | '\t' | '\r' => self.i += 1,
                '\n' => self.single(TokenKind::Newline),
                '(' => self.single(TokenKind::LParen),
                ')' => self.single(TokenKind::RParen),
                '{' => self.single(TokenKind::LBrace),
                '}' => self.single(TokenKind::RBrace),
                '[' => self.single(TokenKind::LBracket),
                ']' => self.single(TokenKind::RBracket),
                '?' => match self.peek_at(1) {
                    Some('?') => {
                        self.push(TokenKind::QuestionQuestion, start, start + 2);
                        self.i += 2;
                    }
                    Some('.') => {
                        self.push(TokenKind::QuestionDot, start, start + 2);
                        self.i += 2;
                    }
                    _ => self.single(TokenKind::Question),
                },
                ',' => self.single(TokenKind::Comma),
                ':' => self.single(TokenKind::Colon),
                '+' => self.one_or_two('=', TokenKind::PlusEq, TokenKind::Plus),
                '*' => self.one_or_two('=', TokenKind::StarEq, TokenKind::Star),
                '%' => self.one_or_two('=', TokenKind::PercentEq, TokenKind::Percent),
                '&' => self.one_or_two('=', TokenKind::AmpEq, TokenKind::Amp),
                '|' => self.one_or_two('=', TokenKind::PipeEq, TokenKind::Pipe),
                '^' => self.one_or_two('=', TokenKind::CaretEq, TokenKind::Caret),
                '~' => self.single(TokenKind::Tilde),
                '<' => self.less(),
                '>' => self.greater(),
                '-' => match self.peek_at(1) {
                    Some('>') => {
                        self.push(TokenKind::Arrow, start, start + 2);
                        self.i += 2;
                    }
                    Some('=') => {
                        self.push(TokenKind::MinusEq, start, start + 2);
                        self.i += 2;
                    }
                    _ => self.single(TokenKind::Minus),
                },
                '=' => match self.peek_at(1) {
                    Some('=') => {
                        self.push(TokenKind::EqEq, start, start + 2);
                        self.i += 2;
                    }
                    Some('>') => {
                        self.push(TokenKind::FatArrow, start, start + 2);
                        self.i += 2;
                    }
                    _ => self.single(TokenKind::Assign),
                },
                '!' => {
                    if self.peek_at(1) == Some('=') {
                        self.push(TokenKind::NotEq, start, start + 2);
                        self.i += 2;
                    } else {
                        self.single(TokenKind::Bang);
                    }
                }
                '.' => {
                    if self.src[start..].starts_with("..=") {
                        self.push(TokenKind::DotDotEq, start, start + 3);
                        self.i += 3;
                    } else if self.src[start..].starts_with("..") {
                        self.push(TokenKind::DotDot, start, start + 2);
                        self.i += 2;
                    } else {
                        self.single(TokenKind::Dot);
                    }
                }
                '/' => self.slash(),
                '"' => self.string(),
                c if c.is_ascii_digit() => self.number(),
                c if is_ident_start(c) => {
                    while self.peek().map(is_ident_part).unwrap_or(false) {
                        self.i += 1;
                    }
                    let word = &self.src[start..self.i];
                    let kind = keyword(word).unwrap_or_else(|| TokenKind::Ident(word.to_string()));
                    self.push(kind, start, self.i);
                }
                ';' => {
                    self.error("E0007", "unexpected character `;`", start, start + 1)
                        .help = Some("Kode does not use semicolons; end a statement with a new line".into());
                    self.i += 1;
                }
                other => {
                    let len = other.len_utf8();
                    self.error("E0007", format!("unexpected character `{other}`"), start, start + len);
                    self.i += len;
                }
            }
        }
        let end = self.src.len();
        self.push(TokenKind::Eof, end, end);
    }

    /// `<`, `<=`, `<<` and `<<=`
    fn less(&mut self) {
        let start = self.i;
        match self.peek_at(1) {
            Some('<') if self.peek_at(2) == Some('=') => {
                self.push(TokenKind::ShlEq, start, start + 3);
                self.i += 3;
            }
            Some('<') => {
                self.push(TokenKind::Shl, start, start + 2);
                self.i += 2;
            }
            Some('=') => {
                self.push(TokenKind::Le, start, start + 2);
                self.i += 2;
            }
            _ => self.single(TokenKind::Lt),
        }
    }

    /// `>`, `>=`, `>>` and `>>=`
    fn greater(&mut self) {
        let start = self.i;
        match self.peek_at(1) {
            Some('>') if self.peek_at(2) == Some('=') => {
                self.push(TokenKind::ShrEq, start, start + 3);
                self.i += 3;
            }
            Some('>') => {
                self.push(TokenKind::Shr, start, start + 2);
                self.i += 2;
            }
            Some('=') => {
                self.push(TokenKind::Ge, start, start + 2);
                self.i += 2;
            }
            _ => self.single(TokenKind::Gt),
        }
    }

    fn slash(&mut self) {
        let start = self.i;
        match self.peek_at(1) {
            Some('/') => {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.i += c.len_utf8();
                }
            }
            Some('*') => match self.src[start + 2..].find("*/") {
                Some(rel) => self.i = start + 2 + rel + 2,
                None => {
                    self.error("E0004", "this block comment is never closed", start, start + 2)
                        .help = Some("end it with `*/`".into());
                    self.i = self.src.len();
                }
            },
            Some('=') => {
                self.push(TokenKind::SlashEq, start, start + 2);
                self.i += 2;
            }
            _ => self.single(TokenKind::Slash),
        }
    }

    fn eat_digits(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '_') {
            self.i += 1;
        }
    }

    fn number(&mut self) {
        let start = self.i;
        let mut is_float = false;
        self.eat_digits();
        // `1.5` is a float, but `1..5` is a range.
        if self.peek() == Some('.') && matches!(self.peek_at(1), Some(c) if c.is_ascii_digit()) {
            is_float = true;
            self.i += 1;
            self.eat_digits();
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let n1 = self.peek_at(1);
            let n2 = self.peek_at(2);
            let digit = |c: Option<char>| matches!(c, Some(c) if c.is_ascii_digit());
            if digit(n1) {
                is_float = true;
                self.i += 1;
                self.eat_digits();
            } else if matches!(n1, Some('+') | Some('-')) && digit(n2) {
                is_float = true;
                self.i += 2;
                self.eat_digits();
            }
        }
        if matches!(self.peek(), Some(c) if is_ident_part(c)) {
            while matches!(self.peek(), Some(c) if is_ident_part(c)) {
                self.i += 1;
            }
            let text = self.src[start..self.i].to_string();
            self.error("E0006", format!("`{text}` is not a valid number"), start, self.i)
                .help = Some("write numbers like 42, 1_000, 3.14 or 2.5e3".into());
            return;
        }
        let text: String = self.src[start..self.i].chars().filter(|c| *c != '_').collect();
        if is_float {
            match text.parse::<f64>() {
                Ok(v) => self.push(TokenKind::Float(v), start, self.i),
                Err(_) => {
                    self.error("E0006", "this is not a valid number", start, self.i);
                }
            }
        } else {
            match text.parse::<i64>() {
                Ok(v) => self.push(TokenKind::Int(v), start, self.i),
                Err(_) => {
                    self.error("E0005", "this integer is too large", start, self.i)
                        .help = Some("int is a 64-bit signed integer (max 9223372036854775807)".into());
                }
            }
        }
    }

    fn string(&mut self) {
        let start = self.i;
        self.i += 1;
        let mut parts: Vec<StrPart> = Vec::new();
        let mut cur = String::new();
        let mut closed = false;

        while let Some(ch) = self.peek() {
            let len = ch.len_utf8();
            match ch {
                '"' => {
                    self.i += len;
                    closed = true;
                    break;
                }
                '\n' => break,
                '\\' => match self.peek_at(1) {
                    Some(e @ ('n' | 't' | 'r' | '0' | '"' | '\\' | '$')) => {
                        cur.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '0' => '\0',
                            other => other,
                        });
                        self.i += 2;
                    }
                    Some(other) if other != '\n' => {
                        let end = self.i + 1 + other.len_utf8();
                        self.error("E0002", format!("unknown escape `\\{other}`"), self.i, end)
                            .help = Some("valid escapes are \\n \\t \\r \\0 \\\" \\\\ and \\$".into());
                        self.i = end;
                    }
                    _ => self.i += 1,
                },
                '$' if self.peek_at(1) == Some('{') => {
                    let code_start = self.i + 2;
                    match find_hole_end(self.src, code_start) {
                        Some(end) => {
                            if !cur.is_empty() {
                                parts.push(StrPart::Lit(std::mem::take(&mut cur)));
                            }
                            parts.push(StrPart::Code {
                                src: self.src[code_start..end].to_string(),
                                offset: code_start,
                            });
                            self.i = end + 1;
                        }
                        None => {
                            self.error("E0003", "this `${` is never closed", self.i, self.i + 2)
                                .help = Some("close it with `}` on the same line".into());
                            self.i += 2;
                        }
                    }
                }
                _ => {
                    cur.push(ch);
                    self.i += len;
                }
            }
        }

        if closed {
            if !cur.is_empty() || parts.is_empty() {
                parts.push(StrPart::Lit(cur));
            }
            let end = self.i;
            self.push(TokenKind::Str(parts), start, end);
        } else {
            self.error("E0001", "this string is never closed", start, start + 1)
                .help = Some("add a closing `\"` at the end of the line".into());
        }
    }

    fn finish(mut self, base: usize) -> Result<Vec<Token>, Vec<Diagnostic>> {
        if base != 0 {
            for t in &mut self.tokens {
                t.span = t.span.shift(base);
                if let TokenKind::Str(parts) = &mut t.kind {
                    for p in parts {
                        if let StrPart::Code { offset, .. } = p {
                            *offset += base;
                        }
                    }
                }
            }
            for e in &mut self.errors {
                e.span = e.span.shift(base);
            }
        }
        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }
}

/// Find the `}` that closes a `${` hole, skipping nested braces and strings.
fn find_hole_end(src: &str, from: usize) -> Option<usize> {
    let mut depth = 1;
    let mut chars = src[from..].char_indices().peekable();
    while let Some((rel, c)) = chars.next() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(from + rel);
                }
            }
            '"' => loop {
                match chars.next() {
                    Some((_, '\\')) => {
                        chars.next();
                    }
                    Some((_, '"')) => break,
                    Some((_, '\n')) | None => return None,
                    _ => {}
                }
            },
            '\n' => return None,
            _ => {}
        }
    }
    None
}
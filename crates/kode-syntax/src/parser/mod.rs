//! Recursive descent parser with Pratt parsing for expressions.

mod expr;
mod item;
mod stmt;

use std::mem::discriminant;

use kode_diagnostics::{Diagnostic, Span};

use crate::ast::*;
use crate::token::{Token, TokenKind};

/// Parse tokens into a program. Collects several errors when it can.
pub fn parse(tokens: Vec<Token>) -> Result<Program, Vec<Diagnostic>> {
    parse_from(tokens, 0)
}

/// Like `parse`, but expression ids start at `first_id`. Used when several
/// files (modules) are merged into one program.
pub fn parse_from(tokens: Vec<Token>, first_id: u32) -> Result<Program, Vec<Diagnostic>> {
    let mut p = Parser::new(tokens, first_id);
    let items = p.parse_items();
    if p.errors.is_empty() {
        Ok(Program {
            items,
            expr_count: p.next_id,
        })
    } else {
        Err(p.errors)
    }
}

pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub(crate) next_id: u32,
    pub(crate) errors: Vec<Diagnostic>,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>, next_id: u32) -> Self {
        Parser {
            tokens,
            pos: 0,
            next_id,
            errors: Vec::new(),
        }
    }

    pub(crate) fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    pub(crate) fn peek_at(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| &t.kind)
    }

    pub(crate) fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    /// Is the next token the same kind as `kind` (ignoring any data inside)?
    pub(crate) fn at(&self, kind: &TokenKind) -> bool {
        discriminant(self.peek()) == discriminant(kind)
    }

    pub(crate) fn bump(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if !matches!(tok.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn skip_newlines(&mut self) {
        while matches!(self.peek(), TokenKind::Newline) {
            self.pos += 1;
        }
    }

    /// After an error, skip to the end of the line so parsing can continue.
    pub(crate) fn recover(&mut self) {
        while !matches!(self.peek(), TokenKind::Newline | TokenKind::Eof) {
            self.pos += 1;
        }
    }

    pub(crate) fn mk(&mut self, span: Span, kind: ExprKind) -> Expr {
        let id = self.next_id;
        self.next_id += 1;
        Expr { id, span, kind }
    }

    pub(crate) fn expect(&mut self, kind: TokenKind, what: &str) -> Result<Token, Diagnostic> {
        if self.at(&kind) {
            Ok(self.bump())
        } else {
            Err(Diagnostic::error(
                "E0103",
                format!("expected {what}, found {}", self.peek().describe()),
                self.span(),
            ))
        }
    }

    pub(crate) fn ident(&mut self, what: &str) -> Result<(String, Span), Diagnostic> {
        match self.peek().clone() {
            TokenKind::Ident(n) => {
                let s = self.bump().span;
                Ok((n, s))
            }
            other => Err(Diagnostic::error(
                "E0110",
                format!("expected {what}, found {}", other.describe()),
                self.span(),
            )),
        }
    }

    /// A statement must end at a new line (or at `}` / the end of the file).
    pub(crate) fn expect_end(&mut self) -> Result<(), Diagnostic> {
        match self.peek() {
            TokenKind::Newline | TokenKind::Eof | TokenKind::RBrace => Ok(()),
            other => Err(Diagnostic::error(
                "E0101",
                format!("expected the end of the line, found {}", other.describe()),
                self.span(),
            )
            .with_help("put each statement on its own line")),
        }
    }

    pub(crate) fn type_ann(&mut self) -> Result<TypeAnn, Diagnostic> {
        let start = self.span();
        let kind = match self.peek().clone() {
            TokenKind::Ident(name) if name == "Task" && matches!(self.peek_at(1), Some(TokenKind::Lt)) => {
                self.bump();
                self.bump(); // <
                let inner = self.type_ann()?;
                self.expect(TokenKind::Gt, "`>`")?;
                TypeAnnKind::Task(Box::new(inner))
            }
            TokenKind::Ident(name) => {
                self.bump();
                TypeAnnKind::Name(name)
            }
            TokenKind::LBracket => {
                self.bump();
                let inner = self.type_ann()?;
                self.expect(TokenKind::RBracket, "`]`")?;
                TypeAnnKind::List(Box::new(inner))
            }
            TokenKind::LBrace => {
                self.bump();
                let key = self.type_ann()?;
                if !matches!(&key.kind, TypeAnnKind::Name(n) if n == "string") {
                    return Err(Diagnostic::error("E0115", "map keys must be strings", key.span)
                        .with_help("write it like `{string: int}`"));
                }
                self.expect(TokenKind::Colon, "`:`")?;
                let value = self.type_ann()?;
                self.expect(TokenKind::RBrace, "`}`")?;
                TypeAnnKind::Map(Box::new(value))
            }
            other => {
                return Err(Diagnostic::error(
                    "E0111",
                    format!("expected a type name, found {}", other.describe()),
                    self.span(),
                )
                .with_help("types look like int, [int], {string: int} or User?"))
            }
        };
        let mut ann = TypeAnn { kind, span: start };
        if matches!(self.peek(), TokenKind::Question) {
            let q = self.bump().span;
            ann = TypeAnn {
                span: start.join(q),
                kind: TypeAnnKind::Opt(Box::new(ann)),
            };
        }
        Ok(ann)
    }
}
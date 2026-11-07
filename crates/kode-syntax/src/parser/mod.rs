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
        let program = Program {
            items,
            expr_count: p.next_id,
        };
        // The parser guards its own recursion; this catches trees that a loop
        // built (a long `+` chain is left-deep) before the type checker sees
        // them.
        if let Some(span) = crate::depth::too_deep(&program) {
            return Err(vec![crate::depth::diagnostic(span)]);
        }
        Ok(program)
    } else {
        Err(p.errors)
    }
}

/// The deepest nesting the parser accepts, counted in expressions, statements
/// and types as it recurses.
///
/// Recursive descent uses a few frames per level, and the type checker, MIR
/// lowering and the code generator recurse over the same tree later. Without a
/// limit, a few hundred levels of `(` exhaust the 8 MB main stack and abort the
/// compiler with no diagnostic (see `docs/STATUS.md`). 128 levels is far more
/// than hand-written code needs and far less than any of those passes can take.
pub(crate) const MAX_NESTING: u32 = 128;

pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub(crate) next_id: u32,
    pub(crate) errors: Vec<Diagnostic>,
    /// Current recursion depth. Guarded by [`MAX_NESTING`].
    depth: u32,
    /// Set by the first depth error: from then on the tree is unreliable, so
    /// parsing stops rather than reporting every enclosing level.
    pub(crate) fatal: bool,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>, next_id: u32) -> Self {
        Parser {
            tokens,
            pos: 0,
            next_id,
            errors: Vec::new(),
            depth: 0,
            fatal: false,
        }
    }

    /// The diagnostic for nesting past [`MAX_NESTING`]. Emitted wherever a
    /// recursive production would go one level deeper.
    pub(crate) fn too_deep(&mut self) -> Diagnostic {
        self.fatal = true;
        Diagnostic::error(
            "E0118",
            format!("this nests deeper than {MAX_NESTING} levels"),
            self.span(),
        )
        .with_help("split it up: bind part of it to a variable on its own line, or use a loop")
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
        // `[int]`, `{string: [int]}` and `fn(int) -> int` recurse too.
        if self.depth >= MAX_NESTING {
            return Err(self.too_deep());
        }
        self.depth += 1;
        let parsed = self.type_ann_at();
        self.depth -= 1;
        parsed
    }

    fn type_ann_at(&mut self) -> Result<TypeAnn, Diagnostic> {
        let start = self.span();
        let kind = match self.peek().clone() {
            TokenKind::Ident(name)
                if (name == "Task" || name == "Channel")
                    && matches!(self.peek_at(1), Some(TokenKind::Lt)) =>
            {
                self.bump();
                self.bump(); // <
                let inner = self.type_ann()?;
                self.expect(TokenKind::Gt, "`>`")?;
                if name == "Task" {
                    TypeAnnKind::Task(Box::new(inner))
                } else {
                    TypeAnnKind::Channel(Box::new(inner))
                }
            }
            TokenKind::Fn => {
                self.bump();
                self.expect(TokenKind::LParen, "`(`")?;
                let mut params = Vec::new();
                if !matches!(self.peek(), TokenKind::RParen) {
                    loop {
                        params.push(self.type_ann()?);
                        if matches!(self.peek(), TokenKind::Comma) {
                            self.bump();
                            self.skip_newlines();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "`)`")?;
                self.expect(TokenKind::Arrow, "`->`")?;
                let ret = self.type_ann()?;
                TypeAnnKind::Fn(params, Box::new(ret))
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
                    return Err(
                        Diagnostic::error("E0115", "map keys must be strings", key.span)
                            .with_help("write it like `{string: int}`"),
                    );
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

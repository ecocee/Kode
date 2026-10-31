use kode_diagnostics::Diagnostic;

use super::expr::BP_ARITH;
use super::Parser;
use crate::ast::*;
use crate::token::TokenKind;

impl Parser {
    /// `{ ... }` with one statement per line.
    pub(crate) fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        let open = self.expect(TokenKind::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::RBrace => {
                    self.bump();
                    return Ok(stmts);
                }
                TokenKind::Eof => {
                    return Err(Diagnostic::error(
                        "E0106",
                        "this block is never closed",
                        open.span,
                    )
                    .with_help("add a matching `}`"));
                }
                _ => {}
            }
            let result = match self.parse_stmt() {
                Ok(s) => self.expect_end().map(|_| s),
                Err(e) => Err(e),
            };
            match result {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    self.errors.push(e);
                    self.recover();
                }
            }
        }
    }

    pub(crate) fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek().clone() {
            TokenKind::If => self.parse_if(),
            TokenKind::While => {
                self.bump();
                let cond = self.parse_expr(0)?;
                let body = self.parse_block()?;
                Ok(Stmt::While { cond, body })
            }
            TokenKind::For => self.parse_for(),
            TokenKind::Break => Ok(Stmt::Break(self.bump().span)),
            TokenKind::Continue => Ok(Stmt::Continue(self.bump().span)),
            TokenKind::Return => {
                let span = self.bump().span;
                let value = if matches!(
                    self.peek(),
                    TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof
                ) {
                    None
                } else {
                    Some(self.parse_expr(0)?)
                };
                Ok(Stmt::Return { value, span })
            }
            TokenKind::Fn => Err(Diagnostic::error(
                "E0105",
                "functions can only be defined at the top level of a file",
                self.span(),
            )
            .with_help("move this `fn` out of the block")),
            TokenKind::Ident(_) => match self.peek_at(1) {
                Some(
                    TokenKind::Assign
                    | TokenKind::Colon
                    | TokenKind::PlusEq
                    | TokenKind::MinusEq
                    | TokenKind::StarEq
                    | TokenKind::SlashEq
                    | TokenKind::PercentEq,
                ) => self.parse_assign(),
                _ => Ok(Stmt::Expr(self.parse_expr(0)?)),
            },
            _ => Ok(Stmt::Expr(self.parse_expr(0)?)),
        }
    }

    fn parse_assign(&mut self) -> Result<Stmt, Diagnostic> {
        let name_tok = self.bump();
        let TokenKind::Ident(name) = name_tok.kind else {
            unreachable!("parse_assign is only called on an identifier");
        };
        let name_span = name_tok.span;

        let mut ty = None;
        if matches!(self.peek(), TokenKind::Colon) {
            self.bump();
            ty = Some(self.type_ann()?);
            if !matches!(self.peek(), TokenKind::Assign) {
                return Err(Diagnostic::error(
                    "E0107",
                    format!("expected `=` after the type, found {}", self.peek().describe()),
                    self.span(),
                )
                .with_help("write it like `count: int = 0`"));
            }
        }

        let op_tok = self.bump();
        let compound = match op_tok.kind {
            TokenKind::Assign => None,
            TokenKind::PlusEq => Some(BinOp::Add),
            TokenKind::MinusEq => Some(BinOp::Sub),
            TokenKind::StarEq => Some(BinOp::Mul),
            TokenKind::SlashEq => Some(BinOp::Div),
            TokenKind::PercentEq => Some(BinOp::Rem),
            _ => unreachable!("checked by parse_stmt"),
        };
        self.skip_newlines();
        let rhs = self.parse_expr(0)?;

        let value = match compound {
            None => rhs,
            Some(op) => {
                // `x += e` means `x = x + e`
                let lhs = self.mk(name_span, ExprKind::Ident(name.clone()));
                let span = name_span.join(rhs.span);
                self.mk(
                    span,
                    ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                )
            }
        };
        Ok(Stmt::Assign {
            name,
            name_span,
            ty,
            value,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, Diagnostic> {
        self.bump(); // if
        let cond = self.parse_expr(0)?;
        let then = self.parse_block()?;
        let else_ = if matches!(self.peek(), TokenKind::Else) {
            self.bump();
            if matches!(self.peek(), TokenKind::If) {
                Some(vec![self.parse_if()?])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If { cond, then, else_ })
    }

    fn parse_for(&mut self) -> Result<Stmt, Diagnostic> {
        self.bump(); // for
        let (var, var_span) = match self.peek().clone() {
            TokenKind::Ident(n) => {
                let s = self.bump().span;
                (n, s)
            }
            other => {
                return Err(Diagnostic::error(
                    "E0108",
                    format!("expected a loop variable name, found {}", other.describe()),
                    self.span(),
                )
                .with_help("write it like `for i in 0..10 { ... }`"))
            }
        };
        self.expect(TokenKind::In, "`in`")?;
        let start = self.parse_expr(BP_ARITH)?;
        let inclusive = match self.peek() {
            TokenKind::DotDot => false,
            TokenKind::DotDotEq => true,
            other => {
                return Err(Diagnostic::error(
                    "E0109",
                    format!("expected `..` or `..=`, found {}", other.describe()),
                    self.span(),
                )
                .with_help("only ranges such as `0..10` can be looped over so far"))
            }
        };
        self.bump();
        let end = self.parse_expr(BP_ARITH)?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            var,
            var_span,
            start,
            end,
            inclusive,
            body,
        })
    }
}

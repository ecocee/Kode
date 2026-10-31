use kode_diagnostics::Diagnostic;

use super::Parser;
use crate::ast::*;
use crate::lexer::lex_at;
use crate::token::{StrPart, TokenKind};

/// Binding powers: higher binds tighter.
/// or 1 < and 3 < comparison 7 < + - 10 < * / % 20 < unary minus 30
const BP_NOT_OPERAND: u8 = 6;
const BP_UNARY: u8 = 30;
/// Used by `for`, so `0..n` does not swallow the `..`.
pub(crate) const BP_ARITH: u8 = 10;

fn infix(kind: &TokenKind) -> Option<(BinOp, u8, u8)> {
    Some(match kind {
        TokenKind::Or => (BinOp::Or, 1, 2),
        TokenKind::And => (BinOp::And, 3, 4),
        TokenKind::EqEq => (BinOp::Eq, 7, 8),
        TokenKind::NotEq => (BinOp::Ne, 7, 8),
        TokenKind::Lt => (BinOp::Lt, 7, 8),
        TokenKind::Le => (BinOp::Le, 7, 8),
        TokenKind::Gt => (BinOp::Gt, 7, 8),
        TokenKind::Ge => (BinOp::Ge, 7, 8),
        TokenKind::Plus => (BinOp::Add, 10, 11),
        TokenKind::Minus => (BinOp::Sub, 10, 11),
        TokenKind::Star => (BinOp::Mul, 20, 21),
        TokenKind::Slash => (BinOp::Div, 20, 21),
        TokenKind::Percent => (BinOp::Rem, 20, 21),
        _ => return None,
    })
}

impl Parser {
    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;
        loop {
            let Some((op, l_bp, r_bp)) = infix(self.peek()) else {
                break;
            };
            if l_bp < min_bp {
                break;
            }
            self.bump();
            self.skip_newlines(); // a line may end with an operator
            let rhs = self.parse_expr(r_bp)?;
            let span = lhs.span.join(rhs.span);
            lhs = self.mk(
                span,
                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            );
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        let (op, bp) = match self.peek() {
            TokenKind::Minus => (UnOp::Neg, BP_UNARY),
            TokenKind::Not => (UnOp::Not, BP_NOT_OPERAND),
            _ => return self.parse_primary(),
        };
        let start = self.bump().span;
        let operand = self.parse_expr(bp)?;
        let span = start.join(operand.span);
        Ok(self.mk(
            span,
            ExprKind::Unary {
                op,
                expr: Box::new(operand),
            },
        ))
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Int(v) => Ok(self.mk(tok.span, ExprKind::Int(v))),
            TokenKind::Float(v) => Ok(self.mk(tok.span, ExprKind::Float(v))),
            TokenKind::True => Ok(self.mk(tok.span, ExprKind::Bool(true))),
            TokenKind::False => Ok(self.mk(tok.span, ExprKind::Bool(false))),
            TokenKind::Str(parts) => self.parse_string(parts, tok.span),
            TokenKind::Ident(name) => {
                if matches!(self.peek(), TokenKind::LParen) {
                    self.parse_call(name, tok.span)
                } else {
                    Ok(self.mk(tok.span, ExprKind::Ident(name)))
                }
            }
            TokenKind::LParen => {
                self.skip_newlines();
                let inner = self.parse_expr(0)?;
                self.skip_newlines();
                self.expect(TokenKind::RParen, "`)`")?;
                Ok(inner)
            }
            other => Err(Diagnostic::error(
                "E0102",
                format!("expected a value, found {}", other.describe()),
                tok.span,
            )),
        }
    }

    fn parse_call(&mut self, callee: String, callee_span: kode_diagnostics::Span) -> Result<Expr, Diagnostic> {
        self.bump(); // (
        self.skip_newlines();
        let mut args = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            args.push(self.parse_expr(0)?);
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Comma) {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        if !matches!(self.peek(), TokenKind::RParen) {
            return Err(Diagnostic::error(
                "E0103",
                format!("expected `)` or `,`, found {}", self.peek().describe()),
                self.span(),
            ));
        }
        let end = self.bump().span;
        Ok(self.mk(
            callee_span.join(end),
            ExprKind::Call {
                callee,
                callee_span,
                args,
            },
        ))
    }

    fn parse_string(&mut self, parts: Vec<StrPart>, span: kode_diagnostics::Span) -> Result<Expr, Diagnostic> {
        if parts.len() == 1 {
            if let StrPart::Lit(s) = &parts[0] {
                return Ok(self.mk(span, ExprKind::Str(s.clone())));
            }
        }
        let mut out = Vec::new();
        for part in parts {
            match part {
                StrPart::Lit(s) => out.push(InterpPart::Lit(s)),
                StrPart::Code { src, offset } => {
                    let tokens = lex_at(&src, offset).map_err(|mut errs| {
                        let first = errs.remove(0);
                        self.errors.extend(errs);
                        first
                    })?;
                    let mut sub = Parser::new(tokens, self.next_id);
                    let e = sub.parse_expr(0)?;
                    if !matches!(sub.peek(), TokenKind::Eof) {
                        return Err(Diagnostic::error(
                            "E0101",
                            format!("unexpected {} inside `${{...}}`", sub.peek().describe()),
                            sub.span(),
                        ));
                    }
                    self.next_id = sub.next_id;
                    self.errors.extend(sub.errors);
                    out.push(InterpPart::Expr(e));
                }
            }
        }
        Ok(self.mk(span, ExprKind::Interp(out)))
    }
}

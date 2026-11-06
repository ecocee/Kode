use kode_diagnostics::{Diagnostic, Span};

use super::Parser;
use crate::ast::*;
use crate::lexer::lex_at;
use crate::token::{StrPart, TokenKind};

/// Binding powers: higher binds tighter.
/// `??` 1 < or 2 < and 4 < `|` 6 < `^` 7 < `&` 8 < comparison/in 10 <
/// `<< >>` 12 < `+ -` 16 < `* / %` 20 < unary minus 30.
const BP_NOT_OPERAND: u8 = 9;
const BP_UNARY: u8 = 30;
/// Used by `for` and `match`, so `0..n` and `=>` are not swallowed.
pub(crate) const BP_ARITH: u8 = 16;

fn infix(kind: &TokenKind) -> Option<(BinOp, u8, u8)> {
    Some(match kind {
        TokenKind::Or => (BinOp::Or, 2, 3),
        TokenKind::And => (BinOp::And, 4, 5),
        TokenKind::Pipe => (BinOp::BitOr, 6, 7),
        TokenKind::Caret => (BinOp::BitXor, 7, 8),
        TokenKind::Amp => (BinOp::BitAnd, 8, 9),
        TokenKind::EqEq => (BinOp::Eq, 10, 11),
        TokenKind::NotEq => (BinOp::Ne, 10, 11),
        TokenKind::Lt => (BinOp::Lt, 10, 11),
        TokenKind::Le => (BinOp::Le, 10, 11),
        TokenKind::Gt => (BinOp::Gt, 10, 11),
        TokenKind::Ge => (BinOp::Ge, 10, 11),
        TokenKind::In => (BinOp::In, 10, 11),
        TokenKind::Shl => (BinOp::Shl, 12, 13),
        TokenKind::Shr => (BinOp::Shr, 12, 13),
        TokenKind::Plus => (BinOp::Add, 16, 17),
        TokenKind::Minus => (BinOp::Sub, 16, 17),
        TokenKind::Star => (BinOp::Mul, 20, 21),
        TokenKind::Slash => (BinOp::Div, 20, 21),
        TokenKind::Percent => (BinOp::Rem, 20, 21),
        _ => return None,
    })
}

impl Parser {
    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        // Every recursive descent into an expression comes through here, so the
        // depth guard lives here and not in the operators.
        if self.depth >= super::MAX_NESTING {
            return Err(self.too_deep());
        }
        self.depth += 1;
        let parsed = self.parse_expr_at(min_bp);
        self.depth -= 1;
        parsed
    }

    fn parse_expr_at(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;
        loop {
            if matches!(self.peek(), TokenKind::QuestionQuestion) && min_bp <= 1 {
                self.bump();
                self.skip_newlines();
                let rhs = self.parse_expr(2)?;
                let span = lhs.span.join(rhs.span);
                lhs = self.mk(
                    span,
                    ExprKind::Coalesce {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                );
                continue;
            }
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
        // `expr catch err { ... }` binds loosest of all.
        if min_bp == 0 && matches!(self.peek(), TokenKind::Catch) {
            self.bump();
            let (var, var_span) = self.ident("a name for the error after `catch`")?;
            let block = self.parse_block()?;
            let span = lhs.span.join(var_span);
            lhs = self.mk(
                span,
                ExprKind::Catch {
                    expr: Box::new(lhs),
                    var,
                    var_span,
                    block,
                },
            );
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        match self.peek() {
            TokenKind::Await => {
                let start = self.bump().span;
                let operand = self.parse_expr(BP_UNARY)?;
                let span = start.join(operand.span);
                return Ok(self.mk(span, ExprKind::Await(Box::new(operand))));
            }
            TokenKind::Spawn => {
                let start = self.bump().span;
                let call = self.parse_postfix()?;
                return match call.kind {
                    ExprKind::Call { callee, callee_span, args } => {
                        let span = start.join(call.span);
                        Ok(self.mk(span, ExprKind::Spawn { callee, callee_span, args }))
                    }
                    _ => Err(Diagnostic::error("E0117", "`spawn` needs a function call", call.span)
                        .with_help("write it like `spawn work(1, 2)`")),
                };
            }
            _ => {}
        }
        // `|x: int| x * 2`: a closure. `||` (two pipes) takes no parameters.
        if matches!(self.peek(), TokenKind::Pipe) {
            let start = self.bump().span;
            let mut params = Vec::new();
            if !matches!(self.peek(), TokenKind::Pipe) {
                loop {
                    let (name, name_span) = self.ident("a parameter name")?;
                    if !matches!(self.peek(), TokenKind::Colon) {
                        return Err(Diagnostic::error(
                            "E0241",
                            format!("the parameter `{name}` of a closure needs a type"),
                            name_span,
                        )
                        .with_help("write it like `|x: int| x * 2`"));
                    }
                    self.bump();
                    let ty = self.type_ann()?;
                    params.push(ClosureParam {
                        name,
                        name_span,
                        ty: Some(ty),
                    });
                    if matches!(self.peek(), TokenKind::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::Pipe, "`|`")?;
            let body = self.parse_expr(0)?;
            let span = start.join(body.span);
            return Ok(self.mk(
                span,
                ExprKind::Closure {
                    params,
                    body: Box::new(body),
                },
            ));
        }
        let (op, bp) = match self.peek() {
            TokenKind::Minus => (UnOp::Neg, BP_UNARY),
            TokenKind::Not => (UnOp::Not, BP_NOT_OPERAND),
            TokenKind::Tilde => (UnOp::BitNot, BP_UNARY),
            _ => return self.parse_postfix(),
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

    /// A primary expression followed by `.name`, `.name(...)`, `?.name`, `[i]` or `!`.
    fn parse_postfix(&mut self) -> Result<Expr, Diagnostic> {
        let mut e = self.parse_primary()?;
        loop {
            match self.peek() {
                TokenKind::Dot => {
                    self.bump();
                    let (name, name_span) = self.ident("a field or method name after `.`")?;
                    if matches!(self.peek(), TokenKind::LParen) {
                        let (args, end) = self.parse_args()?;
                        let span = e.span.join(end);
                        e = self.mk(
                            span,
                            ExprKind::MethodCall {
                                obj: Box::new(e),
                                name,
                                name_span,
                                args,
                            },
                        );
                    } else {
                        let span = e.span.join(name_span);
                        e = self.mk(
                            span,
                            ExprKind::Member {
                                obj: Box::new(e),
                                name,
                                name_span,
                                safe: false,
                            },
                        );
                    }
                }
                TokenKind::QuestionDot => {
                    self.bump();
                    let (name, name_span) = self.ident("a field name after `?.`")?;
                    let span = e.span.join(name_span);
                    e = self.mk(
                        span,
                        ExprKind::Member {
                            obj: Box::new(e),
                            name,
                            name_span,
                            safe: true,
                        },
                    );
                }
                TokenKind::LBracket => {
                    self.bump();
                    self.skip_newlines();
                    // `xs[..]`, `xs[..3]`, `xs[1..3]`, `xs[1..=3]`: a slice.
                    let start = if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
                        None
                    } else {
                        let first = self.parse_expr(0)?;
                        self.skip_newlines();
                        Some(Box::new(first))
                    };
                    let inclusive = matches!(self.peek(), TokenKind::DotDotEq);
                    if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
                        self.bump();
                        self.skip_newlines();
                        let end = if matches!(self.peek(), TokenKind::RBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expr(0)?))
                        };
                        self.skip_newlines();
                        let close = self.expect(TokenKind::RBracket, "`]`")?.span;
                        let span = e.span.join(close);
                        e = self.mk(
                            span,
                            ExprKind::Slice {
                                obj: Box::new(e),
                                start,
                                end,
                                inclusive,
                            },
                        );
                    } else {
                        let Some(index) = start else { unreachable!("`[` without a value is a slice") };
                        let close = self.expect(TokenKind::RBracket, "`]`")?.span;
                        let span = e.span.join(close);
                        e = self.mk(
                            span,
                            ExprKind::Index {
                                obj: Box::new(e),
                                index,
                            },
                        );
                    }
                }
                TokenKind::Bang => {
                    let end = self.bump().span;
                    let span = e.span.join(end);
                    e = self.mk(span, ExprKind::Unwrap(Box::new(e)));
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Int(v) => Ok(self.mk(tok.span, ExprKind::Int(v))),
            TokenKind::Float(v) => Ok(self.mk(tok.span, ExprKind::Float(v))),
            TokenKind::True => Ok(self.mk(tok.span, ExprKind::Bool(true))),
            TokenKind::False => Ok(self.mk(tok.span, ExprKind::Bool(false))),
            TokenKind::None => Ok(self.mk(tok.span, ExprKind::None)),
            TokenKind::Str(parts) => self.parse_string(parts, tok.span),
            TokenKind::Ident(name) => {
                if matches!(self.peek(), TokenKind::LParen) {
                    let (args, end) = self.parse_args()?;
                    Ok(self.mk(
                        tok.span.join(end),
                        ExprKind::Call {
                            callee: name,
                            callee_span: tok.span,
                            args,
                        },
                    ))
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
            TokenKind::LBracket => {
                self.skip_newlines();
                let mut items = Vec::new();
                while !matches!(self.peek(), TokenKind::RBracket) {
                    items.push(self.parse_expr(0)?);
                    self.skip_newlines();
                    if matches!(self.peek(), TokenKind::Comma) {
                        self.bump();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBracket, "`]` or `,`")?.span;
                Ok(self.mk(tok.span.join(end), ExprKind::List(items)))
            }
            TokenKind::LBrace => {
                self.skip_newlines();
                let mut pairs = Vec::new();
                while !matches!(self.peek(), TokenKind::RBrace) {
                    let k = self.parse_expr(0)?;
                    self.expect(TokenKind::Colon, "`:` between a key and its value")?;
                    self.skip_newlines();
                    let v = self.parse_expr(0)?;
                    pairs.push((k, v));
                    self.skip_newlines();
                    if matches!(self.peek(), TokenKind::Comma) {
                        self.bump();
                        self.skip_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBrace, "`}` or `,`")?.span;
                Ok(self.mk(tok.span.join(end), ExprKind::Map(pairs)))
            }
            other => Err(Diagnostic::error(
                "E0102",
                format!("expected a value, found {}", other.describe()),
                tok.span,
            )),
        }
    }

    /// `( arg, name: arg, ... )`. Returns the arguments and the span of `)`.
    fn parse_args(&mut self) -> Result<(Vec<Arg>, Span), Diagnostic> {
        self.bump(); // (
        self.skip_newlines();
        let mut args = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            let name = match (self.peek().clone(), self.peek_at(1)) {
                (TokenKind::Ident(n), Some(TokenKind::Colon)) => {
                    let s = self.bump().span;
                    self.bump(); // :
                    self.skip_newlines();
                    Some((n, s))
                }
                _ => None,
            };
            let value = self.parse_expr(0)?;
            args.push(Arg { name, value });
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
        Ok((args, end))
    }

    fn parse_string(&mut self, parts: Vec<StrPart>, span: Span) -> Result<Expr, Diagnostic> {
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
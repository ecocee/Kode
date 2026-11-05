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
                    return Err(
                        Diagnostic::error("E0106", "this block is never closed", open.span)
                            .with_help("add a matching `}`"),
                    );
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
            TokenKind::Match => self.parse_match(),
            TokenKind::Break => Ok(Stmt::Break(self.bump().span)),
            TokenKind::Continue => Ok(Stmt::Continue(self.bump().span)),
            TokenKind::Throw => {
                let span = self.bump().span;
                let value = self.parse_expr(0)?;
                Ok(Stmt::Throw { value, span })
            }
            TokenKind::Try => {
                self.bump();
                let body = self.parse_block()?;
                let has_catch = matches!(self.peek(), TokenKind::Catch);
                let has_finally = matches!(self.peek(), TokenKind::Finally);
                if !has_catch && !has_finally {
                    self.expect(TokenKind::Catch, "`catch` or `finally` after the `try` block")?;
                }
                let (var, handler) = if has_catch {
                    self.bump();
                    let (var, var_span) = self.ident("a name for the error after `catch`")?;
                    (Some((var, var_span)), Some(self.parse_block()?))
                } else {
                    (None, None)
                };
                let finally = if matches!(self.peek(), TokenKind::Finally) {
                    self.bump();
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Stmt::Try {
                    body,
                    var,
                    handler,
                    finally,
                })
            }
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
            TokenKind::Ident(_) if matches!(self.peek_at(1), Some(TokenKind::Colon)) => {
                self.parse_typed_decl()
            }
            _ => {
                let e = self.parse_expr(0)?;
                if matches!(
                    self.peek(),
                    TokenKind::Assign
                        | TokenKind::PlusEq
                        | TokenKind::MinusEq
                        | TokenKind::StarEq
                        | TokenKind::SlashEq
                        | TokenKind::PercentEq
                        | TokenKind::AmpEq
                        | TokenKind::PipeEq
                        | TokenKind::CaretEq
                        | TokenKind::ShlEq
                        | TokenKind::ShrEq
                ) {
                    self.parse_assign(e)
                } else {
                    Ok(Stmt::Expr(e))
                }
            }
        }
    }

    /// `name: type = value`
    fn parse_typed_decl(&mut self) -> Result<Stmt, Diagnostic> {
        let (name, name_span) = self.ident("a variable name")?;
        self.bump(); // :
        let ty = self.type_ann()?;
        if !matches!(self.peek(), TokenKind::Assign) {
            return Err(Diagnostic::error(
                "E0107",
                format!("expected `=` after the type, found {}", self.peek().describe()),
                self.span(),
            )
            .with_help("write it like `count: int = 0`"));
        }
        self.bump();
        self.skip_newlines();
        let value = self.parse_expr(0)?;
        let span = name_span.join(value.span);
        Ok(Stmt::Assign {
            target: Target::Var(name, name_span),
            ty: Some(ty),
            value,
            span,
        })
    }

    fn parse_assign(&mut self, lhs: Expr) -> Result<Stmt, Diagnostic> {
        let target = match &lhs.kind {
            ExprKind::Ident(n) => Target::Var(n.clone(), lhs.span),
            ExprKind::Index { obj, index } => Target::Index {
                obj: (**obj).clone(),
                index: (**index).clone(),
            },
            ExprKind::Member {
                obj,
                name,
                name_span,
                safe: false,
            } => Target::Field {
                obj: (**obj).clone(),
                name: name.clone(),
                name_span: *name_span,
            },
            _ => {
                return Err(Diagnostic::error(
                    "E0112",
                    "this cannot be assigned to",
                    lhs.span,
                )
                .with_help("assign to a variable, a list item `xs[0]`, a map item `m[\"k\"]` or a field `obj.name`"))
            }
        };
        let op_tok = self.bump();
        let compound = match op_tok.kind {
            TokenKind::Assign => None,
            TokenKind::PlusEq => Some(BinOp::Add),
            TokenKind::MinusEq => Some(BinOp::Sub),
            TokenKind::StarEq => Some(BinOp::Mul),
            TokenKind::SlashEq => Some(BinOp::Div),
            TokenKind::PercentEq => Some(BinOp::Rem),
            TokenKind::AmpEq => Some(BinOp::BitAnd),
            TokenKind::PipeEq => Some(BinOp::BitOr),
            TokenKind::CaretEq => Some(BinOp::BitXor),
            TokenKind::ShlEq => Some(BinOp::Shl),
            TokenKind::ShrEq => Some(BinOp::Shr),
            _ => unreachable!("checked by parse_stmt"),
        };
        self.skip_newlines();
        let rhs = self.parse_expr(0)?;
        let span = lhs.span.join(rhs.span);
        let value = match compound {
            None => rhs,
            // `x += e` means `x = x + e`
            Some(op) => self.mk(
                span,
                ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            ),
        };
        Ok(Stmt::Assign {
            target,
            ty: None,
            value,
            span,
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
        let mut vars = vec![self.ident("a loop variable name")?];
        if matches!(self.peek(), TokenKind::Comma) {
            self.bump();
            vars.push(self.ident("a second loop variable name")?);
        }
        self.expect(TokenKind::In, "`in`")?;
        let first = self.parse_expr(BP_ARITH)?;
        let inclusive = match self.peek() {
            TokenKind::DotDot => Some(false),
            TokenKind::DotDotEq => Some(true),
            _ => None,
        };
        match inclusive {
            Some(inclusive) => {
                if vars.len() != 1 {
                    return Err(Diagnostic::error(
                        "E0108",
                        "a range loop has exactly one variable",
                        vars[1].1,
                    ));
                }
                self.bump();
                let end = self.parse_expr(BP_ARITH)?;
                let body = self.parse_block()?;
                let (var, var_span) = vars.remove(0);
                Ok(Stmt::For {
                    var,
                    var_span,
                    start: first,
                    end,
                    inclusive,
                    body,
                })
            }
            None => {
                let body = self.parse_block()?;
                Ok(Stmt::ForIn {
                    vars,
                    iter: first,
                    body,
                })
            }
        }
    }

    fn parse_match(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.bump().span; // match
        let scrutinee = self.parse_expr(0)?;
        self.expect(TokenKind::LBrace, "`{`")?;
        let mut arms = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::RBrace) {
                self.bump();
                break;
            }
            if matches!(self.peek(), TokenKind::Eof) {
                return Err(Diagnostic::error("E0106", "this `match` is never closed", span));
            }
            let pattern_span = self.span();
            let pattern = match (self.peek().clone(), self.peek_at(1)) {
                (TokenKind::Ident(n), Some(TokenKind::FatArrow)) if n == "_" => {
                    self.bump();
                    Pattern::Wildcard
                }
                _ => Pattern::Value(self.parse_expr(BP_ARITH)?),
            };
            self.expect(TokenKind::FatArrow, "`=>` after the pattern")?;
            let body = if matches!(self.peek(), TokenKind::LBrace) {
                self.parse_block()?
            } else {
                vec![self.parse_stmt()?]
            };
            arms.push(Arm {
                pattern,
                pattern_span,
                body,
            });
            self.expect_end()?;
        }
        Ok(Stmt::Match {
            scrutinee,
            arms,
            span,
        })
    }
}
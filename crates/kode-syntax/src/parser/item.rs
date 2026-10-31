use kode_diagnostics::Diagnostic;

use super::Parser;
use crate::ast::*;
use crate::token::TokenKind;

impl Parser {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Eof) {
                break;
            }
            let result = match self.peek() {
                TokenKind::Fn => self.parse_fn().map(Item::Fn),
                TokenKind::Const => self.parse_const().map(Item::Const),
                _ => self.parse_stmt().map(Item::Stmt),
            };
            let result = match result {
                Ok(item) => self.expect_end().map(|_| item),
                Err(e) => Err(e),
            };
            match result {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.errors.push(e);
                    self.recover();
                }
            }
        }
        items
    }

    fn parse_fn(&mut self) -> Result<FnDecl, Diagnostic> {
        let fn_tok = self.bump();
        let (name, name_span) = match self.peek().clone() {
            TokenKind::Ident(n) => {
                let s = self.bump().span;
                (n, s)
            }
            other => {
                return Err(Diagnostic::error(
                    "E0110",
                    format!("expected a function name, found {}", other.describe()),
                    self.span(),
                ))
            }
        };

        self.expect(TokenKind::LParen, "`(`")?;
        self.skip_newlines();
        let mut params = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            let (pname, pspan) = match self.peek().clone() {
                TokenKind::Ident(n) => {
                    let s = self.bump().span;
                    (n, s)
                }
                other => {
                    return Err(Diagnostic::error(
                        "E0110",
                        format!("expected a parameter name, found {}", other.describe()),
                        self.span(),
                    ))
                }
            };
            if !matches!(self.peek(), TokenKind::Colon) {
                return Err(Diagnostic::error(
                    "E0111",
                    format!("parameter `{pname}` needs a type"),
                    pspan,
                )
                .with_help(format!("write it like `{pname}: int`")));
            }
            self.bump();
            let ty = self.type_ann()?;
            params.push(Param {
                name: pname,
                name_span: pspan,
                ty,
            });
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Comma) {
                self.bump();
                self.skip_newlines();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen, "`)`")?;

        let ret = if matches!(self.peek(), TokenKind::Arrow) {
            self.bump();
            Some(self.type_ann()?)
        } else {
            None
        };

        let body = if matches!(self.peek(), TokenKind::FatArrow) {
            self.bump();
            self.skip_newlines();
            FnBody::Expr(self.parse_expr(0)?)
        } else {
            FnBody::Block(self.parse_block()?)
        };

        Ok(FnDecl {
            span: fn_tok.span.join(name_span),
            name,
            name_span,
            params,
            ret,
            body,
        })
    }

    fn parse_const(&mut self) -> Result<ConstDecl, Diagnostic> {
        self.bump(); // const
        let (name, name_span) = match self.peek().clone() {
            TokenKind::Ident(n) => {
                let s = self.bump().span;
                (n, s)
            }
            other => {
                return Err(Diagnostic::error(
                    "E0110",
                    format!("expected a constant name, found {}", other.describe()),
                    self.span(),
                ))
            }
        };
        let ty = if matches!(self.peek(), TokenKind::Colon) {
            self.bump();
            Some(self.type_ann()?)
        } else {
            None
        };
        self.expect(TokenKind::Assign, "`=`")?;
        let value = self.parse_expr(0)?;
        Ok(ConstDecl {
            name,
            name_span,
            ty,
            value,
        })
    }
}

use kode_diagnostics::Diagnostic;

use super::Parser;
use crate::ast::*;
use crate::token::{StrPart, TokenKind};

impl Parser {
    pub(crate) fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Eof) {
                break;
            }
            let is_pub = matches!(self.peek(), TokenKind::Pub);
            if is_pub {
                self.bump();
            }
            let result = match self.peek() {
                TokenKind::Fn => self.parse_fn().map(Item::Fn),
                TokenKind::Const => self.parse_const().map(Item::Const),
                TokenKind::Class => self.parse_class().map(Item::Class),
                TokenKind::Interface => self.parse_interface().map(Item::Interface),
                TokenKind::Enum => self.parse_enum().map(Item::Enum),
                TokenKind::Import | TokenKind::From => self.parse_import().map(Item::Import),
                TokenKind::Test => self.parse_test().map(Item::Test),
                _ => self.parse_stmt().map(Item::Stmt),
            };
            let result = match result {
                Ok(item) => self.expect_end().map(|_| item),
                Err(e) => Err(e),
            };
            match result {
                Ok(mut item) => {
                    set_pub(&mut item, is_pub);
                    items.push(item);
                }
                Err(e) => {
                    self.errors.push(e);
                    if self.fatal {
                        break;
                    }
                    self.recover();
                }
            }
        }
        items
    }

    pub(crate) fn parse_fn(&mut self) -> Result<FnDecl, Diagnostic> {
        let fn_tok = self.bump();
        let (name, name_span) = self.ident("a function name")?;
        // `fn first<T>(...)`: type parameters, separated by commas.
        let mut type_params = Vec::new();
        if matches!(self.peek(), TokenKind::Lt) {
            self.bump();
            loop {
                let (pname, pspan) = self.ident("a type parameter name")?;
                type_params.push((pname, pspan));
                if matches!(self.peek(), TokenKind::Comma) {
                    self.bump();
                    self.skip_newlines();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::Gt, "`>`")?;
        }

        self.expect(TokenKind::LParen, "`(`")?;
        self.skip_newlines();
        let mut params = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            let (pname, pspan) = self.ident("a parameter name")?;
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
            is_pub: false,
            type_params,
            params,
            ret,
            body,
        })
    }

    fn parse_const(&mut self) -> Result<ConstDecl, Diagnostic> {
        self.bump(); // const
        let (name, name_span) = self.ident("a constant name")?;
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
            is_pub: false,
            ty,
            value,
        })
    }

    fn parse_class(&mut self) -> Result<ClassDecl, Diagnostic> {
        self.bump(); // class
        let (name, name_span) = self.ident("a class name")?;
        // `class Box<T>`: the type parameters (ADR 0040).
        let mut type_params = Vec::new();
        if matches!(self.peek(), TokenKind::Lt) {
            self.bump();
            loop {
                type_params.push(self.ident("a type parameter")?);
                if matches!(self.peek(), TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
            self.expect(TokenKind::Gt, "`>`")?;
        }
        let mut implements = Vec::new();
        if matches!(self.peek(), TokenKind::Implements) {
            self.bump();
            loop {
                implements.push(self.ident("an interface name")?);
                if matches!(self.peek(), TokenKind::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        let open = self.expect(TokenKind::LBrace, "`{`")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Pub) {
                self.bump();
            }
            match self.peek().clone() {
                TokenKind::RBrace => {
                    self.bump();
                    break;
                }
                TokenKind::Eof => {
                    return Err(
                        Diagnostic::error("E0106", "this class is never closed", open.span)
                            .with_help("add a matching `}`"),
                    )
                }
                TokenKind::Fn => methods.push(self.parse_fn()?),
                TokenKind::Ident(_) if matches!(self.peek_at(1), Some(TokenKind::Colon)) => {
                    let (fname, fspan) = self.ident("a field name")?;
                    self.bump(); // :
                    let ty = self.type_ann()?;
                    let default = if matches!(self.peek(), TokenKind::Assign) {
                        self.bump();
                        Some(self.parse_expr(0)?)
                    } else {
                        None
                    };
                    fields.push(FieldDecl {
                        name: fname,
                        name_span: fspan,
                        ty,
                        default,
                    });
                }
                other => {
                    return Err(Diagnostic::error(
                        "E0114",
                        format!("expected a field or a method, found {}", other.describe()),
                        self.span(),
                    )
                    .with_help(
                        "fields look like `name: string`, methods like `fn show() { ... }`",
                    ))
                }
            }
            self.expect_end()?;
        }
        Ok(ClassDecl {
            name,
            name_span,
            is_pub: false,
            type_params,
            implements,
            fields,
            methods,
        })
    }

    fn parse_interface(&mut self) -> Result<InterfaceDecl, Diagnostic> {
        self.bump(); // interface
        let (name, name_span) = self.ident("an interface name")?;
        let open = self.expect(TokenKind::LBrace, "`{`")?;
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Pub) {
                self.bump();
            }
            match self.peek().clone() {
                TokenKind::RBrace => {
                    self.bump();
                    break;
                }
                TokenKind::Eof => {
                    return Err(Diagnostic::error(
                        "E0106",
                        "this interface is never closed",
                        open.span,
                    )
                    .with_help("add a matching `}`"))
                }
                TokenKind::Fn => methods.push(self.parse_iface_method()?),
                other => {
                    return Err(Diagnostic::error(
                        "E0114",
                        format!("expected a method signature, found {}", other.describe()),
                        self.span(),
                    )
                    .with_help("method signatures look like `fn area() -> float`"))
                }
            }
            self.expect_end()?;
        }
        Ok(InterfaceDecl {
            name,
            name_span,
            is_pub: false,
            methods,
        })
    }

    fn parse_iface_method(&mut self) -> Result<IfaceMethod, Diagnostic> {
        let fn_tok = self.bump(); // fn
        let (name, name_span) = self.ident("a method name")?;
        self.expect(TokenKind::LParen, "`(`")?;
        self.skip_newlines();
        let mut params = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen) {
            let (pname, pspan) = self.ident("a parameter name")?;
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
        if matches!(self.peek(), TokenKind::LBrace | TokenKind::FatArrow) {
            return Err(Diagnostic::error(
                "E0236",
                "an interface method has no body",
                fn_tok.span.join(name_span),
            )
            .with_help("write only the signature; the implementing class writes the body"));
        }
        Ok(IfaceMethod {
            span: fn_tok.span.join(name_span),
            name,
            name_span,
            params,
            ret,
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, Diagnostic> {
        self.bump(); // enum
        let (name, name_span) = self.ident("an enum name")?;
        self.expect(TokenKind::LBrace, "`{`")?;
        let mut variants = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::RBrace) {
                self.bump();
                break;
            }
            let (vname, vspan) = self.ident("a variant name")?;
            // `Circle(radius: float)`: a payload variant (ADR 0039). The
            // checker still refuses construction; the syntax and the AST dump
            // carry it so S2's oracle has the shape to reproduce.
            let mut fields = Vec::new();
            if matches!(self.peek(), TokenKind::LParen) {
                self.bump();
                if !matches!(self.peek(), TokenKind::RParen) {
                    loop {
                        let (fname, fspan) = self.ident("a field name")?;
                        self.expect(TokenKind::Colon, "`:`")?;
                        let ty = self.type_ann()?;
                        fields.push(Param {
                            name: fname,
                            name_span: fspan,
                            ty,
                        });
                        if matches!(self.peek(), TokenKind::Comma) {
                            self.bump();
                            self.skip_newlines();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "`)`")?;
            }
            variants.push(EnumVariant {
                name: vname,
                name_span: vspan,
                fields,
            });
            self.skip_newlines();
            if matches!(self.peek(), TokenKind::Comma) {
                self.bump();
            }
        }
        Ok(EnumDecl {
            name,
            name_span,
            is_pub: false,
            variants,
        })
    }

    /// A plain string token (no `${}`): used for import paths and test names.
    fn plain_string(&mut self, what: &str) -> Result<(String, kode_diagnostics::Span), Diagnostic> {
        match self.peek().clone() {
            TokenKind::Str(parts) if parts.len() == 1 && matches!(parts[0], StrPart::Lit(_)) => {
                let span = self.bump().span;
                let StrPart::Lit(s) = &parts[0] else {
                    unreachable!()
                };
                Ok((s.clone(), span))
            }
            other => Err(Diagnostic::error(
                "E0116",
                format!("expected {what}, found {}", other.describe()),
                self.span(),
            )),
        }
    }

    fn parse_import(&mut self) -> Result<ImportDecl, Diagnostic> {
        let from = matches!(self.peek(), TokenKind::From);
        self.bump();
        let (path, path_span) = self.plain_string("a file path in quotes")?;
        let mut names = Vec::new();
        let mut alias = None;
        if from {
            self.expect(TokenKind::Import, "`import`")?;
            names.push(self.ident("a name to import")?);
            while matches!(self.peek(), TokenKind::Comma) {
                self.bump();
                names.push(self.ident("a name to import")?);
            }
        } else if matches!(self.peek(), TokenKind::As) {
            self.bump();
            alias = Some(self.ident("a module name")?);
        }
        Ok(ImportDecl {
            path,
            path_span,
            names,
            alias,
        })
    }

    fn parse_test(&mut self) -> Result<TestDecl, Diagnostic> {
        self.bump(); // test
        let (name, name_span) = self.plain_string("the test name in quotes")?;
        let body = self.parse_block()?;
        Ok(TestDecl {
            name,
            name_span,
            body,
        })
    }
}

/// `pub` belongs to the item that follows it, not to its members.
fn set_pub(item: &mut Item, value: bool) {
    match item {
        Item::Fn(f) => f.is_pub = value,
        Item::Const(c) => c.is_pub = value,
        Item::Class(c) => c.is_pub = value,
        Item::Interface(i) => i.is_pub = value,
        Item::Enum(e) => e.is_pub = value,
        _ => {}
    }
}

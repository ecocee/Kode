use kode_diagnostics::Span;

use super::expr::Expr;
use super::stmt::{Block, Stmt, TypeAnn};

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
    pub expr_count: u32,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDecl),
    Const(ConstDecl),
    Class(ClassDecl),
    Enum(EnumDecl),
    Import(ImportDecl),
    Test(TestDecl),
    /// Top-level statements run in order, like a script.
    Stmt(Stmt),
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub ret: Option<TypeAnn>,
    pub body: FnBody,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum FnBody {
    Block(Block),
    /// `fn double(x: int) -> int => x * 2`
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub name_span: Span,
    pub ty: TypeAnn,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub name_span: Span,
    pub ty: Option<TypeAnn>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub name_span: Span,
    pub ty: TypeAnn,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub name_span: Span,
    pub variants: Vec<(String, Span)>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
    pub path_span: Span,
    /// `from "x" import a, b` lists names; `import "x"` has none.
    pub names: Vec<(String, Span)>,
}

#[derive(Debug, Clone)]
pub struct TestDecl {
    pub name: String,
    pub name_span: Span,
    pub body: Block,
}

impl Program {
    /// Every function in one fixed order: free functions and class methods,
    /// in source order. The type checker and the MIR lowering both rely on it.
    pub fn fn_decls(&self) -> Vec<(Option<&ClassDecl>, &FnDecl)> {
        let mut out = Vec::new();
        for item in &self.items {
            match item {
                Item::Fn(f) => out.push((None, f)),
                Item::Class(c) => {
                    for m in &c.methods {
                        out.push((Some(c), m));
                    }
                }
                _ => {}
            }
        }
        out
    }
}
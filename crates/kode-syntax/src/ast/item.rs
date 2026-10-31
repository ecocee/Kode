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

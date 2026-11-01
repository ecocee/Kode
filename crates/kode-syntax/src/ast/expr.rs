use kode_diagnostics::Span;

use super::stmt::Block;

/// Every expression gets a unique id so later stages can attach types to it.
pub type ExprId = u32;

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: ExprId,
    pub span: Span,
    pub kind: ExprKind,
}

/// A call argument, positional or named (`name: value`).
#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<(String, Span)>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    /// A string with `${...}` holes.
    Interp(Vec<InterpPart>),
    /// The empty value of an optional.
    None,
    Ident(String),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Index {
        obj: Box<Expr>,
        index: Box<Expr>,
    },
    /// `obj.name`, or `obj?.name` when `safe`.
    Member {
        obj: Box<Expr>,
        name: String,
        name_span: Span,
        safe: bool,
    },
    MethodCall {
        obj: Box<Expr>,
        name: String,
        name_span: Span,
        args: Vec<Arg>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `a ?? b`
    Coalesce {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `a!`
    Unwrap(Box<Expr>),
    Call {
        callee: String,
        callee_span: Span,
        args: Vec<Arg>,
    },
    /// `expr catch err { ... }`
    Catch {
        expr: Box<Expr>,
        var: String,
        var_span: Span,
        block: Block,
    },
}

#[derive(Debug, Clone)]
pub enum InterpPart {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    /// `key in map`
    In,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::In => "in",
        }
    }
}
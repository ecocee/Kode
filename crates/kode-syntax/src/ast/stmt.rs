use kode_diagnostics::Span;

use super::expr::Expr;

pub type Block = Vec<Stmt>;

/// A type written in the source, for example `int`.
#[derive(Debug, Clone)]
pub struct TypeAnn {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `name = value` or `name: type = value`. Compound forms such as
    /// `x += 1` are turned into this by the parser.
    Assign {
        name: String,
        name_span: Span,
        ty: Option<TypeAnn>,
        value: Expr,
    },
    /// An expression on its own line, for example `print("hi")`.
    Expr(Expr),
    If {
        cond: Expr,
        then: Block,
        /// `else if` is an `else` block holding one `If` statement.
        else_: Option<Block>,
    },
    While {
        cond: Expr,
        body: Block,
    },
    /// `for i in start..end` (or `..=`)
    For {
        var: String,
        var_span: Span,
        start: Expr,
        end: Expr,
        inclusive: bool,
        body: Block,
    },
    Break(Span),
    Continue(Span),
    Return {
        value: Option<Expr>,
        span: Span,
    },
}

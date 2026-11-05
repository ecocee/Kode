use kode_diagnostics::Span;

use super::expr::Expr;

pub type Block = Vec<Stmt>;

/// A type written in the source: `int`, `[int]`, `{string: int}`, `User?`.
#[derive(Debug, Clone)]
pub struct TypeAnn {
    pub kind: TypeAnnKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeAnnKind {
    Name(String),
    List(Box<TypeAnn>),
    /// `{string: T}`: the key is always a string, this is the value type.
    Map(Box<TypeAnn>),
    Opt(Box<TypeAnn>),
    /// `Task<T>`
    Task(Box<TypeAnn>),
}

/// Something that can be assigned to.
#[derive(Debug, Clone)]
pub enum Target {
    Var(String, Span),
    Index { obj: Expr, index: Expr },
    Field { obj: Expr, name: String, name_span: Span },
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_`
    Wildcard,
    /// A literal or an enum variant such as `Color.Red`.
    Value(Expr),
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub pattern: Pattern,
    pub pattern_span: Span,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `target = value` or `name: type = value`. Compound forms such as
    /// `x += 1` are turned into this by the parser.
    Assign {
        target: Target,
        ty: Option<TypeAnn>,
        value: Expr,
        span: Span,
    },
    Expr(Expr),
    If {
        cond: Expr,
        then: Block,
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
    /// `for x in list`, `for i, x in list`, `for k, v in map`
    ForIn {
        vars: Vec<(String, Span)>,
        iter: Expr,
        body: Block,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<Arm>,
        span: Span,
    },
    Throw {
        value: Expr,
        span: Span,
    },
    Try {
        body: Block,
        /// The name bound to the caught error; `none` when there is no `catch`.
        var: Option<(String, Span)>,
        handler: Option<Block>,
        /// Runs on both the normal and the error path, and before any
        /// `return`, `break` or `continue` that leaves the block.
        finally: Option<Block>,
    },
    Break(Span),
    Continue(Span),
    Return {
        value: Option<Expr>,
        span: Span,
    },
}
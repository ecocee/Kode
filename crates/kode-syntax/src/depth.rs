//! A non-recursive check that the syntax tree is not deeper than the compiler
//! can walk.
//!
//! The parser guards its own recursion (see [`MAX_NESTING`]), but a left-deep
//! tree is built by a loop: `a + b + c + ...` costs the parser two frames no
//! matter how many terms it has, while the type checker, MIR lowering, the code
//! generator and even `Drop` recurse once per term. A few thousand terms abort
//! the compiler with a stack overflow and no diagnostic. This pass walks the
//! finished tree with an explicit stack and measures the depth those passes will
//! see, so the limit holds for every shape and not just the recursive ones.
//!
//! [`MAX_NESTING`]: crate::parser::MAX_NESTING

use kode_diagnostics::{Diagnostic, Span};

use crate::ast::*;
use crate::parser::MAX_NESTING;

/// One node waiting to be visited, with the depth it sits at.
enum Node<'a> {
    Expr(&'a Expr, u32),
    Stmt(&'a Stmt, u32),
    Block(&'a Block, u32),
    Ty(&'a TypeAnn, u32),
    Target(&'a Target, u32),
    Arm(&'a Arm, u32),
    Pattern(&'a Pattern, u32),
}

/// Returns the first node deeper than [`MAX_NESTING`], in a fixed traversal
/// order so the diagnostic is stable.
pub(crate) fn too_deep(program: &Program) -> Option<Span> {
    let mut stack: Vec<Node> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                for p in &f.params {
                    stack.push(Node::Ty(&p.ty, 1));
                }
                if let Some(r) = &f.ret {
                    stack.push(Node::Ty(r, 1));
                }
                match &f.body {
                    FnBody::Block(b) => stack.push(Node::Block(b, 1)),
                    FnBody::Expr(e) => stack.push(Node::Expr(e, 1)),
                }
            }
            Item::Const(c) => {
                if let Some(t) = &c.ty {
                    stack.push(Node::Ty(t, 1));
                }
                stack.push(Node::Expr(&c.value, 1));
            }
            Item::Class(c) => {
                for f in &c.fields {
                    stack.push(Node::Ty(&f.ty, 1));
                    if let Some(d) = &f.default {
                        stack.push(Node::Expr(d, 1));
                    }
                }
                for m in &c.methods {
                    for p in &m.params {
                        stack.push(Node::Ty(&p.ty, 1));
                    }
                    if let Some(r) = &m.ret {
                        stack.push(Node::Ty(r, 1));
                    }
                    match &m.body {
                        FnBody::Block(b) => stack.push(Node::Block(b, 1)),
                        FnBody::Expr(e) => stack.push(Node::Expr(e, 1)),
                    }
                }
            }
            Item::Interface(i) => {
                for m in &i.methods {
                    for p in &m.params {
                        stack.push(Node::Ty(&p.ty, 1));
                    }
                    if let Some(r) = &m.ret {
                        stack.push(Node::Ty(r, 1));
                    }
                }
            }
            Item::Test(t) => stack.push(Node::Block(&t.body, 1)),
            Item::Stmt(s) => stack.push(Node::Stmt(s, 1)),
            Item::Enum(_) | Item::Import(_) => {}
        }
    }

    while let Some(node) = stack.pop() {
        let (depth, span) = match &node {
            Node::Expr(e, d) => (*d, e.span),
            Node::Stmt(s, d) => (*d, stmt_span(s)),
            Node::Block(b, d) => (*d, block_span(b)),
            Node::Ty(t, d) => (*d, t.span),
            Node::Target(t, d) => (*d, target_span(t)),
            Node::Arm(a, d) => (*d, a.pattern_span),
            Node::Pattern(p, d) => (*d, pattern_span(p)),
        };
        if depth > MAX_NESTING {
            return Some(span);
        }
        let next = depth + 1;
        match node {
            Node::Expr(e, _) => match &e.kind {
                ExprKind::Int(_)
                | ExprKind::Float(_)
                | ExprKind::Bool(_)
                | ExprKind::Str(_)
                | ExprKind::None
                | ExprKind::Ident(_) => {}
                ExprKind::Interp(parts) => {
                    for p in parts {
                        if let InterpPart::Expr(x) = p {
                            stack.push(Node::Expr(x, next));
                        }
                    }
                }
                ExprKind::List(items) => {
                    for x in items {
                        stack.push(Node::Expr(x, next));
                    }
                }
                ExprKind::Map(pairs) => {
                    for (k, v) in pairs {
                        stack.push(Node::Expr(k, next));
                        stack.push(Node::Expr(v, next));
                    }
                }
                ExprKind::Index { obj, index } => {
                    stack.push(Node::Expr(obj, next));
                    stack.push(Node::Expr(index, next));
                }
                ExprKind::Slice {
                    obj, start, end, ..
                } => {
                    stack.push(Node::Expr(obj, next));
                    if let Some(s) = start {
                        stack.push(Node::Expr(s, next));
                    }
                    if let Some(e) = end {
                        stack.push(Node::Expr(e, next));
                    }
                }
                ExprKind::Member { obj, .. } => stack.push(Node::Expr(obj, next)),
                ExprKind::MethodCall { obj, args, .. } => {
                    stack.push(Node::Expr(obj, next));
                    push_args(args, next, &mut stack);
                }
                ExprKind::Unary { expr, .. } => stack.push(Node::Expr(expr, next)),
                ExprKind::Binary { lhs, rhs, .. } | ExprKind::Coalesce { lhs, rhs } => {
                    stack.push(Node::Expr(lhs, next));
                    stack.push(Node::Expr(rhs, next));
                }
                ExprKind::Unwrap(x) | ExprKind::Await(x) => stack.push(Node::Expr(x, next)),
                ExprKind::Call { args, .. } | ExprKind::Spawn { args, .. } => {
                    push_args(args, next, &mut stack)
                }
                ExprKind::Closure { body, .. } => stack.push(Node::Expr(body, next)),
                ExprKind::Catch { expr, block, .. } => {
                    stack.push(Node::Expr(expr, next));
                    stack.push(Node::Block(block, next));
                }
            },
            Node::Stmt(s, _) => match s {
                Stmt::Assign {
                    target, ty, value, ..
                } => {
                    stack.push(Node::Target(target, next));
                    if let Some(t) = ty {
                        stack.push(Node::Ty(t, next));
                    }
                    stack.push(Node::Expr(value, next));
                }
                Stmt::Expr(e) => stack.push(Node::Expr(e, next)),
                Stmt::If { cond, then, else_ } => {
                    stack.push(Node::Expr(cond, next));
                    stack.push(Node::Block(then, next));
                    if let Some(b) = else_ {
                        stack.push(Node::Block(b, next));
                    }
                }
                Stmt::While { cond, body } => {
                    stack.push(Node::Expr(cond, next));
                    stack.push(Node::Block(body, next));
                }
                Stmt::For {
                    start, end, body, ..
                } => {
                    stack.push(Node::Expr(start, next));
                    stack.push(Node::Expr(end, next));
                    stack.push(Node::Block(body, next));
                }
                Stmt::ForIn { iter, body, .. } => {
                    stack.push(Node::Expr(iter, next));
                    stack.push(Node::Block(body, next));
                }
                Stmt::Match {
                    scrutinee, arms, ..
                } => {
                    stack.push(Node::Expr(scrutinee, next));
                    for a in arms {
                        stack.push(Node::Arm(a, next));
                    }
                }
                Stmt::Throw { value, .. } => stack.push(Node::Expr(value, next)),
                Stmt::Try {
                    body,
                    handler,
                    finally,
                    ..
                } => {
                    stack.push(Node::Block(body, next));
                    if let Some(b) = handler {
                        stack.push(Node::Block(b, next));
                    }
                    if let Some(b) = finally {
                        stack.push(Node::Block(b, next));
                    }
                }
                Stmt::Return { value, .. } => {
                    if let Some(e) = value {
                        stack.push(Node::Expr(e, next));
                    }
                }
                Stmt::Break(_) | Stmt::Continue(_) => {}
            },
            Node::Block(b, _) => {
                for s in b {
                    stack.push(Node::Stmt(s, next));
                }
            }
            Node::Ty(t, _) => match &t.kind {
                TypeAnnKind::Name(_) => {}
                TypeAnnKind::List(x)
                | TypeAnnKind::Map(x)
                | TypeAnnKind::Opt(x)
                | TypeAnnKind::Task(x)
                | TypeAnnKind::Channel(x) => stack.push(Node::Ty(x, next)),
                TypeAnnKind::Fn(params, ret) => {
                    for p in params {
                        stack.push(Node::Ty(p, next));
                    }
                    stack.push(Node::Ty(ret, next));
                }
            },
            Node::Target(t, _) => match t {
                Target::Var(..) => {}
                Target::Index { obj, index } => {
                    stack.push(Node::Expr(obj, next));
                    stack.push(Node::Expr(index, next));
                }
                Target::Field { obj, .. } => stack.push(Node::Expr(obj, next)),
            },
            Node::Arm(a, _) => {
                stack.push(Node::Pattern(&a.pattern, next));
                stack.push(Node::Block(&a.body, next));
            }
            Node::Pattern(p, _) => {
                if let Pattern::Value(e) = p {
                    stack.push(Node::Expr(e, next));
                }
            }
        }
    }
    None
}

fn push_args<'a>(args: &'a [Arg], depth: u32, stack: &mut Vec<Node<'a>>) {
    for a in args {
        stack.push(Node::Expr(&a.value, depth));
    }
}

fn stmt_span(s: &Stmt) -> Span {
    match s {
        Stmt::Assign { span, .. }
        | Stmt::Throw { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Match { span, .. } => *span,
        Stmt::Break(sp) | Stmt::Continue(sp) => *sp,
        Stmt::Expr(e) => e.span,
        Stmt::If { cond, .. } | Stmt::While { cond, .. } => cond.span,
        Stmt::For { start, .. } => start.span,
        Stmt::ForIn { iter, .. } => iter.span,
        Stmt::Try { body, .. } => block_span(body),
    }
}

fn block_span(b: &Block) -> Span {
    b.first()
        .map(stmt_span)
        .unwrap_or(Span { start: 0, end: 0 })
}

fn target_span(t: &Target) -> Span {
    match t {
        Target::Var(_, sp) => *sp,
        Target::Index { obj, .. } => obj.span,
        Target::Field { name_span, .. } => *name_span,
    }
}

fn pattern_span(p: &Pattern) -> Span {
    match p {
        Pattern::Wildcard => Span { start: 0, end: 0 },
        Pattern::Value(e) => e.span,
    }
}

/// The diagnostic for a tree that is too deep, used by the parser.
pub(crate) fn diagnostic(span: Span) -> Diagnostic {
    Diagnostic::error(
        "E0118",
        format!("this nests deeper than {MAX_NESTING} levels"),
        span,
    )
    .with_help("split it up: bind part of it to a variable on its own line, or use a loop")
}

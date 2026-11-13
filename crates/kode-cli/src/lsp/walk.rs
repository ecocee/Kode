//! A small syntax-tree visitor: every expression in a program, in source order.
//! Used by hover and by anything else that needs the node under a cursor.

use kode_syntax::ast::*;

pub fn items(list: &[Item], f: &mut dyn FnMut(&Expr)) {
    for item in list {
        match item {
            Item::Fn(d) => fn_body(d, f),
            Item::Const(c) => expr(&c.value, f),
            Item::Class(c) => {
                for field in &c.fields {
                    if let Some(default) = &field.default {
                        expr(default, f);
                    }
                }
                for method in &c.methods {
                    fn_body(method, f);
                }
            }
            Item::Test(t) => stmts(&t.body, f),
            Item::Stmt(s) => stmt(s, f),
            Item::Import(_) | Item::Interface(_) | Item::Enum(_) => {}
        }
    }
}

fn fn_body(d: &FnDecl, f: &mut dyn FnMut(&Expr)) {
    match &d.body {
        FnBody::Block(body) => stmts(body, f),
        FnBody::Expr(e) => expr(e, f),
    }
}

pub fn stmts(list: &[Stmt], f: &mut dyn FnMut(&Expr)) {
    for s in list {
        stmt(s, f);
    }
}

fn stmt(s: &Stmt, f: &mut dyn FnMut(&Expr)) {
    match s {
        Stmt::Assign { target, value, .. } => {
            target_exprs(target, f);
            expr(value, f);
        }
        Stmt::Expr(e) => expr(e, f),
        Stmt::If { cond, then, else_ } => {
            expr(cond, f);
            stmts(then, f);
            if let Some(e) = else_ {
                stmts(e, f);
            }
        }
        Stmt::While { cond, body } => {
            expr(cond, f);
            stmts(body, f);
        }
        Stmt::For {
            start, end, body, ..
        } => {
            expr(start, f);
            expr(end, f);
            stmts(body, f);
        }
        Stmt::ForIn { iter, body, .. } => {
            expr(iter, f);
            stmts(body, f);
        }
        Stmt::Match {
            scrutinee, arms, ..
        } => {
            expr(scrutinee, f);
            for arm in arms {
                if let Pattern::Value(p) = &arm.pattern {
                    expr(p, f);
                }
                stmts(&arm.body, f);
            }
        }
        Stmt::Throw { value, .. } => expr(value, f),
        Stmt::Try {
            body,
            handler,
            finally,
            ..
        } => {
            stmts(body, f);
            if let Some(h) = handler {
                stmts(h, f);
            }
            if let Some(fin) = finally {
                stmts(fin, f);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                expr(v, f);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn target_exprs(t: &Target, f: &mut dyn FnMut(&Expr)) {
    match t {
        Target::Var(_, _) => {}
        Target::Index { obj, index } => {
            expr(obj, f);
            expr(index, f);
        }
        Target::Field { obj, .. } => expr(obj, f),
    }
}

pub fn expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    match &e.kind {
        ExprKind::Interp(parts) => {
            for part in parts {
                if let InterpPart::Expr(x) = part {
                    expr(x, f);
                }
            }
        }
        ExprKind::List(items) => {
            for x in items {
                expr(x, f);
            }
        }
        ExprKind::Map(pairs) => {
            for (k, v) in pairs {
                expr(k, f);
                expr(v, f);
            }
        }
        ExprKind::Index { obj, index } => {
            expr(obj, f);
            expr(index, f);
        }
        ExprKind::Slice {
            obj, start, end, ..
        } => {
            expr(obj, f);
            for bound in [start, end].into_iter().flatten() {
                expr(bound, f);
            }
        }
        ExprKind::Member { obj, .. } | ExprKind::Is { obj, .. } => expr(obj, f),
        ExprKind::MethodCall { obj, args, .. } => {
            expr(obj, f);
            for a in args {
                expr(&a.value, f);
            }
        }
        ExprKind::Call { args, .. } | ExprKind::Spawn { args, .. } => {
            for a in args {
                expr(&a.value, f);
            }
        }
        ExprKind::Unary { expr: inner, .. } | ExprKind::Unwrap(inner) | ExprKind::Await(inner) => {
            expr(inner, f)
        }
        ExprKind::Binary { lhs, rhs, .. } | ExprKind::Coalesce { lhs, rhs } => {
            expr(lhs, f);
            expr(rhs, f);
        }
        ExprKind::Closure { body, .. } => expr(body, f),
        ExprKind::Catch {
            expr: inner, block, ..
        } => {
            expr(inner, f);
            stmts(block, f);
        }
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Str(_)
        | ExprKind::None
        | ExprKind::Ident(_) => {}
    }
}

/// The innermost expression whose span contains `offset`.
pub fn innermost_at(items: &[Item], offset: usize) -> Option<(ExprId, kode_diagnostics::Span)> {
    let mut best: Option<(ExprId, kode_diagnostics::Span)> = None;
    items_fn(items, &mut |e| {
        if e.span.start <= offset && offset <= e.span.end {
            let better = best
                .map(|(_, s)| e.span.end - e.span.start < s.end - s.start)
                .unwrap_or(true);
            if better {
                best = Some((e.id, e.span));
            }
        }
    });
    best
}

fn items_fn(list: &[Item], f: &mut dyn FnMut(&Expr)) {
    items(list, f);
}

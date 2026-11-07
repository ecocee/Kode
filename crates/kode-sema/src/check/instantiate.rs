//! Monomorphization of generic functions (ADR 0030).
//!
//! A generic declaration is checked once per concrete set of type arguments.
//! When a call site provides enough information to infer them, this module:
//!
//! 1. unifies each declared parameter type with the argument's type,
//! 2. clones the function with fresh expression ids and every type parameter
//!    replaced by the concrete type,
//! 3. registers the clone as an ordinary function.
//!
//! The clone is added to `Checker::bodies`, which the driver checks in a loop,
//! so instantiating a generic function from inside another generic body is just
//! another entry in that loop and nothing is re-entered mid-check. Identical
//! type arguments share one instantiation.

use std::collections::HashMap;

use kode_diagnostics::{Diagnostic, Span};
use kode_syntax::ast::*;

use super::Checker;
use crate::types::Type;

impl Checker {
    /// `callee` is generic: infer its type arguments from `tys`, then call the
    /// concrete instantiation like any other function.
    ///
    /// Eight parameters, but they are the call site's fields, not a bundle that
    /// belongs together: `id`, `callee`, `gidx`, `args`, `tys`, `span`,
    /// `callee_span` come straight from the expression being checked, and
    /// bundling them would only move the destructuring here.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn generic_call(
        &mut self,
        id: ExprId,
        callee: &str,
        gidx: usize,
        args: &[Arg],
        tys: &[Option<Type>],
        span: Span,
        callee_span: Span,
    ) -> Option<Type> {
        let decl = self.generic_decls[gidx].clone();
        if let Some(a) = args.iter().find(|a| a.name.is_some()) {
            self.errors.push(
                Diagnostic::error(
                    "E0215",
                    format!("`{callee}` is generic, so it takes positional arguments only"),
                    a.value.span,
                )
                .with_help("remove the parameter name from the argument"),
            );
        }
        if args.len() != decl.params.len() {
            self.errors.push(Diagnostic::error(
                "E0207",
                format!(
                    "`{callee}` takes {} argument{}, found {}",
                    decl.params.len(),
                    if decl.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
                span,
            ));
            return None;
        }

        let params: Vec<String> = decl.type_params.iter().map(|(n, _)| n.clone()).collect();
        let mut subst: HashMap<String, Type> = HashMap::new();
        for (i, (p, actual)) in decl.params.iter().zip(tys).enumerate() {
            let Some(actual) = actual else { continue };
            if !unify(&p.ty, actual, &params, &mut subst) {
                self.errors.push(
                    Diagnostic::error(
                        "E0208",
                        format!(
                            "argument {} of `{callee}` should be {}, found {}",
                            i + 1,
                            render(&p.ty, &subst),
                            actual.article()
                        ),
                        args.get(i).map(|a| a.value.span).unwrap_or(callee_span),
                    )
                    .with_help("the type arguments come from the arguments, so they must agree"),
                );
                return None;
            }
        }
        let missing: Vec<&String> = params.iter().filter(|n| !subst.contains_key(*n)).collect();
        if let Some(name) = missing.first() {
            self.errors.push(
                Diagnostic::error(
                    "E0250",
                    format!("cannot infer the type parameter `{name}` of `{callee}`"),
                    span,
                )
                .with_help(
                    "it does not appear in a parameter type; pass a value that mentions it, or use a non-generic function",
                ),
            );
            return None;
        }

        let idx = self.instantiate(gidx, &decl, &params, &subst, callee_span)?;
        self.generic_calls.insert(id, idx);
        let sig = self.fns[idx].clone();
        let defaults = vec![false; sig.params.len()];
        self.check_call_args(id, &sig.params, args, tys, &defaults, span, callee);
        self.note_call(idx);
        Some(sig.ret)
    }

    fn instantiate(
        &mut self,
        gidx: usize,
        decl: &FnDecl,
        params: &[String],
        subst: &HashMap<String, Type>,
        span: Span,
    ) -> Option<usize> {
        let types: Vec<Type> = params.iter().map(|n| subst[n].clone()).collect();
        let key: Vec<String> = types.iter().map(mangle).collect();
        if let Some((_, idx)) = self
            .instances
            .iter()
            .find(|((g, k), _)| *g == gidx && *k == key)
        {
            return Some(*idx);
        }

        let mut body = decl.clone();
        renumber_fn(&mut body, &mut self.next_expr_id);
        for p in &mut body.params {
            substitute_ann(&mut p.ty, subst);
        }
        if let Some(ret) = &mut body.ret {
            substitute_ann(ret, subst);
        }

        let mut sig_params: Vec<(String, Type)> = Vec::new();
        for p in &body.params {
            let ty = self.resolve_type(&p.ty).unwrap_or(Type::Int);
            sig_params.push((p.name.clone(), ty));
        }
        let ret = match &body.ret {
            Some(a) => self.resolve_type(a).unwrap_or(Type::Unit),
            None => Type::Unit,
        };
        let name = format!("{}.{}", decl.name, key.join("_"));
        let idx = self.fns.len();
        self.fns.push(super::FnSig {
            name,
            params: sig_params,
            ret,
            class: None,
            type_params: Vec::new(),
        });
        self.instances.push(((gidx, key), idx));
        self.bodies.push(body);
        self.direct_throw.push(false);
        self.calls.push(Vec::new());
        let _ = span;
        Some(idx)
    }
}

/// Match a declared parameter type against the type of an argument, binding type
/// parameters as they are met. `pair<T>(a: T, b: T)` called with `(1, "x")` fails
/// because `T` is already `int`.
fn unify(
    ann: &TypeAnn,
    actual: &Type,
    params: &[String],
    subst: &mut HashMap<String, Type>,
) -> bool {
    match &ann.kind {
        TypeAnnKind::Name(n) if params.iter().any(|p| p == n) => match subst.get(n) {
            Some(t) => t == actual,
            None => {
                subst.insert(n.clone(), actual.clone());
                true
            }
        },
        TypeAnnKind::Name(n) => match actual {
            Type::Class(c) | Type::Interface(c) | Type::Enum(c) => c == n,
            _ => Type::from_name(n).as_ref() == Some(actual),
        },
        TypeAnnKind::List(inner) => match actual {
            Type::List(a) => unify(inner, a, params, subst),
            _ => false,
        },
        TypeAnnKind::Map(inner) => match actual {
            Type::Map(a) => unify(inner, a, params, subst),
            _ => false,
        },
        // `T?` also accepts a plain `T`, like assignment does.
        TypeAnnKind::Opt(inner) => match actual {
            Type::Opt(a) => unify(inner, a, params, subst),
            other => unify(inner, other, params, subst),
        },
        TypeAnnKind::Task(inner) => match actual {
            Type::Task(a) => unify(inner, a, params, subst),
            _ => false,
        },
        TypeAnnKind::Channel(inner) => match actual {
            Type::Chan(a) => unify(inner, a, params, subst),
            _ => false,
        },
        TypeAnnKind::Fn(ps, ret) => match actual {
            Type::Fn(aps, ar) if aps.len() == ps.len() => {
                ps.iter().zip(aps).all(|(p, a)| unify(p, a, params, subst))
                    && unify(ret, ar, params, subst)
            }
            _ => false,
        },
    }
}

/// Replace every type parameter with its concrete type.
fn substitute_ann(ann: &mut TypeAnn, subst: &HashMap<String, Type>) {
    match &mut ann.kind {
        TypeAnnKind::Name(n) => {
            if let Some(t) = subst.get(n) {
                *ann = ann_for(t, ann.span);
            }
        }
        TypeAnnKind::List(i)
        | TypeAnnKind::Map(i)
        | TypeAnnKind::Opt(i)
        | TypeAnnKind::Task(i)
        | TypeAnnKind::Channel(i) => substitute_ann(i, subst),
        TypeAnnKind::Fn(ps, ret) => {
            for p in ps {
                substitute_ann(p, subst);
            }
            substitute_ann(ret, subst);
        }
    }
}

fn ann_for(t: &Type, span: Span) -> TypeAnn {
    let kind = match t {
        Type::Int => TypeAnnKind::Name("int".into()),
        Type::Float => TypeAnnKind::Name("float".into()),
        Type::Bool => TypeAnnKind::Name("bool".into()),
        Type::Str => TypeAnnKind::Name("string".into()),
        Type::Unit => TypeAnnKind::Name("nothing".into()),
        Type::List(i) => TypeAnnKind::List(Box::new(ann_for(i, span))),
        Type::Map(i) => TypeAnnKind::Map(Box::new(ann_for(i, span))),
        Type::Opt(i) => TypeAnnKind::Opt(Box::new(ann_for(i, span))),
        Type::Task(i) => TypeAnnKind::Task(Box::new(ann_for(i, span))),
        Type::Chan(i) => TypeAnnKind::Channel(Box::new(ann_for(i, span))),
        Type::Fn(ps, r) => TypeAnnKind::Fn(
            ps.iter().map(|p| ann_for(p, span)).collect(),
            Box::new(ann_for(r, span)),
        ),
        Type::Class(n) | Type::Interface(n) | Type::Enum(n) => TypeAnnKind::Name(n.clone()),
        Type::NoneT => TypeAnnKind::Name("none".into()),
    };
    TypeAnn { kind, span }
}

/// A stable text form used in diagnostics and in the names of instantiations.
fn render(ann: &TypeAnn, subst: &HashMap<String, Type>) -> String {
    match &ann.kind {
        TypeAnnKind::Name(n) => subst.get(n).map(|t| t.name()).unwrap_or_else(|| n.clone()),
        TypeAnnKind::List(i) => format!("[{}]", render(i, subst)),
        TypeAnnKind::Map(i) => format!("{{string: {}}}", render(i, subst)),
        TypeAnnKind::Opt(i) => format!("{}?", render(i, subst)),
        TypeAnnKind::Task(i) => format!("Task<{}>", render(i, subst)),
        TypeAnnKind::Channel(i) => format!("Channel<{}>", render(i, subst)),
        TypeAnnKind::Fn(ps, r) => {
            let ps: Vec<String> = ps.iter().map(|p| render(p, subst)).collect();
            format!("fn({}) -> {}", ps.join(", "), render(r, subst))
        }
    }
}

fn mangle(t: &Type) -> String {
    match t {
        Type::Int => "int".into(),
        Type::Float => "float".into(),
        Type::Bool => "bool".into(),
        Type::Str => "str".into(),
        Type::Unit => "unit".into(),
        Type::List(i) => format!("list_{}", mangle(i)),
        Type::Map(i) => format!("map_{}", mangle(i)),
        Type::Opt(i) => format!("opt_{}", mangle(i)),
        Type::Task(i) => format!("task_{}", mangle(i)),
        Type::Chan(i) => format!("chan_{}", mangle(i)),
        Type::Fn(ps, r) => {
            let ps: Vec<String> = ps.iter().map(mangle).collect();
            format!("fn{}_{}_to_{}", ps.len(), ps.join("_"), mangle(r))
        }
        Type::Class(n) | Type::Interface(n) | Type::Enum(n) => {
            format!(
                "cls_{}",
                n.chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '_' })
                    .collect::<String>()
            )
        }
        Type::NoneT => "none".into(),
    }
}

// ---- fresh expression ids -------------------------------------------------

fn renumber_fn(f: &mut FnDecl, next: &mut u32) {
    match &mut f.body {
        FnBody::Block(b) => renumber_block(b, next),
        FnBody::Expr(e) => renumber_expr(e, next),
    }
}

fn renumber_block(block: &mut Block, next: &mut u32) {
    for s in block {
        renumber_stmt(s, next);
    }
}

fn renumber_stmt(s: &mut Stmt, next: &mut u32) {
    match s {
        Stmt::Assign { target, value, .. } => {
            match target {
                Target::Var(_, _) => {}
                Target::Index { obj, index } => {
                    renumber_expr(obj, next);
                    renumber_expr(index, next);
                }
                Target::Field { obj, .. } => renumber_expr(obj, next),
            }
            renumber_expr(value, next);
        }
        Stmt::Expr(e) => renumber_expr(e, next),
        Stmt::If { cond, then, else_ } => {
            renumber_expr(cond, next);
            renumber_block(then, next);
            if let Some(b) = else_ {
                renumber_block(b, next);
            }
        }
        Stmt::While { cond, body } => {
            renumber_expr(cond, next);
            renumber_block(body, next);
        }
        Stmt::For {
            start, end, body, ..
        } => {
            renumber_expr(start, next);
            renumber_expr(end, next);
            renumber_block(body, next);
        }
        Stmt::ForIn { iter, body, .. } => {
            renumber_expr(iter, next);
            renumber_block(body, next);
        }
        Stmt::Match {
            scrutinee, arms, ..
        } => {
            renumber_expr(scrutinee, next);
            for arm in arms {
                if let Pattern::Value(v) = &mut arm.pattern {
                    renumber_expr(v, next);
                }
                renumber_block(&mut arm.body, next);
            }
        }
        Stmt::Throw { value, .. } => renumber_expr(value, next),
        Stmt::Try {
            body,
            handler,
            finally,
            ..
        } => {
            renumber_block(body, next);
            if let Some(b) = handler {
                renumber_block(b, next);
            }
            if let Some(b) = finally {
                renumber_block(b, next);
            }
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
        Stmt::Return { value, .. } => {
            if let Some(e) = value {
                renumber_expr(e, next);
            }
        }
    }
}

fn renumber_expr(e: &mut Expr, next: &mut u32) {
    e.id = *next;
    *next += 1;
    match &mut e.kind {
        ExprKind::Interp(parts) => {
            for p in parts {
                if let InterpPart::Expr(x) = p {
                    renumber_expr(x, next);
                }
            }
        }
        ExprKind::List(items) => {
            for x in items {
                renumber_expr(x, next);
            }
        }
        ExprKind::Map(pairs) => {
            for (k, v) in pairs {
                renumber_expr(k, next);
                renumber_expr(v, next);
            }
        }
        ExprKind::Index { obj, index } => {
            renumber_expr(obj, next);
            renumber_expr(index, next);
        }
        ExprKind::Slice {
            obj, start, end, ..
        } => {
            renumber_expr(obj, next);
            for bound in [start, end].into_iter().flatten() {
                renumber_expr(bound, next);
            }
        }
        ExprKind::Member { obj, .. } => renumber_expr(obj, next),
        ExprKind::MethodCall { obj, args, .. } => {
            renumber_expr(obj, next);
            for a in args {
                renumber_expr(&mut a.value, next);
            }
        }
        ExprKind::Unary { expr, .. } | ExprKind::Unwrap(expr) | ExprKind::Await(expr) => {
            renumber_expr(expr, next)
        }
        ExprKind::Binary { lhs, rhs, .. } | ExprKind::Coalesce { lhs, rhs } => {
            renumber_expr(lhs, next);
            renumber_expr(rhs, next);
        }
        ExprKind::Call { args, .. } | ExprKind::Spawn { args, .. } => {
            for a in args {
                renumber_expr(&mut a.value, next);
            }
        }
        ExprKind::Catch { expr, block, .. } => {
            renumber_expr(expr, next);
            renumber_block(block, next);
        }
        ExprKind::Closure { body, .. } => renumber_expr(body, next),
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Str(_)
        | ExprKind::None
        | ExprKind::Ident(_) => {}
    }
}

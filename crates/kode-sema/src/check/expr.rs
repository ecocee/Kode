use kode_diagnostics::{Diagnostic, Span};
use kode_syntax::ast::*;

use super::Checker;
use crate::builtins::{self, Builtin, BUILTIN_NAMES};
use crate::suggest::closest;
use crate::types::Type;

impl Checker {
    /// Check an expression and return its type. `None` means an error was
    /// already reported (so callers do not pile on more errors).
    pub(crate) fn expr(&mut self, e: &Expr) -> Option<Type> {
        let ty = match &e.kind {
            ExprKind::Int(_) => Some(Type::Int),
            ExprKind::Float(_) => Some(Type::Float),
            ExprKind::Bool(_) => Some(Type::Bool),
            ExprKind::Str(_) => Some(Type::Str),
            ExprKind::Interp(parts) => {
                let mut ok = true;
                for p in parts {
                    if let InterpPart::Expr(x) = p {
                        match self.expr(x) {
                            Some(Type::Unit) => {
                                self.errors.push(Diagnostic::error(
                                    "E0205",
                                    "this call returns nothing, so it cannot go inside a string",
                                    x.span,
                                ));
                                ok = false;
                            }
                            Some(_) => {}
                            None => ok = false,
                        }
                    }
                }
                ok.then_some(Type::Str)
            }
            ExprKind::Ident(name) => self.ident(name, e.span),
            ExprKind::Unary { op, expr } => self.unary(*op, expr, e.span),
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e.span),
            ExprKind::Call {
                callee,
                callee_span,
                args,
            } => self.call(callee, *callee_span, args, e.span),
        };
        if let Some(t) = ty {
            self.expr_types[e.id as usize] = t;
        }
        ty
    }

    fn ident(&mut self, name: &str, span: Span) -> Option<Type> {
        if let Some(t) = self.scopes.use_var(name) {
            return Some(t);
        }
        if let Some((t, _)) = self.consts.get(name) {
            return Some(*t);
        }
        let mut d = Diagnostic::error("E0201", format!("unknown variable `{name}`"), span);
        let candidates = self
            .scopes
            .names()
            .into_iter()
            .chain(self.consts.keys().map(|s| s.as_str()));
        d = match closest(name, candidates) {
            Some(s) => d.with_help(format!("did you mean `{s}`?")),
            None => d.with_help(format!("create it first with `{name} = ...` on an earlier line")),
        };
        self.errors.push(d);
        None
    }

    fn unary(&mut self, op: UnOp, inner: &Expr, span: Span) -> Option<Type> {
        let t = self.expr(inner)?;
        match (op, t) {
            (UnOp::Neg, Type::Int) | (UnOp::Neg, Type::Float) => Some(t),
            (UnOp::Not, Type::Bool) => Some(Type::Bool),
            (UnOp::Neg, _) => {
                self.errors.push(Diagnostic::error(
                    "E0203",
                    format!("cannot negate {}", t.article()),
                    span,
                ));
                None
            }
            (UnOp::Not, _) => {
                self.errors.push(
                    Diagnostic::error(
                        "E0203",
                        format!("`not` needs a bool, found {}", t.article()),
                        span,
                    )
                    .with_help("compare it first, for example `not (x == 0)`"),
                );
                None
            }
        }
    }

    fn binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Option<Type> {
        let l = self.expr(lhs);
        let r = self.expr(rhs);
        let (l, r) = (l?, r?);

        let mixed = matches!((l, r), (Type::Int, Type::Float) | (Type::Float, Type::Int));
        let bad = |this: &mut Checker, help: &str| {
            let mut d = Diagnostic::error(
                "E0203",
                format!(
                    "cannot apply `{}` to {} and {}",
                    op.symbol(),
                    l.article(),
                    r.article()
                ),
                span,
            );
            if !help.is_empty() {
                d = d.with_help(help.to_string());
            }
            this.errors.push(d);
            None
        };
        let mixed_error = |this: &mut Checker| {
            this.errors.push(
                Diagnostic::error(
                    "E0210",
                    format!("cannot mix an int and a float with `{}`", op.symbol()),
                    span,
                )
                .with_help("convert one side: float(x) or int(x)"),
            );
            None
        };

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => match (l, r) {
                (Type::Int, Type::Int) => {
                    if matches!(op, BinOp::Div | BinOp::Rem) && matches!(rhs.kind, ExprKind::Int(0)) {
                        self.errors.push(Diagnostic::error("E0204", "division by zero", rhs.span));
                        return None;
                    }
                    Some(Type::Int)
                }
                (Type::Float, Type::Float) if op != BinOp::Rem => Some(Type::Float),
                (Type::Str, Type::Str) if op == BinOp::Add => Some(Type::Str),
                _ if mixed => mixed_error(self),
                _ => bad(self, "arithmetic works on two ints or two floats; `+` also joins two strings"),
            },
            BinOp::Eq | BinOp::Ne => {
                if l == r && l != Type::Unit {
                    Some(Type::Bool)
                } else if mixed {
                    mixed_error(self)
                } else {
                    bad(self, "both sides must have the same type")
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (l, r) {
                (Type::Int, Type::Int) | (Type::Float, Type::Float) => Some(Type::Bool),
                _ if mixed => mixed_error(self),
                _ => bad(self, "only ints and floats can be ordered so far"),
            },
            BinOp::And | BinOp::Or => {
                if l == Type::Bool && r == Type::Bool {
                    Some(Type::Bool)
                } else {
                    bad(self, "`and` and `or` need two bools")
                }
            }
        }
    }

    fn call(&mut self, callee: &str, callee_span: Span, args: &[Expr], span: Span) -> Option<Type> {
        let mut tys: Vec<Option<Type>> = Vec::new();
        for a in args {
            tys.push(self.expr(a));
        }

        if let Some(b) = builtins::lookup(callee) {
            return self.builtin_call(b, callee, args, &tys, span);
        }

        if let Some(&idx) = self.fn_index.get(callee) {
            let sig = self.fns[idx].clone();
            if args.len() != sig.params.len() {
                self.errors.push(
                    Diagnostic::error(
                        "E0207",
                        format!(
                            "`{callee}` takes {} argument{}, found {}",
                            sig.params.len(),
                            if sig.params.len() == 1 { "" } else { "s" },
                            args.len()
                        ),
                        span,
                    )
                    .with_help(format!(
                        "its parameters are ({})",
                        sig.params
                            .iter()
                            .map(|(n, t)| format!("{n}: {}", t.name()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                );
                return None;
            }
            let mut ok = true;
            for (i, (pname, pty)) in sig.params.iter().enumerate() {
                match tys[i] {
                    Some(t) if t != *pty => {
                        self.errors.push(Diagnostic::error(
                            "E0208",
                            format!(
                                "argument `{pname}` of `{callee}` should be {}, found {}",
                                pty.article(),
                                t.article()
                            ),
                            args[i].span,
                        ));
                        ok = false;
                    }
                    None => ok = false,
                    _ => {}
                }
            }
            return ok.then_some(sig.ret).or(Some(sig.ret));
        }

        let mut d = Diagnostic::error("E0206", format!("unknown function `{callee}`"), callee_span);
        let candidates = self
            .fn_index
            .keys()
            .map(|s| s.as_str())
            .chain(BUILTIN_NAMES.iter().copied());
        d = match closest(callee, candidates) {
            Some(s) => d.with_help(format!("did you mean `{s}`?")),
            None => d.with_help("define it first with `fn name(...) { ... }`"),
        };
        self.errors.push(d);
        None
    }

    fn builtin_call(
        &mut self,
        b: Builtin,
        name: &str,
        args: &[Expr],
        tys: &[Option<Type>],
        span: Span,
    ) -> Option<Type> {
        if b == Builtin::Print {
            for (i, t) in tys.iter().enumerate() {
                if *t == Some(Type::Unit) {
                    self.errors.push(Diagnostic::error(
                        "E0205",
                        "this call returns nothing, so there is nothing to print",
                        args[i].span,
                    ));
                }
            }
            return Some(Type::Unit);
        }

        let (accepts, ret, expects): (&[Type], Type, &str) = match b {
            Builtin::Len => (&[Type::Str], Type::Int, "a string"),
            Builtin::Int => (&[Type::Int, Type::Float, Type::Bool], Type::Int, "an int, float or bool"),
            Builtin::Float => (&[Type::Int, Type::Float], Type::Float, "an int or float"),
            Builtin::Str => (
                &[Type::Int, Type::Float, Type::Bool, Type::Str],
                Type::Str,
                "an int, float, bool or string",
            ),
            Builtin::Assert => (&[Type::Bool], Type::Unit, "a bool"),
            Builtin::Print => unreachable!(),
        };
        if args.len() != 1 {
            self.errors.push(Diagnostic::error(
                "E0207",
                format!("`{name}` takes 1 argument, found {}", args.len()),
                span,
            ));
            return Some(ret);
        }
        if let Some(t) = tys[0] {
            if !accepts.contains(&t) {
                self.errors.push(Diagnostic::error(
                    "E0208",
                    format!("`{name}` expects {expects}, found {}", t.article()),
                    args[0].span,
                ));
            }
        }
        Some(ret)
    }
}

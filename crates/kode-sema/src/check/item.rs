use kode_diagnostics::Diagnostic;
use kode_syntax::ast::*;

use super::{Checker, ConstVal, FnSig};
use crate::builtins;
use crate::scope::Scopes;
use crate::types::Type;

impl Checker {
    pub(crate) fn resolve_type(&mut self, ann: &TypeAnn) -> Option<Type> {
        match Type::from_name(&ann.name) {
            Some(t) => Some(t),
            None => {
                self.errors.push(
                    Diagnostic::error(
                        "E0303",
                        format!("unknown type `{}`", ann.name),
                        ann.span,
                    )
                    .with_help("the types are int, float, bool and string"),
                );
                None
            }
        }
    }

    pub(crate) fn collect_consts(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Const(c) = item else { continue };
            let value = match &c.value.kind {
                ExprKind::Int(v) => Some((Type::Int, ConstVal::Int(*v))),
                ExprKind::Float(v) => Some((Type::Float, ConstVal::Float(*v))),
                ExprKind::Bool(v) => Some((Type::Bool, ConstVal::Bool(*v))),
                ExprKind::Str(s) => Some((Type::Str, ConstVal::Str(s.clone()))),
                ExprKind::Unary { op: UnOp::Neg, expr } => match expr.kind {
                    ExprKind::Int(v) => Some((Type::Int, ConstVal::Int(v.wrapping_neg()))),
                    ExprKind::Float(v) => Some((Type::Float, ConstVal::Float(-v))),
                    _ => None,
                },
                _ => None,
            };
            let Some((ty, val)) = value else {
                self.errors.push(
                    Diagnostic::error(
                        "E0307",
                        "a constant must be a plain value",
                        c.value.span,
                    )
                    .with_help("use a number, true, false or a string without `${}`"),
                );
                continue;
            };
            if let Some(ann) = &c.ty {
                if let Some(want) = self.resolve_type(ann) {
                    if want != ty {
                        self.errors.push(Diagnostic::error(
                            "E0202",
                            format!("`{}` is declared {} but its value is {}", c.name, want.article(), ty.article()),
                            c.value.span,
                        ));
                        continue;
                    }
                }
            }
            if self.consts.contains_key(&c.name) {
                self.errors.push(Diagnostic::error(
                    "E0308",
                    format!("the constant `{}` is defined twice", c.name),
                    c.name_span,
                ));
                continue;
            }
            self.consts.insert(c.name.clone(), (ty, val));
        }
    }

    pub(crate) fn collect_fns(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Fn(f) = item else { continue };
            let mut params: Vec<(String, Type)> = Vec::new();
            for p in &f.params {
                if params.iter().any(|(n, _)| *n == p.name) {
                    self.errors.push(Diagnostic::error(
                        "E0306",
                        format!("the parameter `{}` appears twice", p.name),
                        p.name_span,
                    ));
                }
                let ty = self.resolve_type(&p.ty).unwrap_or(Type::Int);
                params.push((p.name.clone(), ty));
            }
            let ret = match &f.ret {
                Some(a) => self.resolve_type(a).unwrap_or(Type::Unit),
                None => Type::Unit,
            };

            if builtins::lookup(&f.name).is_some() {
                self.errors.push(
                    Diagnostic::error(
                        "E0302",
                        format!("`{}` is a built-in function and cannot be redefined", f.name),
                        f.name_span,
                    )
                    .with_help("pick another name"),
                );
            } else if self.fn_index.contains_key(&f.name) {
                self.errors.push(Diagnostic::error(
                    "E0301",
                    format!("the function `{}` is defined twice", f.name),
                    f.name_span,
                ));
            } else {
                self.fn_index.insert(f.name.clone(), self.fns.len());
            }
            self.fns.push(FnSig {
                name: f.name.clone(),
                params,
                ret,
            });
        }
    }

    pub(crate) fn check_fn(&mut self, f: &FnDecl, index: usize) {
        let sig = self.fns[index].clone();
        self.scopes = Scopes::new();
        self.scopes.push();
        for (i, (name, ty)) in sig.params.iter().enumerate() {
            self.scopes.declare(name, *ty, f.params[i].name_span, true);
        }
        self.ret_type = sig.ret;
        self.in_fn = true;
        self.loop_depth = 0;

        match &f.body {
            FnBody::Block(stmts) => {
                for s in stmts {
                    self.stmt(s);
                }
                if sig.ret != Type::Unit && !super::stmt::always_returns(stmts) {
                    self.errors.push(
                        Diagnostic::error(
                            "E0305",
                            format!(
                                "`{}` must return {} on every path",
                                f.name,
                                sig.ret.article()
                            ),
                            f.name_span,
                        )
                        .with_help("end the function with a `return`, or give both branches of the last `if` a `return`"),
                    );
                }
            }
            FnBody::Expr(e) => {
                if let Some(t) = self.expr(e) {
                    if t != sig.ret {
                        let help = if f.ret.is_none() && t != Type::Unit {
                            format!("write `-> {}` after the parameters", t.name())
                        } else {
                            format!("the body should give {}", sig.ret.article())
                        };
                        self.errors.push(
                            Diagnostic::error(
                                "E0304",
                                format!(
                                    "`{}` is declared to give {} but the body gives {}",
                                    f.name,
                                    sig.ret.article(),
                                    t.article()
                                ),
                                e.span,
                            )
                            .with_help(help),
                        );
                    }
                }
            }
        }
        let mut warnings = std::mem::take(&mut self.warnings);
        self.scopes.pop(&mut warnings);
        self.warnings = warnings;
        self.in_fn = false;
    }
}

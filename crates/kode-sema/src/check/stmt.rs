use kode_diagnostics::Diagnostic;
use kode_syntax::ast::*;

use super::Checker;
use crate::types::Type;

/// True when every path through these statements ends in a `return`.
pub(crate) fn always_returns(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Return { .. } => true,
        Stmt::If {
            then,
            else_: Some(else_),
            ..
        } => always_returns(then) && always_returns(else_),
        _ => false,
    })
}

impl Checker {
    fn block(&mut self, stmts: &[Stmt]) {
        self.scopes.push();
        for s in stmts {
            self.stmt(s);
        }
        let mut warnings = std::mem::take(&mut self.warnings);
        self.scopes.pop(&mut warnings);
        self.warnings = warnings;
    }

    fn condition(&mut self, cond: &Expr, what: &str) {
        if let Some(t) = self.expr(cond) {
            if t != Type::Bool {
                self.errors.push(
                    Diagnostic::error(
                        "E0209",
                        format!("the condition of {what} must be a bool, found {}", t.article()),
                        cond.span,
                    )
                    .with_help("compare it, for example `x != 0`"),
                );
            }
        }
    }

    pub(crate) fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.expr(e);
            }
            Stmt::Assign {
                name,
                name_span,
                ty,
                value,
            } => self.assign(name, *name_span, ty.as_ref(), value),
            Stmt::If { cond, then, else_ } => {
                self.condition(cond, "`if`");
                self.block(then);
                if let Some(e) = else_ {
                    self.block(e);
                }
            }
            Stmt::While { cond, body } => {
                self.condition(cond, "`while`");
                self.loop_depth += 1;
                self.block(body);
                self.loop_depth -= 1;
            }
            Stmt::For {
                var,
                var_span,
                start,
                end,
                body,
                ..
            } => {
                for e in [start, end] {
                    if let Some(t) = self.expr(e) {
                        if t != Type::Int {
                            self.errors.push(
                                Diagnostic::error(
                                    "E0203",
                                    format!("a range needs ints, found {}", t.article()),
                                    e.span,
                                )
                                .with_help("use int(x) to convert a float"),
                            );
                        }
                    }
                }
                self.scopes.push();
                self.scopes.declare(var, Type::Int, *var_span, true);
                self.loop_depth += 1;
                for s in body {
                    self.stmt(s);
                }
                self.loop_depth -= 1;
                let mut warnings = std::mem::take(&mut self.warnings);
                self.scopes.pop(&mut warnings);
                self.warnings = warnings;
            }
            Stmt::Break(span) | Stmt::Continue(span) => {
                if self.loop_depth == 0 {
                    let word = if matches!(stmt, Stmt::Break(_)) { "break" } else { "continue" };
                    self.errors.push(Diagnostic::error(
                        "E0314",
                        format!("`{word}` can only be used inside a loop"),
                        *span,
                    ));
                }
            }
            Stmt::Return { value, span } => {
                if !self.in_fn {
                    self.errors.push(
                        Diagnostic::error("E0311", "`return` can only be used inside a function", *span)
                            .with_help("top-level code simply runs to the end"),
                    );
                    return;
                }
                match value {
                    Some(v) => {
                        if let Some(t) = self.expr(v) {
                            if self.ret_type == Type::Unit {
                                self.errors.push(
                                    Diagnostic::error(
                                        "E0310",
                                        "this function returns nothing, but `return` has a value",
                                        v.span,
                                    )
                                    .with_help(format!("declare the type, for example `-> {}`", t.name())),
                                );
                            } else if t != self.ret_type {
                                self.errors.push(Diagnostic::error(
                                    "E0310",
                                    format!(
                                        "expected to return {}, found {}",
                                        self.ret_type.article(),
                                        t.article()
                                    ),
                                    v.span,
                                ));
                            }
                        }
                    }
                    None => {
                        if self.ret_type != Type::Unit {
                            self.errors.push(Diagnostic::error(
                                "E0310",
                                format!("expected to return {}", self.ret_type.article()),
                                *span,
                            ));
                        }
                    }
                }
            }
        }
    }

    fn assign(&mut self, name: &str, name_span: kode_diagnostics::Span, ty: Option<&TypeAnn>, value: &Expr) {
        let vt = self.expr(value);
        let ann = ty.and_then(|a| self.resolve_type(a));
        let Some(vt) = vt else { return };
        if vt == Type::Unit {
            self.errors.push(
                Diagnostic::error(
                    "E0205",
                    "this call returns nothing, so it cannot be stored in a variable",
                    value.span,
                )
                .with_help("`print(...)` shows text and gives back no value"),
            );
            return;
        }
        if self.consts.contains_key(name) {
            self.errors.push(Diagnostic::error(
                "E0312",
                format!("`{name}` is a constant and cannot be changed"),
                name_span,
            ));
            return;
        }
        match self.scopes.peek_var(name) {
            Some(existing) => {
                if ty.is_some() {
                    self.errors.push(
                        Diagnostic::error(
                            "E0313",
                            format!("`{name}` is already declared"),
                            name_span,
                        )
                        .with_help("write the type only the first time"),
                    );
                } else if existing != vt {
                    self.errors.push(
                        Diagnostic::error(
                            "E0202",
                            format!(
                                "cannot assign {} to `{name}`, which holds {}",
                                vt.article(),
                                existing.article()
                            ),
                            value.span,
                        )
                        .with_help("a variable keeps the type of its first value"),
                    );
                }
            }
            None => {
                if let Some(a) = ann {
                    if a != vt {
                        self.errors.push(
                            Diagnostic::error(
                                "E0202",
                                format!(
                                    "`{name}` is declared {} but the value is {}",
                                    a.article(),
                                    vt.article()
                                ),
                                value.span,
                            )
                            .with_help("numbers never convert automatically; use int(x) or float(x)"),
                        );
                        return;
                    }
                }
                self.scopes.declare(name, vt, name_span, false);
            }
        }
    }
}

use kode_diagnostics::{Diagnostic, Span};
use kode_syntax::ast::*;

use super::Checker;
use crate::suggest::closest;
use crate::types::Type;

pub(crate) fn diverges(s: &Stmt) -> bool {
    matches!(
        s,
        Stmt::Return { .. } | Stmt::Throw { .. } | Stmt::Break(_) | Stmt::Continue(_)
    )
}

/// True when every path through these statements ends in `return` or `throw`.
pub(crate) fn always_returns(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Return { .. } | Stmt::Throw { .. } => true,
        Stmt::If {
            then,
            else_: Some(else_),
            ..
        } => always_returns(then) && always_returns(else_),
        Stmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|a| always_returns(&a.body))
        }
        _ => false,
    })
}

impl Checker {
    pub(crate) fn block(&mut self, stmts: &[Stmt]) {
        self.block_with(stmts, None, None);
    }

    /// Like `block`, but `narrow` holds inside it (an optional local that the
    /// surrounding condition proved is not `none`).
    fn block_with(
        &mut self,
        stmts: &[Stmt],
        narrow: Option<(String, Type)>,
        variant: Option<(String, String, usize)>,
    ) {
        self.scopes.push();
        if let Some((name, ty)) = narrow {
            self.narrow(&name, ty);
        }
        if let Some((name, en, vi)) = variant {
            self.narrow_variant(&name, &en, vi);
        }
        for s in stmts {
            self.stmt(s);
        }
        self.pop_scope();
    }

    /// When `cond` is `x != none` (or `x == none`), the local `x` has its inner
    /// type in the branch where that is known to hold.
    fn narrow_cond(&mut self, cond: &Expr, when_true: bool) -> Option<(String, Type)> {
        let ExprKind::Binary { op, lhs, rhs } = &cond.kind else {
            return None;
        };
        let narrows_when_true = match op {
            BinOp::Ne => true,
            BinOp::Eq => false,
            _ => return None,
        };
        if narrows_when_true != when_true {
            return None;
        }
        // `s is Shape.Circle` narrows to the variant in the true branch only.
        let name = match (&lhs.kind, &rhs.kind) {
            (ExprKind::Ident(n), ExprKind::None) | (ExprKind::None, ExprKind::Ident(n)) => {
                n.clone()
            }
            _ => return None,
        };
        let declared = self.scopes.peek_var(&name)?;
        match declared {
            Type::Opt(inner) => Some((name, *inner)),
            _ => None,
        }
    }

    /// `if s is Shape.Circle`: which local is narrowed, and to which variant.
    fn variant_cond(&mut self, cond: &Expr, when_true: bool) -> Option<(String, String, usize)> {
        if !when_true {
            return None;
        }
        let ExprKind::Is {
            obj,
            enum_name,
            variant,
            ..
        } = &cond.kind
        else {
            return None;
        };
        let ExprKind::Ident(n) = &obj.kind else {
            return None;
        };
        let ei = *self.enum_index.get(enum_name)?;
        let vi = self.enums[ei].variant_index(variant)?;
        Some((n.clone(), enum_name.clone(), vi))
    }

    fn condition(&mut self, cond: &Expr, what: &str) {
        if let Some(t) = self.expr(cond) {
            if t != Type::Bool {
                self.errors.push(
                    Diagnostic::error(
                        "E0209",
                        format!(
                            "the condition of {what} must be a bool, found {}",
                            t.article()
                        ),
                        cond.span,
                    )
                    .with_help("compare it, for example `x != 0` or `x != none`"),
                );
            }
        }
    }

    pub(crate) fn field_of_current_class(&self, name: &str) -> Option<Type> {
        let ci = self.cur_class?;
        self.classes[ci]
            .fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.ty.clone())
    }

    pub(crate) fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.expr(e);
            }
            Stmt::Assign {
                target, ty, value, ..
            } => self.assign(target, ty.as_ref(), value),
            Stmt::If { cond, then, else_ } => {
                self.condition(cond, "`if`");
                let yes = self.narrow_cond(cond, true);
                let no = self.narrow_cond(cond, false);
                let yes_v = self.variant_cond(cond, true);
                self.block_with(then, yes, yes_v);
                if let Some(e) = else_ {
                    self.block_with(e, no.clone(), None);
                }
                // `if x == none { return }`: the code after the branch only runs
                // when the condition was false, so `x` is known there.
                if else_.is_none() && always_returns(then) {
                    if let Some((name, ty)) = no {
                        self.narrow(&name, ty);
                    }
                }
            }
            Stmt::While { cond, body } => {
                self.condition(cond, "`while`");
                let yes = self.narrow_cond(cond, true);
                self.loop_depth += 1;
                // A loop body may run again after its condition changed, so no
                // variant narrowing here.
                self.block_with(body, yes, None);
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
                self.pop_scope();
            }
            Stmt::ForIn { vars, iter, body } => self.for_in(vars, iter, body),
            Stmt::Match {
                scrutinee,
                arms,
                span,
            } => self.match_stmt(scrutinee, arms, *span),
            Stmt::Throw { value, span } => {
                let t = self.expr(value);
                if let Some(t) = t {
                    if t != Type::Str && t != Type::Class("Error".into()) {
                        self.errors.push(
                            Diagnostic::error(
                                "E0226",
                                format!("`throw` needs a string or an `Error`, found {}", t.article()),
                                value.span,
                            )
                            .with_help("write `throw \"message\"` or `throw Error(message: \"...\", code: 404)`"),
                        );
                    }
                }
                let _ = span;
                if let Some(i) = self.cur_fn {
                    self.direct_throw[i] = true;
                }
            }
            Stmt::Try {
                body,
                var,
                handler,
                finally,
            } => {
                self.block(body);
                if let (Some((name, name_span)), Some(handler)) = (var, handler) {
                    self.scopes.push();
                    self.scopes
                        .declare(name, Type::Class("Error".into()), *name_span, true);
                    for s in handler {
                        self.stmt(s);
                    }
                    self.pop_scope();
                }
                if let Some(f) = finally {
                    self.block(f);
                }
            }
            Stmt::Break(span) | Stmt::Continue(span) => {
                if self.loop_depth == 0 {
                    let word = if matches!(stmt, Stmt::Break(_)) {
                        "break"
                    } else {
                        "continue"
                    };
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
                        Diagnostic::error(
                            "E0311",
                            "`return` can only be used inside a function",
                            *span,
                        )
                        .with_help("top-level code simply runs to the end"),
                    );
                    return;
                }
                let ret = self.ret_type.clone();
                match value {
                    Some(v) => {
                        let t = self.expr(v);
                        if let Some(t) = t {
                            if ret == Type::Unit {
                                self.errors.push(
                                    Diagnostic::error(
                                        "E0310",
                                        "this function returns nothing, but `return` has a value",
                                        v.span,
                                    )
                                    .with_help(format!(
                                        "declare the type, for example `-> {}`",
                                        t.name()
                                    )),
                                );
                            } else {
                                self.coerce(v, Some(t), &ret, "the returned value", "E0310");
                            }
                        }
                    }
                    None => {
                        if ret != Type::Unit {
                            self.errors.push(Diagnostic::error(
                                "E0310",
                                format!("expected to return {}", ret.article()),
                                *span,
                            ));
                        }
                    }
                }
            }
        }
    }

    fn assign(&mut self, target: &Target, ty: Option<&TypeAnn>, value: &Expr) {
        let vt = self.expr(value);
        match target {
            Target::Var(name, name_span) => self.assign_var(name, *name_span, ty, value, vt),
            Target::Index { obj, index } => {
                let ot = self.expr(obj);
                let it = self.expr(index);
                let (Some(ot), Some(it)) = (ot, it) else {
                    return;
                };
                match ot {
                    Type::List(el) => {
                        if it != Type::Int {
                            self.errors.push(Diagnostic::error(
                                "E0212",
                                format!("a list index must be an int, found {}", it.article()),
                                index.span,
                            ));
                        }
                        self.coerce(value, vt, &el, "the item", "E0202");
                    }
                    Type::Map(el) => {
                        if it != Type::Str {
                            self.errors.push(Diagnostic::error(
                                "E0212",
                                format!("a map key must be a string, found {}", it.article()),
                                index.span,
                            ));
                        }
                        self.coerce(value, vt, &el, "the value", "E0202");
                    }
                    Type::Class(name) if name == "Json" => self.errors.push(
                        Diagnostic::error("E0211", "a Json value cannot be changed in place", obj.span)
                            .with_help("build a new value with `json_map`, `json_list` and the other `json_*` functions"),
                    ),
                    other => self.errors.push(Diagnostic::error(
                        "E0211",
                        format!("{} cannot be indexed", other.article()),
                        obj.span,
                    )),
                }
            }
            Target::Field {
                obj,
                name,
                name_span,
            } => {
                let Some(ot) = self.expr(obj) else { return };
                let Type::Class(cn) = &ot else {
                    let code = if matches!(ot, Type::Opt(_)) {
                        "E0220"
                    } else {
                        "E0218"
                    };
                    self.errors.push(Diagnostic::error(
                        code,
                        format!("cannot assign a field of {}", ot.article()),
                        obj.span,
                    ));
                    return;
                };
                let ci = self.class_index[cn];
                match self.classes[ci]
                    .fields
                    .iter()
                    .find(|f| f.name == *name)
                    .map(|f| f.ty.clone())
                {
                    Some(ft) => {
                        self.coerce(value, vt, &ft, "the value", "E0202");
                    }
                    None => self.unknown_field(cn, name, *name_span),
                }
            }
        }
    }

    pub(crate) fn unknown_field(&mut self, class: &str, name: &str, span: Span) {
        let ci = self.class_index[class];
        let names: Vec<String> = self.classes[ci]
            .fields
            .iter()
            .map(|f| f.name.clone())
            .chain(self.classes[ci].methods.keys().cloned())
            .collect();
        let mut d = Diagnostic::error(
            "E0217",
            format!("`{class}` has no field or method `{name}`"),
            span,
        );
        if let Some(s) = closest(name, names.iter().map(|s| s.as_str())) {
            d = d.with_help(format!("did you mean `{s}`?"));
        }
        self.errors.push(d);
    }

    fn assign_var(
        &mut self,
        name: &str,
        name_span: Span,
        ty: Option<&TypeAnn>,
        value: &Expr,
        vt: Option<Type>,
    ) {
        // A new value could be `none`, so a narrowed type stops applying.
        self.narrowed.retain(|(_, n, _)| n != name);
        self.narrowed_variants.retain(|(_, n, _, _)| n != name);
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
        let existing = self
            .scopes
            .peek_var(name)
            .or_else(|| self.field_of_current_class(name));
        if let Some(existing) = existing {
            if ty.is_some() {
                self.errors.push(
                    Diagnostic::error("E0313", format!("`{name}` is already declared"), name_span)
                        .with_help("write the type only the first time"),
                );
            } else {
                self.coerce(
                    value,
                    Some(vt),
                    &existing,
                    &format!("the value for `{name}`"),
                    "E0202",
                );
            }
            return;
        }
        let declared = match ann {
            Some(a) => {
                if !self.coerce(
                    value,
                    Some(vt),
                    &a,
                    &format!("the value for `{name}`"),
                    "E0202",
                ) {
                    return;
                }
                a
            }
            None => {
                if vt == Type::NoneT {
                    self.errors.push(
                        Diagnostic::error("E0219", "`none` needs a declared type", value.span)
                            .with_help(format!("write it like `{name}: User? = none`")),
                    );
                    return;
                }
                if vt.is_empty_collection() {
                    self.errors.push(
                        Diagnostic::error(
                            "E0222",
                            "an empty list or map needs a declared type",
                            value.span,
                        )
                        .with_help(format!("write it like `{name}: [int] = []`")),
                    );
                    return;
                }
                vt
            }
        };
        self.scopes.declare(name, declared, name_span, false);
    }

    fn for_in(&mut self, vars: &[(String, Span)], iter: &Expr, body: &[Stmt]) {
        let it = self.expr(iter);
        let tys: Vec<Type> = match &it {
            Some(Type::List(e)) => {
                if vars.len() == 1 {
                    vec![(**e).clone()]
                } else {
                    vec![Type::Int, (**e).clone()]
                }
            }
            Some(Type::Map(v)) => {
                if vars.len() == 1 {
                    vec![Type::Str]
                } else {
                    vec![Type::Str, (**v).clone()]
                }
            }
            Some(t) => {
                self.errors.push(
                    Diagnostic::error(
                        "E0214",
                        format!("cannot loop over {}", t.article()),
                        iter.span,
                    )
                    .with_help("loop over a range `0..10`, a list or a map"),
                );
                vec![Type::Int; vars.len()]
            }
            None => vec![Type::Int; vars.len()],
        };
        self.scopes.push();
        for ((name, span), ty) in vars.iter().zip(tys) {
            self.scopes.declare(name, ty, *span, true);
        }
        self.loop_depth += 1;
        for s in body {
            self.stmt(s);
        }
        self.loop_depth -= 1;
        self.pop_scope();
    }

    /// `Shape.Circle(r)`: a pattern that binds the variant's fields, in
    /// declaration order (ADR 0039 §4.3). Returns true when the pattern has
    /// that shape, with a scope pushed and the names declared in it.
    fn pattern_bindings(&mut self, st: &Option<Type>, p: &Expr) -> Option<String> {
        let Some(Type::Enum(en)) = st else {
            return None;
        };
        let ExprKind::MethodCall {
            obj, name, args, ..
        } = &p.kind
        else {
            return None;
        };
        let ExprKind::Ident(vn) = &obj.kind else {
            return None;
        };
        if vn != en {
            return None;
        }
        let &ei = self.enum_index.get(en)?;
        let info = self.enums[ei].clone();
        let vi = info.variant_index(name)?;
        let fields = info.fields[vi].clone();
        if fields.is_empty() {
            return None;
        }
        self.scopes.push();
        if args.len() != fields.len() {
            let list = fields
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            self.errors.push(
                Diagnostic::error(
                    "E0207",
                    format!(
                        "`{en}.{name}` has {} field{}, so the pattern binds {}",
                        fields.len(),
                        if fields.len() == 1 { "" } else { "s" },
                        fields.len()
                    ),
                    p.span,
                )
                .with_help(format!("write `{en}.{name}({list})`")),
            );
            return Some(name.clone());
        }
        let mut seen: Vec<String> = Vec::new();
        for (a, (_, fty)) in args.iter().zip(fields.iter()) {
            let ExprKind::Ident(b) = &a.value.kind else {
                self.errors.push(
                    Diagnostic::error("E0201", "a pattern binds a name", a.value.span).with_help(
                        "write the field name on its own, for example `Shape.Circle(r)`",
                    ),
                );
                continue;
            };
            if seen.contains(b) {
                self.errors.push(Diagnostic::error(
                    "E0306",
                    format!("the pattern binds `{b}` twice"),
                    a.value.span,
                ));
                continue;
            }
            seen.push(b.clone());
            self.scopes.declare(b, fty.clone(), a.value.span, false);
        }
        Some(name.clone())
    }

    /// The variant a pattern names, when the pattern is `Enum.Variant` and the
    /// scrutinee is that enum.
    fn pattern_variant(&self, st: &Option<Type>, p: &Expr) -> Option<String> {
        let Some(Type::Enum(en)) = st else {
            return None;
        };
        let ExprKind::Member { obj, name, .. } = &p.kind else {
            return None;
        };
        let ExprKind::Ident(vn) = &obj.kind else {
            return None;
        };
        if vn != en {
            return None;
        }
        let info = &self.enums[self.enum_index[en]];
        info.variants.contains(name).then(|| name.clone())
    }

    fn match_stmt(&mut self, scrutinee: &Expr, arms: &[Arm], span: Span) {
        let st = self.expr(scrutinee);
        let mut has_wild = false;
        let mut variants: Vec<String> = Vec::new();
        let (mut saw_true, mut saw_false) = (false, false);
        if let Some(t) = &st {
            if !matches!(t, Type::Int | Type::Str | Type::Bool | Type::Enum(_)) {
                self.errors.push(
                    Diagnostic::error(
                        "E0223",
                        format!("cannot `match` on {}", t.article()),
                        scrutinee.span,
                    )
                    .with_help("match works on ints, strings, bools and enums"),
                );
            }
        }
        for arm in arms {
            match &arm.pattern {
                Pattern::Wildcard => has_wild = true,
                Pattern::Value(p) => {
                    // `Shape.Circle(r)`: the fields are bound for the arm.
                    if let Some(name) = self.pattern_bindings(&st, p) {
                        variants.push(name);
                        self.block(&arm.body);
                        self.pop_scope();
                        continue;
                    }
                    // `Shape.Circle` in a pattern names a tag; the same
                    // expression in value position is a constructor, so it is
                    // resolved here rather than through `self.expr` (ADR 0039).
                    if let Some(name) = self.pattern_variant(&st, p) {
                        variants.push(name);
                        self.block(&arm.body);
                        continue;
                    }
                    let pt = self.expr(p);
                    if let (Some(s), Some(pt)) = (&st, &pt) {
                        if s != pt {
                            self.errors.push(Diagnostic::error(
                                "E0223",
                                format!(
                                    "this pattern is {} but the value is {}",
                                    pt.article(),
                                    s.article()
                                ),
                                arm.pattern_span,
                            ));
                        } else {
                            match &p.kind {
                                ExprKind::Member { name, .. } => variants.push(name.clone()),
                                ExprKind::Bool(true) => saw_true = true,
                                ExprKind::Bool(false) => saw_false = true,
                                _ => {}
                            }
                        }
                    }
                }
            }
            self.block(&arm.body);
        }
        if has_wild {
            return;
        }
        if let Some(t) = &st {
            let missing: Option<String> = match t {
                Type::Enum(n) => {
                    let all = &self.enums[self.enum_index[n]].variants;
                    let m: Vec<&String> = all.iter().filter(|v| !variants.contains(v)).collect();
                    (!m.is_empty()).then(|| {
                        format!(
                            "these cases are missing: {}",
                            m.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                        )
                    })
                }
                Type::Bool => (!(saw_true && saw_false))
                    .then(|| "both true and false must be covered".to_string()),
                Type::Int | Type::Str => Some("add a `_ =>` arm for all other values".to_string()),
                _ => None,
            };
            if let Some(help) = missing {
                self.errors.push(
                    Diagnostic::error("E0224", "this `match` does not cover every case", span)
                        .with_help(help),
                );
            }
        }
    }
}

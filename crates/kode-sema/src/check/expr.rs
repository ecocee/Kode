use kode_diagnostics::{Diagnostic, Span};
use kode_syntax::ast::*;

use super::stmt::diverges;
use super::Checker;
use crate::builtins::{self, Builtin, BUILTIN_NAMES};
use crate::suggest::closest;
use crate::std_fns::{self, StdFn};
use crate::types::{assignable, Type};

impl Checker {
    /// If `e` is an empty `[]`, `{}` or `none`, remember the type it was given.
    pub(crate) fn retype(&mut self, e: &Expr, want: &Type) {
        let inner = match want {
            Type::Opt(i) => i.as_ref().clone(),
            w => w.clone(),
        };
        match &e.kind {
            ExprKind::List(v) if v.is_empty() => self.expr_types[e.id as usize] = inner,
            ExprKind::Map(v) if v.is_empty() => self.expr_types[e.id as usize] = inner,
            ExprKind::None => self.expr_types[e.id as usize] = want.clone(),
            _ => {}
        }
    }

    /// Check that a value of type `got` can go where `want` is expected.
    pub(crate) fn coerce(&mut self, e: &Expr, got: Option<Type>, want: &Type, what: &str, code: &'static str) -> bool {
        let Some(got) = got else { return false };
        if assignable(&got, want) {
            self.retype(e, want);
            return true;
        }
        let mut d = Diagnostic::error(
            code,
            format!("{what} should be {}, found {}", want.article(), got.article()),
            e.span,
        );
        if matches!((&got, want), (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
            d = d.with_help("numbers never convert automatically; use int(x) or float(x)");
        } else if let Type::Opt(inner) = &got {
            if assignable(inner, want) {
                d = d.with_help("this may be none: use `x ?? default` or `x!`");
            }
        }
        self.errors.push(d);
        false
    }

    /// Check an expression and return its type. `None` means an error was
    /// already reported (so callers do not pile on more errors).
    pub(crate) fn expr(&mut self, e: &Expr) -> Option<Type> {
        let ty = match &e.kind {
            ExprKind::Int(_) => Some(Type::Int),
            ExprKind::Float(_) => Some(Type::Float),
            ExprKind::Bool(_) => Some(Type::Bool),
            ExprKind::Str(_) => Some(Type::Str),
            ExprKind::None => Some(Type::NoneT),
            ExprKind::Interp(parts) => {
                let mut ok = true;
                for p in parts {
                    if let InterpPart::Expr(x) = p {
                        match self.expr(x) {
                            Some(Type::Int | Type::Float | Type::Bool | Type::Str) => {}
                            Some(Type::Unit) => {
                                self.errors.push(Diagnostic::error(
                                    "E0205",
                                    "this call returns nothing, so it cannot go inside a string",
                                    x.span,
                                ));
                                ok = false;
                            }
                            Some(other) => {
                                self.errors.push(
                                    Diagnostic::error(
                                        "E0203",
                                        format!("cannot put {} inside a string yet", other.article()),
                                        x.span,
                                    )
                                    .with_help("use its parts, for example `${user.name}`"),
                                );
                                ok = false;
                            }
                            None => ok = false,
                        }
                    }
                }
                ok.then_some(Type::Str)
            }
            ExprKind::Ident(name) => self.ident(name, e.span),
            ExprKind::List(items) => self.list_lit(items),
            ExprKind::Map(pairs) => self.map_lit(pairs),
            ExprKind::Index { obj, index } => self.index(obj, index),
            ExprKind::Member {
                obj,
                name,
                name_span,
                safe,
            } => self.member(obj, name, *name_span, *safe),
            ExprKind::MethodCall { obj, name, name_span, args } => {
                self.method_call(e.id, obj, name, *name_span, args, e.span)
            }
            ExprKind::Unary { op, expr } => self.unary(*op, expr, e.span),
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e.span),
            ExprKind::Coalesce { lhs, rhs } => self.coalesce(lhs, rhs, e.span),
            ExprKind::Unwrap(inner) => {
                let t = self.expr(inner)?;
                match t {
                    Type::Opt(i) => Some(*i),
                    other => {
                        self.errors.push(
                            Diagnostic::error(
                                "E0220",
                                format!("`!` needs an optional, found {}", other.article()),
                                e.span,
                            )
                            .with_help("only values of a type like `User?` can be none"),
                        );
                        None
                    }
                }
            }
            ExprKind::Call {
                callee,
                callee_span,
                args,
            } => self.call(e.id, callee, *callee_span, args, e.span),
            ExprKind::Spawn {
                callee,
                callee_span,
                args,
            } => self.spawn(e.id, callee, *callee_span, args, e.span),
            ExprKind::Await(inner) => {
                let t = self.expr(inner)?;
                match t {
                    Type::Task(r) => {
                        if let Some(i) = self.cur_fn {
                            self.direct_throw[i] = true;
                        }
                        Some(*r)
                    }
                    other => {
                        self.errors.push(
                            Diagnostic::error("E0231", format!("`await` needs a task, found {}", other.article()), inner.span)
                                .with_help("start one with `spawn work(...)`"),
                        );
                        None
                    }
                }
            }
            ExprKind::Catch {
                expr,
                var,
                var_span,
                block,
            } => self.catch_expr(expr, var, *var_span, block),
        };
        if let Some(t) = &ty {
            self.expr_types[e.id as usize] = t.clone();
        }
        ty
    }

    fn ident(&mut self, name: &str, span: Span) -> Option<Type> {
        if let Some(t) = self.scopes.use_var(name) {
            return Some(t);
        }
        if let Some((t, _)) = self.consts.get(name) {
            return Some(t.clone());
        }
        if let Some(t) = self.field_of_current_class(name) {
            return Some(t);
        }
        let mut d = Diagnostic::error("E0201", format!("unknown variable `{name}`"), span);
        let mut names: Vec<String> = self.scopes.names().into_iter().map(String::from).collect();
        names.extend(self.consts.keys().cloned());
        if let Some(ci) = self.cur_class {
            names.extend(self.classes[ci].fields.iter().map(|f| f.name.clone()));
        }
        d = if self.enum_index.contains_key(name) {
            d.with_help(format!("enum values are written like `{name}.VariantName`"))
        } else {
            match closest(name, names.iter().map(|s| s.as_str())) {
                Some(s) => d.with_help(format!("did you mean `{s}`?")),
                None => d.with_help(format!("create it first with `{name} = ...` on an earlier line")),
            }
        };
        self.errors.push(d);
        None
    }

    fn list_lit(&mut self, items: &[Expr]) -> Option<Type> {
        if items.is_empty() {
            return Some(Type::List(Box::new(Type::NoneT)));
        }
        let first = self.expr(&items[0]);
        let mut ok = first.is_some();
        for x in &items[1..] {
            let t = self.expr(x);
            if let (Some(f), Some(t)) = (&first, t) {
                if !assignable(&t, f) {
                    self.errors.push(Diagnostic::error(
                        "E0213",
                        format!("list items must all have the same type: expected {}, found {}", f.article(), t.article()),
                        x.span,
                    ));
                    ok = false;
                }
            }
        }
        let f = first?;
        if matches!(f, Type::NoneT | Type::Unit) {
            self.errors.push(Diagnostic::error("E0213", "a list cannot hold this kind of value", items[0].span));
            return None;
        }
        ok.then(|| Type::List(Box::new(f)))
    }

    fn map_lit(&mut self, pairs: &[(Expr, Expr)]) -> Option<Type> {
        if pairs.is_empty() {
            return Some(Type::Map(Box::new(Type::NoneT)));
        }
        let mut val: Option<Type> = None;
        let mut ok = true;
        for (k, v) in pairs {
            match self.expr(k) {
                Some(Type::Str) => {}
                Some(other) => {
                    self.errors.push(Diagnostic::error(
                        "E0212",
                        format!("a map key must be a string, found {}", other.article()),
                        k.span,
                    ));
                    ok = false;
                }
                None => ok = false,
            }
            let t = self.expr(v);
            match (&val, t) {
                (None, Some(t)) => val = Some(t),
                (Some(f), Some(t)) if !assignable(&t, f) => {
                    self.errors.push(Diagnostic::error(
                        "E0213",
                        format!("map values must all have the same type: expected {}, found {}", f.article(), t.article()),
                        v.span,
                    ));
                    ok = false;
                }
                (_, None) => ok = false,
                _ => {}
            }
        }
        let v = val?;
        if matches!(v, Type::NoneT | Type::Unit) {
            return None;
        }
        ok.then(|| Type::Map(Box::new(v)))
    }

    fn index(&mut self, obj: &Expr, index: &Expr) -> Option<Type> {
        let ot = self.expr(obj);
        let it = self.expr(index);
        let (ot, it) = (ot?, it?);
        match ot {
            Type::List(el) => {
                if it != Type::Int {
                    self.errors.push(Diagnostic::error(
                        "E0212",
                        format!("a list index must be an int, found {}", it.article()),
                        index.span,
                    ));
                    return None;
                }
                Some(*el)
            }
            Type::Map(v) => {
                if it != Type::Str {
                    self.errors.push(Diagnostic::error(
                        "E0212",
                        format!("a map key must be a string, found {}", it.article()),
                        index.span,
                    ));
                    return None;
                }
                Some(*v)
            }
            Type::Class(name) if name == "Json" => match it {
                Type::Str | Type::Int => Some(Type::Class("Json".into())),
                other => {
                    self.errors.push(Diagnostic::error(
                        "E0212",
                        format!(
                            "a Json value needs a string key or an int index, found {}",
                            other.article()
                        ),
                        index.span,
                    ));
                    None
                }
            },
            Type::Opt(_) => {
                self.errors.push(
                    Diagnostic::error("E0220", "this may be none, so it cannot be indexed", obj.span)
                        .with_help("use `x!` or `x ?? default` first"),
                );
                None
            }
            other => {
                self.errors.push(Diagnostic::error(
                    "E0211",
                    format!("{} cannot be indexed", other.article()),
                    obj.span,
                ));
                None
            }
        }
    }

    fn member(&mut self, obj: &Expr, name: &str, name_span: Span, safe: bool) -> Option<Type> {
        // `Color.Red`
        if let ExprKind::Ident(n) = &obj.kind {
            if self.scopes.peek_var(n).is_none()
                && !self.consts.contains_key(n)
                && self.field_of_current_class(n).is_none()
            {
                if let Some(&ei) = self.enum_index.get(n) {
                    if self.enums[ei].variants.iter().any(|v| v == name) {
                        return Some(Type::Enum(n.clone()));
                    }
                    let vs = self.enums[ei].variants.clone();
                    let mut d = Diagnostic::error("E0225", format!("`{n}` has no variant `{name}`"), name_span);
                    if let Some(s) = closest(name, vs.iter().map(|s| s.as_str())) {
                        d = d.with_help(format!("did you mean `{s}`?"));
                    }
                    self.errors.push(d);
                    return None;
                }
            }
        }
        let ot = self.expr(obj)?;
        let (class_ty, is_opt) = match &ot {
            Type::Opt(inner) => ((**inner).clone(), true),
            t => (t.clone(), false),
        };
        if is_opt && !safe {
            self.errors.push(
                Diagnostic::error("E0220", "this may be none, so `.` is not allowed", obj.span)
                    .with_help("use `?.`, `x ?? default` or `x!`"),
            );
            return None;
        }
        if !is_opt && safe {
            self.errors.push(Diagnostic::error(
                "E0221",
                format!("`?.` needs an optional, found {}", ot.article()),
                obj.span,
            ));
            return None;
        }
        let Type::Class(cn) = &class_ty else {
            self.errors.push(Diagnostic::error(
                "E0218",
                format!("{} has no fields", class_ty.article()),
                obj.span,
            ));
            return None;
        };
        let ci = self.class_index[cn];
        let Some(ft) = self.classes[ci].fields.iter().find(|f| f.name == name).map(|f| f.ty.clone()) else {
            self.unknown_field(cn, name, name_span);
            return None;
        };
        if safe {
            if matches!(ft, Type::Opt(_)) {
                return Some(ft);
            }
            if !ft.is_ref() {
                self.errors.push(
                    Diagnostic::error("E0219", format!("`?.` on {} is not supported yet", ft.article()), name_span)
                        .with_help("use `(x!).field`, or check `x != none` first"),
                );
                return None;
            }
            return Some(Type::Opt(Box::new(ft)));
        }
        Some(ft)
    }

    fn unary(&mut self, op: UnOp, inner: &Expr, span: Span) -> Option<Type> {
        let t = self.expr(inner)?;
        match (op, &t) {
            (UnOp::Neg, Type::Int) | (UnOp::Neg, Type::Float) => Some(t),
            (UnOp::Not, Type::Bool) => Some(Type::Bool),
            (UnOp::Neg, _) => {
                self.errors.push(Diagnostic::error("E0203", format!("cannot negate {}", t.article()), span));
                None
            }
            (UnOp::Not, _) => {
                self.errors.push(
                    Diagnostic::error("E0203", format!("`not` needs a bool, found {}", t.article()), span)
                        .with_help("compare it first, for example `not (x == 0)`"),
                );
                None
            }
        }
    }

    fn bad_operands(&mut self, op: BinOp, l: &Type, r: &Type, span: Span, help: &str) -> Option<Type> {
        self.errors.push(
            Diagnostic::error(
                "E0203",
                format!("cannot apply `{}` to {} and {}", op.symbol(), l.article(), r.article()),
                span,
            )
            .with_help(help.to_string()),
        );
        None
    }

    fn binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Option<Type> {
        let l = self.expr(lhs);
        let r = self.expr(rhs);
        let (l, r) = (l?, r?);
        let mixed = matches!((&l, &r), (Type::Int, Type::Float) | (Type::Float, Type::Int));
        if mixed {
            self.errors.push(
                Diagnostic::error(
                    "E0210",
                    format!("cannot mix an int and a float with `{}`", op.symbol()),
                    span,
                )
                .with_help("convert one side: float(x) or int(x)"),
            );
            return None;
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => match (&l, &r) {
                (Type::Int, Type::Int) => {
                    if matches!(op, BinOp::Div | BinOp::Rem) && matches!(rhs.kind, ExprKind::Int(0)) {
                        self.errors.push(Diagnostic::error("E0204", "division by zero", rhs.span));
                        return None;
                    }
                    Some(Type::Int)
                }
                (Type::Float, Type::Float) if op != BinOp::Rem => Some(Type::Float),
                (Type::Str, Type::Str) if op == BinOp::Add => Some(Type::Str),
                _ => self.bad_operands(op, &l, &r, span, "arithmetic works on two ints or two floats; `+` also joins two strings"),
            },
            BinOp::Eq | BinOp::Ne => {
                let ok = (l == r && l != Type::Unit)
                    || (l == Type::NoneT && matches!(r, Type::Opt(_)))
                    || (r == Type::NoneT && matches!(l, Type::Opt(_)));
                if ok {
                    Some(Type::Bool)
                } else {
                    self.bad_operands(op, &l, &r, span, "both sides must have the same type")
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match (&l, &r) {
                (Type::Int, Type::Int) | (Type::Float, Type::Float) => Some(Type::Bool),
                _ => self.bad_operands(op, &l, &r, span, "only ints and floats can be ordered so far"),
            },
            BinOp::And | BinOp::Or => {
                if l == Type::Bool && r == Type::Bool {
                    Some(Type::Bool)
                } else {
                    self.bad_operands(op, &l, &r, span, "`and` and `or` need two bools")
                }
            }
            BinOp::In => {
                if l == Type::Str && matches!(r, Type::Map(_)) {
                    Some(Type::Bool)
                } else {
                    self.bad_operands(op, &l, &r, span, "`in` checks whether a string key is in a map")
                }
            }
        }
    }

    fn coalesce(&mut self, lhs: &Expr, rhs: &Expr, span: Span) -> Option<Type> {
        let l = self.expr(lhs);
        let r = self.expr(rhs);
        let (l, r) = (l?, r?);
        let Type::Opt(inner) = &l else {
            self.errors.push(
                Diagnostic::error("E0221", format!("`??` needs an optional on the left, found {}", l.article()), lhs.span)
                    .with_help("only values of a type like `User?` can be none"),
            );
            return None;
        };
        if assignable(&r, inner) {
            self.retype(rhs, inner);
            Some((**inner).clone())
        } else if assignable(&r, &l) {
            self.retype(rhs, &l);
            Some(l.clone())
        } else {
            self.errors.push(Diagnostic::error(
                "E0221",
                format!("the default should be {}, found {}", inner.article(), r.article()),
                span,
            ));
            None
        }
    }

    fn catch_expr(&mut self, inner: &Expr, var: &str, var_span: Span, block: &[Stmt]) -> Option<Type> {
        let t = self.expr(inner);
        self.scopes.push();
        self.scopes.declare(var, Type::Class("Error".into()), var_span, true);
        let n = block.len();
        for (i, s) in block.iter().enumerate() {
            if i + 1 == n {
                if let Stmt::Expr(x) = s {
                    let xt = self.expr(x);
                    if let Some(t) = &t {
                        if *t != Type::Unit {
                            self.coerce(x, xt, t, "the value of the catch block", "E0227");
                        }
                    }
                    continue;
                }
            }
            self.stmt(s);
        }
        if let Some(t) = &t {
            if *t != Type::Unit {
                let ok = block.last().map(|s| matches!(s, Stmt::Expr(_)) || diverges(s)).unwrap_or(false);
                if !ok {
                    self.errors.push(
                        Diagnostic::error("E0227", "the catch block must end with a value, `return`, `break`, `continue` or `throw`", inner.span)
                            .with_help(format!("the expression has type {}", t.article())),
                    );
                }
            }
        }
        self.pop_scope();
        t
    }

    /// A call to a runtime library function, checked from its table row.
    fn std_call(&mut self, f: &StdFn, recv: Option<&Type>, args: &[Arg], tys: &[Option<Type>], span: Span) -> Option<Type> {
        let elem = match recv {
            Some(Type::List(e)) => Some(&**e),
            _ => None,
        };
        if let Some(a) = args.iter().find(|a| a.name.is_some()) {
            self.errors.push(Diagnostic::error(
                "E0215",
                format!("`{}` does not take named arguments", f.name),
                a.value.span,
            ));
        }
        if args.len() != f.params.len() {
            self.errors.push(Diagnostic::error(
                "E0207",
                format!(
                    "`{}` takes {} argument{}, found {}",
                    f.name,
                    f.params.len(),
                    if f.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
                span,
            ));
        } else {
            for (i, p) in f.params.iter().enumerate() {
                let want = std_fns::to_type(*p, elem);
                self.coerce(&args[i].value, tys[i].clone(), &want, &format!("argument {} of `{}`", i + 1, f.name), "E0208");
            }
        }
        if f.throws {
            if let Some(i) = self.cur_fn {
                self.direct_throw[i] = true;
            }
        }
        Some(std_fns::to_type(f.ret, elem))
    }

    /// `Response.json(value)` / `Response.text(text)`.
    fn response_static(&mut self, name: &str, name_span: Span, args: &[Arg], span: Span) -> Option<Type> {
        let f = match name {
            "json" => std_fns::find_free("response_json").expect("the table has response_json"),
            "text" => std_fns::find_free("response_text").expect("the table has response_text"),
            _ => {
                self.errors.push(
                    Diagnostic::error("E0217", format!("`Response` has no static method `{name}`"), name_span)
                        .with_help("use `Response.json(value)` or `Response.text(text)`"),
                );
                return None;
            }
        };
        let mut tys = Vec::new();
        for a in args {
            tys.push(self.expr(&a.value));
        }
        if args.len() != 1 {
            self.errors.push(Diagnostic::error(
                "E0207",
                format!("`Response.{name}` takes 1 argument, found {}", args.len()),
                span,
            ));
            return Some(Type::Class("Response".into()));
        }
        let want = std_fns::to_type(f.params[0], None);
        self.coerce(&args[0].value, tys[0].clone(), &want, "the value", "E0208");
        if f.throws {
            if let Some(i) = self.cur_fn {
                self.direct_throw[i] = true;
            }
        }
        Some(Type::Class("Response".into()))
    }

    /// `router.get(path, handler)` and friends.
    fn router_method(&mut self, name: &str, name_span: Span, args: &[Arg], span: Span) -> Option<Type> {
        if !matches!(name, "get" | "post" | "put" | "patch" | "delete") {
            self.errors.push(
                Diagnostic::error("E0217", format!("`Router` has no method `{name}`"), name_span)
                    .with_help("add routes with get, post, put, patch or delete"),
            );
            return None;
        }
        if args.len() != 2 {
            self.errors.push(Diagnostic::error(
                "E0207",
                format!("`Router.{name}` takes 2 arguments, found {}", args.len()),
                span,
            ));
            return Some(Type::Unit);
        }
        let path_ty = self.expr(&args[0].value);
        self.coerce(&args[0].value, path_ty, &Type::Str, "the route path", "E0208");
        let handler_ok = match &args[1].value.kind {
            ExprKind::Ident(h) => self.fn_index.get(h).map(|&i| {
                let s = &self.fns[i];
                s.class.is_none()
                    && s.params.len() == 1
                    && s.params[0].1 == Type::Class("Request".into())
                    && s.ret == Type::Class("Response".into())
            }),
            _ => None,
        };
        if handler_ok != Some(true) {
            self.errors.push(
                Diagnostic::error(
                    "E0233",
                    "the route handler must be the name of a function `fn name(req: Request) -> Response`",
                    args[1].value.span,
                )
                .with_help("define it first, then pass it like `routes.get(\"/users/:id\", get_user)`"),
            );
        }
        Some(Type::Unit)
    }

    /// `serve(port, handler)` or `serve(port, router)`.
    fn serve_call(&mut self, args: &[Arg], span: Span) -> Option<Type> {
        if args.len() != 2 {
            self.errors.push(Diagnostic::error("E0207", format!("`serve` takes 2 arguments, found {}", args.len()), span));
            return Some(Type::Unit);
        }
        let pt = self.expr(&args[0].value);
        self.coerce(&args[0].value, pt, &Type::Int, "the port", "E0208");
        let is_fn = matches!(&args[1].value.kind, ExprKind::Ident(n) if self.fn_index.contains_key(n));
        if is_fn {
            let ExprKind::Ident(n) = &args[1].value.kind else { unreachable!() };
            let ok = self.fn_index.get(n).map(|&i| {
                let s = &self.fns[i];
                s.params.iter().map(|p| &p.1).collect::<Vec<_>>() == vec![&Type::Str, &Type::Str, &Type::Str] && s.ret == Type::Str
            });
            if ok != Some(true) {
                self.errors.push(
                    Diagnostic::error("E0232", "the handler must be the name of a function `fn name(method: string, path: string, body: string) -> string`", args[1].value.span)
                        .with_help("it receives the HTTP method, the path and the body, and returns the response text"),
                );
            }
            return Some(Type::Unit);
        }
        let ht = self.expr(&args[1].value);
        if !matches!(ht, Some(Type::Class(ref n)) if n == "Router") {
            self.errors.push(
                Diagnostic::error("E0232", "`serve` needs a `Router` or a handler function", args[1].value.span)
                    .with_help("build one with `routes = Router()` and `routes.get(path, handler)`, or pass `fn name(method: string, path: string, body: string) -> string`"),
            );
        }
        Some(Type::Unit)
    }

    /// `spawn f(args)`
    fn spawn(&mut self, id: ExprId, callee: &str, callee_span: Span, args: &[Arg], span: Span) -> Option<Type> {
        let mut tys: Vec<Option<Type>> = Vec::new();
        for a in args {
            tys.push(self.expr(&a.value));
        }
        let Some(&idx) = self.fn_index.get(callee) else {
            self.errors.push(
                Diagnostic::error("E0230", format!("`spawn` needs one of your functions, `{callee}` is not one"), callee_span)
                    .with_help("define it first with `fn name(...) -> T { ... }`"),
            );
            return None;
        };
        let sig = self.fns[idx].clone();
        for (n, t) in &sig.params {
            if !matches!(t, Type::Int | Type::Float | Type::Bool | Type::Str) {
                self.errors.push(
                    Diagnostic::error(
                        "E0230",
                        format!("a task can only receive ints, floats, bools and strings, but `{n}` is {}", t.article()),
                        callee_span,
                    )
                    .with_help("tasks do not share objects; pass plain values and return the result"),
                );
                return None;
            }
        }
        let defaults = vec![false; sig.params.len()];
        self.check_call_args(id, &sig.params, args, &tys, &defaults, span, callee);
        Some(Type::Task(Box::new(sig.ret)))
    }

    fn note_call(&mut self, callee: usize) {
        if let Some(i) = self.cur_fn {
            self.calls[i].push(callee);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_call_args(
        &mut self,
        id: ExprId,
        params: &[(String, Type)],
        args: &[Arg],
        tys: &[Option<Type>],
        has_default: &[bool],
        span: Span,
        who: &str,
    ) {
        let mut slots: Vec<Option<usize>> = vec![None; params.len()];
        let mut seen_named = false;
        let mut positional = 0;
        let mut overflow = false;
        for (i, a) in args.iter().enumerate() {
            match &a.name {
                None => {
                    if seen_named {
                        self.errors.push(Diagnostic::error(
                            "E0215",
                            "positional arguments must come before named ones",
                            a.value.span,
                        ));
                    } else if positional >= params.len() {
                        overflow = true;
                    } else {
                        slots[positional] = Some(i);
                        positional += 1;
                    }
                }
                Some((n, nspan)) => {
                    seen_named = true;
                    match params.iter().position(|(pn, _)| pn == n) {
                        None => {
                            let mut d = Diagnostic::error("E0215", format!("`{who}` has no parameter `{n}`"), *nspan);
                            if let Some(s) = closest(n, params.iter().map(|(p, _)| p.as_str())) {
                                d = d.with_help(format!("did you mean `{s}`?"));
                            }
                            self.errors.push(d);
                        }
                        Some(pi) if slots[pi].is_some() => {
                            self.errors.push(Diagnostic::error(
                                "E0215",
                                format!("the argument `{n}` is given twice"),
                                *nspan,
                            ));
                        }
                        Some(pi) => slots[pi] = Some(i),
                    }
                }
            }
        }
        if overflow {
            self.errors.push(
                Diagnostic::error(
                    "E0207",
                    format!(
                        "`{who}` takes {} argument{}, found {}",
                        params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        args.len()
                    ),
                    span,
                )
                .with_help(format!(
                    "its parameters are ({})",
                    params.iter().map(|(n, t)| format!("{n}: {}", t.name())).collect::<Vec<_>>().join(", ")
                )),
            );
        }
        for (pi, (pn, pty)) in params.iter().enumerate() {
            match slots[pi] {
                Some(ai) => {
                    self.coerce(&args[ai].value, tys[ai].clone(), pty, &format!("argument `{pn}` of `{who}`"), "E0208");
                }
                None if !has_default[pi] && !overflow => {
                    self.errors.push(
                        Diagnostic::error("E0216", format!("missing argument `{pn}` for `{who}`"), span)
                            .with_help(format!("pass it like `{pn}: ...`")),
                    );
                }
                None => {}
            }
        }
        self.call_args.insert(id, slots);
    }

    fn method_call(&mut self, id: ExprId, obj: &Expr, name: &str, name_span: Span, args: &[Arg], span: Span) -> Option<Type> {
        // `Response.json(doc)` / `Response.text(text)` read like Express helpers.
        if let ExprKind::Ident(n) = &obj.kind {
            if n == "Response"
                && self.scopes.peek_var(n).is_none()
                && !self.consts.contains_key(n)
                && self.field_of_current_class(n).is_none()
            {
                return self.response_static(name, name_span, args, span);
            }
        }
        let ot = self.expr(obj);
        // A route handler is a function name, not an expression: do not evaluate it.
        if matches!(&ot, Some(Type::Class(n)) if n == "Router") {
            return self.router_method(name, name_span, args, span);
        }
        let mut tys = Vec::new();
        for a in args {
            tys.push(self.expr(&a.value));
        }
        let ot = ot?;
        if let Some(f) = std_fns::find_method(&ot, name) {
            return self.std_call(f, Some(&ot), args, &tys, span);
        }
        let arity = |this: &mut Self, n: usize| -> bool {
            if args.len() != n {
                this.errors.push(Diagnostic::error(
                    "E0207",
                    format!("`{name}` takes {n} argument{}, found {}", if n == 1 { "" } else { "s" }, args.len()),
                    span,
                ));
                false
            } else {
                true
            }
        };
        match &ot {
            Type::Str if name == "len" => {
                arity(self, 0);
                Some(Type::Int)
            }
            Type::List(_) if name == "len" => {
                arity(self, 0);
                Some(Type::Int)
            }
            Type::List(el) if name == "add" => {
                if arity(self, 1) {
                    let el = (**el).clone();
                    self.coerce(&args[0].value, tys[0].clone(), &el, "the item", "E0213");
                }
                Some(Type::Unit)
            }
            Type::Map(_) if name == "len" => {
                arity(self, 0);
                Some(Type::Int)
            }
            Type::Map(_) if name == "has" => {
                if arity(self, 1) {
                    self.coerce(&args[0].value, tys[0].clone(), &Type::Str, "the key", "E0212");
                }
                Some(Type::Bool)
            }
            Type::Map(_) if name == "keys" => {
                arity(self, 0);
                Some(Type::List(Box::new(Type::Str)))
            }
            Type::Class(cn) => {
                let ci = self.class_index[cn];
                let Some(&mi) = self.classes[ci].methods.get(name) else {
                    self.unknown_field(cn, name, name_span);
                    return None;
                };
                let sig = self.fns[mi].clone();
                self.note_call(mi);
                let params: Vec<(String, Type)> = sig.params[1..].to_vec();
                let no_defaults = vec![false; params.len()];
                self.check_call_args(id, &params, args, &tys, &no_defaults, span, &sig.name);
                Some(sig.ret)
            }
            Type::Opt(_) => {
                self.errors.push(
                    Diagnostic::error("E0220", "this may be none, so you cannot call a method on it", obj.span)
                        .with_help("use `x!` or `x ?? default` first"),
                );
                None
            }
            other => {
                self.errors.push(Diagnostic::error(
                    "E0217",
                    format!("{} has no method `{name}`", other.article()),
                    name_span,
                ).with_help("lists have add, len; maps have has, len, keys; strings have len"));
                None
            }
        }
    }

    fn call(&mut self, id: ExprId, callee: &str, callee_span: Span, args: &[Arg], span: Span) -> Option<Type> {
        if callee == "serve" {
            return self.serve_call(args, span);
        }
        let mut tys: Vec<Option<Type>> = Vec::new();
        for a in args {
            tys.push(self.expr(&a.value));
        }

        if callee == "Router" {
            if !args.is_empty() {
                self.errors.push(Diagnostic::error(
                    "E0207",
                    format!("`Router` takes no arguments, found {}", args.len()),
                    span,
                ));
            }
            return Some(Type::Class("Router".into()));
        }

        if let Some(b) = builtins::lookup(callee) {
            if let Some(a) = args.iter().find(|a| a.name.is_some()) {
                self.errors.push(Diagnostic::error(
                    "E0215",
                    format!("`{callee}` does not take named arguments"),
                    a.value.span,
                ));
            }
            return self.builtin_call(b, callee, args, &tys, span);
        }

        if let Some(f) = std_fns::find_free(callee) {
            return self.std_call(f, None, args, &tys, span);
        }

        if let Some(&idx) = self.fn_index.get(callee) {
            let sig = self.fns[idx].clone();
            self.note_call(idx);
            let defaults = vec![false; sig.params.len()];
            self.check_call_args(id, &sig.params, args, &tys, &defaults, span, callee);
            return Some(sig.ret);
        }

        if let Some(&ci) = self.class_index.get(callee) {
            let fields = self.classes[ci].fields.clone();
            let params: Vec<(String, Type)> = fields.iter().map(|f| (f.name.clone(), f.ty.clone())).collect();
            let defaults: Vec<bool> = fields.iter().map(|f| f.default.is_some()).collect();
            self.check_call_args(id, &params, args, &tys, &defaults, span, callee);
            return Some(Type::Class(callee.to_string()));
        }

        // A method of the current class, called without `this`.
        if let Some(ci) = self.cur_class {
            if let Some(&mi) = self.classes[ci].methods.get(callee) {
                let sig = self.fns[mi].clone();
                self.note_call(mi);
                let params: Vec<(String, Type)> = sig.params[1..].to_vec();
                let defaults = vec![false; params.len()];
                self.check_call_args(id, &params, args, &tys, &defaults, span, &sig.name);
                return Some(sig.ret);
            }
        }

        let mut d = Diagnostic::error("E0206", format!("unknown function `{callee}`"), callee_span);
        let candidates: Vec<String> = self
            .fn_index
            .keys()
            .cloned()
            .chain(self.class_index.keys().cloned())
            .chain(BUILTIN_NAMES.iter().map(|s| s.to_string()))
            .collect();
        d = match closest(callee, candidates.iter().map(|s| s.as_str())) {
            Some(s) => d.with_help(format!("did you mean `{s}`?")),
            None => d.with_help("define it first with `fn name(...) { ... }`"),
        };
        self.errors.push(d);
        None
    }

    fn builtin_call(&mut self, b: Builtin, name: &str, args: &[Arg], tys: &[Option<Type>], span: Span) -> Option<Type> {
        if b == Builtin::Print {
            for (i, t) in tys.iter().enumerate() {
                match t {
                    Some(Type::Int | Type::Float | Type::Bool | Type::Str) | None => {}
                    Some(Type::Unit) => self.errors.push(Diagnostic::error(
                        "E0205",
                        "this call returns nothing, so there is nothing to print",
                        args[i].value.span,
                    )),
                    Some(other) => self.errors.push(
                        Diagnostic::error(
                            "E0203",
                            format!("cannot print {} yet", other.article()),
                            args[i].value.span,
                        )
                        .with_help("print its parts, for example `print(user.name)` or loop over a list"),
                    ),
                }
            }
            return Some(Type::Unit);
        }
        if b == Builtin::Assert {
            if let Some(i) = self.cur_fn {
                self.direct_throw[i] = true;
            }
        }

        let (accepts, ret, expects): (Vec<Type>, Type, &str) = match b {
            Builtin::Len => (vec![Type::Str], Type::Int, "a string, a list or a map"),
            Builtin::Int => (vec![Type::Int, Type::Float, Type::Bool], Type::Int, "an int, float or bool"),
            Builtin::Float => (vec![Type::Int, Type::Float], Type::Float, "an int or float"),
            Builtin::Str => (
                vec![Type::Int, Type::Float, Type::Bool, Type::Str],
                Type::Str,
                "an int, float, bool or string",
            ),
            Builtin::Assert => (vec![Type::Bool], Type::Unit, "a bool"),
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
        if let Some(t) = &tys[0] {
            let ok = accepts.contains(t) || (b == Builtin::Len && matches!(t, Type::List(_) | Type::Map(_)));
            if !ok {
                self.errors.push(Diagnostic::error(
                    "E0208",
                    format!("`{name}` expects {expects}, found {}", t.article()),
                    args[0].value.span,
                ));
            }
        }
        Some(ret)
    }
}
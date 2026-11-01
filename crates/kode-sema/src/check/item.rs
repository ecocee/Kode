use std::collections::HashSet;

use kode_diagnostics::Diagnostic;
use kode_syntax::ast::*;

use super::{Checker, ClassInfo, ConstVal, EnumInfo, FieldInfo, FnSig};
use crate::builtins;
use crate::types::{assignable, Type};

impl Checker {
    pub(crate) fn resolve_type(&mut self, ann: &TypeAnn) -> Option<Type> {
        match &ann.kind {
            TypeAnnKind::Name(n) => {
                if let Some(t) = Type::from_name(n) {
                    Some(t)
                } else if self.class_index.contains_key(n) {
                    Some(Type::Class(n.clone()))
                } else if self.enum_index.contains_key(n) {
                    Some(Type::Enum(n.clone()))
                } else {
                    self.errors.push(
                        Diagnostic::error("E0303", format!("unknown type `{n}`"), ann.span)
                            .with_help("the types are int, float, bool, string, your classes and enums, [T], {string: T} and T?"),
                    );
                    None
                }
            }
            TypeAnnKind::List(inner) => Some(Type::List(Box::new(self.resolve_type(inner)?))),
            TypeAnnKind::Map(inner) => Some(Type::Map(Box::new(self.resolve_type(inner)?))),
            TypeAnnKind::Opt(inner) => {
                let t = self.resolve_type(inner)?;
                if !t.is_ref() {
                    self.errors.push(
                        Diagnostic::error(
                            "E0219",
                            format!("`{}?` is not supported yet", t.name()),
                            ann.span,
                        )
                        .with_help("optionals work for string, lists, maps and classes"),
                    );
                    return None;
                }
                Some(Type::Opt(Box::new(t)))
            }
        }
    }

    pub(crate) fn collect(&mut self, program: &Program) {
        // 1. class and enum names
        for item in &program.items {
            match item {
                Item::Class(c) => {
                    if self.class_index.contains_key(&c.name) || self.enum_index.contains_key(&c.name) {
                        let what = if c.name == "Error" { "is built in" } else { "is defined twice" };
                        self.errors.push(Diagnostic::error(
                            "E0301",
                            format!("the class `{}` {what}", c.name),
                            c.name_span,
                        ));
                    } else {
                        self.class_index.insert(c.name.clone(), self.classes.len());
                        self.classes.push(ClassInfo {
                            name: c.name.clone(),
                            fields: Vec::new(),
                            methods: Default::default(),
                        });
                    }
                }
                Item::Enum(e) => {
                    if self.class_index.contains_key(&e.name) || self.enum_index.contains_key(&e.name) {
                        self.errors.push(Diagnostic::error(
                            "E0301",
                            format!("the name `{}` is defined twice", e.name),
                            e.name_span,
                        ));
                    } else {
                        self.enum_index.insert(e.name.clone(), self.enums.len());
                        self.enums.push(EnumInfo {
                            name: e.name.clone(),
                            variants: e.variants.iter().map(|(n, _)| n.clone()).collect(),
                        });
                    }
                }
                _ => {}
            }
        }
        // 2. class fields
        let mut done = HashSet::new();
        for item in &program.items {
            let Item::Class(c) = item else { continue };
            if !done.insert(c.name.clone()) || c.name == "Error" {
                continue;
            }
            let ci = self.class_index[&c.name];
            let mut fields: Vec<FieldInfo> = Vec::new();
            for f in &c.fields {
                if fields.iter().any(|x| x.name == f.name) {
                    self.errors.push(Diagnostic::error(
                        "E0306",
                        format!("the field `{}` appears twice", f.name),
                        f.name_span,
                    ));
                }
                let ty = self.resolve_type(&f.ty).unwrap_or(Type::Int);
                fields.push(FieldInfo {
                    name: f.name.clone(),
                    ty,
                    default: f.default.clone(),
                });
            }
            self.classes[ci].fields = fields;
        }
        self.collect_consts(program);
        self.collect_fns(program);
    }

    fn collect_consts(&mut self, program: &Program) {
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
                    Diagnostic::error("E0307", "a constant must be a plain value", c.value.span)
                        .with_help("use a number, true, false or a string without `${}`"),
                );
                continue;
            };
            if let Some(ann) = &c.ty {
                if let Some(want) = self.resolve_type(ann) {
                    if want != ty {
                        self.errors.push(Diagnostic::error(
                            "E0202",
                            format!(
                                "`{}` is declared {} but its value is {}",
                                c.name,
                                want.article(),
                                ty.article()
                            ),
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

    fn collect_fns(&mut self, program: &Program) {
        for (cls, f) in program.fn_decls() {
            let mut params: Vec<(String, Type)> = Vec::new();
            if let Some(c) = cls {
                params.push(("this".into(), Type::Class(c.name.clone())));
            }
            let field_names: Vec<String> = cls
                .and_then(|c| self.class_index.get(&c.name))
                .map(|&ci| self.classes[ci].fields.iter().map(|f| f.name.clone()).collect())
                .unwrap_or_default();
            for p in &f.params {
                if params.iter().any(|(n, _)| *n == p.name) {
                    self.errors.push(Diagnostic::error(
                        "E0306",
                        format!("the parameter `{}` appears twice", p.name),
                        p.name_span,
                    ));
                }
                if field_names.contains(&p.name) {
                    self.errors.push(
                        Diagnostic::error(
                            "E0228",
                            format!("the parameter `{}` has the same name as a field", p.name),
                            p.name_span,
                        )
                        .with_help("rename the parameter; inside a method a field is used by its plain name"),
                    );
                }
                let ty = self.resolve_type(&p.ty).unwrap_or(Type::Int);
                params.push((p.name.clone(), ty));
            }
            let ret = match &f.ret {
                Some(a) => self.resolve_type(a).unwrap_or(Type::Unit),
                None => Type::Unit,
            };
            let index = self.fns.len();
            let name = match cls {
                None => {
                    if builtins::lookup(&f.name).is_some() {
                        self.errors.push(
                            Diagnostic::error(
                                "E0302",
                                format!("`{}` is a built-in function and cannot be redefined", f.name),
                                f.name_span,
                            )
                            .with_help("pick another name"),
                        );
                    } else if self.fn_index.contains_key(&f.name) || self.class_index.contains_key(&f.name) {
                        self.errors.push(Diagnostic::error(
                            "E0301",
                            format!("the name `{}` is defined twice", f.name),
                            f.name_span,
                        ));
                    } else {
                        self.fn_index.insert(f.name.clone(), index);
                    }
                    f.name.clone()
                }
                Some(c) => {
                    if let Some(&ci) = self.class_index.get(&c.name) {
                        if self.classes[ci].methods.contains_key(&f.name) {
                            self.errors.push(Diagnostic::error(
                                "E0301",
                                format!("the method `{}` is defined twice", f.name),
                                f.name_span,
                            ));
                        } else {
                            self.classes[ci].methods.insert(f.name.clone(), index);
                        }
                    }
                    format!("{}.{}", c.name, f.name)
                }
            };
            self.fns.push(FnSig {
                name,
                params,
                ret,
                class: cls.map(|c| c.name.clone()),
            });
            self.direct_throw.push(false);
            self.calls.push(Vec::new());
        }
    }

    pub(crate) fn check_fn(&mut self, f: &FnDecl, index: usize, cls: Option<&ClassDecl>) {
        let sig = self.fns[index].clone();
        self.reset_context();
        self.in_fn = true;
        self.cur_fn = Some(index);
        self.cur_class = cls.and_then(|c| self.class_index.get(&c.name).copied());
        let skip = usize::from(cls.is_some());
        for (i, (name, ty)) in sig.params.iter().enumerate() {
            let span = if i < skip { f.name_span } else { f.params[i - skip].name_span };
            self.scopes.declare(name, ty.clone(), span, true);
        }
        self.ret_type = sig.ret.clone();

        match &f.body {
            FnBody::Block(stmts) => {
                for s in stmts {
                    self.stmt(s);
                }
                if sig.ret != Type::Unit && !super::stmt::always_returns(stmts) {
                    self.errors.push(
                        Diagnostic::error(
                            "E0305",
                            format!("`{}` must return {} on every path", f.name, sig.ret.article()),
                            f.name_span,
                        )
                        .with_help("end the function with a `return`, or give both branches of the last `if` a `return`"),
                    );
                }
            }
            FnBody::Expr(e) => {
                if let Some(t) = self.expr(e) {
                    if !assignable(&t, &sig.ret) {
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
                    } else {
                        self.retype(e, &sig.ret);
                    }
                }
            }
        }
        self.pop_scope();
    }

    pub(crate) fn check_field_defaults(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Class(c) = item else { continue };
            let Some(&ci) = self.class_index.get(&c.name) else { continue };
            if c.name == "Error" {
                continue;
            }
            let fields = self.classes[ci].fields.clone();
            for f in fields {
                if let Some(d) = &f.default {
                    self.reset_context();
                    let got = self.expr(d);
                    self.coerce(d, got, &f.ty, "the default value", "E0202");
                    self.pop_scope();
                }
            }
        }
    }
}
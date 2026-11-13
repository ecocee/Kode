use kode_sema::builtins::{self, Builtin};
use kode_sema::std_fns::{self, StdFn, StdTy};
use kode_sema::Type;
use kode_syntax::ast::{self, Arg, Expr, ExprKind, InterpPart};

use super::{mir_ty, Lowerer, V};
use crate::ir::*;

fn mir_binop(op: ast::BinOp) -> BinOp {
    match op {
        ast::BinOp::Add => BinOp::Add,
        ast::BinOp::Sub => BinOp::Sub,
        ast::BinOp::Mul => BinOp::Mul,
        ast::BinOp::Div => BinOp::Div,
        ast::BinOp::Rem => BinOp::Rem,
        ast::BinOp::Eq => BinOp::Eq,
        ast::BinOp::Ne => BinOp::Ne,
        ast::BinOp::Lt => BinOp::Lt,
        ast::BinOp::Le => BinOp::Le,
        ast::BinOp::Gt => BinOp::Gt,
        ast::BinOp::Ge => BinOp::Ge,
        ast::BinOp::BitAnd => BinOp::BitAnd,
        ast::BinOp::BitOr => BinOp::BitOr,
        ast::BinOp::BitXor => BinOp::BitXor,
        ast::BinOp::Shl => BinOp::Shl,
        ast::BinOp::Shr => BinOp::Shr,
        _ => unreachable!("lowered separately"),
    }
}

impl<'a> Lowerer<'a> {
    pub(crate) fn expr(&mut self, e: &Expr) -> V {
        match &e.kind {
            ExprKind::Int(v) => V::val(Operand::Int(*v)),
            ExprKind::Float(v) => V::val(Operand::Float(*v)),
            ExprKind::Bool(v) => V::val(Operand::Bool(*v)),
            ExprKind::Str(s) => V::val(Operand::Str(self.intern(s))),
            ExprKind::None => V::val(Operand::NullRef),
            ExprKind::Interp(parts) => self.interp(parts),
            ExprKind::Ident(name) => self.ident(name),
            ExprKind::Unary { op, expr } => {
                let v = self.expr(expr);
                let t = self.new_temp(self.ety(e));
                let op = match op {
                    ast::UnOp::Neg => UnOp::Neg,
                    ast::UnOp::Not => UnOp::Not,
                    ast::UnOp::BitNot => UnOp::BitNot,
                };
                self.emit(Inst::Assign {
                    dst: t,
                    rv: Rvalue::Unary(op, v.op),
                });
                V::val(Operand::Local(t))
            }
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e),
            ExprKind::Coalesce { lhs, rhs } => self.coalesce(lhs, rhs, e),
            ExprKind::Unwrap(inner) => self.unwrap(inner, e),
            ExprKind::List(items) => self.list_lit(items, e),
            ExprKind::Map(pairs) => self.map_lit(pairs, e),
            ExprKind::Index { obj, index } => self.index(obj, index, e),
            ExprKind::Slice {
                obj,
                start,
                end,
                inclusive,
            } => self.slice(obj, start.as_deref(), end.as_deref(), *inclusive),
            ExprKind::Member {
                obj, name, safe, ..
            } => self.member(e, obj, name, *safe),
            ExprKind::Is {
                obj,
                enum_name,
                variant,
                ..
            } => self.is_test(obj, enum_name, variant),
            ExprKind::MethodCall {
                obj, name, args, ..
            } => self.method_call(e, obj, name, args),
            ExprKind::Call { callee, args, .. } => self.call(e, callee, args),
            ExprKind::Closure { .. } => self.closure(e),
            ExprKind::Spawn { callee, args, .. } => self.spawn(e, callee, args),
            ExprKind::Await(inner) => self.await_expr(inner, e),
            ExprKind::Catch {
                expr, var, block, ..
            } => self.catch_expr(expr, var, block),
        }
    }

    fn ident(&mut self, name: &str) -> V {
        if let Some(id) = self.lookup(name) {
            if let Some(&flag) = self.opt_flags.get(&id) {
                return V::opt(Operand::Local(id), Operand::Local(flag));
            }
            return V::val(Operand::Local(id));
        }
        if let Some(op) = self.const_operand(name) {
            return V::val(op);
        }
        if let Some(&idx) = self.checked.fn_index.get(name) {
            return self.fn_value(idx);
        }
        let (offset, ty) = self
            .this_field(name)
            .expect("the type checker resolved this name");
        let this = Operand::Local(
            self.this_local
                .expect("a field is only visible inside a method"),
        );
        let dst = self.new_temp(self.mty(&ty));
        self.emit(Inst::Load {
            dst,
            base: this,
            offset,
            ty: self.mty(&ty),
        });
        V::val(Operand::Local(dst))
    }

    /// Offset and type of a field of the current class.
    pub fn this_field(&self, name: &str) -> Option<(i32, Type)> {
        let ci = self.cur_class?;
        let fields = &self.checked.classes[ci].fields;
        let i = fields.iter().position(|f| f.name == name)?;
        Some((24 + 8 * i as i32, fields[i].ty.clone()))
    }

    /// `|x: int| x * 2`: a heap object `[rc][kind][refmask][code][captures...]`.
    /// The closure object is passed as the first argument when it is called.
    fn closure(&mut self, e: &Expr) -> V {
        let ExprKind::Closure { body, .. } = &e.kind else {
            unreachable!("a closure literal")
        };
        let body = (**body).clone();
        let info = self.checked.closures[&e.id].clone();
        let n = info.captures.len();
        // Field 0 is the code address; capture `i` is field `1 + i`.
        let mask: i64 = info
            .captures
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| self.mty(t) == Ty::Ref)
            .fold(0, |m, (i, _)| m | (1 << (i + 1)));
        let obj = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(obj),
            callee: Callee::Rt(RtFn::ObjNew),
            args: vec![Operand::Int((n + 1) as i64), Operand::Int(mask)],
        });
        let spec = crate::lower::ClosureSpec {
            params: info
                .params
                .iter()
                .map(|(n, t)| (n.clone(), self.mty(t)))
                .collect(),
            ret: self.mty(&info.ret),
            captures: info
                .captures
                .iter()
                .map(|(n, t)| (n.clone(), self.mty(t)))
                .collect(),
            body,
        };
        let idx = self.strings.closure(spec);
        self.emit(Inst::Store {
            base: Operand::Local(obj),
            offset: 24,
            value: Operand::ClosureFn(idx),
        });
        for (i, (name, ty)) in info.captures.iter().enumerate() {
            let v = self.ident(name);
            let op = if self.mty(ty) == Ty::Ref {
                self.own(v)
            } else {
                v.op
            };
            self.emit(Inst::Store {
                base: Operand::Local(obj),
                offset: 32 + 8 * i as i32,
                value: op,
            });
        }
        self.owned(Operand::Local(obj))
    }

    /// A named function used as a value: a closure object with no captures.
    fn fn_value(&mut self, idx: usize) -> V {
        let k = self.strings.fnval(idx);
        let obj = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(obj),
            callee: Callee::Rt(RtFn::ObjNew),
            args: vec![Operand::Int(1), Operand::Int(0)],
        });
        self.emit(Inst::Store {
            base: Operand::Local(obj),
            offset: 24,
            value: Operand::FnVal(k),
        });
        self.owned(Operand::Local(obj))
    }

    fn binary(&mut self, op: ast::BinOp, lhs: &Expr, rhs: &Expr, e: &Expr) -> V {
        if matches!(op, ast::BinOp::And | ast::BinOp::Or) {
            return self.short_circuit(op, lhs, rhs);
        }
        if let Some(v) = self.even_check(op, lhs, rhs) {
            return v;
        }
        let lt = self.expr_type(lhs);
        let l = self.expr(lhs);
        let r = self.expr(rhs);

        // `x == none` / `x != none` on an optional number or bool: the answer is
        // the `present` flag, not a pointer comparison.
        if matches!(op, ast::BinOp::Eq | ast::BinOp::Ne) {
            if let Some(flag) = l.present.clone().or_else(|| r.present.clone()) {
                self.release_if_owned(l);
                self.release_if_owned(r);
                if op == ast::BinOp::Ne {
                    return V::val(flag);
                }
                let t = self.new_temp(Ty::Bool);
                self.emit(Inst::Assign {
                    dst: t,
                    rv: Rvalue::Unary(UnOp::Not, flag),
                });
                return V::val(Operand::Local(t));
            }
        }

        if op == ast::BinOp::In {
            let t = self.new_temp(Ty::Bool);
            self.emit(Inst::Call {
                dst: Some(t),
                callee: Callee::Rt(RtFn::MapHas),
                args: vec![r.op.clone(), l.op.clone()],
            });
            self.release_if_owned(l);
            self.release_if_owned(r);
            return V::val(Operand::Local(t));
        }

        if *lt == Type::Str {
            return match op {
                ast::BinOp::Add => {
                    let t = self.new_temp(Ty::Ref);
                    self.emit(Inst::Call {
                        dst: Some(t),
                        callee: Callee::Rt(RtFn::StrConcat),
                        args: vec![l.op.clone(), r.op.clone()],
                    });
                    self.release_if_owned(l);
                    self.release_if_owned(r);
                    self.owned(Operand::Local(t))
                }
                _ => {
                    let t = self.new_temp(Ty::Bool);
                    self.emit(Inst::Call {
                        dst: Some(t),
                        callee: Callee::Rt(RtFn::StrEq),
                        args: vec![l.op.clone(), r.op.clone()],
                    });
                    self.release_if_owned(l);
                    self.release_if_owned(r);
                    if op == ast::BinOp::Ne {
                        let n = self.new_temp(Ty::Bool);
                        self.emit(Inst::Assign {
                            dst: n,
                            rv: Rvalue::Unary(UnOp::Not, Operand::Local(t)),
                        });
                        V::val(Operand::Local(n))
                    } else {
                        V::val(Operand::Local(t))
                    }
                }
            };
        }

        let t = self.new_temp(self.ety(e));
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Binary(mir_binop(op), l.op.clone(), r.op.clone()),
        });
        self.release_if_owned(l);
        self.release_if_owned(r);
        V::val(Operand::Local(t))
    }

    /// `x % 2 == 0` (or `!= 0`) with a power of two: a bit test is much cheaper than a division.
    fn even_check(&mut self, op: ast::BinOp, lhs: &Expr, rhs: &Expr) -> Option<V> {
        if !matches!(op, ast::BinOp::Eq | ast::BinOp::Ne) {
            return None;
        }
        let ExprKind::Int(0) = rhs.kind else {
            return None;
        };
        let ExprKind::Binary {
            op: ast::BinOp::Rem,
            lhs: x,
            rhs: m,
        } = &lhs.kind
        else {
            return None;
        };
        let ExprKind::Int(c) = m.kind else {
            return None;
        };
        if c <= 0 || c & (c - 1) != 0 {
            return None;
        }
        let xv = self.expr(x);
        let masked = self.new_temp(Ty::Int);
        self.emit(Inst::Assign {
            dst: masked,
            rv: Rvalue::Binary(BinOp::BitAnd, xv.op, Operand::Int(c - 1)),
        });
        let t = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Binary(mir_binop(op), Operand::Local(masked), Operand::Int(0)),
        });
        Some(V::val(Operand::Local(t)))
    }

    /// `a and b` / `a or b`: the right side runs only when needed.
    fn short_circuit(&mut self, op: ast::BinOp, lhs: &Expr, rhs: &Expr) -> V {
        let res = self.new_temp(Ty::Bool);
        let l = self.expr(lhs);
        self.emit(Inst::Assign {
            dst: res,
            rv: Rvalue::Use(l.op),
        });
        let rhs_bb = self.new_block();
        let join = self.new_block();
        let (then_bb, else_bb) = if op == ast::BinOp::And {
            (rhs_bb, join)
        } else {
            (join, rhs_bb)
        };
        self.term(Terminator::Branch {
            cond: Operand::Local(res),
            then_bb,
            else_bb,
        });
        self.switch(rhs_bb);
        let r = self.expr(rhs);
        self.emit(Inst::Assign {
            dst: res,
            rv: Rvalue::Use(r.op),
        });
        self.term(Terminator::Goto(join));
        self.switch(join);
        V::val(Operand::Local(res))
    }

    /// `a ?? b`
    fn coalesce(&mut self, lhs: &Expr, rhs: &Expr, e: &Expr) -> V {
        let l = self.expr(lhs);
        // An optional number: take the value or the flag's other side.
        if let Some(flag) = l.present.clone() {
            let ty = self.ety(e);
            let res = self.new_temp(ty);
            self.emit(Inst::Assign {
                dst: res,
                rv: Rvalue::Use(l.op.clone()),
            });
            let rhs_bb = self.new_block();
            let join = self.new_block();
            self.term(Terminator::Branch {
                cond: flag,
                then_bb: join,
                else_bb: rhs_bb,
            });
            self.switch(rhs_bb);
            let r = self.expr(rhs);
            let op = if ty == Ty::Ref { self.own(r) } else { r.op };
            self.emit(Inst::Assign {
                dst: res,
                rv: Rvalue::Use(op),
            });
            self.term(Terminator::Goto(join));
            self.switch(join);
            return if ty == Ty::Ref {
                self.owned(Operand::Local(res))
            } else {
                V::val(Operand::Local(res))
            };
        }
        let t = self.new_temp(Ty::Ref);
        let lo = self.own(l);
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Use(lo),
        });
        let isnull = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: isnull,
            rv: Rvalue::Binary(BinOp::Eq, Operand::Local(t), Operand::NullRef),
        });
        let rhs_bb = self.new_block();
        let join = self.new_block();
        self.term(Terminator::Branch {
            cond: Operand::Local(isnull),
            then_bb: rhs_bb,
            else_bb: join,
        });
        self.switch(rhs_bb);
        let r = self.expr(rhs);
        let ro = self.own(r);
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Use(ro),
        });
        self.term(Terminator::Goto(join));
        self.switch(join);
        self.owned(Operand::Local(t))
    }

    /// `a!`
    fn unwrap(&mut self, inner: &Expr, e: &Expr) -> V {
        let v = self.expr(inner);
        // An optional number: `none` raises, otherwise the value is the answer.
        if let Some(flag) = v.present.clone() {
            let fail = self.new_block();
            let ok = self.new_block();
            self.term(Terminator::Branch {
                cond: flag,
                then_bb: ok,
                else_bb: fail,
            });
            self.switch(fail);
            let line = self.line_of(e.span.start);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Rt(RtFn::UnwrapFail),
                args: vec![Operand::Int(line)],
            });
            self.term(Terminator::Unreachable);
            self.switch(ok);
            return V::val(v.op);
        }
        let isnull = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: isnull,
            rv: Rvalue::Binary(BinOp::Eq, v.op.clone(), Operand::NullRef),
        });
        let fail = self.new_block();
        let ok = self.new_block();
        self.term(Terminator::Branch {
            cond: Operand::Local(isnull),
            then_bb: fail,
            else_bb: ok,
        });
        self.switch(fail);
        let line = self.line_of(e.span.start);
        self.emit(Inst::Call {
            dst: None,
            callee: Callee::Rt(RtFn::UnwrapFail),
            args: vec![Operand::Int(line)],
        });
        self.term(Terminator::Unreachable);
        self.switch(ok);
        v
    }

    /// `s is Shape.Circle`: one tag read and one compare (ADR 0039 §4.4).
    fn is_test(&mut self, obj: &Expr, enum_name: &str, variant: &str) -> V {
        let ei = self.checked.enum_index[enum_name];
        let vi = self.checked.enums[ei].variant_index(variant).unwrap();
        let payload = self.checked.enums[ei].has_payloads();
        let v = self.expr(obj);
        let tag = if payload {
            let t = self.new_temp(Ty::Int);
            self.emit(Inst::Call {
                dst: Some(t),
                callee: Callee::Rt(RtFn::EnumTag),
                args: vec![v.op.clone()],
            });
            Operand::Local(t)
        } else {
            v.op.clone()
        };
        let cond = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: cond,
            rv: Rvalue::Binary(BinOp::Eq, tag, Operand::Int(vi as i64)),
        });
        self.release_if_owned(v);
        V::val(Operand::Local(cond))
    }

    /// The text of an enum value, written the way it is written in source:
    /// `Color.Red`. Every variant is a static string, picked with `select`, so
    /// this needs no runtime call and does not allocate.
    fn enum_str(&mut self, name: &str, value: Operand) -> Operand {
        let ei = self.checked.enum_index[name];
        let variants = self.checked.enums[ei].variants.clone();
        let text = |v: &str| format!("{name}.{v}");
        let mut cur = match variants.last() {
            Some(last) => {
                let s = self.intern(&text(last));
                Operand::Str(s)
            }
            None => {
                let s = self.intern(name);
                Operand::Str(s)
            }
        };
        for (i, variant) in variants.iter().enumerate().rev() {
            if i + 1 == variants.len() {
                continue;
            }
            let cond = self.new_temp(Ty::Bool);
            self.emit(Inst::Assign {
                dst: cond,
                rv: Rvalue::Binary(BinOp::Eq, value.clone(), Operand::Int(i as i64)),
            });
            let s = self.intern(&text(variant));
            let t = self.new_temp(Ty::Ref);
            self.emit(Inst::Assign {
                dst: t,
                rv: Rvalue::Select(Operand::Local(cond), Operand::Str(s), cur),
            });
            cur = Operand::Local(t);
        }
        cur
    }

    /// `Shape.Circle(radius: 1.5)` as text: the variant name, then each field
    /// with its declared name. Concatenation only; no formatting at run time.
    fn enum_payload_str(&mut self, ei: usize, v: Operand) -> Operand {
        let info = self.checked.enums[ei].clone();
        let tag = self.new_temp(Ty::Int);
        self.emit(Inst::Call {
            dst: Some(tag),
            callee: Callee::Rt(RtFn::EnumTag),
            args: vec![v.clone()],
        });
        let out = self.new_temp(Ty::Ref);
        let last = info.variants.len() - 1;
        let first = self.variant_text(ei, last, v.clone());
        self.emit(Inst::Assign {
            dst: out,
            rv: Rvalue::Use(first),
        });
        let join = self.new_block();
        for vi in 0..last {
            let body = self.new_block();
            let next = self.new_block();
            let cond = self.new_temp(Ty::Bool);
            self.emit(Inst::Assign {
                dst: cond,
                rv: Rvalue::Binary(BinOp::Eq, Operand::Local(tag), Operand::Int(vi as i64)),
            });
            self.term(Terminator::Branch {
                cond: Operand::Local(cond),
                then_bb: body,
                else_bb: next,
            });
            self.switch(body);
            let s = self.variant_text(ei, vi, v.clone());
            // `out` holds the previous candidate; replace it, do not leak it.
            self.emit(Inst::Release(Operand::Local(out)));
            self.emit(Inst::Assign {
                dst: out,
                rv: Rvalue::Use(s),
            });
            self.term(Terminator::Goto(join));
            self.switch(next);
        }
        self.term(Terminator::Goto(join));
        self.switch(join);
        Operand::Local(out)
    }

    /// The text of one variant, fields included; always a fresh string.
    fn variant_text(&mut self, ei: usize, vi: usize, v: Operand) -> Operand {
        let info = self.checked.enums[ei].clone();
        let fields = info.fields[vi].clone();
        let head = self.intern(&format!("{}.{}", info.name, info.variants[vi]));
        let cur = Operand::Str(head);
        if fields.is_empty() {
            let empty = Operand::Str(self.intern(""));
            return self.concat(cur, empty);
        }
        let open = Operand::Str(self.intern("("));
        let mut cur = self.concat(cur, open);
        for (i, (fname, fty)) in fields.iter().enumerate() {
            if i > 0 {
                let sep = Operand::Str(self.intern(", "));
                let t = self.concat(cur.clone(), sep);
                self.emit(Inst::Release(cur));
                cur = t;
            }
            let label = Operand::Str(self.intern(&format!("{fname}: ")));
            let t = self.concat(cur.clone(), label);
            self.emit(Inst::Release(cur));
            cur = t;
            let (val, val_owned) = self.enum_field_text(v.clone(), i, fty);
            let t = self.concat(cur.clone(), val.clone());
            self.emit(Inst::Release(cur));
            if val_owned {
                self.emit(Inst::Release(val));
            }
            cur = t;
        }
        let close = Operand::Str(self.intern(")"));
        let out = self.concat(cur.clone(), close);
        self.emit(Inst::Release(cur));
        out
    }

    /// One field of a variant, formatted; `true` when the result must be freed.
    fn enum_field_text(&mut self, v: Operand, i: usize, ty: &Type) -> (Operand, bool) {
        let load_ty = mir_ty(self.checked, ty);
        let f = self.new_temp(load_ty);
        self.emit(Inst::Load {
            dst: f,
            base: v,
            offset: 24 + 8 * i as i32,
            ty: load_ty,
        });
        match ty {
            Type::Str => (Operand::Local(f), false),
            Type::Int => self.field_str(RtFn::IntToStr, Operand::Local(f)),
            Type::Float => self.field_str(RtFn::FloatToStr, Operand::Local(f)),
            Type::Bool => self.field_str(RtFn::BoolToStr, Operand::Local(f)),
            _ => unreachable!("the checker only lets printable fields through"),
        }
    }

    fn field_str(&mut self, rt: RtFn, arg: Operand) -> (Operand, bool) {
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(rt),
            args: vec![arg],
        });
        (Operand::Local(t), true)
    }

    fn concat(&mut self, a: Operand, b: Operand) -> Operand {
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(RtFn::StrConcat),
            args: vec![a, b],
        });
        Operand::Local(t)
    }

    /// Turn any printable value into a string value.
    fn value_to_string(&mut self, v: V, ty: &Type) -> V {
        // An optional number prints `none`, or the number: the branch is here,
        // not duplicated at every call site.
        if let Some(inner) = kode_sema::types::is_opt_scalar(ty) {
            let flag = v.present.clone().expect("an optional number is a pair");
            let rt = match inner {
                Type::Int => RtFn::IntToStr,
                Type::Float => RtFn::FloatToStr,
                _ => RtFn::BoolToStr,
            };
            let out = self.new_temp(Ty::Ref);
            let fmt_bb = self.new_block();
            let none_bb = self.new_block();
            let join = self.new_block();
            self.term(Terminator::Branch {
                cond: flag,
                then_bb: fmt_bb,
                else_bb: none_bb,
            });
            self.switch(fmt_bb);
            let s = self.new_temp(Ty::Ref);
            self.emit(Inst::Call {
                dst: Some(s),
                callee: Callee::Rt(rt),
                args: vec![v.op.clone()],
            });
            self.emit(Inst::Assign {
                dst: out,
                rv: Rvalue::Use(Operand::Local(s)),
            });
            self.term(Terminator::Goto(join));
            self.switch(none_bb);
            let text = Operand::Str(self.intern("none"));
            self.emit(Inst::Assign {
                dst: out,
                rv: Rvalue::Use(text),
            });
            self.term(Terminator::Goto(join));
            self.switch(join);
            return self.owned(Operand::Local(out));
        }
        if let Type::Enum(name) = ty {
            let ei = self.checked.enum_index[name];
            if self.checked.enums[ei].has_payloads() {
                let s = self.enum_payload_str(ei, v.op.clone());
                self.release_if_owned(v);
                return self.owned(s);
            }
            let name = name.clone();
            let s = self.enum_str(&name, v.op.clone());
            self.release_if_owned(v);
            return V::val(s);
        }
        // A list or map: the runtime gets the value and a descriptor for the
        // item types, and returns a new string. `repr_descriptor` is the same
        // function the checker used to accept the print.
        if matches!(ty, Type::List(_) | Type::Map(_)) {
            let desc = kode_sema::types::repr_descriptor(ty)
                .expect("the checker only accepts printable collections");
            let d = Operand::Str(self.intern(&desc));
            let t = self.new_temp(Ty::Ref);
            self.emit(Inst::Call {
                dst: Some(t),
                callee: Callee::Rt(RtFn::Repr),
                args: vec![v.op.clone(), d],
            });
            self.release_if_owned(v);
            return self.owned(Operand::Local(t));
        }
        let rt = match ty {
            Type::Str => return v,
            Type::Int => RtFn::IntToStr,
            Type::Float => RtFn::FloatToStr,
            Type::Bool => RtFn::BoolToStr,
            _ => unreachable!("the type checker rejects this"),
        };
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(rt),
            args: vec![v.op.clone()],
        });
        self.release_if_owned(v);
        self.owned(Operand::Local(t))
    }

    fn interp(&mut self, parts: &[InterpPart]) -> V {
        let mut acc: Option<V> = None;
        for part in parts {
            let piece = match part {
                InterpPart::Lit(s) => V::val(Operand::Str(self.intern(s))),
                InterpPart::Expr(x) => {
                    let ty = self.expr_type(x);
                    let v = self.expr(x);
                    self.value_to_string(v, ty)
                }
            };
            acc = Some(match acc {
                None => {
                    let op = self.own(piece);
                    self.owned(op)
                }
                Some(a) => {
                    let t = self.new_temp(Ty::Ref);
                    self.emit(Inst::Call {
                        dst: Some(t),
                        callee: Callee::Rt(RtFn::StrConcat),
                        args: vec![a.op.clone(), piece.op.clone()],
                    });
                    self.release_if_owned(a);
                    self.release_if_owned(piece);
                    self.owned(Operand::Local(t))
                }
            });
        }
        acc.unwrap_or_else(|| V::val(Operand::Str(self.intern(""))))
    }

    /// Call a user function or method. Heap arguments are borrowed by the callee.
    pub(crate) fn emit_call(&mut self, idx: usize, vals: Vec<V>) -> V {
        let ret = self.mty(&self.checked.fns[idx].ret);
        let ops: Vec<Operand> = vals.iter().map(|v| v.op.clone()).collect();
        let dst = (ret != Ty::Unit).then(|| self.new_temp(ret));
        self.emit(Inst::Call {
            dst,
            callee: Callee::User(idx),
            args: ops,
        });
        if self.checked.throws[idx] {
            self.error_check();
        }
        for v in vals {
            self.release_if_owned(v);
        }
        match dst {
            Some(d) if ret == Ty::Ref => self.owned(Operand::Local(d)),
            Some(d) => V::val(Operand::Local(d)),
            None => V::unit(),
        }
    }

    /// Evaluate call arguments in parameter order, using the slot map from the checker.
    pub(crate) fn call_vals(
        &mut self,
        id: ast::ExprId,
        args: &[Arg],
        first: Option<V>,
        count: usize,
    ) -> Vec<V> {
        let slots = self.checked.call_args[&id].clone();
        let mut vals: Vec<V> = first.into_iter().collect();
        for slot in slots.iter().take(count) {
            let ai = slot.expect("the type checker made sure every argument is given");
            vals.push(self.expr(&args[ai].value));
        }
        vals
    }

    pub(crate) fn call(&mut self, e: &Expr, callee: &str, args: &[Arg]) -> V {
        if callee == "serve" {
            return self.serve(args);
        }
        if callee == "serve_tls" {
            return self.serve_tls(args);
        }
        if callee == "Router" {
            return self.router_new();
        }
        // `f(1)` where `f` holds a function value.
        if self.checked.fn_value_calls.contains(&e.id) {
            let callee_id = self
                .lookup(callee)
                .expect("the type checker resolved this call");
            let n = self.checked.call_args[&e.id].len();
            let vals = self.call_vals(e.id, args, None, n);
            let ret = self.ety(e);
            let dst = (ret != Ty::Unit).then(|| self.new_temp(ret));
            let ops: Vec<Operand> = vals.iter().map(|v| v.op.clone()).collect();
            self.emit(Inst::CallIndirect {
                dst,
                callee: Operand::Local(callee_id),
                args: ops,
            });
            for v in vals {
                self.release_if_owned(v);
            }
            return match dst {
                Some(d) if ret == Ty::Ref => self.owned(Operand::Local(d)),
                Some(d) => V::val(Operand::Local(d)),
                None => V::unit(),
            };
        }
        if let Some(b) = builtins::lookup(callee) {
            return self.builtin(b, args, e);
        }
        if let Some(f) = std_fns::find_free(callee) {
            return self.std_call(f, None, args);
        }
        let checked = self.checked;
        // A generic call resolved to one concrete instantiation.
        if let Some(&idx) = checked.generic_calls.get(&e.id) {
            let n = checked.fns[idx].params.len();
            let vals = self.call_vals(e.id, args, None, n);
            return self.emit_call(idx, vals);
        }
        if let Some(&idx) = checked.fn_index.get(callee) {
            let n = checked.fns[idx].params.len();
            let vals = self.call_vals(e.id, args, None, n);
            return self.emit_call(idx, vals);
        }
        // A constructor of a generic class instance: the source name is not a
        // class, so the checker records which instance this call built (ADR 0040).
        if let Some(&ci) = checked.class_ctor_calls.get(&e.id) {
            return self.construct(e, ci, args);
        }
        if let Some(&ci) = checked.class_index.get(callee) {
            return self.construct(e, ci, args);
        }
        // A method of the current class, called without `this`.
        let ci = self.cur_class.expect("the type checker resolved this call");
        let mi = checked.classes[ci].methods[callee];
        let this = V::val(Operand::Local(self.this_local.unwrap()));
        let n = checked.fns[mi].params.len() - 1;
        let vals = self.call_vals(e.id, args, Some(this), n);
        self.emit_call(mi, vals)
    }

    /// The MIR type of a table type (`elem` is the item type for `Elem`).
    fn std_ty(&self, t: StdTy, elem: Option<&Type>) -> Ty {
        match t {
            StdTy::Int => Ty::Int,
            StdTy::Float => Ty::Float,
            StdTy::Bool => Ty::Bool,
            StdTy::Unit => Ty::Unit,
            StdTy::Str
            | StdTy::ListStr
            | StdTy::OptStr
            | StdTy::ListJson
            | StdTy::MapJson
            | StdTy::MapStrStr
            | StdTy::Json
            | StdTy::Resp
            | StdTy::ProcResult => Ty::Ref,
            StdTy::Elem => elem.map(|t| self.mty(t)).unwrap_or(Ty::Int),
            StdTy::ElemBits => Ty::Int,
            StdTy::ListElem => Ty::Ref,
        }
    }

    /// A call to a runtime library function from the standard-library table.
    pub(crate) fn std_call(&mut self, f: &'static StdFn, recv: Option<&Expr>, args: &[Arg]) -> V {
        let recv_type = recv.map(|r| self.expr_type(r));
        let elem: Option<&Type> = match recv_type {
            Some(Type::List(el)) => Some(el),
            _ => None,
        };
        let mut vals: Vec<V> = Vec::new();
        let mut ptys: Vec<Ty> = Vec::new();
        if let Some(r) = recv {
            vals.push(self.expr(r));
            ptys.push(Ty::Ref);
        }
        for (a, p) in args.iter().zip(f.params) {
            let is_float = *p == StdTy::ElemBits && *self.expr_type(&a.value) == Type::Float;
            let v = self.expr(&a.value);
            let v = if is_float {
                // The runtime stores floats as bits, like list items.
                let bits = self.new_temp(Ty::Int);
                self.emit(Inst::Assign {
                    dst: bits,
                    rv: Rvalue::Cast(CastKind::FloatToBits, v.op.clone()),
                });
                self.release_if_owned(v);
                V::val(Operand::Local(bits))
            } else {
                v
            };
            vals.push(v);
            ptys.push(self.std_ty(*p, elem));
        }
        let rty = self.std_ty(f.ret, elem);
        // a float item comes back from the runtime as raw bits
        let float_item = f.ret == StdTy::Elem && rty == Ty::Float;
        let sig_ret = if float_item { Ty::Int } else { rty };
        let sig = self.std_sig(f.sym, ptys, sig_ret);
        let dst = (sig_ret != Ty::Unit).then(|| self.new_temp(sig_ret));
        let ops: Vec<Operand> = vals.iter().map(|v| v.op.clone()).collect();
        self.emit(Inst::Call {
            dst,
            callee: Callee::Std(sig),
            args: ops,
        });
        if f.throws {
            self.error_check();
        }
        for v in vals {
            self.release_if_owned(v);
        }
        match dst {
            Some(d) if float_item => {
                let c = self.new_temp(Ty::Float);
                self.emit(Inst::Assign {
                    dst: c,
                    rv: Rvalue::Cast(CastKind::BitsToFloat, Operand::Local(d)),
                });
                V::val(Operand::Local(c))
            }
            Some(d) if rty == Ty::Ref => self.owned(Operand::Local(d)),
            Some(d) => V::val(Operand::Local(d)),
            None => V::unit(),
        }
    }

    /// `Router()`: a router object with a runtime-assigned id.
    fn router_new(&mut self) -> V {
        let id = self.new_temp(Ty::Int);
        let sig = self.std_sig("kode_router_new", vec![], Ty::Int);
        self.emit(Inst::Call {
            dst: Some(id),
            callee: Callee::Std(sig),
            args: vec![],
        });
        let p = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(p),
            callee: Callee::Rt(RtFn::ObjNew),
            args: vec![Operand::Int(1), Operand::Int(0)],
        });
        self.emit(Inst::Store {
            base: Operand::Local(p),
            offset: 24,
            value: Operand::Local(id),
        });
        self.owned(Operand::Local(p))
    }

    /// `serve(port, handler)` or `serve(port, router)`
    fn serve(&mut self, args: &[Arg]) -> V {
        let port = self.expr(&args[0].value);
        if matches!(self.expr_type(&args[1].value), Type::Class(n) if n == "Router") {
            let router = self.expr(&args[1].value);
            let sig = self.std_sig("kode_http_serve_router", vec![Ty::Int, Ty::Ref], Ty::Unit);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Std(sig),
                args: vec![port.op, router.op],
            });
            return V::unit();
        }
        let ExprKind::Ident(name) = &args[1].value.kind else {
            unreachable!("checked")
        };
        let idx = self.checked.fn_index[name];
        let sig = self.std_sig("kode_http_serve", vec![Ty::Int, Ty::Int], Ty::Unit);
        self.emit(Inst::Call {
            dst: None,
            callee: Callee::Std(sig),
            args: vec![port.op, Operand::FuncAddr(idx)],
        });
        V::unit()
    }

    /// `serve_tls(port, cert, key, handler)`: the certificate and key are DER files.
    fn serve_tls(&mut self, args: &[Arg]) -> V {
        let port = self.expr(&args[0].value);
        let cert = self.expr(&args[1].value);
        let key = self.expr(&args[2].value);
        if matches!(self.expr_type(&args[3].value), Type::Class(n) if n == "Router") {
            let router = self.expr(&args[3].value);
            let sig = self.std_sig(
                "kode_http_serve_tls_router",
                vec![Ty::Int, Ty::Ref, Ty::Ref, Ty::Ref],
                Ty::Unit,
            );
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Std(sig),
                args: vec![port.op, cert.op, key.op, router.op],
            });
            return V::unit();
        }
        let ExprKind::Ident(name) = &args[3].value.kind else {
            unreachable!("checked")
        };
        let idx = self.checked.fn_index[name];
        let sig = self.std_sig(
            "kode_http_serve_tls",
            vec![Ty::Int, Ty::Ref, Ty::Ref, Ty::Int],
            Ty::Unit,
        );
        self.emit(Inst::Call {
            dst: None,
            callee: Callee::Std(sig),
            args: vec![port.op, cert.op, key.op, Operand::FuncAddr(idx)],
        });
        V::unit()
    }

    /// `spawn f(args)`: pack the arguments into a block and start a task.
    fn spawn(&mut self, e: &Expr, callee: &str, args: &[Arg]) -> V {
        let checked = self.checked;
        let idx = checked.fn_index[callee];
        let sig_fn = &checked.fns[idx];
        let slots = checked.call_args[&e.id].clone();
        let n = sig_fn.params.len();
        let mask: i64 = sig_fn
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| self.mty(&p.1) == Ty::Ref)
            .fold(0, |m, (i, _)| m | (1 << i));
        let p = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(p),
            callee: Callee::Rt(RtFn::ObjNew),
            args: vec![Operand::Int(n as i64), Operand::Int(mask)],
        });
        let block = self.owned(Operand::Local(p));
        for (i, (_, ty)) in sig_fn.params.iter().enumerate() {
            let v = self.expr(&args[slots[i].expect("checked")].value);
            let op = if self.mty(ty) == Ty::Ref {
                self.own(v)
            } else {
                v.op
            };
            self.emit(Inst::Store {
                base: Operand::Local(p),
                offset: 24 + 8 * i as i32,
                value: op,
            });
        }
        let thunk = self.thunk(idx);
        let result_is_ref = self.mty(&sig_fn.ret) == Ty::Ref;
        let task = self.new_temp(Ty::Ref);
        let sig = self.std_sig("kode_task_spawn", vec![Ty::Int, Ty::Ref, Ty::Int], Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(task),
            callee: Callee::Std(sig),
            args: vec![
                Operand::Thunk(thunk),
                Operand::Local(p),
                Operand::Int(result_is_ref as i64),
            ],
        });
        self.own(block); // the task now owns the argument block
        self.owned(Operand::Local(task))
    }

    /// `await task`
    fn await_expr(&mut self, inner: &Expr, e: &Expr) -> V {
        let ty = self.expr_type(e);
        let tv = self.expr(inner);
        let m = self.mty(ty);
        let raw_ty = if m == Ty::Ref { Ty::Ref } else { Ty::Int };
        let sig = self.std_sig("kode_task_await", vec![Ty::Ref], raw_ty);
        let t = self.new_temp(raw_ty);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Std(sig),
            args: vec![tv.op.clone()],
        });
        self.error_check();
        self.release_if_owned(tv);
        match m {
            Ty::Ref => self.owned(Operand::Local(t)),
            Ty::Bool | Ty::Float => {
                let kind = if m == Ty::Bool {
                    CastKind::IntToBool
                } else {
                    CastKind::BitsToFloat
                };
                let c = self.new_temp(m);
                self.emit(Inst::Assign {
                    dst: c,
                    rv: Rvalue::Cast(kind, Operand::Local(t)),
                });
                V::val(Operand::Local(c))
            }
            Ty::Unit => V::unit(),
            Ty::Int => V::val(Operand::Local(t)),
        }
    }

    fn builtin(&mut self, b: Builtin, args: &[Arg], e: &Expr) -> V {
        let a0 = args.first().map(|a| &a.value);
        match b {
            Builtin::Print => {
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.emit(Inst::Call {
                            dst: None,
                            callee: Callee::Rt(RtFn::PrintSep),
                            args: vec![],
                        });
                    }
                    let ty = self.expr_type(&a.value);
                    if let Type::Enum(name) = &ty {
                        let name = name.clone();
                        let ei = self.checked.enum_index[&name];
                        let payload = self.checked.enums[ei].has_payloads();
                        let v = self.expr(&a.value);
                        let s = if payload {
                            self.enum_payload_str(ei, v.op.clone())
                        } else {
                            self.enum_str(&name, v.op.clone())
                        };
                        self.emit(Inst::Call {
                            dst: None,
                            callee: Callee::Rt(RtFn::PrintStr),
                            args: vec![s.clone()],
                        });
                        if payload {
                            self.emit(Inst::Release(s));
                        }
                        self.release_if_owned(v);
                        continue;
                    }
                    if kode_sema::types::is_opt_scalar(ty).is_some()
                        || matches!(ty, Type::List(_) | Type::Map(_))
                    {
                        let v = self.expr(&a.value);
                        let s = self.value_to_string(v, ty);
                        self.emit(Inst::Call {
                            dst: None,
                            callee: Callee::Rt(RtFn::PrintStr),
                            args: vec![s.op.clone()],
                        });
                        self.release_if_owned(s);
                        continue;
                    }
                    let v = self.expr(&a.value);
                    let rt = match ty {
                        Type::Int => RtFn::PrintInt,
                        Type::Float => RtFn::PrintFloat,
                        Type::Bool => RtFn::PrintBool,
                        Type::Str => RtFn::PrintStr,
                        _ => unreachable!("the type checker rejects this"),
                    };
                    self.emit(Inst::Call {
                        dst: None,
                        callee: Callee::Rt(rt),
                        args: vec![v.op.clone()],
                    });
                    self.release_if_owned(v);
                }
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(RtFn::PrintNl),
                    args: vec![],
                });
                V::unit()
            }
            Builtin::Len => {
                let arg = a0.unwrap();
                let rt = match self.expr_type(arg) {
                    Type::List(_) => RtFn::ListLen,
                    Type::Map(_) => RtFn::MapLen,
                    _ => RtFn::StrLen,
                };
                let v = self.expr(arg);
                let t = self.new_temp(Ty::Int);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(rt),
                    args: vec![v.op.clone()],
                });
                self.release_if_owned(v);
                V::val(Operand::Local(t))
            }
            Builtin::Int => {
                let arg = a0.unwrap();
                let from = self.ety(arg);
                let v = self.expr(arg);
                let kind = match from {
                    Ty::Float => CastKind::FloatToInt,
                    Ty::Bool => CastKind::BoolToInt,
                    _ => return v,
                };
                let t = self.new_temp(Ty::Int);
                self.emit(Inst::Assign {
                    dst: t,
                    rv: Rvalue::Cast(kind, v.op),
                });
                V::val(Operand::Local(t))
            }
            Builtin::Float => {
                let arg = a0.unwrap();
                let from = self.ety(arg);
                let v = self.expr(arg);
                if from == Ty::Float {
                    return v;
                }
                let t = self.new_temp(Ty::Float);
                self.emit(Inst::Assign {
                    dst: t,
                    rv: Rvalue::Cast(CastKind::IntToFloat, v.op),
                });
                V::val(Operand::Local(t))
            }
            Builtin::Str => {
                let arg = a0.unwrap();
                let ty = self.expr_type(arg);
                let v = self.expr(arg);
                self.value_to_string(v, ty)
            }
            Builtin::Assert => {
                let c = self.expr(a0.unwrap());
                let ok = self.new_block();
                let fail = self.new_block();
                self.term(Terminator::Branch {
                    cond: c.op,
                    then_bb: ok,
                    else_bb: fail,
                });
                self.switch(fail);
                let line = self.line_of(e.span.start);
                let id = self.intern(&format!("assertion failed at line {line}"));
                let obj = self.make_error(Operand::Str(id));
                self.raise(obj);
                self.switch(ok);
                V::unit()
            }
            Builtin::Channel => {
                let t = self.new_temp(Ty::Ref);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(RtFn::ChanNew),
                    args: vec![],
                });
                self.owned(Operand::Local(t))
            }
            Builtin::Send => {
                let ch = self.expr(&args[0].value);
                let payload = match self.expr_type(&args[0].value) {
                    Type::Chan(t) => (**t).clone(),
                    _ => Type::NoneT,
                };
                let v = self.expr(&args[1].value);
                let is_ref = self.mty(&payload) == Ty::Ref;
                let op = if is_ref { self.own(v) } else { v.op };
                let raw = self.raw_value(op, &payload);
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(RtFn::ChanSend),
                    args: vec![ch.op.clone(), raw, Operand::Int(is_ref as i64)],
                });
                self.error_check();
                self.release_if_owned(ch);
                V::unit()
            }
            Builtin::Recv => {
                let ch = self.expr(&args[0].value);
                let payload = match self.expr_type(&args[0].value) {
                    Type::Chan(t) => (**t).clone(),
                    _ => Type::NoneT,
                };
                let is_ref = self.mty(&payload) == Ty::Ref;
                let raw = self.call_raw(RtFn::ChanRecv, vec![ch.op.clone()], &payload);
                self.error_check();
                self.release_if_owned(ch);
                if is_ref {
                    self.owned(raw)
                } else {
                    V::val(raw)
                }
            }
            Builtin::Close => {
                let ch = self.expr(&args[0].value);
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(RtFn::ChanClose),
                    args: vec![ch.op.clone()],
                });
                self.error_check();
                self.release_if_owned(ch);
                V::unit()
            }
        }
    }
}

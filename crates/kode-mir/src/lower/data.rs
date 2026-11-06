//! Lowering of lists, maps, class instances, optionals and errors.

use kode_syntax::ast::{Arg, Expr, ExprKind, Stmt};

use kode_sema::Type;

use super::{mir_ty, Lowerer, V};
use crate::ir::*;

fn strip_opt(t: &Type) -> &Type {
    match t {
        Type::Opt(i) => i,
        t => t,
    }
}

/// Lists of `bool` are stored with one byte per item.
pub(crate) fn packed(el: &Type) -> bool {
    *el == Type::Bool
}

/// The runtime function for a list operation, depending on the item type.
pub(crate) fn list_rt(el: &Type, base: RtFn) -> RtFn {
    if !packed(el) {
        return base;
    }
    match base {
        RtFn::ListPush => RtFn::ListPush8,
        RtFn::ListGet => RtFn::ListGet8,
        RtFn::ListSet => RtFn::ListSet8,
        RtFn::ListSlice => RtFn::ListSlice8,
        other => other,
    }
}

impl<'a> Lowerer<'a> {
    /// Store a value in a 64-bit slot (list item, map value).
    pub fn to_raw(&mut self, op: Operand, ty: &Type) -> Operand {
        let kind = match mir_ty(ty) {
            Ty::Bool => CastKind::BoolToInt,
            Ty::Float => CastKind::FloatToBits,
            _ => return op,
        };
        let t = self.new_temp(Ty::Int);
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Cast(kind, op),
        });
        Operand::Local(t)
    }

    /// Call a runtime function that returns a 64-bit slot, and convert it to `ty`.
    pub fn call_raw(&mut self, rt: RtFn, args: Vec<Operand>, ty: &Type) -> Operand {
        let m = mir_ty(ty);
        let raw_ty = if matches!(m, Ty::Int | Ty::Ref) { m } else { Ty::Int };
        let t = self.new_temp(raw_ty);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(rt),
            args,
        });
        let kind = match m {
            Ty::Bool => CastKind::IntToBool,
            Ty::Float => CastKind::BitsToFloat,
            _ => return Operand::Local(t),
        };
        let c = self.new_temp(m);
        self.emit(Inst::Assign {
            dst: c,
            rv: Rvalue::Cast(kind, Operand::Local(t)),
        });
        Operand::Local(c)
    }

    /// The result `result` was read out of `base`. If `base` was a temporary we
    /// own, keep the result alive past the release of `base`.
    pub fn borrow_result(&mut self, base: V, result: Operand, ty: &Type) -> V {
        if !base.owned {
            return V::val(result);
        }
        if mir_ty(ty) == Ty::Ref {
            self.emit(Inst::Retain(result.clone()));
            self.release_if_owned(base);
            self.owned(result)
        } else {
            self.release_if_owned(base);
            V::val(result)
        }
    }

    pub fn make_error(&mut self, message: Operand) -> Operand {
        let p = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(p),
            callee: Callee::Rt(RtFn::ObjNew),
            args: vec![Operand::Int(2), Operand::Int(1)],
        });
        self.emit(Inst::Store {
            base: Operand::Local(p),
            offset: 24,
            value: message,
        });
        Operand::Local(p)
    }

    /// Raise an error object (the runtime takes ownership) and jump to the handler.
    pub fn raise(&mut self, obj: Operand) {
        self.emit(Inst::Call {
            dst: None,
            callee: Callee::Rt(RtFn::ErrThrow),
            args: vec![obj],
        });
        self.goto_error();
    }

    pub(crate) fn list_lit(&mut self, items: &[Expr], e: &Expr) -> V {
        let Type::List(el) = self.expr_type(e) else { unreachable!("a list literal has a list type") };
        let el: &Type = el;
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(if packed(el) { RtFn::ListNewPacked } else { RtFn::ListNew }),
            args: if packed(el) { vec![] } else { vec![Operand::Int((el.is_ref() && *el != Type::NoneT) as i64)] },
        });
        let lv = self.owned(Operand::Local(t));
        for x in items {
            let v = self.expr(x);
            let op = if mir_ty(el) == Ty::Ref { self.own(v) } else { v.op };
            let raw = self.to_raw(op, el);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Rt(list_rt(el, RtFn::ListPush)),
                args: vec![Operand::Local(t), raw],
            });
        }
        lv
    }

    pub(crate) fn map_lit(&mut self, pairs: &[(Expr, Expr)], e: &Expr) -> V {
        let Type::Map(el) = self.expr_type(e) else { unreachable!("a map literal has a map type") };
        let el: &Type = el;
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(RtFn::MapNew),
            args: vec![Operand::Int((el.is_ref() && *el != Type::NoneT) as i64)],
        });
        let mv = self.owned(Operand::Local(t));
        for (k, v) in pairs {
            let kv = self.expr(k);
            let vv = self.expr(v);
            let op = if mir_ty(el) == Ty::Ref { self.own(vv) } else { vv.op };
            let raw = self.to_raw(op, el);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Rt(RtFn::MapSet),
                args: vec![Operand::Local(t), kv.op.clone(), raw],
            });
            self.release_if_owned(kv);
        }
        mv
    }

    /// `xs[a..b]`: a new list holding a copy of the range.
    pub(crate) fn slice(&mut self, obj: &Expr, start: Option<&Expr>, end: Option<&Expr>, inclusive: bool) -> V {
        let ot = self.expr_type(obj);
        let Type::List(el) = ot else { unreachable!("checked: a list") };
        let ov = self.expr(obj);
        let s = match start {
            Some(x) => {
                let v = self.expr(x);
                let op = v.op.clone();
                self.release_if_owned(v);
                op
            }
            None => Operand::Int(0),
        };
        let e = match end {
            Some(x) => {
                let v = self.expr(x);
                let mut op = v.op.clone();
                self.release_if_owned(v);
                if inclusive {
                    let t = self.new_temp(Ty::Int);
                    self.emit(Inst::Assign {
                        dst: t,
                        rv: Rvalue::Binary(BinOp::Add, op, Operand::Int(1)),
                    });
                    op = Operand::Local(t);
                }
                op
            }
            // The runtime clamps a bound past the end to the length.
            None => Operand::Int(i64::MAX),
        };
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(list_rt(&el, RtFn::ListSlice)),
            args: vec![ov.op.clone(), s, e],
        });
        self.release_if_owned(ov);
        self.owned(Operand::Local(t))
    }

    /// `xs.map(f)` / `xs.filter(f)`: one inline loop, so the runtime never has to
    /// call back into user code. The source and the closure stay alive for it.
    pub(crate) fn list_map_filter(&mut self, e: &Expr, obj: &Expr, name: &str, cb: &Expr) -> V {
        let Type::List(el) = self.expr_type(obj).clone() else {
            unreachable!("checked: a list")
        };
        let Type::List(out_el) = self.expr_type(e).clone() else {
            unreachable!("checked: map and filter return a list")
        };
        // Keep the source and the closure alive across the loop, and clear them at
        // the end so the function exit does not release them twice.
        let src = self.hidden_ref();
        let sv = self.expr(obj);
        let sop = self.own(sv);
        self.emit(Inst::Assign {
            dst: src,
            rv: Rvalue::Use(sop),
        });
        let fun = self.hidden_ref();
        let fv = self.expr(cb);
        let fop = self.own(fv);
        self.emit(Inst::Assign {
            dst: fun,
            rv: Rvalue::Use(fop),
        });

        let out = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(out),
            callee: Callee::Rt(if packed(&out_el) {
                RtFn::ListNewPacked
            } else {
                RtFn::ListNew
            }),
            args: if packed(&out_el) {
                vec![]
            } else {
                vec![Operand::Int((out_el.is_ref() && *out_el != Type::NoneT) as i64)]
            },
        });
        let result = self.owned(Operand::Local(out));

        let i = self.new_temp(Ty::Int);
        self.emit(Inst::Assign {
            dst: i,
            rv: Rvalue::Use(Operand::Int(0)),
        });
        let len = self.new_temp(Ty::Int);
        self.emit(Inst::Call {
            dst: Some(len),
            callee: Callee::Rt(RtFn::ListLen),
            args: vec![Operand::Local(src)],
        });

        let head = self.new_block();
        let body = self.new_block();
        let done = self.new_block();
        self.term(Terminator::Goto(head));
        self.switch(head);
        let cond = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: cond,
            rv: Rvalue::Binary(BinOp::Lt, Operand::Local(i), Operand::Local(len)),
        });
        self.term(Terminator::Branch {
            cond: Operand::Local(cond),
            then_bb: body,
            else_bb: done,
        });

        self.switch(body);
        let line = Operand::Int(self.line_of(e.span.start));
        let item = self.call_raw(
            list_rt(&el, RtFn::ListGet),
            vec![Operand::Local(src), Operand::Local(i), line],
            &el,
        );
        // `map` returns the element type, `filter` takes a predicate and gets a
        // `bool` back. The destination has to have the type the closure really
        // returns: a wider slot leaves the upper bits of a `bool` result
        // undefined, and the branch below then reads garbage and keeps every
        // item.
        let res_ty = if name == "filter" { Ty::Bool } else { mir_ty(&out_el) };
        let dst = (res_ty != Ty::Unit).then(|| self.new_temp(res_ty));
        self.emit(Inst::CallIndirect {
            dst,
            callee: Operand::Local(fun),
            args: vec![item.clone()],
        });
        if name == "filter" {
            // Keep the original item when the function says so.
            let keep = self.new_block();
            let skip = self.new_block();
            self.term(Terminator::Branch {
                cond: Operand::Local(dst.expect("filter returns a bool")),
                then_bb: keep,
                else_bb: skip,
            });
            self.switch(keep);
            let kept = if mir_ty(&el) == Ty::Ref {
                self.own(V::val(item))
            } else {
                item
            };
            let raw = self.to_raw(kept, &el);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Rt(list_rt(&el, RtFn::ListPush)),
                args: vec![Operand::Local(out), raw],
            });
            self.term(Terminator::Goto(skip));
            self.switch(skip);
        } else {
            // The call produced the only reference to this value, and the list
            // takes it over, so there is nothing to retain or release here.
            let d = dst.expect("map returns a value");
            let raw = self.to_raw(Operand::Local(d), &out_el);
            self.emit(Inst::Call {
                dst: None,
                callee: Callee::Rt(list_rt(&out_el, RtFn::ListPush)),
                args: vec![Operand::Local(out), raw],
            });
        }
        self.emit(Inst::Assign {
            dst: i,
            rv: Rvalue::Binary(BinOp::Add, Operand::Local(i), Operand::Int(1)),
        });
        self.term(Terminator::Goto(head));

        self.switch(done);
        for local in [src, fun] {
            self.emit(Inst::Release(Operand::Local(local)));
            self.emit(Inst::Assign {
                dst: local,
                rv: Rvalue::Use(Operand::NullRef),
            });
        }
        result
    }

    pub(crate) fn index(&mut self, obj: &Expr, index: &Expr, e: &Expr) -> V {
        let ot = self.expr_type(obj);
        let res_ty = self.expr_type(e);
        let ov = self.expr(obj);
        let iv = self.expr(index);
        let line = Operand::Int(self.line_of(e.span.start));
        // `json["key"]` and `json[i]`: the runtime returns a value owned by the parent.
        if matches!(ot, Type::Class(name) if name == "Json") {
            let (sym, params) = match self.expr_type(index) {
                Type::Str => ("kode_json_index_key", vec![Ty::Ref, Ty::Ref, Ty::Int]),
                Type::Int => ("kode_json_index_at", vec![Ty::Ref, Ty::Int, Ty::Int]),
                _ => unreachable!("checked: a Json index is a string or an int"),
            };
            let sig = self.std_sig(sym, params, Ty::Ref);
            let t = self.new_temp(Ty::Ref);
            self.emit(Inst::Call {
                dst: Some(t),
                callee: Callee::Std(sig),
                args: vec![ov.op.clone(), iv.op.clone(), line],
            });
            self.error_check();
            self.release_if_owned(iv);
            return self.borrow_result(ov, Operand::Local(t), res_ty);
        }
        let rt = match ot {
            Type::Map(_) => RtFn::MapGet,
            Type::List(el) => list_rt(el, RtFn::ListGet),
            _ => unreachable!("checked: a list or map"),
        };
        let result = self.call_raw(rt, vec![ov.op.clone(), iv.op.clone(), line], res_ty);
        self.release_if_owned(iv);
        self.borrow_result(ov, result, res_ty)
    }

    pub(crate) fn member(&mut self, obj: &Expr, name: &str, safe: bool) -> V {
        // `util.TAX`: a public constant from an imported module.
        if let ExprKind::Ident(n) = &obj.kind {
            if self.lookup(n).is_none()
                && !self.checked.consts.contains_key(n)
                && self.this_field(n).is_none()
            {
                if let Some(mi) = self.checked.module_index.get(n) {
                    if self.checked.modules[*mi].exports.iter().any(|e| e == name) {
                        if let Some(op) = self.const_operand(name) {
                            return V::val(op);
                        }
                    }
                }
            }
        }
        // `math.pi` and other module constants.
        if let ExprKind::Ident(n) = &obj.kind {
            if self.lookup(n).is_none()
                && !self.checked.consts.contains_key(n)
                && self.this_field(n).is_none()
            {
                if let Some(v) = kode_sema::modules::constant(n, name) {
                    return V::val(Operand::Float(v));
                }
            }
        }
        // `Color.Red`
        if let ExprKind::Ident(n) = &obj.kind {
            if self.lookup(n).is_none()
                && !self.checked.consts.contains_key(n)
                && self.this_field(n).is_none()
            {
                if let Some(&ei) = self.checked.enum_index.get(n) {
                    let tag = self.checked.enums[ei].variants.iter().position(|v| v == name).unwrap();
                    return V::val(Operand::Int(tag as i64));
                }
            }
        }
        let Type::Class(cn) = strip_opt(self.expr_type(obj)) else { unreachable!("checked: a class value") };
        let class = &self.checked.classes[self.checked.class_index[cn]];
        let fi = class.fields.iter().position(|f| f.name == name).unwrap();
        let fty = &class.fields[fi].ty;
        let offset = 24 + 8 * fi as i32;
        let ov = self.expr(obj);

        if !safe {
            let dst = self.new_temp(mir_ty(fty));
            self.emit(Inst::Load {
                dst,
                base: ov.op.clone(),
                offset,
                ty: mir_ty(fty),
            });
            return self.borrow_result(ov, Operand::Local(dst), fty);
        }

        // `obj?.field`: none when `obj` is none.
        let t = self.new_temp(Ty::Ref);
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Use(Operand::NullRef),
        });
        let isnull = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: isnull,
            rv: Rvalue::Binary(BinOp::Eq, ov.op.clone(), Operand::NullRef),
        });
        let load_bb = self.new_block();
        let join = self.new_block();
        self.term(Terminator::Branch {
            cond: Operand::Local(isnull),
            then_bb: join,
            else_bb: load_bb,
        });
        self.switch(load_bb);
        self.emit(Inst::Load {
            dst: t,
            base: ov.op.clone(),
            offset,
            ty: Ty::Ref,
        });
        self.emit(Inst::Retain(Operand::Local(t)));
        self.term(Terminator::Goto(join));
        self.switch(join);
        self.release_if_owned(ov);
        self.owned(Operand::Local(t))
    }

    pub(crate) fn method_call(&mut self, e: &Expr, obj: &Expr, name: &str, args: &[Arg]) -> V {
        // `util.add(...)`: a public function or class from an imported module.
        let module_method = match &obj.kind {
            ExprKind::Ident(n) => {
                self.lookup(n).is_none()
                    && !self.checked.consts.contains_key(n)
                    && self.this_field(n).is_none()
                    && self.checked.module_index.get(n).is_some_and(|&mi| {
                        self.checked.modules[mi].exports.iter().any(|x| x == name)
                    })
            }
            _ => false,
        };
        if module_method {
            return self.call(e, name, args);
        }
        // `Response.json(value)` / `Response.text(text)`
        if let ExprKind::Ident(n) = &obj.kind {
            if n == "Response"
                && self.lookup(n).is_none()
                && !self.checked.consts.contains_key(n)
                && self.this_field(n).is_none()
            {
                let f = match name {
                    "json" => kode_sema::std_fns::find_free("response_json"),
                    "text" => kode_sema::std_fns::find_free("response_text"),
                    _ => None,
                }
                .expect("the type checker validated the static method");
                return self.std_call(f, None, args);
            }
        }
        // `json.parse(...)`, `math.sqrt(...)`: standard-library namespaces.
        if let ExprKind::Ident(n) = &obj.kind {
            if self.lookup(n).is_none()
                && !self.checked.consts.contains_key(n)
                && self.this_field(n).is_none()
            {
                if let Some(f) = kode_sema::modules::function(n, name) {
                    return self.std_call(f, None, args);
                }
            }
        }
        if matches!(self.expr_type(obj), Type::List(_)) && (name == "map" || name == "filter") {
            return self.list_map_filter(e, obj, name, &args[0].value);
        }
        let ot = self.expr_type(obj);
        if let Type::Interface(iname) = ot {
            let ii = self.checked.interface_index[iname];
            let sig = self.checked.interfaces[ii]
                .methods
                .iter()
                .find(|m| m.name == name)
                .expect("the type checker checked the method")
                .clone();
            let idx = *self
                .iface_dispatch
                .get(&(iname.clone(), name.to_string()))
                .expect("dispatch was generated");
            let ov = self.expr(obj);
            let vals = self.call_vals(e.id, args, Some(ov), sig.params.len());
            let ret = mir_ty(&sig.ret);
            let ops: Vec<Operand> = vals.iter().map(|v| v.op.clone()).collect();
            let dst = (ret != Ty::Unit).then(|| self.new_temp(ret));
            self.emit(Inst::Call {
                dst,
                callee: Callee::User(idx),
                args: ops,
            });
            self.error_check();
            for v in vals {
                self.release_if_owned(v);
            }
            return match dst {
                Some(d) if ret == Ty::Ref => self.owned(Operand::Local(d)),
                Some(d) => V::val(Operand::Local(d)),
                None => V::unit(),
            };
        }
        if matches!(ot, Type::Class(n) if n == "Router") {
            return self.router_add(obj, name, args);
        }
        if let Some(f) = kode_sema::std_fns::find_method(ot, name) {
            return self.std_call(f, Some(obj), args);
        }
        let ov = self.expr(obj);
        match (ot, name) {
            (Type::Str, "len") | (Type::List(_), "len") | (Type::Map(_), "len") => {
                let rt = match ot {
                    Type::Str => RtFn::StrLen,
                    Type::List(_) => RtFn::ListLen,
                    _ => RtFn::MapLen,
                };
                let t = self.new_temp(Ty::Int);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(rt),
                    args: vec![ov.op.clone()],
                });
                self.release_if_owned(ov);
                V::val(Operand::Local(t))
            }
            (Type::List(el), "add") => {
                let v = self.expr(&args[0].value);
                let op = if mir_ty(el) == Ty::Ref { self.own(v) } else { v.op };
                let raw = self.to_raw(op, el);
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(list_rt(el, RtFn::ListPush)),
                    args: vec![ov.op.clone(), raw],
                });
                self.release_if_owned(ov);
                V::unit()
            }
            (Type::Map(_), "has") => {
                let k = self.expr(&args[0].value);
                let t = self.new_temp(Ty::Bool);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(RtFn::MapHas),
                    args: vec![ov.op.clone(), k.op.clone()],
                });
                self.release_if_owned(k);
                self.release_if_owned(ov);
                V::val(Operand::Local(t))
            }
            (Type::Map(_), "keys") => {
                let t = self.new_temp(Ty::Ref);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(RtFn::MapKeys),
                    args: vec![ov.op.clone()],
                });
                self.release_if_owned(ov);
                self.owned(Operand::Local(t))
            }
            (Type::Class(cn), m) => {
                let checked = self.checked;
                let mi = checked.classes[checked.class_index[cn]].methods[m];
                let n = checked.fns[mi].params.len() - 1;
                let vals = self.call_vals(e.id, args, Some(ov), n);
                self.emit_call(mi, vals)
            }
            _ => unreachable!("the type checker rejects this"),
        }
    }

    /// `router.get("/users/:id", handler)`
    fn router_add(&mut self, obj: &Expr, name: &str, args: &[Arg]) -> V {
        let checked = self.checked;
        let ExprKind::Ident(handler) = &args[1].value.kind else {
            unreachable!("the type checker made sure the handler is a function name")
        };
        let idx = checked.fn_index[handler];
        let method = match name {
            "get" => "GET",
            "post" => "POST",
            "put" => "PUT",
            "patch" => "PATCH",
            "delete" => "DELETE",
            _ => unreachable!("the type checker checked the method"),
        };
        let router = self.expr(obj);
        let path = self.expr(&args[0].value);
        let m = Operand::Str(self.intern(method));
        let sig = self.std_sig("kode_router_add", vec![Ty::Ref, Ty::Ref, Ty::Ref, Ty::Int], Ty::Unit);
        self.emit(Inst::Call {
            dst: None,
            callee: Callee::Std(sig),
            args: vec![router.op.clone(), m, path.op.clone(), Operand::FuncAddr(idx)],
        });
        self.release_if_owned(path);
        self.release_if_owned(router);
        V::unit()
    }

    /// `ClassName(field: value, ...)`
    pub(crate) fn construct(&mut self, e: &Expr, ci: usize, args: &[Arg]) -> V {
        let checked = self.checked;
        let fields = &checked.classes[ci].fields;
        let mask: i64 = fields
            .iter()
            .enumerate()
            .filter(|(_, f)| mir_ty(&f.ty) == Ty::Ref)
            .fold(0, |m, (i, _)| m | (1 << i));
        let p = self.new_temp(Ty::Ref);
        self.emit(Inst::Call {
            dst: Some(p),
            callee: Callee::Rt(RtFn::ObjNewClass),
            args: vec![Operand::Int(fields.len() as i64), Operand::Int(mask), Operand::Int(ci as i64)],
        });
        let obj = self.owned(Operand::Local(p));
        let slots = checked.call_args[&e.id].clone();
        for (i, f) in fields.iter().enumerate() {
            let v = match slots[i] {
                Some(ai) => self.expr(&args[ai].value),
                None => self.expr(f.default.as_ref().expect("the type checker made sure a default exists")),
            };
            let op = if mir_ty(&f.ty) == Ty::Ref { self.own(v) } else { v.op };
            self.emit(Inst::Store {
                base: Operand::Local(p),
                offset: 24 + 8 * i as i32,
                value: op,
            });
        }
        obj
    }

    /// `expr catch err { ... }`
    pub(crate) fn catch_expr(&mut self, inner: &Expr, var: &str, block: &[Stmt]) -> V {
        let ty = self.ety(inner);
        let res = (ty != Ty::Unit).then(|| self.new_temp(ty));
        let handler = self.new_block();
        let join = self.new_block();

        self.push_handler(handler);
        let v = self.expr(inner);
        self.handlers.pop();
        match res {
            Some(r) => {
                let op = if ty == Ty::Ref { self.own(v) } else { v.op };
                self.emit(Inst::Assign {
                    dst: r,
                    rv: Rvalue::Use(op),
                });
            }
            None => self.release_if_owned(v),
        }
        self.term(Terminator::Goto(join));

        self.switch(handler);
        self.push_scope();
        let ev = self.declare_var(var, Ty::Ref);
        self.emit(Inst::Release(Operand::Local(ev)));
        self.emit(Inst::Call {
            dst: Some(ev),
            callee: Callee::Rt(RtFn::ErrTake),
            args: vec![],
        });
        let n = block.len();
        for (i, s) in block.iter().enumerate() {
            if i + 1 == n && res.is_some() {
                if let Stmt::Expr(x) = s {
                    let v = self.expr(x);
                    let op = if ty == Ty::Ref { self.own(v) } else { v.op };
                    self.emit(Inst::Assign {
                        dst: res.unwrap(),
                        rv: Rvalue::Use(op),
                    });
                    continue;
                }
            }
            self.stmt(s);
        }
        self.pop_scope();
        self.term(Terminator::Goto(join));
        self.switch(join);
        match res {
            Some(r) if ty == Ty::Ref => self.owned(Operand::Local(r)),
            Some(r) => V::val(Operand::Local(r)),
            None => V::unit(),
        }
    }
}
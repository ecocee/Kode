use kode_sema::builtins::{self, Builtin};
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
                };
                self.emit(Inst::Assign {
                    dst: t,
                    rv: Rvalue::Unary(op, v.op),
                });
                V::val(Operand::Local(t))
            }
            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, e),
            ExprKind::Coalesce { lhs, rhs } => self.coalesce(lhs, rhs),
            ExprKind::Unwrap(inner) => self.unwrap(inner, e),
            ExprKind::List(items) => self.list_lit(items, e),
            ExprKind::Map(pairs) => self.map_lit(pairs, e),
            ExprKind::Index { obj, index } => self.index(obj, index, e),
            ExprKind::Member { obj, name, safe, .. } => self.member(obj, name, *safe),
            ExprKind::MethodCall { obj, name, args, .. } => self.method_call(e, obj, name, args),
            ExprKind::Call { callee, args, .. } => self.call(e, callee, args),
            ExprKind::Catch { expr, var, block, .. } => self.catch_expr(expr, var, block),
        }
    }

    fn ident(&mut self, name: &str) -> V {
        if let Some(id) = self.lookup(name) {
            return V::val(Operand::Local(id));
        }
        if let Some(op) = self.const_operand(name) {
            return V::val(op);
        }
        let (offset, ty) = self.this_field(name).expect("the type checker resolved this name");
        let this = Operand::Local(self.this_local.expect("a field is only visible inside a method"));
        let dst = self.new_temp(mir_ty(&ty));
        self.emit(Inst::Load {
            dst,
            base: this,
            offset,
            ty: mir_ty(&ty),
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

    fn binary(&mut self, op: ast::BinOp, lhs: &Expr, rhs: &Expr, e: &Expr) -> V {
        if matches!(op, ast::BinOp::And | ast::BinOp::Or) {
            return self.short_circuit(op, lhs, rhs);
        }
        let lt = self.expr_type(lhs);
        let l = self.expr(lhs);
        let r = self.expr(rhs);

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
        let (then_bb, else_bb) = if op == ast::BinOp::And { (rhs_bb, join) } else { (join, rhs_bb) };
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
    fn coalesce(&mut self, lhs: &Expr, rhs: &Expr) -> V {
        let l = self.expr(lhs);
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

    /// Turn any printable value into a string value.
    fn to_str(&mut self, v: V, ty: &Type) -> V {
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
                    self.to_str(v, ty)
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
        let ret = mir_ty(&self.checked.fns[idx].ret);
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
    pub(crate) fn call_vals(&mut self, id: ast::ExprId, args: &[Arg], first: Option<V>, count: usize) -> Vec<V> {
        let slots = self.checked.call_args[&id].clone();
        let mut vals: Vec<V> = first.into_iter().collect();
        for i in 0..count {
            let ai = slots[i].expect("the type checker made sure every argument is given");
            vals.push(self.expr(&args[ai].value));
        }
        vals
    }

    fn call(&mut self, e: &Expr, callee: &str, args: &[Arg]) -> V {
        if let Some(b) = builtins::lookup(callee) {
            return self.builtin(b, args, e);
        }
        let checked = self.checked;
        if let Some(&idx) = checked.fn_index.get(callee) {
            let n = checked.fns[idx].params.len();
            let vals = self.call_vals(e.id, args, None, n);
            return self.emit_call(idx, vals);
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
                self.to_str(v, ty)
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
        }
    }
}
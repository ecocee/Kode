use kode_sema::Type;
use kode_syntax::ast::{Arm, Expr, ExprKind, Pattern, Stmt, Target};

use super::{mir_ty, Lowerer, V};
use crate::ir::*;

impl<'a> Lowerer<'a> {
    fn block(&mut self, stmts: &[Stmt]) {
        self.push_scope();
        for s in stmts {
            self.stmt(s);
        }
        self.pop_scope();
    }

    /// Store `v` into a local variable. Heap values take ownership and release the old one.
    fn store_local(&mut self, target: LocalId, v: V, ty: Ty) {
        if ty == Ty::Ref {
            let op = self.own(v);
            self.emit(Inst::Release(Operand::Local(target)));
            self.emit(Inst::Assign {
                dst: target,
                rv: Rvalue::Use(op),
            });
        } else {
            self.emit(Inst::Assign {
                dst: target,
                rv: Rvalue::Use(v.op),
            });
        }
    }

    fn store_field(&mut self, base: Operand, offset: i32, ty: Ty, v: V) {
        if ty == Ty::Ref {
            let new = self.own(v);
            let old = self.new_temp(Ty::Ref);
            self.emit(Inst::Load {
                dst: old,
                base: base.clone(),
                offset,
                ty: Ty::Ref,
            });
            self.emit(Inst::Store { base, offset, value: new });
            self.emit(Inst::Release(Operand::Local(old)));
        } else {
            self.emit(Inst::Store {
                base,
                offset,
                value: v.op,
            });
        }
    }

    pub(crate) fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                let v = self.expr(e);
                self.release_if_owned(v);
            }
            Stmt::Assign { target, value, .. } => self.assign(target, value),
            Stmt::If { cond, then, else_ } => {
                let c = self.expr(cond);
                let then_bb = self.new_block();
                let join = self.new_block();
                let else_bb = if else_.is_some() { self.new_block() } else { join };
                self.term(Terminator::Branch {
                    cond: c.op,
                    then_bb,
                    else_bb,
                });
                self.switch(then_bb);
                self.block(then);
                self.term(Terminator::Goto(join));
                if let Some(e) = else_ {
                    self.switch(else_bb);
                    self.block(e);
                    self.term(Terminator::Goto(join));
                }
                self.switch(join);
            }
            Stmt::While { cond, body } => {
                let header = self.new_block();
                let body_bb = self.new_block();
                let exit = self.new_block();
                self.term(Terminator::Goto(header));
                self.switch(header);
                let c = self.expr(cond);
                self.term(Terminator::Branch {
                    cond: c.op,
                    then_bb: body_bb,
                    else_bb: exit,
                });
                self.switch(body_bb);
                self.loops.push((header, exit));
                self.block(body);
                self.loops.pop();
                self.term(Terminator::Goto(header));
                self.switch(exit);
            }
            Stmt::For {
                var,
                start,
                end,
                inclusive,
                body,
                ..
            } => self.lower_for(var, start, end, *inclusive, body),
            Stmt::ForIn { vars, iter, body } => self.lower_for_in(vars, iter, body),
            Stmt::Match { scrutinee, arms, .. } => self.lower_match(scrutinee, arms),
            Stmt::Throw { value, .. } => {
                let ty = self.expr_type(value);
                let v = self.expr(value);
                let obj = if *ty == Type::Str {
                    let msg = self.own(v);
                    self.make_error(msg)
                } else {
                    self.own(v)
                };
                self.raise(obj);
                self.start_dead_block();
            }
            Stmt::Try {
                body,
                var,
                handler,
                ..
            } => {
                let handler_bb = self.new_block();
                let join = self.new_block();
                self.push_handler(handler_bb);
                self.block(body);
                self.handlers.pop();
                self.term(Terminator::Goto(join));

                self.switch(handler_bb);
                self.push_scope();
                let ev = self.declare_var(var, Ty::Ref);
                self.emit(Inst::Release(Operand::Local(ev)));
                self.emit(Inst::Call {
                    dst: Some(ev),
                    callee: Callee::Rt(RtFn::ErrTake),
                    args: vec![],
                });
                for s in handler {
                    self.stmt(s);
                }
                self.pop_scope();
                self.term(Terminator::Goto(join));
                self.switch(join);
            }
            Stmt::Break(_) => {
                let (_, exit) = *self.loops.last().expect("checked: inside a loop");
                self.term(Terminator::Goto(exit));
                self.start_dead_block();
            }
            Stmt::Continue(_) => {
                let (cont, _) = *self.loops.last().expect("checked: inside a loop");
                self.term(Terminator::Goto(cont));
                self.start_dead_block();
            }
            Stmt::Return { value, .. } => {
                match value {
                    Some(e) => {
                        let v = self.expr(e);
                        let ret = self.ret;
                        self.return_value(v, ret);
                    }
                    None => self.goto_exit(),
                }
                self.start_dead_block();
            }
        }
    }

    fn assign(&mut self, target: &Target, value: &Expr) {
        match target {
            Target::Var(name, _) => {
                let ty = mir_ty(self.expr_type(value));
                let v = self.expr(value);
                if let Some(id) = self.lookup(name) {
                    self.store_local(id, v, ty);
                } else if let Some((offset, fty)) = self.this_field(name) {
                    let this = Operand::Local(self.this_local.unwrap());
                    self.store_field(this, offset, mir_ty(&fty), v);
                } else {
                    let id = self.declare_var(name, ty);
                    self.store_local(id, v, ty);
                }
            }
            Target::Index { obj, index } => {
                let ot = self.expr_type(obj);
                let ov = self.expr(obj);
                let iv = self.expr(index);
                let v = self.expr(value);
                let line = Operand::Int(self.line_of(value.span.start));
                let (el, rt) = match ot {
                    Type::Map(el) => (el, RtFn::MapSet),
                    Type::List(el) => (el, RtFn::ListSet),
                    _ => unreachable!("checked: a list or map"),
                };
                let op = if mir_ty(el) == Ty::Ref { self.own(v) } else { v.op };
                let raw = self.to_raw(op, el);
                let args = if rt == RtFn::MapSet {
                    vec![ov.op.clone(), iv.op.clone(), raw]
                } else {
                    vec![ov.op.clone(), iv.op.clone(), raw, line]
                };
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(rt),
                    args,
                });
                self.release_if_owned(iv);
                self.release_if_owned(ov);
            }
            Target::Field { obj, name, .. } => {
                let ot = self.expr_type(obj);
                let Type::Class(cn) = ot else { unreachable!("checked: a class value") };
                let class = &self.checked.classes[self.checked.class_index[cn]];
                let fi = class.fields.iter().position(|f| f.name == *name).unwrap();
                let fty = mir_ty(&class.fields[fi].ty);
                let ov = self.expr(obj);
                let v = self.expr(value);
                self.store_field(ov.op.clone(), 24 + 8 * fi as i32, fty, v);
                self.release_if_owned(ov);
            }
        }
    }

    fn lower_for(&mut self, var: &str, start: &Expr, end: &Expr, inclusive: bool, body: &[Stmt]) {
        let s = self.expr(start);
        let en = self.expr(end);

        self.push_scope();
        let i = self.declare_var(var, Ty::Int);
        // The end of the range is evaluated once.
        let end_local = self.new_temp(Ty::Int);
        self.emit(Inst::Assign {
            dst: i,
            rv: Rvalue::Use(s.op),
        });
        self.emit(Inst::Assign {
            dst: end_local,
            rv: Rvalue::Use(en.op),
        });

        let header = self.new_block();
        let body_bb = self.new_block();
        let step = self.new_block();
        let exit = self.new_block();

        self.term(Terminator::Goto(header));
        self.switch(header);
        let cond = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: cond,
            rv: Rvalue::Binary(
                if inclusive { BinOp::Le } else { BinOp::Lt },
                Operand::Local(i),
                Operand::Local(end_local),
            ),
        });
        self.term(Terminator::Branch {
            cond: Operand::Local(cond),
            then_bb: body_bb,
            else_bb: exit,
        });

        self.switch(body_bb);
        self.loops.push((step, exit));
        self.block(body);
        self.loops.pop();
        self.term(Terminator::Goto(step));

        self.switch(step);
        self.emit(Inst::Assign {
            dst: i,
            rv: Rvalue::Binary(BinOp::Add, Operand::Local(i), Operand::Int(1)),
        });
        self.term(Terminator::Goto(header));

        self.switch(exit);
        self.pop_scope();
    }

    /// `for x in list`, `for i, x in list`, `for k, v in map`
    fn lower_for_in(&mut self, vars: &[(String, kode_diagnostics::Span)], iter: &Expr, body: &[Stmt]) {
        let it_ty = self.expr_type(iter);
        let is_map = matches!(it_ty, Type::Map(_));
        let iv = self.expr(iter);

        // Keep the collection alive for the whole loop.
        let holder = self.hidden_ref();
        let op = self.own(iv);
        self.emit(Inst::Release(Operand::Local(holder)));
        self.emit(Inst::Assign {
            dst: holder,
            rv: Rvalue::Use(op),
        });
        let h = Operand::Local(holder);

        let n = self.new_temp(Ty::Int);
        self.emit(Inst::Call {
            dst: Some(n),
            callee: Callee::Rt(if is_map { RtFn::MapLen } else { RtFn::ListLen }),
            args: vec![h.clone()],
        });
        let idx = self.new_temp(Ty::Int);
        self.emit(Inst::Assign {
            dst: idx,
            rv: Rvalue::Use(Operand::Int(0)),
        });

        let header = self.new_block();
        let body_bb = self.new_block();
        let step = self.new_block();
        let exit = self.new_block();
        self.term(Terminator::Goto(header));
        self.switch(header);
        let cond = self.new_temp(Ty::Bool);
        self.emit(Inst::Assign {
            dst: cond,
            rv: Rvalue::Binary(BinOp::Lt, Operand::Local(idx), Operand::Local(n)),
        });
        self.term(Terminator::Branch {
            cond: Operand::Local(cond),
            then_bb: body_bb,
            else_bb: exit,
        });

        self.switch(body_bb);
        self.push_scope();
        let line = Operand::Int(self.line_of(iter.span.start));
        match it_ty {
            Type::List(el) => {
                let item = self.call_raw(RtFn::ListGet, vec![h.clone(), Operand::Local(idx), line], el);
                if vars.len() == 2 {
                    let iv = self.declare_var(&vars[0].0, Ty::Int);
                    self.emit(Inst::Assign {
                        dst: iv,
                        rv: Rvalue::Use(Operand::Local(idx)),
                    });
                }
                let name = &vars[vars.len() - 1].0;
                let ety = mir_ty(el);
                let var = self.declare_var(name, ety);
                self.store_local(var, V::val(item), ety);
            }
            Type::Map(el) => {
                let key = self.new_temp(Ty::Ref);
                self.emit(Inst::Call {
                    dst: Some(key),
                    callee: Callee::Rt(RtFn::MapKeyAt),
                    args: vec![h.clone(), Operand::Local(idx)],
                });
                let kv = self.declare_var(&vars[0].0, Ty::Ref);
                self.store_local(kv, V::val(Operand::Local(key)), Ty::Ref);
                if vars.len() == 2 {
                    let val = self.call_raw(RtFn::MapValAt, vec![h.clone(), Operand::Local(idx)], el);
                    let ety = mir_ty(el);
                    let vv = self.declare_var(&vars[1].0, ety);
                    self.store_local(vv, V::val(val), ety);
                }
            }
            _ => unreachable!("checked: a list or map"),
        }
        self.loops.push((step, exit));
        for s in body {
            self.stmt(s);
        }
        self.loops.pop();
        self.pop_scope();
        self.term(Terminator::Goto(step));

        self.switch(step);
        self.emit(Inst::Assign {
            dst: idx,
            rv: Rvalue::Binary(BinOp::Add, Operand::Local(idx), Operand::Int(1)),
        });
        self.term(Terminator::Goto(header));
        self.switch(exit);
    }

    fn lower_match(&mut self, scrutinee: &Expr, arms: &[Arm]) {
        let st = self.expr_type(scrutinee);
        let sv = self.expr(scrutinee);
        // A string scrutinee is kept alive in a hidden variable.
        let sop = if *st == Type::Str {
            let holder = self.hidden_ref();
            let op = self.own(sv);
            self.emit(Inst::Release(Operand::Local(holder)));
            self.emit(Inst::Assign {
                dst: holder,
                rv: Rvalue::Use(op),
            });
            Operand::Local(holder)
        } else {
            sv.op
        };

        let join = self.new_block();
        for arm in arms {
            match &arm.pattern {
                Pattern::Wildcard => {
                    self.block(&arm.body);
                    self.term(Terminator::Goto(join));
                    self.switch(join);
                    return;
                }
                Pattern::Value(p) => {
                    let pv = self.expr(p);
                    let cond = self.new_temp(Ty::Bool);
                    if *st == Type::Str {
                        self.emit(Inst::Call {
                            dst: Some(cond),
                            callee: Callee::Rt(RtFn::StrEq),
                            args: vec![sop.clone(), pv.op.clone()],
                        });
                    } else {
                        self.emit(Inst::Assign {
                            dst: cond,
                            rv: Rvalue::Binary(BinOp::Eq, sop.clone(), pv.op.clone()),
                        });
                    }
                    self.release_if_owned(pv);
                    let arm_bb = self.new_block();
                    let next = self.new_block();
                    self.term(Terminator::Branch {
                        cond: Operand::Local(cond),
                        then_bb: arm_bb,
                        else_bb: next,
                    });
                    self.switch(arm_bb);
                    self.block(&arm.body);
                    self.term(Terminator::Goto(join));
                    self.switch(next);
                }
            }
        }
        // No arm matched (the checker made sure this cannot happen for enums and bools).
        self.term(Terminator::Goto(join));
        self.switch(join);
    }
}

#[allow(dead_code)]
fn _unused(_: &ExprKind) {}
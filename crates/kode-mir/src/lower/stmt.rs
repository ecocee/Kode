use kode_syntax::ast::{Expr, Stmt};

use super::Lowerer;
use crate::ir::*;

impl<'a> Lowerer<'a> {
    fn block(&mut self, stmts: &[Stmt]) {
        self.push_scope();
        for s in stmts {
            self.stmt(s);
        }
        self.pop_scope();
    }

    pub(crate) fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                let v = self.expr(e);
                self.release_if_owned(v);
            }
            Stmt::Assign { name, value, .. } => {
                let ty = self.expr_ty(value);
                let v = self.expr(value);
                let target = match self.lookup(name) {
                    Some(id) => id,
                    None => self.declare_var(name, ty),
                };
                if ty == Ty::Str {
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
                let op = value.as_ref().map(|e| {
                    let ty = self.expr_ty(e);
                    let v = self.expr(e);
                    if ty == Ty::Str {
                        self.own(v)
                    } else {
                        v.op
                    }
                });
                self.release_locals();
                self.term(Terminator::Return(op));
                self.start_dead_block();
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
}

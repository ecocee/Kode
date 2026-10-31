use kode_sema::builtins::{self, Builtin};
use kode_syntax::ast::{self, Expr, ExprKind, InterpPart};

use super::{Lowerer, V};
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
        ast::BinOp::And | ast::BinOp::Or => unreachable!("lowered to control flow"),
    }
}

impl<'a> Lowerer<'a> {
    pub(crate) fn expr(&mut self, e: &Expr) -> V {
        match &e.kind {
            ExprKind::Int(v) => V::val(Operand::Int(*v)),
            ExprKind::Float(v) => V::val(Operand::Float(*v)),
            ExprKind::Bool(v) => V::val(Operand::Bool(*v)),
            ExprKind::Str(s) => V::val(Operand::Str(self.intern(s))),
            ExprKind::Interp(parts) => self.interp(parts),
            ExprKind::Ident(name) => {
                if let Some(id) = self.lookup(name) {
                    V::val(Operand::Local(id))
                } else {
                    let op = self
                        .const_operand(name)
                        .expect("the type checker resolved this name");
                    V::val(op)
                }
            }
            ExprKind::Unary { op, expr } => {
                let v = self.expr(expr);
                let ty = self.expr_ty(e);
                let t = self.new_temp(ty);
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
            ExprKind::Call { callee, args, .. } => self.call(callee, args, e),
        }
    }

    fn binary(&mut self, op: ast::BinOp, lhs: &Expr, rhs: &Expr, e: &Expr) -> V {
        if matches!(op, ast::BinOp::And | ast::BinOp::Or) {
            return self.short_circuit(op, lhs, rhs);
        }
        let operand_ty = self.expr_ty(lhs);
        let l = self.expr(lhs);
        let r = self.expr(rhs);

        if operand_ty == Ty::Str {
            return match op {
                ast::BinOp::Add => {
                    let t = self.new_temp(Ty::Str);
                    self.emit(Inst::Call {
                        dst: Some(t),
                        callee: Callee::Rt(RtFn::StrConcat),
                        args: vec![l.op.clone(), r.op.clone()],
                    });
                    self.release_if_owned(l);
                    self.release_if_owned(r);
                    V::owned(Operand::Local(t))
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

        let t = self.new_temp(self.expr_ty(e));
        self.emit(Inst::Assign {
            dst: t,
            rv: Rvalue::Binary(mir_binop(op), l.op, r.op),
        });
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

    /// Turn any printable value into a string value.
    fn to_str(&mut self, v: V, ty: Ty) -> V {
        let rt = match ty {
            Ty::Str => return v,
            Ty::Int => RtFn::IntToStr,
            Ty::Float => RtFn::FloatToStr,
            Ty::Bool => RtFn::BoolToStr,
            Ty::Unit => unreachable!("the type checker rejects this"),
        };
        let t = self.new_temp(Ty::Str);
        self.emit(Inst::Call {
            dst: Some(t),
            callee: Callee::Rt(rt),
            args: vec![v.op],
        });
        V::owned(Operand::Local(t))
    }

    fn interp(&mut self, parts: &[InterpPart]) -> V {
        let mut acc: Option<V> = None;
        for part in parts {
            let piece = match part {
                InterpPart::Lit(s) => V::val(Operand::Str(self.intern(s))),
                InterpPart::Expr(x) => {
                    let ty = self.expr_ty(x);
                    let v = self.expr(x);
                    self.to_str(v, ty)
                }
            };
            acc = Some(match acc {
                None => {
                    let op = self.own(piece);
                    V::owned(op)
                }
                Some(a) => {
                    let t = self.new_temp(Ty::Str);
                    self.emit(Inst::Call {
                        dst: Some(t),
                        callee: Callee::Rt(RtFn::StrConcat),
                        args: vec![a.op.clone(), piece.op.clone()],
                    });
                    self.release_if_owned(a);
                    self.release_if_owned(piece);
                    V::owned(Operand::Local(t))
                }
            });
        }
        acc.unwrap_or_else(|| V::val(Operand::Str(self.intern(""))))
    }

    fn call(&mut self, callee: &str, args: &[Expr], e: &Expr) -> V {
        if let Some(b) = builtins::lookup(callee) {
            return self.builtin(b, args, e);
        }
        let idx = self.checked.fn_index[callee];
        let ret = super::to_ty(self.checked.fns[idx].ret);
        let vals: Vec<V> = args.iter().map(|a| self.expr(a)).collect();
        let ops: Vec<Operand> = vals.iter().map(|v| v.op.clone()).collect();
        let dst = (ret != Ty::Unit).then(|| self.new_temp(ret));
        self.emit(Inst::Call {
            dst,
            callee: Callee::User(idx),
            args: ops,
        });
        for v in vals {
            self.release_if_owned(v);
        }
        match dst {
            Some(d) if ret == Ty::Str => V::owned(Operand::Local(d)),
            Some(d) => V::val(Operand::Local(d)),
            None => V::unit(),
        }
    }

    fn builtin(&mut self, b: Builtin, args: &[Expr], e: &Expr) -> V {
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
                    let ty = self.expr_ty(a);
                    let v = self.expr(a);
                    let rt = match ty {
                        Ty::Int => RtFn::PrintInt,
                        Ty::Float => RtFn::PrintFloat,
                        Ty::Bool => RtFn::PrintBool,
                        Ty::Str => RtFn::PrintStr,
                        Ty::Unit => unreachable!("the type checker rejects this"),
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
                let v = self.expr(&args[0]);
                let t = self.new_temp(Ty::Int);
                self.emit(Inst::Call {
                    dst: Some(t),
                    callee: Callee::Rt(RtFn::StrLen),
                    args: vec![v.op.clone()],
                });
                self.release_if_owned(v);
                V::val(Operand::Local(t))
            }
            Builtin::Int => {
                let from = self.expr_ty(&args[0]);
                let v = self.expr(&args[0]);
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
                let from = self.expr_ty(&args[0]);
                let v = self.expr(&args[0]);
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
                let ty = self.expr_ty(&args[0]);
                let v = self.expr(&args[0]);
                self.to_str(v, ty)
            }
            Builtin::Assert => {
                let c = self.expr(&args[0]);
                let ok = self.new_block();
                let fail = self.new_block();
                self.term(Terminator::Branch {
                    cond: c.op,
                    then_bb: ok,
                    else_bb: fail,
                });
                self.switch(fail);
                let line = self.line_of(e.span.start);
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(RtFn::AssertFail),
                    args: vec![Operand::Int(line)],
                });
                self.term(Terminator::Unreachable);
                self.switch(ok);
                V::unit()
            }
        }
    }
}

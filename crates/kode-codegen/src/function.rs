//! Translate one MIR function into Cranelift IR.

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::immediates::Ieee64;
use cranelift_codegen::ir::{self, types, InstBuilder, TrapCode, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataId, FuncId, Module};
use cranelift_object::ObjectModule;
use kode_mir::*;

use crate::clif_ty;
use crate::runtime::{self, Rt};

pub(crate) struct Env<'a> {
    pub module: &'a mut ObjectModule,
    pub rt: &'a Rt,
    pub user: &'a [FuncId],
    pub strings: &'a [DataId],
}

struct Gen<'a, 'b, 'c> {
    b: FunctionBuilder<'b>,
    f: &'a MirFunc,
    env: &'a mut Env<'c>,
    vars: Vec<Variable>,
    blocks: Vec<ir::Block>,
}

pub(crate) fn translate(
    f: &MirFunc,
    func: &mut ir::Function,
    fb_ctx: &mut FunctionBuilderContext,
    env: &mut Env,
) -> Result<(), String> {
    let mut b = FunctionBuilder::new(func, fb_ctx);
    let blocks: Vec<ir::Block> = (0..f.blocks.len()).map(|_| b.create_block()).collect();
    b.append_block_params_for_function_params(blocks[0]);
    let vars: Vec<Variable> = f.locals.iter().map(|l| b.declare_var(clif_ty(l.ty))).collect();
    b.switch_to_block(blocks[0]);
    let params: Vec<Value> = b.block_params(blocks[0]).to_vec();
    for (i, v) in params.into_iter().enumerate() {
        b.def_var(vars[i], v);
    }

    let mut g = Gen { b, f, env, vars, blocks };
    for (bi, blk) in f.blocks.iter().enumerate() {
        if bi != 0 {
            g.b.switch_to_block(g.blocks[bi]);
        }
        for inst in &blk.insts {
            g.inst(inst)?;
        }
        g.terminator(&blk.term);
    }
    g.b.seal_all_blocks();
    g.b.finalize();
    Ok(())
}

impl<'a, 'b, 'c> Gen<'a, 'b, 'c> {
    fn val(&mut self, op: &Operand) -> Value {
        match op {
            Operand::Local(i) => self.b.use_var(self.vars[*i]),
            Operand::Int(v) => self.b.ins().iconst(types::I64, *v),
            Operand::Float(v) => self.b.ins().f64const(Ieee64::with_float(*v)),
            Operand::Bool(v) => self.b.ins().iconst(types::I8, *v as i64),
            Operand::Str(i) => {
                let gv = self.env.module.declare_data_in_func(self.env.strings[*i], self.b.func);
                self.b.ins().global_value(types::I64, gv)
            }
            Operand::NullStr => self.b.ins().iconst(types::I64, 0),
        }
    }

    fn call_func(&mut self, id: FuncId, args: &[Value]) -> ir::Inst {
        let fref = self.env.module.declare_func_in_func(id, self.b.func);
        self.b.ins().call(fref, args)
    }

    fn inst(&mut self, inst: &Inst) -> Result<(), String> {
        match inst {
            Inst::Assign { dst, rv } => {
                let v = self.rvalue(rv)?;
                self.b.def_var(self.vars[*dst], v);
            }
            Inst::Call { dst, callee, args } => {
                let mut vals: Vec<Value> = args.iter().map(|a| self.val(a)).collect();
                let (id, is_str_eq) = match callee {
                    Callee::User(i) => (self.env.user[*i], false),
                    Callee::Rt(r) => {
                        // bools travel to the runtime as 64-bit integers
                        if matches!(r, RtFn::PrintBool | RtFn::BoolToStr) {
                            vals[0] = self.b.ins().uextend(types::I64, vals[0]);
                        }
                        (self.env.rt.funcs[r], *r == RtFn::StrEq)
                    }
                };
                if let Callee::Rt(r) = callee {
                    debug_assert_eq!(runtime::info(*r).1.len(), vals.len());
                }
                let call = self.call_func(id, &vals);
                if let Some(d) = dst {
                    let mut r = self.b.inst_results(call)[0];
                    if is_str_eq {
                        r = self.b.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    }
                    self.b.def_var(self.vars[*d], r);
                }
            }
            Inst::Retain(op) => {
                let v = self.val(op);
                self.call_func(self.env.rt.retain, &[v]);
            }
            Inst::Release(op) => {
                let v = self.val(op);
                self.call_func(self.env.rt.release, &[v]);
            }
        }
        Ok(())
    }

    fn rvalue(&mut self, rv: &Rvalue) -> Result<Value, String> {
        Ok(match rv {
            Rvalue::Use(a) => self.val(a),
            Rvalue::Unary(op, a) => {
                let ty = self.f.operand_ty(a);
                let v = self.val(a);
                match (op, ty) {
                    (UnOp::Neg, Ty::Int) => self.b.ins().ineg(v),
                    (UnOp::Neg, Ty::Float) => self.b.ins().fneg(v),
                    (UnOp::Not, Ty::Bool) => self.b.ins().icmp_imm(IntCC::Equal, v, 0),
                    _ => return Err(format!("bad unary operation on {ty:?}")),
                }
            }
            Rvalue::Cast(kind, a) => {
                let v = self.val(a);
                match kind {
                    CastKind::IntToFloat => self.b.ins().fcvt_from_sint(types::F64, v),
                    CastKind::FloatToInt => self.b.ins().fcvt_to_sint_sat(types::I64, v),
                    CastKind::BoolToInt => self.b.ins().uextend(types::I64, v),
                }
            }
            Rvalue::Binary(op, x, y) => {
                let ty = self.f.operand_ty(x);
                let (a, c) = (self.val(x), self.val(y));
                match ty {
                    Ty::Int => self.int_binary(*op, a, c),
                    Ty::Float => self.float_binary(*op, a, c)?,
                    Ty::Bool => match op {
                        BinOp::Eq => self.b.ins().icmp(IntCC::Equal, a, c),
                        BinOp::Ne => self.b.ins().icmp(IntCC::NotEqual, a, c),
                        _ => return Err("bad operation on bools".into()),
                    },
                    _ => return Err(format!("bad binary operation on {ty:?}")),
                }
            }
        })
    }

    fn int_binary(&mut self, op: BinOp, a: Value, c: Value) -> Value {
        let ins = |g: &mut Self, cc| g.b.ins().icmp(cc, a, c);
        match op {
            BinOp::Add => self.b.ins().iadd(a, c),
            BinOp::Sub => self.b.ins().isub(a, c),
            BinOp::Mul => self.b.ins().imul(a, c),
            BinOp::Div => self.divide(a, c, false),
            BinOp::Rem => self.divide(a, c, true),
            BinOp::Eq => ins(self, IntCC::Equal),
            BinOp::Ne => ins(self, IntCC::NotEqual),
            BinOp::Lt => ins(self, IntCC::SignedLessThan),
            BinOp::Le => ins(self, IntCC::SignedLessThanOrEqual),
            BinOp::Gt => ins(self, IntCC::SignedGreaterThan),
            BinOp::Ge => ins(self, IntCC::SignedGreaterThanOrEqual),
        }
    }

    fn float_binary(&mut self, op: BinOp, a: Value, c: Value) -> Result<Value, String> {
        let cmp = |g: &mut Self, cc| g.b.ins().fcmp(cc, a, c);
        Ok(match op {
            BinOp::Add => self.b.ins().fadd(a, c),
            BinOp::Sub => self.b.ins().fsub(a, c),
            BinOp::Mul => self.b.ins().fmul(a, c),
            BinOp::Div => self.b.ins().fdiv(a, c),
            BinOp::Eq => cmp(self, FloatCC::Equal),
            BinOp::Ne => cmp(self, FloatCC::NotEqual),
            BinOp::Lt => cmp(self, FloatCC::LessThan),
            BinOp::Le => cmp(self, FloatCC::LessThanOrEqual),
            BinOp::Gt => cmp(self, FloatCC::GreaterThan),
            BinOp::Ge => cmp(self, FloatCC::GreaterThanOrEqual),
            BinOp::Rem => return Err("`%` on floats is not supported".into()),
        })
    }

    /// Signed division or remainder that never crashes the program:
    /// a zero divisor stops with a clear message, and `MIN / -1` wraps.
    fn divide(&mut self, a: Value, c: Value, remainder: bool) -> Value {
        let trap = self.b.create_block();
        let ok = self.b.create_block();
        self.b.set_cold_block(trap);
        let is_zero = self.b.ins().icmp_imm(IntCC::Equal, c, 0);
        self.b.ins().brif(is_zero, trap, &[], ok, &[]);

        self.b.switch_to_block(trap);
        self.call_func(self.env.rt.div_zero, &[]);
        self.b.ins().trap(TrapCode::INTEGER_DIVISION_BY_ZERO);

        self.b.switch_to_block(ok);
        let is_m1 = self.b.ins().icmp_imm(IntCC::Equal, c, -1);
        let one = self.b.ins().iconst(types::I64, 1);
        let safe = self.b.ins().select(is_m1, one, c);
        if remainder {
            let r = self.b.ins().srem(a, safe);
            let zero = self.b.ins().iconst(types::I64, 0);
            self.b.ins().select(is_m1, zero, r)
        } else {
            let q = self.b.ins().sdiv(a, safe);
            let neg = self.b.ins().ineg(a);
            self.b.ins().select(is_m1, neg, q)
        }
    }

    fn terminator(&mut self, t: &Terminator) {
        match t {
            Terminator::Goto(bb) => {
                self.b.ins().jump(self.blocks[*bb], &[]);
            }
            Terminator::Branch { cond, then_bb, else_bb } => {
                let c = self.val(cond);
                self.b
                    .ins()
                    .brif(c, self.blocks[*then_bb], &[], self.blocks[*else_bb], &[]);
            }
            Terminator::Return(None) => {
                self.b.ins().return_(&[]);
            }
            Terminator::Return(Some(op)) => {
                let v = self.val(op);
                self.b.ins().return_(&[v]);
            }
            Terminator::Unreachable => {
                self.b.ins().trap(TrapCode::INTEGER_OVERFLOW);
            }
        }
    }
}

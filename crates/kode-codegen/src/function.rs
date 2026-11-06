//! Translate one MIR function into Cranelift IR.

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::immediates::Ieee64;
use cranelift_codegen::ir::{self, types, InstBuilder, MemFlags, TrapCode, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataId, FuncId, Module};
use cranelift_object::ObjectModule;
use kode_mir::*;
use kode_mir::StdSig;

use crate::clif_ty;
use crate::runtime::{self, Rt};

pub(crate) struct Env<'a> {
    pub module: &'a mut ObjectModule,
    pub rt: &'a Rt,
    pub user: &'a [FuncId],
    pub thunks: &'a [FuncId],
    /// Bodies of closure literals.
    pub closures: &'a [FuncId],
    /// Wrappers that adapt a named function to the closure calling convention.
    pub fnvals: &'a [FuncId],
    pub strings: &'a [DataId],
    pub std: &'a [FuncId],
    pub std_sigs: &'a [StdSig],
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
            Operand::NullRef => self.b.ins().iconst(types::I64, 0),
            Operand::Thunk(i) => {
                let r = self.env.module.declare_func_in_func(self.env.thunks[*i], self.b.func);
                self.b.ins().func_addr(types::I64, r)
            }
            Operand::FuncAddr(i) => {
                let r = self.env.module.declare_func_in_func(self.env.user[*i], self.b.func);
                self.b.ins().func_addr(types::I64, r)
            }
            Operand::ClosureFn(i) => {
                let r = self.env.module.declare_func_in_func(self.env.closures[*i], self.b.func);
                self.b.ins().func_addr(types::I64, r)
            }
            Operand::FnVal(i) => {
                let r = self.env.module.declare_func_in_func(self.env.fnvals[*i], self.b.func);
                self.b.ins().func_addr(types::I64, r)
            }
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
            Inst::Call { dst, callee, args } if self.try_inline_list(dst, callee, args) => {}
            Inst::Call { dst, callee, args } => {
                let mut vals: Vec<Value> = args.iter().map(|a| self.val(a)).collect();
                let (id, to_bool) = match callee {
                    Callee::User(i) => (self.env.user[*i], false),
                    Callee::Std(i) => {
                        // bools travel to the runtime as 64-bit integers
                        for (k, a) in args.iter().enumerate() {
                            if self.f.operand_ty(a) == Ty::Bool {
                                vals[k] = self.b.ins().uextend(types::I64, vals[k]);
                            }
                        }
                        (self.env.std[*i], self.env.std_sigs[*i].ret == Ty::Bool)
                    }
                    Callee::Rt(r) => {
                        // bools travel to the runtime as 64-bit integers
                        if matches!(r, RtFn::PrintBool | RtFn::BoolToStr) {
                            vals[0] = self.b.ins().uextend(types::I64, vals[0]);
                        }
                        (self.env.rt.funcs[r], runtime::returns_bool(*r))
                    }
                };
                if let Callee::Rt(r) = callee {
                    debug_assert_eq!(runtime::info(*r).1.len(), vals.len());
                }
                let call = self.call_func(id, &vals);
                if let Some(d) = dst {
                    let mut r = self.b.inst_results(call)[0];
                    if to_bool {
                        r = self.b.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    }
                    self.b.def_var(self.vars[*d], r);
                }
            }
            // A closure object is `[rc][kind][refmask][code][captures...]`: the code
            // address is field 0, and the object itself is passed as the first argument.
            Inst::CallIndirect { dst, callee, args } => {
                let clo = self.val(callee);
                let code = self.b.ins().load(types::I64, MemFlags::trusted(), clo, 24);
                let mut sig = self.env.module.make_signature();
                sig.params.push(ir::AbiParam::new(types::I64));
                let mut vals = vec![clo];
                for a in args {
                    sig.params.push(ir::AbiParam::new(clif_ty(self.f.operand_ty(a))));
                    vals.push(self.val(a));
                }
                let ret = match dst {
                    Some(d) => self.f.locals[*d].ty,
                    None => Ty::Unit,
                };
                if ret != Ty::Unit {
                    sig.returns.push(ir::AbiParam::new(clif_ty(ret)));
                }
                let sigref = self.b.import_signature(sig);
                let call = self.b.ins().call_indirect(sigref, code, &vals);
                if let Some(d) = dst {
                    let r = self.b.inst_results(call)[0];
                    self.b.def_var(self.vars[*d], r);
                }
            }
            Inst::Load { dst, base, offset, ty } => {
                let b = self.val(base);
                let v = self.b.ins().load(clif_ty(*ty), MemFlags::trusted(), b, *offset);
                self.b.def_var(self.vars[*dst], v);
            }
            Inst::Store { base, offset, value } => {
                let b = self.val(base);
                let v = self.val(value);
                self.b.ins().store(MemFlags::trusted(), v, b, *offset);
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

    /// Inline `len`, item reads and item writes of lists (no function call).
    /// Layout: `[rc][kind][len][capacity][data pointer]`, 8 bytes per item.
    fn try_inline_list(&mut self, dst: &Option<LocalId>, callee: &Callee, args: &[Operand]) -> bool {
        let Callee::Rt(rt) = callee else { return false };
        match rt {
            RtFn::ListLen => {
                let list = self.val(&args[0]);
                let len = self.b.ins().load(types::I64, MemFlags::trusted(), list, 16);
                self.b.def_var(self.vars[dst.unwrap()], len);
                true
            }
            RtFn::ListGet => {
                let list = self.val(&args[0]);
                let idx = self.val(&args[1]);
                let line = self.val(&args[2]);
                let addr = self.checked_item_addr(list, idx, line, 3);
                let v = self.b.ins().load(types::I64, MemFlags::trusted(), addr, 0);
                self.b.def_var(self.vars[dst.unwrap()], v);
                true
            }
            RtFn::ListGet8 => {
                let list = self.val(&args[0]);
                let idx = self.val(&args[1]);
                let line = self.val(&args[2]);
                let addr = self.checked_item_addr(list, idx, line, 0);
                let v = self.b.ins().uload8(types::I64, MemFlags::trusted(), addr, 0);
                self.b.def_var(self.vars[dst.unwrap()], v);
                true
            }
            RtFn::ListSet8 => {
                let list = self.val(&args[0]);
                let idx = self.val(&args[1]);
                let raw = self.val(&args[2]);
                let line = self.val(&args[3]);
                let addr = self.checked_item_addr(list, idx, line, 0);
                self.b.ins().istore8(MemFlags::trusted(), raw, addr, 0);
                true
            }
            RtFn::ListPush | RtFn::ListPush8 => {
                let packed = *rt == RtFn::ListPush8;
                let list = self.val(&args[0]);
                let raw = self.val(&args[1]);
                let len = self.b.ins().load(types::I64, MemFlags::trusted(), list, 16);
                let cap = self.b.ins().load(types::I64, MemFlags::trusted(), list, 24);
                let full = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, len, cap);
                let slow = self.b.create_block();
                let fast = self.b.create_block();
                let join = self.b.create_block();
                self.b.set_cold_block(slow);
                self.b.ins().brif(full, slow, &[], fast, &[]);
                self.b.switch_to_block(fast);
                let data = self.b.ins().load(types::I64, MemFlags::trusted(), list, 32);
                let off = if packed { len } else { self.b.ins().ishl_imm(len, 3) };
                let addr = self.b.ins().iadd(data, off);
                if packed {
                    self.b.ins().istore8(MemFlags::trusted(), raw, addr, 0);
                } else {
                    self.b.ins().store(MemFlags::trusted(), raw, addr, 0);
                }
                let new_len = self.b.ins().iadd_imm(len, 1);
                self.b.ins().store(MemFlags::trusted(), new_len, list, 16);
                self.b.ins().jump(join, &[]);
                self.b.switch_to_block(slow);
                self.call_func(self.env.rt.funcs[rt], &[list, raw]);
                self.b.ins().jump(join, &[]);
                self.b.switch_to_block(join);
                true
            }
            // Items that are heap values must release the old item: use the runtime.
            RtFn::ListSet if self.f.operand_ty(&args[2]) != Ty::Ref => {
                let list = self.val(&args[0]);
                let idx = self.val(&args[1]);
                let raw = self.val(&args[2]);
                let line = self.val(&args[3]);
                let addr = self.checked_item_addr(list, idx, line, 3);
                self.b.ins().store(MemFlags::trusted(), raw, addr, 0);
                true
            }
            _ => false,
        }
    }

    /// Bounds-check `idx` (unsigned compare also catches negatives) and return the item address.
    fn checked_item_addr(&mut self, list: Value, idx: Value, line: Value, shift: i64) -> Value {
        let len = self.b.ins().load(types::I64, MemFlags::trusted(), list, 16);
        let oob = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, idx, len);
        let bad = self.b.create_block();
        let ok = self.b.create_block();
        self.b.set_cold_block(bad);
        self.b.ins().brif(oob, bad, &[], ok, &[]);
        self.b.switch_to_block(bad);
        self.call_func(self.env.rt.funcs[&RtFn::ListOob], &[idx, len, line]);
        self.b.ins().trap(TrapCode::INTEGER_OVERFLOW);
        self.b.switch_to_block(ok);
        let data = self.b.ins().load(types::I64, MemFlags::trusted(), list, 32);
        let off = if shift == 0 { idx } else { self.b.ins().ishl_imm(idx, shift) };
        self.b.ins().iadd(data, off)
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
                    (UnOp::BitNot, Ty::Int) => self.b.ins().bnot(v),
                    _ => return Err(format!("bad unary operation on {ty:?}")),
                }
            }
            Rvalue::Cast(kind, a) => {
                let v = self.val(a);
                match kind {
                    CastKind::IntToFloat => self.b.ins().fcvt_from_sint(types::F64, v),
                    CastKind::FloatToInt => self.b.ins().fcvt_to_sint_sat(types::I64, v),
                    CastKind::BoolToInt => self.b.ins().uextend(types::I64, v),
                    CastKind::IntToBool => self.b.ins().icmp_imm(IntCC::NotEqual, v, 0),
                    CastKind::FloatToBits => self.b.ins().bitcast(types::I64, MemFlags::new(), v),
                    CastKind::BitsToFloat => self.b.ins().bitcast(types::F64, MemFlags::new(), v),
                }
            }
            Rvalue::Select(c, a, b) => {
                let (cv, av, bv) = (self.val(c), self.val(a), self.val(b));
                self.b.ins().select(cv, av, bv)
            }
            Rvalue::Binary(op, x, y) => {
                let ty = self.f.operand_ty(x);
                let (a, c) = (self.val(x), self.val(y));
                let k = if let Operand::Int(k) = y { Some(*k) } else { None };
                match ty {
                    Ty::Int | Ty::Ref => self.int_binary(*op, a, c, k),
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

    fn int_binary(&mut self, op: BinOp, a: Value, c: Value, k: Option<i64>) -> Value {
        let ins = |g: &mut Self, cc| g.b.ins().icmp(cc, a, c);
        match op {
            BinOp::Add => self.b.ins().iadd(a, c),
            BinOp::Sub => self.b.ins().isub(a, c),
            BinOp::Mul => self.b.ins().imul(a, c),
            // A constant divisor other than 0 and -1 needs no checks, and Cranelift
            // turns the division into multiplications and shifts.
            BinOp::Div if matches!(k, Some(k) if k != 0 && k != -1) => self.b.ins().sdiv(a, c),
            BinOp::Rem if matches!(k, Some(k) if k != 0 && k != -1) => self.b.ins().srem(a, c),
            BinOp::Div => self.divide(a, c, false),
            BinOp::Rem => self.divide(a, c, true),
            BinOp::BitAnd => self.b.ins().band(a, c),
            BinOp::BitOr => self.b.ins().bor(a, c),
            BinOp::BitXor => self.b.ins().bxor(a, c),
            BinOp::Shl => match k {
                Some(n) if (0..64).contains(&n) => self.b.ins().ishl(a, c),
                Some(_) => self.b.ins().iconst(types::I64, 0),
                None => self.shift(a, c, true),
            },
            BinOp::Shr => match k {
                Some(n) if (0..64).contains(&n) => self.b.ins().sshr(a, c),
                Some(_) => self.b.ins().iconst(types::I64, 0),
                None => self.shift(a, c, false),
            },
            BinOp::Eq => ins(self, IntCC::Equal),
            BinOp::Ne => ins(self, IntCC::NotEqual),
            BinOp::Lt => ins(self, IntCC::SignedLessThan),
            BinOp::Le => ins(self, IntCC::SignedLessThanOrEqual),
            BinOp::Gt => ins(self, IntCC::SignedGreaterThan),
            BinOp::Ge => ins(self, IntCC::SignedGreaterThanOrEqual),
        }
    }

    /// Shifts by 64 or more (or by a negative count) give 0 instead of wrapping.
    fn shift(&mut self, a: Value, amount: Value, left: bool) -> Value {
        let zero = self.b.ins().iconst(types::I64, 0);
        let big = self.b.ins().icmp_imm(IntCC::UnsignedGreaterThanOrEqual, amount, 64);
        let shifted = if left {
            self.b.ins().ishl(a, amount)
        } else {
            self.b.ins().sshr(a, amount)
        };
        self.b.ins().select(big, zero, shifted)
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
            BinOp::Rem | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                return Err("this operator is not supported on floats".into())
            }
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
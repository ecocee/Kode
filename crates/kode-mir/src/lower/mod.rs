//! AST -> MIR.
//!
//! Memory rules (reference counting) for every heap value: strings, lists,
//! maps and class instances.
//!  * A value is either *owned* (we hold one reference that we must release or
//!    hand over) or *borrowed* (a variable, a static literal, or a field).
//!  * Storing a value in a variable, a field, a list or a map, or returning it,
//!    takes ownership (`retain` first if it was only borrowed).
//!  * A variable is released when it is re-assigned and when its function exits.
//!    Every function has one exit block that releases all its variables.
//!  * Function parameters are borrowed: the callee never frees them.
//!  * Temporaries are released right after use. If an error is raised while a
//!    temporary is alive, the error path releases it too (`pending`).

pub(crate) mod data;
mod expr;
mod stmt;
mod tests;

use std::collections::HashMap;

use kode_sema::{Checked, ConstVal, IfaceMethod, InterfaceInfo, Type};
use kode_syntax::ast::{Expr, FnBody, FnDecl, Item, Program};

use crate::ir::*;
use crate::LineMap;

/// `Run` compiles the program's top-level code; `Test` compiles its `test` blocks
/// into a test runner.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Run,
    Test,
}

pub fn lower(program: &Program, checked: &Checked, src: &dyn LineMap, mode: Mode) -> MirProgram {
    let mut strings = Strings::default();
    let mut funcs = Vec::new();

    let n_user = checked.bodies.len();
    let mut dispatches = Vec::new();
    let iface_dispatch = plan_dispatches(checked, &mut strings, n_user, &mut dispatches);

    // Declared functions and methods, then every generic instantiation.
    for (i, f) in checked.bodies.iter().enumerate() {
        let is_method = checked.fns[i].class.is_some();
        funcs.push(lower_fn(f, i, is_method, checked, src, &mut strings, &iface_dispatch));
    }
    funcs.extend(dispatches);

    let entry = match mode {
        Mode::Run => {
            let mut lw = Lowerer::new(checked, src, &mut strings, &iface_dispatch, Ty::Unit, true);
            for item in &program.items {
                if let Item::Stmt(s) = item {
                    lw.stmt(s);
                }
            }
            lw.goto_exit();
            let e = funcs.len();
            funcs.push(lw.finish("<main>".to_string()));
            e
        }
        Mode::Test =>            tests::lower_tests(program, checked, src, &mut strings, &mut funcs, &iface_dispatch),
    };

    let thunks: Vec<MirFunc> = strings.thunk_specs.iter().map(|&i| make_thunk(checked, i)).collect();
    // Closure bodies may contain more closures, so keep going until none is left.
    let mut closures: Vec<MirFunc> = Vec::new();
    let mut ci = 0;
    while ci < strings.closure_specs.len() {
        let spec = strings.closure_specs[ci].clone();
        closures.push(lower_closure(&spec, checked, src, &mut strings, &iface_dispatch));
        ci += 1;
    }
    let fnvals: Vec<MirFunc> = strings.fnval_specs.iter().map(|&i| make_fnval(checked, i)).collect();
    MirProgram {
        funcs,
        strings: strings.list,
        entry,
        thunks,
        closures,
        fnvals,
        std_sigs: strings.std_sigs,
    }
}

/// A closure literal, waiting to be compiled once the enclosing function is done.
#[derive(Clone)]
pub(crate) struct ClosureSpec {
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub captures: Vec<(String, Ty)>,
    pub body: Expr,
}

/// The body of a closure. Parameter 0 is the closure object: captures are read
/// from its fields, so the body needs no other environment (ADR 0028).
fn lower_closure(
    spec: &ClosureSpec,
    checked: &Checked,
    src: &dyn LineMap,
    strings: &mut Strings,
    dispatch: &HashMap<(String, String), usize>,
) -> MirFunc {
    let mut lw = Lowerer::new(checked, src, strings, dispatch, spec.ret, false);
    let clo = lw.new_local(Ty::Ref, Some("closure".into()));
    for (name, ty) in &spec.params {
        let id = lw.new_local(*ty, Some(name.clone()));
        lw.declare(name, id);
    }
    lw.param_count = 1 + spec.params.len();
    // Captures are borrowed from the closure object, which owns them.
    for (i, (name, ty)) in spec.captures.iter().enumerate() {
        let id = lw.new_local(*ty, Some(name.clone()));
        lw.declare(name, id);
        lw.emit(Inst::Load {
            dst: id,
            base: Operand::Local(clo),
            offset: 32 + 8 * i as i32,
            ty: *ty,
        });
    }
    let v = lw.expr(&spec.body);
    lw.return_value(v, spec.ret);
    lw.finish("<closure>".to_string())
}

/// A named function used as a value: the wrapper ignores the closure object and
/// calls the real function, so both share one calling convention.
fn make_fnval(checked: &Checked, idx: usize) -> MirFunc {
    let sig = &checked.fns[idx];
    let mut locals = vec![LocalDecl {
        ty: Ty::Ref,
        name: Some("closure".into()),
    }];
    for (name, ty) in &sig.params {
        locals.push(LocalDecl {
            ty: mir_ty(ty),
            name: Some(name.clone()),
        });
    }
    let args: Vec<Operand> = (1..=sig.params.len()).map(Operand::Local).collect();
    let ret = mir_ty(&sig.ret);
    let dst = if ret == Ty::Unit {
        None
    } else {
        locals.push(LocalDecl { ty: ret, name: None });
        Some(locals.len() - 1)
    };
    let term = match dst {
        Some(d) => Terminator::Return(Some(Operand::Local(d))),
        None => Terminator::Return(None),
    };
    MirFunc {
        name: format!("fnval:{}", sig.name),
        param_count: 1 + sig.params.len(),
        ret,
        locals,
        blocks: vec![Block {
            insts: vec![Inst::Call {
                dst,
                callee: Callee::User(idx),
                args,
            }],
            term,
        }],
    }
}

/// The function a new task runs: it unpacks the arguments from a block and calls
/// the user function. The result goes back as a raw 64-bit value.
fn make_thunk(checked: &Checked, idx: usize) -> MirFunc {
    let sig = &checked.fns[idx];
    let mut locals = vec![LocalDecl {
        ty: Ty::Ref,
        name: Some("block".into()),
    }];
    let mut insts = Vec::new();
    let mut args = Vec::new();
    for (i, (name, ty)) in sig.params.iter().enumerate() {
        let t = mir_ty(ty);
        locals.push(LocalDecl {
            ty: t,
            name: Some(name.clone()),
        });
        let id = locals.len() - 1;
        insts.push(Inst::Load {
            dst: id,
            base: Operand::Local(0),
            offset: 24 + 8 * i as i32,
            ty: t,
        });
        args.push(Operand::Local(id));
    }
    let ret = mir_ty(&sig.ret);
    let raw = if ret == Ty::Unit {
        insts.push(Inst::Call {
            dst: None,
            callee: Callee::User(idx),
            args,
        });
        Operand::Int(0)
    } else {
        locals.push(LocalDecl { ty: ret, name: None });
        let r = locals.len() - 1;
        insts.push(Inst::Call {
            dst: Some(r),
            callee: Callee::User(idx),
            args,
        });
        let kind = match ret {
            Ty::Bool => Some(CastKind::BoolToInt),
            Ty::Float => Some(CastKind::FloatToBits),
            _ => None,
        };
        match kind {
            Some(k) => {
                locals.push(LocalDecl { ty: Ty::Int, name: None });
                let c = locals.len() - 1;
                insts.push(Inst::Assign {
                    dst: c,
                    rv: Rvalue::Cast(k, Operand::Local(r)),
                });
                Operand::Local(c)
            }
            None => Operand::Local(r),
        }
    };
    MirFunc {
        name: format!("thunk:{}", sig.name),
        param_count: 1,
        ret: Ty::Int,
        locals,
        blocks: vec![Block {
            insts,
            term: Terminator::Return(Some(raw)),
        }],
    }
}

/// Build one dispatch function per interface method. Each compares the object's
/// class id (stored in the object header) with every implementing class and
/// calls that class's method. Indexes are assigned after the user functions.
fn plan_dispatches(
    checked: &Checked,
    strings: &mut Strings,
    offset: usize,
    out: &mut Vec<MirFunc>,
) -> HashMap<(String, String), usize> {
    let mut map = HashMap::new();
    for iface in &checked.interfaces {
        for m in &iface.methods {
            let mut impls: Vec<(i64, usize)> = Vec::new();
            for (ci, class) in checked.classes.iter().enumerate() {
                if class.implements.iter().any(|x| x == &iface.name) {
                    if let Some(&mi) = class.methods.get(&m.name) {
                        impls.push((ci as i64, mi));
                    }
                }
            }
            map.insert((iface.name.clone(), m.name.clone()), offset + out.len());
            out.push(build_dispatch(iface, m, &impls, strings));
        }
    }
    map
}

fn build_dispatch(
    iface: &InterfaceInfo,
    m: &IfaceMethod,
    impls: &[(i64, usize)],
    strings: &mut Strings,
) -> MirFunc {
    let ret = mir_ty(&m.ret);
    let mut locals = vec![LocalDecl {
        ty: Ty::Ref,
        name: Some("this".into()),
    }];
    for (name, ty) in &m.params {
        locals.push(LocalDecl {
            ty: mir_ty(ty),
            name: Some(name.clone()),
        });
    }
    let n = impls.len();
    if n == 0 {
        return MirFunc {
            name: format!("{}.{}", iface.name, m.name),
            param_count: 1 + m.params.len(),
            ret,
            locals,
            blocks: vec![Block {
                insts: Vec::new(),
                term: Terminator::Unreachable,
            }],
        };
    }
    let kind = locals.len();
    locals.push(LocalDecl { ty: Ty::Int, name: None });
    let conds: Vec<LocalId> = (0..n)
        .map(|_| {
            locals.push(LocalDecl { ty: Ty::Bool, name: None });
            locals.len() - 1
        })
        .collect();
    let dst = if ret == Ty::Unit {
        None
    } else {
        locals.push(LocalDecl { ty: ret, name: None });
        Some(locals.len() - 1)
    };
    let mut blocks: Vec<Block> = (0..2 * n + 1)
        .map(|_| Block {
            insts: Vec::new(),
            term: Terminator::Unreachable,
        })
        .collect();
    let class_sig = strings.std_sig("kode_obj_class", vec![Ty::Ref], Ty::Int);
    blocks[0].insts.push(Inst::Call {
        dst: Some(kind),
        callee: Callee::Std(class_sig),
        args: vec![Operand::Local(0)],
    });
    let args: Vec<Operand> = (0..=m.params.len()).map(Operand::Local).collect();
    for (i, (cid, mi)) in impls.iter().enumerate() {
        let cmp = 2 * i;
        blocks[cmp].insts.push(Inst::Assign {
            dst: conds[i],
            rv: Rvalue::Binary(BinOp::Eq, Operand::Local(kind), Operand::Int(*cid)),
        });
        let next = if i + 1 < n { 2 * (i + 1) } else { 2 * n };
        blocks[cmp].term = Terminator::Branch {
            cond: Operand::Local(conds[i]),
            then_bb: cmp + 1,
            else_bb: next,
        };
        blocks[cmp + 1].insts.push(Inst::Call {
            dst,
            callee: Callee::User(*mi),
            args: args.clone(),
        });
        blocks[cmp + 1].term = Terminator::Return(dst.map(Operand::Local));
    }
    MirFunc {
        name: format!("{}.{}", iface.name, m.name),
        param_count: 1 + m.params.len(),
        ret,
        locals,
        blocks,
    }
}

fn lower_fn(
    f: &FnDecl,
    index: usize,
    is_method: bool,
    checked: &Checked,
    src: &dyn LineMap,
    strings: &mut Strings,
    dispatch: &HashMap<(String, String), usize>,
) -> MirFunc {
    let sig = &checked.fns[index];
    let ret = mir_ty(&sig.ret);
    let mut lw = Lowerer::new(checked, src, strings, dispatch, ret, false);
    for (name, ty) in &sig.params {
        let id = lw.new_local(mir_ty(ty), Some(name.clone()));
        lw.declare(name, id);
    }
    lw.param_count = sig.params.len();
    if is_method {
        lw.this_local = Some(0);
        lw.cur_class = sig.class.as_ref().and_then(|c| checked.class_index.get(c).copied());
    }

    match &f.body {
        FnBody::Block(stmts) => {
            for s in stmts {
                lw.stmt(s);
            }
            if ret == Ty::Unit {
                lw.goto_exit();
            } else {
                lw.term(Terminator::Unreachable);
            }
        }
        FnBody::Expr(e) => {
            let v = lw.expr(e);
            lw.return_value(v, ret);
        }
    }
    lw.finish(sig.name.clone())
}

pub(crate) fn mir_ty(t: &Type) -> Ty {
    match t {
        Type::Int | Type::Enum(_) => Ty::Int,
        Type::Float => Ty::Float,
        Type::Bool => Ty::Bool,
        Type::Unit => Ty::Unit,
        _ => Ty::Ref,
    }
}

#[derive(Default)]
pub(crate) struct Strings {
    list: Vec<String>,
    map: HashMap<String, usize>,
    std_sigs: Vec<StdSig>,
    thunk_specs: Vec<usize>,
    closure_specs: Vec<ClosureSpec>,
    fnval_specs: Vec<usize>,
}

impl Strings {
    /// Register a runtime library function (once per name) and return its index.
    pub(crate) fn std_sig(&mut self, name: &'static str, params: Vec<Ty>, ret: Ty) -> usize {
        if let Some(i) = self.std_sigs.iter().position(|s| s.name == name) {
            return i;
        }
        self.std_sigs.push(StdSig { name, params, ret });
        self.std_sigs.len() - 1
    }

    /// The task thunk for user function `idx` (created once).
    pub fn thunk(&mut self, idx: usize) -> usize {
        if let Some(i) = self.thunk_specs.iter().position(|&x| x == idx) {
            return i;
        }
        self.thunk_specs.push(idx);
        self.thunk_specs.len() - 1
    }

    /// Register a closure body and return its index.
    pub fn closure(&mut self, spec: ClosureSpec) -> usize {
        self.closure_specs.push(spec);
        self.closure_specs.len() - 1
    }

    /// The wrapper for user function `idx` as a value (created once).
    pub fn fnval(&mut self, idx: usize) -> usize {
        if let Some(i) = self.fnval_specs.iter().position(|&x| x == idx) {
            return i;
        }
        self.fnval_specs.push(idx);
        self.fnval_specs.len() - 1
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(i) = self.map.get(s) {
            return *i;
        }
        let i = self.list.len();
        self.list.push(s.to_string());
        self.map.insert(s.to_string(), i);
        i
    }
}

/// A lowered value and whether we own a reference to it.
#[derive(Clone)]
pub(crate) struct V {
    pub op: Operand,
    pub owned: bool,
    /// For an optional number or bool (`int?`): the `present` flag. The pair is
    /// two locals, never a heap object (ADR 0024).
    pub present: Option<Operand>,
}

impl V {
    pub fn val(op: Operand) -> V {
        V {
            op,
            owned: false,
            present: None,
        }
    }
    /// A value paired with its `present` flag.
    pub fn opt(op: Operand, flag: Operand) -> V {
        V {
            op,
            owned: false,
            present: Some(flag),
        }
    }
    pub fn unit() -> V {
        V::val(Operand::Int(0))
    }
}

pub(crate) struct Lowerer<'a> {
    pub checked: &'a Checked,
    pub src: &'a dyn LineMap,
    strings: &'a mut Strings,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<Block>,
    pub cur: BlockId,
    scopes: Vec<HashMap<String, LocalId>>,
    /// Optional numbers and bools: the value local and its `present` flag.
    opt_flags: HashMap<LocalId, LocalId>,
    pub loops: Vec<(BlockId, BlockId)>,
    /// `finally` blocks that must run before leaving the current code, with the
    /// loop depth at which they were entered.
    cleanups: Vec<(Vec<kode_syntax::ast::Stmt>, usize)>,
    ref_locals: Vec<LocalId>,
    pub param_count: usize,
    pub ret: Ty,
    ret_local: Option<LocalId>,
    exit_bb: BlockId,
    propagate_bb: Option<BlockId>,
    /// Open `try`/`catch` blocks: (handler block, how many temporaries were live outside it).
    pub handlers: Vec<(BlockId, usize)>,
    pending: Vec<Operand>,
    pub is_script: bool,
    pub this_local: Option<LocalId>,
    pub cur_class: Option<usize>,
    pub iface_dispatch: &'a HashMap<(String, String), usize>,
}

impl<'a> Lowerer<'a> {
    pub(crate) fn new(
        checked: &'a Checked,
        src: &'a dyn LineMap,
        strings: &'a mut Strings,
        iface_dispatch: &'a HashMap<(String, String), usize>,
        ret: Ty,
        is_script: bool,
    ) -> Self {
        let mut lw = Lowerer {
            checked,
            src,
            strings,
            locals: Vec::new(),
            blocks: vec![Block {
                insts: Vec::new(),
                term: Terminator::Unreachable,
            }],
            cur: 0,
            scopes: vec![HashMap::new()],
            opt_flags: HashMap::new(),
            loops: Vec::new(),
            cleanups: Vec::new(),
            ref_locals: Vec::new(),
            param_count: 0,
            ret,
            ret_local: None,
            exit_bb: 0,
            propagate_bb: None,
            handlers: Vec::new(),
            pending: Vec::new(),
            is_script,
            this_local: None,
            cur_class: None,
            iface_dispatch,
        };
        lw.exit_bb = lw.new_block();
        lw
    }

    pub fn new_local(&mut self, ty: Ty, name: Option<String>) -> LocalId {
        self.locals.push(LocalDecl { ty, name });
        self.locals.len() - 1
    }

    pub fn new_temp(&mut self, ty: Ty) -> LocalId {
        self.new_local(ty, None)
    }

    pub fn new_block(&mut self) -> BlockId {
        self.blocks.push(Block {
            insts: Vec::new(),
            term: Terminator::Unreachable,
        });
        self.blocks.len() - 1
    }

    pub fn switch(&mut self, b: BlockId) {
        self.cur = b;
    }

    pub fn emit(&mut self, inst: Inst) {
        self.blocks[self.cur].insts.push(inst);
    }

    pub fn term(&mut self, t: Terminator) {
        self.blocks[self.cur].term = t;
    }

    /// After a jump or return, further code goes into a block nobody enters.
    pub fn start_dead_block(&mut self) {
        let b = self.new_block();
        self.switch(b);
    }

    pub fn intern(&mut self, s: &str) -> usize {
        self.strings.intern(s)
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: &str, id: LocalId) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), id);
    }

    /// Declare an optional number or bool: a value local plus a flag local.
    /// Neither is a heap value, so neither is released at exit.
    pub fn declare_var_opt(&mut self, name: &str, scalar: Ty) -> LocalId {
        let id = self.new_local(scalar, Some(name.to_string()));
        let flag = self.new_local(Ty::Bool, None);
        self.opt_flags.insert(id, flag);
        self.declare(name, id);
        id
    }

    /// Declare a variable. Heap values are released when the function exits.
    pub fn declare_var(&mut self, name: &str, ty: Ty) -> LocalId {
        let id = self.new_local(ty, Some(name.to_string()));
        self.declare(name, id);
        if ty == Ty::Ref {
            self.ref_locals.push(id);
        }
        id
    }

    /// A reference variable with no name, used to keep a value alive.
    pub fn hidden_ref(&mut self) -> LocalId {
        let id = self.new_local(Ty::Ref, None);
        self.ref_locals.push(id);
        id
    }

    pub fn lookup(&self, name: &str) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }

    pub fn const_operand(&mut self, name: &str) -> Option<Operand> {
        let (_, val) = self.checked.consts.get(name)?;
        Some(match val {
            ConstVal::Int(v) => Operand::Int(*v),
            ConstVal::Float(v) => Operand::Float(*v),
            ConstVal::Bool(v) => Operand::Bool(*v),
            ConstVal::Str(s) => {
                let s = s.clone();
                Operand::Str(self.strings.intern(&s))
            }
        })
    }

    pub fn expr_type(&self, e: &Expr) -> &'a Type {
        &self.checked.expr_types[e.id as usize]
    }

    pub fn ety(&self, e: &Expr) -> Ty {
        mir_ty(self.expr_type(e))
    }

    pub fn line_of(&self, offset: usize) -> i64 {
        self.src.line_of(offset)
    }

    // ---- ownership helpers -------------------------------------------------

    pub(crate) fn std_sig(&mut self, name: &'static str, params: Vec<Ty>, ret: Ty) -> usize {
        self.strings.std_sig(name, params, ret)
    }

    pub(crate) fn thunk(&mut self, idx: usize) -> usize {
        self.strings.thunk(idx)
    }

    fn track(&mut self, op: &Operand) {
        self.pending.push(op.clone());
    }

    fn untrack(&mut self, op: &Operand) {
        if let Some(i) = self.pending.iter().rposition(|p| p == op) {
            self.pending.remove(i);
        }
    }

    /// A value we own (it is tracked until it is consumed).
    pub fn owned(&mut self, op: Operand) -> V {
        self.track(&op);
        V {
            op,
            owned: true,
            present: None,
        }
    }

    /// Take ownership of a heap value (retain it if we only borrowed it).
    pub fn own(&mut self, v: V) -> Operand {
        let is_static = matches!(v.op, Operand::Str(_) | Operand::NullRef);
        if v.owned {
            self.untrack(&v.op);
        } else if !is_static {
            self.emit(Inst::Retain(v.op.clone()));
        }
        v.op
    }

    pub fn release_if_owned(&mut self, v: V) {
        if v.owned {
            self.untrack(&v.op);
            self.emit(Inst::Release(v.op));
        }
    }

    // ---- errors ------------------------------------------------------------

    /// Where an error goes (the innermost `try`/`catch`, or out of the function)
    /// and how many live temporaries stay alive there.
    fn error_target(&mut self) -> (BlockId, usize) {
        if let Some(h) = self.handlers.last() {
            return *h;
        }
        if self.propagate_bb.is_none() {
            let b = self.new_block();
            self.propagate_bb = Some(b);
        }
        (self.propagate_bb.unwrap(), 0)
    }

    pub fn push_handler(&mut self, bb: BlockId) {
        let n = self.pending.len();
        self.handlers.push((bb, n));
    }

    /// Jump to the error target from the current block, releasing the
    /// temporaries that die on the way.
    pub fn goto_error(&mut self) {
        let (t, keep) = self.error_target();
        let first = keep.min(self.pending.len());
        for i in first..self.pending.len() {
            let p = self.pending[i].clone();
            self.emit(Inst::Release(p));
        }
        self.term(Terminator::Goto(t));
    }

    /// After a call that may raise an error: branch to the error path if one is pending.
    pub fn error_check(&mut self) {
        let flag = self.new_temp(Ty::Bool);
        self.emit(Inst::Call {
            dst: Some(flag),
            callee: Callee::Rt(RtFn::ErrPending),
            args: vec![],
        });
        let err_bb = self.new_block();
        let cont = self.new_block();
        self.term(Terminator::Branch {
            cond: Operand::Local(flag),
            then_bb: err_bb,
            else_bb: cont,
        });
        self.switch(err_bb);
        self.goto_error();
        self.switch(cont);
    }

    // ---- function exit -----------------------------------------------------

    pub fn return_value(&mut self, v: V, ret: Ty) {
        if ret == Ty::Unit {
            self.release_if_owned(v);
        } else {
            let op = if ret == Ty::Ref { self.own(v) } else { v.op };
            let rl = self.ret_local(ret);
            self.emit(Inst::Assign {
                dst: rl,
                rv: Rvalue::Use(op),
            });
        }
        self.goto_exit();
    }

    fn ret_local(&mut self, ret: Ty) -> LocalId {
        if let Some(l) = self.ret_local {
            return l;
        }
        let l = self.new_local(ret, None);
        self.ret_local = Some(l);
        l
    }

    pub fn goto_exit(&mut self) {
        let e = self.exit_bb;
        self.term(Terminator::Goto(e));
    }

    pub(crate) fn finish(mut self, name: String) -> MirFunc {
        // Normal exit: release every heap variable, then return.
        let rets = self.ret_local;
        let default = default_operand(self.ret);
        self.switch(self.exit_bb);
        for id in self.ref_locals.clone() {
            self.emit(Inst::Release(Operand::Local(id)));
        }
        let rv = match (self.ret, rets) {
            (Ty::Unit, _) => None,
            (_, Some(l)) => Some(Operand::Local(l)),
            (_, None) => Some(default.clone()),
        };
        self.term(Terminator::Return(rv));

        // Error exit.
        if let Some(pb) = self.propagate_bb {
            self.switch(pb);
            if self.is_script {
                self.emit(Inst::Call {
                    dst: None,
                    callee: Callee::Rt(RtFn::Uncaught),
                    args: vec![],
                });
                self.term(Terminator::Unreachable);
            } else {
                for id in self.ref_locals.clone() {
                    self.emit(Inst::Release(Operand::Local(id)));
                }
                self.term(Terminator::Return(if self.ret == Ty::Unit { None } else { Some(default) }));
            }
        }

        // Every non-parameter local starts with a known value, so a variable
        // that was never assigned releases harmlessly.
        let inits: Vec<Inst> = self
            .locals
            .iter()
            .enumerate()
            .skip(self.param_count)
            .map(|(i, l)| Inst::Assign {
                dst: i,
                rv: Rvalue::Use(default_operand(l.ty)),
            })
            .collect();
        self.blocks[0].insts.splice(0..0, inits);
        // Block 0 flows into the first real block.
        MirFunc {
            name,
            param_count: self.param_count,
            ret: self.ret,
            locals: self.locals,
            blocks: self.blocks,
        }
    }
}

fn default_operand(t: Ty) -> Operand {
    match t {
        Ty::Int | Ty::Unit => Operand::Int(0),
        Ty::Float => Operand::Float(0.0),
        Ty::Bool => Operand::Bool(false),
        Ty::Ref => Operand::NullRef,
    }
}
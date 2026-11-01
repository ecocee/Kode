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

mod data;
mod expr;
mod stmt;
mod tests;

use std::collections::HashMap;

use kode_sema::{Checked, ConstVal, Type};
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

    for (i, (cls, f)) in program.fn_decls().into_iter().enumerate() {
        funcs.push(lower_fn(f, i, cls.is_some(), checked, src, &mut strings));
    }

    let entry = match mode {
        Mode::Run => {
            let mut lw = Lowerer::new(checked, src, &mut strings, Ty::Unit, true);
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
        Mode::Test => tests::lower_tests(program, checked, src, &mut strings, &mut funcs),
    };

    MirProgram {
        funcs,
        strings: strings.list,
        entry,
    }
}

fn lower_fn(f: &FnDecl, index: usize, is_method: bool, checked: &Checked, src: &dyn LineMap, strings: &mut Strings) -> MirFunc {
    let sig = &checked.fns[index];
    let ret = mir_ty(&sig.ret);
    let mut lw = Lowerer::new(checked, src, strings, ret, false);
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
}

impl Strings {
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
}

impl V {
    pub fn val(op: Operand) -> V {
        V { op, owned: false }
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
    pub loops: Vec<(BlockId, BlockId)>,
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
}

impl<'a> Lowerer<'a> {
    pub(crate) fn new(checked: &'a Checked, src: &'a dyn LineMap, strings: &'a mut Strings, ret: Ty, is_script: bool) -> Self {
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
            loops: Vec::new(),
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
        V { op, owned: true }
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
        for p in self.pending[keep.min(self.pending.len())..].to_vec() {
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
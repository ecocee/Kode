//! AST -> MIR.
//!
//! Reference counting rules for strings (the "memory model" of v0.1):
//!  * Every string value is either *owned* (we hold one reference that we must
//!    release or hand over) or *borrowed* (a variable or a static literal).
//!  * Storing a string in a variable or returning it takes ownership
//!    (`retain` first if it was only borrowed).
//!  * A variable is released when it is re-assigned and when the function ends.
//!  * Function parameters are borrowed: the callee never frees them.
//!  * Temporaries are released right after they are used.

mod expr;
mod stmt;

use std::collections::HashMap;

use kode_sema::{Checked, ConstVal, Type};
use kode_syntax::ast::{Expr, FnBody, FnDecl, Item, Program};

use crate::ir::*;

pub fn lower(program: &Program, checked: &Checked, src: &str) -> MirProgram {
    let mut strings = Strings::default();
    let mut funcs = Vec::new();

    let mut index = 0;
    for item in &program.items {
        if let Item::Fn(f) = item {
            funcs.push(lower_fn(f, index, checked, src, &mut strings));
            index += 1;
        }
    }

    // The top-level script becomes the entry function.
    let mut lw = Lowerer::new(checked, src, &mut strings, Ty::Unit);
    for item in &program.items {
        if let Item::Stmt(s) = item {
            lw.stmt(s);
        }
    }
    lw.release_locals();
    lw.term(Terminator::Return(None));
    let entry = funcs.len();
    funcs.push(lw.finish("<main>".to_string()));

    MirProgram {
        funcs,
        strings: strings.list,
        entry,
    }
}

fn lower_fn(f: &FnDecl, index: usize, checked: &Checked, src: &str, strings: &mut Strings) -> MirFunc {
    let sig = &checked.fns[index];
    let mut lw = Lowerer::new(checked, src, strings, to_ty(sig.ret));
    for (name, ty) in &sig.params {
        let id = lw.new_local(to_ty(*ty), Some(name.clone()));
        lw.declare(name, id);
    }
    lw.param_count = sig.params.len();

    match &f.body {
        FnBody::Block(stmts) => {
            for s in stmts {
                lw.stmt(s);
            }
            if lw.ret == Ty::Unit {
                lw.release_locals();
                lw.term(Terminator::Return(None));
            } else {
                lw.term(Terminator::Unreachable);
            }
        }
        FnBody::Expr(e) => {
            let v = lw.expr(e);
            if lw.ret == Ty::Unit {
                lw.release_if_owned(v);
                lw.release_locals();
                lw.term(Terminator::Return(None));
            } else {
                let op = if lw.ret == Ty::Str { lw.own(v) } else { v.op };
                lw.release_locals();
                lw.term(Terminator::Return(Some(op)));
            }
        }
    }
    lw.finish(sig.name.clone())
}

pub(crate) fn to_ty(t: Type) -> Ty {
    match t {
        Type::Int => Ty::Int,
        Type::Float => Ty::Float,
        Type::Bool => Ty::Bool,
        Type::Str => Ty::Str,
        Type::Unit => Ty::Unit,
    }
}

#[derive(Default)]
pub(crate) struct Strings {
    list: Vec<String>,
    map: HashMap<String, usize>,
}

impl Strings {
    fn intern(&mut self, s: &str) -> usize {
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
    pub fn owned(op: Operand) -> V {
        V { op, owned: true }
    }
    pub fn unit() -> V {
        V::val(Operand::Int(0))
    }
}

pub(crate) struct Lowerer<'a> {
    pub checked: &'a Checked,
    pub src: &'a str,
    strings: &'a mut Strings,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<Block>,
    pub cur: BlockId,
    scopes: Vec<HashMap<String, LocalId>>,
    pub loops: Vec<(BlockId, BlockId)>,
    str_locals: Vec<LocalId>,
    pub param_count: usize,
    pub ret: Ty,
}

impl<'a> Lowerer<'a> {
    fn new(checked: &'a Checked, src: &'a str, strings: &'a mut Strings, ret: Ty) -> Self {
        Lowerer {
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
            str_locals: Vec::new(),
            param_count: 0,
            ret,
        }
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

    /// Declare a variable that holds a string: it is released at function end.
    pub fn declare_var(&mut self, name: &str, ty: Ty) -> LocalId {
        let id = self.new_local(ty, Some(name.to_string()));
        self.declare(name, id);
        if ty == Ty::Str {
            self.str_locals.push(id);
        }
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

    pub fn expr_ty(&self, e: &Expr) -> Ty {
        to_ty(self.checked.expr_types[e.id as usize])
    }

    pub fn line_of(&self, offset: usize) -> i64 {
        self.src[..offset.min(self.src.len())].matches('\n').count() as i64 + 1
    }

    /// Take ownership of a string value (retain it if we only borrowed it).
    pub fn own(&mut self, v: V) -> Operand {
        let is_static = matches!(v.op, Operand::Str(_) | Operand::NullStr);
        if !v.owned && !is_static {
            self.emit(Inst::Retain(v.op.clone()));
        }
        v.op
    }

    pub fn release_if_owned(&mut self, v: V) {
        if v.owned {
            self.emit(Inst::Release(v.op));
        }
    }

    pub fn release_locals(&mut self) {
        for id in self.str_locals.clone() {
            self.emit(Inst::Release(Operand::Local(id)));
        }
    }

    fn finish(mut self, name: String) -> MirFunc {
        // Every non-parameter local starts with a known value, so a string
        // variable that was never assigned releases harmlessly.
        let inits: Vec<Inst> = self
            .locals
            .iter()
            .enumerate()
            .skip(self.param_count)
            .map(|(i, l)| Inst::Assign {
                dst: i,
                rv: Rvalue::Use(match l.ty {
                    Ty::Int | Ty::Unit => Operand::Int(0),
                    Ty::Float => Operand::Float(0.0),
                    Ty::Bool => Operand::Bool(false),
                    Ty::Str => Operand::NullStr,
                }),
            })
            .collect();
        self.blocks[0].insts.splice(0..0, inits);
        MirFunc {
            name,
            param_count: self.param_count,
            ret: self.ret,
            locals: self.locals,
            blocks: self.blocks,
        }
    }
}

//! `kode test`: compile `test` blocks into functions plus a small runner.

use kode_syntax::ast::{Item, Program};

use kode_sema::Checked;

use std::collections::HashMap;

use super::{Lowerer, Strings};
use crate::LineMap;
use crate::ir::*;

pub(crate) fn lower_tests(
    program: &Program,
    checked: &Checked,
    src: &dyn LineMap,
    strings: &mut Strings,
    funcs: &mut Vec<MirFunc>,
    dispatch: &HashMap<(String, String), usize>,
) -> usize {
    let first = funcs.len();
    let mut names: Vec<String> = Vec::new();
    for item in &program.items {
        if let Item::Test(t) = item {
            let mut lw = Lowerer::new(checked, src, strings, dispatch, Ty::Unit, false);
            for s in &t.body {
                lw.stmt(s);
            }
            lw.goto_exit();
            funcs.push(lw.finish(format!("test:{}", t.name)));
            names.push(t.name.clone());
        }
    }

    let mut lw = Lowerer::new(checked, src, strings, dispatch, Ty::Unit, true);
    let failed = lw.new_local(Ty::Int, Some("failed".into()));
    let print_str = |lw: &mut Lowerer, s: &str| {
        let id = lw.intern(s);
        lw.emit(Inst::Call {
            dst: None,
            callee: Callee::Rt(RtFn::PrintStr),
            args: vec![Operand::Str(id)],
        });
    };
    let nl = |lw: &mut Lowerer| {
        lw.emit(Inst::Call {
            dst: None,
            callee: Callee::Rt(RtFn::PrintNl),
            args: vec![],
        });
    };

    for (i, name) in names.iter().enumerate() {
        lw.emit(Inst::Call {
            dst: None,
            callee: Callee::User(first + i),
            args: vec![],
        });
        let flag = lw.new_temp(Ty::Bool);
        lw.emit(Inst::Call {
            dst: Some(flag),
            callee: Callee::Rt(RtFn::ErrPending),
            args: vec![],
        });
        let pass_bb = lw.new_block();
        let fail_bb = lw.new_block();
        let next = lw.new_block();
        lw.term(Terminator::Branch {
            cond: Operand::Local(flag),
            then_bb: fail_bb,
            else_bb: pass_bb,
        });

        lw.switch(pass_bb);
        print_str(&mut lw, "PASS ");
        print_str(&mut lw, name);
        nl(&mut lw);
        lw.term(Terminator::Goto(next));

        lw.switch(fail_bb);
        let err = lw.new_temp(Ty::Ref);
        lw.emit(Inst::Call {
            dst: Some(err),
            callee: Callee::Rt(RtFn::ErrTake),
            args: vec![],
        });
        let msg = lw.new_temp(Ty::Ref);
        lw.emit(Inst::Load {
            dst: msg,
            base: Operand::Local(err),
            offset: 24,
            ty: Ty::Ref,
        });
        print_str(&mut lw, "FAIL ");
        print_str(&mut lw, name);
        print_str(&mut lw, ": ");
        lw.emit(Inst::Call {
            dst: None,
            callee: Callee::Rt(RtFn::PrintStr),
            args: vec![Operand::Local(msg)],
        });
        nl(&mut lw);
        lw.emit(Inst::Release(Operand::Local(err)));
        lw.emit(Inst::Assign {
            dst: failed,
            rv: Rvalue::Binary(BinOp::Add, Operand::Local(failed), Operand::Int(1)),
        });
        lw.term(Terminator::Goto(next));
        lw.switch(next);
    }

    // summary
    let passed = lw.new_temp(Ty::Int);
    lw.emit(Inst::Assign {
        dst: passed,
        rv: Rvalue::Binary(BinOp::Sub, Operand::Int(names.len() as i64), Operand::Local(failed)),
    });
    lw.emit(Inst::Call {
        dst: None,
        callee: Callee::Rt(RtFn::PrintInt),
        args: vec![Operand::Local(passed)],
    });
    print_str(&mut lw, " passed, ");
    lw.emit(Inst::Call {
        dst: None,
        callee: Callee::Rt(RtFn::PrintInt),
        args: vec![Operand::Local(failed)],
    });
    print_str(&mut lw, " failed");
    nl(&mut lw);
    let bad = lw.new_temp(Ty::Bool);
    lw.emit(Inst::Assign {
        dst: bad,
        rv: Rvalue::Binary(BinOp::Gt, Operand::Local(failed), Operand::Int(0)),
    });
    let exit_bad = lw.new_block();
    let ok = lw.new_block();
    lw.term(Terminator::Branch {
        cond: Operand::Local(bad),
        then_bb: exit_bad,
        else_bb: ok,
    });
    lw.switch(exit_bad);
    lw.emit(Inst::Call {
        dst: None,
        callee: Callee::Rt(RtFn::Exit),
        args: vec![Operand::Int(1)],
    });
    lw.term(Terminator::Unreachable);
    lw.switch(ok);
    lw.goto_exit();

    let entry = funcs.len();
    funcs.push(lw.finish("<tests>".to_string()));
    entry
}
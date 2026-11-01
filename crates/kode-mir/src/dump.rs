//! Text form of MIR, for `kode mir file.kode` and for debugging.

use std::fmt::Write;

use crate::ir::*;

fn ty(t: Ty) -> &'static str {
    match t {
        Ty::Int => "int",
        Ty::Float => "float",
        Ty::Bool => "bool",
        Ty::Ref => "ref",
        Ty::Unit => "unit",
    }
}

fn op(p: &MirProgram, o: &Operand) -> String {
    match o {
        Operand::Local(i) => format!("_{i}"),
        Operand::Int(v) => v.to_string(),
        Operand::Float(v) => format!("{v:?}"),
        Operand::Bool(v) => v.to_string(),
        Operand::Str(i) => format!("{:?}", p.strings[*i]),
        Operand::NullRef => "null".to_string(),
    }
}

pub fn dump(p: &MirProgram) -> String {
    let mut out = String::new();
    for (fi, f) in p.funcs.iter().enumerate() {
        let params: Vec<String> = (0..f.param_count)
            .map(|i| format!("_{i}: {}", ty(f.locals[i].ty)))
            .collect();
        let marker = if fi == p.entry { "  // entry" } else { "" };
        let _ = writeln!(out, "fn {}({}) -> {} {{{marker}", f.name, params.join(", "), ty(f.ret));
        for (i, l) in f.locals.iter().enumerate().skip(f.param_count) {
            let name = l.name.as_deref().map(|n| format!("  // {n}")).unwrap_or_default();
            let _ = writeln!(out, "    let _{i}: {}{name}", ty(l.ty));
        }
        for (bi, b) in f.blocks.iter().enumerate() {
            let _ = writeln!(out, "  bb{bi}:");
            for inst in &b.insts {
                let line = match inst {
                    Inst::Assign { dst, rv } => {
                        let r = match rv {
                            Rvalue::Use(a) => op(p, a),
                            Rvalue::Unary(u, a) => format!("{} {}", format!("{u:?}").to_lowercase(), op(p, a)),
                            Rvalue::Binary(b, x, y) => {
                                format!("{} {}, {}", format!("{b:?}").to_lowercase(), op(p, x), op(p, y))
                            }
                            Rvalue::Cast(c, a) => format!("cast {c:?} {}", op(p, a)),
                        };
                        format!("_{dst} = {r}")
                    }
                    Inst::Call { dst, callee, args } => {
                        let name = match callee {
                            Callee::User(i) => p.funcs[*i].name.clone(),
                            Callee::Rt(r) => format!("rt::{r:?}"),
                        };
                        let args: Vec<String> = args.iter().map(|a| op(p, a)).collect();
                        match dst {
                            Some(d) => format!("_{d} = call {name}({})", args.join(", ")),
                            None => format!("call {name}({})", args.join(", ")),
                        }
                    }
                    Inst::Load { dst, base, offset, .. } => format!("_{dst} = load {}+{offset}", op(p, base)),
                    Inst::Store { base, offset, value } => format!("store {}+{offset}, {}", op(p, base), op(p, value)),
                    Inst::Retain(a) => format!("retain {}", op(p, a)),
                    Inst::Release(a) => format!("release {}", op(p, a)),
                };
                let _ = writeln!(out, "    {line}");
            }
            let t = match &b.term {
                Terminator::Goto(t) => format!("goto bb{t}"),
                Terminator::Branch { cond, then_bb, else_bb } => {
                    format!("branch {}, bb{then_bb}, bb{else_bb}", op(p, cond))
                }
                Terminator::Return(Some(v)) => format!("return {}", op(p, v)),
                Terminator::Return(None) => "return".to_string(),
                Terminator::Unreachable => "unreachable".to_string(),
            };
            let _ = writeln!(out, "    {t}");
        }
        out.push_str("}\n\n");
    }
    out
}
//! Cheap sanity checks on MIR. A failure here is a compiler bug, not a user error.

use crate::ir::*;

pub fn verify(p: &MirProgram) -> Result<(), String> {
    if p.entry >= p.funcs.len() {
        return Err("entry function is missing".into());
    }
    for f in &p.funcs {
        let nl = f.locals.len();
        let nb = f.blocks.len();
        let check_op = |o: &Operand| -> Result<(), String> {
            match o {
                Operand::Local(i) if *i >= nl => Err(format!("{}: local _{i} does not exist", f.name)),
                Operand::Str(i) if *i >= p.strings.len() => {
                    Err(format!("{}: string #{i} does not exist", f.name))
                }
                _ => Ok(()),
            }
        };
        if f.param_count > nl {
            return Err(format!("{}: more parameters than locals", f.name));
        }
        for (bi, b) in f.blocks.iter().enumerate() {
            for inst in &b.insts {
                match inst {
                    Inst::Assign { dst, rv } => {
                        if *dst >= nl {
                            return Err(format!("{}: bb{bi} assigns to missing local _{dst}", f.name));
                        }
                        match rv {
                            Rvalue::Use(a) | Rvalue::Unary(_, a) | Rvalue::Cast(_, a) => check_op(a)?,
                            Rvalue::Binary(_, a, c) => {
                                check_op(a)?;
                                check_op(c)?;
                            }
                        }
                    }
                    Inst::Call { dst, callee, args } => {
                        if let Some(d) = dst {
                            if *d >= nl {
                                return Err(format!("{}: bb{bi} call result _{d} missing", f.name));
                            }
                        }
                        if let Callee::User(i) = callee {
                            if *i >= p.funcs.len() {
                                return Err(format!("{}: bb{bi} calls missing function #{i}", f.name));
                            }
                        }
                        for a in args {
                            check_op(a)?;
                        }
                    }
                    Inst::Retain(a) | Inst::Release(a) => check_op(a)?,
                }
            }
            match &b.term {
                Terminator::Goto(t) if *t >= nb => {
                    return Err(format!("{}: bb{bi} jumps to missing bb{t}", f.name))
                }
                Terminator::Branch { cond, then_bb, else_bb } => {
                    check_op(cond)?;
                    if *then_bb >= nb || *else_bb >= nb {
                        return Err(format!("{}: bb{bi} branches to a missing block", f.name));
                    }
                }
                Terminator::Return(Some(v)) => check_op(v)?,
                _ => {}
            }
        }
    }
    Ok(())
}

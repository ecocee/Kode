//! MIR optimizations.
//!
//! `inline_calls`: calls to small functions are replaced by a copy of their body
//! (a recursive function is inlined into itself once).
//!
//! `if_convert`: a small `if / else` that only does cheap arithmetic is computed on both
//! sides and merged with a branch-free `select`. This removes branch mispredictions in
//! code such as `if x % 2 == 0 { x = x / 2 } else { x = 3 * x + 1 }`.

use std::collections::HashMap;

use crate::ir::*;

pub fn optimize(p: &mut MirProgram) {
    inline_calls(p);
    for f in p.funcs.iter_mut().chain(p.thunks.iter_mut()) {
        for _ in 0..3 {
            if !if_convert(f) {
                break;
            }
        }
    }
}

/// Can this instruction run on a path where it would not have run before?
fn speculable(f: &MirFunc, inst: &Inst) -> bool {
    let Inst::Assign { dst, rv } = inst else {
        return false;
    };
    if f.locals[*dst].ty == Ty::Ref || f.locals[*dst].ty == Ty::Unit {
        return false;
    }
    match rv {
        Rvalue::Use(_) | Rvalue::Unary(..) | Rvalue::Select(..) => true,
        Rvalue::Cast(..) => true,
        // division can trap, so it must stay behind its branch
        Rvalue::Binary(BinOp::Div | BinOp::Rem, _, d) => {
            matches!(d, Operand::Int(k) if *k != 0 && *k != -1)
        }
        Rvalue::Binary(..) => true,
    }
}

fn map_op(o: &Operand, m: &HashMap<LocalId, LocalId>) -> Operand {
    match o {
        Operand::Local(l) => Operand::Local(*m.get(l).unwrap_or(l)),
        other => other.clone(),
    }
}

fn map_rv(rv: &Rvalue, m: &HashMap<LocalId, LocalId>) -> Rvalue {
    match rv {
        Rvalue::Use(a) => Rvalue::Use(map_op(a, m)),
        Rvalue::Unary(u, a) => Rvalue::Unary(*u, map_op(a, m)),
        Rvalue::Binary(b, x, y) => Rvalue::Binary(*b, map_op(x, m), map_op(y, m)),
        Rvalue::Cast(c, a) => Rvalue::Cast(*c, map_op(a, m)),
        Rvalue::Select(c, a, b) => Rvalue::Select(map_op(c, m), map_op(a, m), map_op(b, m)),
    }
}

/// Run an arm's instructions into fresh locals. Returns the new instructions and the
/// map from each assigned local to the fresh local that holds its new value.
fn speculate(f: &mut MirFunc, insts: &[Inst]) -> (Vec<Inst>, HashMap<LocalId, LocalId>) {
    let mut map: HashMap<LocalId, LocalId> = HashMap::new();
    let mut out = Vec::new();
    for inst in insts {
        let Inst::Assign { dst, rv } = inst else {
            unreachable!("checked by speculable")
        };
        let rv = map_rv(rv, &map);
        let ty = f.locals[*dst].ty;
        f.locals.push(LocalDecl { ty, name: None });
        let fresh = f.locals.len() - 1;
        map.insert(*dst, fresh);
        out.push(Inst::Assign { dst: fresh, rv });
    }
    (out, map)
}

fn if_convert(f: &mut MirFunc) -> bool {
    let mut preds = vec![0usize; f.blocks.len()];
    for b in &f.blocks {
        match &b.term {
            Terminator::Goto(t) => preds[*t] += 1,
            Terminator::Branch {
                then_bb, else_bb, ..
            } => {
                preds[*then_bb] += 1;
                preds[*else_bb] += 1;
            }
            _ => {}
        }
    }
    let mut changed = false;
    for bi in 0..f.blocks.len() {
        let Terminator::Branch {
            cond,
            then_bb,
            else_bb,
        } = f.blocks[bi].term.clone()
        else {
            continue;
        };
        if then_bb == else_bb || then_bb == bi || else_bb == bi {
            continue;
        }
        // The join block: both arms jump to it (an arm may be the join itself).
        let arm_target = |b: BlockId| match &f.blocks[b].term {
            Terminator::Goto(j) => Some(*j),
            _ => None,
        };
        let (join, then_arm, else_arm): (BlockId, Option<BlockId>, Option<BlockId>) =
            match (arm_target(then_bb), arm_target(else_bb)) {
                (Some(a), Some(b)) if a == b && preds[then_bb] == 1 && preds[else_bb] == 1 => {
                    (a, Some(then_bb), Some(else_bb))
                }
                (Some(a), _) if a == else_bb && preds[then_bb] == 1 => (a, Some(then_bb), None),
                (_, Some(b)) if b == then_bb && preds[else_bb] == 1 => (b, None, Some(else_bb)),
                _ => continue,
            };
        if join == bi {
            continue;
        }
        let then_insts = then_arm
            .map(|b| f.blocks[b].insts.clone())
            .unwrap_or_default();
        let else_insts = else_arm
            .map(|b| f.blocks[b].insts.clone())
            .unwrap_or_default();
        let total = then_insts.len() + else_insts.len();
        if total == 0 || then_insts.len() > 4 || else_insts.len() > 4 || total > 6 {
            continue;
        }
        if !then_insts
            .iter()
            .chain(else_insts.iter())
            .all(|i| speculable(f, i))
        {
            continue;
        }
        let (t_code, t_map) = speculate(f, &then_insts);
        let (e_code, e_map) = speculate(f, &else_insts);
        let mut assigned: Vec<LocalId> = t_map.keys().chain(e_map.keys()).copied().collect();
        assigned.sort_unstable();
        assigned.dedup();
        if let Operand::Local(c) = &cond {
            if assigned.contains(c) {
                continue; // a select would overwrite its own condition
            }
        }
        let blk = &mut f.blocks[bi];
        blk.insts.extend(t_code);
        blk.insts.extend(e_code);
        for d in assigned {
            let a = Operand::Local(*t_map.get(&d).unwrap_or(&d));
            let b = Operand::Local(*e_map.get(&d).unwrap_or(&d));
            blk.insts.push(Inst::Assign {
                dst: d,
                rv: Rvalue::Select(cond.clone(), a, b),
            });
        }
        blk.term = Terminator::Goto(join);
        // the arms are now dead
        for arm in [then_arm, else_arm].into_iter().flatten() {
            f.blocks[arm].insts.clear();
            f.blocks[arm].term = Terminator::Unreachable;
        }
        changed = true;
        break; // block numbers and predecessor counts changed: start over
    }
    changed
}

// ---------------------------------------------------------------------------
// Inlining
// ---------------------------------------------------------------------------

const MAX_CALLEE_BLOCKS: usize = 8;
const MAX_CALLEE_INSTS: usize = 24;
const MAX_CALLER_INSTS: usize = 700;

fn inst_count(f: &MirFunc) -> usize {
    f.blocks.iter().map(|b| b.insts.len() + 1).sum()
}

fn inlinable(f: &MirFunc) -> bool {
    f.blocks.len() <= MAX_CALLEE_BLOCKS && inst_count(f) <= MAX_CALLEE_INSTS
}

fn shift_op(o: &Operand, loff: usize) -> Operand {
    match o {
        Operand::Local(l) => Operand::Local(l + loff),
        other => other.clone(),
    }
}

fn shift_rv(rv: &Rvalue, loff: usize) -> Rvalue {
    let s = |o: &Operand| shift_op(o, loff);
    match rv {
        Rvalue::Use(a) => Rvalue::Use(s(a)),
        Rvalue::Unary(u, a) => Rvalue::Unary(*u, s(a)),
        Rvalue::Binary(b, x, y) => Rvalue::Binary(*b, s(x), s(y)),
        Rvalue::Cast(c, a) => Rvalue::Cast(*c, s(a)),
        Rvalue::Select(c, a, b) => Rvalue::Select(s(c), s(a), s(b)),
    }
}

fn shift_inst(i: &Inst, loff: usize) -> Inst {
    let s = |o: &Operand| shift_op(o, loff);
    match i {
        Inst::Assign { dst, rv } => Inst::Assign {
            dst: dst + loff,
            rv: shift_rv(rv, loff),
        },
        Inst::Call { dst, callee, args } => Inst::Call {
            dst: dst.map(|d| d + loff),
            callee: *callee,
            args: args.iter().map(s).collect(),
        },
        Inst::CallIndirect { dst, callee, args } => Inst::CallIndirect {
            dst: dst.map(|d| d + loff),
            callee: s(callee),
            args: args.iter().map(s).collect(),
        },
        Inst::Load {
            dst,
            base,
            offset,
            ty,
        } => Inst::Load {
            dst: dst + loff,
            base: s(base),
            offset: *offset,
            ty: *ty,
        },
        Inst::Store {
            base,
            offset,
            value,
        } => Inst::Store {
            base: s(base),
            offset: *offset,
            value: s(value),
        },
        Inst::Retain(a) => Inst::Retain(s(a)),
        Inst::Release(a) => Inst::Release(s(a)),
    }
}

/// Replace the call at `f.blocks[bi].insts[ii]` with a copy of `callee`.
/// Returns the range of blocks that hold the copy.
fn inline_one(
    f: &mut MirFunc,
    callee: &MirFunc,
    bi: BlockId,
    ii: usize,
) -> std::ops::Range<BlockId> {
    let Inst::Call { dst, args, .. } = f.blocks[bi].insts[ii].clone() else {
        unreachable!()
    };
    let loff = f.locals.len();
    f.locals.extend(callee.locals.iter().cloned());
    let cont = f.blocks.len();
    let boff = cont + 1;

    // split the block at the call
    let post: Vec<Inst> = f.blocks[bi].insts.split_off(ii + 1);
    f.blocks[bi].insts.pop(); // the call itself
    let old_term = std::mem::replace(&mut f.blocks[bi].term, Terminator::Goto(boff));
    for (k, a) in args.iter().enumerate() {
        f.blocks[bi].insts.push(Inst::Assign {
            dst: loff + k,
            rv: Rvalue::Use(a.clone()),
        });
    }
    f.blocks.push(Block {
        insts: post,
        term: old_term,
    });

    for cb in &callee.blocks {
        let mut insts: Vec<Inst> = cb.insts.iter().map(|i| shift_inst(i, loff)).collect();
        let term = match &cb.term {
            Terminator::Goto(t) => Terminator::Goto(t + boff),
            Terminator::Branch {
                cond,
                then_bb,
                else_bb,
            } => Terminator::Branch {
                cond: shift_op(cond, loff),
                then_bb: then_bb + boff,
                else_bb: else_bb + boff,
            },
            Terminator::Return(v) => {
                if let (Some(d), Some(v)) = (dst, v) {
                    insts.push(Inst::Assign {
                        dst: d,
                        rv: Rvalue::Use(shift_op(v, loff)),
                    });
                }
                Terminator::Goto(cont)
            }
            Terminator::Unreachable => Terminator::Unreachable,
        };
        f.blocks.push(Block { insts, term });
    }
    boff..f.blocks.len()
}

fn inline_calls(p: &mut MirProgram) {
    let snapshot: Vec<MirFunc> = p.funcs.clone();
    for fi in 0..p.funcs.len() {
        let mut skip: Vec<std::ops::Range<BlockId>> = Vec::new();
        let mut bi = 0;
        while bi < p.funcs[fi].blocks.len() {
            if skip.iter().any(|r| r.contains(&bi)) {
                bi += 1;
                continue;
            }
            let mut inlined = false;
            for ii in 0..p.funcs[fi].blocks[bi].insts.len() {
                let Inst::Call {
                    callee: Callee::User(g),
                    ..
                } = &p.funcs[fi].blocks[bi].insts[ii]
                else {
                    continue;
                };
                let g = *g;
                if g == p.entry
                    || !inlinable(&snapshot[g])
                    || inst_count(&p.funcs[fi]) > MAX_CALLER_INSTS
                {
                    continue;
                }
                let range = inline_one(&mut p.funcs[fi], &snapshot[g], bi, ii);
                skip.push(range);
                inlined = true;
                break; // this block was split: its tail now lives in a new block
            }
            if !inlined {
                bi += 1;
            }
        }
    }
}

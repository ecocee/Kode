//! MIR: Kode's mid-level intermediate representation.
//!
//! The AST is a tree for people. MIR is a flat control-flow graph for the
//! compiler: functions made of basic blocks, simple instructions, and
//! explicit `retain` / `release` for reference-counted strings.
//! Every backend (Cranelift now, LLVM or others later) reads MIR only.

pub mod dump;
pub mod ir;
pub mod lower;
pub mod verify;

pub use ir::*;
pub use lower::lower;

//! MIR: Kode's mid-level intermediate representation.
//!
//! The AST is a tree for people. MIR is a flat control-flow graph for the
//! compiler: functions made of basic blocks, simple instructions, and
//! explicit `retain` / `release` for reference-counted strings.
//! Every backend (LLVM today) reads MIR only.

pub mod dump;
pub mod ir;
pub mod lower;
pub mod opt;
pub mod verify;

pub use ir::*;

/// Maps a byte offset to a line number (used for runtime error messages).
pub trait LineMap {
    fn line_of(&self, offset: usize) -> i64;
}

impl LineMap for str {
    fn line_of(&self, offset: usize) -> i64 {
        self[..offset.min(self.len())].matches('\n').count() as i64 + 1
    }
}

pub use lower::{lower, Mode};

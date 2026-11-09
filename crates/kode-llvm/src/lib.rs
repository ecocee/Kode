//! Kode MIR -> textual LLVM IR, for the experimental LLVM backend (ADR 0035).
//!
//! The crate reads `MirProgram` and nothing else: no AST, no type checker.
//! It writes `.ll` text and asks `clang` to build the object file; LLVM is a
//! build-time tool, never a Rust dependency.
//!
//! This is stage B1 of ADR 0035: integers, floats, bools, arguments and
//! returns, arithmetic and comparisons, branches, loops, direct calls, and the
//! runtime printing functions. Collections, classes, closures, errors, channels
//! and tasks come in later stages.

mod clang;
mod front;
mod lower;
mod rtlib;

pub use clang::{clang_program, compile_ir, ClangError};
pub use front::{load_program, program_from_source, Lines};
pub use lower::emit;
pub use rtlib::find_runtime_archive;

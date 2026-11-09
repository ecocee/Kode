//! Linking a Kode object file with the runtime archive.
//!
//! Code generation lives in `kode-llvm`: it turns MIR into textual LLVM IR and
//! asks clang for an object. This crate only knows how to hand that object and
//! the runtime to the platform C compiler, which is the linker driver Kode uses
//! on every supported platform.

mod link;

pub use link::{cc_program, link};

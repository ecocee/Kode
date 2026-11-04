//! Name resolution and type checking for Kode.
//!
//! Input: the AST. Output: `Checked`, which records the type of every
//! expression and the signature of every function, or a list of errors.

pub mod builtins;
pub mod modules;
mod check;
pub mod scope;
pub mod std_fns;
pub mod suggest;
pub mod types;

pub use check::{check, Checked, ClassInfo, ConstVal, EnumInfo, FieldInfo, FnSig};
pub use types::{assignable, Type};
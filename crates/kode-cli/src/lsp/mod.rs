//! `kode lsp`: a language server over stdio, so editors get the compiler's own
//! diagnostics, types and formatting.

pub mod features;
pub mod json;
mod nav;
pub mod position;
pub mod server;
pub mod walk;

pub use server::serve;

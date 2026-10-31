//! Source spans and compiler messages shared by every stage of the compiler.

pub mod diagnostic;
pub mod span;

pub use diagnostic::{json_escape, Diagnostic, Severity};
pub use span::Span;

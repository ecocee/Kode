//! Front end of the Kode compiler: source text -> tokens -> AST.

pub mod ast;
mod depth;
pub mod lexer;
pub mod parser;
pub mod token;

use kode_diagnostics::Diagnostic;

/// Lex and parse a whole source file, refusing trees deeper than the later
/// passes can walk.
pub fn parse_source(src: &str) -> Result<ast::Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}

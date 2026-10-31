//! Front end of the Kode compiler: source text -> tokens -> AST.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use kode_diagnostics::Diagnostic;

/// Lex and parse a whole source file.
pub fn parse_source(src: &str) -> Result<ast::Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}

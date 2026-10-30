use crate::{
    ast::{Program, Statement},
    diagnostics::KodeError,
};

/// Temporary execution engine for v0.1.
/// This module can later be replaced by native code generation.
pub fn execute(program: &Program) -> Result<(), KodeError> {
    for statement in &program.statements {
        match statement {
            Statement::Print {
                value,
                line: _,
                column: _,
            } => println!("{}", value),
        }
    }

    Ok(())
}

use kode_diagnostics::{Diagnostic, Span};

use crate::types::Type;

pub struct VarInfo {
    pub name: String,
    pub ty: Type,
    pub span: Span,
    pub used: bool,
}

/// A stack of block scopes. A variable lives until the end of the block
/// in which it was first assigned.
#[derive(Default)]
pub struct Scopes {
    stack: Vec<Vec<VarInfo>>,
}

impl Scopes {
    pub fn new() -> Self {
        Scopes { stack: Vec::new() }
    }

    pub fn push(&mut self) {
        self.stack.push(Vec::new());
    }

    /// Leave a block. Unused variables become warnings.
    pub fn pop(&mut self, warnings: &mut Vec<Diagnostic>) {
        if let Some(scope) = self.stack.pop() {
            for v in scope {
                if !v.used && !v.name.starts_with('_') {
                    warnings.push(
                        Diagnostic::warning(
                            "W0001",
                            format!("variable `{}` is assigned but never used", v.name),
                            v.span,
                        )
                        .with_help("remove it, or start the name with `_` if this is intentional"),
                    );
                }
            }
        }
    }

    fn find_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        self.stack
            .iter_mut()
            .rev()
            .flat_map(|s| s.iter_mut())
            .find(|v| v.name == name)
    }

    /// Look a variable up and mark it as read.
    pub fn use_var(&mut self, name: &str) -> Option<Type> {
        self.find_mut(name).map(|v| {
            v.used = true;
            v.ty
        })
    }

    /// Look a variable up without marking it as read.
    pub fn peek_var(&mut self, name: &str) -> Option<Type> {
        self.find_mut(name).map(|v| v.ty)
    }

    /// Declare a variable in the innermost scope.
    pub fn declare(&mut self, name: &str, ty: Type, span: Span, used: bool) {
        self.stack.last_mut().expect("a scope is open").push(VarInfo {
            name: name.to_string(),
            ty,
            span,
            used,
        });
    }

    pub fn names(&self) -> Vec<&str> {
        self.stack
            .iter()
            .flat_map(|s| s.iter().map(|v| v.name.as_str()))
            .collect()
    }
}

use std::collections::HashMap;

use kode_diagnostics::Diagnostic;
use kode_syntax::ast::*;

use crate::scope::Scopes;
use crate::types::Type;

mod expr;
mod item;
mod stmt;

#[derive(Clone, Debug)]
pub struct FnSig {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

/// Result of a successful check. Later stages read everything from here.
#[derive(Debug)]
pub struct Checked {
    /// Type of every expression, indexed by `ExprId`.
    pub expr_types: Vec<Type>,
    /// Every function, in source order. Index = position among `Item::Fn`.
    pub fns: Vec<FnSig>,
    pub fn_index: HashMap<String, usize>,
    pub consts: HashMap<String, (Type, ConstVal)>,
    pub warnings: Vec<Diagnostic>,
}

pub(crate) struct Checker {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub expr_types: Vec<Type>,
    pub fns: Vec<FnSig>,
    pub fn_index: HashMap<String, usize>,
    pub consts: HashMap<String, (Type, ConstVal)>,
    pub scopes: Scopes,
    pub ret_type: Type,
    pub in_fn: bool,
    pub loop_depth: usize,
}

pub fn check(program: &Program) -> Result<Checked, Vec<Diagnostic>> {
    let mut c = Checker {
        errors: Vec::new(),
        warnings: Vec::new(),
        expr_types: vec![Type::Unit; program.expr_count as usize],
        fns: Vec::new(),
        fn_index: HashMap::new(),
        consts: HashMap::new(),
        scopes: Scopes::new(),
        ret_type: Type::Unit,
        in_fn: false,
        loop_depth: 0,
    };

    c.collect_consts(program);
    c.collect_fns(program);

    // Function bodies.
    let mut index = 0;
    for item in &program.items {
        if let Item::Fn(f) = item {
            c.check_fn(f, index);
            index += 1;
        }
    }

    // The top-level script.
    c.scopes = Scopes::new();
    c.scopes.push();
    c.ret_type = Type::Unit;
    c.in_fn = false;
    c.loop_depth = 0;
    for item in &program.items {
        if let Item::Stmt(s) = item {
            c.stmt(s);
        }
    }
    let mut warnings = std::mem::take(&mut c.warnings);
    c.scopes.pop(&mut warnings);
    c.warnings = warnings;

    if !c.errors.is_empty() {
        return Err(c.errors);
    }
    Ok(Checked {
        expr_types: c.expr_types,
        fns: c.fns,
        fn_index: c.fn_index,
        consts: c.consts,
        warnings: c.warnings,
    })
}

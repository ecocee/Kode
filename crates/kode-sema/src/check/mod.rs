use std::collections::HashMap;

use kode_diagnostics::{Diagnostic, Span};
use kode_syntax::ast::*;

use crate::scope::Scopes;
use crate::types::Type;

mod expr;
mod instantiate;
mod item;
mod stmt;

#[derive(Clone, Debug)]
pub struct FnSig {
    /// Free functions: `name`. Methods: `Class.name`.
    pub name: String,
    /// Methods have an implicit first parameter called `this`.
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub class: Option<String>,
    /// Non-empty only for a generic declaration; an instantiation has none.
    pub type_params: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: String,
    pub ty: Type,
    pub default: Option<Expr>,
}

#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    /// method name -> index into `Checked::fns`
    pub methods: HashMap<String, usize>,
    /// Interfaces this class declares with `implements`.
    pub implements: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct IfaceMethod {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
}

#[derive(Clone, Debug)]
pub struct InterfaceInfo {
    pub name: String,
    pub methods: Vec<IfaceMethod>,
}

/// One `import "path" as alias` in the program. The loader lists every name
/// the file defines, and the public subset that the alias may reach.
#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub alias: String,
    pub path: String,
    pub exports: Vec<String>,
    pub all: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

/// What a closure literal needs to be compiled: its parameters, its result and
/// a copy of every variable that was visible where it was written.
#[derive(Clone, Debug)]
pub struct ClosureInfo {
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub captures: Vec<(String, Type)>,
}

/// Result of a successful check. Later stages read everything from here.
#[derive(Debug)]
pub struct Checked {
    /// Type of every expression, indexed by `ExprId`.
    pub expr_types: Vec<Type>,
    /// Every function in `Program::fn_decls()` order (free functions and methods).
    pub fns: Vec<FnSig>,
    pub fn_index: HashMap<String, usize>,
    /// Class 0 is the built-in `Error` class.
    pub classes: Vec<ClassInfo>,
    pub class_index: HashMap<String, usize>,
    pub enums: Vec<EnumInfo>,
    pub enum_index: HashMap<String, usize>,
    pub interfaces: Vec<InterfaceInfo>,
    pub interface_index: HashMap<String, usize>,
    pub modules: Vec<ModuleInfo>,
    pub module_index: HashMap<String, usize>,
    pub consts: HashMap<String, (Type, ConstVal)>,
    /// For each call: which argument fills each parameter (`None` = use the default).
    pub call_args: HashMap<ExprId, Vec<Option<usize>>>,
    /// Can this function raise an error (directly or through a call)?
    pub throws: Vec<bool>,
    /// Calls that go through a function value (`f(1)` where `f` is a variable).
    pub fn_value_calls: std::collections::HashSet<ExprId>,
    /// Closure literals, by expression id.
    pub closures: HashMap<ExprId, ClosureInfo>,
    /// The body of every function in `fns`, in the same order: declared
    /// functions and methods first, then each monomorphized instantiation.
    pub bodies: Vec<FnDecl>,
    /// For a generic call: the instantiated function it resolved to.
    pub generic_calls: HashMap<ExprId, usize>,
    /// For a constructor call on a generic class: the instance it built.
    pub class_ctor_calls: HashMap<ExprId, usize>,
    pub warnings: Vec<Diagnostic>,
}

pub(crate) struct Checker {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub expr_types: Vec<Type>,
    pub fns: Vec<FnSig>,
    pub fn_index: HashMap<String, usize>,
    pub classes: Vec<ClassInfo>,
    pub class_index: HashMap<String, usize>,
    pub enums: Vec<EnumInfo>,
    pub enum_index: HashMap<String, usize>,
    pub interfaces: Vec<InterfaceInfo>,
    pub interface_index: HashMap<String, usize>,
    pub modules: Vec<ModuleInfo>,
    pub module_index: HashMap<String, usize>,
    pub consts: HashMap<String, (Type, ConstVal)>,
    pub call_args: HashMap<ExprId, Vec<Option<usize>>>,
    pub fn_value_calls: std::collections::HashSet<ExprId>,
    pub closures: HashMap<ExprId, ClosureInfo>,
    /// Inside a closure literal: fields of `this` cannot be reached (E0242).
    pub in_closure: bool,
    /// Generic declarations, by name. They are never lowered themselves.
    pub generic_index: HashMap<String, usize>,
    pub generic_decls: Vec<FnDecl>,
    /// (generic index, concrete type names) -> instantiated function index.
    pub instances: Vec<((usize, Vec<String>), usize)>,
    /// Fresh expression ids for instantiated bodies.
    pub next_expr_id: u32,
    /// How many entries of `bodies` have been checked so far.
    pub checked_bodies: usize,
    pub generic_calls: HashMap<ExprId, usize>,
    /// Constructor calls that resolved to a generic class instance, and which
    /// class instance each one is (ADR 0040).
    pub class_ctor_calls: HashMap<ExprId, usize>,
    /// Generic class declarations, by name. They are never instantiated
    /// themselves and never lowered; the source name is not a class in
    /// `class_index`, which is what makes a bare `Box` an E0423.
    pub generic_class_index: HashMap<String, usize>,
    pub generic_classes: Vec<ClassDecl>,
    /// Bodies of `fns`, in the same order.
    pub bodies: Vec<FnDecl>,
    pub direct_throw: Vec<bool>,
    pub calls: Vec<Vec<usize>>,
    pub scopes: Scopes,
    /// Locals proven not to be `none` in this block: (scope depth, name, type).
    narrowed: Vec<(usize, String, Type)>,
    pub ret_type: Type,
    pub in_fn: bool,
    pub loop_depth: usize,
    pub cur_class: Option<usize>,
    pub cur_fn: Option<usize>,
}

impl Checker {
    /// Interface values are class references, so a class can go anywhere its
    /// interfaces are expected. Everything else uses the plain type rules.
    pub(crate) fn assignable(&self, from: &Type, to: &Type) -> bool {
        if from == to || crate::types::assignable(from, to) {
            return true;
        }
        match (from, to) {
            (Type::Class(c), Type::Interface(i)) => self
                .class_index
                .get(c)
                .map(|&ci| self.classes[ci].implements.iter().any(|x| x == i))
                .unwrap_or(false),
            (_, Type::Opt(inner)) => self.assignable(from, inner),
            _ => false,
        }
    }

    /// Record the type of an expression, growing the table for cloned bodies.
    pub(crate) fn set_expr_type(&mut self, id: ExprId, ty: Type) {
        let i = id as usize;
        if self.expr_types.len() <= i {
            self.expr_types.resize(i + 1, Type::Unit);
        }
        self.expr_types[i] = ty;
    }

    fn new(program: &Program) -> Self {
        let n = program.expr_count as usize;
        let mut expr_types = vec![Type::Unit; n + 1];
        expr_types[n] = Type::Int;
        let error_class = ClassInfo {
            name: "Error".into(),
            fields: vec![
                FieldInfo {
                    name: "message".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "code".into(),
                    ty: Type::Int,
                    default: Some(Expr {
                        id: n as ExprId,
                        span: Span::default(),
                        kind: ExprKind::Int(0),
                    }),
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        // Built-in `Json` (a dynamic JSON value) and `HttpResponse`.
        // Field order and types are mirrored by `kode-rt/src/json.rs` and
        // `kode-rt/src/http_client.rs`; change both together.
        let json = Type::Class("Json".into());
        let json_class = ClassInfo {
            name: "Json".into(),
            fields: vec![
                FieldInfo {
                    name: "kind".into(),
                    ty: Type::Enum("JsonKind".into()),
                    default: None,
                },
                FieldInfo {
                    name: "int_value".into(),
                    ty: Type::Int,
                    default: None,
                },
                FieldInfo {
                    name: "float_value".into(),
                    ty: Type::Float,
                    default: None,
                },
                FieldInfo {
                    name: "string_value".into(),
                    ty: Type::Opt(Box::new(Type::Str)),
                    default: None,
                },
                FieldInfo {
                    name: "list_value".into(),
                    ty: Type::Opt(Box::new(Type::List(Box::new(json.clone())))),
                    default: None,
                },
                FieldInfo {
                    name: "map_value".into(),
                    ty: Type::Opt(Box::new(Type::Map(Box::new(json)))),
                    default: None,
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        let response_class = ClassInfo {
            name: "HttpResponse".into(),
            fields: vec![
                FieldInfo {
                    name: "status".into(),
                    ty: Type::Int,
                    default: None,
                },
                FieldInfo {
                    name: "body".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "headers".into(),
                    ty: Type::Map(Box::new(Type::Str)),
                    default: None,
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        // `Request`, `Response` and `Router` are the simple web API layer.
        // Field order is mirrored by `kode-rt/src/router.rs`; change both together.
        // The result of `exec`. Order and types mirror `kode-rt/src/process.rs`.
        let process_result = ClassInfo {
            name: "ProcessResult".into(),
            fields: vec![
                FieldInfo {
                    name: "code".into(),
                    ty: Type::Int,
                    default: None,
                },
                FieldInfo {
                    name: "stdout".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "stderr".into(),
                    ty: Type::Str,
                    default: None,
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        let request_class = ClassInfo {
            name: "Request".into(),
            fields: vec![
                FieldInfo {
                    name: "method".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "path".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "params".into(),
                    ty: Type::Map(Box::new(Type::Str)),
                    default: None,
                },
                FieldInfo {
                    name: "query".into(),
                    ty: Type::Map(Box::new(Type::Str)),
                    default: None,
                },
                FieldInfo {
                    name: "headers".into(),
                    ty: Type::Map(Box::new(Type::Str)),
                    default: None,
                },
                FieldInfo {
                    name: "body".into(),
                    ty: Type::Str,
                    default: None,
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        let reply_class = ClassInfo {
            name: "Response".into(),
            fields: vec![
                FieldInfo {
                    name: "status".into(),
                    ty: Type::Int,
                    default: None,
                },
                FieldInfo {
                    name: "body".into(),
                    ty: Type::Str,
                    default: None,
                },
                FieldInfo {
                    name: "content_type".into(),
                    ty: Type::Str,
                    default: None,
                },
            ],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        let router_class = ClassInfo {
            name: "Router".into(),
            fields: vec![FieldInfo {
                name: "id".into(),
                ty: Type::Int,
                default: None,
            }],
            methods: HashMap::new(),
            implements: Vec::new(),
        };
        let mut class_index = HashMap::new();
        class_index.insert("Error".to_string(), 0);
        class_index.insert("Json".to_string(), 1);
        class_index.insert("HttpResponse".to_string(), 2);
        class_index.insert("Request".to_string(), 3);
        class_index.insert("Response".to_string(), 4);
        class_index.insert("Router".to_string(), 5);
        class_index.insert("ProcessResult".to_string(), 6);
        let json_kind = EnumInfo {
            name: "JsonKind".into(),
            variants: ["Null", "Bool", "Int", "Float", "Str", "List", "Map"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let mut enum_index = HashMap::new();
        enum_index.insert("JsonKind".to_string(), 0);
        Checker {
            errors: Vec::new(),
            warnings: Vec::new(),
            expr_types,
            fns: Vec::new(),
            fn_index: HashMap::new(),
            classes: vec![
                error_class,
                json_class,
                response_class,
                request_class,
                reply_class,
                router_class,
                process_result,
            ],
            class_index,
            enums: vec![json_kind],
            enum_index,
            interfaces: Vec::new(),
            interface_index: HashMap::new(),
            modules: Vec::new(),
            module_index: HashMap::new(),
            consts: HashMap::new(),
            call_args: HashMap::new(),
            fn_value_calls: Default::default(),
            closures: HashMap::new(),
            in_closure: false,
            generic_index: HashMap::new(),
            generic_decls: Vec::new(),
            class_ctor_calls: HashMap::new(),
            generic_class_index: HashMap::new(),
            generic_classes: Vec::new(),
            instances: Vec::new(),
            next_expr_id: program.expr_count + 1,
            bodies: Vec::new(),
            checked_bodies: 0,
            generic_calls: HashMap::new(),
            direct_throw: Vec::new(),
            calls: Vec::new(),
            scopes: Scopes::new(),
            narrowed: Vec::new(),
            ret_type: Type::Unit,
            in_fn: false,
            loop_depth: 0,
            cur_class: None,
            cur_fn: None,
        }
    }

    pub(crate) fn pop_scope(&mut self) {
        let mut warnings = std::mem::take(&mut self.warnings);
        self.scopes.pop(&mut warnings);
        self.warnings = warnings;
        let depth = self.scopes.depth();
        self.narrowed.retain(|(d, _, _)| *d <= depth);
    }

    /// Remember that `name` is a `ty` (not `none`) until this block ends or the
    /// variable is assigned.
    pub(crate) fn narrow(&mut self, name: &str, ty: Type) {
        let depth = self.scopes.depth();
        self.narrowed
            .retain(|(d, n, _)| !(*d == depth && n == name));
        self.narrowed.push((depth, name.to_string(), ty));
    }

    /// The narrowed type of a local, if a check proved it is not `none`.
    pub(crate) fn narrowed_type(&self, name: &str) -> Option<Type> {
        self.narrowed
            .iter()
            .rev()
            .find(|(_, n, _)| n == name)
            .map(|(_, _, t)| t.clone())
    }

    /// Check every body not checked yet, including ones instantiated by a body
    /// checked in this pass.
    fn check_bodies(&mut self) {
        while self.checked_bodies < self.bodies.len() {
            let index = self.checked_bodies;
            let body = self.bodies[index].clone();
            let class = self.fns[index].class.clone();
            self.check_fn(&body, index, class.as_deref());
            self.checked_bodies += 1;
        }
    }

    pub(crate) fn reset_context(&mut self) {
        self.scopes = Scopes::new();
        self.scopes.push();
        self.narrowed.clear();
        self.ret_type = Type::Unit;
        self.in_fn = false;
        self.loop_depth = 0;
        self.cur_class = None;
        self.cur_fn = None;
    }
}

pub fn check(program: &Program, modules: &[ModuleInfo]) -> Result<Checked, Vec<Diagnostic>> {
    let mut c = Checker::new(program);
    c.modules = modules.to_vec();
    c.collect(program);

    // Function and method bodies. A generic call appends an instantiation to
    // `bodies` while a body is being checked, so each pass keeps going until no
    // new instantiation appears.
    c.check_bodies();
    c.check_field_defaults(program);
    c.check_bodies();

    // Test bodies.
    for item in &program.items {
        if let Item::Test(t) = item {
            c.reset_context();
            c.in_fn = true;
            for s in &t.body {
                c.stmt(s);
            }
            c.pop_scope();
        }
    }
    c.check_bodies();

    // The top-level script.
    c.reset_context();
    for item in &program.items {
        if let Item::Stmt(s) = item {
            c.stmt(s);
        }
    }
    c.pop_scope();
    // Top-level calls may instantiate a generic function too.
    c.check_bodies();

    if !c.errors.is_empty() {
        return Err(c.errors);
    }

    // A function throws if it throws directly or calls one that does.
    let mut throws = c.direct_throw.clone();
    loop {
        let mut changed = false;
        for i in 0..throws.len() {
            if !throws[i] && c.calls[i].iter().any(|&j| throws[j]) {
                throws[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    Ok(Checked {
        expr_types: c.expr_types,
        fns: c.fns,
        fn_index: c.fn_index,
        classes: c.classes,
        class_index: c.class_index,
        enums: c.enums,
        enum_index: c.enum_index,
        interfaces: c.interfaces,
        interface_index: c.interface_index,
        modules: c.modules,
        module_index: c.module_index,
        consts: c.consts,
        call_args: c.call_args,
        fn_value_calls: c.fn_value_calls,
        closures: c.closures,
        bodies: c.bodies,
        generic_calls: c.generic_calls,
        class_ctor_calls: c.class_ctor_calls,
        throws,
        warnings: c.warnings,
    })
}

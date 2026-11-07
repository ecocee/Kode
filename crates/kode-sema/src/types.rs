#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    /// The result of a call that returns nothing, such as `print(...)`.
    Unit,
    List(Box<Type>),
    /// A map with string keys; this is the value type.
    Map(Box<Type>),
    Class(String),
    Interface(String),
    Enum(String),
    /// `T?`: only for types that live on the heap (string, list, map, class).
    Opt(Box<Type>),
    /// The type of the `none` literal.
    NoneT,
    /// `Task<T>`: a running task that will produce a `T`.
    Task(Box<Type>),
    /// `Channel<T>`: a queue that can be shared between tasks. A channel made by
    /// `channel()` has payload `none` until a declared type gives it one (E0240).
    Chan(Box<Type>),
    /// `fn(int) -> int`: a function value. The parameters and the return type.
    Fn(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Int => "int".into(),
            Type::Float => "float".into(),
            Type::Bool => "bool".into(),
            Type::Str => "string".into(),
            Type::Unit => "nothing".into(),
            Type::List(t) => format!("[{}]", t.name()),
            Type::Map(t) => format!("{{string: {}}}", t.name()),
            Type::Class(n) | Type::Interface(n) | Type::Enum(n) => n.clone(),
            Type::Opt(t) => format!("{}?", t.name()),
            Type::NoneT => "none".into(),
            Type::Task(t) => format!("Task<{}>", t.name()),
            Type::Chan(t) => format!("Channel<{}>", t.name()),
            Type::Fn(params, ret) => {
                let ps: Vec<String> = params.iter().map(|p| p.name()).collect();
                format!("fn({}) -> {}", ps.join(", "), ret.name())
            }
        }
    }

    /// "an int", "a string", "`[int]`": used in messages.
    pub fn article(&self) -> String {
        match self {
            Type::Int => "an int".into(),
            Type::Float => "a float".into(),
            Type::Bool => "a bool".into(),
            Type::Str => "a string".into(),
            Type::Unit => "nothing".into(),
            other => format!("`{}`", other.name()),
        }
    }

    /// Values that live on the heap and are reference counted.
    pub fn is_ref(&self) -> bool {
        matches!(
            self,
            Type::Str
                | Type::List(_)
                | Type::Map(_)
                | Type::Class(_)
                | Type::Interface(_)
                | Type::Opt(_)
                | Type::NoneT
                | Type::Task(_)
                | Type::Chan(_)
                | Type::Fn(_, _)
        )
    }

    /// `[]` and `{}` have no element type until they meet a declared type.
    pub fn is_empty_collection(&self) -> bool {
        matches!(self, Type::List(t) | Type::Map(t) if **t == Type::NoneT)
    }

    pub fn from_name(name: &str) -> Option<Type> {
        match name {
            "int" => Some(Type::Int),
            "float" => Some(Type::Float),
            "bool" => Some(Type::Bool),
            "string" => Some(Type::Str),
            _ => None,
        }
    }
}

/// Can a value of type `from` be stored where `to` is expected?
/// An optional number or bool (`int?`, `float?`, `bool?`). These are a pair
/// (value and *present* flag), never a box, so they need their own handling
/// wherever a value is stored or printed. Returns the inner type.
pub fn is_opt_scalar(t: &Type) -> Option<Type> {
    match t {
        Type::Opt(i) if matches!(**i, Type::Int | Type::Float | Type::Bool) => Some((**i).clone()),
        _ => None,
    }
}

/// The descriptor `kode_repr` uses to print a list or map, or `None` when the
/// collection holds a value that cannot be printed yet.
///
/// The descriptor is built here, once, so the checker and the lowering agree:
/// the checker uses `Some`/`None` to accept or reject a `print`, and the
/// lowering puts the text in the program. Keeping it in one function is what
/// stops the two from drifting apart.
pub fn repr_descriptor(t: &Type) -> Option<String> {
    match t {
        Type::Int => Some("i".to_string()),
        Type::Float => Some("f".to_string()),
        Type::Bool => Some("b".to_string()),
        Type::Str => Some("s".to_string()),
        Type::List(inner) => Some(format!("[{}]", repr_descriptor(inner)?)),
        Type::Map(val) => Some(format!("{{{}}}", repr_descriptor(val)?)),
        _ => None,
    }
}

/// Resolve a type annotation without reporting errors, for later stages that
/// need the declared type of a variable (the AST keeps only the annotation).
pub fn ann_type(ann: &kode_syntax::ast::TypeAnn, checked: &crate::Checked) -> Option<Type> {
    use kode_syntax::ast::TypeAnnKind;
    match &ann.kind {
        TypeAnnKind::Name(n) => Type::from_name(n)
            .or_else(|| checked.class_index.get(n).map(|_| Type::Class(n.clone())))
            .or_else(|| {
                checked
                    .interface_index
                    .get(n)
                    .map(|_| Type::Interface(n.clone()))
            })
            .or_else(|| checked.enum_index.get(n).map(|_| Type::Enum(n.clone()))),
        TypeAnnKind::List(i) => Some(Type::List(Box::new(ann_type(i, checked)?))),
        TypeAnnKind::Map(i) => Some(Type::Map(Box::new(ann_type(i, checked)?))),
        TypeAnnKind::Opt(i) => Some(Type::Opt(Box::new(ann_type(i, checked)?))),
        TypeAnnKind::Task(i) => Some(Type::Task(Box::new(ann_type(i, checked)?))),
        TypeAnnKind::Channel(i) => Some(Type::Chan(Box::new(ann_type(i, checked)?))),
        TypeAnnKind::Fn(params, ret) => {
            let ps = params
                .iter()
                .map(|p| ann_type(p, checked))
                .collect::<Option<Vec<_>>>()?;
            Some(Type::Fn(ps, Box::new(ann_type(ret, checked)?)))
        }
    }
}

pub fn assignable(from: &Type, to: &Type) -> bool {
    if from == to {
        return true;
    }
    match (from, to) {
        (Type::NoneT, Type::Opt(_)) => true,
        (Type::List(a), Type::List(_)) | (Type::Map(a), Type::Map(_)) if **a == Type::NoneT => true,
        // `channel()` has no payload until a declared type meets it, like `[]`.
        (Type::Chan(a), Type::Chan(_)) if **a == Type::NoneT => true,
        (_, Type::Opt(inner)) => assignable(from, inner),
        _ => false,
    }
}

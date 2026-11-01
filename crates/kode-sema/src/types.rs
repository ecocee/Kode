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
    Enum(String),
    /// `T?`: only for types that live on the heap (string, list, map, class).
    Opt(Box<Type>),
    /// The type of the `none` literal.
    NoneT,
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
            Type::Class(n) | Type::Enum(n) => n.clone(),
            Type::Opt(t) => format!("{}?", t.name()),
            Type::NoneT => "none".into(),
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
            Type::Str | Type::List(_) | Type::Map(_) | Type::Class(_) | Type::Opt(_) | Type::NoneT
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
pub fn assignable(from: &Type, to: &Type) -> bool {
    if from == to {
        return true;
    }
    match (from, to) {
        (Type::NoneT, Type::Opt(_)) => true,
        (Type::List(a), Type::List(_)) | (Type::Map(a), Type::Map(_)) if **a == Type::NoneT => true,
        (_, Type::Opt(inner)) => assignable(from, inner),
        _ => false,
    }
}
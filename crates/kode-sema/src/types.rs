#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    /// The result of a call that returns nothing, such as `print(...)`.
    Unit,
}

impl Type {
    pub fn name(self) -> &'static str {
        match self {
            Type::Int => "int",
            Type::Float => "float",
            Type::Bool => "bool",
            Type::Str => "string",
            Type::Unit => "nothing",
        }
    }

    /// "an int", "a string": used in messages.
    pub fn article(self) -> &'static str {
        match self {
            Type::Int => "an int",
            Type::Float => "a float",
            Type::Bool => "a bool",
            Type::Str => "a string",
            Type::Unit => "nothing",
        }
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

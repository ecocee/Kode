/// Functions that are part of the language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Print,
    Len,
    Int,
    Float,
    Str,
    Assert,
}

pub const BUILTIN_NAMES: &[&str] = &["print", "len", "int", "float", "string", "assert"];

pub fn lookup(name: &str) -> Option<Builtin> {
    Some(match name {
        "print" => Builtin::Print,
        "len" => Builtin::Len,
        "int" => Builtin::Int,
        "float" => Builtin::Float,
        "string" => Builtin::Str,
        "assert" => Builtin::Assert,
        _ => return None,
    })
}

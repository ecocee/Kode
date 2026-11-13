/// Functions that are part of the language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Print,
    Len,
    Bytes,
    Int,
    Float,
    Str,
    Assert,
    Channel,
    Send,
    Recv,
    Close,
}

pub const BUILTIN_NAMES: &[&str] = &[
    "print", "len", "bytes", "int", "float", "string", "assert", "channel", "send", "recv", "close",
];

pub fn lookup(name: &str) -> Option<Builtin> {
    Some(match name {
        "print" => Builtin::Print,
        "len" => Builtin::Len,
        "bytes" => Builtin::Bytes,
        "int" => Builtin::Int,
        "float" => Builtin::Float,
        "string" => Builtin::Str,
        "assert" => Builtin::Assert,
        "channel" => Builtin::Channel,
        "send" => Builtin::Send,
        "recv" => Builtin::Recv,
        "close" => Builtin::Close,
        _ => return None,
    })
}

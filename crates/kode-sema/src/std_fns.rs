//! The standard library functions that are implemented by the runtime.
//! One table row is all the compiler needs to know about a function:
//! its name, receiver, parameter and result types, and the runtime symbol.

use crate::types::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recv {
    /// a plain function: `read_file(path)`
    Free,
    Str,
    ListStr,
    ListInt,
    ListFloat,
    /// any list except `[bool]`
    ListAny,
    Map,
    /// the built-in dynamic JSON value
    Json,
    /// the request a route handler receives
    Request,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StdTy {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    ListStr,
    OptStr,
    ListJson,
    MapJson,
    MapStrStr,
    Json,
    Resp,
    /// the result of `exec`: `code`, `stdout`, `stderr`
    ProcResult,
    /// the item type of the list the method is called on
    Elem,
    /// the item type, passed as raw bits (`insert` into a `[float]`)
    ElemBits,
    /// the item type of the list the method returns (`values()`)
    ListElem,
}

#[derive(Debug)]
pub struct StdFn {
    pub name: &'static str,
    pub recv: Recv,
    pub params: &'static [StdTy],
    pub ret: StdTy,
    /// can raise an error (the caller must check for it)
    pub throws: bool,
    pub sym: &'static str,
}

use Recv::*;
use StdTy as T;

macro_rules! f {
    ($name:expr, $recv:expr, [$($p:expr),*], $ret:expr, $throws:expr, $sym:expr) => {
        StdFn { name: $name, recv: $recv, params: &[$($p),*], ret: $ret, throws: $throws, sym: $sym }
    };
}

pub static STD: &[StdFn] = &[
    // files, environment, time
    f!("read_file", Free, [T::Str], T::Str, true, "kode_read_file"),
    f!("write_file", Free, [T::Str, T::Str], T::Unit, true, "kode_write_file"),
    f!("args", Free, [], T::ListStr, false, "kode_args"),
    f!("env", Free, [T::Str], T::OptStr, false, "kode_env"),
    f!("exit", Free, [T::Int], T::Unit, false, "kode_exit"),
    f!("sleep", Free, [T::Int], T::Unit, false, "kode_sleep"),
    f!("now_ms", Free, [], T::Int, false, "kode_now_ms"),
    f!("chr", Free, [T::Int], T::Str, false, "kode_chr"),
    // processes: run a program directly, without a shell
    f!("exec", Free, [T::Str, T::ListStr, T::Str], T::ProcResult, true, "kode_proc_exec"),
    // strings
    f!("size", Str, [], T::Int, false, "kode_str_size"),
    f!("byte_at", Str, [T::Int], T::Int, false, "kode_str_byte_at"),
    f!("slice", Str, [T::Int, T::Int], T::Str, true, "kode_str_slice"),
    f!("contains", Str, [T::Str], T::Bool, false, "kode_str_contains"),
    f!("starts_with", Str, [T::Str], T::Bool, false, "kode_str_starts_with"),
    f!("ends_with", Str, [T::Str], T::Bool, false, "kode_str_ends_with"),
    f!("find", Str, [T::Str], T::Int, false, "kode_str_find"),
    f!("upper", Str, [], T::Str, false, "kode_str_upper"),
    f!("lower", Str, [], T::Str, false, "kode_str_lower"),
    f!("trim", Str, [], T::Str, false, "kode_str_trim"),
    f!("replace", Str, [T::Str, T::Str], T::Str, false, "kode_str_replace"),
    f!("repeat", Str, [T::Int], T::Str, false, "kode_str_repeat"),
    f!("split", Str, [T::Str], T::ListStr, false, "kode_str_split"),
    f!("to_int", Str, [], T::Int, true, "kode_str_to_int"),
    f!("to_float", Str, [], T::Float, true, "kode_str_to_float"),
    f!("lines", Str, [], T::ListStr, false, "kode_str_lines"),
    f!("char_at", Str, [T::Int], T::Str, true, "kode_str_char_at"),
    f!("is_empty", Str, [], T::Bool, false, "kode_str_is_empty"),
    f!("count", Str, [T::Str], T::Int, false, "kode_str_count"),
    // lists
    f!("join", ListStr, [T::Str], T::Str, false, "kode_list_join"),
    f!("sort", ListStr, [], T::Unit, false, "kode_list_sort_str"),
    f!("sort", ListInt, [], T::Unit, false, "kode_list_sort_int"),
    f!("sort", ListFloat, [], T::Unit, false, "kode_list_sort_float"),
    f!("pop", ListAny, [], T::Elem, false, "kode_list_pop"),
    f!("clear", ListAny, [], T::Unit, false, "kode_list_clear"),
    f!("contains", ListStr, [T::Str], T::Bool, false, "kode_list_contains_str"),
    f!("contains", ListInt, [T::Int], T::Bool, false, "kode_list_contains_int"),
    f!("contains", ListFloat, [T::Float], T::Bool, false, "kode_list_contains_float"),
    f!("index_of", ListStr, [T::Str], T::Int, false, "kode_list_index_str"),
    f!("index_of", ListInt, [T::Int], T::Int, false, "kode_list_index_int"),
    f!("index_of", ListFloat, [T::Float], T::Int, false, "kode_list_index_float"),
    f!("reverse", ListAny, [], T::Unit, false, "kode_list_reverse"),
    f!("insert", ListAny, [T::Int, T::ElemBits], T::Unit, true, "kode_list_insert"),
    f!("remove_at", ListAny, [T::Int], T::Unit, true, "kode_list_remove_at"),
    f!("first", ListAny, [], T::Elem, true, "kode_list_first"),
    f!("last", ListAny, [], T::Elem, true, "kode_list_last"),
    // maps
    f!("remove", Map, [T::Str], T::Unit, false, "kode_map_remove"),
    f!("values", Map, [], T::ListElem, false, "kode_map_values"),
    // JSON: build, read, write
    f!("json_parse", Free, [T::Str], T::Json, true, "kode_json_parse"),
    f!("json_null", Free, [], T::Json, false, "kode_json_null"),
    f!("json_bool", Free, [T::Bool], T::Json, false, "kode_json_bool"),
    f!("json_int", Free, [T::Int], T::Json, false, "kode_json_int"),
    f!("json_float", Free, [T::Float], T::Json, false, "kode_json_float"),
    f!("json_string", Free, [T::Str], T::Json, false, "kode_json_string"),
    f!("json_list", Free, [T::ListJson], T::Json, false, "kode_json_list"),
    f!("json_map", Free, [T::MapJson], T::Json, false, "kode_json_map"),
    f!("json_encode", Free, [T::Json], T::Str, true, "kode_json_encode"),
    // math (the `math.*` names are aliases for these)
    f!("math_sqrt", Free, [T::Float], T::Float, false, "kode_math_sqrt"),
    f!("math_abs", Free, [T::Float], T::Float, false, "kode_math_abs"),
    f!("math_floor", Free, [T::Float], T::Float, false, "kode_math_floor"),
    f!("math_ceil", Free, [T::Float], T::Float, false, "kode_math_ceil"),
    f!("math_round", Free, [T::Float], T::Float, false, "kode_math_round"),
    f!("math_sin", Free, [T::Float], T::Float, false, "kode_math_sin"),
    f!("math_cos", Free, [T::Float], T::Float, false, "kode_math_cos"),
    f!("math_tan", Free, [T::Float], T::Float, false, "kode_math_tan"),
    f!("math_log", Free, [T::Float], T::Float, false, "kode_math_log"),
    f!("math_exp", Free, [T::Float], T::Float, false, "kode_math_exp"),
    f!("math_pow", Free, [T::Float, T::Float], T::Float, false, "kode_math_pow"),
    f!("math_min", Free, [T::Float, T::Float], T::Float, false, "kode_math_min"),
    f!("math_max", Free, [T::Float, T::Float], T::Float, false, "kode_math_max"),
    f!("is_null", Json, [], T::Bool, false, "kode_json_is_null"),
    f!("as_bool", Json, [], T::Bool, true, "kode_json_as_bool"),
    f!("as_int", Json, [], T::Int, true, "kode_json_as_int"),
    f!("as_float", Json, [], T::Float, true, "kode_json_as_float"),
    f!("as_string", Json, [], T::Str, true, "kode_json_as_string"),
    f!("as_list", Json, [], T::ListJson, true, "kode_json_as_list"),
    f!("as_map", Json, [], T::MapJson, true, "kode_json_as_map"),
    f!("len", Json, [], T::Int, true, "kode_json_len"),
    f!("has", Json, [T::Str], T::Bool, true, "kode_json_has"),
    f!("encode", Json, [], T::Str, true, "kode_json_encode"),
    // HTTP client (plain HTTP; TLS is not implemented yet)
    f!("http_get", Free, [T::Str], T::Resp, true, "kode_http_get"),
    f!("http_post", Free, [T::Str, T::Str, T::Str], T::Resp, true, "kode_http_post"),
    f!("http_request", Free, [T::Str, T::Str, T::MapStrStr, T::Str], T::Resp, true, "kode_http_request"),
    // web API helpers: build JSON objects, build responses, read a request body
    f!("json_obj", Free, [], T::MapJson, false, "kode_json_obj"),
    f!("response_json", Free, [T::Json], T::Resp, true, "kode_response_json"),
    f!("response_text", Free, [T::Str], T::Resp, false, "kode_response_text"),
    f!("response", Free, [T::Int, T::Str, T::Str], T::Resp, false, "kode_response_new"),
    f!("json", Request, [], T::Json, true, "kode_request_json"),
];

pub fn find_free(name: &str) -> Option<&'static StdFn> {
    STD.iter().find(|f| f.recv == Free && f.name == name)
}

/// Which receiver kinds a value of this type can be used as.
fn recvs_of(t: &Type) -> Vec<Recv> {
    match t {
        Type::Str => vec![Str],
        Type::List(e) => match **e {
            Type::Str => vec![ListStr, ListAny],
            Type::Int => vec![ListInt, ListAny],
            Type::Float => vec![ListFloat, ListAny],
            Type::Bool => vec![],
            _ => vec![ListAny],
        },
        Type::Map(_) => vec![Map],
        Type::Class(n) if n == "Json" => vec![Json],
        Type::Class(n) if n == "Request" => vec![Request],
        _ => vec![],
    }
}

pub fn find_method(recv: &Type, name: &str) -> Option<&'static StdFn> {
    let kinds = recvs_of(recv);
    STD.iter().find(|f| f.name == name && f.recv != Free && kinds.contains(&f.recv))
}

/// The language type of a table type (`elem` is the list item type for `Elem`).
pub fn to_type(t: StdTy, elem: Option<&Type>) -> Type {
    match t {
        StdTy::Int => Type::Int,
        StdTy::Float => Type::Float,
        StdTy::Bool => Type::Bool,
        StdTy::Str => Type::Str,
        StdTy::Unit => Type::Unit,
        StdTy::ListStr => Type::List(Box::new(Type::Str)),
        StdTy::OptStr => Type::Opt(Box::new(Type::Str)),
        StdTy::ListJson => Type::List(Box::new(Type::Class("Json".into()))),
        StdTy::MapJson => Type::Map(Box::new(Type::Class("Json".into()))),
        StdTy::MapStrStr => Type::Map(Box::new(Type::Str)),
        StdTy::Json => Type::Class("Json".into()),
        StdTy::Resp => Type::Class("HttpResponse".into()),
        StdTy::ProcResult => Type::Class("ProcessResult".into()),
        StdTy::Elem | StdTy::ElemBits => elem.cloned().unwrap_or(Type::Int),
        StdTy::ListElem => Type::List(Box::new(elem.cloned().unwrap_or(Type::Int))),
    }
}
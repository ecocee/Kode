//! Python-style namespaces for the standard library: `json.parse(text)`,
//! `math.sqrt(x)`, `http.get(url)`, `fs.read(path)`, `time.now_ms()`.
//!
//! Every entry is an alias for a function in `std_fns`, so namespaces add no
//! runtime code. A user variable with the same name shadows the module.

use crate::std_fns::{self, StdFn};

pub struct Alias {
    pub module: &'static str,
    pub name: &'static str,
    pub target: &'static str,
}

pub static ALIASES: &[Alias] = &[
    Alias { module: "json", name: "parse", target: "json_parse" },
    Alias { module: "json", name: "stringify", target: "json_encode" },
    Alias { module: "json", name: "object", target: "json_obj" },
    Alias { module: "json", name: "null", target: "json_null" },
    Alias { module: "json", name: "bool", target: "json_bool" },
    Alias { module: "json", name: "int", target: "json_int" },
    Alias { module: "json", name: "float", target: "json_float" },
    Alias { module: "json", name: "string", target: "json_string" },
    Alias { module: "json", name: "list", target: "json_list" },
    Alias { module: "json", name: "map", target: "json_map" },
    Alias { module: "http", name: "get", target: "http_get" },
    Alias { module: "http", name: "post", target: "http_post" },
    Alias { module: "http", name: "request", target: "http_request" },
    Alias { module: "fs", name: "read", target: "read_file" },
    Alias { module: "fs", name: "write", target: "write_file" },
    Alias { module: "time", name: "now_ms", target: "now_ms" },
    Alias { module: "time", name: "sleep", target: "sleep" },
    Alias { module: "math", name: "sqrt", target: "math_sqrt" },
    Alias { module: "math", name: "abs", target: "math_abs" },
    Alias { module: "math", name: "floor", target: "math_floor" },
    Alias { module: "math", name: "ceil", target: "math_ceil" },
    Alias { module: "math", name: "round", target: "math_round" },
    Alias { module: "math", name: "pow", target: "math_pow" },
    Alias { module: "math", name: "min", target: "math_min" },
    Alias { module: "math", name: "max", target: "math_max" },
    Alias { module: "math", name: "sin", target: "math_sin" },
    Alias { module: "math", name: "cos", target: "math_cos" },
    Alias { module: "math", name: "tan", target: "math_tan" },
    Alias { module: "math", name: "log", target: "math_log" },
    Alias { module: "math", name: "exp", target: "math_exp" },
];

pub fn function(module: &str, name: &str) -> Option<&'static StdFn> {
    ALIASES
        .iter()
        .find(|a| a.module == module && a.name == name)
        .and_then(|a| std_fns::find_free(a.target))
}

/// Module constants such as `math.pi`.
pub fn constant(module: &str, name: &str) -> Option<f64> {
    match (module, name) {
        ("math", "pi") => Some(std::f64::consts::PI),
        ("math", "e") => Some(std::f64::consts::E),
        _ => None,
    }
}

//! JSON values for the standard library.
//!
//! A JSON value is an instance of the built-in `Json` class declared in
//! `kode-sema/src/check/mod.rs`. That definition and this layout must match:
//!
//! ```text
//! 0 kind: JsonKind   1 int_value: int   2 float_value: float
//! 3 string_value: string?   4 list_value: [Json]?   5 map_value: {string: Json}?
//! ```
//!
//! Fields 3, 4 and 5 are heap values, so the refmask is `0b111000`.
//! The parser lives in `json_parse.rs`; it uses the `make_*` / `wrap_*` helpers
//! and hands ownership of the finished value to the caller.

use crate::error::throw_msg;
use crate::list::{kode_list_get, kode_list_len};
use crate::map::{entries, kode_map_get, kode_map_has, kode_map_len};
use crate::object::{kode_obj_new, kode_retain, Header};
use crate::string::{bytes, from_bytes};

pub(crate) const NULL: i64 = 0;
pub(crate) const BOOL: i64 = 1;
pub(crate) const INT: i64 = 2;
pub(crate) const FLOAT: i64 = 3;
pub(crate) const STR: i64 = 4;
pub(crate) const LIST: i64 = 5;
pub(crate) const MAP: i64 = 6;

const NFIELDS: i64 = 6;
const REFMASK: i64 = 0b111000;
const MAX_DEPTH: usize = 256;

unsafe fn field(p: *mut Header, i: usize) -> *mut u8 {
    (p as *mut u8).add(24 + 8 * i)
}

pub(crate) unsafe fn kind_of(p: *mut Header) -> i64 {
    *(field(p, 0) as *const i64)
}

pub(crate) fn kind_name(kind: i64) -> &'static str {
    match kind {
        NULL => "null",
        BOOL => "a bool",
        INT => "an int",
        FLOAT => "a float",
        STR => "a string",
        LIST => "a list",
        MAP => "a map",
        _ => "unknown",
    }
}

unsafe fn node(kind: i64) -> *mut Header {
    let p = kode_obj_new(NFIELDS, REFMASK);
    *(field(p, 0) as *mut i64) = kind;
    p
}

pub(crate) unsafe fn make_null() -> *mut Header {
    node(NULL)
}

pub(crate) unsafe fn make_bool(v: bool) -> *mut Header {
    let p = node(BOOL);
    *(field(p, 1) as *mut i64) = i64::from(v);
    p
}

pub(crate) unsafe fn make_int(v: i64) -> *mut Header {
    let p = node(INT);
    *(field(p, 1) as *mut i64) = v;
    p
}

pub(crate) unsafe fn make_float(v: f64) -> *mut Header {
    let p = node(FLOAT);
    *(field(p, 2) as *mut f64) = v;
    p
}

/// Takes ownership of `s`.
pub(crate) unsafe fn make_string(s: *mut Header) -> *mut Header {
    let p = node(STR);
    *(field(p, 3) as *mut *mut Header) = s;
    p
}

/// Takes ownership of `list`.
pub(crate) unsafe fn wrap_list(list: *mut Header) -> *mut Header {
    let p = node(LIST);
    *(field(p, 4) as *mut *mut Header) = list;
    p
}

/// Takes ownership of `map`.
pub(crate) unsafe fn wrap_map(map: *mut Header) -> *mut Header {
    let p = node(MAP);
    *(field(p, 5) as *mut *mut Header) = map;
    p
}

fn fail(msg: String) -> *mut Header {
    throw_msg(&msg);
    std::ptr::null_mut()
}

unsafe fn expect(j: *mut Header, kind: i64, what: &str) -> Result<(), String> {
    if j.is_null() {
        return Err("this Json value is none".into());
    }
    let actual = kind_of(j);
    if actual != kind {
        return Err(format!("this Json value is {}, not {}", kind_name(actual), what));
    }
    Ok(())
}

unsafe fn container(j: *mut Header, field_index: usize, what: &str) -> Result<*mut Header, String> {
    let p = *(field(j, field_index) as *const *mut Header);
    if p.is_null() {
        return Err(format!("this Json value has no {what}"));
    }
    Ok(p)
}

// ---- constructors ----

#[no_mangle]
pub extern "C" fn kode_json_null() -> *mut Header {
    unsafe { make_null() }
}

#[no_mangle]
pub extern "C" fn kode_json_bool(v: i64) -> *mut Header {
    unsafe { make_bool(v != 0) }
}

#[no_mangle]
pub extern "C" fn kode_json_int(v: i64) -> *mut Header {
    unsafe { make_int(v) }
}

#[no_mangle]
pub extern "C" fn kode_json_float(v: f64) -> *mut Header {
    unsafe { make_float(v) }
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_json_string(s: *const Header) -> *mut Header {
    make_string(from_bytes(bytes(s)))
}

/// # Safety
/// `list` must be a list of `Json` values. It is borrowed; the result keeps a reference.
#[no_mangle]
pub unsafe extern "C" fn kode_json_list(list: *mut Header) -> *mut Header {
    kode_retain(list);
    wrap_list(list)
}

/// # Safety
/// `map` must be a map of `Json` values. It is borrowed; the result keeps a reference.
#[no_mangle]
pub unsafe extern "C" fn kode_json_map(map: *mut Header) -> *mut Header {
    kode_retain(map);
    wrap_map(map)
}

// ---- parsing and encoding ----

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_json_parse(s: *const Header) -> *mut Header {
    let text = std::str::from_utf8_unchecked(bytes(s));
    match crate::json_parse::parse(text) {
        Ok(v) => v,
        Err(msg) => fail(msg),
    }
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_encode(j: *mut Header) -> *mut Header {
    let mut out = Vec::new();
    match encode(j, &mut out, 0) {
        Ok(()) => from_bytes(&out),
        Err(msg) => fail(msg),
    }
}

unsafe fn encode(j: *mut Header, out: &mut Vec<u8>, depth: usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("this Json value is nested too deeply to encode".into());
    }
    if j.is_null() {
        return Err("this Json value is none".into());
    }
    match kind_of(j) {
        NULL => out.extend_from_slice(b"null"),
        BOOL => {
            let v = *(field(j, 1) as *const i64) != 0;
            out.extend_from_slice(if v { b"true" } else { b"false" });
        }
        INT => {
            let v = *(field(j, 1) as *const i64);
            out.extend_from_slice(v.to_string().as_bytes());
        }
        FLOAT => {
            let v = *(field(j, 2) as *const f64);
            if !v.is_finite() {
                return Err("JSON cannot hold NaN or infinity".into());
            }
            out.extend_from_slice(format!("{v:?}").as_bytes());
        }
        STR => {
            let s = container(j, 3, "string")?;
            encode_string(std::str::from_utf8_unchecked(bytes(s)), out);
        }
        LIST => {
            let list = container(j, 4, "list")?;
            let n = kode_list_len(list);
            out.push(b'[');
            for i in 0..n {
                if i > 0 {
                    out.push(b',');
                }
                let item = kode_list_get(list, i, 0) as *mut Header;
                encode(item, out, depth + 1)?;
            }
            out.push(b']');
        }
        MAP => {
            let map = container(j, 5, "map")?;
            out.push(b'{');
            for (i, (key, value)) in entries(map).into_iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                encode_string(std::str::from_utf8_unchecked(bytes(key)), out);
                out.push(b':');
                encode(value as *mut Header, out, depth + 1)?;
            }
            out.push(b'}');
        }
        other => return Err(format!("cannot encode a Json value of kind {other}")),
    }
    Ok(())
}

fn encode_string(text: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for c in text.chars() {
        match c {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            c if (c as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    out.push(b'"');
}

// ---- accessors ----

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_is_null(j: *mut Header) -> i64 {
    (j.is_null() || kind_of(j) == NULL) as i64
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_bool(j: *mut Header) -> i64 {
    if let Err(msg) = expect(j, BOOL, "a bool") {
        throw_msg(&msg);
        return 0;
    }
    *(field(j, 1) as *const i64)
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_int(j: *mut Header) -> i64 {
    if let Err(msg) = expect(j, INT, "an int") {
        throw_msg(&msg);
        return 0;
    }
    *(field(j, 1) as *const i64)
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_float(j: *mut Header) -> f64 {
    if j.is_null() {
        throw_msg("this Json value is none");
        return 0.0;
    }
    match kind_of(j) {
        FLOAT => *(field(j, 2) as *const f64),
        INT => *(field(j, 1) as *const i64) as f64,
        other => {
            throw_msg(&format!("this Json value is {}, not a float", kind_name(other)));
            0.0
        }
    }
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_string(j: *mut Header) -> *mut Header {
    if let Err(msg) = expect(j, STR, "a string") {
        throw_msg(&msg);
        return std::ptr::null_mut();
    }
    match container(j, 3, "string") {
        Ok(s) => {
            kode_retain(s);
            s
        }
        Err(msg) => {
            throw_msg(&msg);
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_list(j: *mut Header) -> *mut Header {
    if let Err(msg) = expect(j, LIST, "a list") {
        throw_msg(&msg);
        return std::ptr::null_mut();
    }
    match container(j, 4, "list") {
        Ok(list) => {
            kode_retain(list);
            list
        }
        Err(msg) => {
            throw_msg(&msg);
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_as_map(j: *mut Header) -> *mut Header {
    if let Err(msg) = expect(j, MAP, "a map") {
        throw_msg(&msg);
        return std::ptr::null_mut();
    }
    match container(j, 5, "map") {
        Ok(map) => {
            kode_retain(map);
            map
        }
        Err(msg) => {
            throw_msg(&msg);
            std::ptr::null_mut()
        }
    }
}

/// Number of characters in a JSON string, or items in a list or map.
///
/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_len(j: *mut Header) -> i64 {
    if j.is_null() {
        throw_msg("this Json value is none");
        return 0;
    }
    match kind_of(j) {
        STR => {
            let s = *(field(j, 3) as *const *mut Header);
            crate::string::kode_str_len(s)
        }
        LIST => kode_list_len(*(field(j, 4) as *const *mut Header)),
        MAP => kode_map_len(*(field(j, 5) as *const *mut Header)),
        other => {
            throw_msg(&format!("{} has no length", kind_name(other)));
            0
        }
    }
}

/// # Safety
/// `j` must be a valid JSON value and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_json_has(j: *mut Header, key: *mut Header) -> i64 {
    if let Err(msg) = expect(j, MAP, "a map") {
        throw_msg(&msg);
        return 0;
    }
    match container(j, 5, "map") {
        Ok(map) => kode_map_has(map, key),
        Err(msg) => {
            throw_msg(&msg);
            0
        }
    }
}

/// # Safety
/// `j` must be a valid JSON value and `key` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_json_index_key(j: *mut Header, key: *mut Header, line: i64) -> *mut Header {
    if let Err(msg) = expect(j, MAP, "a map") {
        throw_msg(&format!("{msg} (line {line})"));
        return std::ptr::null_mut();
    }
    let map = match container(j, 5, "map") {
        Ok(m) => m,
        Err(msg) => {
            throw_msg(&msg);
            return std::ptr::null_mut();
        }
    };
    if kode_map_has(map, key) == 0 {
        throw_msg(&format!(
            "key \"{}\" is not in this Json object at line {line}",
            String::from_utf8_lossy(bytes(key))
        ));
        return std::ptr::null_mut();
    }
    kode_map_get(map, key, line) as *mut Header
}

/// # Safety
/// `j` must be a valid JSON value.
#[no_mangle]
pub unsafe extern "C" fn kode_json_index_at(j: *mut Header, index: i64, line: i64) -> *mut Header {
    if let Err(msg) = expect(j, LIST, "a list") {
        throw_msg(&format!("{msg} (line {line})"));
        return std::ptr::null_mut();
    }
    let list = match container(j, 4, "list") {
        Ok(l) => l,
        Err(msg) => {
            throw_msg(&msg);
            return std::ptr::null_mut();
        }
    };
    let len = kode_list_len(list);
    if index < 0 || index >= len {
        throw_msg(&format!(
            "Json index {index} is out of range (length {len}) at line {line}"
        ));
        return std::ptr::null_mut();
    }
    kode_list_get(list, index, line) as *mut Header
}

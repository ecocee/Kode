//! Text for a list or a map, for `print(xs)` and `"${xs}"`.
//!
//! The compiler knows the element type, the runtime does not, so the compiler
//! passes a descriptor string and `kode_repr` walks both together:
//!
//! | descriptor | value |
//! |---|---|
//! | `i` | int |
//! | `f` | float |
//! | `b` | bool |
//! | `s` | string |
//! | `[...]` | list of the inner descriptor |
//! | `{...}` | map with value type of the inner descriptor (keys are strings) |
//!
//! Output is Python-like: `[1, 2, 3]`, `["a", "b"]`, `{"name": "Asha"}`.
//! Strings inside a collection are quoted and escaped, so a list is
//! unambiguous; a bare `print("text")` stays unquoted.

use crate::list::{items_for_repr, Items};
use crate::map;
use crate::object::Header;
use crate::string::{bytes, format_float, from_bytes};

enum Desc {
    Int,
    Float,
    Bool,
    Str,
    List(Box<Desc>),
    Map(Box<Desc>),
}

fn parse(d: &[u8], i: &mut usize) -> Option<Desc> {
    let c = *d.get(*i)?;
    *i += 1;
    match c {
        b'i' => Some(Desc::Int),
        b'f' => Some(Desc::Float),
        b'b' => Some(Desc::Bool),
        b's' => Some(Desc::Str),
        b'[' => {
            let inner = parse(d, i)?;
            if d.get(*i) != Some(&b']') {
                return None;
            }
            *i += 1;
            Some(Desc::List(Box::new(inner)))
        }
        b'{' => {
            let inner = parse(d, i)?;
            if d.get(*i) != Some(&b'}') {
                return None;
            }
            *i += 1;
            Some(Desc::Map(Box::new(inner)))
        }
        _ => None,
    }
}

/// A string value inside a collection: quoted, with the escapes that would
/// otherwise make the output ambiguous.
unsafe fn quoted(out: &mut String, p: *const Header) {
    out.push('"');
    for ch in String::from_utf8_lossy(bytes(p)).chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
}

unsafe fn write(out: &mut String, raw: u64, d: &Desc) {
    match d {
        Desc::Int => out.push_str(&(raw as i64).to_string()),
        Desc::Float => out.push_str(&format_float(f64::from_bits(raw))),
        Desc::Bool => out.push_str(if raw != 0 { "true" } else { "false" }),
        Desc::Str => quoted(out, raw as *const Header),
        Desc::List(inner) => {
            out.push('[');
            match items_for_repr(raw as *const Header) {
                Items::Wide(items) => {
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write(out, *item, inner);
                    }
                }
                Items::Packed(items) => {
                    for (i, item) in items.iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write(out, *item as u64, inner);
                    }
                }
            }
            out.push(']');
        }
        Desc::Map(inner) => {
            out.push('{');
            for (i, (key, val)) in map::entries(raw as *const Header).into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                quoted(out, key);
                out.push_str(": ");
                write(out, val, inner);
            }
            out.push('}');
        }
    }
}

/// The text of a collection. `desc` is the descriptor for `v`.
///
/// # Safety
/// `v` must be a valid list or map matching `desc`, and `desc` a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_repr(v: *const Header, desc: *const Header) -> *mut Header {
    let raw = bytes(desc);
    let mut at = 0;
    let mut out = String::new();
    match parse(raw, &mut at) {
        Some(d) => write(&mut out, v as u64, &d),
        // The compiler builds the descriptor, so this is unreachable in
        // practice; print something usable instead of crashing.
        None => out.push_str("<unprintable>"),
    }
    from_bytes(out.as_bytes())
}

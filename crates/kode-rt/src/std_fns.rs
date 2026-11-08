//! Standard library functions called from compiled code (see `kode-sema/src/std_fns.rs`).
//! Functions that can fail raise a Kode `Error` and return a default value.

use std::time::Duration;

use crate::clock_time;
use crate::error::{fatal, throw_msg};
use crate::list::{kode_list_new, kode_list_push};
use crate::object::Header;
use crate::string::{bytes, from_bytes};

unsafe fn text<'a>(p: *const Header) -> &'a str {
    std::str::from_utf8_unchecked(bytes(p))
}

fn string_list(items: impl Iterator<Item = String>) -> *mut Header {
    unsafe {
        let list = kode_list_new(1);
        for s in items {
            kode_list_push(list, from_bytes(s.as_bytes()) as i64);
        }
        list
    }
}

// ---- files, environment, time ----

/// # Safety
/// `path` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_read_file(path: *const Header) -> *mut Header {
    match std::fs::read(text(path)) {
        Ok(b) => match String::from_utf8(b) {
            Ok(s) => from_bytes(s.as_bytes()),
            Err(_) => {
                throw_msg(&format!("{} is not valid UTF-8 text", text(path)));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            throw_msg(&format!("cannot read {}: {e}", text(path)));
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_write_file(path: *const Header, content: *const Header) {
    if let Err(e) = std::fs::write(text(path), bytes(content)) {
        throw_msg(&format!("cannot write {}: {e}", text(path)));
    }
}

#[no_mangle]
pub extern "C" fn kode_args() -> *mut Header {
    string_list(std::env::args().skip(1))
}

/// # Safety
/// `name` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_env(name: *const Header) -> *mut Header {
    match std::env::var(text(name)) {
        Ok(v) => from_bytes(v.as_bytes()),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn kode_sleep(ms: i64) {
    std::thread::sleep(Duration::from_millis(ms.max(0) as u64));
}

/// Nanoseconds since the first clock call in the program (monotonic).
#[no_mangle]
pub extern "C" fn kode_now_ns() -> i64 {
    clock_time::now_ns()
}

/// Milliseconds since the same starting point as `kode_now_ns`.
#[no_mangle]
pub extern "C" fn kode_now_ms() -> i64 {
    clock_time::now_ms()
}

#[no_mangle]
pub extern "C" fn kode_chr(n: i64) -> *mut Header {
    let c = u32::try_from(n)
        .ok()
        .and_then(char::from_u32)
        .unwrap_or('\u{fffd}');
    from_bytes(c.to_string().as_bytes())
}

// ---- strings ----

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_size(s: *const Header) -> i64 {
    bytes(s).len() as i64
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_byte_at(s: *const Header, i: i64) -> i64 {
    let b = bytes(s);
    if i < 0 || i as usize >= b.len() {
        fatal(&format!(
            "byte index {i} is out of range (size {})",
            b.len()
        ));
    }
    b[i as usize] as i64
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_slice(s: *const Header, a: i64, b: i64) -> *mut Header {
    let t = text(s);
    if a < 0
        || b < a
        || b as usize > t.len()
        || !t.is_char_boundary(a as usize)
        || !t.is_char_boundary(b as usize)
    {
        throw_msg(&format!(
            "slice({a}, {b}) is not valid for a string of size {}",
            t.len()
        ));
        return std::ptr::null_mut();
    }
    from_bytes(&t.as_bytes()[a as usize..b as usize])
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_contains(s: *const Header, t: *const Header) -> i64 {
    text(s).contains(text(t)) as i64
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_starts_with(s: *const Header, t: *const Header) -> i64 {
    text(s).starts_with(text(t)) as i64
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_ends_with(s: *const Header, t: *const Header) -> i64 {
    text(s).ends_with(text(t)) as i64
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_find(s: *const Header, t: *const Header) -> i64 {
    text(s).find(text(t)).map(|i| i as i64).unwrap_or(-1)
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_upper(s: *const Header) -> *mut Header {
    from_bytes(text(s).to_uppercase().as_bytes())
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_lower(s: *const Header) -> *mut Header {
    from_bytes(text(s).to_lowercase().as_bytes())
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_trim(s: *const Header) -> *mut Header {
    from_bytes(text(s).trim().as_bytes())
}

/// # Safety
/// All arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_replace(
    s: *const Header,
    a: *const Header,
    b: *const Header,
) -> *mut Header {
    let from = text(a);
    if from.is_empty() {
        return from_bytes(bytes(s));
    }
    from_bytes(text(s).replace(from, text(b)).as_bytes())
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_repeat(s: *const Header, n: i64) -> *mut Header {
    from_bytes(text(s).repeat(n.max(0) as usize).as_bytes())
}

/// # Safety
/// Both arguments must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_split(s: *const Header, sep: *const Header) -> *mut Header {
    let (t, d) = (text(s), text(sep));
    if d.is_empty() {
        return string_list(t.chars().map(|c| c.to_string()));
    }
    string_list(t.split(d).map(|x| x.to_string()))
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_to_int(s: *const Header) -> i64 {
    match text(s).trim().parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            throw_msg(&format!("not an int: \"{}\"", text(s)));
            0
        }
    }
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_to_float(s: *const Header) -> f64 {
    match text(s).trim().parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            throw_msg(&format!("not a float: \"{}\"", text(s)));
            0.0
        }
    }
}

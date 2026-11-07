//! Strings: `[rc][kind = STR][length][bytes...]`.

use crate::object::{alloc_object, free_object, Header, STR};

const HEADER: usize = 24;

unsafe fn len_of(p: *const Header) -> usize {
    *((p as *const i64).add(2)) as usize
}

unsafe fn data(p: *const Header) -> *mut u8 {
    (p as *mut u8).add(HEADER)
}

/// # Safety
/// `p` must point to a valid string.
pub unsafe fn bytes<'a>(p: *const Header) -> &'a [u8] {
    std::slice::from_raw_parts(data(p), len_of(p))
}

pub unsafe fn drop_str(p: *mut Header) {
    free_object(p, HEADER + len_of(p));
}

pub fn from_bytes(b: &[u8]) -> *mut Header {
    unsafe {
        let p = alloc_object(HEADER + b.len(), STR);
        *((p as *mut i64).add(2)) = b.len() as i64;
        std::ptr::copy_nonoverlapping(b.as_ptr(), data(p), b.len());
        p
    }
}

/// Build a Kode string from bytes that did not come from a Rust `str` (a request
/// body, a response body, process output). Invalid UTF-8 becomes U+FFFD, so every
/// Kode string stays valid UTF-8 and the `from_utf8_unchecked` calls elsewhere in
/// the runtime stay sound.
pub fn from_external(b: &[u8]) -> *mut Header {
    match std::str::from_utf8(b) {
        Ok(_) => from_bytes(b),
        Err(_) => from_bytes(String::from_utf8_lossy(b).as_bytes()),
    }
}

/// # Safety
/// Both pointers must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_concat(a: *const Header, b: *const Header) -> *mut Header {
    let (x, y) = (bytes(a), bytes(b));
    let p = alloc_object(HEADER + x.len() + y.len(), STR);
    *((p as *mut i64).add(2)) = (x.len() + y.len()) as i64;
    std::ptr::copy_nonoverlapping(x.as_ptr(), data(p), x.len());
    std::ptr::copy_nonoverlapping(y.as_ptr(), data(p).add(x.len()), y.len());
    p
}

/// # Safety
/// Both pointers must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_eq(a: *const Header, b: *const Header) -> i64 {
    (bytes(a) == bytes(b)) as i64
}

/// Length in characters (not bytes).
///
/// # Safety
/// `a` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_len(a: *const Header) -> i64 {
    std::str::from_utf8_unchecked(bytes(a)).chars().count() as i64
}

#[no_mangle]
pub extern "C" fn kode_int_to_str(v: i64) -> *mut Header {
    from_bytes(v.to_string().as_bytes())
}

#[no_mangle]
pub extern "C" fn kode_float_to_str(v: f64) -> *mut Header {
    from_bytes(format_float(v).as_bytes())
}

#[no_mangle]
pub extern "C" fn kode_bool_to_str(v: i64) -> *mut Header {
    from_bytes(if v != 0 { b"true" } else { b"false" })
}

/// 3.0 prints as `3.0`, like Python.
pub fn format_float(v: f64) -> String {
    format!("{v:?}")
}

// ---- library functions ----

/// Split on `\n`, dropping a trailing `\r` from each line.
///
/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_lines(s: *const Header) -> *mut Header {
    let text = std::str::from_utf8_unchecked(bytes(s));
    let list = crate::list::kode_list_new(1);
    for line in text.lines() {
        crate::list::kode_list_push(list, from_bytes(line.as_bytes()) as i64);
    }
    list
}

/// One character (not byte) of the string, or a catchable error.
///
/// A static empty string (`rc = -1`). Error paths return it instead of
/// allocating, so nothing can be leaked when the caller jumps to a handler.
#[repr(C)]
struct StaticStr {
    rc: i64,
    kind: i64,
    len: i64,
}

static EMPTY: StaticStr = StaticStr {
    rc: -1,
    kind: STR,
    len: 0,
};

fn empty() -> *mut Header {
    &EMPTY as *const StaticStr as *mut Header
}

/// One character (not byte) of the string, or a catchable error.
///
/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_char_at(s: *const Header, idx: i64) -> *mut Header {
    let text = std::str::from_utf8_unchecked(bytes(s));
    let n = text.chars().count() as i64;
    if idx < 0 || idx >= n {
        crate::error::throw_msg(&format!("string index {idx} is out of range (length {n})"));
        return empty();
    }
    let mut buf = [0u8; 4];
    let ch = text.chars().nth(idx as usize).unwrap_or('\u{fffd}');
    from_bytes(ch.encode_utf8(&mut buf).as_bytes())
}

/// # Safety
/// `s` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_is_empty(s: *const Header) -> i64 {
    bytes(s).is_empty() as i64
}

/// Non-overlapping occurrences of `sub`.
///
/// # Safety
/// Both pointers must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_count(s: *const Header, sub: *const Header) -> i64 {
    let (hay, needle) = (bytes(s), bytes(sub));
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0i64;
    let mut i = 0usize;
    while i + needle.len() <= hay.len() {
        if &hay[i..i + needle.len()] == needle {
            count += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    count
}

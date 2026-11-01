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
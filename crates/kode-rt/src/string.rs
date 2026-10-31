//! Reference-counted strings.
//!
//! Layout in memory: `[ refcount: i64 | length: i64 | bytes... ]`.
//! A string value is a pointer to that header. Static literals are written
//! by the compiler with `refcount = -1`, which means "never free".

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::sync::atomic::{AtomicI64, Ordering};

#[repr(C)]
pub struct KStr {
    rc: i64,
    len: i64,
}

const HEADER: usize = 16;

static LIVE: AtomicI64 = AtomicI64::new(0);

pub fn live_strings() -> i64 {
    LIVE.load(Ordering::Relaxed)
}

fn layout(len: usize) -> Layout {
    Layout::from_size_align(HEADER + len, 8).expect("string too large")
}

unsafe fn data(p: *const KStr) -> *mut u8 {
    (p as *mut u8).add(HEADER)
}

/// # Safety
/// `p` must point to a valid string.
pub(crate) unsafe fn bytes<'a>(p: *const KStr) -> &'a [u8] {
    std::slice::from_raw_parts(data(p), (*p).len as usize)
}

fn alloc_str(len: usize) -> *mut KStr {
    unsafe {
        let l = layout(len);
        let p = alloc(l) as *mut KStr;
        if p.is_null() {
            handle_alloc_error(l);
        }
        (*p).rc = 1;
        (*p).len = len as i64;
        LIVE.fetch_add(1, Ordering::Relaxed);
        p
    }
}

fn from_bytes(b: &[u8]) -> *mut KStr {
    let p = alloc_str(b.len());
    unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), data(p), b.len()) };
    p
}

/// # Safety
/// `p` must be null or a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_retain(p: *mut KStr) {
    if !p.is_null() && (*p).rc > 0 {
        (*p).rc += 1;
    }
}

/// # Safety
/// `p` must be null or a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_release(p: *mut KStr) {
    if p.is_null() || (*p).rc <= 0 {
        return;
    }
    (*p).rc -= 1;
    if (*p).rc == 0 {
        let len = (*p).len as usize;
        dealloc(p as *mut u8, layout(len));
        LIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

/// # Safety
/// Both pointers must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_concat(a: *const KStr, b: *const KStr) -> *mut KStr {
    let (x, y) = (bytes(a), bytes(b));
    let p = alloc_str(x.len() + y.len());
    std::ptr::copy_nonoverlapping(x.as_ptr(), data(p), x.len());
    std::ptr::copy_nonoverlapping(y.as_ptr(), data(p).add(x.len()), y.len());
    p
}

/// # Safety
/// Both pointers must be valid strings.
#[no_mangle]
pub unsafe extern "C" fn kode_str_eq(a: *const KStr, b: *const KStr) -> i64 {
    (bytes(a) == bytes(b)) as i64
}

/// Length in characters (not bytes).
///
/// # Safety
/// `a` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_str_len(a: *const KStr) -> i64 {
    std::str::from_utf8_unchecked(bytes(a)).chars().count() as i64
}

#[no_mangle]
pub extern "C" fn kode_int_to_str(v: i64) -> *mut KStr {
    from_bytes(v.to_string().as_bytes())
}

#[no_mangle]
pub extern "C" fn kode_float_to_str(v: f64) -> *mut KStr {
    from_bytes(format_float(v).as_bytes())
}

#[no_mangle]
pub extern "C" fn kode_bool_to_str(v: i64) -> *mut KStr {
    from_bytes(if v != 0 { b"true" } else { b"false" })
}

/// 3.0 prints as `3.0`, like Python.
pub(crate) fn format_float(v: f64) -> String {
    format!("{v:?}")
}

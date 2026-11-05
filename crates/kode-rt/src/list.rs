//! Lists: `[rc][kind = LIST | flags][length][capacity][data pointer]`.
//! Every item takes 8 bytes (ints, float bits, bools, or pointers).

use std::mem::ManuallyDrop;

use crate::error::fatal;
use crate::object::{alloc_object, free_object, kode_release, kode_retain, Header, ITEMS_ARE_REFS, LIST, PACKED1};

const SIZE: usize = 40;

struct Raw(*mut Header);

impl Raw {
    unsafe fn len(&self) -> &mut i64 {
        &mut *((self.0 as *mut i64).add(2))
    }
    unsafe fn cap(&self) -> &mut i64 {
        &mut *((self.0 as *mut i64).add(3))
    }
    unsafe fn ptr(&self) -> &mut *mut u64 {
        &mut *((self.0 as *mut *mut u64).add(4))
    }
    unsafe fn is_refs(&self) -> bool {
        (*self.0).kind & ITEMS_ARE_REFS != 0
    }
    /// Run `f` on the items as a `Vec`, then store it back.
    unsafe fn with_vec<R>(&self, f: impl FnOnce(&mut Vec<u64>) -> R) -> R {
        let mut v = ManuallyDrop::new(Vec::from_raw_parts(*self.ptr(), *self.len() as usize, *self.cap() as usize));
        let r = f(&mut v);
        *self.ptr() = v.as_mut_ptr();
        *self.len() = v.len() as i64;
        *self.cap() = v.capacity() as i64;
        r
    }
}

#[no_mangle]
pub extern "C" fn kode_list_new(items_are_refs: i64) -> *mut Header {
    unsafe {
        let kind = LIST | if items_are_refs != 0 { ITEMS_ARE_REFS } else { 0 };
        let p = alloc_object(SIZE, kind);
        let r = Raw(p);
        let mut v = ManuallyDrop::new(Vec::<u64>::new());
        *r.ptr() = v.as_mut_ptr();
        *r.len() = 0;
        *r.cap() = 0;
        p
    }
}

pub unsafe fn drop_list(p: *mut Header) {
    let r = Raw(p);
    if (*p).kind & PACKED1 != 0 {
        drop(Vec::from_raw_parts(*r.ptr() as *mut u8, *r.len() as usize, *r.cap() as usize));
        free_object(p, SIZE);
        return;
    }
    let refs = r.is_refs();
    let v = Vec::from_raw_parts(*r.ptr(), *r.len() as usize, *r.cap() as usize);
    if refs {
        for item in &v {
            kode_release(*item as *mut Header);
        }
    }
    drop(v);
    free_object(p, SIZE);
}

/// # Safety
/// `p` must be a valid list. The list takes ownership of `raw` if its items are heap values.
#[no_mangle]
pub unsafe extern "C" fn kode_list_push(p: *mut Header, raw: i64) {
    Raw(p).with_vec(|v| v.push(raw as u64));
}

/// # Safety
/// `p` must be a valid list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_len(p: *const Header) -> i64 {
    *Raw(p as *mut Header).len()
}

/// Returns the item (a borrowed value when it is a heap value).
///
/// # Safety
/// `p` must be a valid list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_get(p: *mut Header, idx: i64, line: i64) -> i64 {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx >= len {
        fatal(&format!("list index {idx} is out of range (length {len}) at line {line}"));
    }
    *(*r.ptr()).add(idx as usize) as i64
}

/// # Safety
/// `p` must be a valid list. The list takes ownership of `raw` if its items are heap values.
#[no_mangle]
pub unsafe extern "C" fn kode_list_set(p: *mut Header, idx: i64, raw: i64, line: i64) {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx >= len {
        fatal(&format!("list index {idx} is out of range (length {len}) at line {line}"));
    }
    let slot = (*r.ptr()).add(idx as usize);
    let old = *slot;
    *slot = raw as u64;
    if r.is_refs() {
        kode_release(old as *mut Header);
    }
}

/// Called by compiled code when an inlined list access is out of range.
#[no_mangle]
pub extern "C" fn kode_list_oob(idx: i64, len: i64, line: i64) -> ! {
    fatal(&format!("list index {idx} is out of range (length {len}) at line {line}"));
}

// ---- lists of bool: one byte per item ----

#[no_mangle]
pub extern "C" fn kode_list_new_packed() -> *mut Header {
    unsafe {
        let p = alloc_object(SIZE, LIST | PACKED1);
        let r = Raw(p);
        let mut v = ManuallyDrop::new(Vec::<u8>::new());
        *r.ptr() = v.as_mut_ptr() as *mut u64;
        *r.len() = 0;
        *r.cap() = 0;
        p
    }
}

/// # Safety
/// `p` must be a valid packed list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_push8(p: *mut Header, raw: i64) {
    let r = Raw(p);
    let mut v = ManuallyDrop::new(Vec::from_raw_parts(*r.ptr() as *mut u8, *r.len() as usize, *r.cap() as usize));
    v.push(raw as u8);
    *r.ptr() = v.as_mut_ptr() as *mut u64;
    *r.len() = v.len() as i64;
    *r.cap() = v.capacity() as i64;
}

/// # Safety
/// `p` must be a valid packed list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_get8(p: *mut Header, idx: i64, line: i64) -> i64 {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx >= len {
        fatal(&format!("list index {idx} is out of range (length {len}) at line {line}"));
    }
    *(*r.ptr() as *const u8).add(idx as usize) as i64
}

/// # Safety
/// `p` must be a valid packed list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_set8(p: *mut Header, idx: i64, raw: i64, line: i64) {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx >= len {
        fatal(&format!("list index {idx} is out of range (length {len}) at line {line}"));
    }
    *(*r.ptr() as *mut u8).add(idx as usize) = raw as u8;
}

// ---- slicing: a copy of a range ----

/// A new list holding `start..end`. Bounds are clamped to the list, so a bound
/// past the end stops at the end and `start >= end` gives an empty list.
///
/// # Safety
/// `p` must be a valid list. Items are retained, so the copy owns them.
#[no_mangle]
pub unsafe extern "C" fn kode_list_slice(p: *mut Header, start: i64, end: i64) -> *mut Header {
    let r = Raw(p);
    let len = *r.len();
    let (s, e) = (start.clamp(0, len), end.clamp(0, len));
    let out = kode_list_new(r.is_refs() as i64);
    if s < e {
        let refs = r.is_refs();
        for item in &items(p)[s as usize..e as usize] {
            if refs {
                kode_retain(*item as *mut Header);
            }
            kode_list_push(out, *item as i64);
        }
    }
    out
}

/// A new packed list (bools, one byte each) holding `start..end`.
///
/// # Safety
/// `p` must be a valid packed list.
#[no_mangle]
pub unsafe extern "C" fn kode_list_slice8(p: *mut Header, start: i64, end: i64) -> *mut Header {
    let r = Raw(p);
    let len = *r.len();
    let (s, e) = (start.clamp(0, len), end.clamp(0, len));
    let out = kode_list_new_packed();
    if s < e {
        let bytes = std::slice::from_raw_parts(*r.ptr() as *const u8, len as usize);
        for byte in &bytes[s as usize..e as usize] {
            kode_list_push8(out, *byte as i64);
        }
    }
    out
}

// ---- library functions for lists ----

/// # Safety
/// `p` must be a list of strings and `sep` a string.
#[no_mangle]
pub unsafe extern "C" fn kode_list_join(p: *mut Header, sep: *const Header) -> *mut Header {
    let r = Raw(p);
    let items = std::slice::from_raw_parts(*r.ptr(), *r.len() as usize);
    let sep = crate::string::bytes(sep);
    let mut out: Vec<u8> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        if i > 0 {
            out.extend_from_slice(sep);
        }
        out.extend_from_slice(crate::string::bytes(*it as *const Header));
    }
    crate::string::from_bytes(&out)
}

unsafe fn items_mut<'a>(p: *mut Header) -> &'a mut [u64] {
    let r = Raw(p);
    std::slice::from_raw_parts_mut(*r.ptr(), *r.len() as usize)
}

/// # Safety
/// `p` must be a list of ints.
#[no_mangle]
pub unsafe extern "C" fn kode_list_sort_int(p: *mut Header) {
    items_mut(p).sort_by_key(|v| *v as i64);
}

/// # Safety
/// `p` must be a list of floats.
#[no_mangle]
pub unsafe extern "C" fn kode_list_sort_float(p: *mut Header) {
    items_mut(p).sort_by(|a, b| f64::from_bits(*a).partial_cmp(&f64::from_bits(*b)).unwrap_or(std::cmp::Ordering::Equal));
}

/// # Safety
/// `p` must be a list of strings.
#[no_mangle]
pub unsafe extern "C" fn kode_list_sort_str(p: *mut Header) {
    items_mut(p).sort_by(|a, b| crate::string::bytes(*a as *const Header).cmp(crate::string::bytes(*b as *const Header)));
}

/// Remove the last item and give it to the caller.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_pop(p: *mut Header) -> i64 {
    let r = Raw(p);
    if *r.len() == 0 {
        fatal("pop from an empty list");
    }
    r.with_vec(|v| v.pop().unwrap_or(0)) as i64
}

/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_clear(p: *mut Header) {
    let r = Raw(p);
    let refs = r.is_refs();
    let old: Vec<u64> = r.with_vec(|v| std::mem::take(v));
    if refs {
        for it in old {
            kode_release(it as *mut Header);
        }
    }
}

// ---- searching, reordering and editing ----

unsafe fn items<'a>(p: *mut Header) -> &'a [u64] {
    let r = Raw(p);
    std::slice::from_raw_parts(*r.ptr(), *r.len() as usize)
}

/// # Safety
/// `p` must be a list of ints.
#[no_mangle]
pub unsafe extern "C" fn kode_list_contains_int(p: *mut Header, v: i64) -> i64 {
    items(p).iter().any(|x| *x as i64 == v) as i64
}

/// # Safety
/// `p` must be a list of floats.
#[no_mangle]
pub unsafe extern "C" fn kode_list_contains_float(p: *mut Header, v: f64) -> i64 {
    items(p).iter().any(|x| f64::from_bits(*x) == v) as i64
}

/// # Safety
/// `p` must be a list of strings and `v` a string.
#[no_mangle]
pub unsafe extern "C" fn kode_list_contains_str(p: *mut Header, v: *const Header) -> i64 {
    let needle = crate::string::bytes(v);
    items(p).iter().any(|x| crate::string::bytes(*x as *const Header) == needle) as i64
}

/// # Safety
/// `p` must be a list of ints.
#[no_mangle]
pub unsafe extern "C" fn kode_list_index_int(p: *mut Header, v: i64) -> i64 {
    items(p).iter().position(|x| *x as i64 == v).map(|i| i as i64).unwrap_or(-1)
}

/// # Safety
/// `p` must be a list of floats.
#[no_mangle]
pub unsafe extern "C" fn kode_list_index_float(p: *mut Header, v: f64) -> i64 {
    items(p).iter().position(|x| f64::from_bits(*x) == v).map(|i| i as i64).unwrap_or(-1)
}

/// # Safety
/// `p` must be a list of strings and `v` a string.
#[no_mangle]
pub unsafe extern "C" fn kode_list_index_str(p: *mut Header, v: *const Header) -> i64 {
    let needle = crate::string::bytes(v);
    items(p)
        .iter()
        .position(|x| crate::string::bytes(*x as *const Header) == needle)
        .map(|i| i as i64)
        .unwrap_or(-1)
}

/// Reverse in place. Ownership does not change, so nothing is retained or released.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_reverse(p: *mut Header) {
    items_mut(p).reverse();
}

/// Insert before `idx`. The list retains the item when it holds heap values.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_insert(p: *mut Header, idx: i64, raw: i64, line: i64) {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx > len {
        crate::error::throw_msg(&format!("insert index {idx} is out of range (length {len}) at line {line}"));
        return;
    }
    if r.is_refs() {
        kode_retain(raw as *mut Header);
    }
    r.with_vec(|v| v.insert(idx as usize, raw as u64));
}

/// Remove one item and release it when the list holds heap values.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_remove_at(p: *mut Header, idx: i64, line: i64) {
    let r = Raw(p);
    let len = *r.len();
    if idx < 0 || idx >= len {
        crate::error::throw_msg(&format!("remove index {idx} is out of range (length {len}) at line {line}"));
        return;
    }
    let refs = r.is_refs();
    let removed = r.with_vec(|v| v.remove(idx as usize));
    if refs {
        kode_release(removed as *mut Header);
    }
}

/// The first item, or a catchable error. The caller gets a new owner when the
/// list holds heap values.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_first(p: *mut Header) -> i64 {
    let r = Raw(p);
    if *r.len() == 0 {
        crate::error::throw_msg("`first()` on an empty list");
        return 0;
    }
    let raw = *(*r.ptr());
    if r.is_refs() {
        kode_retain(raw as *mut Header);
    }
    raw as i64
}

/// The last item, or a catchable error.
///
/// # Safety
/// `p` must be a list that is not packed.
#[no_mangle]
pub unsafe extern "C" fn kode_list_last(p: *mut Header) -> i64 {
    let r = Raw(p);
    let len = *r.len();
    if len == 0 {
        crate::error::throw_msg("`last()` on an empty list");
        return 0;
    }
    let raw = *(*r.ptr()).add(len as usize - 1);
    if r.is_refs() {
        kode_retain(raw as *mut Header);
    }
    raw as i64
}
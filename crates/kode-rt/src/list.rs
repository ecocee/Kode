//! Lists: `[rc][kind = LIST | flags][length][capacity][data pointer]`.
//! Every item takes 8 bytes (ints, float bits, bools, or pointers).

use std::mem::ManuallyDrop;

use crate::error::fatal;
use crate::object::{alloc_object, free_object, kode_release, Header, ITEMS_ARE_REFS, LIST};

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
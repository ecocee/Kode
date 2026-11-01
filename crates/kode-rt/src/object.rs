//! Every heap value starts with the same header: `[refcount: i64][kind: i64]`.
//! A refcount of -1 marks a static value (a string literal) that is never freed.

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::sync::atomic::{AtomicI64, Ordering};

pub const STR: i64 = 1;
pub const LIST: i64 = 2;
pub const MAP: i64 = 3;
pub const OBJ: i64 = 4;
/// Set in `kind` for lists and maps whose items are heap values.
pub const ITEMS_ARE_REFS: i64 = 1 << 8;

#[repr(C)]
pub struct Header {
    pub rc: i64,
    pub kind: i64,
}

static LIVE: AtomicI64 = AtomicI64::new(0);

pub fn live_objects() -> i64 {
    LIVE.load(Ordering::Relaxed)
}

pub fn layout(size: usize) -> Layout {
    Layout::from_size_align(size, 8).expect("object too large")
}

/// Allocate zeroed memory with a header (refcount 1).
pub unsafe fn alloc_object(size: usize, kind: i64) -> *mut Header {
    let l = layout(size);
    let p = alloc_zeroed(l) as *mut Header;
    if p.is_null() {
        handle_alloc_error(l);
    }
    (*p).rc = 1;
    (*p).kind = kind;
    LIVE.fetch_add(1, Ordering::Relaxed);
    p
}

pub unsafe fn free_object(p: *mut Header, size: usize) {
    dealloc(p as *mut u8, layout(size));
    LIVE.fetch_sub(1, Ordering::Relaxed);
}

/// # Safety
/// `p` must be null or a valid object.
#[no_mangle]
pub unsafe extern "C" fn kode_retain(p: *mut Header) {
    if !p.is_null() && (*p).rc > 0 {
        (*p).rc += 1;
    }
}

/// # Safety
/// `p` must be null or a valid object.
#[no_mangle]
pub unsafe extern "C" fn kode_release(p: *mut Header) {
    if p.is_null() || (*p).rc <= 0 {
        return;
    }
    (*p).rc -= 1;
    if (*p).rc == 0 {
        match (*p).kind & 0xff {
            STR => crate::string::drop_str(p),
            LIST => crate::list::drop_list(p),
            MAP => crate::map::drop_map(p),
            OBJ => drop_obj(p),
            _ => {}
        }
    }
}

// ---- class instances: [rc][kind | nfields << 16][refmask][fields...] ----

/// # Safety
/// Always safe to call; the result is owned by the caller.
#[no_mangle]
pub unsafe extern "C" fn kode_obj_new(nfields: i64, refmask: i64) -> *mut Header {
    let size = 24 + 8 * nfields as usize;
    let p = alloc_object(size, OBJ | (nfields << 16));
    *((p as *mut i64).add(2)) = refmask;
    p
}

unsafe fn drop_obj(p: *mut Header) {
    let nfields = ((*p).kind >> 16) as usize;
    let mask = *((p as *const i64).add(2));
    let fields = (p as *mut *mut Header).add(3);
    for i in 0..nfields.min(63) {
        if mask & (1 << i) != 0 {
            kode_release(*fields.add(i));
        }
    }
    free_object(p, 24 + 8 * nfields);
}

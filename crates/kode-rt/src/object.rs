//! Every heap value starts with the same header: `[refcount: i64][kind: i64]`.
//! A refcount of -1 marks a static value (a string literal) that is never freed.

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::sync::atomic::{fence, AtomicBool, AtomicI64, Ordering};

pub const STR: i64 = 1;
pub const LIST: i64 = 2;
pub const MAP: i64 = 3;
pub const OBJ: i64 = 4;
pub const TASK: i64 = 5;
/// Set in `kind` for lists and maps whose items are heap values.
pub const ITEMS_ARE_REFS: i64 = 1 << 8;
/// Set in `kind` for lists that store one byte per item (lists of bool).
pub const PACKED1: i64 = 1 << 9;

#[repr(C)]
pub struct Header {
    pub rc: i64,
    pub kind: i64,
}

static LIVE: AtomicI64 = AtomicI64::new(0);
/// Count live objects (only for `KODE_LEAK_CHECK`; counting costs an atomic operation).
pub static COUNT_LIVE: AtomicBool = AtomicBool::new(false);
/// Set when the first task or server thread starts: from then on reference counts are atomic.
pub static THREADED: AtomicBool = AtomicBool::new(false);

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
    if COUNT_LIVE.load(Ordering::Relaxed) {
        LIVE.fetch_add(1, Ordering::Relaxed);
    }
    p
}

pub unsafe fn free_object(p: *mut Header, size: usize) {
    dealloc(p as *mut u8, layout(size));
    if COUNT_LIVE.load(Ordering::Relaxed) {
        LIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Called first thing in `main`.
#[no_mangle]
pub extern "C" fn kode_rt_init() {
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        COUNT_LIVE.store(true, Ordering::Relaxed);
    }
}

/// # Safety
/// `p` must be null or a valid object.
#[no_mangle]
pub unsafe extern "C" fn kode_retain(p: *mut Header) {
    if p.is_null() || (*p).rc <= 0 {
        return;
    }
    if THREADED.load(Ordering::Relaxed) {
        AtomicI64::from_ptr(&mut (*p).rc).fetch_add(1, Ordering::Relaxed);
    } else {
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
    let last = if THREADED.load(Ordering::Relaxed) {
        let last = AtomicI64::from_ptr(&mut (*p).rc).fetch_sub(1, Ordering::Release) == 1;
        if last {
            fence(Ordering::Acquire);
        }
        last
    } else {
        (*p).rc -= 1;
        (*p).rc == 0
    };
    if last {
        match (*p).kind & 0xff {
            STR => crate::string::drop_str(p),
            LIST => crate::list::drop_list(p),
            MAP => crate::map::drop_map(p),
            OBJ => drop_obj(p),
            TASK => crate::task::drop_task(p),
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
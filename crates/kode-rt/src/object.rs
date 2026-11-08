//! Every heap value starts with the same header: `[refcount: i64][kind: i64]`.
//! A refcount of -1 marks a static value (a string literal) that is never freed.

use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::ptr::addr_of_mut;
use std::sync::atomic::{fence, AtomicBool, AtomicI64, Ordering};

pub const STR: i64 = 1;
pub const LIST: i64 = 2;
pub const MAP: i64 = 3;
pub const OBJ: i64 = 4;
pub const TASK: i64 = 5;
pub const CHAN: i64 = 6;

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

/// Count live objects (only for `KODE_LEAK_CHECK`).
pub static COUNT_LIVE: AtomicBool = AtomicBool::new(false);

/// Per-thread live count for the tests in this crate. `LIVE` is process-wide, and
/// `cargo test` runs tests in parallel, so a delta measured on it can include
/// another test's allocations. The thread-local count is exact for tests that
/// allocate and free on one thread, without weakening the assertions.
#[cfg(test)]
thread_local! {
    static LOCAL_COUNTING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static LOCAL_LIVE: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn local_count(delta: i64) {
    if LOCAL_COUNTING.with(|on| on.get()) {
        LOCAL_LIVE.with(|live| live.set(live.get() + delta));
    }
}

/// Set when the first task or server thread starts.
/// From then on reference counts are atomic.
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

    #[cfg(test)]
    local_count(1);

    p
}

/// Free an allocated object.
pub unsafe fn free_object(p: *mut Header, size: usize) {
    dealloc(p as *mut u8, layout(size));

    if COUNT_LIVE.load(Ordering::Relaxed) {
        LIVE.fetch_sub(1, Ordering::Relaxed);
    }

    #[cfg(test)]
    local_count(-1);
}

/// Called first thing in `main`.
#[no_mangle]
pub extern "C" fn kode_rt_init() {
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        COUNT_LIVE.store(true, Ordering::Relaxed);
    }
}

/// View the refcount as an atomic.
///
/// Every access in threaded mode goes through this,
/// so a plain read can never race with an atomic update.
#[inline(always)]
unsafe fn rc_atomic<'a>(p: *mut Header) -> &'a AtomicI64 {
    AtomicI64::from_ptr(addr_of_mut!((*p).rc))
}

/// # Safety
/// `p` must be null or a valid object.
#[no_mangle]
pub unsafe extern "C" fn kode_retain(p: *mut Header) {
    if p.is_null() {
        return;
    }

    if THREADED.load(Ordering::Relaxed) {
        let rc = rc_atomic(p);
        let before = rc.load(Ordering::Relaxed);
        if before <= 0 {
            return;
        }
        rc.fetch_add(1, Ordering::Relaxed);
    } else {
        if (*p).rc <= 0 {
            return;
        }

        (*p).rc += 1;
    }
}

/// # Safety
/// `p` must be null or a valid object.
#[no_mangle]
pub unsafe extern "C" fn kode_release(p: *mut Header) {
    if p.is_null() {
        return;
    }

    let last = if THREADED.load(Ordering::Relaxed) {
        let rc = rc_atomic(p);
        let before = rc.load(Ordering::Relaxed);
        if before <= 0 {
            return;
        }

        let prev = rc.fetch_sub(1, Ordering::Release);

        debug_assert!(prev >= 1, "kode_release: double release");

        if prev == 1 {
            fence(Ordering::Acquire);
            true
        } else {
            false
        }
    } else {
        if (*p).rc <= 0 {
            return;
        }

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
            CHAN => crate::channel::drop_chan(p),
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

/// A class instance that remembers its class ID (bits 8..16 of `kind`),
/// so an interface call can dispatch to the right method.
///
/// # Safety
/// Always safe to call; the result is owned by the caller.
#[no_mangle]
pub unsafe extern "C" fn kode_obj_new_class(
    nfields: i64,
    refmask: i64,
    class_id: i64,
) -> *mut Header {
    let size = 24 + 8 * nfields as usize;

    let p = alloc_object(size, OBJ | ((class_id & 0xff) << 8) | (nfields << 16));

    *((p as *mut i64).add(2)) = refmask;

    p
}

/// The class ID stored in a class instance.
/// Returns 0 for objects made by the runtime.
#[no_mangle]
pub extern "C" fn kode_obj_class(p: *mut Header) -> i64 {
    if p.is_null() {
        return -1;
    }

    unsafe { ((*p).kind >> 8) & 0xff }
}

/// Release all referenced fields and free a class instance.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Tests that count live objects take this lock,
    /// so their deltas are exact.
    static LOCK: Mutex<()> = Mutex::new(());

    fn counting() -> std::sync::MutexGuard<'static, ()> {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());

        COUNT_LIVE.store(true, Ordering::Relaxed);
        LOCAL_COUNTING.with(|on| on.set(true));

        guard
    }

    /// Objects this thread allocated and has not freed yet.
    fn local_live() -> i64 {
        LOCAL_LIVE.with(|live| live.get())
    }

    fn rc_of(p: *mut Header) -> i64 {
        unsafe { AtomicI64::from_ptr(std::ptr::addr_of_mut!((*p).rc)).load(Ordering::SeqCst) }
    }

    #[test]
    fn static_objects_are_never_changed() {
        let mut h = Header { rc: -1, kind: STR };

        let p = &mut h as *mut Header;

        unsafe {
            kode_retain(p);
            kode_release(p);
        }

        assert_eq!(rc_of(p), -1);

        THREADED.store(true, Ordering::SeqCst);

        unsafe {
            kode_retain(p);
            kode_release(p);
        }

        assert_eq!(rc_of(p), -1);
    }

    #[test]
    fn concurrent_retain_release_stays_balanced() {
        THREADED.store(true, Ordering::SeqCst);

        let s = crate::string::from_bytes(b"shared");
        let addr = s as usize;

        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(move || {
                    for _ in 0..50_000 {
                        unsafe {
                            kode_retain(addr as *mut Header);
                            kode_release(addr as *mut Header);
                        }
                    }
                })
            })
            .collect();

        for t in threads {
            t.join().expect("worker");
        }

        assert_eq!(rc_of(s), 1);

        unsafe {
            kode_release(s);
        }
    }

    #[test]
    fn last_release_from_many_threads_frees_exactly_once() {
        let _g = counting();

        THREADED.store(true, Ordering::SeqCst);

        let before = local_live();

        let s = crate::string::from_bytes(b"many owners");

        unsafe {
            kode_retain(s);
            kode_retain(s);
            kode_retain(s);
        }

        assert_eq!(rc_of(s), 4);

        unsafe {
            kode_release(s);
            kode_release(s);
            kode_release(s);
            kode_release(s);
        }

        assert_eq!(local_live(), before, "the string was freed exactly once");
    }

    #[test]
    fn nested_objects_free_everything() {
        let _g = counting();

        let before = local_live();

        unsafe {
            let list = crate::list::kode_list_new(1);

            for word in [&b"a"[..], b"bb", b"ccc"] {
                crate::list::kode_list_push(list, crate::string::from_bytes(word) as i64);
            }

            let obj = kode_obj_new(2, 0b11);

            *((obj as *mut *mut Header).add(3)) = crate::string::from_bytes(b"name");

            *((obj as *mut *mut Header).add(4)) = list;

            kode_release(obj);
        }

        assert_eq!(local_live(), before);
    }

    #[test]
    fn maps_containing_references_free_everything() {
        let _g = counting();

        let before = local_live();

        unsafe {
            let map = crate::map::kode_map_new(1);

            for (k, v) in [("one", "1"), ("two", "2")] {
                let key = crate::string::from_bytes(k.as_bytes());
                let val = crate::string::from_bytes(v.as_bytes());

                crate::map::kode_map_set(map, key, val as i64);

                kode_release(key);
            }

            // Replacing a value releases the old one.
            let key = crate::string::from_bytes(b"one");

            crate::map::kode_map_set(map, key, crate::string::from_bytes(b"uno") as i64);

            kode_release(key);
            kode_release(map);
        }

        assert_eq!(local_live(), before);
    }

    #[test]
    fn replacing_a_pending_error_releases_the_old_one() {
        let _g = counting();

        let before = local_live();

        crate::error::throw_msg("first");
        crate::error::throw_msg("second");

        let e = crate::error::take_err();

        assert!(!e.is_null());

        unsafe {
            kode_release(e);
        }

        assert_eq!(local_live(), before);
    }

    // The closure is required: an extern "C" fn item
    // does not implement FnOnce.
    #[allow(clippy::redundant_closure)]
    #[test]
    fn errors_are_isolated_between_threads() {
        crate::error::throw_msg("only on this thread");

        let other = std::thread::spawn(|| crate::error::kode_err_pending())
            .join()
            .expect("thread");

        assert_eq!(other, 0);

        unsafe {
            kode_release(crate::error::take_err());
        }
    }

    #[test]
    fn external_bytes_always_become_valid_utf8() {
        unsafe {
            let s = crate::string::from_external(&[b'o', b'k', 0xff, 0xfe, b'!']);

            assert!(std::str::from_utf8(crate::string::bytes(s)).is_ok());

            kode_release(s);
        }
    }
}

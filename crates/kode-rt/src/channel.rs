//! `Channel<T>`: a queue shared between tasks.
//!
//! Layout matches the other objects: `[rc][kind = CHAN][..]`, with a pointer to
//! the queue at offset 16. The queue is guarded by a mutex and a condition
//! variable, so `recv` blocks instead of spinning.

use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Condvar, Mutex};

use crate::error::throw_msg;
use crate::object::{alloc_object, free_object, kode_release, kode_retain, Header, CHAN, THREADED};

const SIZE: usize = 24;

#[derive(Default)]
struct Inner {
    items: VecDeque<(u64, bool)>,
    closed: bool,
}

struct Chan {
    inner: Mutex<Inner>,
    ready: Condvar,
}

/// # Safety
/// `p` must be a channel.
unsafe fn chan(p: *mut Header) -> &'static Chan {
    &**((p as *mut *mut Chan).add(2))
}

#[no_mangle]
pub extern "C" fn kode_chan_new() -> *mut Header {
    // Channels exist to move values between threads, so reference counts must be atomic.
    THREADED.store(true, Ordering::SeqCst);
    unsafe {
        let p = alloc_object(SIZE, CHAN);
        let boxed = Box::new(Chan {
            inner: Mutex::new(Inner {
                items: VecDeque::new(),
                closed: false,
            }),
            ready: Condvar::new(),
        });
        *((p as *mut *mut Chan).add(2)) = Box::into_raw(boxed);
        p
    }
}

/// Queue a payload. The channel takes ownership of `raw` when `is_ref` is 1.
/// Sending on a closed channel raises a catchable error.
///
/// # Safety
/// `p` must be a channel.
#[no_mangle]
pub unsafe extern "C" fn kode_chan_send(p: *mut Header, raw: i64, is_ref: i64) {
    let c = chan(p);
    let mut guard = c.inner.lock().unwrap_or_else(|e| e.into_inner());
    if guard.closed {
        drop(guard);
        throw_msg("send on a closed channel");
        return;
    }
    if is_ref != 0 {
        kode_retain(raw as *mut Header);
    }
    guard.items.push_back((raw as u64, is_ref != 0));
    drop(guard);
    c.ready.notify_one();
}

/// Wait for a payload (the caller owns it when it is a heap value).
/// Receiving from a closed, empty channel raises a catchable error.
///
/// # Safety
/// `p` must be a channel.
#[no_mangle]
pub unsafe extern "C" fn kode_chan_recv(p: *mut Header) -> i64 {
    let c = chan(p);
    let mut guard = c.inner.lock().unwrap_or_else(|e| e.into_inner());
    loop {
        if let Some((raw, _)) = guard.items.pop_front() {
            return raw as i64;
        }
        if guard.closed {
            drop(guard);
            throw_msg("receive from a closed channel");
            return 0;
        }
        guard = c.ready.wait(guard).unwrap_or_else(|e| e.into_inner());
    }
}

/// Close the channel. Values already queued can still be received.
///
/// # Safety
/// `p` must be a channel.
#[no_mangle]
pub unsafe extern "C" fn kode_chan_close(p: *mut Header) {
    let c = chan(p);
    let mut guard = c.inner.lock().unwrap_or_else(|e| e.into_inner());
    guard.closed = true;
    drop(guard);
    c.ready.notify_all();
}

pub unsafe fn drop_chan(p: *mut Header) {
    let boxed = Box::from_raw(*((p as *mut *mut Chan).add(2)));
    let inner = std::mem::take(&mut *boxed.inner.lock().unwrap_or_else(|e| e.into_inner()));
    for (raw, is_ref) in inner.items {
        if is_ref {
            kode_release(raw as *mut Header);
        }
    }
    free_object(p, SIZE);
}

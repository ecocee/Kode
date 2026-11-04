//! `throw` / `catch` and runtime errors.
//!
//! A thrown error is parked in one global slot. After a call that can throw,
//! compiled code asks `kode_err_pending` and jumps to its handler if set.

use std::io::Write;
use std::cell::Cell;

use crate::io::flush;
use crate::object::{kode_release, Header};
use crate::string::bytes;

thread_local! {
    static ERR: Cell<*mut Header> = const { Cell::new(std::ptr::null_mut()) };
}

/// Take this thread's pending error (null if none).
pub fn take_err() -> *mut Header {
    ERR.with(|e| e.replace(std::ptr::null_mut()))
}

/// Build an `Error` object (message and code).
pub fn make_error(msg: &str, code: i64) -> *mut Header {
    unsafe {
        let e = crate::object::kode_obj_new(2, 1);
        *((e as *mut *mut Header).add(3)) = crate::string::from_bytes(msg.as_bytes());
        *((e as *mut i64).add(4)) = code;
        e
    }
}

/// Raise an error with a message from inside a runtime function.
pub fn throw_msg(msg: &str) {
    unsafe { kode_err_throw(make_error(msg, 0)) }
}

/// Stop the program with a runtime error.
pub fn fatal(msg: &str) -> ! {
    flush();
    let _ = writeln!(std::io::stderr(), "kode: runtime error: {msg}");
    std::process::exit(1);
}

/// # Safety
/// `obj` must be an `Error` object; the runtime takes ownership.
#[no_mangle]
pub unsafe extern "C" fn kode_err_throw(obj: *mut Header) {
    let old = ERR.with(|e| e.replace(obj));
    kode_release(old);
}

#[no_mangle]
pub extern "C" fn kode_err_pending() -> i64 {
    ERR.with(|e| !e.get().is_null()) as i64
}

/// Take the pending error; the caller owns it.
#[no_mangle]
pub extern "C" fn kode_err_take() -> *mut Header {
    take_err()
}

/// An error reached the top of the program.
#[no_mangle]
pub extern "C" fn kode_uncaught() -> ! {
    let e = kode_err_take();
    flush();
    unsafe {
        let msg_ptr = *((e as *const *const Header).add(3));
        let code = *((e as *const i64).add(4));
        let msg = if msg_ptr.is_null() {
            String::new()
        } else {
            String::from_utf8_lossy(bytes(msg_ptr)).to_string()
        };
        if code != 0 {
            let _ = writeln!(std::io::stderr(), "kode: error: {msg} (code {code})");
        } else {
            let _ = writeln!(std::io::stderr(), "kode: error: {msg}");
        }
    }
    std::process::exit(1);
}

#[no_mangle]
pub extern "C" fn kode_unwrap_fail(line: i64) -> ! {
    fatal(&format!("`!` was used on a none value at line {line}"));
}

#[no_mangle]
pub extern "C" fn kode_div_zero() -> ! {
    fatal("division by zero");
}

#[no_mangle]
pub extern "C" fn kode_exit(code: i64) -> ! {
    flush();
    std::process::exit(code as i32);
}
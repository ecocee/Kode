//! `throw` / `catch` and runtime errors.
//!
//! A thrown error is parked in one global slot. After a call that can throw,
//! compiled code asks `kode_err_pending` and jumps to its handler if set.

use std::io::Write;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::io::flush;
use crate::object::{kode_release, Header};
use crate::string::bytes;

static ERR: AtomicPtr<Header> = AtomicPtr::new(std::ptr::null_mut());

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
    let old = ERR.swap(obj, Ordering::SeqCst);
    kode_release(old);
}

#[no_mangle]
pub extern "C" fn kode_err_pending() -> i64 {
    (!ERR.load(Ordering::SeqCst).is_null()) as i64
}

/// Take the pending error; the caller owns it.
#[no_mangle]
pub extern "C" fn kode_err_take() -> *mut Header {
    ERR.swap(std::ptr::null_mut(), Ordering::SeqCst)
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
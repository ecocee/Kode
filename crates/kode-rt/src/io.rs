use std::io::Write;

use crate::object::Header;
use crate::string::{bytes, format_float};

pub(crate) fn flush() {
    let _ = std::io::stdout().flush();
}

// ---- stderr: `stderr(...)` writes like `print`, on the diagnostic stream ----

#[no_mangle]
pub extern "C" fn kode_err_int(v: i64) {
    let _ = write!(std::io::stderr().lock(), "{v}");
}

#[no_mangle]
pub extern "C" fn kode_err_float(v: f64) {
    let _ = write!(std::io::stderr().lock(), "{}", format_float(v));
}

#[no_mangle]
pub extern "C" fn kode_err_bool(v: i64) {
    let _ = std::io::stderr()
        .lock()
        .write_all(if v != 0 { b"true" } else { b"false" });
}

/// # Safety
/// `p` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_err_str(p: *const Header) {
    let _ = std::io::stderr().lock().write_all(bytes(p));
}

#[no_mangle]
pub extern "C" fn kode_err_sep() {
    let _ = std::io::stderr().lock().write_all(b" ");
}

#[no_mangle]
pub extern "C" fn kode_err_nl() {
    let mut out = std::io::stderr().lock();
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

#[no_mangle]
pub extern "C" fn kode_print_int(v: i64) {
    let _ = write!(std::io::stdout().lock(), "{v}");
}

#[no_mangle]
pub extern "C" fn kode_print_float(v: f64) {
    let _ = write!(std::io::stdout().lock(), "{}", format_float(v));
}

#[no_mangle]
pub extern "C" fn kode_print_bool(v: i64) {
    let _ = std::io::stdout()
        .lock()
        .write_all(if v != 0 { b"true" } else { b"false" });
}

/// # Safety
/// `p` must be a valid string.
#[no_mangle]
pub unsafe extern "C" fn kode_print_str(p: *const Header) {
    let _ = std::io::stdout().lock().write_all(bytes(p));
}

#[no_mangle]
pub extern "C" fn kode_print_sep() {
    let _ = std::io::stdout().lock().write_all(b" ");
}

#[no_mangle]
pub extern "C" fn kode_print_nl() {
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(b"\n");
    let _ = out.flush();
}

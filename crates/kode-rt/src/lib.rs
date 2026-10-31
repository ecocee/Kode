//! Runtime library linked into every compiled Kode program.
//! Compiled code calls these `extern "C"` functions.
//!
//! * `string`: reference-counted strings
//! * `io`: printing
//! * `panic`: runtime errors that stop the program with a clear message

mod io;
mod panic;
mod string;

/// Called once when the program ends. With `KODE_LEAK_CHECK=1` it reports
/// strings that were never freed (used by the test suite).
#[no_mangle]
pub extern "C" fn kode_rt_finish() {
    io::flush();
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        let live = string::live_strings();
        if live != 0 {
            eprintln!("kode: memory leak: {live} string(s) were never freed");
            std::process::exit(3);
        }
    }
}

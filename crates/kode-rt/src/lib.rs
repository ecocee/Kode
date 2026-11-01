//! Runtime library linked into every compiled Kode program.
//! Compiled code calls these `extern "C"` functions.
//!
//! * `object`: the shared header and reference counting for every heap value
//! * `string`, `list`, `map`: the built-in heap types
//! * `error`: `throw`, `catch` and runtime errors
//! * `io`: printing

mod error;
mod io;
mod list;
mod map;
mod object;
mod string;

/// Called once when the program ends. With `KODE_LEAK_CHECK=1` it reports
/// objects that were never freed (used by the test suite).
#[no_mangle]
pub extern "C" fn kode_rt_finish() {
    io::flush();
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        let live = object::live_objects();
        if live != 0 {
            eprintln!("kode: memory leak: {live} object(s) were never freed");
            std::process::exit(3);
        }
    }
}
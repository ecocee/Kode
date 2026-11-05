//! Runtime library linked into every compiled Kode program.
//! Compiled code calls these `extern "C"` functions.
//!
//! * `object`: the shared header and reference counting for every heap value
//! * `string`, `list`, `map`: the built-in heap types
//! * `error`: `throw`, `catch` and runtime errors
//! * `task`: `spawn` / `await`; `http`: the web server; `http_client`: the HTTP client
//! * `json`: dynamic JSON values and their parser; `std_fns`: strings, files, sorting, ...
//! * `router`: the `Router` / `Request` / `Response` web API layer
//! * `io`: printing

mod channel;
mod error;
mod http;
mod http_client;
mod io;
mod json;
mod json_parse;
mod list;
mod map;
mod math;
mod object;
mod router;
mod std_fns;
mod string;
mod task;

/// ABI marker. The compiler checks that the runtime library it links contains this
/// symbol name. Rename it (and `RT_ABI` in `kode-codegen/src/link.rs`) whenever the
/// runtime functions or object layouts change, so an old library is never used by mistake.
#[no_mangle]
pub extern "C" fn kode_rt_abi_0002() {}

/// Called once when the program ends. With `KODE_LEAK_CHECK=1` it reports
/// objects that were never freed (used by the test suite).
#[no_mangle]
pub extern "C" fn kode_rt_finish() {
    kode_rt_abi_0002();
    io::flush();
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        let live = object::live_objects();
        if object::COUNT_LIVE.load(std::sync::atomic::Ordering::Relaxed) && live != 0 {
            eprintln!("kode: memory leak: {live} object(s) were never freed");
            std::process::exit(3);
        }
    }
}
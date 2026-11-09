//! Runtime library linked into every compiled Kode program.
//! Compiled code calls these `extern "C"` functions.
//!
//! * `object`: the shared header and reference counting for every heap value
//! * `string`, `list`, `map`: the built-in heap types
//! * `error`: `throw`, `catch` and runtime errors
//! * `task`: `spawn` / `await`; `http`: the web server; `http_client`: the HTTP client
//! * `http_wire`: request parsing, limits, timeouts and response writing shared by both servers
//! * `json`: dynamic JSON values and their parser; `std_fns`: strings, files, sorting, ...
//! * `process`: `exec`, so a Kode program can run a compiler or a linker
//! * `router`: the `Router` / `Request` / `Response` web API layer
//! * `tls`: client TLS (rustls), shared by the PostgreSQL and HTTP clients
//! * `io`: printing; `repr`: the text of a list or map

mod base64;
mod channel;
mod clock_time;
mod error;
mod http;
mod http_client;
mod http_wire;
mod io;
mod json;
mod json_parse;
mod list;
mod map;
mod math;
mod object;
mod pg;
mod pg_message;
mod pg_scram;
mod pg_url;
mod pg_wire;
mod process;
mod repr;
mod router;
mod sha256;
mod std_fns;
mod string;
mod task;
mod tls;

/// ABI marker. The compiler checks that the runtime library it links contains this
/// symbol name. Rename it (and `RT_ABI` in `kode-cli/src/doctor.rs`) whenever the
/// runtime functions or object layouts change, so an old library is never used by mistake.
#[no_mangle]
pub extern "C" fn kode_rt_abi_0004() {}

/// Called once when the program ends. With `KODE_LEAK_CHECK=1` it reports
/// objects that were never freed (used by the test suite).
#[no_mangle]
pub extern "C" fn kode_rt_finish() {
    kode_rt_abi_0004();
    io::flush();
    if std::env::var_os("KODE_LEAK_CHECK").is_some() {
        let live = object::live_objects();
        if object::COUNT_LIVE.load(std::sync::atomic::Ordering::Relaxed) && live != 0 {
            eprintln!("kode: memory leak: {live} object(s) were never freed");
            std::process::exit(3);
        }
    }
}

//! `exec(program, args, cwd)`: run another program and collect its output.
//!
//! This is the process API a self-hosted Kode compiler needs: it must be able to
//! call the C compiler and the linker. Arguments are passed as a list, never
//! through a shell, so a path or an argument containing spaces or quotes is safe.
//!
//! `std::process::Command::output` reads stdout and stderr concurrently, so a
//! program that writes more than a pipe buffer to either stream cannot deadlock.
//! stdin is closed: a program that asks for input sees end of file instead of
//! hanging the compiler.

use std::process::Command;

use crate::error::throw_msg;
use crate::list::{kode_list_get, kode_list_len};
use crate::object::{kode_obj_new, Header};
use crate::string::{bytes, from_bytes};

/// `ProcessResult { code: int, stdout: string, stderr: string }`, mirroring the
/// class declared in `kode-sema/src/check/mod.rs`.
const FIELDS: i64 = 3;
/// Fields 1 and 2 are strings; field 0 (`code`) is an int.
const MASK: i64 = 0b110;

unsafe fn text(p: *const Header) -> String {
    String::from_utf8_lossy(bytes(p)).to_string()
}

unsafe fn result(code: i64, out: &[u8], err: &[u8]) -> *mut Header {
    let p = kode_obj_new(FIELDS, MASK);
    *((p as *mut i64).add(3)) = code;
    *((p as *mut *mut Header).add(4)) = from_bytes(out);
    *((p as *mut *mut Header).add(5)) = from_bytes(err);
    p
}

/// # Safety
/// `program` and `cwd` must be valid strings, and `args` a list of strings.
#[no_mangle]
pub unsafe extern "C" fn kode_proc_exec(
    program: *const Header,
    args: *const Header,
    cwd: *const Header,
) -> *mut Header {
    let name = text(program);
    if name.is_empty() {
        throw_msg("exec needs a program name");
        return std::ptr::null_mut();
    }
    let mut cmd = Command::new(&name);
    for i in 0..kode_list_len(args) {
        let item = kode_list_get(args as *mut Header, i, 0) as *const Header;
        cmd.arg(text(item));
    }
    let dir = text(cwd);
    if !dir.is_empty() {
        cmd.current_dir(&dir);
    }
    match cmd.output() {
        Ok(out) => result(
            out.status.code().map(|c| c as i64).unwrap_or(-1),
            &out.stdout,
            &out.stderr,
        ),
        Err(e) => {
            throw_msg(&format!("cannot run `{name}`: {e}"));
            std::ptr::null_mut()
        }
    }
}

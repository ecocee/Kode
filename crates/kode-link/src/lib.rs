//! Linking a Kode object file with the runtime archive.
//!
//! Code generation lives in `kode-llvm`: it turns MIR into textual LLVM IR and
//! asks clang for an object. This crate only knows how to hand that object and
//! the runtime to the platform C compiler, which is the linker driver Kode uses
//! on every supported platform.

use std::process::Command;

mod link;

pub use link::{cc_program, link};

/// Force `std::process` to start children through fork/exec instead of its
/// `posix_spawn` fast path.
///
/// `posix_spawn` can report success for a child whose `exec` failed. On Linux
/// x86-64 under Docker's Rosetta emulation that turned a missing clang or linker
/// into exit status 127 instead of a spawn error. The fork path returns the
/// exec errno through a pipe, so the caller sees `Err` and can print its install
/// hint. No-op on Windows, which has no fork.
pub fn force_fork_exec(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: the closure runs in the forked child, before exec, where only
        // async-signal-safe work is allowed. It does nothing and reports success,
        // so it allocates nothing, takes no locks and calls no libc function; the
        // fork/exec machinery itself is `std`'s responsibility.
        unsafe {
            cmd.pre_exec(|| Ok(()));
        }
    }
    #[cfg(not(unix))]
    let _ = cmd;
}

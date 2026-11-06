# ADR 0029: Running other programs (`exec`)

Status: accepted. 2026-11-05.

## Context

A self-hosted Kode compiler has to call the platform toolchain: an assembler or
`cc` to turn generated assembly into an object file, and a linker to produce an
executable. Kode could read and write files but had no way to start a process, so
the native backend could not be written in Kode at all. This is the first
capability the bootstrap needs that the language did not have.

Three questions had to be answered: what the call looks like, how arguments are
passed, and what happens to the three output streams.

## Decision

### One free function, one result object

```
r = exec("/usr/bin/cc", ["-c", "main.s", "-o", "main.o"], "")
print(r.code, r.stdout, r.stderr)
```

`exec(program, args, cwd) -> ProcessResult`, where `ProcessResult` is a built-in
class with `code: int`, `stdout: string` and `stderr: string`. The class follows
the `HttpResponse` precedent: declared in `kode-sema/src/check/mod.rs` and built
by the runtime with the same field order (`kode-rt/src/process.rs`). An empty
`cwd` means "inherit this process's working directory".

The call is synchronous and returns only when the program exits. A compiler is a
batch tool: there is nothing useful to do while `cc` runs, and a synchronous call
needs no new scheduling machinery. `spawn` remains available for work that should
run concurrently.

### Arguments never pass through a shell

`args` is a `[string]` handed straight to the platform's `exec`, so a filename or
argument containing a space, quote, `$` or `;` is delivered literally. Building a
command line as one string and handing it to a shell is the usual source of
injection bugs; leaving the shell out removes the class of bug entirely.

`tests/run-pass/process_posix.kode` pins this down: `/bin/echo` receives `a b`
and `$HOME` as two arguments and prints them unchanged.

### Both streams are captured, and stdin is closed

The runtime uses `std::process::Command::output`, which reads stdout and stderr
**concurrently**; a program that writes more than a pipe buffer to stderr while
its stdout is being read cannot deadlock. stdin is closed, so a program that
prompts for input sees end of file instead of hanging a build.

A full pipe buffer is a real failure mode for a compiler driver — `cc` writes
warnings to stderr — which is why this is not left to the caller.

### Failure to start is a catchable error

A missing program, a missing permission bit or a `cwd` that does not exist throws
a normal Kode error (`cannot run ...`) that `try`/`catch` can handle. A program
that runs and exits non-zero is **not** an error: it is a result with a non-zero
`code`, because a compiler must be able to read `cc`'s diagnostics from `stderr`
and decide what to report.

If the process is killed by a signal, `code` is `-1`.

## Consequences

- `exec` joins the standard-library table (`kode-sema/src/std_fns.rs`) as a free
  function, so it needs no new MIR instruction and no codegen change: it is an
  ordinary runtime call.
- The runtime ABI marker moves to `kode_rt_abi_0003`, so an older runtime library
  is never linked against a program that calls `exec`.
- The conformance runner gained one narrow convention, `// skip-os: windows` (or
  `unix`) on a test's first line, because a test that runs `/bin/sh` cannot run
  on Windows. Nothing else is skipped, and the portable failure-path test still
  runs everywhere.
- Timeouts, environment overrides, stdin input and streaming output are **not**
  provided. A compiler driver does not need them; adding them later is additive
  (a second result field or a `exec_with` variant) and does not change this
  shape.

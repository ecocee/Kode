# 45. A `stderr` stream

- Status: accepted, and implemented
- Date: 2026-11-18
- Work packet: WP-10

## Context

A compiler writes two kinds of output and they go to different places: the
program's answer on stdout, and diagnostics on stderr. Kode had only `print`,
which writes to stdout, so the port reported parse errors by opening a device
file:

```
write_file("/dev/stderr", report)
```

That works on Unix and nowhere else, it reopens the stream for every report, and
it writes bytes while the runtime holds stdout in a locked handle — so the two
streams have no ordering discipline at all. `docs/self-hosting.md` listed "no
stderr stream" as the last language gap S5 needed; the WP-10 probe was:

```
stderr("kode: a diagnostic")     # E0206: unknown function `stderr`
print("stdout still works")
```

## Decision

`stderr(...)` is a builtin with **the same argument rules as `print`**:
variadic, space-separated, one newline at the end, the same conversions for
floats, enums, optionals, lists and maps, and the same rejections (`E0203` for a
value that cannot be printed, `E0205` for a call that returns nothing). Only the
stream differs.

The alternative — `stderr(text: string)` taking one string — was rejected: a
diagnostic is usually assembled from a message and a location, and forcing that
through interpolation or `string()` makes every call site construct a string the
runtime is about to copy anyway. Sharing `print`'s rules also means there is one
formatting path to keep true, so `stderr(1.5)` and `print(1.5)` can never drift.

The runtime side is six thin symbols in `crates/kode-rt/src/io.rs`
(`kode_err_int/float/bool/str/sep/nl`) mirroring the `kode_print_*` family;
`kode_err_nl` flushes stderr, so a diagnostic is on the terminal before a crash
or an `exit`. In MIR the `print` arm of `builtin()` serves both builtins and
picks the stream once (`Builtin::Print | Builtin::Stderr`). The runtime ABI
marker moves to `0006`.

The port switches from one `write_file` of a concatenated report to one
`stderr(render(...))` per error. `render` already ends with a newline and
`stderr` adds one, which is exactly the oracle's blank line between reports —
so `selfhost/spec/parse-errors` still compares byte for byte.

## Alternatives

- **A `Stream` object (`err = stderr()`, `err.write(...)`).** Rejected: a new
  value type, methods and an owning handle for one call shape, and the port has
  no use for a handle it passes around.
- **A `to: stderr` named argument on `print`.** Rejected: named arguments for
  builtins are new syntax for no additional capability.
- **Keep `write_file("/dev/stderr")`.** Rejected: Unix-only, and the probe exists
  because S5 must be able to report failures on Windows too.

## Consequences

- Diagnostics no longer depend on a device file, and the port drops its last
  platform-specific workaround.
- stdout and stderr are flushed separately: a program that prints on both
  streams sees each stream in order, not their interleaving.
- `stderr` is not a value and has no handle, so it cannot be passed to a
  function that logs. A logging handle would be its own feature.
- A run-pass program that writes to stderr states its expected text in a
  `name.err` file; `llvm_conformance` compares it exactly, and a program without
  one must keep stderr empty, so the leak check still fails loudly.

## Evidence

- `selfhost/spec/gates/wp10-stderr.kode` compiles and runs: `gates` reports
  **11 met, 2 blocked** (`weak`, `u8`/`i32`).
- `./scripts/selfhost.sh all` stays green with the port's diagnostics routed
  through `stderr`: stage0 and stage1 green, 51/51 parse cases byte-identical
  (stdout *and* stderr), 25/25 self-parse, 270 token dumps.
- `tests/run-pass/stderr_stream.kode` (+ `.out`, `.err`) covers a plain line,
  several arguments, a list, an empty call, and stdout still working;
  `tests/compile-fail/e0203_stderr_bytes.kode` pins that `bytes` cannot be
  written.

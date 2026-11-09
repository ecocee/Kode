# kode-llvm

MIR in, textual LLVM IR out, `clang` builds the object, the existing linker and
runtime produce the binary. This is the compiler's only backend (ADR 0035): `kode`
calls `emit` + `compile_ir` for every build. The `kode-llvm` binary below is a
development tool for looking at one file's IR without the CLI.

```text
kode-llvm emit  file.kode [-o file.ll]   # the IR, for reading
kode-llvm build file.kode [-o program]   # IR -> object -> binary
kode-llvm run   file.kode [-- args]      # build in a temp dir and run
```

`KODE_CLANG` selects the compiler (default `clang`), `KODE_RT_LIB` the runtime
archive (default: newest `libkode_rt.a` under `target/`), and
`KODE_LLVM_KEEP_IR=1` keeps the `.ll` next to the object.

## What the lowering covers

| MIR | LLVM |
|---|---|
| `Int`, `Ref` | `i64` |
| `Float` | `double` |
| `Bool` | `i8` |
| functions, params, returns | `define`, allocas for locals (mem2reg at `-O2`) |
| `Add`/`Sub`/`Mul`, bitwise, shifts | `add`/`sub`/`mul`/`and`/`or`/`xor`, `shl`/`ashr` behind an `icmp uge 64` guard |
| `Div`/`Rem` | `kode_div_zero()` on a zero divisor, `MIN / -1` wraps via `select` |
| comparisons | `icmp`/`fcmp` + `zext` to `i8` |
| `Cast` | `sitofp`, **saturating** `llvm.fptosi.sat`, `bitcast`, `zext`, `icmp` |
| `Select` | `select` |
| blocks, `Goto`, `Branch`, `Return` | labels, `br`, `ret` |
| `Call` user / `Rt` / `Std` | direct calls with the C ABI of `kode-rt` |
| `CallIndirect` | code pointer at offset 24, `inttoptr`, indirect call |
| `Retain`/`Release` | `kode_retain`/`kode_release` |
| `Load`/`Store` | `getelementptr` + `load`/`store` |
| string literals | `[rc=-1][kind=1][len][bytes]` globals, the runtime's layout |

Errors: `clang` missing prints what is missing and how to point at it
(`KODE_CLANG`); rejected IR prints clang's own diagnostic and the offending
line. There is no fallback to another backend, by design.

## Tests

`cargo test -p kode-llvm` covers IR shape (no clang needed) and end-to-end
programs built and run with `KODE_LEAK_CHECK=1`: primitives, strings and
reference counting, collections, objects and dispatch, errors, tasks and
channels. The CLI's conformance test (`crates/kode-cli/tests/llvm_conformance.rs`)
runs every `tests/run-pass` program, the examples and every compile-fail program
through `kode`.

The stage documents B2.md-B8.md record how each part was verified; B6 and B7
mention Cranelift because it was the reference backend until B8 removed it.

## Measurements

See `benchmarks/report.md` for the current table and
`benchmarks/llvm_readiness/run.py` to reproduce it. The short version: loop-heavy
code is faster than the removed Cranelift backend and within about 1.3x of C
`-O2`; runtime-bound code (maps, JSON, strings, tasks, I/O) is at parity because
it spends its time in `kode-rt`.

## Integration

`kode` calls `kode_llvm::emit` and `kode_llvm::compile_ir` from
`object_for` in `crates/kode-cli/src/main.rs`, then links through
`kode_codegen::link` with the runtime archive embedded in the CLI. The crate's
own `front.rs` (single-file parsing) exists for the `kode-llvm` development
binary and its tests; the CLI always uses its loader, so imports and packages
work.

## Known limits

- Debug info, `-g`, LTO and cross-compilation are not attempted.
- The IR is unoptimised before clang; `-O2` does the work.
- A missing clang is an error with no fallback; the compiler needs LLVM 15+.

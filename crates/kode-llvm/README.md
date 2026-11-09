# kode-llvm (stage B1, experimental)

MIR in, textual LLVM IR out, `clang` builds the object, the existing linker and
runtime produce the binary. This crate is a prototype for ADR 0035: it is not
wired into `kode`, it does not change the production backend, and Cranelift is
untouched.

```text
kode-llvm emit  file.kode [-o file.ll]   # the IR, for reading
kode-llvm build file.kode [-o program]   # IR -> object -> binary
kode-llvm run   file.kode [-- args]      # build in a temp dir and run
```

`KODE_CLANG` selects the compiler (default `clang`), `KODE_RT_LIB` the runtime
archive (default: newest `libkode_rt.a` under `target/`), and
`KODE_LLVM_KEEP_IR=1` keeps the `.ll` next to the object.

## What B1 covers

| MIR | LLVM |
|---|---|
| `Int`, `Ref` | `i64` |
| `Float` | `double` |
| `Bool` | `i8` |
| functions, params, returns | `define`, allocas for locals (mem2reg at `-O2`) |
| `Add`/`Sub`/`Mul`, bitwise, shifts | `add`/`sub`/`mul`/`and`/`or`/`xor`, `shl`/`ashr` behind an `icmp uge 64` guard |
| `Div`/`Rem` | `kode_div_zero()` on a zero divisor, `MIN / -1` wraps via `select`, like Cranelift |
| comparisons | `icmp`/`fcmp` + `zext` to `i8` |
| `Cast` | `sitofp`, **saturating** `llvm.fptosi.sat`, `bitcast`, `zext`, `icmp` |
| `Select` | `select` |
| blocks, `Goto`, `Branch`, `Return` | labels, `br`, `ret` |
| `Call` user / `Rt` / `Std` | direct calls with the C ABI of `kode-rt` |
| `CallIndirect` | code pointer at offset 24, `inttoptr`, indirect call |
| `Retain`/`Release` | `kode_retain`/`kode_release` |
| `Load`/`Store` | `getelementptr` + `load`/`store` |
| string literals | `[rc=-1][kind=1][len][bytes]` globals, same layout as Cranelift |

Errors: `clang` missing prints what is missing and how to point at it
(`KODE_CLANG`); rejected IR prints clang's own diagnostic and the offending
line. There is no fallback to another backend, by design.

## Tests

`cargo test -p kode-llvm` runs 12 tests: six IR-shape tests (no clang needed) and
six end-to-end tests that build and run programs through the real binary.

As an out-of-tree check, all 45 `tests/run-pass` programs were built with this
backend and run: **44 print byte-identical output**. The exception is
`modules_alias.kode`, which needs `kode-cli`'s import/package loader; the
prototype parses a single file. List, map, class, closure, error and channel
programs already work because the runtime calls are generic, not because those
stages are done.

## Measurements

macOS, arm64, 10-core laptop; best of three runs; programs built by each backend
and run without arguments.

| Program | Cranelift | LLVM (`-O2`) | C `-O2` | binary (Cranelift / LLVM) |
|---|---:|---:|---:|---:|
| fib(38) | 0.095 s | **0.059 s** | 0.058 s | 1,236,392 / 1,234,184 |
| loop_mod | 0.031 s | **0.016 s** | 0.012 s | 1,236,392 / 1,234,144 |
| mandel | 0.015 s | 0.017 s | 0.012 s | 1,236,392 / 1,234,136 |
| sieve | 0.028 s | **0.303 s** | 0.013 s | 1,371,208 / 1,368,360 |

Correctness: all four print the same value as Cranelift, C and
`benchmarks/*/expected.txt` (39088169, 24975000000, 6292386, 664579).

Compile time for `fib` is 0.051 s for both backends on this machine (the
prototype's simpler front end pays for the `clang` invocation).

`KODE_LEAK_CHECK=1` reports no leaks for `fib` or `sieve` built by this backend.

## Why `sieve` is slow here

Cranelift inlines `list.len`, item reads/writes and the push fast path
(`try_inline_list` in `kode-codegen/src/function.rs`). B1 calls the runtime for
every one of them, and sieve performs about 30 million of those calls. This is
stage B3 work, not a code-generation defect: `fib` and `loop_mod`, which have no
collections, now match C.

## Integration changes required (not made)

No existing file had to change for this prototype, and none was changed. To put
it behind `kode`:

1. `crates/kode-cli/src/main.rs`: accept `--backend=llvm` and, in `object_for`,
   call `kode_llvm::emit(&mir)` + `kode_llvm::compile_ir(...)` instead of
   `kode_codegen::compile_to_object`. The linker call and the embedded runtime
   archive are reused unchanged.
2. `crates/kode-cli/src/main.rs` (front end): pass the MIR the CLI already
   builds; the prototype's `front.rs` then becomes test-only. Imports and
   packages keep working because the CLI keeps loading them.
3. Later stages (B6/B7): flip the default and remove Cranelift, per ADR 0035.

Nothing else: the workspace's `members = ["crates/*"]` already builds this
crate, and `kode-codegen::link` is public.

## Known limits at B1

- Single-file programs only; no `import`, no `kode.toml`.
- No inlined collection fast paths (see sieve), no `Repr` specialisation.
- Debug info, `-g`, LTO and cross-compilation are not attempted.
- The IR is unoptimised before clang; `-O2` does the work.

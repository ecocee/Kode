# ADR 0035: LLVM as the production backend

Status: implemented (2026-11-09). LLVM became the default in B7 and the only
backend in B8; Cranelift, its crates and the backend-selection flag are gone.
`--backend=llvm` is still accepted as a no-op. Supersedes the "Cranelift only"
assumption in ADR 0002.

## Context

Cranelift is fast to compile and has carried the language through M7, but the
benchmark report has shown a stable gap against C `-O2` that is code quality,
not runtime work: `fib` 1.62x, the hot loop 2.40x, `mandel` 1.4x on the same
machine, with the same runtime functions. Instruction selection, register
allocation and the absence of a loop optimizer live in the backend, and
Cranelift's optimizer is deliberately minimal.

Two paths were possible: keep improving the Cranelift lowering, or emit LLVM IR
and let `clang`/LLVM own optimization. The maintainer chose LLVM as the final
production backend and as the only backend once it passes conformance.

## Decision

1. **A new crate, `crates/kode-llvm`, emits textual LLVM IR** from `MirProgram`
   and nothing else. Rule 4 still holds: the backend (`kode-llvm`, the linker is `kode-link`) may not read the AST,
   and the LLVM crate must not either.
2. **No `llvm-sys`, no `inkwell`, no embedded LLVM.** The compiler writes `.ll`
   text to a temporary directory and invokes `clang -O2 -c` to produce the
   object, then links with the same runtime archive the Cranelift path links.
   The compiler stays buildable with `cargo build` alone; LLVM is a build-time
   tool, never a Rust dependency.
3. **A missing `clang` is a clear error**, not a fallback: `E0900` (toolchain)
   "the LLVM backend needs `clang` on PATH", with the exact command that failed.
   There is no fallback in either direction; after B8 `--backend=cranelift`
   reports that the backend was removed.
4. **LLVM becomes the default** when and only when all of these pass on LLVM:
   the full `tests/run-pass` suite with `KODE_LEAK_CHECK=1`, the full
   `tests/compile-fail` suite, `examples/*.kode` under `kode check`, the fmt
   round-trip, the HTTP/TLS runtime tests, and the benchmark report showing no
   correctness or memory regression. While any of them fails, `main` keeps
   Cranelift as default and the LLVM work stays on its branch.
5. **Cranelift is removed** after LLVM has been the default for the full gate,
   together with its dependencies, the backend-selection machinery and the
   `cranelift` module names (done in B8). The runtime archive and `RtFn` table
   stay the contract between MIR and the runtime.

## Staged migration

Each stage is a separate branch, and each one ends with the full gate
(`cargo build`, leak-checked tests, clippy, fmt check, `kode check`, hello size)
plus a diff of the produced binary's behaviour against Cranelift on the same
program:

| Stage | Branch | Scope |
|---|---|---|
| B1 | `feat/llvm-primitives` | crate, `--backend=llvm`, module/function/global emission, ints, floats, bools, arithmetic, comparisons, branches, direct calls, `println` of primitives |
| B2 | `feat/llvm-runtime` | the `kode_rt_*` ABI: strings, retain/release placement from MIR, string concat and comparison |
| B3 | `feat/llvm-collections` | lists, maps, indexing, iteration, bounds panics |
| B4 | `feat/llvm-objects` | classes, generics (monomorphized by sema already), closures and their environments |
| B5 | `feat/llvm-async` | errors/optional returns, channels, tasks, `await` |
| B6 | `feat/llvm-default` | flip the default after full conformance; benchmark report |
| B7 | `feat/llvm-remove-cranelift` | delete the Cranelift backend and its dependencies |
| B8 | `docs/llvm` | README, PLAN, STATUS, spec chapter 12, benchmark notes |

## Alternatives

- **`inkwell`/`llvm-sys` bindings.** Idiomatic Rust API, but pins the compiler
  to one LLVM major version, makes `cargo build` depend on `llvm-config`, and
  adds a large C++ dependency to a project whose selling point is a small,
  simple build. Rejected.
- **Keep Cranelift and improve it.** No loop-level optimizer to build on; the
  measured gaps would need hand-written passes in this repository. Rejected by
  the maintainer's decision.
- **Emit object files directly through LLVM's C API.** Same coupling problem as
  `llvm-sys`, without the Rust ergonomics.
- **A third backend later (e.g. wasm).** Textual IR behind a `Backend` trait
  leaves that door open without promising it.

## Consequences

- Building a Kode program now needs a C toolchain with LLVM: `clang` is already
  required for linking, so on the supported platforms this adds no new
  dependency, but LLVM 15+ is a documented requirement and `kode doctor` will
  report the detected version.
- Compile time will grow (clang invocation per program, roughly 50-150 ms on the
  benchmark machine). This is a build-time cost, not a runtime one, and it is
  measured in the benchmark report rather than assumed.
- The IR is text: readable in `kode mir --backend=llvm` output and debuggable
  with `clang -S`, but larger than a bitcode path would be. Temporary files go
  under the existing build directory and are cleaned up.
- During the migration two backends had to stay behaviourally identical and
  every conformance test ran on both. Since B8 there is one backend, so the
  suite now compares against the checked-in expectations and the runtime
  contract instead of a second implementation.
- Binary size may change; the 1 MB budget is already missed (STATUS), so the B7
  gate compared against the Cranelift build and recorded the delta rather than
  pretending the budget was met.

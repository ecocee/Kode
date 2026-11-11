# Self-hosting Kode: status

**Kode is not self-hosted.** The only compiler is the Rust one (Stage 0). This
page records exactly what exists, what is missing, and the order the remaining
work has to happen in, so progress does not have to be rediscovered.

Status date: 2026-11-11. The sequence of stages, their gates and their
acceptance tests are in [PLAN-self-hosting.md](PLAN-self-hosting.md).

## Verified baseline

| Item | Value |
|---|---|
| Rust compiler | **27,606 lines** in `src` across 8 crates (31,081 with tests) |
| `tests/run-pass` | 46 programs, each with a `.out` file |
| `tests/compile-fail` | 98 programs, each with a `.err` code |
| Conformance run | `cargo test --workspace`, all green, leak checking on |
| Runtime ABI marker | `kode_rt_abi_0004` |
| Native output | LLVM object file from `clang`, linked with the system `cc` |

## Language capabilities

### COMPLETE

- **Function values and closures** — ADR 0028. `fn(int) -> int` types,
  `|x: int| x * 2` literals, calling a value, passing and returning functions,
  `xs.map(f)` / `xs.filter(f)`. Closures are one heap object holding a code
  address and their captures. Tests: `tests/run-pass/{fn_values,closures,list_map_filter}.kode`.
- **Process execution** — ADR 0029. `exec(program, args, cwd)` runs a program
  without a shell and returns `ProcessResult { code, stdout, stderr }`. This is
  the capability a Kode-written compiler needs to call `cc` and `ld`.
  Tests: `tests/run-pass/{process,process_posix}.kode`.
- **Generic functions and monomorphization** — ADR 0030. Type parameters on free
  functions, inferred at each call site; every distinct type combination becomes
  its own native function. Tests:
  `tests/run-pass/{generics,generics_nested}.kode`.
- **Bootstrap harness and its oracles** — S0. `kode tokens` and `kode ast` print
  a stable dump (text, and `--format=json`) of token text, byte spans,
  line/column and tree shape; `scripts/selfhost.sh stage0|stage1|stage2|compare`
  dumps the whole corpus through them, checks Stage 0 against itself first, and
  records wall clock and peak RSS. `stage1` needs `selfhost/main.kode`, which
  arrives with the port. Format: `crates/kode-syntax/src/dump.rs`.

### NOT IMPLEMENTED

- **Generic classes, generic methods, constraints, explicit type arguments** —
  9.9. A generic function used as a value is E0251, a generic method is E0252.
- **Enum payloads and type tests.** Enums are tags only (ADR 0009); there is no
  `Shape.Circle(radius)`, no `x is Circle`, no safe downcast.
- **Fixed-width integers and binary data.** Only 64-bit `int`. No `u8`/`i32`
  families, no byte buffer, no `StringBuilder`, no binary encode/decode.
- **Integer map keys.** Maps are string-keyed by decision (ADR 0014).
- **Optional types beyond locals.** `string?`, `[T]?`, class optionals work as
  fields and parameters; `int?`/`float?`/`bool?` work only as locals (ADR 0024).
- **Cycle-safe memory management.** Reference counting only; no `weak`, no
  arenas, no regions.
- **A compiler written in Kode.** No lexer, parser, checker, MIR builder,
  evaluator or native backend exists in Kode.
- **Compiler-scale standard library.** No binary IO, no process streaming, no
  timeouts, no sorting with a comparator, no growable string builder.

## Dependency order for the remaining work

1. **Generics with monomorphization.** Every compiler data structure needs it:
   token buffers, AST node lists, symbol tables, diagnostics. This is the single
   biggest blocker and should be ADR 0030. Start with generic functions and
   `Vec<T>`-style collections over reference types, add inference from argument
   types only, and monomorphize in the checker so MIR stays concrete.
2. **Enum payloads and `is`/downcast.** The AST is a tagged union; without
   payloads it has to be modelled as N parallel fields plus a tag, which is
   exactly the duplication the mission forbids.
3. **Byte buffers, `StringBuilder` and fixed-width integers.** Needed to encode
   object files and instruction words, and to build output without quadratic
   string concatenation.
4. **Arena or `weak` references.** The AST is a tree with parent pointers; with
   reference counting alone a parent pointer is a cycle and leaks every node.
5. **Port order:** lexer/parser + spans, then the checker, then MIR, then an
   evaluator (which makes the port testable long before codegen works), then
   native code generation.
6. **Bootstrap:** `scripts/bootstrap.sh` running Stage 0 → Stage 1 → Stage 2,
   comparing diagnostics and program output, then object/binary bytes where the
   build is deterministic.

Detailed backend note: the LLVM path is the only one and is Stage 0. A
Kode-written backend should emit textual assembly and call `cc` through `exec`
(ADR 0029), rather than attempting to emit object files by hand. That is the
smallest path to a working Stage 1 backend and needs no FFI.

## Reproduce the current state

```sh
cargo build --workspace
cargo test --workspace                       # conformance + leak checks

./target/debug/kode run tests/run-pass/process_posix.kode
./target/debug/kode check --strict --format=json tests/compile-fail/e0208_exec_args.kode
./target/debug/kode mir tests/run-pass/closures.kode   # indirect calls
bash docs/site/build.sh
```

## Rules

Keep every stage green: `cargo test --workspace` must pass before each commit,
and the Rust compiler must remain able to build every Kode program in `tests/`.


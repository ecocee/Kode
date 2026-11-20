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
  records wall clock and peak RSS. Format: `crates/kode-syntax/src/dump.rs`.
- **The lexer, written in Kode** — S1. `selfhost/lexer.kode` is the scanner,
  `tokens.kode` its token set and the names the dump prints, `keywords.kode` the
  keyword table, `bytes.kode` the UTF-8 width and character-class predicates,
  `strings.kode` the `${...}` hole scanner, `dump.kode` the line format and
  `main.kode` the entry point. `./scripts/selfhost.sh stage1` builds it with Stage
  0 and agrees on **184 of 184** token dumps (text, byte spans, line/column, and
  the exit status of the cases that must fail to lex). Parser and later stages do
  not exist yet.

### NOT IMPLEMENTED

- **Generic classes, generic methods, constraints, explicit type arguments** —
  9.9. A generic function used as a value is E0251, a generic method is E0252.
- **Enum payloads and type tests.** Enums are tags only (ADR 0009); there is no
  `Shape.Circle(radius)`, no `x is Circle`, no safe downcast.
- **Fixed-width integers and binary data.** Only 64-bit `int`. No `u8`/`i32`
  families, no binary encode/decode. Byte buffers (`bytes(n)`, ADR 0043) and
  `StringBuilder` (ADR 0044) are done.
- **Integer map keys.** Maps are string-keyed by decision (ADR 0014).
- **Optional types beyond locals.** `string?`, `[T]?`, class optionals work as
  fields and parameters; `int?`/`float?`/`bool?` work only as locals (ADR 0024).
- **Cycle-safe memory management.** Reference counting only; no `weak`, no
  arenas, no regions.
- **A compiler written in Kode.** The lexer and parser do (`selfhost/lexer.kode` S1,
  `selfhost/parser.kode` S2, with the 128-level nesting limit in
  `selfhost/depth.kode`), and the checker resolves names across a loaded module
  graph (`selfhost/sema.kode`, `modules.kode`, `namespace.kode`, `names.kode`,
  `typing.kode`, `annotations.kode`, `items.kode`, `binding.kode`, `operands.kode`,
  `scope.kode`, `signatures.kode`, `functions.kode`, `stdlib.kode`, `containers.kode`,
  `optional.kode`, `members.kode`, `generics.kode`, `suggest.kode`; S3, 170/170
  programs on its gate with 170/170 reports byte-identical, and 55/55 of the port's own files
  clean); S4 has begun: `selfhost/mir.kode` lowers `print` of scalar literals, locals
  (declaration, reassignment), the five integer operators, run-time strings,
  interpolation and comparisons with the oracle's temporaries and retain/release
  lines, plus `if`/`else` and `while` blocks with `break`/`continue`, `fn` declarations
  and calls, and `selfhost/eval.kode` interprets it (`kode-stage1 mir` and `run`, 59/59
  fixtures each) while keeping shadow reference counts — a release of what it does not
  own, a use after release and a leak all fail the run (`./scripts/selfhost.sh own`,
  4/4 cases, and a mutant port that turns every `retain` into a `release` is caught), so
  the real corpus stands at 6/58 programs with the other 52 needing 320 distinct
  constructs, all listed in `selfhost/spec/corpus-out-of-scope.txt` (generated by
  `kode-stage1 report`, so it names every blocker and not just the first), and no native
  backend exists in Kode yet.
- **Compiler-scale standard library.** No binary IO, no process streaming, no
  timeouts and no sorting with a comparator. `stderr(...)` exists (ADR 0045), so
  the port reports diagnostics without a device file.

## Dependency order for the remaining work

1. **Generics with monomorphization.** Every compiler data structure needs it:
   token buffers, AST node lists, symbol tables, diagnostics. This is the single
   biggest blocker and should be ADR 0030. Start with generic functions and
   `Vec<T>`-style collections over reference types, add inference from argument
   types only, and monomorphize in the checker so MIR stays concrete.
2. **Enum payloads and `is`/downcast.** The AST is a tagged union; without
   payloads it has to be modelled as N parallel fields plus a tag, which is
   exactly the duplication the mission forbids.
3. **Fixed-width integers.** Byte buffers (ADR 0043) and `StringBuilder`
   (ADR 0044) are in; `u8`/`i32` are what is left for encoding object files and
   instruction words.
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


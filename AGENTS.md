# Instructions for AI agents working on Kode

Kode is a statically typed language compiled to small native binaries. The compiler is written in Rust and uses Cranelift. Memory is reference counted. Each program is one self-contained binary with only the runtime functions it uses. Read `README.md`, then `docs/PLAN-v1.md` (the current plan), before changing anything.

The maintainer works under 10 hours a week alone. Keep every change small, tested and reviewable.

## Commands

```
cargo build --workspace          # build compiler and runtime (needed before running tests)
cargo test --workspace           # conformance suite (sets KODE_LEAK_CHECK=1)
cargo clippy --workspace         # lints must stay clean
./target/debug/kode fmt --check <paths>
./target/debug/kode check file.kode --format=json   # structured errors
./target/debug/kode mir file.kode                    # see what the compiler produced
./target/debug/kode doctor                           # toolchain and package state
benchmarks/run.sh                                    # wall clock and peak RSS, writes benchmarks/report.md
```

## Where things live

Paths below were true when this file was last checked. If one is wrong, fix this file in the same change.

| To change... | Edit |
|---|---|
| tokens, keywords, string escapes | `crates/kode-syntax/src/token.rs`, `lexer.rs` |
| syntax tree | `crates/kode-syntax/src/ast/` |
| grammar | `crates/kode-syntax/src/parser/` (expr, stmt, item) |
| type rules, error messages | `crates/kode-sema/src/check/` |
| built-in and standard-library functions | `kode-sema/src/std_fns.rs` (one row per function; older code used `builtins.rs`), `kode-mir/src/lower/expr.rs`, `crates/kode-llvm/src/lower.rs` (the `RT` table), `kode-rt/src/` |
| memory management (retain/release) | `crates/kode-mir/src/lower/mod.rs` (rules), `expr.rs`, `data.rs`, `stmt.rs` |
| lists, maps, classes, optionals, errors in MIR | `crates/kode-mir/src/lower/data.rs` |
| generics and monomorphization | see ADR 0030, then `kode-sema` and `kode-mir` |
| closures, channels, tasks | ADRs 0026, 0028; `kode-mir`, `kode-rt` |
| `kode test` runner | `crates/kode-mir/src/lower/tests.rs`, `crates/kode-cli/src/main.rs` |
| imports, packages, `kode.toml`, `kode.lock` | `crates/kode-cli/src/driver.rs`, `source.rs` |
| `kode fmt`, `kode explain` | `crates/kode-cli/src/fmt.rs`, `explain.rs` (reads `docs/errors/index.md`) |
| language server | `kode lsp` in `kode-cli`; editor setup in `docs/editors.md` |
| `kode doc` and the docs site | `docs/site/build.sh` |
| PostgreSQL client | `crates/kode-rt/src/pg.rs` (API, handles, JSON results), `pg_wire.rs` (protocol, SCRAM), `sha256.rs` + `base64.rs` (SCRAM hashing) |
| LLVM IR and machine code | `crates/kode-llvm/src/lower.rs`, `clang.rs` |
| linking objects with the runtime | `crates/kode-link/src/link.rs` |
| runtime functions | `crates/kode-rt/src/` (object, string, list, map, error, io, and others); register new ones in `kode-mir/src/ir.rs` (`RtFn`) and `crates/kode-llvm/src/lower.rs` (`RT`) |
| benchmarks and measurements | `benchmarks/`, `benchmarks/report.md` |

## Rules

1. **Tests first.** A new feature starts as a `.kode` file with a `.out` file in `tests/run-pass/`, or a `.kode` file with an `.err` file (the error code) in `tests/compile-fail/`.
2. **Every error has a stable code** (`E0xxx`, warnings `W0xxx`) and a hint. Add new codes to `docs/errors/index.md`, and make sure `kode explain` works for them.
3. **Do not change language behavior without an ADR** in `docs/adr/` (context, decision, alternatives, consequences).
4. **Only MIR reaches the backend.** Do not read the AST in `kode-llvm`.
5. **Strings and objects are reference counted.** Follow the ownership rules at the top of `crates/kode-mir/src/lower/mod.rs`. `cargo test` must still report no leaks. Never loosen, skip or delete a leak check or a failing test to get green.
6. Keep files under about 500 lines; split by idea.
7. **Docs never lag the compiler.** When behavior changes, update in the same change: `README.md`, `CHANGELOG.md`, `docs/ROADMAP.md`, `docs/PLAN-v1.md`, `docs/cheatsheet.md` and the relevant `docs/spec/` chapter.
8. **No claim without a measurement.** Performance, memory and binary-size claims need before/after numbers in `benchmarks/report.md`. Do not merge an optimization without them.
9. **Keep compatibility.** `--format=json` diagnostics stay backward compatible, and `kode fmt` output stays stable unless an ADR says otherwise.
10. **Stay in scope.** Make the smallest change that passes the tests. No refactors outside the task.

## Workflow

- One branch per plan item, named `feat/<item>` or `fix/<item>`, with its tests, ADR and docs in the same commit set, so `git log` reads like the plan.
- Write a short plan (files, tests, risks) before starting an item and wait for approval at each gate in the plan.
- If a failure is still unresolved after two attempts, stop and report what you found.
- After each item, report: what changed, tests and ADRs added, measurements, risks, and the next item.

## Ask the maintainer first (do not decide alone)

- Third-party crates in the runtime (TLS, database, regex, crypto). Never hand-write crypto. Show the binary-size impact against the budget (under 1 MB stripped for a typical server).
- Any change to the memory model, the binary model or the backend.
- Any new syntax or breaking change to existing behavior.
- A performance budget that cannot be met.

## Non-goals for v1.0

A JIT, a shared `.so` runtime, a package-manager-installed VM, and the skipped M8 features (memory budgets, `uses` effects, typed `ask`/`tool`).
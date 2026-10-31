# Instructions for AI agents working on Kode

Kode is a compiled language written in Rust. Read `README.md` first.

## Commands

```
cargo build --workspace     # build compiler and runtime (needed before running tests)
cargo test --workspace      # run the conformance suite
./target/debug/kode check file.kode --format=json   # structured errors
./target/debug/kode mir file.kode                    # see what the compiler produced
```

## Where things live

| To change... | Edit |
|---|---|
| tokens, keywords, string escapes | `crates/kode-syntax/src/token.rs`, `lexer.rs` |
| syntax tree | `crates/kode-syntax/src/ast/` |
| grammar | `crates/kode-syntax/src/parser/` (expr, stmt, item) |
| type rules, error messages | `crates/kode-sema/src/check/` |
| built-in functions | `kode-sema/src/builtins.rs`, `kode-mir/src/lower/expr.rs`, `kode-codegen/src/runtime.rs`, `kode-rt/src/` |
| memory management (retain/release) | `crates/kode-mir/src/lower/` |
| machine code | `crates/kode-codegen/src/function.rs` |
| runtime functions | `crates/kode-rt/src/` |

## Rules

1. **Tests first.** A new feature starts as a `.kode` file with a `.out` file in `tests/run-pass/`,
   or a `.kode` file with an `.err` file (the error code) in `tests/compile-fail/`.
2. **Every error has a stable code** (`E0xxx`, warnings `W0xxx`). Add new codes to `docs/errors/index.md`.
3. **Do not change language behavior without an ADR** in `docs/adr/`.
4. **Only MIR reaches the backend.** Do not read the AST in `kode-codegen`.
5. Strings are reference counted. If you add a feature that creates strings, follow the
   ownership rules at the top of `crates/kode-mir/src/lower/mod.rs`, and make sure
   `cargo test` (which sets `KODE_LEAK_CHECK=1`) still reports no leaks.
6. Keep files under about 500 lines; split by idea.

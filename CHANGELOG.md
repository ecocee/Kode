# Changelog

## 0.2.0 (M2 to M5)
- Lists `[T]` and maps `{string: T}`: literals, indexing, `.add`, `.len`, `in`, `.keys`, `for x in list`, `for k, v in map`.
- Classes with fields, defaults, methods and named constructor arguments; enums; `match`.
- Optionals `T?` for heap types with `none`, `?.`, `??`, `!`.
- Errors: `throw`, `try / catch`, `expr catch e { ... }`, built-in `Error` class, errors that are freed on every path.
- Named arguments for every call.
- Imports across files (`import`, `from ... import`) with cycle and missing-file errors.
- `test` blocks and `kode test`; `kode fmt`; `kode explain`; `kode mir` shows field loads and stores.
- Runtime: one object header and one `release` for strings, lists, maps and objects; leak counter covers all of them.
- MIR: single exit block per function, error edges that release live temporaries.
- 19 run-pass programs (leak checked), 70 compile-fail programs, tests for `kode test`, `fmt`, `explain`.

## 0.1.0 (first working core)
- Rust rewrite: lexer, parser, type checker, MIR, Cranelift backend, runtime.
- Types: int, float, bool, string. Block-scoped variables, constants.
- Control flow: if / else if / else, while, for over ranges, break, continue.
- Functions: parameters, return values, recursion, expression bodies.
- Strings: reference counted, `+`, `${...}` interpolation, `len`, conversions.
- Diagnostics with stable codes, "did you mean" hints and JSON output.
- Conformance suite: 11 run-pass programs (leak checked) and 44 compile-fail programs.

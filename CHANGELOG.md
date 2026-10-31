# Changelog

## 0.1.0 (first working core)
- Rust rewrite: lexer, parser, type checker, MIR, Cranelift backend, runtime.
- Types: int, float, bool, string. Block-scoped variables, constants.
- Control flow: if / else if / else, while, for over ranges, break, continue.
- Functions: parameters, return values, recursion, expression bodies.
- Strings: reference counted, `+`, `${...}` interpolation, `len`, conversions.
- Diagnostics with stable codes, "did you mean" hints and JSON output.
- Conformance suite: 11 run-pass programs (leak checked) and 44 compile-fail programs.

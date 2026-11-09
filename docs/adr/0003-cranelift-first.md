# ADR 0003: Cranelift first, LLVM later
Status: superseded by ADR 0035 (LLVM); the Cranelift backend was removed in B8.
Kept as a historical decision record — the text below describes the original
choice, not the current compiler.

Cranelift is a small pure-Rust library that compiles very fast, which keeps the edit-run loop short.
LLVM generates faster code but is a very large dependency. Because only MIR reaches the backend,
an LLVM backend for `--release` can be added later without touching the front end.

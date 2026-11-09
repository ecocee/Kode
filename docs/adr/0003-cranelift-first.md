# ADR 0003: Cranelift first, LLVM later
Status: accepted; superseded in part by ADR 0035 (LLVM is now the production
backend, with Cranelift transitional until conformance passes)

Cranelift is a small pure-Rust library that compiles very fast, which keeps the edit-run loop short.
LLVM generates faster code but is a very large dependency. Because only MIR reaches the backend,
an LLVM backend for `--release` can be added later without touching the front end.

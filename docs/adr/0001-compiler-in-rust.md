# ADR 0001: The compiler is written in Rust
Status: accepted

Rust gives enums and `match` for syntax trees, safe code, fast builds of the compiler, and the
best compiler-backend libraries (Cranelift, LLVM bindings). An earlier Go version is kept in `legacy/`.

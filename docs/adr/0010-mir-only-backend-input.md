# ADR 0010: Only MIR reaches the backend
Status: accepted

`kode-codegen` never reads the AST. This keeps stages independent and lets us add or replace
backends (LLVM, WebAssembly) and add optimization passes on MIR.

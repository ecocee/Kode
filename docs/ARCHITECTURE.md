# Kode Architecture

*Last updated 2026*

## Design Philosophy

Kode is designed as a concurrency-first, statically typed language for backend and distributed systems. It emphasizes simplicity, safety, and performance through compilation to Go code.

## Compilation Pipeline

1. **Lexing**: Tokenize source code into tokens.
2. **Parsing**: Build AST from tokens, supporting types and concurrency.
3. **Type Checking**: Infer and check types using Hindley-Milner style.
4. **IR Generation**: Lower AST to custom SSA-like IR.
5. **Go Code Generation**: Emit idiomatic Go code from IR.
6. **Go Compilation**: Use Go toolchain to compile to binary.

## Concurrency Model

Kode provides CSP-style concurrency with channels and goroutines:

- `go` statements spawn goroutines.
- `chan` types for typed channels.
- `select` for channel multiplexing.

## Runtime

The runtime includes:
- Lightweight scheduler for goroutines.
- Channel abstractions with context-aware operations.
- Memory-safe execution.

## Ecosystem Vision

Future extensions include:
- Standard library for HTTP, JSON, etc.
- Package management.
- Cloud deployment tools.
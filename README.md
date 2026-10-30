# KODE

KODE is an experimental, Python-inspired, statically typed language implemented in Rust.

## Current status: v0.1.0

Supported:
- `print("text")`
- Multiple statements
- Comments beginning with `#`
- Escaped string characters: `\n`, `\r`, `\t`, `\"`, `\\`
- `kode run` and `kode check`

The current release uses a temporary interpreter. Native compilation, static types,
ownership checking, and Cranelift integration are future milestones.

## Build

```bash
cargo build
```

## Run an example

```bash
cargo run -- run examples/hello.kode
```

## Check syntax

```bash
cargo run -- check examples/hello.kode
```

## Release build

```bash
cargo build --release
```

## Design principles

- Python-inspired readable syntax
- Static typing with inference (planned)
- Ownership and borrowing (planned)
- Ahead-of-time native compilation (planned)
- Custom KODE intermediate representation (planned)
- Cranelift backend initially (planned)

## Project structure

- `kode-lexer`: source text to tokens
- `kode-parser`: tokens to AST
- `kode-ast`: language syntax tree
- `kode-interpreter`: temporary v0.1 execution engine
- `kode-types`: future type checker
- `kode-semantics`: future name and scope analysis
- `kode-borrowck`: future ownership checker
- `kode-ir`: future compiler IR
- `kode-codegen`: future native backend
- `kode-runtime`: future minimal runtime
- `kode-stdlib`: future standard library

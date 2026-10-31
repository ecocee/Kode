# Kode

**A compiled language that is easy to write and cheap to run.**
Python-readable, JavaScript-familiar, compiled to small native binaries,
built for cloud backends and for AI agents to write.

```
fn fib(n: int) -> int {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

name = "Asha"
print("${name}: fib(30) =", fib(30))
```

Kode is written in Rust. The compiler turns source into a native program:

```
source -> lexer -> parser -> AST -> type check -> MIR -> Cranelift -> object file -> linker -> binary
```

## Status: v0.1 core (working)

Works today: `int`, `float`, `bool`, `string`; variables, constants, typed declarations;
arithmetic, comparison and logic operators; `if / else if / else`, `while`, `for` over ranges,
`break`, `continue`; functions with recursion; reference-counted strings with `+`, `${...}`
interpolation, `len`, `string()`, `int()`, `float()`; `print`, `assert`; errors with codes and
JSON output. Planned next: collections, classes, errors and optionals, modules, concurrency.
See [docs/ROADMAP.md](docs/ROADMAP.md).

Measured on one dev machine (recursive Fibonacci of 35):
Kode 0.07 s, Python 1.76 s, release binary about 320 KB.
Treat this as a first data point, not a claim. See `benchmarks/`.

## Build and run

Needs Rust 1.91 or newer and a C compiler (`cc`). On a Mac: `xcode-select --install`.

```
cargo build --workspace
cargo test --workspace
./target/debug/kode run examples/hello.kode
```

To type just `kode` in this terminal:

```
export PATH="$PWD/target/debug:$PATH"
kode run examples/tour.kode
```

## Commands

| Command | What it does |
|---|---|
| `kode run file.kode` | compile and run |
| `kode build file.kode [-o out] [--release]` | make a native binary |
| `kode check file.kode` | errors only, no code generation |
| `kode tokens file.kode` | print tokens (debugging) |
| `kode ast file.kode` | print the syntax tree (debugging) |
| `kode mir file.kode` | print the MIR, including retain/release (debugging) |
| `--format=json` | diagnostics as JSON, for editors and AI agents |
| `--strict` | warnings count as errors |

## Repository map

```
crates/
  kode-diagnostics/   spans and error messages (human + JSON)
  kode-syntax/        tokens, lexer, AST, parser
  kode-sema/          names and types
  kode-mir/           mid-level IR, lowering, dump, verifier
  kode-codegen/       MIR -> Cranelift -> object file -> linker
  kode-rt/            runtime library linked into every program
  kode-cli/           the `kode` command
tests/run-pass/       programs that must run (.kode + .out), checked for leaks
tests/compile-fail/   programs that must fail (.kode + .err with the error code)
examples/             small programs
benchmarks/           proof of the cost claims
docs/                 spec, design notes, decisions (ADRs), roadmap, error codes
```

## License

MIT

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

## Status: v0.3 (working)

Works today: `int`, `float`, `bool`, `string`; variables and constants; operators; `if`, `while`,
`for` over ranges, lists and maps, `break`, `continue`, `match`; functions with recursion and named
arguments; **lists** `[T]` and **maps** `{string: T}`; **classes** with methods and defaults;
**enums**; **optionals** `T?` with `?.`, `??`, `!`; **errors** with `throw`, `try / catch` and
`expr catch e { ... }`; **imports** and **packages** (`kode.toml` source roots, `import ... as`
qualified access, `pub`-gated module exports); `test` blocks; reference-counted memory with a leak
checker; errors with codes, hints and JSON output; **interfaces** with `implements` and run-time
dispatch; **bit operators** `& | ^ ~ << >>` with compound assignment; and **standard-library
namespaces** like `json.parse`, `math.sqrt`, `http.get` and `pg.query`.

Tools: `kode run | build | check | test | fmt | explain | mir | lsp | doctor`, and packages:
`kode init | new | add | remove | install` with path dependencies and a checked-in `kode.lock`.

**Tasks** (`spawn` / `await`), **channels** (`Channel<T>`, `send`/`recv`/`close`), a **web server**
(`serve`, or a **Router** with `Request`/`Response`), **JSON**, an **HTTP client**, strings, files,
arguments and sorting are in too. Function values and closures (`fn(int) -> int`, `|x: int| x * 2`),
generic functions, path dependencies and printing lists and maps (`[1, 2]`, `{"name": "Asha"}`)
work as well.

There is a **PostgreSQL client** in the runtime (`pg.connect`, `pg.exec`, `pg.query`,
`pg.close`) with SCRAM-SHA-256 and JSON results; it speaks plain TCP, so TLS is still missing
(ADR 0033, example: [examples/db/postgres.kode](examples/db/postgres.kode)).

Next: the M9 production track — TLS, cycle-safe memory, byte buffers, enum payloads; see
[docs/PLAN-v1.md](docs/PLAN-v1.md). M8 (budgets, `uses`, typed `ask`/`tool`) is
skipped. Measurements and known gaps: [docs/STATUS.md](docs/STATUS.md).
See [docs/ROADMAP.md](docs/ROADMAP.md) and the one-page [cheat sheet](docs/cheatsheet.md).

Measured on one dev machine (macOS arm64, Apple silicon): `fib(35)` in 93 ms against Python's
4,658 ms and C's 58 ms; `examples/hello.kode` builds to a 450,696-byte binary, 373,336 bytes after
`strip -x`. Full numbers, targets and the rows that miss: `benchmarks/report.md`.

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

On Windows, create the compiler with the GNU Rust toolchain and MinGW-w64
(see `docs/windows.md`). Windows is not a release target yet: the release
workflow builds Linux x86-64 and macOS arm64 only.

## Commands

| Command | What it does |
|---|---|
| `kode run file.kode` | compile and run |
| `kode build file.kode [-o out] [--release]` | make a native binary |
| `kode check file.kode` | errors only, no code generation |
| `kode test [file or folder]` | run `test` blocks |
| `kode fmt <files or folders> [--check]` | format source files |
| `kode explain E0201` | explain an error code |
| `kode init [name]` | create a package here (`kode.toml` + `src/`) |
| `kode new <name>` | create a package in a new folder |
| `kode add <name> --path <dir>` | add a path dependency |
| `kode remove <name>` | remove a dependency |
| `kode install [--check]` | resolve dependencies and write `kode.lock` |
| `kode doctor` | print version, target, linker, runtime ABI, package state |
| `kode doc <folder> [-o out]` | HTML reference for `pub` items |
| `kode tokens file.kode` | print tokens (debugging) |
| `kode ast file.kode` | print the syntax tree (debugging) |
| `kode mir file.kode` | print the MIR, including retain/release (debugging) |
| `kode lsp` | language server on stdio: diagnostics, hover, completion, symbols, format |
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
examples/             small programs, one idea each (see examples/README.md)
benchmarks/           proof of the cost claims
docs/                 spec, cheat sheet, design notes, decisions (ADRs), roadmap, error codes
.github/workflows/    CI (ci.yml) and release (release.yml)
```

Start with [`docs/cheatsheet.md`](docs/cheatsheet.md) for the shape of the language and
[`docs/spec/`](docs/spec/) for the rules.

## License

MIT

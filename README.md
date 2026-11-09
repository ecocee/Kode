# Kode

A compiled language that is easy to write and cheap to run.

Python-readable, JavaScript-familiar, compiled to small native binaries. Built for cloud
backends and for AI agents to write.

```kode
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

```text
source -> lexer -> parser -> AST -> type check -> MIR -> LLVM IR -> clang -> object file -> linker -> binary
```

## Status

**v0.3 — working compiler, active standard library.**

Core language: `int`, `float`, `bool`, `string`; variables and constants; operators; `if`,
`while`, `for` over ranges, lists and maps, `break`, `continue`, `match`; functions with recursion
and named arguments.

Data: **lists** `[T]`, **maps** `{string: T}`, **classes** with methods and defaults, **enums**,
**optionals** `T?` with `?.`, `??`, `!`, and **errors** with `throw`, `try / catch` and
`expr catch e { ... }`.

Modules: **imports** and **packages** (`kode.toml` source roots, `import ... as` qualified access,
`pub`-gated module exports). Testing: `test` blocks.

Memory and runtime: reference-counted memory with a leak checker; errors with codes, hints and
JSON output.

Advanced features: **interfaces** with `implements` and run-time dispatch; **bit operators**
`& | ^ ~ << >>` with compound assignment; **function values and closures**
(`fn(int) -> int`, `|x: int| x * 2`); **generic functions**; path dependencies; printing lists and
maps (`[1, 2]`, `{"name": "Asha"}`).

Concurrency and I/O: **tasks** (`spawn` / `await`), **channels** (`Channel<T>`,
`send`/`recv`/`close`), a **web server** (`serve`/`serve_tls`, or a **Router** with
`Request`/`Response`), **JSON**, an **HTTP client** (`http://` and `https://`), strings, files,
command-line arguments and sorting.

Standard-library namespaces include `json.parse`, `math.sqrt`, `http.get` and `pg.query`.

## PostgreSQL client

There is a **PostgreSQL client** in the runtime (`pg.connect`, `pg.exec`, `pg.query`, `pg.close`)
with SCRAM-SHA-256 authentication, opportunistic TLS (`sslmode=prefer`) and JSON results; a pool
is available through `pg.pool`. [ADR 0033](docs/adr/0033-postgres-client.md) describes the
protocol work. A worked example lives at [examples/db/postgres.kode](examples/db/postgres.kode).

## Roadmap

TLS shipped for the HTTP client, the HTTP server (`serve_tls`) and the PostgreSQL client
([ADR 0034](docs/adr/0034-tls-in-the-runtime.md)). Next up is the rest of the **M9 production
track**: cycle-safe memory, byte buffers, enum payloads and generic classes.
See [docs/PLAN-v1.md](docs/PLAN-v1.md). M8 (budgets, `uses` effects, typed `ask`/`tool`) is
skipped. For current measurements and known gaps, see [docs/STATUS.md](docs/STATUS.md) and
[docs/ROADMAP.md](docs/ROADMAP.md). A one-page overview is at
[docs/cheatsheet.md](docs/cheatsheet.md).

## Performance

Measured on one development machine (macOS arm64, Apple silicon):

- `fib(35)` in 93 ms against Python's 4,658 ms and C's 58 ms.
- `examples/hello.kode` builds to a 450,696-byte binary, 373,336 bytes after `strip -x`.

Full numbers, targets and the rows that still miss are in [benchmarks/report.md](benchmarks/report.md).

## Build and run

Needs Rust 1.91 or newer and a C compiler (`cc`). On macOS, install the command-line tools with
`xcode-select --install`.

```bash
cargo build --workspace
cargo test --workspace
./target/debug/kode run examples/hello.kode
```

To type just `kode` in this terminal:

```bash
export PATH="$PWD/target/debug:$PATH"
kode run examples/tour.kode
```

macOS arm64, Linux arm64 and Linux x86-64 are validated; Windows is a preview
target. Requirements per platform, the required clang version, CI runners and
known gaps are in `docs/platforms.md` (Windows setup: `docs/windows.md`).

## Commands

| Command | What it does |
| --- | --- |
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
| `kode doctor` | print version, target, linker, runtime ABI, package state; fails if `kode.toml` needs a newer compiler |
| `kode doc <folder> [-o out]` | HTML reference for `pub` items |
| `kode tokens file.kode` | print tokens (debugging) |
| `kode ast file.kode` | print the syntax tree (debugging) |
| `kode mir file.kode` | print the MIR, including retain/release (debugging) |
| `kode lsp` | language server on stdio: diagnostics, hover, completion, symbols, format |
| `--format=json` | diagnostics as JSON, for editors and AI agents |
| `--strict` | warnings count as errors |
| `--backend=llvm` | accepted for compatibility; LLVM is the only backend |

LLVM is the only backend (ADR 0035): it consumes MIR, writes textual LLVM IR
and asks `clang` (15 or newer, or `KODE_CLANG`) for the object file, then links
with the runtime archive. Without clang the build stops with a message naming the
missing program; there is no fallback of any kind. `--backend=llvm` is accepted
as a no-op for scripts written while the flag was being introduced, and
`--backend=cranelift` reports that the Cranelift backend was removed.

## Repository map

```text
crates/
  kode-diagnostics/   spans and error messages (human + JSON)
  kode-syntax/        tokens, lexer, AST, parser
  kode-sema/          names and types
  kode-mir/           mid-level IR, lowering, dump, verifier
  kode-link/          object linking: linker driver and runtime archive
  kode-rt/            runtime library linked into every program
  kode-cli/           the `kode` command
tests/run-pass/       programs that must run (.kode + .out), checked for leaks
tests/compile-fail/   programs that must fail (.kode + .err with the error code)
examples/             small programs, one idea each (see examples/README.md)
benchmarks/           proof of the cost claims
docs/                 spec, cheat sheet, design notes, decisions (ADRs), roadmap, error codes
.github/workflows/    CI (ci.yml) and release (release.yml)
```

Start with [docs/cheatsheet.md](docs/cheatsheet.md) for the shape of the language and
[docs/spec/](docs/spec/) for the rules.

## License

Apache-2.0

<div align="center">

# Kode

### **Write. Compile. Ship.**

A modern, statically typed programming language combining the readability of Python with the performance, safety, and single-binary deployment of native machine code.

[![CI Status](https://github.com/ecocee/Kode/actions/workflows/ci.yml/badge.svg)](https://github.com/ecocee/Kode/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/ecocee/Kode?color=7c3aed&label=release)](https://github.com/ecocee/Kode/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows%20(Preview)-0ea5e9)](docs/platforms.md)
[![Backend](https://img.shields.io/badge/backend-LLVM-6366f1)](docs/adr/0035-llvm-backend.md)

<p align="center">
  <a href="#why-kode"><strong>Why Kode?</strong></a> •
  <a href="#a-language-built-for-shipping"><strong>Capabilities</strong></a> •
  <a href="#see-kode-in-action"><strong>Tour</strong></a> •
  <a href="#quick-start"><strong>Quick Start</strong></a> •
  <a href="#features-and-status"><strong>Features</strong></a> •
  <a href="#how-kode-works"><strong>Architecture</strong></a> •
  <a href="#the-self-hosting-journey"><strong>Self-Hosting</strong></a> •
  <a href="#verified-benchmarks"><strong>Performance</strong></a> •
  <a href="#contributing"><strong>Contributing</strong></a>
</p>

</div>

---

## Why Kode?

Developers building modern microservices, CLI tools, and automated agent systems constantly face an unwanted trade-off:

* **Scripting languages** (Python, JavaScript, Ruby) offer great ergonomics and fast iteration, but require multi-hundred-megabyte Docker images, introduce JIT warm-up latency, and consume significant memory per process.
* **Low-level systems languages** (C, C++, Rust) generate fast, compact binaries, but impose cognitive friction—manual memory tracking, borrow checker intricacies, complex syntax, and slow compilation cycles.

**Kode bridges this gap.**

Kode provides clean, expressive syntax that reads naturally—with top-level entry points, string interpolation, and pattern matching—while compiling ahead-of-time (AOT) to lean, standalone native executables.

* **No Virtual Machine**: Compiles directly to native machine code via LLVM and Clang.
* **No Stop-the-World GC**: Memory is reference counted deterministically, with cycle detection work in flight and built-in leak checking (`KODE_LEAK_CHECK=1`).
* **Sub-Megabyte Native Footprint**: A stripped Hello World executable is **~302 KB**—not 40 MB.
* **Engineered for AI & Humans**: Deterministic compiler diagnostics with stable error codes (`E0xxx`), human-readable hints, and first-class `--format=json` output designed for editor tooling and AI agent workflows.

---

## A Language Built for Shipping

<table>
  <tr>
    <td width="50%">
      <h3>Readable & Expressive Syntax</h3>
      <p>Clean syntax inspired by modern languages. Features top-level execution, type inference, first-class functions, closures, interfaces, and expressive pattern matching.</p>
    </td>
    <td width="50%">
      <h3>AOT Native Compilation</h3>
      <p>Compiles down to optimized machine code via LLVM IR and Clang. Output binaries are standalone single-file executables with zero external runtime dependencies.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Deterministic Memory Model</h3>
      <p>Automated reference counting manages memory with zero garbage collection pause spikes. Continuous integration verifies that test suites finish with zero memory leaks.</p>
    </td>
    <td width="50%">
      <h3>Batteries-Included Standard Library</h3>
      <p>Built-in HTTP/1.1 client and server with TLS, dynamic URL routing, JSON serialization, channels, tasks (<code>spawn</code>/<code>await</code>), and a native PostgreSQL client with SCRAM-SHA-256 authentication and connection pooling.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>First-Class Developer Experience</h3>
      <p>Everything you need is built into a single tool: <code>kode run</code>, <code>kode build</code>, <code>kode test</code>, <code>kode fmt</code>, <code>kode doctor</code>, and an integrated language server (<code>kode lsp</code>).</p>
    </td>
    <td width="50%">
      <h3>Agent-Ready Architecture</h3>
      <p>Structured <code>--format=json</code> compiler diagnostics, stable error codes with hints (<code>kode explain E0201</code>), and clean semantics make Kode an ideal target for AI code generation.</p>
    </td>
  </tr>
</table>

---

## See Kode in Action

### Concise, Readable Code

Programs execute from top-level statements with full static type checking:

```kode
fn fib(n: int) -> int {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

name = "Developer"
print("Welcome to Kode, ${name}!")

for i in 1..=5 {
    print("  fib(${i}) = ${fib(i)}")
}
```

### Build and Run

Run directly or compile into an optimized standalone binary:

```bash
# Compile and execute in memory
kode run app.kode

# Or build an optimized native executable
kode build app.kode -o app --release
./app
```

```text
Welcome to Kode, Developer!
  fib(1) = 1
  fib(2) = 1
  fib(3) = 2
  fib(4) = 3
  fib(5) = 5
```

<details>
<summary><strong>View Concurrent Pipeline Example (Tasks & Channels)</strong></summary>

```kode
// Fan work out to worker tasks over a channel and collect results
fn worker(jobs: Channel<int>, out: Channel<string>, count: int) {
    for i in 0..count {
        n = recv(jobs)
        send(out, "job ${n}: square is ${n * n}")
    }
}

jobs: Channel<int> = channel()
out: Channel<string> = channel()

spawn worker(jobs, out, 3)

for n in 1..=3 {
    send(jobs, n)
}

for i in 0..3 {
    print(recv(out))
}
```

</details>

<details>
<summary><strong>View Microservice Example (HTTP Router & JSON)</strong></summary>

```kode
// Lightweight JSON REST service
fn get_user(req: Request) -> Response {
    id = req.params["id"].to_int()
    user = json_obj()
    user["id"] = json_int(id)
    user["name"] = json_string("Asha")
    return Response.json(json_map(user))
}

router = Router.new()
router.get("/users/:id", get_user)

print("Server listening on port 8080...")
serve(8080, router)
```

</details>

---

## Quick Start

### Prerequisites

* **Rust** 1.91 or newer (stable toolchain)
* **Clang** 15 or newer (used by the compiler backend for code generation)
* **C compiler / linker driver** (`cc` on macOS, `gcc` on Linux, MinGW-w64 on Windows)

See [docs/platforms.md](docs/platforms.md) for detailed platform requirements and [docs/windows.md](docs/windows.md) for Windows setup.

### Option 1: Download Pre-Built Release

Download verified release archives and Debian packages from the [GitHub Releases](https://github.com/ecocee/Kode/releases) page (`v0.3.6`):

* **Linux x86-64**: `kode-v0.3.6-linux-x86_64.tar.gz` or `.deb`
* **macOS arm64**: `kode-v0.3.6-macos-arm64.tar.gz`
* **Windows x86-64**: `kode-v0.3.6-windows-x86_64.tar.gz` (Preview)

Extract and place the `kode` binary in your system `PATH`.

### Option 2: Build From Source

```bash
# 1. Clone the repository
git clone https://github.com/ecocee/Kode.git
cd Kode

# 2. Build the compiler and runtime
cargo build --release -p kode-cli

# 3. Add to your PATH
export PATH="$PWD/target/release:$PATH"

# 4. Verify your environment
kode doctor
```

Output of `kode doctor`:
```text
kode 0.3.6 — toolchain check
ok  compiler  kode 0.3.6
ok    target  macos-aarch64
ok   runtime  12535 KB embedded, ABI 0006
ok    linker  cc: Apple clang version 21.0.0
ok   package  no kode.toml here (fine for a single file)
```

Run your first program:

```bash
kode run examples/hello.kode
```

---

## Features and Status

Kode is currently at **v0.3.6**. The compiler produces production-grade standalone binaries for CLI tools and microservices on Linux and macOS.

| Domain | Feature | Status | Notes |
|---|---|:---:|---|
| **Core Types** | `int`, `float`, `bool`, `string` | **Stable** | 64-bit precision; arithmetic, bitwise, and comparison operators |
| **Control Flow** | `if` / `else`, `while`, `for` ranges, `match` | **Stable** | Includes `break`, `continue`, and expressions |
| **Collections** | Lists `[T]`, Maps `{string: T}` | **Stable** | Dynamic resizing, `.len()`, `.add()`, `.keys()`, `sort()`, indexing |
| **Object Model** | `class`, fields, methods, constructors | **Stable** | Value defaults, named constructors, methods |
| **Interfaces** | `interface`, `implements`, dynamic dispatch | **Stable** | Signature verification and runtime interface tables |
| **Generics** | Generic functions (`fn id<T>(x: T) -> T`) | **Stable** | Monomorphized at MIR lowering (ADR 0030) |
| **Generics** | Generic classes & generic methods | **In Progress** | Working in selfhost work packets (WP-01, WP-02; ADR 0040) |
| **Enums** | Simple enums & match branches | **Stable** | Named variants |
| **Enums** | Enum payloads & pattern matching | **In Progress** | Implemented and met in selfhost work packets (WP-03, WP-04; ADR 0039) |
| **Optionals** | `T?`, `?.`, `??`, `!` | **Stable** | Safe unwrapping and navigation for reference types |
| **Errors** | `throw`, `try` / `catch`, `catch e { ... }` | **Stable** | Structured errors with numeric codes and messages |
| **Concurrency** | Tasks (`spawn` / `await`), `Channel<T>` | **Stable** | Atomic reference counting, message-passing concurrency |
| **Networking** | HTTP/1.1 client & server (`serve`), Router | **Stable** | Includes TLS (`rustls`), `Request`, `Response`, JSON payloads |
| **Databases** | PostgreSQL client & connection pool | **Stable** | SCRAM-SHA-256, opportunistic TLS, JSON row output (ADR 0033) |
| **Memory** | Reference counting & leak checker | **Stable** | `KODE_LEAK_CHECK=1` validates zero residual allocations |
| **Memory** | Cycle collection (`weak` references) | **Planned** | Tracked in ADR 0038 |
| **Package System** | `kode.toml`, `kode.lock`, `kode add`, `import` | **Stable** | Path dependencies, package root resolution, and locking |
| **Developer Tools** | `kode fmt`, `kode test`, `kode explain`, LSP | **Stable** | Built-in stdio LSP, automated formatting, doc generator |

*Non-Goals for v1.0*: JIT compilation, shared `.so` runtime dependencies, external VM installations, and effect systems (M8).

---

## How Kode Works

Kode uses a multi-stage, clean-room compilation pipeline designed for low binary overhead and high compiler velocity:

```text
  Source Code (.kode)
         │
         ▼
  ┌───────────────┐
  │ Lexer & Parser│ ──> AST (Abstract Syntax Tree)
  └───────────────┘
         │
         ▼
  ┌───────────────┐
  │ Type Checker  │ ──> Symbol resolution, type inference, interface validation
  └───────────────┘
         │
         ▼
  ┌───────────────┐
  │  Mid-level IR │ ──> Ownership lowering, retain/release tracking, MIR optimizations
  │     (MIR)     │
  └───────────────┘
         │
         ▼
  ┌───────────────┐
  │  LLVM Lowering│ ──> Generates clean, textual LLVM IR (ADR 0035)
  └───────────────┘
         │
         ▼
  ┌───────────────┐
  │ Host Clang &  │ ──> Assembles machine code & links minimal runtime archive
  │ Linker Driver │
  └───────────────┘
         │
         ▼
 Standalone Native Executable
```

### Architectural Principles

1. **Only MIR Reaches the Backend**: The AST is completely decoupled from code generation. Mid-level IR (MIR) explicitly models memory management (retain/release instructions), control flow graphs, and data layouts.
2. **Lean Compiler Binary (No Embedded LLVM Libraries)**: Instead of linking gigabytes of LLVM C++ libraries into the `kode` binary, Kode generates textual LLVM IR and coordinates with the host's existing `clang`. This keeps the `kode` compiler binary lean (~12 MB).
3. **Modular Runtime Archive**: The runtime library (`crates/kode-rt`) is split. High-overhead subsystems (such as TLS cryptography and the PostgreSQL protocol engine) are only linked when referenced by your source code.

---

## The Self-Hosting Journey

Kode is executing a disciplined, multi-stage bootstrap roadmap to become a self-hosting programming language:

```
  Stage 0 (Oracle)            Stage 1                     Stage 2
┌──────────────────┐    ┌──────────────────┐        ┌──────────────────┐
│  Rust Compiler   │ ─> │  Kode Compiler   │ ─────> │  Bootstrapped    │
│ (crates/ on main)│    │(selfhost/ branch)│        │  Kode Compiler   │
└──────────────────┘    └──────────────────┘        └──────────────────┘
```

* **Stage 0 (The Oracle)**: The current official compiler on `main` is written in Rust. It serves as the trusted, permanent reference implementation.
* **Stage 1 (Under Active Development)**: A clean-room compiler written entirely in Kode, developed in `selfhost/` on the [`next/selfhost`](https://github.com/ecocee/Kode/tree/next/selfhost) branch.
* **Stage 2 (Full Bootstrap)**: Stage 1 compiling itself, producing identical diagnostics, AST dumps, and binaries byte-for-byte.

### Verified Progress Today

Development progress is enforced through automated readiness gates (`./scripts/selfhost.sh readiness`):

* **R0 (Oracle Validation)**: Verified.
* **R1 (Lexer Parity)**: **100% byte-identical** across 283 token dumps.
* **S2 (Parser Parity)**: **100% byte-identical** across 173/173 test cases and all 55 self-host source files.
* **S3 (Checker Parity)**: **100% byte-identical** diagnostic output across 170 test cases.
* **Work Packets**: Generic classes (WP-01), generic methods (WP-02), enum payloads (WP-03/04), optional scalars (WP-06), and byte buffers (WP-08) have met their formal test targets.

We transparently track the self-hosting journey in [docs/PLAN-self-hosting.md](docs/PLAN-self-hosting.md).

---

## Verified Benchmarks

*All measurements recorded on Darwin arm64 (Apple silicon, 10 cores) with the release compiler and Apple clang 21.0.0. Full methodology, configurations, and reproduction instructions are documented in [benchmarks/report.md](benchmarks/report.md).*

### Standalone Executable Size

Binary sizes after release optimization and symbol stripping (`strip -x`):

| Program | Stripped Binary Size | Notes |
|---|---:|---|
| **`hello.kode`** | **302 KB** | Standard output and core runtime |
| **`tasks.kode`** | **420 KB** | Full CLI application with JSON disk storage |
| **`pipeline.kode`** | **320 KB** | Multi-threaded task spawning and channels |
| **Plaintext HTTP Server (`serve`)** | **454 KB** | Event loop, request parsing, HTTP router |
| **TLS HTTPS Server (`serve_tls`)** | **2.78 MB** | Includes `rustls`, TLS handshakes, and crypto |

### Wall Clock Execution Time (Median)

Benchmarked using `hyperfine` (warm cache, 5–20 runs). Binaries built at `kode build --release` vs `cc -O2` and `go build`:

| Benchmark | Kode | C (`-O2`) | Go | Node.js | Python | Kode vs C |
|---|---:|---:|---:|---:|---:|:---:|
| `loop_mod` | **11 ms** | 11 ms | 29 ms | 49 ms | 3,026 ms | **1.01×** |
| `mandel` | **13 ms** | 11 ms | 13 ms | 30 ms | 1,155 ms | **1.16×** |
| `collatz` | **200 ms** | 150 ms | 228 ms | 1,799 ms | 14,166 ms | **1.33×** |
| `sieve` | **30 ms** | 12 ms | 21 ms | 151 ms | 2,231 ms | **2.58×** |

> **Note on recursion (`fib(38)`)**: In default release builds, Kode runs `fib(38)` in ~4 ms vs C's 57 ms because Kode's MIR inliner inlines recursive calls once prior to Clang code generation. In like-for-like unoptimized comparisons (`kode --opt=none` vs `cc -O0`), execution is **268 ms** (Kode) vs **106 ms** (C). See [benchmarks/report.md](benchmarks/report.md) for full analysis.

### Memory Footprint (Peak RSS)

Peak Resident Set Size (RSS) measured via system high-water mark:

| Benchmark | Kode | C | Go | Node.js | Python |
|---|---:|---:|---:|---:|---:|
| `loop_mod` | **1.76 MB** | 1.69 MB | 4.34 MB | 48.57 MB | 9.36 MB |
| `mandel` | **1.76 MB** | 1.68 MB | 4.32 MB | 49.76 MB | 9.44 MB |
| `collatz` | **1.76 MB** | 1.69 MB | 4.36 MB | 48.48 MB | 9.39 MB |

---

## Command Reference

The `kode` CLI provides an integrated toolchain for building, formatting, and analyzing programs:

| Command | Description |
|---|---|
| `kode run <file.kode> [-- args]` | Compile and execute program immediately in memory |
| `kode build <file.kode> [-o out] [--release]` | Compile program to a standalone native binary |
| `kode check <file.kode> [--format=json]` | Fast syntax and type verification without code generation |
| `kode test [path]` | Execute built-in `test` blocks across files or directories |
| `kode fmt <paths...> [--check]` | Format source code cleanly according to language conventions |
| `kode explain <CODE>` | Display detailed diagnostic explanations (e.g. `kode explain E0201`) |
| `kode doctor` | Diagnose environment, compiler, target, linker, and runtime ABI |
| `kode lsp` | Run the built-in Language Server Protocol over stdio for editor integration |
| `kode doc <folder> [-o dir]` | Generate HTML API documentation for all `pub` items |
| `kode init [name]` | Initialize a new package with `kode.toml` in the current directory |
| `kode new <name>` | Create a new package in a new subfolder |
| `kode add <name> --path <dir>` | Add a path-based dependency to `kode.toml` |
| `kode remove <name>` | Remove a dependency from `kode.toml` |
| `kode install [--check]` | Resolve dependencies and generate `kode.lock` |
| `kode mir <file.kode>` | Inspect Mid-Level IR, retain/release tracking, and lowering (debug) |
| `kode ast <file.kode>` | Inspect syntax tree representation (debug) |
| `kode tokens <file.kode>` | Inspect lexer token output (debug) |

### Compiler Flags

* `--release`: Optimizes code generation and strips symbols from the output binary.
* `--debug`: Disables MIR optimization passes for faster compile times (backend Clang remains `-O2`).
* `--opt=none`: Disables optimization in both MIR and Clang (used for baseline performance measurements).
* `--format=json`: Emits compiler errors, warnings, and hints as structured JSON for IDEs and AI tools.
* `--strict`: Treats compiler warnings as fatal errors.

---

## Documentation

* [Language Cheatsheet](docs/cheatsheet.md) — One-page quick reference for syntax, collections, and standard functions.
* [Language Specification](docs/spec/) — Complete 13-chapter formal specification.
* [Platform Support & Requirements](docs/platforms.md) — Detailed targets, Clang dependencies, and CI runners.
* [Windows Setup Guide](docs/windows.md) — WinLibs POSIX UCRT and MinGW-w64 instructions.
* [Roadmap & Plan to v1.0](docs/PLAN-v1.md) — Architectural goals, milestones, and design constraints.
* [Architectural Decision Records (ADRs)](docs/adr/) — Technical decisions, alternatives, and rationale (35+ records).
* [Compiler Error Index](docs/errors/index.md) — Complete catalogue of compiler diagnostic codes and hints.
* [Example Gallery](examples/README.md) — Real-world programs covering APIs, databases, pipelines, and CLI tools.

---

## Contributing

Kode is developed by a focused team and welcomes thoughtful open-source contributions.

Because Kode is a compiler, **tests come first**:
* New language features must include run-pass tests (`tests/run-pass/name.kode` and `name.out`).
* Invalid syntax or type errors must include compile-fail tests (`tests/compile-fail/name.kode` and `name.err`).
* Memory leak checks run on every test suite invocation via `KODE_LEAK_CHECK=1`.

### Pre-Commit Checklist

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./target/release/kode fmt examples tests/run-pass --check
```

### Branches & Pull Requests

* **`main`**: Targets the official Rust-based compiler, runtime, CLI, and release documentation.
* **`next/selfhost`**: Targets the self-hosted compiler implementation in `selfhost/`.
* **Developer Certificate of Origin (DCO)**: Every commit must carry a sign-off line (`git commit -s`).

Review [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for full development guidelines.

---

## Roadmap

| Milestone | Scope | Status |
|---|---|:---:|
| **M1 Core** | Primitives, control flow, functions, recursion, MIR, LLVM backend, JSON diagnostics | **Done** |
| **M2 Collections** | Dynamic lists `[T]`, maps `{string: T}`, iteration, sorting, filtering | **Done** |
| **M3 Classes** | Classes, fields, methods, constructors, basic enums, pattern matching | **Done** |
| **M4 Errors & Optionals** | Optionals `T?`, `?.`, `??`, `!`, structured errors with `try` / `catch` / `throw` | **Done** |
| **M5 Tooling** | Packages (`kode.toml`), modules, `test` blocks, `kode fmt`, `kode explain`, LSP | **Done** |
| **M6 Concurrency** | Tasks (`spawn` / `await`), `Channel<T>`, atomic reference counting | **Done** |
| **M7 Services** | HTTP/1.1 client & server (`serve`), JSON API Router, PostgreSQL client (SCRAM, TLS, pool) | **Done** |
| **M7.5 Ergonomics** | Bitwise operators, compound assignment, standard library namespaces | **Done** |
| **M7.6 Interfaces** | `interface` declarations, `implements`, dynamic dispatch table | **Done** |
| **M7.7 Packages** | Package-relative imports, `import ... as`, `pub`-gated exports, `kode.lock` | **Done** |
| ~~**M8 Budgets & AI**~~ | Memory budgets, declared effects (`uses`), typed AI tools | *Skipped (off path to v1.0)* |
| **M9 Production Core** | Enum payloads, generic classes, byte buffers, `StringBuilder`, cycle breaking (`weak`), Windows verification | **In Progress** |

For granular milestone planning, see [docs/PLAN-v1.md](docs/PLAN-v1.md) and [docs/ROADMAP.md](docs/ROADMAP.md).

---

## Community & License

Kode is licensed under the **[Apache License 2.0](LICENSE)**.
Copyright © 2025–2026 Ecocee.

* **Trademarks**: The names "Kode", "Ecocee", and project logos are trademarks of the project owner. See [TRADEMARKS.md](TRADEMARKS.md) for usage policies.
* **Code of Conduct**: We are committed to providing a friendly and welcoming environment. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
* **Security Reports**: Security vulnerabilities should be reported privately according to [SECURITY.md](SECURITY.md).
* **Shipping Programs**: Binaries compiled with `kode build` link the Kode runtime archive, which incorporates selected open-source dependencies. When redistributing compiled binaries, include [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) alongside your release. Your own Kode source code remains entirely yours.

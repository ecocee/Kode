# Kode v1.0 plan

One plan for everything between v0.3 and a language that can be released, used all day,
and compared honestly with C, Go, Node and Python.

The roadmap in [ROADMAP.md](ROADMAP.md) lists milestones. This file is the working plan:
what ships, in which branch, and how we know it is done. Every phase is one branch and
one commit set, so `git log` reads like the plan.

## 1. Where we are

| Area | Today | Gap for v1.0 |
|---|---|---|
| Language | int/float/bool/string, lists, maps, classes, enums, interfaces, `match`, `try`/`catch`/`finally`, `T?` (locals for `int?`), channels, packages, modules, closures, generic functions | cycle-safe memory (`weak`), enum payloads, fixed-width ints, generic classes |
| Standard library | strings, lists, maps, files, args, env, sort, JSON, HTTP client, server, router, math, time | collection helpers (`contains`, `index_of`, `reverse`, `insert`, …), `map.values()`, string ergonomics |
| Errors | stable codes, hints, JSON output, `kode explain`, `finally` | suggestions everywhere, better runtime messages, a depth-limit diagnostic |
| Tools | run/build/check/test/fmt/explain/mir/lsp/doc/doctor, JSON diagnostics | language server, `kode doc`, `kode doctor`, editor integration |
| Packages | `kode.toml` with one source root, `import ... as`, `pub` | dependencies (path first), `kode.lock`, `kode add/install`, reuse across projects |
| Performance | LLVM (the only backend since B8), inlining, if-conversion, RC strings/objects | measured numbers per release, RSS budget, `--release` tuning |
| Concurrency | `spawn` / `await`, `Task<T>`, atomic RC | channels, task cancellation/timeout |
| Portability | macOS, Linux (x86-64/arm64) | Windows (MinGW-w64 first), documented cross-compilation, per-platform installers |
| Docs | spec chapters, ADRs, cheat sheet, editors guide | generated API docs, docs site build, editor setup |

## 2. Binary model: one compiler, one binary per program

There are two well-known shapes, and Kode deliberately picks the first.

**Model A — Go / Zig (chosen).**
`kode` is a single self-contained compiler binary with the runtime archive embedded.
Each `kode build` produces a standalone native executable that contains only the runtime
functions it uses. Nothing else is needed at run time: no VM, no shared runtime installed
on the machine, no interpreter, no GC heap.

**Model B — .NET / Python.**
One shared runtime (VM or interpreter) is installed once; programs are small payloads
that need it. That gives shared updates and a small per-program artifact, but every
program pays for the runtime in start-up time, memory, and deployment complexity.

Why Model A:

- **Memory and start-up.** A compiled Kode program has no VM, no JIT, no GC and no
  interpreter warm-up. Measured RSS is what the program allocates, not what the runtime
  reserves. This is the whole performance premise of the language.
- **Deployment.** `scp` one file. Works in a scratch container, a cron job, a lambda-like
  sandbox.
- **Agents and CI.** One command, no environment to provision, no version skew between
  compiler and runtime.

Accepted costs, stated plainly:

- Each program carries its own copy of the runtime it uses (currently ~330 KB stripped;
  budget: under 1 MB for a typical server).
- Fixing a runtime bug means rebuilding programs; there is no shared-patch channel.
- Cross-compilation needs a target sysroot and linker, exactly like C.

Non-goals for v1.0: a JIT, a shared `.so` runtime, a package-manager-installed VM.
A shared runtime mode can be revisited for fleets of servers later, and would be an ADR.

## 3. Performance and memory budget

Targets, measured on one 1 vCPU / 1 GB Linux VM, against the existing
`benchmarks/` programs:

| Program | Metric | Target | Reference |
|---|---|---|---|
| `fib(35)` | wall clock | within 2× of C `-O2` | C, Go, Node, Python |
| `loop_mod` | wall clock | within 2× of C `-O2` | as above |
| `sieve` | wall clock | within 2× of C `-O2` | as above |
| `mandel` | wall clock | within 3× of C `-O2` | floats, `f64` only |
| idle HTTP server | RSS | under 3 MB above the binary | Node/Python baseline |
| echo server | RSS at 1000 requests | flat, no growth | leak checker |
| hello world | binary size | under 1 MB stripped | — |
| compile time | `kode check` | under 100 ms on 5k lines | — |

Rules: no performance claim without a measurement in `benchmarks/report.md`, and no
optimization merged without before/after numbers. Memory discipline stays the current
reference-counting model (see `docs/design/memory-model.md`); allocations are counted
by `KODE_LEAK_CHECK=1`.

## 4. Phases

Each phase is a branch off the previous one. Acceptance = build green, conformance green,
plus the listed checks.

Status at the end of the first implementation session:

| Phase | Branch | State |
|---|---|---|
| P0 plan | `plan/v1.0` | done |
| P1 language server | `feat/lsp` | done |
| P2 packages | `feat/packages` | done |
| P3 standard library | `feat/stdlib` | done |
| P4 errors and optionals | `feat/error-model` | `finally`, runtime hints and ADR 0024 done; suggestions were already in place |
| P5 CLI | `feat/cli-ux` | done |
| P6 docs and Windows | `feat/docs-windows` | `kode doc` and the site done; Windows links and is documented, but has not been run on Windows |
| P7 channels | `feat/channels` | done: `Channel<T>`, `send`/`recv`/`close`, tasks take channels (ADR 0026) |
| P8 performance | `perf/budgets` | harness and measurements done; no optimization work yet |
| packed `[bool]` fix | `fix/bool-lists` | index writes and for-in used 8-byte accessors: wrong counts, then a crash at ten million items |

**M8 (budgets, `uses` effects, typed `ask`/`tool`) is skipped.** It was a differential
feature for AI agents, not something a general-purpose language needs to be usable.
M9 below is the production track and takes its place.

## 4b. M9 — production core (the current plan)

Ordered by what actually stops a real program from shipping. Each item is a branch with
tests and an ADR where behavior changes; the list is the order, not a schedule.

| # | Item | Why it blocks production | State |
|---|---|---|---|
| 9.1 | Printing collections (ADR 0031) | you could not look at a list or map without a loop | **done** |
| 9.2 | Docs that match the compiler | channels, `finally`, generics, LSP nav were documented as missing | **done** |
| 9.3 | TLS for the HTTP client and server | every real service talks `https://`; today it raises an error | **done** (ADR 0034): `https://` client with webpki roots, `serve_tls` with DER certificates, one shared transport; the TLS server runs one thread per connection |
| 9.4 | Database client (PostgreSQL wire protocol) | no data store means no backends | **done** (ADR 0033): SCRAM-SHA-256, simple + extended protocol, JSON results, opportunistic TLS, `pg.pool`; md5, `COPY`, binary format and cancellation remain |
| 9.5 | `weak` references | two objects pointing at each other leak forever | |
| 9.6 | Byte buffers and fixed-width ints (`u8`, `i32`, `[]byte`) | binary protocols, files and crypto need bytes, not 64-bit ints | |
| 9.7 | Enum payloads, `is`, safe downcast | tag-only enums force unsafe casts for real data shapes | |
| 9.8 | Optional scalars as fields and parameters | `int?` works only as a local (E0219) | |
| 9.9 | Generic classes, generic methods, constraints | generic containers cannot be written in Kode | |
| 9.10 | Checked arithmetic on `+ - *` | silent wraparound is a correctness bug in a language people trust | |
| 9.11 | Package registry and publishing | path-only dependencies do not scale past one machine | |
| 9.12 | Verified Windows release | the CI job exists; a real Windows run must pass before anyone downloads it | |
| 9.13 | LLVM backend, replacing Cranelift | the measured 1.6-2.4x gap against C `-O2` is code quality; Cranelift has no loop optimizer to build on | **done** (ADR 0035, B7): textual LLVM IR from MIR, `clang` builds the object, no `llvm-sys`; LLVM is the only backend since B8; the Cranelift implementation and its crates are gone |

**9.1-9.4 are done.** 9.5 (`weak` references) is the next branch, and needs an ADR
and leak evidence. TLS was the large one: rustls plus webpki-roots are now a
deliberate runtime dependency, measured in `benchmarks/http/tls.md`, and the
1 MB binary budget is **missed** (hello world is 1,077,600 bytes stripped).

### P0 — This plan (branch `plan/v1.0`)
- Single plan document, binary-model decision, budget table, phase contract.
- **Done when:** this file exists and `ROADMAP.md` points at it.

### P1 — Language server (branch `feat/lsp`)
- `kode lsp`: JSON-RPC over stdio, `Content-Length` framing.
- `initialize` / `shutdown` / `exit`, `textDocument/didOpen|didChange|didClose|didSave`.
- Diagnostics pushed on open and change, using the same checker as `kode check`.
- Hover (type of the expression under the cursor), completion (keywords, items,
  variables, stdlib functions), document symbols, formatting (`kode fmt`).
- Works with an unsaved buffer: the loader reads the open document, not the file on disk.
- `docs/editors.md` with VS Code and Neovim setup.
- **Done when:** an editor session shows errors live, `kode lsp` survives malformed
  requests, and `cargo test` covers framing, offset mapping and diagnostics.

### P2 — Packages and reuse (branch `feat/packages`)
- `[dependencies]` in `kode.toml` with `path` (git and registry stay out until a
  registry exists).
- `kode.lock` with resolved paths and versions; `kode install`, `kode add`, `kode remove`.
- `kode init` and `kode new` scaffold a package.
- Bare imports resolve into dependencies: `import "utils/strings"` finds the `utils`
  dependency's source root; `import ... as` still applies.
- Clear errors: missing dependency (E0412), version mismatch (E0413), name clash (E0414).
- **Done when:** a two-package workspace compiles and runs, the lockfile is stable across
  runs, and a missing dependency has an actionable message.

### P3 — Standard library depth (branch `feat/stdlib`)
- Lists: `contains`, `index_of`, `reverse`, `insert`, `remove_at`, `first`, `last`.
- Maps: `values`.
- Strings: `lines`, `char_at`, `is_empty`, `count`.
- Every function is one row in `kode-sema/src/std_fns.rs` plus one runtime symbol, with
  reference counting handled and covered by a `tests/run-pass` program.
- **Done when:** the new functions pass the conformance suite with no leaks.

### P4 — Errors and optionals (branch `feat/error-model`)
- `try { } catch e { } finally { }`, always runs, on both the normal and error path.
- A "did you mean" suggestion for unknown variables, functions and types, and a hint that
  names the closest standard-library function.
- Runtime errors carry the line number and a fix hint (`!` on `none`, index bounds,
  division by zero).
- `int?` and narrowing are designed in an ADR and scheduled, not half-implemented.
- **Done when:** `finally` has parser, checker, MIR, spec text and tests; new hints have
  tests in `tests/compile-fail`.

### P5 — CLI and tooling polish (branch `feat/cli-ux`)
- Colour that respects `NO_COLOR` and non-TTY output; diagnostics stay plain when piped.
- `kode doctor`: compiler version, target, linker, runtime ABI, package manifest.
- `kode check` / `kode build` report timing and what was produced.
- One consistent help screen listing every command.
- **Done when:** `--format=json` output is unchanged, piping produces no escape codes,
  and `kode doctor` reports a usable toolchain on a clean machine.

### P6 — Docs and Windows (branch `feat/docs-windows`)  - `kode doc <folder> -o docs/api`: HTML API reference from `pub` items, no dependencies.
  - `docs/site/` build script that assembles the spec, ADRs, error index and API pages.
  - Release files: `docs/releasing.md` describes the per-platform installers the
    release workflow now produces.
  - Windows: linker discovery for MinGW-w64 (`gcc`/`cc`), MSVC detection with a clear

  message, documented status and limits.
- **Done when:** `kode doc` generates a browsable site for `examples/`, and the Windows
  path either links or fails with an actionable message.

### P7 — Concurrency (branch `feat/channels`)
- `Channel<T>` with `channel()`, `send`, `recv`, `close`; blocking receive, thread-safe,
  reference-counted payloads.
- `Task` helpers for timeout/cancellation.
- Two-thread and producer/consumer tests, including error propagation.
- **Done when:** a producer/consumer program passes under `KODE_LEAK_CHECK=1` and
  `kode test` shows no data races under repeated runs.

### P8 — Performance (branch `perf/budgets`)
- `benchmarks/run.sh` measuring wall clock and peak RSS for Kode, C, Go, Node, Python
  when present, writing `benchmarks/report.md`.
- MIR pass work only where the numbers justify it.
- **Done when:** the budget table in section 3 is filled with real numbers, or the gap is
  documented with the cause.

## 5. Working agreement

1. Tests first: `tests/run-pass` (`.kode` + `.out`), `tests/compile-fail` (`.kode` + `.err`).
2. Behaviour changes need an ADR in `docs/adr/`; new error codes go in `docs/errors/index.md`.
3. One branch per phase, named as above, committed with its tests and docs.
4. `cargo test --workspace` stays green, and leak checking stays enabled.
5. Claims in README and docs cite a measurement or a test.

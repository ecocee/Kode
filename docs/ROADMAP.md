# Roadmap

For the working plan behind these milestones — branches, budgets and acceptance checks —
see [PLAN-v1.md](PLAN-v1.md).

Goal: a language that serves more requests per cloud server than Java, Node and Python,
is simple to write, and is easy for AI agents to use.

Time budget assumption: under 10 hours a week, so each milestone is small and ends with a
working, tested compiler.

| Milestone | Content | Status |
|---|---|---|
| **M1 Core** | int, float, bool, string; variables, const; operators; if, while, for ranges; functions and recursion; reference-counted strings; MIR; Cranelift; diagnostics with codes and JSON; conformance suite | **Done** |
| **M2 Collections** | `[T]` lists and `{string: T}` maps, indexing, `for x in list`, `.add`, `.len()`, `in`, `.keys()` (sorting, `map` and `filter` still to come) | **Done** |
| **M3 Classes** | `class` with fields and methods, named constructors, reference counting for objects, enums, `match` (`weak` fields still to come) | **Done** |
| **M4 Errors and optionals** | `throw` / `catch` / `try`, `T?` for heap types, `none`, `?.`, `??`, `!` (`finally`, `int?` and narrowing still to come) | **Done** |
| **M5 Tooling** | `import` and modules, `test` blocks and `kode test`, `kode fmt`, `kode explain`, agent cheat sheet | **Done** |
| **M6 Concurrency** | `spawn` / `await`, `Task<T>`, atomic reference counts, per-thread errors (channels and `shared` classes still to come) | **Done** |
| **M7 Services** | `serve(port, handler)`, strings, files, args, environment, sorting, JSON, HTTP client, Router/`Request`/`Response` (database client and the REST benchmark still to come) | **Done** (core) |
| **M7.5 Ergonomics** | integer bit operators and compound assignment; Python-style stdlib namespaces (`json.parse`, `math.sqrt`, `http.get`, `fs.read`, `time.now_ms`) | **Done** |
| **M7.6 Interfaces** | `interface` signatures, `implements`, dynamic dispatch, signature checks | **Done** |
| **M7.7 Packages** | `kode.toml` manifests with a source root, package-relative imports, `import ... as` qualified access, `pub`-gated module exports (dependencies and a registry still to come) | **Done** |
| **M8 Budgets and AI** | `memory` budgets, declared effects (`uses`), typed `ask` / `tool` calls | Planned |
| **M9 Release** | LLVM backend for `--release` (optional), Windows, language server, docs site, open source | Planned |

## Checkpoints (go / no-go)

- After M2: the same small program in Kode and Python, compared on speed and lines of code. (`examples/bank.kode` is the first candidate.)
- After M7: the REST benchmark on a 1 vCPU / 1 GB VM. Kode must clearly beat Python and Node on
  requests per second and RAM, and be close to Go. If not, change the plan before M8.

## Principles

1. The conformance suite is the source of truth for the language.
2. One backend input: MIR.
3. Every error has a code and a hint.
4. Measure before claiming.

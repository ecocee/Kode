# Roadmap

Goal: a language that serves more requests per cloud server than Java, Node and Python,
is simple to write, and is easy for AI agents to use.

Time budget assumption: under 10 hours a week, so each milestone is small and ends with a
working, tested compiler.

| Milestone | Content | Status |
|---|---|---|
| **M1 Core** | int, float, bool, string; variables, const; operators; if, while, for ranges; functions and recursion; reference-counted strings; MIR; Cranelift; diagnostics with codes and JSON; conformance suite | **Done** |
| **M2 Collections** | `[T]` lists and `{K: V}` maps, indexing, `for x in list`, `.add`, `.len()`, sorting, `map` and `filter` with lambdas | Next |
| **M3 Classes** | `class` with fields and methods, built-in constructor, `weak` fields, reference counting for objects, enums, `match` | Planned |
| **M4 Errors and optionals** | `throw` / `catch` / `try`, `T?`, `none`, `?.`, `??`, flow typing | Planned |
| **M5 Tooling** | `import` and modules, `test` blocks and `kode test`, `kode fmt`, `kode explain E0201`, project layout, agent cheat sheet | Planned |
| **M6 Concurrency** | `spawn` / `await`, lightweight tasks, channels, atomic reference counts, `shared` classes | Planned |
| **M7 Services** | `serve` / routes, HTTP and JSON in the standard library, database client, the REST benchmark against Go, Node, Python and Java | Planned |
| **M8 Budgets and AI** | `memory` budgets, declared effects (`uses`), typed `ask` / `tool` calls | Planned |
| **M9 Release** | LLVM backend for `--release` (optional), Windows, language server, docs site, open source | Planned |

## Checkpoints (go / no-go)

- After M2: the same small program in Kode and Python, compared on speed and lines of code.
- After M7: the REST benchmark on a 1 vCPU / 1 GB VM. Kode must clearly beat Python and Node on
  requests per second and RAM, and be close to Go. If not, change the plan before M8.

## Principles

1. The conformance suite is the source of truth for the language.
2. One backend input: MIR.
3. Every error has a code and a hint.
4. Measure before claiming.

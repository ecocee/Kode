# 7. Planned (not implemented yet)

This chapter lists what the language deliberately does not do yet. It is kept
honest: things that shipped are removed from here, and the chapter that covers
them describes the behavior instead. See [PLAN-v1.md](../PLAN-v1.md) for the
current production track (M9).

**Shipped and documented elsewhere:** channels (chapter 12), `finally`, generic
functions (chapter 5), printing lists and maps (chapter 8, ADR 0031), the language
server with go-to-definition, references and rename, `kode doc`, `kode doctor`,
and path dependencies with `kode.lock` (chapter 11).

Still missing, in the order the plan takes them:

- **`weak` references.** Reference counting cannot break a cycle, so two objects
  that point at each other leak (see `docs/design/memory-model.md`).
- **Byte buffers and fixed-width integers.** Only 64-bit `int`; no `u8`/`i32`
  families, no `[]byte`, no `StringBuilder`, no binary encode/decode.
- **Enum payloads and type tests.** Enums are tags only: no `Shape.Circle(radius)`,
  no `x is Circle`, no safe downcast.
- **Integer map keys.** Maps are string-keyed by decision (ADR 0014).
- **Optional scalars beyond locals.** `string?`, `[T]?` and class optionals work as
  fields and parameters; `int?`/`float?`/`bool?` work only as locals (ADR 0024).
- **Generic classes, generic methods and constraints.** Free functions only.
- **Checked arithmetic.** `+ - *` wrap on overflow; checked operators are planned.
- **A package registry.** Dependencies are `path` only; `kode.lock` pins them.
- **A verified Windows release.** The build path and CI job exist; a real Windows
  run must pass before a download is offered.
- **Bounded channels** (`channel(16)`), `shared` classes, and a task thread pool
  (every task is currently a thread).
- **A compiler written in Kode**, and the bootstrap automation around it.

**Not planned:** resource budgets (`memory 64mb`), declared effects (`uses db`) and
typed `ask` / `tool` calls. Those were M8; the project skipped M8 to focus on being a
general-purpose language first.

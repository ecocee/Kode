# Memory model

Every heap value (string, list, map, class instance) is **reference counted**. The compiler
inserts `retain` and `release` in MIR; users never write them.

All heap values share one header: `[refcount: i64][kind: i64]`. A refcount of -1 marks a static
value (string literals) that is never freed. The `kind` says what it is, so one `release`
function can free anything, including the items inside a list or the fields of an object.

| Kind | Layout after the header |
|---|---|
| string | `[length][bytes...]` |
| list | `[length][capacity][pointer to 8-byte items]`; a flag says whether the items are heap values |
| map | `[pointer to the runtime map]` (string keys, insertion order); same flag for values |
| class instance | `[refmask][fields...]`, 8 bytes per field; `refmask` says which fields are heap values |

## Rules (`crates/kode-mir/src/lower/mod.rs`)

1. A value is **owned** (we hold a reference) or **borrowed** (a variable, a literal, a field).
2. Storing in a variable, a field, a list or a map, or returning, takes ownership
   (`retain` if it was only borrowed).
3. Assigning to a variable first releases the old value. Variables start as `null`; releasing
   null is a no-op.
4. Every function has **one exit block** that releases all its heap variables. `return`
   jumps there, so no path can forget to release.
5. Parameters are borrowed. Callers keep ownership of what they pass.
6. Temporaries are released right after use.
7. **Errors:** if an error is raised while a temporary is alive, the error path releases it too
   (the lowering tracks the live temporaries as `pending`).
8. A collection or object that is being looped over is held in a hidden variable so it cannot
   be freed during the loop.

## Checking it

The runtime counts live objects. `KODE_LEAK_CHECK=1` makes a program exit with code 3 if any
object is left at the end. `cargo test` sets it for every run-pass program.

## Known limits

- A variable lives until its function exits or it is re-assigned, not until its block ends.
- Counts use atomics once a program creates its first channel (`THREADED` in the object header,
  set in `crates/kode-rt/src/channel.rs`); single-threaded programs pay nothing extra.
- Two objects that point at each other leak, because reference counting cannot see a cycle.
  Measured with `KODE_LEAK_CHECK=1`: a two-object cycle leaks 2 objects, a ring of 10,000 leaks
  10,000 (`benchmarks/leaks/report.md`). `weak` references are designed in
  [ADR 0038](../adr/0038-weak-references.md) but not implemented; until then keep back-links out
  of the object graph or break them by hand (`a.next = none`). `weak` makes a cycle avoidable —
  it does not collect one that is already built from strong fields.

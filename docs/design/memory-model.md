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
- Counts are not atomic yet; concurrency (M6) will make them atomic.
- Two objects that point at each other leak. `weak` references are planned.

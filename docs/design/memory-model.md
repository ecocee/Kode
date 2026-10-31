# Memory model (v0.1: strings)

Strings use **reference counting** with the ownership rules below. The compiler inserts
`retain` and `release` in MIR; users never write them.

Layout: `[refcount: i64][length: i64][bytes...]`. A string value is a pointer to that header.
Static literals are stored in the program with `refcount = -1`, which means "never free".

Rules (`crates/kode-mir/src/lower/mod.rs`):

1. A string value is **owned** (we hold a reference) or **borrowed** (a variable or a literal).
2. Storing in a variable, or returning, takes ownership (`retain` if it was borrowed).
3. Assigning to a variable first releases the old value. Variables start as `null`; releasing null is a no-op.
4. All string variables of a function are released when it returns.
5. Parameters are borrowed. Callers keep ownership of what they pass.
6. Temporaries are released right after use.

Effects: no garbage collector, no pauses, memory is freed at once. The runtime counts live
strings; `KODE_LEAK_CHECK=1` makes a program fail if any remain at exit.

Known limits and next steps:
- A string variable lives until its function ends or it is re-assigned, not until its block ends.
- Counts are not atomic yet. Concurrency (M6) will make them atomic.
- Classes (M3) reuse these rules. Two objects pointing at each other would leak, so `weak`
  references and a compiler warning for likely cycles are planned.

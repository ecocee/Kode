# ADR 0027: Printing enums and slicing lists

Status: accepted. 2026-11-05.

## Context

`print` accepted ints, floats, bools and strings. The type checker already let an
enum through in `match`, and the MIR lowering for `print` had a catch-all
`unreachable!`, so an enum had no printable form at all: `print(Color.Red)`
crashed the compiler instead of printing.

Lists could be indexed one item at a time but had no way to take a range.
`items[1..3]` did not parse, so the usual workaround was a `for` loop and
`add`, which is a lot of noise for a common operation.

Both are the first things a new user tries.

## Decision

### Enums print the way they are written

`print(c)` writes `Color.Red`: the same text that names the value in source.
The same conversion serves `${c}` in an interpolated string.

The lowering builds a chain of `select` instructions over the variant tags, one
arm per variant, each arm a static string. Two consequences:

- No runtime function, string table, or ABI change is needed. The result is an
  ordinary string constant, so it is never allocated and never freed.
- Enums are still plain ints (ADR 0009). Printing changes nothing about the
  representation, so `c == Color.Red` and `match` keep working as before.

An unknown tag cannot occur: the only way to get an enum value is a variant
name, and the type checker validates it (E0225).

### A slice is a copy

`xs[a..b]` is a new list holding the items in `[a, b)`. `xs[a..=b]` includes
`b`. Either bound may be left out: `xs[..2]` and `xs[2..]`.

Bounds are **clamped** to `[0, len]`:

- `xs[10..20]` on a five-item list is empty, not an error.
- A negative bound counts from the start, that is it becomes `0`.
- `a >= b` is empty.

Only lists can be sliced; slicing a string is not provided yet (E0211).

Rejected alternatives:

- **A view that shares the parent's storage.** It would need a new object kind,
  a retain on the parent, and careful rules about mutating the parent while the
  view is live, plus an ABI bump. A copy costs one runtime function and no
  memory-model change.
- **Python-style negative indices counting from the end.** `xs[-1]` is an error
  today; giving `-1` a second meaning inside slices only would be a trap.
- **Reusing the index path, with out-of-range bounds fatal.** An index mistake
  is almost always a bug, and the runtime says so (`kode_list_oob`). A slice is
  naturally a range operation, where "past the end" means "stop at the end".

## Consequences

- Lists of bools are packed (one byte per item), so they slice through
  `kode_list_slice8`; everything else uses `kode_list_slice`.
- Items are retained when copied, so a slice of strings owns its references and
  outlives the list it came from.
- `E0211` now covers slicing as well as indexing, and `E0212` covers a non-int
  bound.
- `kode_list_slice` returns a list with a reference count of one; the MIR
  lowering marks the result owned, so normal ownership rules apply to it.

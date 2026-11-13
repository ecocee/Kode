# ADR 0041: optional scalars in class fields

Status: **accepted** — slice 1 landed; slice 2 (parameters, returns, collections) is
WP-06b.

## Context

`int?`, `float?` and `bool?` are a pair, the value and a `present` flag, never a box
(ADR 0024). They worked as locals only: E0219 refused them in a field or a parameter.
S2 needs them in fields before the AST exists, because a span with "no parent" and a
"line if known" is exactly this shape, and without it the port has to keep two
parallel fields (`line` and `has_line`) and remember to move them together.

A class object is a header and one 8-byte slot per field; `nfields` in the class id
bits is both the size and the drop count, and the refmask says which slots own a
reference.

## Decision

1. **Two slots per optional scalar field.** The field's value is at its slot and the
   flag in the slot after it. `nfields` counts slots, so the object is 16 bytes per
   such field; neither slot owns a reference, so the refmask bit is clear for both and
   drop is unchanged.
2. **One layout function.** Lowering computes each class's offsets once
   (`Lowerer::field_offsets`) from the field types: 8 bytes, or 16 for an optional
   scalar. Construction, field reads, field writes and `this`-reads all use it, so no
   site can drift.
3. **The flag travels with the value.** Construction and assignment write both slots:
   a value that already carries a flag (`x: int?`) passes its own flag, `none` writes
   a clear flag, and a plain value the checker widened writes a set flag. A read hands
   back the pair, so `??`, `!`, `== none` and narrowing are the same code as for a
   local.
4. **`?.` stays safe.** `p?.n` on an optional scalar field is the null check followed
   by both loads, with a clear flag on the none path.
5. **Not in this slice:** a parameter, a return type, and a list or map of an optional
   scalar stay E0219. A parameter would have to be two ABI operands, which changes the
   calling convention, function values, closures and interface dispatch; that is
   WP-06b, not a side effect of this ADR.

## Consequences

- A class with one optional scalar field costs 8 bytes more than the field count
  suggests; `nfields` is a slot count, not a field count, and anything that walks
  fields must use the offsets.
- The class id's 8-bit class number and 8-bit field count are unchanged, so this is
  the same object header as before.
- A field's slot index is no longer its field index. Every future field walk has to go
  through `field_offsets` (the checker's `FieldInfo` still lists fields, not slots).
- Locals keep the one-local-pair representation; the two are identical in meaning, so
  a value moves between them without conversion.

## Alternatives

- **Box the pair.** One slot, params and collections work immediately, but every
  optional scalar local would allocate, and `int?` is used in hot paths (spans,
  indices). Rejected for now, not forever.
- **Pack value and flag into one word.** Works for `bool?` and `float?` bit tricks,
  loses range or precision for `int?`, and needs per-type unwrap code. Rejected.
- **Two parallel fields in the AST.** What the port would do today: `line` and
  `has_line`, plus every site remembering both. That is the bug this prevents.

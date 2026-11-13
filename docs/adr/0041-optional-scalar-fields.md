# ADR 0041: optional scalars in class fields

Status: **accepted** — fields, parameters and returns landed; a list or map of an
optional number is still E0219.

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
5. **A call boundary is one reference.** A parameter or a return of type `int?`
   would need two ABI operands or a second return slot, which reaches the calling
   convention, function values, closures, thunks and interface dispatch. Instead the
   pair is boxed at the boundary: `[value][flag]`, or `null` for none. The callee
   decodes the box into its value and flag locals at entry and boxes its pair on the
   way out, so `??`, `!`, `== none` and narrowing inside the function are the code a
   local already uses. `pass_vals` and `decode_ret` are the only two places that know
   this: direct calls, method calls, interface dispatch and calls through a function
   value all go through them. A box is owned by the caller and released after the
   call; a parameter's box is borrowed, a returned one is owned and released by the
   caller after decoding.
6. **Still not supported:** a list or map of an optional number is E0219. A list item
   is one slot and would need the same box, which changes every list operation.

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
- A call that takes or returns an optional number allocates one box per such
  parameter or result. That is the price of not changing the calling convention; a
  function with none of them is unaffected, which is every function today.

## Alternatives

- **Box the pair everywhere.** One representation, no conversion at a call, but every
  optional scalar local would allocate, and `int?` is used in hot paths (spans,
  indices). Rejected: boxing stays at the boundary.
- **Two ABI operands, or a second return slot.** No allocation, but every call site,
  function value, closure, thunk and interface dispatch would have to describe the
  wider signature, and the backend would return a pair. Rejected for now: it is a
  calling-convention change for a shape the AST does not call hot.
- **Pack value and flag into one word.** Works for `bool?` and `float?` bit tricks,
  loses range or precision for `int?`, and needs per-type unwrap code. Rejected.
- **Two parallel fields in the AST.** What the port would do today: `line` and
  `has_line`, plus every site remembering both. That is the bug this prevents.

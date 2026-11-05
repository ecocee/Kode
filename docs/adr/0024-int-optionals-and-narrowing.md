# 0024 — `int?`, `float?`, `bool?` and narrowing

Status: accepted, implemented for locals (branch `feat/optional-numbers`).

## Context

`T?` is a nullable pointer today, which is free for strings, lists, maps and
classes (E0219 rejects `int?`, `float?` and `bool?`). Numbers and bools are
unboxed, so there is no spare bit pattern for `none` and boxing them would cost
an allocation per value -- against the memory budget in `PLAN-v1.md`.

## Decision

- **Representation.** An optional number or bool is a pair: the value and a
  `present` flag. In MIR that is two locals (or two object fields) of the same
  lifetime, one byte of extra storage per value, no allocation, no indirection.
  There is no `Option<i64>`-shaped heap object.
- **Operations.** `??`, `!`, `== none` / `!= none` work as they do for heap
  types; `!` raises the existing runtime error. `?.` stays a heap-type feature.
- **Narrowing.** The checker narrows a **local variable** after
  `if x != none { ... }`, `while x != none { ... }`, and in the code after
  `if x == none { return }` / `throw`. Inside the narrowed region the variable's
  type is `T`, so plain use is allowed. Narrowing ends at the end of the block,
  or at any assignment to the variable. Fields, parameters and elements are not
  narrowed in this version, and there is no aliasing narrowing (`a? = b`, then
  using `a!`) -- those need an effect analysis and are out of scope.
- **Diagnostics.** E0219 disappears for these three types once this ships; hover
  and `kode check` still show the declared type (`int?`), never the narrowed one.
- **Strings and interpolation.** `"${x}"` on an optional number prints `none` or
  the number; the branch lives in the runtime formatter, not at the call site.

## Consequences

- Lists and maps of optionals (`[int?]`) need the pair representation in their
  element layout too; that is a follow-up ADR, so `[int?]` stays an error
  (E0219) for now.
- MIR gains a `Ty::Opt(Ty)` shape for scalars only; codegen sees the pair and a
  select, not a new value class.
- `tests/run-pass/optional_numbers.kode` (narrowing, `??`, `!`) and a
  `tests/compile-fail` case for a non-narrowed use land with the implementation.
- **Limit in this version.** An optional number or bool is a *local*. Parameters,
  return types, class fields, and list or map elements still report E0219,
  because the pair lives in two locals while every other storage slot (an
  argument block, a field, a list item) is one word. Lifting that means changing
  argument blocks and field layout, so it is a follow-up.

## Schedule

Branch `feat/optional-numbers`, after `feat/channels`: checker narrowing first,
then MIR pairs, then the formatter. Until then this ADR is the reference, and
`int?` reports E0219 with a pointer to it.

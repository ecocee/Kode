# 31. Printing lists and maps

Status: accepted (M9, production core).

## Context

`print` and string interpolation accepted ints, floats, bools, strings, enums and
optional numbers. A list or a map was E0203: "cannot print `[int]` yet". That made
debugging awkward in the most ordinary case — you cannot look at a collection
without writing a loop — and it made `${xs}` unusable in messages and tests.

A general-purpose language needs a way to see a value. Every comparable language has
one: Python `[1, 2]`, Go `fmt.Println(xs)` printing `[1 2]`, JavaScript `[1,2]`.

## Decision

`print(xs)` and `"${xs}"` work for a list or a map whose **items can already print**.
Item types with a text form: `int`, `float`, `bool`, `string`, and lists or maps of
those, to any depth.

The output is Python-like:

```
print([1, 2, 3])              # [1, 2, 3]
print(["a"])                  # ["a"]
print([true, false])          # [true, false]
print({"a": 1})               # {"a": 1}
print([[1], [2, 3]])          # [[1], [2, 3]]
```

- **Strings inside a collection are quoted and escaped** (`"`, `\`, newline, tab,
  carriage return). A top-level `print("text")` stays unquoted. Without quoting,
  `["a, b"]` and `["a", "b"]` would print identically.
- **Maps keep insertion order.** Keys are strings by ADR 0014, so they print quoted.
- **Empty collections print `[]` and `{}`.**

Class instances, enums, interfaces, function values, tasks and channels are still
E0203, with a hint to loop and print the parts. An enum inside a list would need the
variant table at run time; a class would need field names. Both are separate
decisions, not this one.

## How it works

The compiler knows the item type and the runtime does not, so the compiler passes a
**descriptor string** and the runtime walks the value and the descriptor together:

| descriptor | value |
|---|---|
| `i` | int |
| `f` | float |
| `b` | bool |
| `s` | string |
| `[...]` | list of the inner descriptor |
| `{...}` | map with that value type |

`kode_sema::types::repr_descriptor` is the single place that builds it. The checker
uses it to accept or reject a `print`; the lowering uses it to emit the text, so the
two cannot disagree. The runtime entry point is `kode_repr(value, descriptor)` in
`crates/kode-rt/src/repr.rs`; MIR sees it as `RtFn::Repr`.

Lists of `bool` are packed one byte per item (ADR 0019), so the formatter reads packed
and wide lists separately. Nothing is retained: the formatter borrows the collection
for the call and builds one new string, which the caller owns and releases. This is
what keeps `KODE_LEAK_CHECK=1` quiet.

## Consequences

- Debugging and tests get much shorter; `print(xs)` replaces a loop.
- The descriptor is a compiler/runtime contract. Adding a printable type means
  extending `repr_descriptor`, `parse` and `write` in `repr.rs`.
- Output is a debugging form, not a serialisation format. `json.stringify` remains
  the way to produce data; `print` output is for humans and may change.
- Nested collections cost one allocation for the whole text, not one per item.

Tests: `tests/run-pass/collection_print.kode` (every supported shape, run under
`KODE_LEAK_CHECK=1`) and `tests/compile-fail/e0203_print_class_list.kode`.

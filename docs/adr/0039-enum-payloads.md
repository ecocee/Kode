# ADR 0039: Enum payloads

Status: **accepted** — slices 1-3 landed. Payload variants parse and appear
in `kode ast`, are constructed (`Shape.Circle(radius: 1.5)`), print with their
field names, and a `match` compares the tag; the object layout is the class
layout with the variant tag in the class-id bits, and `kode_enum_tag` is the one
new runtime symbol. The representation rule of §2 is exactly as written: tag-only
enums stay `int`, one payload variant makes every variant a heap object. `E0253`
is gone; `E0417`, `E0419` and `E0420` landed with their rows. Slice 2 (pattern
bindings, `Shape.Circle(r)`) binds positionally, names are ordinary locals, and a
recursive `Value` tree runs leak-clean. Slice 3 (`is` and narrowing) is in: `s is
Shape.Circle` is one tag compare, the true branch may read `s.radius`, and a read
without a narrowing is `E0419` rather than a runtime check. Only `E0418` stays
folded into `E0208`/`E0216` (constructor field names), and a pattern with the wrong
number of names is `E0207`.
Date: 2026-11-11.
Plan item: `docs/PLAN-v1.md` 9.7, `docs/STATUS.md` 1.1. Blocks S2 of
`docs/PLAN-self-hosting.md`.
Related: ADR 0009 (tag enums today), ADR 0027 (enum printing), ADR 0010 (only MIR
reaches the backend).

## 1. Context

Enums are tags only. `EnumInfo` stores `variants: Vec<String>` and the MIR type
of an enum is `Ty::Int` (`kode-mir/src/lower/mod.rs`: `Type::Int | Type::Enum(_)
=> Ty::Int`), so `Color.Green` is the integer 1 and `match` is an integer switch.

That is exactly right for a tag enum and exactly wrong for an AST, which is a
tagged union: a node is a call *with* a callee and arguments, or an identifier
*with* a name. `docs/PLAN-v1.md` states the consequence already — without
payloads the AST becomes "N parallel fields plus a tag", the duplication the
project exists to avoid. S2 (the parser in Kode) cannot be written honestly
before this lands.

## 2. Decision

An enum variant may declare fields. One syntax, one rule about representation:

```kode
enum Shape {
    Circle(radius: float)
    Rect(width: float, height: float)
    Empty
}

s = Shape.Circle(radius: 1.5)
print(s)                       // Shape.Circle(radius: 1.5)

match s {
    Shape.Circle(r) => print("circle", r)
    Shape.Rect(w, h) => print("rect", w * h)
    Shape.Empty => print("empty")
}
```

- A variant declares **named** fields, in parentheses, in order. Names are
  documentation and printing; patterns bind **positionally**, in that order.
- A variant with no parentheses is unchanged (`Shape.Empty`).
- **One rule decides the representation:** an enum whose every variant has no
  fields keeps today's `int` representation, unchanged — no allocation, no ABI
  change, no cost for existing programs. An enum with *at least one* payload
  variant represents **every** variant as a heap object, including the
  payload-less ones (0 fields). A program never has to think about which it got;
  the rule is visible in the declaration.
- A payload variant is **only** usable as a constructor call: `Shape.Circle` on
  its own is `E0417`, not a function value.
- `==` is `E0420` on a payload enum; use `match` or `is`. Structural equality on
  heap values needs a definition of equality for every payload (and, for
  classes, a decision about pointer identity vs structure) that this ADR does
  not open.

## 3. Runtime representation

Reuse the class-instance layout exactly as it is (`crates/kode-rt/src/object.rs`):

```
[rc: i64][kind: i64][refmask: i64][field0][field1]...
                ^ kind = OBJ | tag << 8 | nfields << 16
                  size = 24 + 8 * nfields
```

`tag` is the variant index in declaration order — the number tag enums already
use — and `nfields` is that variant's field count. Because the tag and the count
live in the header, `kode_retain`, `kode_release`, `drop_obj` and the refmask
sweep work unchanged, and `nfields` at drop time is correct even though the
static type does not know which variant it holds.

Consequences:

- **No ABI change**, no new object kind, no allocator change. `kode_rt_abi_0004`
  stays.
- One new runtime entry point, registered in `kode-mir/src/ir.rs` (`RtFn`) and
  `crates/kode-llvm/src/lower.rs` (`RT`): `kode_enum_tag(ref) -> int`, which
  reads the tag out of the header for `match` and `is`. Construction uses the
  existing `kode_obj_new_class(tag, nfields, refmask)`; field reads and writes
  use the existing class paths.
- Enum identity is **not** in the header. `match` inside one enum needs only the
  tag, and a value of the wrong enum type cannot reach that code: the checker
  rejects it. Adding an enum id would cost 8 bytes per value to defend against
  a mistake the type system already prevents.

## 4. Semantics

1. **Construction** evaluates each field expression once, left to right, retains
   it into the object, and may name arguments (`Shape.Rect(width: 1.0,
   height: 2.0)`), matching class constructors (ADR 0015). Arity, names and types
   are checked exactly like a constructor; a wrong name is `E0418`.
2. **Reading a field** is only possible through `match`; there is no `.radius`
   access on `Shape`. That keeps downcasting explicit and total.
3. **`match`** compares the tag and binds the fields of the matched variant. A
   binding retains the field for the arm; leaving the arm releases it, which is
   the rule class field reads already follow (`kode-mir/src/lower/mod.rs`).
   Exhaustiveness is the existing rule (E0224): every variant, or `_`.
4. **`is`** tests a variant and narrows in the same way `!= none` narrows today:

   ```kode
   if s is Shape.Circle {
       print(s.radius)      // narrowed: the Circle variant
   }
   ```

   `s is Shape.Circle` is a compile-time constant when the static type already
   proves it; otherwise it is a tag compare. `s.radius` after a failed `is`
   (reachable only through `else`) is `E0419`.
5. **Printing** keeps the source form, as ADR 0027 established for tag enums:
   `Shape.Circle(radius: 1.5)`, `Shape.Empty`. Field names come from the
   declaration and the text is built at compile time (static string operands),
   so printing costs no reflection.
6. **Lists, maps, optionals and arguments** carry payload enums as ordinary
   references; `Shape?` works for the same reason `string?` does (ADR 0013).
7. **No `match` on a field of a value that moved**: payloads are shared
   references, not linear values; there is no move.

## 5. Checker changes

| crate | change |
|---|---|
| `kode-syntax` | `EnumDecl.variants: Vec<EnumVariant { name, name_span, fields: Vec<Param> }>`; parse `Name(field: Type, ...)`; the AST dump gains the variant fields; `fmt` prints them back |
| `kode-sema` | `EnumInfo` gains per-variant fields and the `has_payloads` flag; `Type::Enum(name)` keeps its identity, but `Type::enum_payload(..)` answers "is this variant a constructor, and with which signature"; construction type-checking reuses the class-constructor path; `match` patterns resolve variant + field count; `is` narrows; `E0417`–`E0420` |
| `kode-mir` | payload enums lower as references (`Ty::Ref`) instead of `Ty::Int`: constructor (`kode_obj_new_class` with tag and count), tag read (`kode_enum_tag`), field bind, print |
| `kode-llvm` | one `RT` row for `kode_enum_tag`; nothing else — IR sees only MIR |

`E0253` ("enum payloads are not supported yet") is today's diagnostic for
`Shape.Circle(1)`; slice 1 removes it, and it must be deleted from
`docs/errors/index.md` in that commit rather than left as a dead code.

Reserved codes, to land with their implementations and their
`docs/errors/index.md` rows so `kode explain` works:

| code | condition | hint |
|---|---|---|
| `E0417` | a payload variant used as a value | "`Shape.Circle` is a constructor; write `Shape.Circle(radius: 1.0)`" |
| `E0418` | unknown or duplicate field in a variant constructor or pattern | "the variant declares `radius` and `height`" |
| `E0419` | a field read on a variant the value may not be | "narrow with `if s is Shape.Circle { }` first" |
| `E0420` | `==` on an enum with payloads | "use `match`, or `is` to compare the variant" |

## 6. Ownership and leaks

- Construction retains each field, so a constructed enum owns its payloads.
- `drop_obj` already releases fields through `refmask`; an enum object sets the
  same mask, so nothing new is needed in the runtime.
- A match binding retains the field when it is bound and releases it when the
  arm's scope ends, so the object may be dropped while the binding lives.
- No cycles are introduced: an enum cannot contain itself except through a list,
  class or optional, which are the existing reference types.
- `KODE_LEAK_CHECK=1` must stay at zero live objects for every new test; a
  payload enum in a list, matched and dropped, is the case that catches a missed
  release.

## 7. Alternatives considered

- **Payloads as plain structs or classes.** No exhaustiveness, no `is`, and the
  AST would be N parallel optional fields plus a tag — the duplication this ADR
  exists to remove.
- **A fixed union of scalar slots instead of a heap object.** Fits tag-only
  enums but cannot hold strings, lists or nested enums; the AST needs all three.
- **Boxed payloads behind a pointer with a separate tag word.** Costs the same
  allocation and one more indirection, and buys nothing: the header already has
  room for the tag.
- **Structural `==` on payload enums.** Would need equality for every payload
  type, a cycle policy, and a decision about classes; `match` covers the real use
  and keeps the rule simple. Revisit if a program needs it.
- **Field access without narrowing (`s.radius`).** Unsafe by construction (the
  value may be another variant) and would need a runtime check with a
  catchable error for a mistake the type system can catch statically.

## 8. Costs

- Program with payload enums: one allocation per constructed variant, one tag
  read per `match` arm, 8 bytes per field — the same as a class with the same
  fields. Tag-only programs pay nothing and their ABI is unchanged.
- Compiler: new syntax in the parser, dump and formatter; checker rules; one MIR
  path; one runtime function. Binary-size effect of `kode_enum_tag` is a few
  dozen bytes and must still be measured against the budget before merging
  (AGENTS.md rule 8, `benchmarks/report.md`).

## 9. Slices (each one branch, one commit set, tests first)

| slice | branch | tests |
|---|---|---|
| 1. declaration, construction, printing | `feat/enum-payloads` | `tests/run-pass/enum_payloads.kode` (+`.out`), `enum_print.kode`; `compile-fail/e0417_payload_value.kode`, `e0418_unknown_field.kode` |
| 2. `match` destructuring and exhaustiveness | `feat/enum-match` | `tests/run-pass/enum_match.kode`; `compile-fail/e0224_enum_match_incomplete.kode` |
| 3. `is` and narrowing | `feat/enum-is` | `tests/run-pass/enum_is.kode`; `compile-fail/e0419_field_without_narrow.kode` |
| 4. `==` diagnostic and dump/fmt polish | `feat/enum-eq-diagnostic` | `compile-fail/e0420_enum_eq.kode`; `crates/kode-syntax` dump test |

Each slice ends green: `cargo test --workspace` with `KODE_LEAK_CHECK=1`,
`cargo clippy --workspace`, `./target/debug/kode fmt --check <paths>`, and — once
`kode ast` exists for the port — `./scripts/selfhost.sh stage0`.

## 10. Open questions for the maintainer

1. Confirm the representation rule (§2): tag-only enums stay `int`, any payload
   makes the whole enum heap. The alternative — every enum heap — is simpler to
   explain but makes every existing `match` an allocation-free switch no longer,
   and changes the ABI for programs that never asked for it.
2. Positional bindings in patterns (`Shape.Rect(w, h)`) with named fields in the
   declaration, or named bindings in patterns too (`Shape.Rect(width: w)`)?
3. Is `is` plus narrowing the right naming, or should the downcast be an
   expression (`s as Shape.Circle`) with a `none` result for non-matches?
4. Do we want payload enums printable by default (as in §4.5), or is a
   `print` that says `Shape.Circle(...)` too much magic for a systems language?

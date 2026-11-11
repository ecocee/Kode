# ADR 0040: Generic classes

Status: **proposed** — design only. No compiler change has been made; the three
slices in §9 are separate, test-first changes and need the maintainer's approval,
because this adds syntax (AGENTS.md "Ask the maintainer first").
Date: 2026-11-11.
Plan item: `docs/PLAN-v1.md` 9.9, `docs/STATUS.md` 1.2. Blocks S2 of
`docs/PLAN-self-hosting.md`.
Related: ADR 0030 (generic functions and monomorphization), ADR 0006 (no
inheritance), ADR 0031 (printing collections), ADR 0010 (only MIR reaches the
backend).

## 1. Context

Generic *functions* work and are monomorphized (ADR 0030): a call is resolved to
one concrete instance, named `first.int`, and the checker then checks a
declaration whose annotations are already concrete, so `Type` never contains a
type variable.

Generic *classes* are refused today: `class Box<T>` is a parse error, `ClassInfo`
has no type parameters, and a method with type parameters is E0252.

Every compiler data structure S2/S3 needs is a container of a type argument:
token buffers, AST node lists, symbol tables, diagnostic lists. Without generic
classes each container is duplicated per element type, or everything is a
`string`/`int` map — the second is what the port would do untreated, and it is
the kind of untyped shortcut the language otherwise refuses.

## 2. Decision

A class may declare type parameters, and one concrete instance is compiled per
set of type arguments — the same strategy, and the same machinery, as ADR 0030.

```kode
class Box<T> {
    value: T
    fn get() -> T => value
    fn replace(next: T) -> T {
        old = value
        value = next
        return old
    }
}

b = Box(value: 1)          // Box<int>, inferred from the argument
i = b.get()                // int
s = Box(value: "hi")       // Box<string>, a second instance

boxes = [Box(value: 1), Box(value: 2)]   // [Box<int>]
```

- Type arguments are **inferred at the constructor**, from the argument types,
  exactly as a generic function's are from its parameters. Each declared type
  parameter must be bound by at least one field or parameter type; otherwise the
  constructor is `E0422`.
- In a type annotation, a class name may carry arguments: `Box<int>`,
  `[Box<int>]`, `{string: Box<int>}`, `Box<int>?`. An annotation with the wrong
  number of arguments is `E0421`.
- A method may use the class's type parameters. It may **not** declare its own:
  that stays E0252 (see §6).
- No explicit type arguments at the constructor (`Box<int>(value: 1)`); a value
  that cannot be inferred is an error, not a hint to annotate.

## 3. Where the work happens

Monomorphization is a **checker** transformation; MIR and the backend keep seeing
only concrete types, which is rule 4 of the project.

1. `ClassDecl` gains `type_params: Vec<(String, Span)>`; `TypeAnn` gains
   `args: Vec<TypeAnn>` on a named type. Parser, AST dump and `kode fmt` learn
   the syntax.
2. Collection records `ClassInfo` with its type parameters and field types
   containing them, as it records generic `FnDecl`s today.
3. A constructor call with `T`s infers each argument from the field types: the
   first binding wins, and a second occurrence must agree (`Box(value: [1],
   other: ["a"])` with `T = int` and `T = string` is E0208, the existing
   "same parameter, two types" code).
4. Instantiation produces one concrete class **per distinct tuple** of type
   arguments: clone the declaration, substitute every type parameter, mangle the
   name (`Box<int>` → `Box.int`, `Box<[string]>` → `Box.list_str`), and record it
   in `Checked::classes` with `class_index` pointing at it. Instances are
   created on demand and reused: `Box(value: 1)` twice is one class.
5. Each method is instantiated with **the class's** arguments, mangled
   `Box.int.get`, and appended to `Checked::bodies` like any other function. The
   existing resumable body pass (`Checker::check_bodies`) checks them, so a
   generic class calling a generic function inside a generic class terminates
   for the same reason ADR 0030 terminates.
6. The constructor expression records which instance it resolved to, so lowering
   emits the right field count and the right method names. MIR gains **nothing**:
   instances are ordinary classes with ordinary fields.

## 4. How instances stay distinct

`Type::Class(String)` keeps a name, and that name is the mangled instance name, so
field lookup, `[]` element types, optionals, interfaces and printing all keep
working without new cases. `Box<int>` and `Box<string>` are two classes that
happen to come from one declaration; nothing at run time can mix them, because
the checker never lets a `Box<string>` where a `Box<int>` is expected (E0201).

Instantiation depth is bounded by the program: `Box<Box<int>>` is a third class,
`[Box<Box<int>>]` a fourth. There is no recursion through a type argument unless
the program writes one, and a self-referential instantiation (`class Tree<T> {
    child: Tree<T>? }`) resolves to the *same* name and terminates.

## 5. Interfaces and printing

- `class Stack<T> implements Container` instantiates the interface methods per
  instance, with the same field-name mangling; the interface itself stays
  non-generic (`Container` methods take `string`, or the class adapts).
  Generic interfaces are **not** part of this ADR.
- `print(box)` prints like any instance, using the mangled name's **source**
  name and field names: `Box(value: 1)`. The mangling is an implementation
  detail and must not leak into output; ADR 0031's collection printing covers
  `[Box(value: 1), Box(value: 2)]`.

## 6. Limits (deliberately not implemented here)

- **No generic methods.** A method may use its class's parameters only. S2's
  containers do not need more, and generic methods raise the same inference
  questions as generic functions plus a receiver.
- **No constraints/traits.** A type parameter can be used where every type is
  allowed; a call on a `T` value is an error. This is ADR 0030's position,
  unchanged.
- **No explicit type arguments** at constructors or in `new`-style positions.
- **No generic interfaces, no variance, no wildcards.**
- **No generic class as a value** without an instantiation (`f = Box`), same
  reason as E0251 for functions: there is no type to give `f`.
- **`none` and payload-less enums** cannot bind a type parameter on their own:
  `Box(value: none)` is E0422, since no argument type can be inferred.

## 7. Reserved codes

| code | condition | hint |
|---|---|---|
| `E0421` | wrong number of type arguments in an annotation | "`Box` takes one type argument: `Box<int>`" |
| `E0422` | a type argument cannot be inferred at a constructor | "pass a field that mentions `T`, or use a non-generic class" |
| `E0423` | a generic class used where a concrete one is required | "write `Box<int>`, not `Box`" |

Each lands with its implementation, its `docs/errors/index.md` row and a
`kode explain` page, as rule 2 requires.

## 8. Costs

- **Zero runtime change**: instances are ordinary class instances, so field
  access, allocation and reference counting are exactly today's cost, and no ABI
  version moves.
- **Compile time and binary size grow with the number of distinct instantiations**,
  which is the same trade ADR 0030 accepted for functions. A program that uses
  `Box` with ten element types gets ten classes; that is the price of
  monomorphization and it must be measured for the port's own containers before
  S2 relies on it (AGENTS.md rule 8, `benchmarks/report.md`).
- **Checker complexity**: instance management is shared with generic functions,
  so the new logic is constructor inference and annotation substitution, not a
  second type-variable system.

## 9. Slices (each one branch, one commit set, tests first)

| slice | branch | tests |
|---|---|---|
| 1. syntax, one class, one type argument, constructor inference | `feat/generic-classes` | `tests/run-pass/generic_classes.kode` (+`.out`); `compile-fail/e0421_type_arg_count.kode`, `e0422_cannot_infer_class.kode` |
| 2. annotations (`Box<int>`, `[Box<int>]`, optionals), two instances, methods | `feat/generic-class-annotations` | `tests/run-pass/generic_classes_nested.kode`; `compile-fail/e0423_generic_class_bare.kode` |
| 3. interfaces per instance, printing, `kode fmt` and AST dump | `feat/generic-class-interfaces` | `tests/run-pass/generic_class_implements.kode`; `crates/kode-syntax` dump test |

Each slice ends green under `cargo test --workspace` with `KODE_LEAK_CHECK=1`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `kode fmt --check`.

## 10. Open questions for the maintainer

1. Mangling scheme for instances: `Box.int` (dot, like generic functions today),
   `Box$int`, or `Box_int`? It is visible in `kode mir` and in generic-function
   instance names already, so it should be chosen once for both.
2. Should the constructor accept explicit arguments (`Box<int>(value: none)`) in
   this ADR, or stay inference-only and require a non-empty field to infer from?
3. Generic interfaces and generic methods: separate ADRs after S2 proves the
   container pattern, or fold generic methods into this one now?
4. Do we want a limit on distinct instantiations (compile-time guard) before
   programs can generate thousands accidentally?

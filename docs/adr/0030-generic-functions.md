# ADR 0030: Generic functions and monomorphization

Status: accepted (partial language support: free functions only)

## Context

Kode had no way to write a function once and use it with several types. Lists,
maps and closures already carry type parameters internally (`[T]`, `fn(T) -> U`),
and the standard library needs helpers such as "first item" and "map with a
callback" for every element type. Writing one copy per type is noise, and the
self-hosting effort needs the same abstraction.

Two designs were possible:

1. **Boxed generics.** One compiled function taking an untyped pointer, with type
   information erased. Simple to implement, but it costs an indirection on every
   operation, needs runtime type information, and interacts badly with the
   reference-counting rules (the callee would not know whether to retain).
2. **Monomorphization.** Compile one concrete function per set of type arguments.
   The backend keeps seeing only concrete types, which preserves rule 4 of the
   project (only MIR reaches the backend) and keeps performance equal to
   hand-written code.

Kode chooses monomorphization. The type checker never needs a "type variable"
type: inference happens on the declaration's annotations before a body is
checked, so `Type` stays entirely concrete.

## Decision

A free function may declare type parameters after its name:

```
fn first<T>(xs: [T]) -> T {
    return xs[0]
}
```

Type arguments are inferred at the call site from the argument types, in
declaration order:

- each declared parameter type is unified with the argument's type, binding type
  parameters as they are met;
- a parameter that appears twice must receive the same type (`pair(1, "a")` is
  E0208);
- every type parameter must be bound by a parameter type, otherwise the call is
  E0250.

An instantiation is produced once per `(generic function, concrete types)`:

1. clone the declaration;
2. give every expression in the clone a fresh `ExprId` (the checker's tables are
   indexed by id, and clones outlive `Program::expr_count`);
3. replace every type parameter in the clone's annotations with the concrete
   type;
4. resolve the now-concrete parameter and return types into an ordinary `FnSig`
   with a mangled name (`first.int`, `apply_all.str_int`);
5. append the clone to `Checked::bodies`, which is aligned with `Checked::fns`
   and is what MIR lowering iterates.

Step 5 is how recursion and generic-calls-generic terminate: the instance is
recorded *before* its body is checked, so a recursive call resolves to the same
index. Body checking is a resumable pass (`Checker::check_bodies`), run after
collection, after field defaults, and after the top-level script; a call from any
of those places appends an instance that the same pass then checks. Checking a
clone mid-body would destroy the caller's scope stack, so instantiation only
records work and never re-enters the checker.

Because annotations are substituted before checking, no new type rule is needed:
an instantiated body is checked exactly like a hand-written concrete function,
and MIR lowering never sees a type parameter.

## Limits (deliberately not implemented yet)

- Only free functions. A method with type parameters is E0252; generic classes
  and generic methods wait for a design that also covers field layout.
- No explicit type arguments (`first::<int>`).
- No constraints/traits: a type parameter can only be used where every type is
  allowed. Element access, comparison, `map`/`filter` callbacks and calls to
  other generic functions work because they are checked per instantiation.
- A generic function cannot be used as a value (`f = first`): there is no type to
  give `f` without a call. Wrap a concrete call in a closure instead (E0251).
- Inference does not look at the expected result type, so a type parameter that
  only appears in the return type cannot be inferred (E0250).
- Instances are unbounded: `many` distinct type arguments produce `many`
  functions. Fine for a compiled, statically typed language; a size limit can be
  added if a program ever needs one.

## Consequences

- `Program::fn_decls()` is no longer the list of compiled functions. Later stages
  read `Checked::bodies`, which starts with the declared functions and methods and
  appends instantiations. `Checked::generic_calls` records the instantiation each
  call expression resolved to, so lowering emits a direct call.
- Diagnostics for an instantiated body are reported against the generic
  declaration's source spans, which is where the mistake is written.
- Tests: `tests/run-pass/generics.kode`, `tests/run-pass/generics_nested.kode`,
  and the compile-fail cases `e0250_generic_cannot_infer`,
  `e0251_generic_value`, `e0252_generic_method`, `e0208_generic_mismatch`.

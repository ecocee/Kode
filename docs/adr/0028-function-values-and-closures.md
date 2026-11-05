# ADR 0028: Function values and closures

Status: accepted. 2026-11-05.

## Context

Until now a function name was a value only where the compiler expected one:
`spawn work(1)`, `serve(port, handler)`, a route table, an HTTP handler. Every
other place needed a dedicated built-in. `map` and `filter` were blocked on this
and listed as "not yet" in the collections chapter: a list could be walked with
`for`, but the loop had to be written out every time.

ADR 0018 recorded "there are no function values or closures" as a simplifying
assumption for the web layer. That assumption is what this ADR removes.

## Decision

### One function type

`fn(int) -> int` is a type like `[int]`. `fn()` takes no arguments. A named
function and a closure have the same type, so either can be passed, returned,
assigned and stored:

```
fn double(n: int) -> int => n * 2
fn apply(f: fn(int) -> int, n: int) -> int => f(n)

f = double
print(apply(double, 5))          # 10
print(apply(|x: int| x * x, 7))  # 49
```

A closure is written `|x: int| x * 2` and has an expression body. Its parameters
**must** be annotated (E0241): Kode never infers a type across a closure
boundary, and an untyped parameter would make the error appear at the call site
instead of the definition.

### Representation: an ordinary heap object

A closure is allocated by the existing `kode_obj_new`:

```
[rc][kind][refmask][code address][capture 0][capture 1]...
  0    8      16          24           32
```

- Field 0 is the address of the closure body.
- The captures follow it as normal object fields, and the object's `refmask`
  makes `drop_obj` release captured references exactly as it releases class
  fields. No new object kind, no new runtime function, no new drop logic.
- The closure is reference counted like any other heap value, so the MIR
  ownership rules apply to it unchanged.

### One calling convention

A closure body is a compiled function whose **first parameter is the closure
object itself**. Captures are read from it (`load closure + 32`) instead of from
a separate environment pointer. The declared parameters follow.

A named function used as a value gets a generated wrapper with the same shape:
it ignores parameter 0 and calls the real function. Wrappers are created once per
function.

`CallIndirect` loads the code address from offset 24 and calls it with the object
plus the arguments. Parameter types come from MIR, so `bool` stays one byte,
`float` stays `f64` and references stay pointers: there is no boxing and no
generic ABI.

### Capture is by value, and v1 captures everything

A closure captures **every variable visible where it is written**, except
optional numbers and bools (E0219). There is no free-variable analysis.

This is deliberately blunt. It cannot miss a name that a later refactor
introduces inside the body, the closure's lifetime rules stay identical to a
class instance's, and the compiler needs no new analysis pass. The cost is that
an unused variable is retained for as long as the closure lives; that is visible
in a memory profile but never in behaviour.

Nested closures need no special case: the outer closure's captures are ordinary
locals in its body, so the inner closure captures them from there.

Rejected alternatives:

- **Capture by reference (cells).** It would change what assignment does inside a
  closure, need a new heap cell and a retain for every captured variable, and
  make `x = 1` inside a closure visible outside it. Capture by value is the rule
  in the spec and matches the rest of the memory model.
- **A separate environment object and an extra ABI parameter.** It needs a second
  allocation per closure and a second object kind to free; reusing the object's
  own field area costs nothing.
- **Runtime `map`/`filter`.** A Rust runtime function cannot call back into
  generated code without a callback ABI. The loop is compiled inline instead.
- **Inferring closure parameter types from the expected type.** It works for
  `xs.map(` but not for `f = |x| x + 1`, and would need bidirectional checking
  threaded through every expression position.

### Two restrictions, both temporary

- A closure cannot read a field of `this` (E0242): the calling convention has no
  `this`. Copy the field into a variable first. Methods of `this` are likewise
  not reachable.
- A function value is called by name (`f(1)` where `f` is a variable), not on an
  arbitrary expression: `(make())(1)` does not parse yet.

## Consequences

- `map` and `filter` are available on lists (`[int]`, `[float]`, `[string]`,
  `[bool]` and reference types) and are lowered to one loop with one result
  allocation. `filter` retains the items it keeps.
- `E0241` (closure parameter needs a type) and `E0242` (bad function value or
  `map`/`filter` callback) join the error table.
- New MIR: `Inst::CallIndirect`, `Operand::ClosureFn`/`FnVal`,
  `MirProgram::closures`/`fnvals`. `kode mir` shows `call _1 with (...)`.
- The LSP walker visits closure bodies, so hover and go-to-definition work inside
  them.
- This ADR supersedes the "no function values" assumptions noted in ADRs 0018
  and 0020; their calling conventions for `serve` and routes are unchanged.

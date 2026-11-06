---
title: Generic functions
---

A Kode function can declare type parameters after its name:

```kode
fn first<T>(xs: [T]) -> T {
    return xs[0]
}

fn pair<T>(a: T, b: T) -> [T] => [a, b]

fn apply_all<T, U>(xs: [T], f: fn(T) -> U) -> [U] => xs.map(f)

print(first([10, 20, 30]))                        # 10
print(first(["a", "b"]))                          # a
print(pair(1, 2)[1])                              # 2
```

## How type arguments are inferred

Kode infers the type arguments at each call from the argument types. Each declared
parameter type is unified with the type of the corresponding argument, binding type
parameters in declaration order. A parameter that appears twice must receive the same
type: `pair(1, "a")` is error E0208 because `T` is first `int` and then `string`.

- Every type parameter must appear in a parameter type. If it does not, the call is
  error E0250 ("cannot infer the type parameter `T`").
- A generic function is not a value. The expression `f = first` is error E0251
  ("a generic function cannot be used as a value"). To store one, wrap a concrete
  call in a closure.
- A method with type parameters is error E0252 ("a method cannot have type parameters
  yet").

## Monomorphization

Kode compiles each distinct combination of type arguments into its own native function.
Two calls with the same concrete types share one compiled function.

```kode
fn identity<T>(x: T) -> T {
    return x
}

print(identity(41) + 1)     # identity.int
print(identity("hi"))       # identity.str
```

The compiler never re-enters the type checker for an instantiation. At a call site,
it clones the generic function, gives every expression a fresh id, replaces every type
parameter in the clone's annotations with the concrete type, resolves the parameter and
return types into an ordinary function signature, and appends the clone to the list of
bodies that the checker then checks and the compiler lowers. That keeps the type system
unchanged and makes recursion and generic-calls-generic work naturally: the instance is
registered before its body is checked, so a recursive call resolves to the same index.

## Generics and the built-in types

The built-in collection types are written with type arguments and work with generic
functions. They are built into the checker rather than being first-class generic types
that a user can define yet.

```kode
fn keep<T>(xs: [T], f: fn(T) -> bool) -> [T] => xs.filter(f)
fn count_of<T>(xs: [T], x: T) -> int {
    n = 0
    for v in xs {
        if v == x { n = n + 1 }
    }
    return n
}

print(keep([1, 2, 3, 4], |x: int| x % 2 == 0).len())  # 2
print(count_of([1, 2, 2, 3], 2))                       # 2
```

## What is not implemented yet

- Generic classes and generic methods.
- Explicit type arguments (`first::<int>`).
- Constraints/traits. A parameter can only be used where every type is allowed; the
  per-instantiation checking is what makes `map`/`filter` callbacks and element access
  type-safe.
- User-defined generic types. The built-in `[T]`, `{string: T}`, `Task<T>` and
  `Channel<T>` are used with generic functions but are not themselves generic types a
  user can declare.

## Examples

```kode
fn twice<T>(f: fn(T) -> T, x: T) -> T => f(f(x))

print(twice(|x: int| x + 1, 5))      # 7
print(twice(|s: string| s + "!", "hi"))  # hi!!
```

Two type parameters in one function work the same way: inference walks the
declaration in order and binds both. The return type can be one of the parameters
(two, here), because each is already bound by a parameter type.

## Errors

| Code | When |
|---|---|
| E0250 | A type parameter cannot be inferred from the call. |
| E0251 | A generic function was used as a value. |
| E0252 | A method was given type parameters. |

Two existing error codes cover the other cases: E0207 when the call has the wrong
number of arguments, and E0208 when a parameter that is used more than once would get
two different types. These are the same codes used for non-generic calls, because the
checker checks an instantiated body with exactly the same rules as any other function.

# 2. Types, variables, constants

| Type | Meaning |
|---|---|
| `int` | 64-bit signed integer. `+ - *` wrap on overflow for now. |
| `float` | 64-bit IEEE decimal. Prints like `3.0`, `0.5`. |
| `bool` | `true` or `false` |
| `string` | immutable UTF-8 text, reference counted |

Numbers never convert automatically. Use `int(x)` and `float(x)`; mixing them is E0210.

## Variables

```
name = "Asha"          # the first assignment declares it; the type is fixed
age: int = 30          # with an explicit type
age = age + 1          # assigning again changes it
age += 1               # also -=  *=  /=  %=
```

- **Block scope.** A variable lives until the end of the block (`{ }`) where it was first assigned.
  Assigning to a name from an outer block changes that variable; it does not create a new one.
- A variable keeps the type of its first value (E0202).
- Unused variables give warning W0001. Names starting with `_` are exempt.
- A `for` loop variable belongs to the loop.

## Constants

```
const LIMIT = 100
const PI = 3.14159
```

A constant is a plain literal value, can be used anywhere, and cannot be changed (E0312).

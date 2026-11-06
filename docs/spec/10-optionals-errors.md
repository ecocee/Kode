# 10. Optionals and errors

## Optionals

```
class User {
    name: string
    nickname: string? = none
}

u: User? = none
print(u == none)
name = u?.name ?? "guest"       # ?. stops at none, ?? gives a default
first = u!                       # stop with an error if it is none
```

- `T?` can hold a value of type `T` or `none`. It works for `string`, lists, maps, classes,
  and for `int`, `float` and `bool`.
- `int?`, `float?` and `bool?` are stored as a pair (the value and a `present` flag): no boxing
  and no allocation, one byte of extra storage. They work as a **local variable**; a parameter,
  return type, field, or a list or map of them is still E0219.
- A value that may be none cannot be used with `.` or passed where `T` is needed (E0220).
  Use `?.`, `??` or `!`.
- Narrowing: after `if x != none { ... }`, inside `while x != none { ... }`, and in the code
  after `if x == none { return }` (or `throw`), `x` has type `T` again. Narrowing ends at the
  end of the block, or at any assignment to `x`.
- `?.` works on fields whose type is a heap type (string, list, map, class).
- `"${x}"` and `print(x)` on an optional number write the number or `none`.

```
n: int? = none
n = 5
print(n ?? 0, n == none, n! + 1)
if n != none {
    print(n * 2)          # narrowed to int inside the branch
}
```

## Errors

```
fn parse(s: string) -> int {
    if s == "bad" {
        throw "not a number: ${s}"                # short for Error(message: ...)
    }
    if s == "neg" {
        throw Error(message: "negative", code: 7)
    }
    return 42
}

v = parse("bad") catch e {          # handle one expression
    print(e.message)
    -1                              # the value of the catch block
}

try {                               # handle a block
    parse("bad")
} catch err {
    print(err.message, err.code)
} finally {
    print("done")                  # runs on both paths
}
```

- An error that is not caught leaves the function and goes to its caller.
  At the top of the program it prints `kode: error: message` and exits with code 1.
- The block after `catch` must end with a value (when the call has one), or with `return`,
  `break`, `continue` or `throw` (E0227).
- `assert(cond)` throws an `Error("assertion failed at line N")`.
- Index and key errors, division by zero and `!` on none stop the program; they are not catchable yet.
- `catch` can be left out when you only need cleanup: `try { use(f) } finally { close(f) }`
  lets the error travel on after `finally` runs.
- `finally` runs when the block finishes, when it throws, when its handler throws again,
  and before a `return`, `break` or `continue` that leaves the block. It cannot swallow an
  error: a caught error still leaves the function.

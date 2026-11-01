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

- `T?` can hold a value of type `T` or `none`. It works for `string`, lists, maps and classes.
  `int?`, `float?` and `bool?` are not supported yet (E0219).
- A value that may be none cannot be used with `.` or passed where `T` is needed (E0220).
  Use `?.`, `??` or `!`.
- `?.` works on fields whose type is a heap type (string, list, map, class).
- Not yet: narrowing after `if x != none`. Use `x!` or `??`.

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
}
```

- An error that is not caught leaves the function and goes to its caller.
  At the top of the program it prints `kode: error: message` and exits with code 1.
- The block after `catch` must end with a value (when the call has one), or with `return`,
  `break`, `continue` or `throw` (E0227).
- `assert(cond)` throws an `Error("assertion failed at line N")`.
- Index and key errors, division by zero and `!` on none stop the program; they are not catchable yet.
- Not yet: `finally`, custom error classes.

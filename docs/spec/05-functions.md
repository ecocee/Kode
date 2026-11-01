# 5. Functions

```
fn add(a: int, b: int) -> int {
    return a + b
}

fn square(x: int) -> int => x * x      # one expression

fn log(msg: string) {                  # returns nothing: no `->`
    print("log:", msg)
}
```

- Functions are defined at the top level only and may be used before their definition.
- Parameters must have types. **The return type must be written when the function returns a value.**
  This is a deliberate rule: the signature is the contract, which also helps AI agents (ADR 0007).
- Every path of a value-returning function must `return` (E0305).
- Recursion and mutual recursion work.
- Arguments can be named: `greet(name: "Asha")`. Positional arguments come first.
- A function that can fail (`throw`, or a call to one that can) just propagates errors to its caller.
- Built-in functions: `print(...)`, `len(string | list | map)`, `int(x)`, `float(x)`, `string(x)`, `assert(cond)`.
  They cannot be redefined (E0302).
- `print` takes any number of values, separates them with a space and ends with a new line.

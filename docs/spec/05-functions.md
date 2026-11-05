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

## Function values and closures

```
fn double(n: int) -> int => n * 2

fn apply(f: fn(int) -> int, n: int) -> int {
    return f(n)
}

f = double
print(f(21))                 # 42
print(apply(double, 5))      # 10

add = |x: int| x + 10        # a closure
print(add(5))                # 15
print(apply(|x: int| x * x, 7))   # 49
```

- A function type is written `fn(int) -> int`; `fn()` takes no arguments.
- A closure `|x: int| x * 2` has an expression body. Its parameters must have
  types (E0241): nothing is inferred across a closure boundary.
- A closure captures **by value** every variable visible where it is written, so
  changing a variable afterwards does not change the closure. Optional numbers
  and bools cannot be captured yet (E0219).
- A function value is called by name: `f(1)` where `f` is a variable. It cannot
  be called on an expression such as `make()(1)`.
- Inside a closure, a field of `this` is not visible (E0242); copy it into a
  variable before the closure and capture that.
- Named functions and closures have the same type, so either can be passed,
  returned or stored. A function value cannot be printed (E0203).
- ADR 0028 records the representation: one heap object holding the code address
  and the captures.

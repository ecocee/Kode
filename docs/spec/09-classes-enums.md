# 9. Classes, enums and match

```
class Account {
    owner: string
    balance: int = 0                 # default value
    tags: [string] = []

    fn deposit(amount: int) {
        balance += amount            # fields are used by their plain name
    }

    fn summary() -> string => "${owner}: ${balance}"
}

acct = Account(owner: "Asha")        # named arguments
b = Account("Maya", 7)               # positional also works, in field order
acct.deposit(50)
acct.balance = 10
print(acct.summary())
other = acct                         # the same object, not a copy
```

- A class is a reference type: `other = acct` shares one object. Objects are freed when the last reference goes.
- A field without a default must be given when the object is created (E0216).
- Inside a method a field is used by its plain name, and sibling methods are called by plain name.
  A parameter may not have the same name as a field (E0228).
- Named arguments work for functions, methods and constructors. Positional ones come first (E0215).
- No inheritance (ADR 0006). Use composition: a class holds other classes.
- A class may declare type parameters, `class Box<T>`, and an annotation may
  pass them, `Box<int>` (ADR 0040). Both parse today; instantiating one is still
  `E0424`, so there is no generic class at run time yet.
- Equality `==` on class values compares identity.
- Built-in class `Error` has `message: string` and `code: int = 0`.

## Enums and match

```
enum Color { Red, Green, Blue }

c = Color.Green
print(c)                     # Color.Green
print("c is ${c}")           # c is Color.Green
match c {
    Color.Red => print("red")
    Color.Green => {
        print("green")
    }
    _ => print("other")
}
```

- `match` works on `int`, `string`, `bool` and enums. It must cover every case or end with `_ =>` (E0224).
- Enum values are compared with `==`: they are plain ints with a name.
- A variant may declare fields, `Circle(radius: float)`, which is the syntax
  payload enums will use (ADR 0039). The parser and `kode ast` accept it today;
  constructing one is still `E0253`, because payloads have no representation
  yet.
- `print(c)` and `"${c}"` write the value the way it is written in source,
  `Color.Green`. The text is chosen from static strings, so printing allocates
  nothing and the representation does not change (ADR 0027).
- `match` is a statement, not an expression.

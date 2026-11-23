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
- A class may declare type parameters and an annotation may pass them:
  `class Box<T> { value: T }`, then `Box(value: 1)` is a `Box<int>` (ADR 0040).
  One class is compiled per set of type arguments, inferred at the constructor;
  `Box<int>` and `Box<string>` are different types (E0201 if mixed). A method
  may use the class's parameters but cannot declare its own yet (E0252), and
  `Box` without arguments is E0423.
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
- A tag-only enum value is compared with `==`: it is a plain int with a name.
- A variant may declare fields, and `Shape.Circle(radius: 1.5)` builds it
  (ADR 0039). **One rule decides the representation:** if every variant has no
  fields the enum stays an `int` with no allocation; if any variant has fields,
  every value of that enum is a heap object with the tag in the object header.
- `print(s)` writes `Shape.Circle(radius: 1.5)`, field names included, for
  `int`, `float`, `bool` and `string` fields; a payload enum with any other field
  type is not printable yet (E0203).
- `match` on a payload enum compares the variant tag, not the pointer, and a
  pattern binds the fields in declaration order: `Shape.Rect(w, h)`. The names
  are ordinary locals inside the arm. Binding the wrong number of names is E0207,
  a non-name in a binding position is E0201, and a name bound twice is E0306.
- A method may declare its own type parameters, and is compiled once per set of
  type arguments at the call: `fn id<T>(x: T) -> T => x` inside `class Util` gives
  `Util.id.int` and `Util.id.str`, each an ordinary method with `this` first, so
  it can read fields and its own parameters in one body. A generic class's methods
  cannot declare their own parameters yet (E0252).
- `s is Shape.Circle` is a bool: one tag compare, also usable in `and`/`not`.
  Inside `if s is Shape.Circle { ... }` the value is narrowed to that variant, so
  `s.radius` may be read there; reading it without a narrowing is E0419, not a
  runtime check. `is` works on tag-only enums too.
- `==` on a payload enum is E0420: it would compare two heap addresses. Use the
  variant, or compare the fields.
- `print(c)` and `"${c}"` write the value the way it is written in source,
  `Color.Green`. The text is chosen from static strings, so printing allocates
  nothing and the representation does not change (ADR 0027).
- `match` is a statement, not an expression.

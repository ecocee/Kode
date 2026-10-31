# 3. Expressions

From highest to lowest precedence:

| Level | Operators |
|---|---|
| 1 | call `f(x)`, parentheses |
| 2 | unary `-` |
| 3 | `*  /  %` |
| 4 | `+  -` |
| 5 | `<  <=  >  >=  ==  !=` |
| 6 | `not` |
| 7 | `and` |
| 8 | `or` |

- `int / int` truncates toward zero. `int % int` keeps the sign of the left side.
  Dividing by zero stops the program with an error message (constant zero is a compile error, E0204).
- `float` supports `+ - * /` and the comparisons. `%` is for ints only.
- `+` also joins two strings. `==` and `!=` work on all four types. `< <= > >=` work on numbers.
- `and` and `or` stop early: the right side runs only when needed.
- Conditions must be `bool`. There is no truthy or falsy (E0209).
- `if` is a statement in v0.1, not an expression.

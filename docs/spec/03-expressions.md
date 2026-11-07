# 3. Expressions

From highest to lowest precedence:

| Level | Operators |
|---|---|
| 1 | call `f(x)`, parentheses |
| 2 | unary `-  ~` |
| 3 | `*  /  %` |
| 4 | `+  -` |
| 5 | `<<  >>` |
| 6 | `<  <=  >  >=  ==  !=  in` |
| 7 | `not` |
| 8 | `&` |
| 9 | `^` |
| 10 | `|` |
| 11 | `and` |
| 12 | `or` |

- `int / int` truncates toward zero. `int % int` keeps the sign of the left side.
  Dividing by zero stops the program with an error message (constant zero is a compile error, E0204).
- `float` supports `+ - * /` and the comparisons. `%` is for ints only.
- `+` also joins two strings. `==` and `!=` work on all four types. `< <= > >=` work on numbers.
- `&  |  ^  ~  <<  >>` work on ints only. `~x` is `-x - 1`; `>>` keeps the sign.
  A shift by 64 or more, or by a negative amount, gives `0`.
- `x &= y`, `x |= y`, `x ^= y`, `x <<= y` and `x >>= y` are shorthand for the matching
  binary assignment.
- `and` and `or` stop early: the right side runs only when needed.
- Conditions must be `bool`. There is no truthy or falsy (E0209).
- `if` is a statement in v0.1, not an expression.

## Nesting limit

An expression or block may nest at most 128 levels deep (ADR 0032). Depth is
counted the way the compiler walks the tree: one level per nested expression,
statement or block, and one level per operand of a left-to-right chain, so
`a + b + c` is two levels and 129 chained `+` operators are refused. Deeper
input is `E0118` with a hint to split it, not a compiler crash.

```
# 129 terms in one expression: E0118
x: int = 1 + 1 + ...

# the same value, accepted
total = 0
for i in 0..129 {
    total = total + 1
}
```

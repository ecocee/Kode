# 4. Statements

```
if score >= 90 {
    grade = "A"
} else if score >= 75 {
    grade = "B"
} else {
    grade = "C"
}

while n < 10 {
    n += 1
}

for i in 0..5 { print(i) }        # 0 1 2 3 4
for i in 0..=5 { print(i) }       # 0 to 5
```

- `else` must be on the same line as the closing `}`.
- A range end is evaluated once. An empty range (`5..2`) runs zero times.
- `break` leaves the nearest loop. `continue` jumps to the next round (E0314 outside loops).
- Braces are required. Parentheses around conditions are not.
- Top-level statements run in order, like a script.

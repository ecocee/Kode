# 8. Lists and maps

```
xs = [3, 1, 2]                 # [int]
xs.add(10)                     # append
print(xs[0], xs.len(), len(xs))
xs[1] = 99                     # change an item

names: [string] = []           # an empty list needs a declared type

ages = {"asha": 30, "ravi": 25}    # {string: int}, keys are always strings
ages["maya"] = 28
ages["asha"] += 1
print("ravi" in ages, ages.has("zed"), ages.len())
for k, v in ages { print(k, v) }   # insertion order
for k in ages { print(k) }
ks = ages.keys()                   # a new [string]

for x in xs { print(x) }           # items
for i, x in xs { print(i, x) }     # index and item
```

- Item types are `int`, `float`, `bool`, `string`, classes, and other lists and maps.
  All items must have the same type (E0213).
- Index out of range, and a missing map key, stop the program with an error that names the line.
- Lists and maps are reference counted like strings; `b = a` shares the same list.
- `xs.map(f)` returns a new list holding the results; `xs.filter(f)` keeps the
  items where `f` returns true. The function takes one item:

```
xs = [1, 2, 3, 4]
doubled = xs.map(|x: int| x * 2)        # [2, 4, 6, 8]
evens = xs.filter(|x: int| x % 2 == 0)  # [2, 4]
```

- The function must take one item of the list's type and return a value (E0242);
  the function given to `filter` must return a `bool`.
- Not yet: integer map keys, printing a whole list.

## Slices

```
xs = [10, 20, 30, 40, 50]
print(xs[1..3].len())       # 2: items 1 and 2
print(xs[..2].len())        # 2: from the start
print(xs[3..].len())        # 2: to the end
print(xs[0..=2].len())      # 3: includes index 2
print(xs[10..20].len())     # 0: bounds are clamped

part = xs[1..3]
part.add(99)
print(xs.len())             # 5: a slice is a copy, not a view
```

- `xs[a..b]` is a **new list** holding the items in `[a, b)`. `xs[a..=b]` includes
  `b`. Either bound may be left out.
- Bounds are clamped to `[0, len]`: a bound past the end stops at the end, a
  negative bound counts from the start (it becomes 0), and `a >= b` is empty.
  Indexing is still strict, because an index mistake is usually a bug.
- Only a list can be sliced (E0211), and the bounds must be ints (E0212).
- Items are retained when copied, so a slice of strings owns its references. A
  slice of `[bool]` stays packed, one byte per item.
- ADR 0027 records the decision and the rejected view and negative-index designs.

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
- Not yet: slicing, removing items, sorting, `map`/`filter`, integer map keys, printing a whole list.

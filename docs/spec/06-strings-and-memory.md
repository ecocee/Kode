# 6. Strings and memory

Strings are immutable and reference counted. There is no garbage collector and no manual freeing.

```
s = "Hello" + ", " + name           # new string
t = "${s} (${len(s)} chars)"         # interpolation
```

- `len(s)` counts characters, not bytes.
- A string is freed as soon as the last variable that holds it is re-assigned or its function ends.
- Function parameters are borrowed, so passing a string never copies it.
- Run any program with `KODE_LEAK_CHECK=1` to make the runtime fail when a string was not freed.
  The test suite does this for every run-pass program.

Lists, maps and class instances follow the same rules. Design details: `docs/design/memory-model.md`.

# 11. Modules, tests and tools

## Imports

```
import "./util"                       # all items of util.kode
import "models/user" as users         # package import with a qualified name
from "./models/user" import User, make_user
```

- `./` and `../` paths are relative to the importing file; `.kode` is added.
- With a `kode.toml` at or above the entry file, a path without `./` is relative to the
  package source root:

```
[package]
name = "shop"
version = "0.1.0"
root = "src"
```

  `import "models/user"` reads `src/models/user.kode`. A missing root is E0407.
- `import "x" as name` adds a qualified name: `name.add(...)`, `name.TAX`,
  `name.User(...)`. An alias is not a value; it lives in the shared namespace like a
  declaration, so it may not hide one (E0409) or be declared twice (E0408).
- Only `pub` items are visible through an alias and through `from ... import` (E0411).
  A missing name is E0410 through an alias, E0405 in a `from` list.
- A module may contain only `fn`, `const`, `class`, `interface`, `enum` and imports.
  Top-level code in a module is an error (E0403).
- All files still share one namespace: `import "./util"` merges every item, and an
  unqualified name can reach a non-`pub` item. `pub` gates qualified and `from` access,
  not the shared namespace yet. A name defined twice is an error (E0301).
- Cycles are an error (E0402). A file imported twice is loaded once.

## Tests

```
test "add works" {
    assert(add(2, 3) == 5)
}
```

`kode test file.kode` (or a folder) runs every `test` block and prints `PASS name` or
`FAIL name: message`, then `N passed, M failed`. The exit code is 1 when a test fails.
A failing `assert` stops only its own test. Top-level code is not run in test mode.

## Tools

| Command | What it does |
|---|---|
| `kode fmt <files or folders>` | fix indentation and blank lines; `--check` only reports |
| `kode explain E0201` | longer explanation of an error code with an example |
| `kode check --format=json` | structured errors for editors and AI agents |
| `kode mir file.kode` | show the compiler's middle layer |

`kode fmt` only changes whitespace at the start and end of lines and collapses blank lines. It never changes what a program means and keeps all comments.

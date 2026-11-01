# 11. Modules, tests and tools

## Imports

```
import "./util"                       # all items of util.kode
from "./models/user" import User, make_user
```

- The path is relative to the importing file; `.kode` is added.
- A module may contain only `fn`, `const`, `class`, `enum` (and imports).
  Top-level code in a module is an error (E0403).
- In v0.2 **all files share one namespace**. A name defined twice is an error (E0301).
  `pub` is accepted for the future but not enforced yet. A `from ... import` list is checked
  (E0405) but does not hide other names.
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

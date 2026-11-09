# Architecture

```
          kode-diagnostics  (Span, Diagnostic: shared by all stages)
                 |
source -> kode-syntax -> kode-sema -> kode-mir -> kode-llvm -> clang -> object file -> kode-link -> binary
          lex, parse     types        CFG          LLVM IR -> clang               + kode-rt
                                                                     
kode-cli drives the pipeline (and loads imported files into one program, formats code, runs tests,
explains errors).   kode-rt is the runtime library every binary links against.
```

| Crate | Input | Output |
|---|---|---|
| `kode-syntax` | text | `Program` (AST) |
| `kode-sema` | AST | `Checked`: type of every expression, function signatures, constants, warnings |
| `kode-mir` | AST + `Checked` | `MirProgram` |
| `kode-llvm` | `MirProgram` | LLVM IR, then a native object file |
| `kode-link` | object file | executable |
| `kode-rt` | | static library with print, strings, runtime errors |

Rules:

1. Each crate depends only on the ones before it. Replacing the backend never touches the parser.
2. `kode-llvm` reads only MIR.
3. Errors are data (`Diagnostic` with a code), never panics.
4. Generated names: user function `f` becomes `kode_fn_f`, so it can never clash with a C library symbol.

## Adding a feature

1. Write the test program (`tests/run-pass` or `tests/compile-fail`).
2. Token and lexer, AST node, parser rule.
3. Type rules in `kode-sema`, with error codes.
4. Lowering in `kode-mir` (and the ownership rules if it creates strings or objects).
5. Backend support only if MIR gained a new instruction.

# Architecture

```
          kode-diagnostics  (Span, Diagnostic: shared by all stages)
                 |
source -> kode-syntax -> kode-sema -> kode-mir -> kode-codegen -> object file -> cc -> binary
          lex, parse     types        CFG          Cranelift                      + kode-rt
                                                                     
kode-cli drives the pipeline.   kode-rt is the runtime library every binary links against.
```

| Crate | Input | Output |
|---|---|---|
| `kode-syntax` | text | `Program` (AST) |
| `kode-sema` | AST | `Checked`: type of every expression, function signatures, constants, warnings |
| `kode-mir` | AST + `Checked` | `MirProgram` |
| `kode-codegen` | `MirProgram` | native object file, then an executable |
| `kode-rt` | | static library with print, strings, runtime errors |

Rules:

1. Each crate depends only on the ones before it. Replacing the backend never touches the parser.
2. `kode-codegen` reads only MIR.
3. Errors are data (`Diagnostic` with a code), never panics.
4. Generated names: user function `f` becomes `kode_fn_f`, so it can never clash with a C library symbol.

## Adding a feature

1. Write the test program (`tests/run-pass` or `tests/compile-fail`).
2. Token and lexer, AST node, parser rule.
3. Type rules in `kode-sema`, with error codes.
4. Lowering in `kode-mir` (and the ownership rules if it creates strings or objects).
5. Backend support only if MIR gained a new instruction.

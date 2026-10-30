# Kode — Project Status

**Updated:** March 5, 2026
**Version:** 0.3.4
**Codebase:** ~8,400 lines of Go across 21 source files

---

## Pipeline

```
Source (.kode)
  → Lexer        (internal/lexer/lexer.go        — 643 lines)
  → Parser       (internal/parser/parser.go       — 1810 lines)
  → AST          (pkg/ast/ast.go                  — 640 lines)
  → Typer        (internal/typer/typer.go         — 1000 lines)
  → IR           (pkg/ir/ir.go                    — 117 lines)
  → IR Compiler  (internal/compiler/compiler.go   — 255 lines)
  → BC Compiler  (pkg/bytecode/compiler.go        — 1817 lines)
  → VM           (pkg/bytecode/vm.go              — 2045 lines)
  → .kbc file    (pkg/bytecode/bytecode.go        — 458 lines)
```

---

## Test Status

| Package | Result |
|---------|--------|
| `internal/lexer` | ✅ PASS |
| `internal/parser` | ✅ PASS |
| `internal/typer` | ✅ PASS |
| `internal/compiler` | ✅ PASS |
| `internal/cli` | ❌ FAIL — `TestBuildCommandProducesBinary` |
| `pkg/bytecode` | — no test files |

---

## Fully Implemented and Working

### Core Language

| Feature | Status | Notes |
|---------|--------|-------|
| `let` variable declarations | ✅ Working | With or without type annotation |
| `const` declarations | ✅ Working | Requires explicit type |
| `int`, `float`, `string`, `bool` primitives | ✅ Working | Full type inference |
| Array type `[]T` | ✅ Working | Homogeneous |
| `nil` literal | ✅ Working | Can be assigned / compared |
| Type annotations (optional) | ✅ Working | Checked by typer, optional at let |

### Operators

| Feature | Status | Notes |
|---------|--------|-------|
| Arithmetic `+` `-` `*` `/` `%` | ✅ Working | int + float mixed |
| Compound assignment `+=` `-=` `*=` `/=` `%=` | ✅ Working | All arithmetic |
| Increment `++` / decrement `--` | ✅ Working | Post-increment/decrement |
| Comparison `==` `!=` `<` `>` `<=` `>=` | ✅ Working | Returns bool |
| Logical `&&` `\|\|` `!` | ✅ Working | Truthy: non-zero int, non-empty string |
| Bitwise `&` `\|` `^` `~` `<<` `>>` | ✅ Working | Integer only |
| String concatenation via `+` | ✅ Working | `"a" + "b"` → `"ab"` |
| String interpolation `"${expr}"` | ✅ Working | Arbitrary expressions |
| Unary negation `-expr` | ✅ Working | Int and float |

### Control Flow

| Feature | Status | Notes |
|---------|--------|-------|
| `if` / `else` | ✅ Working | Nested and chained |
| `for` loop (C-style) | ✅ Working | `for (let i=0; i<n; i++)` |
| `for x in arr` (for-in) | ✅ Working | Array element iteration |
| `while` loop | ✅ Working | `while (cond) { }` |
| `break` | ✅ Working | Correct VM jump target via two-phase patching |
| `continue` | ✅ Working | Jumps to correct increment/condition phase |

### Functions

| Feature | Status | Notes |
|---------|--------|-------|
| Block body `fn f() { }` | ✅ Working | Multi-statement |
| Expression body `fn f() = expr` or `=>` | ✅ Working | Single expression |
| Parameters with/without type annotation | ✅ Working | `fn(x: int)` and `fn(x)` |
| Return type annotation | ✅ Working | Optional |
| `return value` | ✅ Working | Including from nested branches |
| Recursion | ✅ Working | Fully working, stack-limited |
| First-class functions / closures `fn(x) { }` | ✅ Working | `fn` keyword, captures outer variables |
| Higher-order functions | ✅ Working | Functions as arguments and return values |

### Arrays

| Feature | Status | Notes |
|---------|--------|-------|
| Array literal `[1, 2, 3]` | ✅ Working | Homogeneous |
| Array indexing `arr[i]` | ✅ Working | Zero-based |
| Array assignment `arr[i] = v` | ✅ Working | In-place mutation |
| Array `.len` property | ✅ Working | Returns int |
| Array `.push(v)` | ✅ Working | Appends element, returns new array |
| Array `.pop()` | ✅ Working | Removes and returns last element |
| `for x in arr` iteration | ✅ Working | Via for-in loop |

### Structs

| Feature | Status | Notes |
|---------|--------|-------|
| `struct` declaration | ✅ Working | Named fields with types |
| Struct instantiation | ✅ Working | `MyStruct { field: val }` |
| Field read access `.field` | ✅ Working | Via `OpMemberAccess` |
| Field mutation | ❌ Not compiled | v0.4 target |
| `impl` methods | ❌ Not compiled | v0.4 target |

### Enums

| Feature | Status | Notes |
|---------|--------|-------|
| `enum` declaration | ✅ Parsed + compiled | Variants stored |
| Enum variant creation | ✅ VM support | `OpEnumVariant` — creates tagged struct value |
| Enum `::` variant access | ❌ Not in parser | v0.4 target |

### Pattern Matching

| Feature | Status | Notes |
|---------|--------|-------|
| `match` statement | ✅ Working | Full bytecode via `OpDup`/`OpEq`/`OpJmpIfFalse` |
| Literal patterns `42` | ✅ Working | |
| Wildcard `_` | ✅ Working | Always matches |
| Identifier binding `n =>` | ✅ Working | Binds matched value to name |
| Struct/enum destructuring | ❌ Not yet | v0.4 target |

### Error Handling

| Feature | Status | Notes |
|---------|--------|-------|
| `try { } catch (e) { }` | ✅ Working | `OpTryBegin` / `OpTryEnd` / `OpThrow` |
| Division by zero runtime error | ✅ Working | Catchable |
| Modulo by zero runtime error | ✅ Working | Catchable |
| Nil operand arithmetic error | ✅ Working | Catchable |
| Array index out of bounds error | ✅ Working | Catchable |

### Defer

| Feature | Status | Notes |
|---------|--------|-------|
| `defer { }` statement | ✅ Working | LIFO execution on function return |

### Module System

| Feature | Status | Notes |
|---------|--------|-------|
| `import { f, g } from "mod"` | ✅ Working | Named destructured import |
| `import "mod" as alias` | ✅ Working | Namespace alias |
| `import "mod"` | ✅ Working | Simple import (all exported) |
| `export func` | ✅ Working | Export function from module |
| `export const` / `export let` | ✅ Working | Export values from module |
| Module path resolution | ✅ Working | Current dir → `stdlib/` → `examples/` dir |
| `server` stdlib module | ✅ Working | `import { newServer, get, post, … } from "server"` |

### Built-in Functions (VM `callBuiltin`)

| Category | Functions |
|----------|-----------|
| Output | `print(...)`, `println(...)`, `printf(fmt, ...)` |
| Input | `input()` |
| Type | `type(x)`, `int(x)`, `float(x)`, `string(x)`, `bool(x)`, `len(x)` |
| Range | `range(n)`, `range(s,e)`, `range(s,e,step)` |
| Math | `abs`, `sqrt`, `pow`, `floor`, `ceil`, `round`, `min`, `max`, `random`, `math.pi`, `math.e` |
| String | `upper`, `lower`, `trim`, `split`, `contains`, `startsWith`, `endsWith`, `replace`, `indexOf`, `repeat` |
| Array | `sort`, `reverse`, `join`, `has`, `keys`, `values` |
| File I/O | `readFile`, `writeFile`, `appendFile`, `fileExists`, `readDir`, `joinPath` |

String builtins also work as method calls: `s.upper()`, `s.trim()`, `s.split(",")`, etc.

### VM and Compilation

| Feature | Status |
|---------|--------|
| Bytecode compilation to `.kbc` | ✅ Working |
| Bytecode serialization / deserialization | ✅ Working |
| Bytecode execution (`kode file.kbc`) | ✅ Working |
| Stack-based VM with 53 opcodes | ✅ Working |
| Global and local variable scopes | ✅ Working |
| Closure capture via `OpMakeClosure` | ✅ Working |
| Dynamic dispatch `OpCallDynamic` | ✅ Working |
| Method call `OpMethodCall` on receiver | ✅ Working |
| Static type checker with inference | ✅ Working |
| Line comments `//` | ✅ Working |
| Block comments `/* */` (nested-safe) | ✅ Working |
| UTF-8 BOM handling | ✅ Working |

---

## Parsed but Runtime-Limited

| Feature | Behavior | Target |
|---------|----------|--------|
| `async` / `await` | Tokenised + parsed; no runtime semantics | v0.5 |
| `spawn` statement | Parsed; silently skipped at compile time | v0.5 |
| `chan` / `select` | Parsed; silently skipped | v0.5 |
| `trait` declarations | Parsed; type-checker aware only | v0.6 |
| `impl Trait for Type` | Parsed; not compiled | v0.6 |
| `service` / Server routes | ✅ Working — native `server` stdlib | v0.3.4 |
| Walrus `:=` operator | Tokenised; not used in parser | future |

---

## Known Bugs and Limitations

| # | Severity | Issue | Notes |
|---|----------|-------|-------|
| 1 | Medium | `TestBuildCommandProducesBinary` CLI test fails | Test expects specific binary output; command line args changed |
| 2 | Low | Deep recursion causes Go stack overflow | No tail-call optimisation |
| 3 | Low | Struct field mutation not supported | `p.x = 5` is not compiled; v0.4 |
| 4 | Low | Enum `::` variant access missing from parser | `Color::Red` not parsed; v0.4 |
| 5 | Low | `impl` method bodies not compiled | `impl Foo { func bar() {} }` silently skipped |
| 6 | Info | Source map not serialized to `.kbc` | Runtime errors after `kode build` lose line info |
| 7 | Info | `print(nil)` prints empty string silently | Nil printing has no warning |
| 8 | Info | Number type inconsistency: `int` vs `int64` | Lexer emits `int64`, VM uses `int`; handled by coercion |

---

## Roadmap

| Version | Focus | Status |
|---------|-------|--------|
| v0.1.0 | Lexer, parser, basic interpreter | ✅ Done |
| v0.2.0 | Bytecode VM, operators, control flow | ✅ Done |
| v0.3.0 | Arrays, array indexing | ✅ Done |
| v0.3.1 | Module system, structs, bytecode polish | ✅ Done |
| v0.3.2 | Closures, try/catch, defer, string interp, match, 40+ builtins | ✅ Done |
| v0.3.4 | `server` stdlib module, `fn`-only keyword, production stdlib | ✅ Done |
| v0.4.0 | Struct mutation + methods, enum variants, full match patterns | ⏳ Next |
| v0.5.0 | Concurrency — spawn, channels, select | ⏳ Planned |
| v0.6.0 | Standard library, generics, traits | ⏳ Planned |

---

## Recommended Next Steps (Priority Order)

1. **Fix `TestBuildCommandProducesBinary` CLI test** — Update expected behavior in test
2. **Implement enum `::` variant access** — Parser + bytecode (`Color::Red` → `OpEnumVariant`)
3. **Compile `impl` method bodies** — `OpMethodCall` dispatch already in VM; need compiler support
4. **Struct field mutation** — `p.x = 5` → load struct, update field, store back
5. **Full match destructuring** — Enum/struct pattern bindings
6. **Add `pkg/bytecode` test suite** — No tests exist for bytecode compiler + VM
7. **Serialize source map** — Include line info in `.kbc` for runtime error messages

---

## File Index (Current)

| File | Lines | Role |
|------|-------|------|
| `pkg/bytecode/vm.go` | 2045 | Bytecode executor (stack VM) |
| `pkg/bytecode/compiler.go` | 1817 | AST/IR → bytecode instructions |
| `internal/parser/parser.go` | 1810 | Recursive descent parser → AST |
| `internal/typer/typer.go` | 1000 | Static type checker + inference |
| `internal/lexer/lexer.go` | 643 | Tokenizer |
| `pkg/ast/ast.go` | 640 | AST node type definitions |
| `pkg/bytecode/bytecode.go` | 458 | Opcodes, program, serialization |
| `internal/compiler/compiler.go` | 255 | AST → IR (thin pass) |
| `internal/cli/root.go` | 212 | CLI root, direct run + `.kbc` dispatch |
| `pkg/ir/ir.go` | 117 | IR node definitions |
| `internal/cli/doctor.go` | 109 | `kode doctor` command |
| `internal/cli/run.go` | 167 | `kode run` + import resolution |
| `internal/cli/build.go` | 131 | `kode build` command |
| `internal/cli/new.go` | 89 | `kode new` scaffold |
| `internal/version/version.go` | 17 | Version constant (`0.3.4`) |
| `pkg/bytecode/server_module.go` | ~200 | Native Go `server.*` implementation |
| `stdlib/server.kode` | ~430 | Production server standard library |


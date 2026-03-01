# Kode

> A statically typed, compiled programming language built in Go
> Version **0.3.1** — February 2026

[![Version](https://img.shields.io/badge/version-0.3.1-blue)]()
[![Language](https://img.shields.io/badge/written_in-Go-00ADD8)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()

---

## What is Kode?

Kode is a statically typed programming language written in Go. It features a complete lexer, parser, type checker, bytecode compiler, and a tree-walking interpreter. Kode has a clean syntax inspired by Go and a fully functional module system for splitting code across files.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Commands](#cli-commands)
- [Language Syntax](#language-syntax)
  - [Variables](#variables)
  - [Constants](#constants)
  - [Types](#types)
  - [Operators](#operators)
  - [Functions](#functions)
  - [Control Flow](#control-flow)
  - [Arrays](#arrays)
  - [Structs](#structs)
  - [Module System](#module-system)
  - [Built-in Functions](#built-in-functions)
- [Working Examples](#working-examples)
- [Feature Status](#feature-status)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Project Structure](#project-structure)

---

## Installation

### Prerequisites

- **Go 1.21+** is required to build from source.

### Build from Source

```bash
git clone https://github.com/ecocee/kode-go
cd kode-go
go build -o kode ./cmd/kode
```

### Verify

```bash
kode version
# Kode v0.3.1
# Built with Go go1.21.x
# Platform: windows/amd64
```

---

## Quick Start

```bash
# Create a file
echo 'print("Hello, Kode!")' > hello.kode

# Run it
kode hello.kode
# Hello, Kode!

# Compile to bytecode
kode build hello.kode     # produces hello.kbc
kode hello.kbc       # run the bytecode
```

---

## CLI Commands

```
kode    <file.kode>   Run a source file directly
kode build  <file.kode>   Compile to .kbc bytecode
kode   <file.kbc>    Execute compiled bytecode
kode check  <file.kode>   Type-check without running
kode fmt    <file.kode>   Format source files
kode new    <name>        Scaffold a new project
kode clean                Remove build artifacts
kode env                  Show compiler environment info
kode doctor               Diagnose environment issues
kode version              Print version
```

---

## Language Syntax

### Variables

Use `let` to declare a variable. Type annotation is optional — Kode infers types automatically.

```kode
let x = 10
let name = "Kode"
let pi = 3.14
let active = true

// Explicit type annotation
let count: int = 0
let message: string = "hello"
```

Variables are mutable. Reassign with `=`:

```kode
let x = 5
x = x + 1
print(x)    // 6
```

---

### Constants

Constants require an explicit type and cannot be reassigned.

```kode
const MAX: int = 100
const PI: float = 3.14159
const APP_NAME: string = "MyApp"
```

---

### Types

| Type | Example values |
|------|---------------|
| `int` | `0`, `42`, `-7` |
| `float` | `3.14`, `-0.5`, `1.0` |
| `string` | `"hello"`, `"world"` |
| `bool` | `true`, `false` |
| `[]int` | `[1, 2, 3]` |
| `[]string` | `["a", "b", "c"]` |

---

### Operators

#### Arithmetic

```kode
let a = 15
let b = 4

print(a + b)    // 19  — addition
print(a - b)    // 11  — subtraction
print(a * b)    // 60  — multiplication
print(a / b)    // 3   — integer division
print(a % b)    // 3   — modulo
```

#### Comparison

```kode
print(10 == 10)   // true
print(10 != 5)    // true
print(10 > 5)     // true
print(10 < 5)     // false
print(10 >= 10)   // true
print(5 <= 9)     // true
```

#### Logical

```kode
print(true && false)   // false  — AND
print(true || false)   // true   — OR
print(!true)           // false  — NOT

// Integer truthy/falsy (0 is false, non-zero is true)
print(1 && 0)    // false
print(1 || 0)    // true
print(!0)        // true
```

#### Bitwise

```kode
let x = 12   // binary: 1100
let y = 5    // binary: 0101

print(x & y)    // 4    — AND
print(x | y)    // 13   — OR
print(x ^ y)    // 9    — XOR
print(x << 2)   // 48   — left shift
print(x >> 1)   // 6    — right shift
print(~x)       // -13  — NOT
```

#### Operator Precedence (high to low)

```
~  !  (unary)
*  /  %
+  -
<< >>
&
^
|
== != < > <= >=
&&
||
=  (assignment)
```

---

### Functions

Two declaration styles:

#### Expression body — for single-expression functions

```kode
func add(a: int, b: int) = a + b
func square(x: int) = x * x
func isEven(n: int) = n % 2 == 0
func max(a: int, b: int) = if (a > b) { a } else { b }
```

#### Block body — for multi-statement functions

```kode
func greet(name: string) {
    print("Hello, ")
    print(name)
}

func power(base: int, exp: int) {
    let result = 1
    let i = 0
    while (i < exp) {
        result = result * base
        i = i + 1
    }
    print(result)
}
```

#### Calling functions

```kode
print(add(3, 4))       // 7
print(square(7))       // 49
print(isEven(8))       // true
greet("World")         // Hello, World
power(2, 10)           // 1024
```

#### Recursion

```kode
func factorial(n: int) = if (n <= 1) { 1 } else { n * factorial(n - 1) }

print(factorial(5))    // 120
print(factorial(10))   // 3628800
```

---

### Control Flow

#### If / Else

Conditions require parentheses. Block requires braces.

```kode
let x = 10

if (x > 5) {
    print("big")
} else {
    print("small")
}
```

#### Chained if / else if

```kode
let score = 75

if (score >= 90) {
    print("A")
} else {
    if (score >= 80) {
        print("B")
    } else {
        if (score >= 70) {
            print("C")
        } else {
            print("F")
        }
    }
}
```

#### For loop

C-style. The init variable must use `let`.

```kode
for (let i = 0; i < 5; i++) {
    print(i)
}
// 0
// 1
// 2
// 3
// 4
```

#### Nested for loops

```kode
for (let i = 1; i <= 3; i++) {
    for (let j = 1; j <= 3; j++) {
        print(i * j)
    }
}
```

#### While loop

```kode
let i = 5
while (i > 0) {
    print(i)
    i = i - 1
}
// 5 4 3 2 1
```

---

### Arrays

Arrays store ordered collections of the same type.

#### Creating arrays

```kode
let nums = [1, 2, 3, 4, 5]
let words = ["hello", "world", "kode"]
let empty = []
```

#### Reading elements (zero-indexed)

```kode
let arr = [10, 20, 30, 40, 50]
print(arr[0])   // 10
print(arr[2])   // 30
print(arr[4])   // 50
```

#### Dynamic index

```kode
let i = 2
print(arr[i])       // 30
print(arr[i + 1])   // 40
```

#### Array length

```kode
let nums = [1, 2, 3, 4, 5]
print(nums.len)   // 5
```

#### Iterating over arrays

```kode
let arr = [10, 20, 30, 40, 50]

for (let i = 0; i < arr.len; i++) {
    print(arr[i])
}
```

#### Array expressions

Elements can be computed at creation time:

```kode
let computed = [5 + 5, 3 * 4, 20 - 8]
print(computed[0])   // 10
print(computed[1])   // 12
print(computed[2])   // 12
```

---

### Structs

Structs group named fields into a single type. Field names in declarations must be separated by commas.

#### Declaring a struct

```kode
struct Point {
    x: int,
    y: int
}

struct Person {
    name: string,
    age: int,
    active: bool
}
```

#### Creating a struct instance

```kode
let p = Point { x: 10, y: 20 }
let u = Person { name: "Alice", age: 30, active: true }
```

#### Accessing fields

```kode
print(p.x)        // 10
print(p.y)        // 20
print(u.name)     // Alice
print(u.age)      // 30
print(u.active)   // true
```

> Note: Struct field mutation and methods are not yet implemented (v0.4 planned). Field access is read-only.

---

### Module System

Kode has a fully working module system. You can split code across `.kode` files.

#### Defining a module

```kode
// math.kode — export functions and constants

export func add(a: int, b: int) {
    print(a + b)
}

export func subtract(a: int, b: int) {
    print(a - b)
}

export func multiply(a: int, b: int) {
    print(a * b)
}

export func divide(a: int, b: int) {
    if (b == 0) {
        print("Error: division by zero")
    } else {
        print(a / b)
    }
}

export func power(base: int, exp: int) {
    let result = 1
    let i = 0
    while (i < exp) {
        result = result * base
        i = i + 1
    }
    print(result)
}
```

#### Named import (destructured)

Import only the symbols you need from a module:

```kode
import { add, subtract, multiply } from "math"

add(10, 5)         // 15
subtract(10, 5)    // 5
multiply(4, 6)     // 24
```

#### Simple import

```kode
import "config"
```

#### Namespace import (aliased)

```kode
import "math" as m
```

#### Exporting constants and variables

```kode
// config.kode
export const VERSION: string = "0.3.1"
export let DEBUG: bool = false
export const MAX_RETRIES: int = 3
```

#### Module path resolution

Kode searches for modules in:
1. Current directory — `./math.kode`
2. `examples/` directory — `examples/math.kode`

---

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print any value to stdout |
| `input()` | Read a line from stdin, returns `string` |

```kode
print(42)           // 42
print("hello")      // hello
print(true)         // true
print(3.14)         // 3.14
print([1, 2, 3])    // array output

// Read input
let name = input()
print(name)
```

---

## Working Examples

### Fibonacci

```kode
func fib(n: int) = if (n <= 1) { n } else { fib(n - 1) + fib(n - 2) }

print(fib(0))    // 0
print(fib(1))    // 1
print(fib(7))    // 13
print(fib(10))   // 55
```

### Sum 1 to N

```kode
func sumTo(n: int) {
    let total = 0
    let i = 1
    while (i <= n) {
        total = total + i
        i = i + 1
    }
    print(total)
}

sumTo(10)    // 55
sumTo(100)   // 5050
```

### Array sum

```kode
let nums = [3, 7, 12, 5, 8, 1]

let sum = 0
for (let i = 0; i < nums.len; i++) {
    sum = sum + nums[i]
}
print(sum)   // 36
```

### Calculator with modules

```kode
// math.kode
export func add(a: int, b: int) { print(a + b) }
export func multiply(a: int, b: int) { print(a * b) }
export func divide(a: int, b: int) {
    if (b == 0) { print("Error: division by zero") } else { print(a / b) }
}
```

```kode
// main.kode
import { add, multiply, divide } from "math"

add(25, 15)        // 40
multiply(12, 8)    // 96
divide(144, 12)    // 12
divide(10, 0)      // Error: division by zero
```

### Bitwise flags

```kode
const READ: int = 1    // 001
const WRITE: int = 2   // 010
const EXEC: int = 4    // 100

let perms = READ | WRITE   // 011

if (perms & READ == 1) {
    print("can read")
}

if (perms & EXEC == 0) {
    print("cannot execute")
}
```

---

## Feature Status

### Implemented and Working (v0.3.1)

| Feature | Notes |
|---------|-------|
| `let` variables | With or without type annotation |
| `const` constants | Requires explicit type |
| `int`, `float`, `string`, `bool` types | Full inference |
| Arithmetic: `+` `-` `*` `/` `%` | Integer and float |
| Comparison: `==` `!=` `<` `>` `<=` `>=` | Returns bool |
| Logical: `&&` `\|\|` `!` | Truthy/falsy for int |
| Bitwise: `&` `\|` `^` `~` `<<` `>>` | Integer only |
| `if` / `else` | Nested supported |
| `for` loop (C-style) | `for (let i=0; i<n; i++)` |
| `while` loop | `while (cond) { }` |
| Functions — expression body | `func f(x: int) = x + 1` |
| Functions — block body | Multi-statement |
| Recursion | Fully working |
| Arrays `[1, 2, 3]` | Homogeneous |
| Array indexing `arr[i]` | Zero-based |
| Array `.len` property | Returns int |
| `import { } from ""` | Named destructured imports |
| `import "" as alias` | Namespace import |
| `import ""` | Simple import |
| `export func` | Export functions |
| `export const` / `export let` | Export values |
| Module loading at runtime | Full symbol resolution |
| `print()` built-in | All types |
| `input()` built-in | Returns string |
| Structs | Declaration + instantiation + field access — see [Structs](#structs) |
| Static type checking | With inference |
| Bytecode compilation | `.kbc` format |
| Bytecode execution | Via `kode` |

### Parser-Only (declaration parses, cannot use values)

| Feature | Notes |
|---------|-------|
| Enums (`enum Color { }`) | Can declare variants, cannot access them (`Color::Red` not in parser) |
| `break` / `continue` | Parsed in loops, runtime handling partial |

### Not Yet Implemented (v0.4+)

| Feature | Target |
|---------|--------|
| Enum variant access and usage | v0.4 |
| Pattern matching (`match`) | v0.4 — parser partial, no runtime handler |
| Error handling (`try` / `catch` / `panic`) | v0.4 — no lexer, parser, or runtime support |
| Struct methods | v0.4 |
| Concurrency: `spawn`, channels, `select` | v0.5 |
| Standard library | v0.5 |
| Generics | v0.6 |
| Traits and interfaces | v0.6 |

---

## Architecture

Kode is a multi-phase compiler pipeline:

```
Source (.kode)
      │
      ▼
   Lexer           Tokenizes source into tokens
      │             internal/lexer
      ▼
   Parser          Builds AST via recursive descent
      │             internal/parser
      ▼
  Type Checker     Static analysis, inference
      │             internal/typer
      ▼
 IR Generator      Intermediate representation
      │             pkg/ir
      ▼
Bytecode Compiler     Generates stack-based VM instructions
      │             pkg/bytecode
      ▼
 .kbc                    Portable bytecode file
      │
      ▼
   Kode VM         Stack-based virtual machine execution
      │             pkg/bytecode
      ▼
   Output / Side Effects
```

### Bytecode VM

Stack-based VM with 37 opcodes:

```
Stack:     OpPush, OpPop, OpDup
Arith:     OpAdd, OpSub, OpMul, OpDiv, OpMod, OpNeg, OpIncr, OpDecr
Bitwise:   OpBitAnd, OpBitOr, OpBitXor, OpBitNot, OpBitShl, OpBitShr
Compare:   OpEq, OpNe, OpLt, OpLte, OpGt, OpGte
Logic:     OpAnd, OpOr, OpNot
Vars:      OpLoadGlobal, OpStoreGlobal
Control:   OpJmp, OpJmpIfFalse, OpJmpIfTrue
Functions: OpCall, OpReturn, OpReturnValue
Arrays:    OpArrayCreate, OpArrayAccess, OpArrayStore, OpArrayLen, OpMemberAccess
I/O:       OpPrint, OpInput
Other:     OpHalt, OpBreak, OpContinue
```

---

## Roadmap

| Version | Focus | Status |
|---------|-------|--------|
| v0.1.0 | Lexer, parser, basic interpreter | ✅ Done |
| v0.2.0 | Bytecode VM, all operators, control flow | ✅ Done |
| v0.3.0 | Arrays and array indexing | ✅ Done |
| v0.3.1 | Array methods, module system, operator fixes | ✅ **Current** |
| v0.4.0 | Structs, enums, pattern matching, error handling | 🚧 Next |
| v0.5.0 | Concurrency — spawn, channels, select | ⏳ Planned |
| v0.6.0 | Standard library, generics, traits | ⏳ Planned |

---

## Project Structure

```
kode/
├── cmd/kode/           Entry point
├── internal/
│   ├── cli/            CLI commands (run, build, check, fmt, ...)
│   ├── lexer/          Tokenizer
│   ├── parser/         Recursive descent parser
│   ├── typer/          Type checker
│   ├── compiler/       Code generation helpers
│   ├── codegen/        Go backend (wip)
│   └── version/        Version constant
├── pkg/
│   ├── ast/            AST node types
│   ├── ir/             Intermediate representation
│   ├── bytecode/       Bytecode compiler + stack VM
│   └── bridge/         Runtime bridge
├── examples/           Working .kode programs
├── test/               Test .kode files
├── docs/               Documentation
├── grammars/           TextMate syntax grammar
├── go.mod
└── README.md
```

---

## Contributing

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make changes with tests
4. Run: `go test ./...`
5. Submit a pull request

---

## License

MIT License — © 2026 ECOCEE

---

[Changelog](./CHANGELOG.md) · [Architecture](./docs/ARCHITECTURE.md) · [Syntax Docs](./docs/syntax.md) · [GitHub](https://github.com/ecocee/kode-go)

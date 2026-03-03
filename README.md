# Kode

> A statically typed, compiled programming language built in Go
> Version **0.3.2-dev** — March 2026

[![Version](https://img.shields.io/badge/version-0.3.2--dev-blue)]()
[![Language](https://img.shields.io/badge/written_in-Go-00ADD8)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()

---

## What is Kode?

Kode is a statically typed programming language written in Go. It features a complete lexer, parser, type checker, bytecode compiler, and a stack-based VM. Kode has a clean syntax inspired by Go and Rust, a fully functional module system, closures, pattern matching, structured error handling, and an extensive set of built-in functions.

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
  - [String Interpolation](#string-interpolation)
  - [Functions and Closures](#functions-and-closures)
  - [Control Flow](#control-flow)
  - [Arrays](#arrays)
  - [Structs](#structs)
  - [Enums](#enums)
  - [Pattern Matching](#pattern-matching)
  - [Error Handling](#error-handling)
  - [Defer](#defer)
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
# Kode v0.3.2-dev
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

#### Compound Assignment

```kode
let x = 10
x += 5    // x = 15
x -= 3    // x = 12
x *= 2    // x = 24
x /= 4    // x = 6
x %= 4    // x = 2
```

#### Increment / Decrement

```kode
let n = 5
n++    // n = 6
n--    // n = 5
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

### String Interpolation

Embed expressions directly inside strings using `${}`:

```kode
let name = "Kode"
let version = 3

print("Hello from ${name} v${version}!")
// Hello from Kode v3!

let x = 10
let y = 20
print("Sum of ${x} and ${y} is ${x + y}")
// Sum of 10 and 20 is 30
```

---

### Functions and Closures

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

#### Closures (first-class functions)

Use `fn` for anonymous functions and closures that capture variables from surrounding scope:

```kode
let add = fn(a: int, b: int) { return a + b }
print(add(3, 4))    // 7

// Higher-order: function returning a function
func makeAdder(n: int) {
    return fn(x: int) { return x + n }
}

let add5 = makeAdder(5)
let add10 = makeAdder(10)
print(add5(3))     // 8
print(add10(3))    // 13
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

#### For-in loop (array iteration)

```kode
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits {
    print(fruit)
}
// apple
// banana
// cherry
```

#### Break and Continue

```kode
for (let i = 0; i < 10; i++) {
    if (i == 5) { break }
    if (i % 2 == 0) { continue }
    print(i)
}
// 1 3
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

#### Mutation and dynamic methods

```kode
let nums = [1, 2, 3]

// In-place assignment
nums[1] = 99
print(nums[1])    // 99

// push: append an element
nums.push(4)
print(nums.len)   // 4

// pop: remove and return last element
let last = nums.pop()
print(last)       // 4
print(nums.len)   // 3
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

> **Limitation:** Struct field mutation after construction and `impl` methods are not yet compiled (v0.4 planned). Field access is read-only.

---

### Enums

```kode
enum Direction {
    North,
    South,
    East,
    West
}

enum Status {
    Active,
    Inactive,
    Pending
}
```

> **Limitation:** Enum variant access using `::` syntax (e.g., `Direction::North`) is not yet implemented in the parser. Enum declarations compile cleanly but variant values cannot be used in expressions yet.

---

### Pattern Matching

`match` works on literals, the wildcard `_`, and identifier binding patterns:

```kode
let x = 42

match x {
    0 => { print("zero") }
    42 => { print("the answer") }
    _ => { print("something else") }
}
// the answer

// Bind matched value to a name
func describe(n: int) {
    match n {
        0 => { print("zero") }
        1 => { print("one") }
        n => { print("many: ${n}") }
    }
}

describe(0)    // zero
describe(5)    // many: 5
```

---

### Error Handling

Use `try` / `catch` for structured runtime error handling:

```kode
try {
    let result = 10 / 0
    print(result)
} catch (e) {
    print("Error caught: ${e}")
}
// Error caught: division by zero
```

Errors propagate automatically for:
- Division by zero
- Modulo by zero
- Nil value in arithmetic
- Array index out of bounds

```kode
func safeDivide(a: int, b: int) int {
    try {
        return a / b
    } catch (e) {
        print("Error: ${e}")
        return -1
    }
    return 0
}

print(safeDivide(10, 2))   // 5
print(safeDivide(10, 0))   // Error: division by zero  →  -1
```

---

### Defer

`defer` schedules a statement to run just before the enclosing function returns. Multiple defers run in LIFO order:

```kode
func riskyOp() {
    defer { print("cleanup done") }
    print("doing work...")
}

riskyOp()
// doing work...
// cleanup done
```

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

#### Output

| Function | Description |
|----------|-------------|
| `print(...)` | Print values to stdout, space-separated, variadic |
| `println(...)` | Alias for `print` |
| `printf(fmt, ...)` | Format string output |

#### Input

| Function | Description |
|----------|-------------|
| `input()` | Read a line from stdin, returns `string` |

#### Type utilities

| Function | Description |
|----------|-------------|
| `type(x)` | Returns type name: `"int"`, `"float"`, `"bool"`, `"string"`, `"array"`, `"struct"`, `"function"`, `"nil"` |
| `int(x)` | Convert to int |
| `float(x)` | Convert to float |
| `string(x)` | Convert to string |
| `bool(x)` | Convert to bool |
| `len(x)` | Length of string or array |

#### Range

| Function | Description |
|----------|-------------|
| `range(n)` | Returns `[0, 1, ..., n-1]` |
| `range(start, end)` | Returns `[start, ..., end-1]` |
| `range(start, end, step)` | Stepped range |

#### Math

| Function | Description |
|----------|-------------|
| `abs(n)` | Absolute value |
| `sqrt(n)` | Square root |
| `pow(base, exp)` | Power |
| `floor(n)` | Floor (returns int) |
| `ceil(n)` | Ceiling (returns int) |
| `round(n)` | Round to nearest int |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |
| `random()` | Random float in `[0.0, 1.0)` |
| `math.pi` | π constant |
| `math.e` | e constant |

Also callable as `math.sqrt(n)`, `math.abs(n)`, etc.

#### String methods

Available as method calls (`s.upper()`) or flat calls (`upper(s)`):

| Method | Description |
|--------|-------------|
| `.upper()` | Uppercase |
| `.lower()` | Lowercase |
| `.trim()` | Strip leading/trailing whitespace |
| `.split(sep)` | Split into array of strings |
| `.contains(sub)` | Substring check |
| `.startsWith(pre)` | Prefix check |
| `.endsWith(suf)` | Suffix check |
| `.replace(old, new)` | Replace all occurrences |
| `.indexOf(sub)` | First index, `-1` if not found |
| `.len` | Character count |
| `.repeat(n)` | Repeat string n times |

#### Array functions

| Function | Description |
|----------|-------------|
| `sort(arr)` | Return sorted copy of array |
| `reverse(arr)` | Return reversed copy |
| `join(arr, sep)` | Concatenate elements to string |
| `has(arr, val)` | Check if value exists in array |
| `keys(struct)` | Field names of a struct |
| `values(struct)` | Field values of a struct |

#### File I/O

| Function | Description |
|----------|-------------|
| `readFile(path)` | Read file as string |
| `writeFile(path, content)` | Write string to file |
| `appendFile(path, content)` | Append string to file |
| `fileExists(path)` | Check if file path exists |
| `readDir(path)` | List directory entries as string array |
| `joinPath(...)` | Join path segments |

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
func sumTo(n: int) int {
    let total = 0
    let i = 1
    while (i <= n) {
        total += i
        i++
    }
    return total
}

print(sumTo(10))    // 55
print(sumTo(100))   // 5050
```

### For-in with range

```kode
let sum = 0
for n in range(1, 11) {
    sum += n
}
print(sum)    // 55
```

### String interpolation

```kode
let user = "Alice"
let score = 97
print("Player ${user} scored ${score} points!")
// Player Alice scored 97 points!
```

### Closures — adder factory

```kode
func makeAdder(n: int) {
    return fn(x: int) { return x + n }
}

let add5 = makeAdder(5)
print(add5(3))    // 8
print(add5(10))   // 15
```

### Try/catch with division

```kode
func safeDivide(a: int, b: int) int {
    try {
        return a / b
    } catch (e) {
        print("Error: ${e}")
        return -1
    }
    return 0
}

print(safeDivide(10, 2))   // 5
print(safeDivide(10, 0))   // Error: division by zero  →  -1
```

### Pattern matching

```kode
func describe(n: int) {
    match n {
        0 => { print("zero") }
        1 => { print("one") }
        _ => { print("many: ${n}") }
    }
}

describe(0)    // zero
describe(1)    // one
describe(42)   // many: 42
```

### Calculator with modules

```kode
// math.kode
export func add(a: int, b: int) int { return a + b }
export func multiply(a: int, b: int) int { return a * b }
```

```kode
// main.kode
import { add, multiply } from "math"

print(add(25, 15))        // 40
print(multiply(12, 8))    // 96
```

### Bitwise flags

```kode
const READ: int = 1
const WRITE: int = 2
const EXEC: int = 4

let perms = READ | WRITE

if (perms & READ == 1) { print("can read") }
if (perms & EXEC == 0) { print("cannot execute") }
```

---

## Feature Status

### Fully Implemented and Working (v0.3.2-dev)

| Feature | Notes |
|---------|-------|
| `let` variables | With or without type annotation |
| `const` constants | Requires explicit type |
| `int`, `float`, `string`, `bool` types | Full inference |
| Array type `[]T` | Homogeneous |
| Arithmetic: `+` `-` `*` `/` `%` | Integer and float mixed |
| Compound assignment: `+=` `-=` `*=` `/=` `%=` | All arithmetic operators |
| Increment/decrement: `++` `--` | Post-increment and post-decrement |
| Comparison: `==` `!=` `<` `>` `<=` `>=` | Returns `bool` |
| Logical: `&&` `\|\|` `!` | Truthy/falsy for `int` |
| Bitwise: `&` `\|` `^` `~` `<<` `>>` | Integer only |
| String interpolation `"${expr}"` | Arbitrary expressions in strings |
| `if` / `else` | Nested supported |
| `for` loop (C-style) | `for (let i=0; i<n; i++)` |
| `for x in arr` (for-in) | Array element iteration |
| `while` loop | `while (cond) { }` |
| `break` / `continue` | Correct VM jump target patching |
| Functions — expression body | `func f(x: int) = x + 1` or `=>` |
| Functions — block body | Multi-statement with `return` |
| Closures / first-class functions | `fn(x) { ... }` captures outer variables |
| Recursion | Fully working |
| Arrays `[1, 2, 3]` | Homogeneous literal |
| Array indexing `arr[i]` | Zero-based |
| Array assignment `arr[i] = v` | In-place element mutation |
| Array `.len` property | Returns int |
| Array `.push(v)` / `.pop()` methods | Append / remove last |
| Structs | Declaration + instantiation + field read |
| Enums | Declaration + variant object creation in VM |
| Pattern matching (`match`) | Literal, wildcard `_`, identifier binding |
| `try` / `catch` error handling | Full bytecode + VM support |
| `defer` statement | LIFO execution on function return |
| `nil` literal | Null value |
| Module system — named import | `import { f, g } from "mod"` |
| Module system — namespace import | `import "mod" as alias` |
| Module system — simple import | `import "mod"` |
| `export func` / `export const` / `export let` | Export from module |
| Module path resolution | Current dir + `examples/` |
| `print(...)` | Variadic, all types |
| `println(...)` / `printf(fmt, ...)` | Aliases and format output |
| `input()` | Reads line from stdin |
| `type(x)` | Returns type name string |
| `int(x)`, `float(x)`, `string(x)`, `bool(x)` | Type conversions |
| `len(x)` | Length of string or array |
| `range(n)`, `range(s, e)`, `range(s, e, step)` | Integer range array |
| Math: `abs`, `sqrt`, `pow`, `floor`, `ceil`, `round`, `min`, `max`, `random` | Numeric |
| Math constants: `math.pi`, `math.e` | Float64 |
| String builtins: `upper`, `lower`, `trim`, `split`, `replace`, `contains`, `startsWith`, `endsWith`, `indexOf`, `repeat` | Both flat and method-call |
| Array functions: `sort`, `reverse`, `join`, `has` | Return new arrays |
| `keys(struct)`, `values(struct)` | Struct introspection |
| File I/O: `readFile`, `writeFile`, `appendFile`, `fileExists`, `readDir`, `joinPath` | OS-level |
| Static type checking | With inference |
| Bytecode compilation | `.kbc` portable format |
| Bytecode VM execution | Stack-based, 53 opcodes |
| Line comments `//` and block comments `/* */` | Nested block comments supported |

### Parsed but Runtime-Limited

| Feature | Status |
|---------|--------|
| Enum variant access `Color::Red` | Lexer/parser not yet implemented — enums declare but values cannot be referenced |
| `impl` methods on structs | Parsed, but method bodies are not compiled or dispatched |
| `async` / `await` | Parsed, no async runtime semantics |
| `spawn` statement | Parsed, silently skipped at compile time |
| `chan` channels / `select` | Parsed, silently skipped at compile time |
| `trait` / `impl Trait for Type` | Parsed, type checker aware only |
| `service` / HTTP routes | Parsed, no HTTP runtime |

### Not Yet Implemented (v0.4+)

| Feature | Target |
|---------|--------|
| Struct field mutation after construction | v0.4 |
| `impl` block method compilation + dispatch | v0.4 |
| Enum `::` variant access in expressions | v0.4 |
| Full match destructuring (enum, struct, tuple) | v0.4 |
| `panic` / `Result` type error model | v0.4 |
| Concurrency — spawn, channels, `select` | v0.5 |
| Standard library modules | v0.5 |
| Generics | v0.6 |
| Traits and interface dispatch | v0.6 |

---

## Architecture

Kode is a multi-phase compiler pipeline:

```
Source (.kode)
      │
      ▼
   Lexer              Tokenizes source into tokens
      │                internal/lexer  (643 lines)
      ▼
   Parser             Recursive descent parser, builds AST
      │                internal/parser  (1810 lines)
      ▼
  Type Checker        Static analysis with type inference
      │                internal/typer  (1000 lines)
      ▼
 IR Generator         Intermediate representation
      │                pkg/ir / internal/compiler
      ▼
Bytecode Compiler     Generates stack-based VM instructions
      │                pkg/bytecode/compiler.go  (1817 lines)
      ▼
   .kbc               Portable serialised bytecode file
      │                pkg/bytecode/bytecode.go  (458 lines)
      ▼
   Kode VM            Stack-based virtual machine
      │                pkg/bytecode/vm.go  (2045 lines)
      ▼
   Output / Side Effects
```

### Bytecode VM

Stack-based VM with **53 opcodes**:

```
Stack:      OpPush, OpPop, OpDup, OpLoad, OpStore, OpLoadGlobal, OpStoreGlobal
Arithmetic: OpAdd, OpSub, OpMul, OpDiv, OpMod, OpNeg, OpIncr, OpDecr
Bitwise:    OpBitAnd, OpBitOr, OpBitXor, OpBitNot, OpBitShl, OpBitShr
Comparison: OpEq, OpNe, OpLt, OpLte, OpGt, OpGte
Logic:      OpAnd, OpOr, OpNot
Control:    OpJmp, OpJmpIfFalse, OpJmpIfTrue, OpBreak, OpContinue
Functions:  OpCall, OpCallDynamic, OpReturn, OpReturnValue, OpMakeClosure, OpMethodCall
Arrays:     OpArrayCreate, OpArrayAccess, OpArrayStore, OpArrayLen, OpArrayPush, OpArrayPop
Structs:    OpStructCreate, OpStructField, OpMemberAccess
Enums:      OpEnumVariant
I/O:        OpPrint, OpInput, OpInputWithPrompt
Errors:     OpTryBegin, OpTryEnd, OpThrow, OpDefer
Other:      OpNoop, OpHalt
```

### Test Status

| Package | Status |
|---------|--------|
| `internal/lexer` | ✅ PASS |
| `internal/parser` | ✅ PASS |
| `internal/typer` | ✅ PASS |
| `internal/compiler` | ✅ PASS |
| `internal/cli` | ❌ `TestBuildCommandProducesBinary` failing |
| `pkg/bytecode` | — no test files yet |

---

## Roadmap

| Version | Focus | Status |
|---------|-------|--------|
| v0.1.0 | Lexer, parser, basic interpreter | ✅ Done |
| v0.2.0 | Bytecode VM, all operators, control flow | ✅ Done |
| v0.3.0 | Arrays, array indexing | ✅ Done |
| v0.3.1 | Module system, structs, bytecode polish | ✅ Done |
| v0.3.2 | Closures, try/catch, defer, string interpolation, match, 40+ builtins | 🚧 In Progress |
| v0.4.0 | Struct mutation + methods, enum variants, full match patterns | ⏳ Next |
| v0.5.0 | Concurrency — spawn, channels, select | ⏳ Planned |
| v0.6.0 | Standard library, generics, traits | ⏳ Planned |

---

## Project Structure

```
kode/
├── cmd/kode/           Entry point
├── internal/
│   ├── cli/            CLI commands (run, build, check, fmt, ...)
│   ├── lexer/          Tokenizer (643 lines)
│   ├── parser/         Recursive descent parser (1810 lines)
│   ├── typer/          Type checker with inference (1000 lines)
│   ├── compiler/       IR code generation helpers
│   └── version/        Version constant
├── pkg/
│   ├── ast/            AST node types (640 lines)
│   ├── ir/             Intermediate representation
│   ├── bytecode/       Bytecode compiler (1817 lines) + stack VM (2045 lines)
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

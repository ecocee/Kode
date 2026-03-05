# 📚 Kode Language Wiki

> A comprehensive guide to the modern Kode programming language (v0.3.1+ edition, 2026)

**Maintained by the Kode project**

---

## 🧩 What is Kode?

Kode is a **modern, statically-typed systems programming language** designed for performance, safety, and productivity. It features:

- **Stack-based VM execution** - Portable bytecode compilation to `.kbc` format (default)
- **Multiple backends** - Optional Go code generation for native performance and interoperability
- **Complete module system** - Powerful import/export system for code organization and reusability
- **Comprehensive operator support** - Arithmetic, bitwise, logical, comparison, and string operators
- **Advanced control flow** - If-else, for/while loops, pattern matching, and error handling
- **Static type system** - Full type inference with optional explicit annotations
- **Object-oriented features** - Structs with methods, traits/interfaces, enumerations
- **Concurrency-first design** - Goroutines, channels, select statements, and mutex synchronization
- **Functional programming** - First-class functions, closures, higher-order functions, and generics
- **Precise error reporting** - Line number tracking through entire compilation pipeline

---

## 🛠️ Installation & Setup

### Requirements
- **Go 1.18 or later** - [Download Go](https://golang.org/dl/)
- Git (for cloning repository)

### Quick Installation

#### Windows (PowerShell/CMD)
```powershell
go install github.com/ecocee/kode-go/cmd/kode@latest
kode version
```

#### macOS / Linux (Bash)
```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
kode version

# Add to PATH if needed
export PATH=$PATH:$(go env GOPATH)/bin
```

### Build from Source (All Platforms)

```bash
git clone https://github.com/ecocee/kode
cd kode-go
go build -o kode ./cmd/kode
./kode version
```

---

## 🚀 Quick Start

### Hello World

```kode
fn main() {
    print("Hello, World!")
}
```

Run with: `kode hello.kode`

### Working with Variables

```kode
let name = "Alice"
let age = 30
const PI = 3.14159

print("${name} is ${age} years old")
print("Pi is approximately ${PI}")
```

### Using Operators

```kode
// Arithmetic
let a = 10
let b = 3
print(a + b)        // 13
print(a * b)        // 30

// Bitwise
print(a & b)        // 2 (binary AND)
print(a << 2)       // 40 (left shift)

// Comparison
print(a > b)        // true
print(a == 10)      // true

// String concatenation
let greeting = "Hello, " + name
print(greeting)
```

### Arrays and Collections

```kode
let numbers = [1, 2, 3, 4, 5]

// Access elements
print(numbers[0])      // 1

// Array methods
print(len(numbers))    // 5
push(numbers, 6)       // [1, 2, 3, 4, 5, 6]

// Higher-order functions
let doubled = map(numbers, fn(x) { return x * 2 })
let evens = filter(numbers, fn(x) { return x % 2 == 0 })
let sum = reduce(numbers, 0, fn(acc, x) { return acc + x })
```

### Error Handling

```kode
fn divide(a: int, b: int) -> Result<int, string> {
    if (b == 0) {
        return Result.Error("Division by zero")
    }
    return Result.Success(a / b)
}

match (divide(10, 2)) {
    Result.Success(value) => print("Result: ${value}"),
    Result.Error(msg) => print("Error: ${msg}")
}
```

### Concurrency

```kode
// Spawn lightweight threads
spawn {
    for (let i = 0; i < 5; i = i + 1) {
        print("Worker: ${i}")
    }
}

// Channels for communication
let chan = make(Channel<int>)

spawn {
    send(chan, 42)
}

let value = receive(chan)
print("Got: ${value}")  // 42
```

---

## 📦 CLI Commands Reference

### Run Commands

```bash
# Direct interpretation (fastest startup, best for development)
kode program.kode

# Run with verbose output
kode --verbose program.kode

# Specify module search path
kode --module-path ./my_modules program.kode
```

### Build Commands

```bash
# Build to bytecode (default, portable)
kode build program.kode
# Output: program.kbc

# Build to bytecode with output name
kode build -o output.kbc program.kode

# Build to Go source code (legacy, native performance)
kode build --go program.kode
# Output: program.go (can then compile with `go build program.go`)

# Build with optimizations
kode build --optimize program.kode

# Cross-compile to target platform
kode build --go --target windows program.kode
kode build --go --target linux program.kode
```

### Type Checking & Formatting

```bash
# Type check without running
kode check program.kode

# Strict type checking
kode check --strict program.kode

# Format code
kode fmt program.kode

# Format with in-place modification
kode fmt --in-place program.kode

# Check formatting without modifying
kode fmt --check program.kode
```

### Project Management

```bash
# Create new project
kode new myproject
kode new myproject --lib          # Library project
kode new myproject --with-examples # Include examples

# Type check entire project
kode check .

# Format entire project
kode fmt .

# Clean build artifacts
kode clean
kode clean --all                  # Also clean cache
```

### Information Commands

```bash
# Show version
kode version

# Show environment info
kode doctor

# List available commands
kode help
kode help build
```

### Execution Comparison

| Command | Startup | Performance | Best For |
|---------|---------|-------------|----------|
| `kode` | ⚡ Instant | 🟡 Good | Development & testing |
| `kode build` (bytecode) | ⚡ Fast | 🟡 Good | Distribution, portability |
| `kode build --go` (native) | 🟡 Varies | ⚡ Excellent | Performance-critical code |

---

## 📚 Module System

Kode's module system enables code organization, reusability, and maintainability through imports and exports.

### Module Basics

**math.kode:**
```kode
export fn add(a: int, b: int) -> int {
    return a + b
}

export fn subtract(a: int, b: int) -> int {
    return a - b
}

export const PI = 3.14159
```

**main.kode:**
```kode
import { add, subtract, PI } from "math"

fn main() {
    print(add(10, 5))        // 15
    print(subtract(10, 5))   // 5
    print(PI)                // 3.14159
}
```

### Import Styles

```kode
// Named import - import specific items
import { add, subtract } from "math"

// Namespace import - import as namespace
import math from "math"
math.add(5, 3)

// Wildcard import - import all exports
import * from "math"

// Module alias
import math as M from "./utilities/math"
M.add(5, 3)
```

### Export Syntax

```kode
// Export functions
export fn calculate(x: int) -> int {
    return x * 2
}

// Export variables
export let APP_NAME = "MyApp"

// Export constants
export const VERSION = "1.0.0"

// Export structs
export struct Point {
    x: int,
    y: int
}

// Export enums
export enum Status {
    Active,
    Inactive
}

// Export traits
export trait Drawable {
    fn draw() -> void
}
```

### Module Resolution

Modules are resolved in this order:
1. Relative to current file (`./module`, `../module`)
2. Relative to project root (`/module`)
3. Standard library paths

### Path Examples

```kode
// Same directory
import { utils } from "./utils"

// Parent directory
import { helpers } from "../helpers"

// Nested directories
import { db } from "../services/database"

// Absolute from project root
import { config } from "/config"
```

---

## 🧬 Advanced Features

### Pattern Matching

```kode
let value = 5

match (value) {
    1 => print("one"),
    2 => print("two"),
    3 | 4 => print("three or four"),
    5..10 => print("five to ten"),
    _ => print("other")
}
```

### Destructuring

```kode
let (x, y) = (10, 20)
print("${x}, ${y}")  // 10, 20

match (person) {
    Person { name: n, age: a } => print("${n} is ${a}")
}
```

### Higher-Order Functions

```kode
fn apply(f: (int) -> int, x: int) -> int {
    return f(x)
}

let double = fn(x) { return x * 2 }
print(apply(double, 5))  // 10
```

### Generics

```kode
struct Container<T> {
    value: T
}

impl<T> Container<T> {
    fn get() -> T {
        return this.value
    }
}

let intContainer = Container<int> { value: 42 }
let stringContainer = Container<string> { value: "hello" }
```

### Traits & Polymorphism

```kode
trait Animal {
    fn speak() -> void
}

struct Dog {}
impl Animal for Dog {
    fn speak() -> void { print("Woof!") }
}

struct Cat {}
impl Animal for Cat {
    fn speak() -> void { print("Meow!") }
}

fn animalSound(animal: Animal) {
    animal.speak()
}

animalSound(Dog {})  // Woof!
animalSound(Cat {})  // Meow!
```

---

## 📖 Code Examples

### Factorial (Recursion)

```kode
fn factorial(n: int) -> int {
    if (n <= 1) {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}

fn main() {
    print(factorial(5))  // 120
}
```

### Array Sum

```kode
fn sumArray(arr: [int]) -> int {
    let total = 0
    for (item in arr) {
        total = total + item
    }
    return total
}

fn main() {
    let numbers = [1, 2, 3, 4, 5]
    print(sumArray(numbers))  // 15
}
```

### Functional Array Operations

```kode
fn main() {
    let numbers = [1, 2, 3, 4, 5]
    
    // Map - transform each element
    let doubled = map(numbers, fn(x) { return x * 2 })
    print(doubled)  // [2, 4, 6, 8, 10]
    
    // Filter - keep elements that match condition
    let evens = filter(numbers, fn(x) { return x % 2 == 0 })
    print(evens)    // [2, 4]
    
    // Reduce - combine into single value
    let sum = reduce(numbers, 0, fn(acc, x) { return acc + x })
    print(sum)      // 15
}
```

### Working with Structs

```kode
struct Person {
    name: string,
    age: int,
    email: string
}

impl Person {
    fn isAdult() -> bool {
        return this.age >= 18
    }
    
    fn greet() -> string {
        return "Hello, I'm ${this.name}"
    }
}

fn main() {
    let person = Person {
        name: "Alice",
        age: 30,
        email: "alice@example.com"
    }
    
    if (person.isAdult()) {
        print(person.greet())
    }
}
```

### Error Handling with Result Types

```kode
enum FileError {
    NotFound,
    PermissionDenied,
    IoError(message: string)
}

fn readFile(path: string) -> Result<string, FileError> {
    // Simulation
    if (path == "") {
        return Result.Error(FileError.NotFound)
    }
    return Result.Success("file content")
}

fn main() {
    match (readFile("config.txt")) {
        Result.Success(content) => print("Content: ${content}"),
        Result.Error(err) => print("Error reading file")
    }
}
```

### Concurrent Processing

```kode
fn main() {
    let messages = make(Channel<string>)
    
    spawn {
        send(messages, "Hello from worker 1")
        send(messages, "Worker 1 done")
    }
    
    spawn {
        send(messages, "Hello from worker 2")
        send(messages, "Worker 2 done")
    }
    
    // Receive 4 messages
    for (let i = 0; i < 4; i = i + 1) {
        let msg = receive(messages)
        print(msg)
    }
}
```

---

## 📌 Feature Status (v0.3.4)

### ✅ Fully Implemented

- ✅ Complete type system with inference
- ✅ All operators (arithmetic, bitwise, logical, comparison)
- ✅ Control flow (if/else, for, while, do-while, match)
- ✅ Functions with named parameters and defaults
- ✅ Structs with methods and impl blocks
- ✅ Enums with associated values
- ✅ Traits and interface implementations
- ✅ Pattern matching and destructuring
- ✅ First-class functions and closures
- ✅ Generics and generic constraints
- ✅ Arrays with higher-order operations
- ✅ Result and Option types for error handling
- ✅ Module system with imports/exports
- ✅ Concurrency primitives (spawn, channels, select)
- ✅ Bytecode VM execution
- ✅ Go code generation backend
- ✅ Full line number tracking and error reporting
- ✅ `fn`-only function keyword (`func` removed)
- ✅ Production server stdlib (`stdlib/server.kode`) — `import { newServer, get, okJSON } from "server"`

### ⏳ In Development

- ⏳ File I/O operations
- ⏳ String manipulation utilities
- ⏳ Math library functions
- ⏳ Package manager (Kodepm)

### 🔮 Planned

- 🔮 Async/await syntax sugar
- 🔮 Macro system
- 🔮 LLVM backend option
- 🔮 JIT compilation
- 🔮 WebAssembly target
- 🔮 FFI (Foreign Function Interface)

---

## 🙌 Contributing

Found a bug? Want to contribute code or documentation? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Getting Involved

1. **Report Issues**: [GitHub Issues](https://github.com/ecocee/kode/issues)
2. **Submit PRs**: Fork → Create branch → Submit pull request
3. **Join Discussion**: Participate in GitHub Discussions
4. **Improve Docs**: Update documentation and examples

---

*Kode v0.3.1+ - Modern Systems Programming Language*

**Learn More:** [ARCHITECTURE.md](./ARCHITECTURE.md) | [syntax.md](./syntax.md) | [roadmap.md](./roadmap.md)

© 2026 Kode Programming Language Project

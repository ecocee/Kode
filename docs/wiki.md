# 📚 Kode Language Wiki

> A complete guide to the Kode programming language (2026 edition)

**Maintained by the Kode project**

---

## 🧩 What is Kode?

Kode is a modern programming language designed for high-performance systems. It features:

- **Bytecode compilation** to portable `.kbc` format with stack-based VM execution (default)
- **Go code generation backend** option for native performance and ecosystem interop (legacy)
- **Complete operator support** including arithmetic, bitwise, logical, and comparison operators
- **Full control flow** with if-else, for loops, while loops, and functions
- **Static type system** with type inference
- **Concurrency-first design** with channels and goroutines (in development)

---

## 🛠️ Installation

Install the CLI with the Go toolchain:

```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
```

The `kode` command will then be available in your `PATH`.

---

## 🚀 Getting Started

### Quick Example

```kode
// Simple arithmetic and bitwise operations
let a = 12
let b = 5
print(a + b)    // 17
print(a & b)    // 4 (bitwise AND)
print(a << 2)   // 48 (left shift)
```

### For Loops

```kode
for (let i = 0; i < 5; i++) {
  print(i)
}
// Output: 0 1 2 3 4
```

### While Loops

```kode
let x = 0
while (x < 3) {
  print(x)
  x = x + 1
}
// Output: 0 1 2
```

### Functions

```kode
func add(a: int, b: int) = a + b
print(add(3, 4))  // Output: 7
```

---

## 📦 CLI Reference

### Build to Bytecode (Default)

```bash
kode build program.kode       # Creates program.kbc
kode program.kbc              # Execute the bytecode
```

### Build to Go Binary (Legacy)

```bash
kode build --go program.kode  # Creates program (or .exe on Windows)
./program                     # Run the binary
```

### Other Commands

```bash
kode run program.kode         # Compile and execute in one step
kode check program.kode       # Type check only
kode fmt program.kode         # Format source code
kode clean                    # Remove generated artifacts
kode version                  # Show version info
```

---

## 📘 Language Overview

Kode uses clean, familiar syntax inspired by C. Code is organized as:
- Statements ending with semicolons
- Blocks enclosed in curly braces `{}`
- Variables declared with `let`
- Functions defined with `func`

### Basic Syntax

```kode
let x = 10
x = x + 5           // Update variable
if (x > 10) {
  print("x is large")
}
```

### Operators

#### Arithmetic Operators
```kode
10 + 5    // Addition: 15
10 - 3    // Subtraction: 7
4 * 5     // Multiplication: 20
10 / 2    // Division: 5
10 % 3    // Modulo: 1
-5        // Negation
```

#### Comparison Operators
```kode
5 == 5    // Equal: true
5 != 3    // Not equal: true
5 < 10    // Less than: true
5 > 3     // Greater than: true
5 <= 5    // Less or equal: true
5 >= 5    // Greater or equal: true
```

#### Logical Operators
```kode
1 && 1    // Logical AND: true
1 || 0    // Logical OR: true
!0        // Logical NOT: true
```

#### Bitwise Operators (NEW!)
```kode
12 & 5    // Bitwise AND: 4
12 | 5    // Bitwise OR: 13
12 ^ 5    // Bitwise XOR: 9
8 << 2    // Left shift: 32
8 >> 2    // Right shift: 2
~5        // Bitwise NOT: -6
```

### Types & Inference

Kode supports these primary types:
- `int` - Integer values
- `float` - Floating point numbers
- `string` - Text strings
- `bool` - Boolean (true/false)

Type inference is automatic in most cases:

```kode
let x = 10        // inferred as int
let y = 3.14      // inferred as float
let s = "hello"   // inferred as string
let b = true      // inferred as bool
```

### Control Flow

#### If-Else

```kode
if (x > 10) {
  print("greater")
} else {
  print("less or equal")
}
```

#### For Loops

```kode
for (let i = 0; i < 5; i++) {
  print(i)  // Prints 0 1 2 3 4
}
```

#### While Loops

```kode
let i = 0
while (i < 5) {
  print(i)
  i = i + 1
}
```

### Functions

Define functions with typed parameters:

```kode
func multiply(a: int, b: int) = a * b
print(multiply(6, 7))  // Output: 42
```

Recursive functions are supported:

```kode
func factorial(n: int) = if (n <= 1) { 1 } else { n * factorial(n - 1) }
print(factorial(5))  // Output: 120
```

### Standard Library

Core built-in functions available:
- `print(value)` - Output to console
- `input()` - Read user input

More functions will be added in future releases.

## 📚 Examples

### Hello World

```kode
print("Hello, World!")
```

### Arithmetic with Bitwise Operations

```kode
let x = 15
let y = 7
print(x + y)      // 22
print(x & y)      // 7 (bitwise AND)
print(x | y)      // 15 (bitwise OR)
print(x << 1)     // 30 (left shift)
```

### Looping

```kode
print("Count to 5:")
for (let i = 1; i <= 5; i++) {
  print(i)
}

print("Countdown from 3:")
let j = 3
while (j > 0) {
  print(j)
  j = j - 1
}
```

### Functions and Recursion

```kode
func square(x: int) = x * x
func power(base: int, exp: int) = 
  if (exp == 0) { 1 } else { base * power(base, exp - 1) }

print(square(5))      // 25
print(power(2, 10))   // 1024
```

See the `examples/` directory for more code snippets.

## 📅 Roadmap & Development

Kode development follows these phases:

### v0.2 - Core Language (Current ✅)
- ✅ Lexer, parser, and type checker
- ✅ Complete operator support (arithmetic, bitwise, logical, comparison)
- ✅ Control flow (if/else, for, while)
- ✅ Functions with parameter inlining  
- ✅ Bytecode compiler and VM
- ✅ Colored CLI

### v0.3 - Standard Library & I/O
- ⏳ Expanded built-in functions
- ⏳ File I/O operations
- ⏳ String utilities
- ⏳ Math library

### v0.4 - Advanced Features
- ⏳ Structs and methods
- ⏳ Error handling (Result types)
- ⏳ Collections (arrays, maps)
- ⏳ Generic types

### v0.5+ - Future
- ⏳ Concurrency (channels, goroutines)
- ⏳ Module system and packages
- ⏳ Async/await
- ⏳ JIT compilation option
- ⏳ LLVM backend

See [docs/roadmap.md](./roadmap.md) for detailed development plans.

## 🙌 Contributing

Contributions, bug reports and documentation updates are welcome. Please read
[CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

*Documentation generated 2026 — stay tuned for updates!*

```kode
fn main() {
    print "Hello, World!";
}
```

### Factorial

```kode
fn factorial(n) {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn main() {
    print factorial(5);  // 120
}
```

### Array Manipulation

```kode
fn main() {
    let numbers = [1, 2, 3, 4, 5];
    
    // Sum all numbers
    let sum = 0;
    for (let i = 0; i < 5; i = i + 1) {
        sum = sum + numbers[i];
    }
    
    print sum;  // 15
    
    // Double each number
    for (let i = 0; i < 5; i = i + 1) {
        numbers[i] = numbers[i] * 2;
    }
    
    print numbers;  // [2, 4, 6, 8, 10]
}
```

### Error Handling

```kode
fn divide(a, b) {
    if (b == 0) {
        // This would trigger an error
        return a / b;
    }
    return a / b;
}

fn main() {
    try {
        print divide(10, 0);
    } catch {
        print "Cannot divide by zero";
    }
}
```

---

## 🔄 Language Comparisons

### Kode vs JavaScript
- Similar syntax for variables, functions, and control flow
- Kode is simpler with fewer built-in objects and methods
- Kode lacks JavaScript's prototype-based OOP and more advanced features

### Kode vs Python
- Kode uses explicit braces for blocks instead of indentation
- Kode requires semicolons to end statements
- Kode's syntax is closer to C-family languages
- Python has more extensive libraries and language features

### Kode vs Rust
- Kode is interpreted while Rust is compiled
- Kode is dynamically typed while Rust is statically typed
- Kode lacks Rust's ownership system and advanced type features
- Syntax has some similarities but Kode is much simpler

---

## ⚠️ Current Status

### ✅ Implemented Features

- ✅ Variable declarations and assignments
- ✅ All arithmetic operators (+, -, *, /, %)
- ✅ All comparison operators (==, !=, <, <=, >, >=)
- ✅ All logical operators (&&, ||, !)
- ✅ **All bitwise operators** (&, |, ^, ~, <<, >>)
- ✅ If-else conditional statements
- ✅ For loops with proper variable tracking
- ✅ While loops with condition checking
- ✅ Function definitions with typed parameters
- ✅ Function calls with argument passing
- ✅ Recursive functions
- ✅ Built-in functions (print, input)
- ✅ Break and Continue statements (basic)
- ✅ Bytecode compilation and execution
- ✅ Colored CLI with helpful error messages

### ⏳ Planned Features

- ⏳ Concurrency (goroutines, channels, select)
- ⏳ Object-oriented programming (structs, methods)
- ⏳ Error handling (try-catch with specific error types)
- ⏳ Collections (arrays, maps, sets)
- ⏳ Module system and packages
- ⏳ Generic types
- ⏳ Higher-order functions and closures
- ⏳ Type annotations in function signatures

### ❌ Not Yet Implemented

- ❌ Import/module system
- ❌ Structs and enums
- ❌ Pattern matching
- ❌ Async/await
- ❌ REPL (interactive shell)
- ❌ Standard library beyond print/input
- ❌ Null safety mechanisms
- ❌ Null values (uses nil for uninitialized)

---

## 🔮 Future Development

See the [roadmap](roadmap.md) for detailed development plans. Key areas of focus include:

1. Adding more data structures (maps, sets)
2. Implementing a robust standard library
3. Adding object-oriented programming features
4. Improving error handling with specific error types
5. Adding type annotations and optional static type checking
6. Implementing a package manager
7. Adding async/await functionality
8. Improving performance with JIT compilation

---

*Wiki maintained by Sreeraj V Rajesh*

© 2025 Kode Programming Language
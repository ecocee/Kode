# 📚 Kode Language Wiki

> A complete guide to the Kode programming language (2026 edition)

**Maintained by the Kode project**

---

## 🧩 What is Kode?

Kode is a concurrency‑first, statically typed compiled language that generates
idiomatic Go code. It targets backend services, microservices, and distributed
systems with a focus on predictable performance, strong type inference, and
built-in channels & goroutines.

## 🛠️ Installation

Install the CLI with the Go toolchain:

```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
```

The `kode` command will then be available in your `PATH`.

## 🚀 Getting Started

```bash
kode new myproject      # scaffold a new project
cd myproject
kode build .            # generate Go code and compile
./myproject             # run the binary
```

## 📦 CLI Reference

```bash
kode run path/to/file.kode       # type check, compile and execute
kode build path/to/file.kode     # compile to Go + go build
kode fmt path/to/file.kode       # format source code
kode check path/to/file.kode     # type check only
kode clean                       # remove generated artifacts
kode version                     # show version and build info
``` 

> See [docs/cli.md](./cli.md) for the full command reference and options.

## 📘 Language Overview

### Syntax

Kode uses a clean, familiar syntax with C‑inspired braces and semicolons.
Statements end with `;` and blocks use `{}`.

```kode
fn main() {
    let x = 10;
    print x;
}
```

### Types & Inference

The static type system employs Hindley‑Milner inference; you rarely need to
annotate types unless distinguishing channel element types or generics.

### Concurrency

Concurrency is first class:

- `go` launches lightweight goroutines
- `chan<T>` is a typed channel
- `select` multiplexes channel operations

Example:

```kode
let ch: chan<int> = chan.new();
go fn() { ch <- 42; }();
let v = <-ch;
```

### Standard Library (v0.3)

The standard library currently includes HTTP server/client helpers, basic
collections, and I/O utilities. More packages are added each release.

## 📚 Examples

See the `examples/` directory for ready‑made code snippets on concurrency,
services, error handling and more.

## 📅 Roadmap & Development

The project follows a phased roadmap: core language (v0.2), concurrency & stdlib
(v0.3), optimization & packages (v0.4), then long‑term features such as a JIT
and cloud SDKs. Full details are in [docs/roadmap.md](./roadmap.md).

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

## ⚠️ Current Limitations

Kode is in active development and has several limitations:

1. **No Object-Oriented Programming** - No classes or methods
2. **Limited Type System** - Dynamic typing with no type annotations or checks
3. **Basic Standard Library** - Limited built-in functions and utilities
4. **Performance** - As an interpreted language, it's not as fast as compiled languages
5. **Basic Error Handling** - Simple try-catch with no specific error types
6. **Limited Module System** - Basic import functionality without namespacing
7. **No Async Support** - No built-in support for asynchronous programming
8. **Limited Collections** - Only arrays, no dictionaries/maps, sets, etc.

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
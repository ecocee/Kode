# Kode

> A modern, statically typed systems programming language with first-class concurrency  
> Compiles to portable bytecode or native Go. Multi-platform support. *(v1.0 - 2026 edition)*

---

## 🚀 What is Kode?

Kode is an open-source programming language designed for high-throughput servers, microservices,
data processing pipelines, and systems programming. It combines:

- **Static typing** with powerful type inference
- **Multi-backend compilation**: Bytecode (default) or Go code generation
- **Full concurrency support**: Goroutines, channels, select statements
- **Modern language features**: Pattern matching, traits, structs, enums
- **Comprehensive tooling**: Bytecode VM, CLI, formatter, project generator
- **Rich standard library**: Collections, error handling, modules

## 🎯 Why choose Kode?

- **Fast to write, easy to read**: Clean syntax inspired by Python & Go
- **Production-ready**: Stable v1.0 with comprehensive error handling
- **Portable bytecode**: Run anywhere with the Kode VM or compile to native Go
- **Powerful type system**: Automatic inference, pattern matching, generic types
- **First-class concurrency**: Lightweight goroutines and channels
- **Go interoperability**: Optional Go backend for ecosystem access
- **Performance**: Stack-based VM with O(1) instruction execution

## 🌟 Key Features (v1.0+)

### Core Language
- **Variables & Constants**: `let` and `const` declarations with type inference
- **Explicit typing**: Optional type annotations: `let x: int = 42`
- **Functions**: First-class functions, closures, default parameters, variadic arguments
- **Type system**: int, float, string, bool, arrays, tuples, maps, generics

### Data Structures
- **Arrays**: `[1, 2, 3]` with homogeneous type checking, indexing, slicing
- **Structures**: User-defined structs with methods and constructors
- **Enumerations**: Tagged unions with pattern matching support
- **Maps**: Key-value collections with type safety
- **Tuples**: Fixed-size heterogeneous collections

### Advanced Features
- **Pattern Matching**: `match` expressions with destructuring and guards
- **Traits**: Interface definitions with multiple implementations
- **Generics**: Parametric types across structs and functions
- **Module System**: Imports, exports, namespacing, circular dependency detection
- **Error Handling**: `try/catch`, `Result` types, error propagation

### Concurrency (v1.0+)
- **Goroutines**: Lightweight concurrent execution with `spawn`
- **Channels**: Type-safe message passing: `make(Channel<T>)`
- **Select**: Multiplex channel operations: `select { ... }`
- **Synchronization**: Mutexes and atomic operations
- **CSP model**: Communication Sequential Processes paradigm

### Control Flow
- **Conditionals**: if/else with expression form
- **Loops**: for, while, do-while, for-in, for-each with index
- **Loop control**: break, continue with label support
- **Match**: Pattern matching for complex branching

### Compilation Backends
- **Bytecode (Default)**: Platform-independent `.kbc` format, Execute with Kode VM
- **Go Code**: Optional native compilation via Go backend
- **Cross-platform**: Windows, macOS, Linux with identical semantics

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`, `^` (power)
- **Bitwise**: `&`, `|`, `^`, `~`, `<<`, `>>`
- **Logical**: `&&`, `||`, `!`
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `%=`

### Tooling
- **CLI**: `run`, `build`, `check`, `fmt`, `new`, `clean`, `version`, `doctor`
- **Colored output**: ANSI-colored diagnostic messages
- **Project generator**: Scaffold new projects with `kode new`
- **Code formatter**: Consistent code style with `kode fmt`
- **Type checker**: Verify code without running with `kode check`

## 📅 Release Timeline

Kode follows semantic versioning with a clear evolution:

| Version | Focus | Status |
|---------|-------|--------|
| v0.1 | Lexer, parser, basic runtime | ✅ Completed |
| v0.2 | Bytecode VM, operators, control flow | ✅ Completed |
| v0.3 | Data structures (arrays, structs, enums) | ✅ Completed |
| v0.4 | Module system, imports/exports | ✅ Completed |
| v0.5 | Concurrency, channels, select | ✅ Completed |
| v0.6 | Standard library, packages | ✅ Completed |
| **v1.0** | **Production-ready, all features** | ✅ **CURRENT** |
| v1.1+ | Optimizations, ecosystem | 🚧 In Progress |

### Platform Support
- ✅ **Windows** - Fully supported (tested on Windows 10/11)
- ✅ **macOS** - Fully supported (Intel & Apple Silicon)
- ✅ **Linux** - Fully supported (Ubuntu, Fedora, Debian, CentOS, Alpine)

## 📦 Installation

### Prerequisites
- **Go 1.18+** (required for building from source; not needed for bytecode execution)

### Quick Install (From Source)
```bash
git clone https://github.com/ecocee/kode-go
cd kode-go
go build -o kode ./cmd/kode

# Add to PATH
export PATH=$PATH:$(pwd)  # macOS/Linux
# Or copy to system path
```

### Using Go Install
```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
kode version  # Verify installation
```

### Package Managers (When Available)
```bash
# macOS
brew install kode

# Debian/Ubuntu
apt-get install kode

# Fedora/CentOS
dnf install kode

# Arch Linux
pacman -S kode
```

## 🛠️ Quick Start

### Hello World
```bash
echo 'print("Hello, Kode!");' > hello.kode
kode run hello.kode
```

### Run Bytecode
```bash
kode build hello.kode      # Generates hello.kbc
kode hello.kbc             # Execute bytecode
```

### New Project
```bash
kode new myproject
cd myproject
kode run main.kode
```

## 📚 Language Example

```kode
// Import concurrent primitives
import { spawn, Channel } from "concurrency";

// Define a structure
struct Worker {
    id: int,
    name: string
}

impl Worker {
    fn process(self) -> int {
        return self.id * 2;
    }
}

// Concurrent worker pool
fn main() {
    let ch: Channel<int> = Channel::new();
    
    for (let i = 0; i < 5; i = i + 1) {
        let worker = Worker { id: i, name: "Worker ${i}" };
        
        spawn {
            let result = worker.process();
            send(ch, result);
        };
    }
    
    // Collect results
    for (let j = 0; j < 5; j = j + 1) {
        let value = receive(ch);
        print("Result: ${value}");
    }
}
```

## 🔧 CLI Commands

### Build Commands
```bash
kode run file.kode              # Compile and execute directly
kode build file.kode            # Build to bytecode (default)
kode build --go file.kode       # Build to Go binary (native)
kode build --optimize file.kode # With optimizations enabled
```

### Development Commands
```bash
kode check file.kode         # Type-check without execution
kode fmt directory/          # Format all Kode files
kode fmt --check file.kode   # Check formatting without modifying
```

### Project Commands
```bash
kode new myproject           # Create new project scaffold
kode new --lib mylib         # Create library project
kode new --example myexample # Create example project
```

### Utility Commands
```bash
kode version                 # Show version information
kode doctor                  # Check environment/dependencies
kode clean                   # Remove build artifacts
kode help                    # Show help for all commands
```

## 📚 Documentation

Comprehensive guides are available in the `docs/` directory:

- **[syntax.md](./docs/syntax.md)** - Complete language reference with examples
- **[ARCHITECTURE.md](./docs/ARCHITECTURE.md)** - Compiler pipeline, VM design, type system
- **[roadmap.md](./docs/roadmap.md)** - Feature roadmap and future plans
- **[cli.md](./docs/cli.md)** - Command-line interface reference
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Contribution guidelines
- **[CHANGELOG.md](./CHANGELOG.md)** - Complete version history

## 🔗 Example Programs

Check the `examples/` directory for complete working programs:

- **basic.kode** - Simple variables and functions
- **calculator.kode** - Arithmetic operations with module imports
- **concurrency.kode** - Goroutines and channels
- **error_handling.kode** - Try-catch and Result types
- **structs.kode** - Struct definitions and methods
- **services.kode** - Module-based service architecture

## 🤝 Contributing

We welcome contributions! Please:

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Fork the repository
3. Create a feature branch
4. Write tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 🐛 Bug Reports & Feature Requests

- **Issues**: Open on [GitHub Issues](https://github.com/ecocee/kode-go/issues)
- **Discussions**: Join our [GitHub Discussions](https://github.com/ecocee/kode-go/discussions)
- **Security**: Report to security@ecocee.dev

## 📊 Project Status

**Status**: ✅ **PRODUCTION READY (v1.0)**

Kode v1.0 is feature-complete and production-ready. Ongoing development focuses on:
- Performance optimizations
- Standard library expansion
- Tooling improvements
- Community contributions

See [CHANGELOG.md](./CHANGELOG.md) for recent updates and [roadmap.md](./docs/roadmap.md) for future directions.

## 📄 License

MIT License © 2026 ECOCEE

## ❤️ Acknowledgments

Kode builds on ideas from:
- Go (concurrency, simplicity)
- Rust (type safety, pattern matching)
- Python (readability)
- Lisp (homoiconicity, macros)

---

**Created with ❤️ by Sreeraj V Rajesh and the Kode community**

[GitHub](https://github.com/ecocee/kode-go) | [Documentation](./docs) | [Examples](./examples)

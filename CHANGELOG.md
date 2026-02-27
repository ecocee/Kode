# 📦 Changelog

All notable changes to the Kode Programming Language are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### 🚧 In Progress (v0.4+)
- Struct and enum runtime implementation (parsing complete)
- Pattern matching with destructuring
- Concurrency features (spawn, channels, select)
- Advanced error handling (try/catch, Result types)
- Standard library expansion
- Performance optimizations

---

## [v0.3.1] - 2026-02-27

### ✨ Core Language Features
- **Complete operator support**: Arithmetic (`+`, `-`, `*`, `/`, `%`), bitwise (`&`, `|`, `^`, `~`, `<<`, `>>`), logical (`&&`, `||`, `!`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`)
- **Array operations**: Array literals `[1, 2, 3]`, indexing `arr[0]`, length method `arr.len`
- **Control structures**: `if/else`, `for`, `while` loops with full expression support
- **Functions**: Parameterized functions with return values and recursion
- **Type system**: Static typing with inference for `int`, `float`, `string`, `bool`

### 🔧 Technical Improvements
- **Runtime evaluator**: Complete expression evaluation engine
- **Type checker**: Full type checking for all operators and constructs
- **Parser**: Comprehensive syntax parsing with operator precedence
- **Bytecode compiler**: Working compilation pipeline (though currently using runtime mode)
- **Error handling**: Meaningful error messages with line numbers

### 📚 Documentation & Testing
- **Comprehensive test suite**: Full coverage of implemented features
- **Example programs**: Working demonstrations of all features
- **Updated documentation**: Accurate feature descriptions for v0.3.1

### ✨ Module System (✅ FULLY IMPLEMENTED)
- **Import syntax**: Multiple import styles supported
  - Named destructuring: `import { add, multiply } from "math"`
  - Namespace imports: `import "math" as m`
  - Simple imports: `import "config"`
  - Aliased imports: `import "math" as mathematics`
- **Export syntax**: Function and constant exports
  - `export func add(a: int, b: int) { ... }`
  - `export const PI = 3.14159`
  - `export let VERSION = "1.0.0"`
- **Runtime module loading**: Full module resolution and execution
- **Module path resolution**: Searches current directory and examples/
- **Symbol management**: Exported symbols available in importing modules

### 🔧 Error Handling
- **Basic error checking**: Conditional error detection
- **Runtime error reporting**: Meaningful error messages
- **Division by zero protection**: Automatic handling in arithmetic operations

---

## [v0.3.0] - 2026-02-26

### ✨ Array Support
- **Array literals**: `[1, 2, 3, 4, 5]` syntax
- **Array indexing**: `arr[0]`, `arr[index]` 
- **Type checking**: Homogeneous array elements
- **Bytecode operations**: OpArrayCreate, OpArrayAccess, OpArrayStore, OpArrayLen

### 🔧 Technical Features
- **Lexer enhancements**: Token recognition for array syntax
- **Parser updates**: Array expression parsing
- **Type checker**: Array type validation
- **Runtime support**: Array operations in evaluator
- **Compiler support**: Array bytecode generation

---

## [v0.2.0] - 2026-02-25

### ✨ Bytecode VM & Operators
- **Complete operator suite**: Arithmetic, bitwise, comparison, logical
- **Stack-based VM**: Efficient bytecode execution
- **Control flow**: Jump instructions for loops and conditionals
- **Function calls**: Parameter passing and return values
- **Variable management**: Global and local scope handling

### 🏗️ Architecture
- **Modular compiler**: Separate compilation phases
- **IR generation**: Intermediate representation
- **Bytecode format**: Serializable instruction stream
- **Runtime integration**: Dual execution modes

---

## [v0.1.0] - 2026-02-24

### ✨ Core Language Foundation
- **Lexer**: Complete token recognition
- **Parser**: Expression parsing with precedence
- **Basic types**: int, float, string, bool
- **Variables**: let declarations
- **Functions**: Basic function definitions
- **Control structures**: if/else, for, while

### 🏗️ Project Setup
- **Go project structure**: Standard Go layout
- **CLI tool**: `kode` command-line interface
- **Build system**: Go modules
- **Testing framework**: Comprehensive test suite

---
- **Optimization**: Performance enhancements
  - Constant folding
  - Dead code elimination
  - Function inlining
  - Instruction cache optimization
- **Error Diagnostics**: Enhanced error messages
  - Show source context in errors
  - Suggest fixes for common mistakes
  - Distinguish error types clearly
  - Include root cause in messages

#### CLI Enhancements
- **Commands**: Full command set
  - `kode run` - Direct compilation and execution
  - `kode build` - Build to bytecode (default) or Go
  - `kode check` - Type-check without execution
  - `kode fmt` - Code formatting
  - `kode new` - Project scaffolding
  - `kode clean` - Remove build artifacts
  - `kode doctor` - Environment diagnostics
  - `kode version` - Version information
- **Flags and Options**:
  - `--go` - Compile to Go binary
  - `--optimize` - Enable optimizations
  - `--verbose` - Detailed output
  - `--check` - Verify without modifying
  - `--in-place` - Modify files in place

### 🔧 Internal Changes

#### Bytecode VM
- 50+ instruction opcodes covering all language features
- Stack-based architecture with O(1) per-instruction execution
- Efficient memory management with garbage collection
- Support for all data types and operations

#### Parser & Type Checker
- Two-pass type checking with function hoisting
- Complete recursive descent parser
- Full operator precedence (12 levels)
- Comprehensive type inference engine

#### IR & Code Generation
- Platform-independent intermediate representation
- Multiple backend support (bytecode and Go)
- Bytecode format (.kbc) with versioning
- Go code generation for native compilation

### 🐛 Bug Fixes
- Fixed line number tracking in imported modules
- Fixed error message clarity for module import failures
- Fixed integer division behavior
- Fixed string concatenation with null values
- Fixed function parameter passing in recursion
- Fixed array bounds checking
- Fixed enum variant matching

### 📊 Platform Support (Verified)
- ✅ **Windows** - Windows 10+ (tested)
- ✅ **macOS** - Intel and Apple Silicon (tested)
- ✅ **Linux** - glibc and musl (tested)

### 📚 Documentation (Complete)
- Architecture guide with system design diagrams
- Comprehensive syntax reference
- CLI command documentation
- Type system explanation
- Concurrency patterns guide
- Module system documentation
- Complete CHANGELOG and ROADMAP

---

## [v0.6.0] - 2026-02-20

### ✨ Added - Standard Library & Ecosystem
- Core collections: Array, Map, Set, Queue, Stack
- String manipulation functions
- Numeric library with math functions
- File I/O primitives
- Time and date handling
- Random number generation
- Error handling best practices
- Standard library module organization

### 🔄 Changed
- Improved module search paths
- Enhanced error handling in module loading
- Optimized constant folding

---

## [v0.5.0] - 2026-02-15

### ✨ Added - Concurrency
- **Goroutines**: `spawn` keyword for lightweight concurrency
- **Channels**: Type-safe `Channel<T>` for message passing
- **Select**: Channel multiplexing with `select` statement
- **Synchronization**: Mutex and atomic operations
- **CSP Model**: Communication Sequential Processes support

### 🔧 Internal
- Goroutine scheduler implementation
- Channel buffer management
- Select statement compilation

---

## [v0.4.0] - 2026-02-10

### ✨ Added - Module System
- **Imports**: Named, namespace, and wildcard import styles
- **Exports**: Public visibility for functions, types, constants
- **Module Resolution**: Path resolution and circular dependency detection
- **Module Caching**: Prevent re-parsing of modules
- **Re-exports**: Module aggregation support

### 🔄 Changed
- Enhanced parser for import/export statements
- Improved type checker for module symbols
- Better error reporting for module errors

---

## [v0.3.3] - 2026-02-05

### ✨ Added - Enums with Pattern Matching
- Enum type definitions
- Enum variant creation
- Pattern matching on enums
- Associated values in enum variants
- Bytecode support for enums

---

## [v0.3.2] - 2026-02-02

### ✨ Added - Structs
- Struct type definitions
- Struct literals
- Field access with dot notation
- Constructor methods
- Bytecode support for structs

---

## [v0.3.1] - 2026-01-30

### ✨ Added - Member Access & Array Methods
- Dot notation for member/method access
- Array `.len` property
- Foundation for methods

---

## [v0.3.0] - 2026-01-28

### ✨ Added - Data Structures
- Array literals: `[1, 2, 3]`
- Array indexing: `arr[0]`, `arr[i]`
- Array type checking
- Bytecode support for arrays

---

## [v0.2.0] - 2025-12-01

### ⚠️ Breaking Changes
- Bytecode is now default build format (changed from Go)
- REPL removed from CLI
- `.kbc` extension for bytecode files

### ✨ Added - Bytecode & VM
- Stack-based bytecode VM
- `.kbc` bytecode format
- 30+ instruction opcodes
- Bitwise operators (complete suite)
- Assignment statements
- Loop control (break/continue)

### 🔧 Internal
- Complete operator precedence (12 levels)
- Parser optimizations
- VM performance improvements

---

## [v0.1.0] - 2025-10-01

### ✨ Added - Core Language
- Lexer and parser
- Type system (basic)
- Go code generation backend
- CLI with basic commands
- Function definitions
- Control flow (if/else, loops)
- Variable declarations
- Closures and arrays
- Module imports

---

*Changelog maintained by Sreeraj V Rajesh*
*For detailed information, see [docs/](./docs/) and [README.md](./README.md)*

### ✨ Added - Enums with Pattern Matching

#### Enum Declarations
- Enum type definitions: `enum Status { Active, Inactive, Pending }`
- Enum variant creation
- Pattern matching support (foundation laid)
- Bytecode opcode: `OpEnumVariant`
- Runtime and bytecode VM support

#### Features
- Enum variants with optional associated values
- Type-safe enum handling in typer
- Support for all platforms (Windows, macOS, Linux)

#### Platform Support
- ✅ **Windows** - Fully tested and working
- ✅ **macOS** - Fully tested and working
- ✅ **Linux** - Fully tested and working

---

## [v0.3.2] - 2026-02-26

### ✨ Added - Structs with Field Access

#### Struct Declarations
- Struct type definitions: `struct Person { name: string, age: int }`
- Struct literal syntax: `Person { name: "Alice", age: 30 }`
- Field access with dot notation: `person.name`
- Parser support for struct declarations and literals
- Type checking in typer for struct fields

#### Runtime Support
- Struct instances represented as maps with metadata
- Field access through member access expressions
- Full bytecode support: `OpStructCreate`, `OpStructField`
- Runtime evaluator support for all struct operations

#### Features
- Type-safe struct field checking
- Multiple struct types in same program
- Nested field access potential
- Memory-efficient struct representation

#### Platform Support
- ✅ **Windows** - Fully tested and working
- ✅ **macOS** - Fully tested and working
- ✅ **Linux** - Fully tested and working

#### Examples
- Updated: `examples/structs_enums.kode` - Struct usage examples

---

## [v0.3.1] - 2026-02-26

### ✨ Added - Array Methods & Member Access

#### Member Access (Dot Notation)
- Dot notation for member/method access: `obj.member`
- Parser support for member access in postfix expressions
- Type checking for array methods
- Bytecode opcode: `OpMemberAccess`
- Runtime and bytecode VM support

#### Array Methods
- `.len` property - Get array length
  - Syntax: `arr.len` returns integer
  - Works with arrays of any element type
  - Constant time O(1) operation
  - Example: `let length = [1,2,3].len  // 3`

#### Platform Support
- ✅ **Windows** - Fully tested and working
- ✅ **macOS** - Fully tested and working
- ✅ **Linux** - Fully tested and working

#### Examples
- New: `examples/array_methods.kode` - Array method usage examples
- Test files: `test/array_methods_test.kode`

---

## [v0.3.0] - 2026-02-26

### ✨ Added - Data Structures

#### Arrays
- Array literal syntax: `[1, 2, 3]`
- Array indexing support: `arr[0]`, `arr[i]`
- Proper array type checking with element type validation
- Type-safe array operations in bytecode VM
- Array support for integers, floats, strings, and mixed types
- Bytecode operations: `OpArrayCreate`, `OpArrayAccess`, `OpArrayStore`, `OpArrayLen`
- Compiled example: `examples/arrays.kode`

#### Platform Support
- ✅ **Windows** - Fully tested and working
- ✅ **macOS** - Fully tested and working
- ✅ **Linux** - Fully tested and working

### ✨ Added - Type System Improvements

#### Logical Operator Coercion
- Logical operators (`&&`, `||`, `!`) now accept any type with proper coercion
- Truthy/falsy value semantics for non-boolean types
- Better type inference for logical expressions
- Improved error messages for type mismatches

### 📝 Documentation
- New: `ROADMAP_v0.3+.md` - Comprehensive feature roadmap
- New: `FEATURE_IMPLEMENTATION.md` - Implementation guide and architecture notes
- New: `examples/arrays.kode` - Array usage examples
- Updated CLI styling guide with array operations
- Type system documentation improvements

---

## [Release - v0.2] - 2026-02-26

### ⚠️ Breaking  
- Removed `repl` command and interactive REPL support from CLI
- **Bytecode is now the default build format** - `kode build <file>` now generates `.kbc` bytecode files by default instead of Go binaries
- Changed executable output extension from `.bytecode` to `.kbc` for consistency

### ✨ Added

#### Bytecode Compilation & Execution
- Bytecode compilation system with stack-based VM execution
- `.kbc` bytecode file format for portable, Go-independent execution
- `kode <file>` command to execute `.kbc` bytecode files
- Shorthand bytecode execution: `kode file.kbc` (instead of `kode file.kbc`)

#### Operators
- **Bitwise Operators (Complete Suite):**
  - Bitwise AND (`&`): `12 & 5 = 4` ✓
  - Bitwise OR (`|`): `12 | 5 = 13` ✓
  - Bitwise XOR (`^`): `12 ^ 5 = 9` ✓
  - Bitwise NOT (`~`): `~5 = -5` ✓
  - Left Shift (`<<`): `8 << 2 = 32` ✓
  - Right Shift (`>>`): `32 >> 2 = 8` ✓

#### Language Features
- Assignment statements (`var = expr`) for variable updates
- Full loop support (for/while) with proper variable increment tracking
- Function parameter inlining for efficient execution
- Two-pass type checking with function hoisting
- Break and Continue statements (basic support)
- Input function for user input with optional prompts

#### CLI Enhancements
- **Colored output** with ANSI escape codes
- **Status symbols:** ✗ for errors, ⏳ for progress, ✓ for success
- **Verbose mode** with detailed build information
- `--go` flag to use legacy Go compilation backend
- Improved error messages with line numbers and suggestions

### 🔄 Changed
- `kode build` now generates `.kbc` bytecode files by default (no Go compiler required)
- Use `kode build --go` to compile to Go binaries (legacy mode)
- For loops now properly track and increment loop variables with assignment support
- Parser updated with correct operator precedence (12 levels)
- Bytecode support expanded: Variables, arithmetic, comparisons, bitwise, logic, control flow (if/else, for, while), functions, and I/O

### 🔧 Internal
- Lexer now properly distinguishes between bitwise (`&`, `|`) and logical (`&&`, `||`) operators
- Parser implements complete operator precedence hierarchy
- Bytecode compiler supports 30+ opcodes
- VM stack-based execution engine with 6 new bitwise operation handlers
- AST now includes bitwise operator definitions

### 🐛 Bug Fixes
- Top-level `let` declarations were not being compiled, causing globals to be nil at runtime ✓
- `kode build` could leave an executable containing only a shebang when Go build failed; now stale output is removed and the build errors are surfaced ✓
- For loop infinite loop issue - postfix increment operators now properly update loop variables ✓
- OpStoreGlobal now correctly pops values from stack, maintaining proper stack discipline ✓
- Assignment statements now properly compile to bytecode ✓
- Bitwise operators now compile with correct opcodes ✓

## [0.1.0] - 2025-05-03

### ✨ Added
- `build` command to generate `.kdc` bytecode files
- `run_bytecode_file()` function to interpret compiled files
- CLI flags: `--no-run`, `--time`, `--optimize`
- Support for `try/catch` error handling
- `FunctionDef` support with optional `is_main` and `file_prefix`
- Closures and arrays implementation
- Module importing with `import` statement

### 🔄 Changed
- Unified logic for running `.kode` and `.kdc` files in `main.rs`
- AST structure updated to support new language features
- Enhanced REPL functionality and user experience

### 🐛 Fixed
- File path validation for both source and bytecode files
- REPL stability improvements
- Better syntax error messages and formatting
- Execution flow and scope handling

### 🔧 Internal
- Refactored interpreter for better performance
- Improved error handling throughout the codebase
- Enhanced documentation and comments

### ✨ Added
- Initial release of Kode programming language
- Basic lexer, parser, and interpreter implementation
- Core language features:
  - Variable declarations with `let`
  - Conditionals with `if`/`else`
  - Loops with `while` and `for`
  - Output with `print`
  - Function returns with `return`
- REPL with basic command support
- CLI commands: `run`, `repl`, `version`, `help`
- Basic error handling and reporting

---

*Changelog maintained by Sreeraj V Rajesh*
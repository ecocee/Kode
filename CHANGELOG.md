# 📦 Changelog

All notable changes to the Kode Programming Language are documented in this file.

---

## [Unreleased] - 2026-02-26

### ⚠️ Breaking
- Removed `repl` command and interactive REPL support from CLI
- **Bytecode is now the default build format** - `kode build <file>` now generates `.kbc` bytecode files by default instead of Go binaries
- Changed executable output extension from `.bytecode` to `.kbc` for consistency

### ✨ Added

#### Bytecode Compilation & Execution
- Bytecode compilation system with stack-based VM execution
- `.kbc` bytecode file format for portable, Go-independent execution
- `kode exec <file>` command to execute `.kbc` bytecode files
- Shorthand bytecode execution: `kode file.kbc` (instead of `kode exec file.kbc`)

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
- Input function for user input

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
# 📦 Changelog

All notable changes to the Kode Programming Language are documented in this file.

---

## [Unreleased] - 2026-02-26

### ⚠️ Breaking
- Removed `repl` command and interactive REPL support from CLI

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
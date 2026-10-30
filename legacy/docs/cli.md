# 📄 Kode CLI Commands Reference

This document provides usage details for all CLI commands available in the `kode` binary.

## 🔧 Commands

| Command               | Description                                |
|------------------------|--------------------------------------------|
| `kode <file>`     | Runs a `.kode` source file (type checks & executes) |
| `kode build <file>`   | Compiles `.kode` to `.kbc` bytecode (default) or Go binary with `--go` flag |
| `kode <file.kbc>`     | **Shorthand** to execute compiled `.kbc` bytecode files |
| `kode <file.kbc>`| Explicitly executes a compiled `.kbc` bytecode file |
| `kode check <file>`   | Type checks only, no compilation |
| `kode fmt <file>`     | Formats source code |
| `kode clean`          | Removes generated artifacts |
| `kode version`        | Prints the version number and build info   |
| `kode help`           | Displays CLI usage instructions            |

## ⚙️ Build Options

| Flag           | Description                                 |
|----------------|---------------------------------------------|
| `-o, --output` | Specify custom output filename              |
| `--verbose`    | Prints additional internal debug information|
| `--go`         | Use Go backend for compilation (legacy, generates binary) |
| `--llvm`       | Use LLVM backend for compilation (experimental) |
| `--release`    | Build in release mode (optimizations)       |
| `--target`     | Target platform for cross-compilation      |

## 🛴 Default Build Behavior

By default, `kode build` compiles to `.kbc` bytecode format, which:
- ✅ Requires **no Go compiler** installed on the target system
- ✅ **Faster** to compile than Go
- ✅ Can be executed with `kode <file.kbc>` shorthand
- ✅ **Portable** across platforms (Windows, macOS, Linux)
- ✅ Uses stack-based VM execution

## 📄 Output & Feedback

The CLI provides styled output with helpful symbols:

```
✗ Error messages in red with error symbol
🛳 Build status with progress indicator  
✓ Success messages with checkmark
```

### Example Build Output

```
$ kode build main.kode
🛳 Building file: main.kode
✓ Successfully built: main.kbc
```

### Example With Errors

```
$ kode build bad.kode
✗ Error parsing file: Expected expression at {5 15}
  Suggestion: Check syntax near line 5
```

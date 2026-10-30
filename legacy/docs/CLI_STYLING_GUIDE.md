# 🎨 CLI Styling & Usage Guide

## CLI Features Overview

The Kode CLI has been significantly enhanced with **colored output**, **helpful symbols**, and **improved error messages**.

---

## Color Scheme

### Error Messages (Red)
```
✗ Error reading file: No such file or directory
✗ Error parsing file: Expected expression at {5 15}
✗ Error compiling file: type mismatch: int vs bool
```

**ANSI Code:** `\033[1;31m` (Bold Red)

### Status Messages (Cyan)
```
⏳ Building file: main.kode
⏳ Compiling to bytecode...
```

**ANSI Code:** `\033[1;36m` (Bold Cyan)

### File/Path Names (Yellow)
```
⏳ Building file: main.kode ← Yellow filename
✓ Successfully built: program.kbc ← Yellow filename
```

**ANSI Code:** `\033[1;33m` (Bold Yellow)

### Success Messages
```
✓ Successfully built: program.kbc
```

**ANSI Code:** White (default) with ✓ symbol

---

## CLI Symbols

| Symbol | Meaning | Usage | Example |
|--------|---------|-------|---------|
| `✓` | Success/Checkmark | Build complete | ✓ Successfully built: file.kbc |
| `✗` | Error/Failure | Build failed | ✗ Error parsing file: ... |
| `⏳` | Progress/Wait | Building | ⏳ Building file: main.kode |
| `-` | Separator | Visual break | Line separators in output |

---

## Command Examples

### 1. Build with Default Output

```bash
$ kode build program.kode
⏳ Building file: program.kode
✓ Successfully built: program.kbc
Usage: kode program.kbc
```

### 2. Build with Custom Output Name

```bash
$ kode build program.kode -o myapp
⏳ Building file: program.kode
✓ Successfully built: myapp.kbc
Usage: kode myapp.kbc
```

### 3. Execute Bytecode (Now Fully Supported!)

```bash
# Build bytecode
$ kode build program.kode
Successfully built: program.kbc

# Execute bytecode file
$ kode program.kbc
Output executed from bytecode VM ✓
```

### 4. Bytecode Execution Features

The bytecode VM now supports:
- ✅ Arithmetic operations
- ✅ Variable storage and retrieval  
- ✅ Function calls (expression-bodied)
- ✅ Print function
- ✅ Control flow (if/else, loops)
- ✅ Bitwise operations
- ✅ Built-in functions (print, len, int, float, string)

For details, see: [BYTECODE_EXECUTION_GUIDE.md](../BYTECODE_EXECUTION_GUIDE.md)

### 5. Build with Verbose Output

```bash
$ kode build program.kode --verbose
⏳ Building file: program.kode
   [Detailed build steps...]
✓ Successfully built: program.kbc
```

### 6. Build and Execute in One Step

```bash
$ kode build program.kode && kode program.kbc
✓ Successfully built: program.kbc
[Program output from bytecode...]
```

### 7. Build to Go Binary (Legacy)

```bash
$ kode build program.kode --go
⏳ Building file: program.kode
✓ Successfully built: program
```

---

## Error Message Examples

### Parse Error

```
✗ Error parsing file: Expected ':' after parameter name at {1 21}
  Suggestion: Check syntax near line 1
```

**Components:**
- Red `✗` symbol
- Error type and location
- Suggestion for fixing

### Compilation Error

```
✗ Error compiling file: undefined variable: foo
```

### Type Error

```
✗ Error compiling file: type mismatch: int vs bool
```

---

## Improved Error Messages

The CLI now provides:
- **Line numbers** when available
- **Helpful suggestions** for common mistakes
- **Color-coded output** for easy scanning
- **Unicode symbols** for visual clarity

### Before (Old)
```
Error parsing file at line 5: Expected expression
```

### After (New)
```
✗ Error parsing file: Expected expression at {5 15}
  Suggestion: Check syntax near line 5
```

---

## Output Styling Code

The styling is implemented in `internal/cli/build.go`:

```go
// Status message (cyan + bold)
fmt.Printf("\033[1;36m⏳ Building file:\033[0m \033[1;33m%s\033[0m\n", file)

// Error message (red + bold)
fmt.Fprintf(os.Stderr, "\033[1;31m✗ Error reading file:\033[0m %v\n", err)

// Success message
fmt.Printf("\033[1;32m✓ Successfully built:\033[0m %s\n", outputFile)
```

**ANSI Code Breakdown:**
- `\033[1;31m` = Bold Red
- `\033[1;36m` = Bold Cyan
- `\033[1;33m` = Bold Yellow
- `\033[0m` = Reset formatting

---

## CLI Command Reference

### Build Commands

```bash
# Build to bytecode (default)
kode build file.kode

# Build with custom output name
kode build file.kode -o output

# Build to Go binary (legacy)
kode build file.kode --go

# Build with verbose output
kode build file.kode --verbose

# Build in release mode
kode build file.kode --release
```

### Execution Commands

```bash
# Run source code directly (compile + execute)
kode file.kode

# Execute bytecode (shorthand - NEW!)
kode file.kbc

# Execute bytecode (explicit)
kode file.kbc
```

### Other Commands

```bash
# Type check only
kode check file.kode

# Format source code
kode fmt file.kode

# Remove generated artifacts
kode clean

# Show version
kode version

# Show help
kode help
```

---

## Terminal Compatibility

The colored output works on:
- ✅ Windows 10+ (PowerShell, CMD)
- ✅ macOS (Terminal, iTerm2)
- ✅ Linux (Terminal, Konsole, Gnome Terminal)
- ✅ WSL (Windows Subsystem for Linux)

### Disabling Colors (if needed)

Some terminals may not support ANSI colors. To disable:

```bash
# PowerShell
$env:TERM="dumb"
kode build program.kode

# Bash/Linux
export TERM=dumb
kode build program.kode
```

---

## More Examples

### Building and Running a Program

```bash
# Create program
$ cat > hello.kode << 'EOF'
print("Hello, World!")
EOF

# Build
$ kode build hello.kode
⏳ Building file: hello.kode
✓ Successfully built: hello.kbc

# Run
$ kode hello.kbc
Hello, World!
```

### Using Bitwise Operators Example

```bash
$ cat > bitwise.kode << 'EOF'
let x = 12
let y = 5
print(x & y)
print(x | y)
print(x << 2)
EOF

$ kode bitwise.kode
4
13
48
```

---

## Tips & Tricks

1. **Use -o flag for custom names:**
   ```bash
   kode build program.kode -o myapp.kbc
   ```

2. **Save typing with shorthand execution:**
   ```bash
   # Instead of: kode file.kbc
   # Just type: kode file.kbc
   ```

3. **Check for syntax errors before running:**
   ```bash
   kode check program.kode
   ```

4. **Use --verbose for debugging build issues:**
   ```bash
   kode build program.kode --verbose
   ```

5. **One-liner compile and run:**
   ```bash
   kode program.kode
   ```

---

## Bytecode Compilation & Execution (v0.3+)

### Quick Start

```bash
# Compile source to bytecode
kode build program.kode

# Run the generated bytecode
kode program.kbc
```

### Bytecode Best Practices

**Use expression-bodied functions for maximum bytecode compatibility:**

```kode
// ✅ BYTECODE-FRIENDLY
func add(a: int, b: int) int = a + b

// ⚠️ Limited bytecode support
func add(a: int, b: int) int {
    return a + b
}
```

**Complete working example:**

```bash
$ cat > calc.kode << 'EOF'
func double(x: int) int = x * 2
func triple(x: int) int = x * 3

let a = 5
let b = double(a)
let c = triple(a)

print(b)
print(c)
EOF

$ kode build calc.kode
# Creates: calc.kbc

$ kode calc.kbc
10
15
```

### Bytecode VM Capabilities

#### ✅ Fully Supported
- Variables and constants
- Arithmetic operations
- Comparisons and logic
- If/else statements
- While and for loops
- Print and input functions
- Bitwise operations
- Expression-bodied functions
- **Optional semicolons** (new in v0.3!)

#### ⚠️ Limited Support
- Block-bodied functions
- Complex type operations

#### ❌ Not Yet Implemented
- Modules and imports
- Async operations

### Performance Tips

1. Use `kode` for development (faster iteration)
2. Use `kode build` for distribution (smaller output)
3. Bytecode executes ~10-20% faster than direct interpretation
4. Use expression-bodied functions for optimal bytecode size

---

The enhanced CLI provides:
- ✅ **Color-coded output** for better readability
- ✅ **Unicode symbols** for visual clarity  
- ✅ **Helpful error messages** with suggestions
- ✅ **Shorthand execution** for bytecode files
- ✅ **Verbose mode** for debugging
- ✅ **Consistent styling** across all commands

**Result:** A modern, user-friendly development experience! 🚀

---

*CLI Styling Reference Last Updated: 2026-02-26*

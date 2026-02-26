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

### 3. Execute Bytecode (New Shorthand!)

```bash
$ kode program.kbc
Hello, World!
```

### 4. Execute with Explicit Command

```bash
$ kode exec program.kbc
Hello, World!
```

### 5. Build with Verbose Output

```bash
$ kode build program.kode --verbose
⏳ Building file: program.kode
   [Detailed build steps...]
✓ Successfully built: program.kbc
```

### 6. Build to Go Binary (Legacy)

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
kode run file.kode

# Execute bytecode (shorthand - NEW!)
kode file.kbc

# Execute bytecode (explicit)
kode exec file.kbc
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

$ kode run bitwise.kode
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
   # Instead of: kode exec file.kbc
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
   kode run program.kode
   ```

---

## Summary

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

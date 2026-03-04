# Kode Bytecode Execution Guide

## Overview

The Kode language supports two execution modes:
1. **Direct Execution** (`kode file.kode`) - Uses the runtime engine
2. **Bytecode Compilation** (`kode build file.kode`) - Compiles to bytecode, then executes with the VM

Both modes support most language features, but there are some differences in what they support.

---

## Execution Modes Comparison

| Feature | `kode` | Bytecode (`kode build` + `kode exec`) |
|---------|-----------|------|
| Variables | ✅ | ✅ |
| Constants | ✅ | ✅ |
| Arithmetic | ✅ | ✅ |
| Comparisons | ✅ | ✅ |
| Bitwise Ops | ✅ | ✅ |
| If/Else | ✅ | ✅ |
| While loops | ✅ | ✅ |
| For loops | ✅ | ✅ |
| Print function | ✅ | ✅ |
| Expression-bodied functions | ✅ | ✅ |
| Block-bodied functions | ✅ | ⚠️ Limited |
| User-defined types | ✅ | ⚠️ Limited |
| Imports | ✅ | ❌ |
| Modules | ✅ | ❌ |

---

## Using Bytecode

### Building Bytecode

```bash
# Build with default output name
kode build program.kode
# Creates: program.kbc

# Build with custom output name
kode build program.kode -o myapp.kbc

# Build to bytecode and execute
kode build program.kode
kode program.kbc
```

### Executing Bytecode

```bash
# Run bytecode file
kode program.kbc

# The .kbc extension tells kode it's bytecode
```

---

## Function Types

### Expression-Bodied Functions (Recommended for Bytecode)

Expression-bodied functions use the `=` syntax and work perfectly with bytecode:

```kode
// Expression-bodied (works with bytecode!)
func add(a: int, b: int) int = a + b

func multiply(x: int, y: int) int = x * y

func isPositive(n: int) bool = n > 0

let result1 = add(5, 3)  // 8
let result2 = multiply(4, 2)  // 8
let check = isPositive(-5)  // false
```

**Benefits:**
- ✅ Works with both `run` and `build`
- ✅ Smaller bytecode size
- ✅ No special handling needed

### Block-Bodied Functions

Block-bodied functions work great with `run`, but have limited support in bytecode:

```kode
// Block-bodied (works with run, limited with bytecode)
func add(a: int, b: int) int {
    return a + b
}

func greet(name: string) string {
    let greeting = "Hello, " + name
    return greeting
}
```

---

## Complete Bytecode Example

### Source Code (hello.kode)

```kode
// Functions (use expression-bodied for bytecode)
func double(x: int) int = x * 2
func triple(x: int) int = x * 3

// Variables and expressions
let a = 5
let b = 10
let c = double(a)  // 10
let d = triple(b)  // 30

// Print results
print(a)
print(b)
print(c)
print(d)

// Control flow
if (c > a) {
    print("c is greater than a")
}

// Arithmetic
let sum = c + d  // 40
print(sum)
```

### Build and Run

```bash
# Compile to bytecode
$ kode build hello.kode
✓ Successfully built: hello.kbc

# Execute bytecode
$ kode hello.kbc
5
10
10
30
c is greater than a
40
```

---

## Common Bytecode Issues and Solutions

### Issue: "unknown opcode" Error

**Cause**: Using a language feature not yet implemented in the bytecode VM.

**Solution**: Use `kode` instead, or simplify your code to use only supported features.

### Issue: Function Output Missing

**Cause**: Block-bodied functions aren't compiled properly for bytecode.

**Solution**: Convert to expression-bodied functions using `=`:

```kode
// ❌ May not work with bytecode
func add(a: int, b: int) int {
    return a + b
}

// ✅ Works with bytecode
func add(a: int, b: int) int = a + b
```

### Issue: Variables Not Printing

**Cause**: Variable scope or storage issue in bytecode compilation.

**Solution**: Ensure variables are defined before use, and try using `run` first to diagnose.

---

## Bytecode Performance Tips

1. **Use expression-bodied functions** - They compile to fewer instructions
2. **Avoid unnecessary variables** - Each variable adds to bytecode size
3. **Pre-calculate constants** - Replace computed values with const declarations
4. **Use directly without building** - For quick testing, use `kode`

---

## Testing Bytecode

Create test files that use bytecode-compatible features:

```bash
# Create test file
cat > test.kode << 'EOF'
func add(x: int, y: int) int = x + y
func mul(x: int, y: int) int = x * y

let r1 = add(3, 4)
let r2 = mul(5, 6)

print(r1)
print(r2)

if (r1 < r2) {
    print("mul result is larger")
}
EOF

# Test with run
kode test.kode

# Test with bytecode
kode build test.kode
kode test.kbc
```

---

## Future Enhancements  

The bytecode VM is being enhanced to support:
- [ ] Block-bodied user-defined functions
- [ ] Structs and enums
- [ ] Module system
- [ ] More built-in functions
- [ ] Better error messages in bytecode execution

---

## Best Practices

### For Production / Release Builds
```bash
# Build optimized bytecode
kode build program.kode -o program.kbc

# Then distribute program.kbc
kode program.kbc
```

### For Development
```bash
# Use direct execution for immediate feedback
kode program.kode

# Switch to bytecode testing when ready
kode build program.kode
kode program.kbc
```

### For Maximum Compatibility
- Use semicolons (optional, but helpful for visual clarity)
- Use expression-bodied functions
- Stick to built-in types (int, float, string, bool)
- Use simple control flow (if/else, while, for)
- Avoid advanced features (structs, enums, modules)

---

## Summary

✅ **Recommended for Bytecode:**
- Expression-bodied functions
- Simple variables and constants
- Arithmetic and comparisons
- If/else and loops
- Print statements

❌ **Not Ready for Bytecode:**
- Block-bodied functions
- Module imports
- Advanced type systems
- Complex closures

**Use `kode` for everything until you're sure your code works with bytecode!**

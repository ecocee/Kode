# Bytecode Implementation Progress

## Overview
The Kode project now has a fully functional bytecode compiler and virtual machine. This is the default build backend for Kode programs.

## Completed Features

### ✅ Core Arithmetic Operations
- Addition (+)
- Subtraction (-)
- Multiplication (*)
- Division (/)
- Modulo (%)
- Negation (unary -)

### ✅ Comparison Operations
- Equal (==)
- Not Equal (!=)
- Less Than (<)
- Less Than or Equal (<=)
- Greater Than (>)
- Greater Than or Equal (>=)

### ✅ Logical Operations
- Logical AND (&&)
- Logical OR (||)
- Logical NOT (!)

### ✅ Bitwise Operations (NEW - Complete Suite!)
- **Bitwise AND** (`&`): `12 & 5 = 4` ✅
- **Bitwise OR** (`|`): `12 | 5 = 13` ✅
- **Bitwise XOR** (`^`): `12 ^ 5 = 9` ✅
- **Bitwise NOT** (`~`): `~5 = -6` ✅
- **Left Shift** (`<<`): `8 << 2 = 32` ✅
- **Right Shift** (`>>`): `8 >> 2 = 2` ✅

### ✅ Control Flow
- **If-Else Statements**: Full conditional branching with nested support
- **For Loops**: Complete with initialization, condition, and increment (assignment support)
- **While Loops**: Condition-based looping with state updates
- **Break/Continue**: Loop control statements (basic support)

### ✅ Function Support
- Function definitions with typed parameters: `func name(param: type) = expr`
- Function calls with argument passing
- Return values and expressions
- Function hoisting (functions can be called before definition)
- Parameter inlining for optimized execution

### ✅ Variables & Assignment
- Local and global variables with `let` statements
- Assignment statements for variable updates: `var = expr`
- Variable loading and storage

### ✅ Built-in Functions
- `print()`: Output to stdout
- `input()`: Read user input

### ✅ CLI Improvements
- **Colored output** with ANSI escape codes (red for errors, cyan for status)
- **Status symbols**: ✗ for errors, ⏳ for progress, ✓ for success
- **Verbose mode** with detailed build information
- **Shorthand execution**: `kode file.kbc` (no need for `exec`)
- **Better error messages**: Line numbers and helpful suggestions

### ✅ Data Types
- Integers
- Floating-point numbers
- Booleans (`true`, `false`)
- Strings

## Recent Fixes & Improvements

### Issue 1: Filename Duplication ✅
**Problem**: Bytecode files were created with `.kbc.kbc` extension when specifying `.kbc` output
**Fix**: Added check to prevent double `.kbc` extension

### Issue 2: Function Hoisting ✅
**Problem**: Functions called before definition caused "undefined variable" errors
**Fix**: Implemented two-pass type checking:
- First pass: Hoist all function definitions into type environment
- Second pass: Type check all statements normally

### Issue 3: Function Call Implementation ✅
**Problem**: OpCall instruction (opcode 27) was not implemented in the VM
**Fix**: Implemented function call inlining for expression-bodied functions:
- Collects function definitions from AST
- Substitutes parameters with actual arguments
- Inlines function body at call site

### Issue 4: Assignment in Loop Increment ✅
**Problem**: Loop increment expressions like `i = i + 1` failed with "unknown operator" error
**Fix**: Added special handling for assignment-like binary expressions in loop increments

## Build & Execution

### Build a Kode file to bytecode:
```bash
kode build myprogram.kode
```
This creates `myprogram.kbc`

### Execute bytecode:
```bash
kode myprogram.kbc
# or explicitly:
kode exec myprogram.kbc
```

### Verbose output:
```bash
kode build myprogram.kode --verbose
```

### Example Programs

#### Basic Arithmetic
```kode
let a = 10
let b = 3
print(a + b)   // 13
print(a * b)   // 30
```

#### Bitwise Operations
```kode
let x = 12
let y = 5
print(x & y)   // 4
print(x | y)   // 13
print(x << 2)  // 48
```

#### Loops
```kode
for (let i = 0; i < 5; i++) {
  print(i)  // prints 0 1 2 3 4
}
```

#### Functions
```kode
func square(n: int) = n * n
print(square(7))  // 49
```

## Test Files

- `test/simple.kode` - Basic variable declarations and arithmetic
- `test/simple_loop.kode` - For loop test
- `test/bytecode_simple_test.kode` - Basic bytecode features
- `test/bytecode_func_test.kode` - Function calls
- `test/bytecode_loop_test.kode` - Loop with assignment
- `examples/basic.kode` - Function definition and calling

## Architecture

### Compilation Pipeline
```
Kode Source (.kode)
    ↓
Parser (lexer/parser) → AST
    ↓
Type Checker (with function hoisting)
    ↓
Compiler → IR (Intermediate Representation)
    ↓
Bytecode Compiler → Bytecode Instructions
    ↓
Bytecode Serializer → Binary (.kbc file)
    ↓
Bytecode VM → Executes instructions
```

### Bytecode Instructions (pkg/bytecode/bytecode.go)
- **Stack Ops**: Push, Pop, Dup, Load, Store, LoadGlobal, StoreGlobal
- **Arithmetic**: Add, Sub, Mul, Div, Mod, Neg, Incr, Decr
- **Bitwise** (NEW): BitAnd, BitOr, BitXor, BitNot, BitShl, BitShr
- **Comparison**: Eq, Ne, Lt, Lte, Gt, Gte
- **Logic**: And, Or, Not
- **Control Flow**: Jmp, JmpIfFalse, JmpIfTrue, Break, Continue
- **Functions**: Call, Return, ReturnValue
- **Other**: Print, Input, Noop, Halt

## Known Limitations & Future Work

- [ ] Structs and custom types
- [ ] Array/List/Dict support
- [ ] While loop optimization
- [ ] Error handling (try-catch)
- [ ] Concurrency/spawn support
- [ ] Complex nested function calls (currently limited to expression-bodied functions)
- [ ] Import/module system
- [ ] Debugging information

## Performance Notes

Expression-bodied function calls are currently inlined at compile time, which:
- ✅ Avoids runtime function call overhead
- ✅ Enables parameter substitution and optimization
- ✅ Works well for small functions
- ⚠️  May cause code bloat for large/recursive functions (future: implement proper stack frames)

## Default Backend

The bytecode backend is now the **default** for `kode build`. Other backends (Go, LLVM) can still be selected with:
- `kode build file.kode --go` - Use Go backend
- `kode build file.kode --llvm` - Use LLVM backend

## Bytecode Format

Bytecode files (`.kbc`) are binary format with:
- Magic number: "KODE"
- Constants pool (with type tags)
- Global variable table
- Instruction stream
- Proper serialization/deserialization support

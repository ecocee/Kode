# Kode Architecture

*Last updated 2026*

## Design Philosophy

Kode is a modern systems programming language focused on:
- **Simplicity**: Clean, readable syntax
- **Performance**: Efficient bytecode execution via stack-based VM
- **Portability**: Platform-independent bytecode format
- **Compatibility**: Optional Go backend for interop

## Compilation Pipeline (Default)

```
Source Code (.kode)
       ↓
   Lexer (Tokenization)
       ↓
   Parser (AST Generation)
       ↓
   Type Checker (Inference & Checking)
       ↓
   IR Generator (Intermediate Representation)
       ↓
Bytecode Compiler (Stack-based VM instructions)
       ↓
   Bytecode (.kbc)
       ↓
   Kode VM (Execution)
       ↓
   Output
```

## Bytecode VM Architecture

The Kode Virtual Machine is **stack-based**, providing:

### Stack Operations
- Push constant values
- Pop from stack
- Duplicate stack top
- Load/Store variables

### Arithmetic Operations
- Add, Subtract, Multiply, Divide, Modulo
- Unary negation, increment, decrement

### Bitwise Operations (NEW!)
- AND, OR, XOR, NOT
- Left/Right shift operations

### Comparison & Logic
- Equality, less/greater comparisons
- Logical AND/OR/NOT operations

### Control Flow
- Conditional jumps (JmpIfFalse, JmpIfTrue)
- Unconditional jumps
- Break/Continue support

### Function Support
- Function calls with parameter passing
- Return values
- Parameter inlining optimization

### I/O Operations
- Print output
- Input reading

## Alternate Backend: Go Code Generation

For compatibility and interop, Kode supports Go code generation:

```
Kode Source → Parser → Type Checker → IR Generator → Go Code → Go Compiler → Binary
```

Use the `--go` flag to compile to Go binaries.

## Type System

- **Inference**: Automatic type deduction in most cases
- **Static Checking**: Two-pass type checking with function hoisting
- **Primitive Types**: int, float, string, bool
- **Type Safety**: Compile-time verification

## Operator Precedence

A 12-level precedence hierarchy ensures correct expression evaluation:

1. **Literals & Variables** (highest precedence)
2. **Unary** (!,  ~, -)
3. **Multiplicative** (*, /, %)
4. **Additive** (+, -)
5. **Shift** (<<, >>)
6. **Relational** (<, <=, >, >=)
7. **Equality** (==, !=)
8. **Bitwise AND** (&)
9. **Bitwise XOR** (^)
10. **Bitwise OR** (|)
11. **Logical AND** (&&)
12. **Logical OR** (||) (lowest precedence)

## Performance Characteristics

- **Bytecode Size**: Compact instruction format
- **Execution**: O(1) per instruction
- **Memory**: Stack-based allocation
- **Functions**: Parameter inlining reduces overhead
- **Startup**: Fast VM initialization

## File Format: .kbc Bytecode

```
[HEADER: "KODE"]
[CONSTANTS]   (values with type tags)
[GLOBALS]     (variable name → index mapping)
[INSTRUCTIONS] (opcode + arguments stream)
```

The .kbc format is:
- **Portable** across platforms
- **Compact** and efficient
- **Executable** by the Kode VM
- **Self-contained** (no external dependencies)

## Ecosystem Vision

Future enhancements include:
- **Concurrency**: Goroutines and channels
- **Standard Library**: Expanded built-in functions
- **Package System**: Module management
- **JIT Compilation**: Dynamic optimization
- **IDE Support**: Language server protocol
- **Debugging**: Line numbers and debug symbols
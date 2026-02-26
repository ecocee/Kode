# Kode Language Feature Implementation Guide
## v0.3 Development Notes

### Architecture Overview

```
Source Code (.kode)
    ↓
Lexer (tokenization + string validation)
    ↓
Parser (AST construction)
    ↓
Typer (semantic analysis & type checking)
    ↓
Compiler (IR generation)
    ↓
Bytecode Compiler (bytecode generation)
    ↓
Bytecode VM (execution)
```

### Type System Design

**Core Types:**
- `int` - 64-bit integers (signed)
- `float` - 64-bit floats  
- `bool` - True/false
- `string` - UTF-8 strings
- `void` - No return value
- `[]T` - Array of type T
- `{key: T}` - Dict/map type (future)

**Type Coercion Rules:**
1. Integers are truthy if non-zero
2. Floats are truthy if non-zero
3. Strings are truthy if non-empty
4. Bools used as-is

**Logical Operators (New Semantics v0.3):**
- `a && b` - Returns first falsy value, or last value if all truthy
- `a || b` - Returns first truthy value, or last value if all falsy
- `!a` - Logical NOT (works with any type via coercion)

---

### Features Being Implemented

#### 1. Type Checking Improvements (v0.3-a1)

**Current Status:** Logical operators now accept any type (with proper coercion)

**Changes Made:**
- Updated typer to allow int/string/any type for logical operators
- Maintains type safety through inference
- Better error messages for type mismatches in other operations

**Test Case:**
```kode
let x = 5 && 10        // true (both truthy)
let y = 0 || 42        // 42 (0 is falsy)
let z = !0             // true (0 is falsy, so !0 is true)
```

---

#### 2. Nested Loop Break/Continue (v0.3-a2) - UPCOMING

**Current Feature:** Basic break/continue work
**Improvement:** Support labeled breaks for nested loops

**Proposed Syntax:**
```kode
'outer: for i = 0; i < 10; i++ {
  for j = 0; j < 10; j++ {
    if (condition) {
      break 'outer      // breaks out of both loops
    }
    continue 'outer     // continues outer loop
  }
}
```

**Implementation Plan:**
1. Add label token to lexer
2. Add label parsing to parser  
3. Update bytecode with label support
4. Implement in VM execution

---

#### 3. Array Support (v0.3-b1) - UPCOMING

**Current Status:** Partial AST support via ArrayExpr, ArrayAccessExpr

**Array Syntax:**
```kode
// Declaration and initialization
let arr: []int = [1, 2, 3, 4, 5]

// Access
print(arr[0])       // 1

// Modification
arr[0] = 10

// Built-in operations
arr.push(6)
arr.pop()
arr.len()

// Iteration
for i = 0; i < arr.len(); i++ {
  print(arr[i])
}
```

**Implementation Tasks:**
1. [ ] Parse array literals: `[1, 2, 3]`
2. [ ] Add ArrayType to typer
3. [ ] Compile ArrayExpr to bytecode
4. [ ] Implement array indexing (OpArrayAccess)
5. [ ] Add array assignment (OpArrayStore)
6. [ ] Implement `.len()` method
7. [ ] Implement `.push()` method
8. [ ] Add runtime array support in VM

**Bytecode Operations (New):**
- `OpArrayCreate` - Create array from stack values
- `OpArrayAccess` - Get element at index
- `OpArrayStore` - Set element at index
- `OpArrayLen` - Get array length
- `OpArrayPush` - Push value to array  
- `OpArrayPop` - Pop value from array

---

#### 4. Struct Support (v0.3-b2) - UPCOMING

**Struct Syntax:**
```kode
struct Point {
  x: int,
  y: int,
}

struct Person {
  name: string,
  age: int,
}

// Creation
let p = Point { x: 10, y: 20 }

// Field access
print(p.x)

// Field update
p.x = 15

// Methods (future)
impl Point {
  func distance() = sqrt(x^2 + y^2)
}
```

---

#### 5. Enum Support (v0.3-b3) - UPCOMING

**Enum Syntax:**
```kode
enum Color {
  Red,
  Green,
  Blue,
}

enum Result {
  Ok,
  Error,
}

let color = Color::Red

// Pattern matching (future)
match color {
  Color::Red => print("Red"),
  Color::Green => print("Green"),
  _ => print("Other"),
}
```

---

#### 6. Try-Catch/Error Handling (v0.3-c1) - UPCOMING

**Syntax:**
```kode
try {
  let result = divideByZero(5, 0)
  print(result)
} catch err {
  print("Error:", err)
}

// Throw
throw "Division by zero"
```

---

#### 7. Spawn/Concurrency (v0.3-d1) - UPCOMING

**Syntax:**
```kode
spawn {
  print("Running concurrently")
}

spawn myFunction()

// Future: channels
let ch = channel(int)
spawn {
  ch <- 42
}
value = <- ch
```

---

### Performance Optimization Priorities

1. **Constant Folding** - Evaluate constants at compile-time
2. **Dead Code Elimination** - Remove unused code paths
3. **String Interning** - Deduplicate identical strings
4. **Instruction Caching** - Cache frequently used bytecode
5. **Loop Unrolling** - Unroll simple loops

---

### Testing Requirements

#### Unit Tests
- Array operations
- Struct field access
- Enum variant handling
- Try-catch flow
- Labeled breaks

#### Integration Tests
- Complex data structures
- Error propagation
- Concurrent operations
- Mixed feature usage

---

### Documentation Updates Needed

1. **SYNTAX.md** - Add array, struct, enum, try-catch syntax
2. **ARCHITECTURE.md** - Explain new bytecode operations
3. **EXAMPLES/** - Create comprehensive examples:
   - `arrays.kode`
   - `structs.kode`
   - `enums.kode`
   - `error_handling.kode`
   - `concurrency.kode`
4. **API Reference** - Document built-in methods

---

### Breaking Changes

**NONE** - All new features are additive and backward compatible

---

### Timeline Estimate (Total: 4-6 weeks)

- **Week 1-2:** Arrays + Structs (highest impact)
- **Week 2-3:** Enums + Try-Catch
- **Week 3-4:** Labeled breaks + optimization
- **Week 4-5:** Spawn/concurrency basics
- **Week 5-6:** Testing, documentation, polish

---

### Known Limitations (Current v0.2)

- No generics (type parameters)
- No trait system (method dispatch)
- No module imports
- No garbage collection
- Limited error types
- No pattern matching
- No async/await syntax

---

### Next Steps

1. Finalize array implementation  
2. Add struct compilation support
3. Implement enum pattern matching
4. Add try-catch runtime support
5. Update all documentation
6. Create comprehensive examples
7. Performance profiling and optimization


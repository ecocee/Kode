# Kode Language - Quick Syntax Reference

## ✅ Functions (Correct Syntax)

### Without Semicolons (Modern Style)
```kode
func add(a: int, b: int) int {
    return a + b
}

func greet(name: string) string {
    return "Hello, " + name
}

let sum = add(5, 3)
print(sum)
```

### With Semicolons (Legacy Style)  
```kode
func add(a: int, b: int) int {
    return a + b;
}

func greet(name: string) string {
    return "Hello, " + name;
}

let sum = add(5, 3);
print(sum);
```

## ✅ Control Flow (Correct Syntax)

### If Statements (Parentheses Required)
```kode
// ✅ CORRECT
if (x > 5) {
    print("greater")
}

// ❌ WRONG - missing parentheses
// if x > 5 {
//     print("greater")
// }
```

### Loops
```kode
// While loop
while (condition) {
    // code
}

// For loop - requires semicolon structure
for (let i = 0; i < 10; i = i + 1) {
    print(i)
}
```

## ✅ Variables (No Semicolons Required)

### Let (Mutable Variables)
```kode
let x = 10
let y: int = 20
let name = "Alice"
```

### Const (Constants)
```kode
const PI: float = 3.14159
const MAX: int = 100
```

## ✅ Imports (No Semicolons Required)

```kode
import "module"
import "module" as m
import { func1, func2 } from "module"
```

## ❌ Common Mistakes

| Mistake | Correct |
|---------|---------|
| `fn add(a: int) {...}` | `func add(a: int) int {...}` |
| `if x > 5 { ... }` | `if (x > 5) { ... }` |
| `print(x);` (in print) | `print(x)` (optional) |

## 📝 Key Rules

1. **Use `func` keyword**, not `fn`
2. **Specify return types** for functions
3. **Use parentheses** around if/while conditions
4. **Semicolons are optional** everywhere
5. **Type annotations** use colons: `name: type`
6. **Variables are mutable** by default (use `let`)

## 🎯 Summary

The Kode language now offers:
- ✅ Optional semicolons for cleaner code
- ✅ Backward compatibility with old code
- ✅ Better error messages with context
- ✅ Modern syntax inspired by Go and Rust

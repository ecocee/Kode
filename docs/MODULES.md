# Module System & Import/Export Guide

> **Status:** New Feature (v0.4+)  
> **Platforms:** Windows, macOS, Linux

## Overview

Kode's module system enables code organization and reusability through import and export statements. Functions, variables, structs, enums, and types can be exported from one file and imported into another.

---

## Quick Start

### Creating a Module (math.kode)

```kode
// Export individual functions
export func add(a: int, b: int) -> int {
    a + b
}

export func multiply(a: int, b: int) -> int {
    a * b
}

// Internal function (not exported)
func helper(x: int) -> int {
    x * 2
}

// Export variables and constants
export let PI = 3.14159;
export const MAX_VALUE: int = 100;
```

### Using a Module (main.kode)

```kode
// Import all exports with alias
import * as math from "math"

// Usage
func main() {
    print(math.add(5, 3))           // 8
    print(math.multiply(4, 7))      // 28
    print(math.PI)                  // 3.14159
}
```

---

## Import Syntax Styles

### Style 1: Default Module Import

**Single import of entire module:**

```kode
import "math"
```

Imports all exports into current namespace. Function calls: `add(5, 3)`

### Style 2: Aliased Module Import

**Import with alias:**

```kode
import "math" as m
```

All exports available under `m` prefix. Function calls: `m.add(5, 3)`

### Style 3: Named Imports (Destructuring)

**Import specific exports:**

```kode
import { add, multiply, PI } from "math"
```

Only specified items imported. Direct access: `add(5, 3)`, `multiply(x, y)`, `PI`

### Style 4: Named Imports with Aliases

**Rename specific imports:**

```kode
import { add as sum, multiply as product } from "math"
```

Imported items renamed: `sum(5, 3)`, `product(x, y)`

### Style 5: Namespace Import

**Import all as namespace:**

```kode
import * from "math"
```

All exports in current namespace. Equivalent to Style 1 without specifying module name.

### Style 6: Mixed Imports in a Single File

```kode
// Multiple imports
import "utils"
import { read, write } from "io"
import * as math from "math"
import { log as debug } from "debug"

func main() {
    // Use all imported items
}
```

---

## Export Syntax Styles

### Style 1: Individual Exports

```kode
export func add(a: int, b: int) -> int {
    a + b
}

export func multiply(a: int, b: int) -> int {
    a * b
}

export let PI = 3.14159;

export const MAX: int = 100;

export struct Point {
    x: int,
    y: int
}

export enum Color {
    Red,
    Green,
    Blue
}
```

### Style 2: Export Block

```kode
export {
    add,
    multiply,
    PI,
    MAX
}
```

This assumes `add`, `multiply`, `PI`, `MAX` are already declared.

### Style 3: Re-export

**Re-export items from another module:**

```kode
// In module-aggregator.kode
export { add, multiply } from "math"
export { read, write } from "io"
export { log } from "debug"
```

### Style 4: Mixed Exports

```kode
// Some items are exported, others are not
fn privateHelper() {
    // Not exported - internal only
}

export func publicAPI() {
    // Calls privateHelper internally
    privateHelper()
}

let internalVar = 10

export let configValue = 50
```

---

## Module Structure Examples

### Example 1: Simple Math Module

**math.kode:**
```kode
export func add(a: int, b: int) -> int {
    a + b
}

export func subtract(a: int, b: int) -> int {
    a - b
}

export func multiply(a: int, b: int) -> int {
    a * b
}

export func divide(a: int, b: int) -> int {
    if b == 0 {
        print("Error: Division by zero")
        0
    } else {
        a / b
    }
}
```

**Usage in main.kode:**
```kode
import { add, multiply } from "math"

func main() {
    let result = add(10, 5)
    print(result)           // 15
    
    let product = multiply(3, 4)
    print(product)          // 12
}
```

### Example 2: Data Structures Module

**data.kode:**
```kode
export struct Person {
    name: string,
    age: int,
    email: string
}

export struct Address {
    street: string,
    city: string,
    zipcode: string
}

export func createPerson(name: string, age: int, email: string) -> Person {
    Person { name: name, age: age, email: email }
}

export func getPersonInfo(p: Person) -> string {
    p.name  // Simplified - would build full string
}
```

**Usage:**
```kode
import { Person, createPerson, Address } from "data"

func main() {
    let person = createPerson("Alice", 30, "alice@example.com")
    print(person.name)      // Alice
    
    let address = Address {
        street: "123 Main St",
        city: "Boston",
        zipcode: "02101"
    }
}
```

### Example 3: Constants and Configuration Module

**config.kode:**
```kode
export const APP_NAME: string = "MyApp";
export const VERSION: string = "1.0.0";
export const DEBUG_MODE: bool = true;
export const MAX_CONNECTIONS: int = 100;

export let DEFAULT_TIMEOUT: int = 5000;
export let RETRY_COUNT: int = 3;

export func getConfig() -> string {
    APP_NAME
}
```

**Usage:**
```kode
import * as config from "config"

func main() {
    print(config.APP_NAME)          // MyApp
    print(config.VERSION)           // 1.0.0
    print(config.MAX_CONNECTIONS)   // 100
}
```

### Example 4: Large Project with Multiple Modules

**Directory structure:**
```
project/
├── main.kode
├── modules/
│   ├── math.kode
│   ├── utils.kode
│   ├── config.kode
│   └── db.kode
└── types/
    ├── models.kode
    └── errors.kode
```

**main.kode:**
```kode
import { add, multiply } from "modules/math"
import { formatString, trim } from "modules/utils"
import * as config from "modules/config"
import { User, Post } from "types/models"
import { DatabaseError } from "types/errors"

func main() {
    let sum = add(10, 20)
    let text = formatString("hello world")
    print(config.APP_NAME)
}
```

---

## Module Resolution

Kode resolves module paths relative to:

1. **Relative to current file:**
   ```kode
   import "utils"              // Same directory
   import "./utils"            // Explicit current directory
   import "../common"          // Parent directory
   ```

2. **Subdirectories:**
   ```kode
   import "modules/math"       // modules/math.kode
   import "types/models"       // types/models.kode
   ```

3. **Built-in modules (future):**
   ```kode
   import { print, input } from "std"
   import { read, write } from "fs"
   import { now } from "time"
   ```

---

## Circular Imports

Circular imports are **NOT allowed** and will result in a compilation error:

```kode
// module_a.kode
import { funcB } from "module_b"

export func funcA() {
    funcB()
}

// module_b.kode  
import { funcA } from "module_a"  // ERROR: Circular dependency!

export func funcB() {
    funcA()
}
```

**Solution:** Refactor to eliminate circular dependency by creating a third module.

---

## Visibility Rules

- **Exported items:** Available when imported into other modules
- **Non-exported items:** Private to their module, not accessible from other modules
- **Re-exports:** Items can be re-exported to aggregate multiple modules

### Example:

```kode
// private.kode - NOT exported
fn internalHelper() {
    print("Helper")
}

// main.kode
import "private"

// ERROR: Cannot call internalHelper() - not exported
internalHelper()

// OK: Can call from within the module
fn useHelper() {
    internalHelper()  // This works locally
}
```

---

## Best Practices

1. **Clear exports:** Explicitly list what's exported from each module
2. **Avoid circular deps:** Design module hierarchy to prevent cycles
3. **Group related functionality:** Keep similar functions in same module
4. **Use meaningful names:** Clear function/variable names in exports
5. **Document modules:** Add comments explaining each module's purpose

### Good Module Design:

```kode
// math_utils.kode - Clear focused module
export func add(a: int, b: int) -> int { a + b }
export func sub(a: int, b: int) -> int { a - b }
export func mul(a: int, b: int) -> int { a * b }

// Not recommended: Mixed functionality
export func add(a: int, b: int) -> int { ... }
export func read_file(path: string) -> string { ... }
export func send_http(url: string) -> string { ... }
```

---

## CLI Commands

```bash
# Run program with module system
kode main.kode

# Build with modules
kode build main.kode

# Check modules for errors
kode check main.kode

# Format module files
kode fmt -r .             # Format recursively
```

---

## Limitations & Future Work

### Current Limitations (v0.4)

- ⏳ No standard library modules yet (`std`, `fs`, `http`)
- ⏳ No re-exports from modules
- ⏳ No conditional exports
- ⏳ No type-only exports/imports

### Planned (v0.5+)

- 🔜 Built-in modules (std, fs, http, time, etc.)
- 🔜 Re-exports from multiple modules
- 🔜 Type-only imports for better static analysis
- 🔜 Module-level initialization functions
- 🔜 Package management and dependency resolution
- 🔜 Namespace isolation improvements

---

## Troubleshooting

### "Module not found"
- Check file path and extension (.kode)
- Verify relative paths are correct
- Ensure file exists in expected location

### "Item not exported"
- Verify item has `export` keyword in source module
- Check correct import syntax is used
- Ensure module is imported

### "Circular dependency detected"
- Review import dependencies
- Refactor to eliminate circular imports
- Create intermediate module if needed

### "Duplicate export"
- Each item can only be exported once
- Check for `export` statements with same name
- Use aliases if needed in re-exports

---

## See Also

- [docs/ARCHITECTURE.md](./ARCHITECTURE.md) - Compilation pipeline
- [docs/syntax.md](./syntax.md) - Language syntax reference
- [docs/cli.md](./cli.md) - CLI reference

---

**Last Updated:** 2026-02-26  
**Status:** New Feature  
**Maintainer:** Kode Team

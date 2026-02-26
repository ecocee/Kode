# 🧠 Kode Language Syntax & Grammar

> A complete reference for the Kode programming language syntax

**Created by Sreeraj V Rajesh**

---

## 📋 Table of Contents

- [Basic Syntax](#-basic-syntax)
- [Comments](#-comments)
- [Variables](#-variables)
- [Data Types](#-data-types)
- [Operators](#-operators)
- [Control Flow](#-control-flow)
- [Functions](#-functions)
- [Arrays](#-arrays)
- [Closures](#-closures)
- [Error Handling](#-error-handling)
- [Modules](#-modules)
- [Keywords](#-keywords)

---

## 📝 Basic Syntax

Kode uses a clean, familiar syntax with these core rules:

- Each statement ends with a semicolon (`;`)
- Code blocks are enclosed in curly braces (`{}`)
- Variables are declared with `let`
- Functions are defined with `fn`
- Whitespace is generally ignored

Example:
```kode
fn main() {
    let x = 42;
    print x;
}
```

---

## 💬 Comments

```kode
// Single-line comment

/* Multi-line
   comment
   block */

/* Comments can be /* nested */ like this */
```

---

## 📊 Variables

Variables in Kode are dynamically typed and declared using the `let` keyword.

```kode
// Variable declaration
let name = "Alice";
let age = 30;
let isStudent = true;

// Reassignment (no 'let' keyword needed)
age = 31;
isStudent = false;
```

Variables can be reassigned to values of different types:

```kode
let x = 10;
x = "now a string";  // Valid in Kode
```

---

## 🧩 Data Types

Kode supports these primitive data types:

### Integers
```kode
let age = 30;
let negative = -15;
```

### Floats
```kode
let pi = 3.14159;
let temperature = -2.5;
```

### Booleans
```kode
let isActive = true;
let isComplete = false;
```

### Strings
```kode
let name = "John Smith";
let greeting = "Hello, world!";
```

### Arrays
```kode
let numbers = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Charlie"];
let mixed = [1, "two", true, 4.5];  // Different types allowed
```

### Functions
```kode
let add = fn(a, b) { return a + b; };
```

### Null/Undefined Equivalent
```kode
let empty = null;  // Not yet implemented in v0.2.0
```

---

## ➗ Operators

### Arithmetic Operators
```kode
let a = 10;
let b = 3;

let sum = a + b;        // Addition: 13
let difference = a - b; // Subtraction: 7
let product = a * b;    // Multiplication: 30
let quotient = a / b;   // Division: 3 (integer division)
let remainder = a % b;  // Modulo: 1
```

### Comparison Operators
```kode
let x = 5;
let y = 10;

let isEqual = x == y;        // Equal to: false
let isNotEqual = x != y;     // Not equal to: true
let isGreater = x > y;       // Greater than: false
let isLess = x < y;          // Less than: true
let isGreaterEqual = x >= y; // Greater than or equal to: false
let isLessEqual = x <= y;    // Less than or equal to: true
```

### Logical Operators
```kode
let a = true;
let b = false;

let andResult = a && b; // Logical AND: false
let orResult = a || b;  // Logical OR: true
let notResult = !a;     // Logical NOT: false
```

### Assignment Operator
```kode
let x = 5;  // Assign 5 to x
x = x + 3;  // Reassign with value 8
```

### String Concatenation
```kode
let firstName = "John";
let lastName = "Doe";
let fullName = firstName + " " + lastName;  // "John Doe"
```

---

## 🔄 Control Flow

### If-Else Statements
```kode
if (condition) {
    // Code to execute when condition is true
} else if (anotherCondition) {
    // Code to execute when anotherCondition is true
} else {
    // Code to execute when all conditions are false
}
```

Example:
```kode
let age = 18;

if (age >= 18) {
    print "You are an adult";
} else if (age >= 13) {
    print "You are a teenager";
} else {
    print "You are a child";
}
```

### While Loops
```kode
while (condition) {
    // Code to execute repeatedly while condition is true
}
```

Example:
```kode
let count = 1;
while (count <= 5) {
    print count;
    count = count + 1;
}
```

### For Loops
```kode
for (initialization; condition; update) {
    // Code to execute repeatedly
}
```

Example:
```kode
for (let i = 0; i < 5; i = i + 1) {
    print i;
}
```

### Break and Continue
*Not implemented in v0.2.0*

---

## 🔧 Functions

### Function Declaration
```kode
fn functionName(parameter1, parameter2) {
    // Function body
    return value;  // Optional return statement
}
```

Example:
```kode
fn add(a, b) {
    return a + b;
}

let result = add(5, 3);  // 8
```

### Main Function
```kode
fn main() {
    // Entry point of the program
}
```

### Functions with No Return
```kode
fn greet(name) {
    print "Hello, " + name;
    // No return statement needed
}
```

### Functions as Values
```kode
let multiply = fn(a, b) {
    return a * b;
};

let result = multiply(4, 5);  // 20
```

---

## 📦 Arrays

### Array Declaration
```kode
let emptyArray = [];
let numbers = [1, 2, 3, 4, 5];
let mixed = [1, "two", true];
```

### Array Access
```kode
let fruits = ["apple", "banana", "cherry"];
let firstFruit = fruits[0];  // "apple" (zero-indexed)
```

### Array Modification
```kode
let numbers = [1, 2, 3];
numbers[1] = 99;  // Now [1, 99, 3]
```

*Note: Kode v0.2.0 does not include built-in array methods like push, pop, etc.*

---

## 🧮 Closures

Closures are anonymous functions that can access variables from their containing scope.

```kode
fn createCounter() {
    let count = 0;
    return fn() {
        count = count + 1;
        return count;
    };
}

let counter = createCounter();
print counter();  // 1
print counter();  // 2
```

### Passing Functions as Arguments
```kode
fn applyOperation(a, b, operation) {
    return operation(a, b);
}

let sum = applyOperation(5, 3, fn(x, y) {
    return x + y;
});  // 8

let product = applyOperation(5, 3, fn(x, y) {
    return x * y;
});  // 15
```

---

## ⚠️ Error Handling

Kode provides basic error handling through try-catch blocks:

```kode
try {
    // Code that might cause an error
    riskyOperation();
} catch {
    // Error handling code
    print "An error occurred";
}
```

Example:
```kode
fn divide(a, b) {
    if (b == 0) {
        // This would trigger an error
        return a / b;
    }
    return a / b;
}

fn main() {
    try {
        print divide(10, 0);
    } catch {
        print "Division by zero error";
    }
}
```

---

## 📚 Modules

### Module Import
```kode
import moduleName;
```

Example:
```kode
// In math.kode
fn square(x) {
    return x * x;
}

// In main.kode
import math;
print math.square(5);  // 25
```

*Note: Kode v0.2.0 has basic module support without namespaces or selective imports.*

---

## 🔑 Keywords

Kode reserves the following keywords:

| Keyword | Description |
|---------|-------------|
| `let` | Variable declaration |
| `fn` | Function definition |
| `return` | Return a value from a function |
| `if` | Start a conditional block |
| `else` | Alternative branch in conditional |
| `while` | Start a while loop |
| `for` | Start a for loop |
| `print` | Output value to console |
| `true` | Boolean literal |
| `false` | Boolean literal |
| `import` | Import a module |
| `try` | Begin try-catch block |
| `catch` | Handle errors from try block |

---

*Created with ❤️ by Sreeraj V Rajesh*

© 2025 Kode Programming Language
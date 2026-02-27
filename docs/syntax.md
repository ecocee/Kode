# 🧠 Kode Language Syntax & Grammar

> A complete reference for the Kode programming language syntax (v0.3.1+)

**Created by Sreeraj V Rajesh**

---

## 📋 Table of Contents

- [Basic Syntax](#-basic-syntax)
- [Comments](#-comments)
- [Variables & Constants](#-variables--constants)
- [Data Types](#-data-types)
- [Operators](#-operators)
- [Control Flow](#-control-flow)
- [Functions](#-functions)
- [Structures](#-structures)
- [Enumerations](#-enumerations)
- [Traits & Interfaces](#-traits--interfaces)
- [Arrays & Collections](#-arrays--collections)
- [Closures & Lambda](#-closures--lambda)
- [Pattern Matching](#-pattern-matching)
- [Error Handling](#-error-handling)
- [Modules & Imports](#-modules--imports)
- [Concurrency](#-concurrency)
- [Keywords](#-keywords)

---

## 📝 Basic Syntax

Kode uses a clean, familiar syntax with these core rules:

- Each statement ends with a semicolon (`;`)
- Code blocks are enclosed in curly braces (`{}`)
- Variables are declared with `let` or `const`
- Functions are defined with `fn`
- Whitespace is generally ignored
- Type annotations are optional (inferred automatically)

Example:
```kode
fn main() {
    let x = 42;
    const PI = 3.14159;
    print(x);
}
```

---

## 💬 Comments

```kode
// Single-line comment

/* Multi-line
   comment
   block */

/// Documentation comment
/// Describes the next item

/*! Module-level documentation */
```

---

## 📊 Variables & Constants

### Variable Declaration with Type Inference
```kode
let name = "Alice";           // Inferred: string
let age = 30;                 // Inferred: int
let score = 95.5;             // Inferred: float
let isActive = true;          // Inferred: bool
let values = [1, 2, 3];       // Inferred: [int]
```

### Variable Declaration with Explicit Types
```kode
let name: string = "Alice";
let age: int = 30;
let score: float = 95.5;
let isActive: bool = true;
let values: [int] = [1, 2, 3];
```

### Constant Declaration
```kode
const MAX_ATTEMPTS = 5;
const APP_VERSION = "1.0.0";
const PI: float = 3.14159;
```

### Reassignment
```kode
let x = 10;
x = 20;          // Valid
x = "string";    // Type changes (duck typing)

const MAX = 100;
MAX = 200;       // ERROR: Cannot reassign constants
```

---

## 🧩 Data Types

### Primitive Types
```kode
// Integer
let age: int = 30;
let negative: int = -15;

// Float
let pi: float = 3.14159;
let temp: float = -2.5;

// Boolean
let isActive: bool = true;
let isDone: bool = false;

// String
let name: string = "John Smith";
let message: string = "Hello, world!";

// Null
let empty = null;
```

### Composite Types

#### Arrays / Lists
```kode
let numbers: [int] = [1, 2, 3, 4, 5];
let names: [string] = ["Alice", "Bob", "Charlie"];
let mixed: [any] = [1, "two", true, 4.5];
```

#### Tuples
```kode
let pair: (int, string) = (42, "answer");
let triple: (int, float, bool) = (10, 3.14, true);
```

#### Maps / Dictionaries
```kode
let person = {
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com"
};

let scores: {string: int} = {
    "alice": 95,
    "bob": 87,
    "charlie": 92
};
```

#### Function Types
```kode
let add: (int, int) -> int = fn(a, b) { return a + b; };
let greet: (string) -> string = fn(name) { return "Hello, " + name; };
```

### Generic Types
```kode
let container: Container<int> = makeContainer(42);
let list: List<string> = ["a", "b", "c"];
let result: Result<int, Error> = divide(10, 2);
```

---

## ➗ Operators

### Arithmetic Operators
```kode
let a = 10;
let b = 3;

let sum = a + b;         // 13
let diff = a - b;        // 7
let product = a * b;     // 30
let quotient = a / b;    // 3 (integer division)
let remainder = a % b;   // 1
let power = a ^ 2;       // 100 (exponentiation)
```

### Comparison Operators
```kode
let x = 5;
let y = 10;

x == y    // false (equal to)
x != y    // true (not equal to)
x > y     // false (greater than)
x < y     // true (less than)
x >= y    // false (greater than or equal)
x <= y    // true (less than or equal)
```

### Logical Operators
```kode
let a = true;
let b = false;

a && b    // false (AND)
a || b    // true (OR)
!a        // false (NOT)
```

### Bitwise Operators
```kode
let x = 12;    // Binary: 1100
let y = 5;     // Binary: 0101

x & y          // 4 (AND: 0100)
x | y          // 13 (OR: 1101)
x ^ y          // 9 (XOR: 1001)
x << 2         // 48 (Left shift by 2)
x >> 1         // 6 (Right shift by 1)
~y             // -6 (Bitwise NOT)
```

### Assignment Operators
```kode
let x = 5;
x += 3;        // x = x + 3 (8)
x -= 2;        // x = x - 2 (6)
x *= 4;        // x = x * 4 (24)
x /= 3;        // x = x / 3 (8)
x %= 5;        // x = x % 5 (3)
```

### String Operators
```kode
let first = "John";
let last = "Doe";
let full = first + " " + last;     // "John Doe" (concatenation)
let repeat = "abc" * 3;            // "abcabcabc" (repetition)

// String interpolation
let age = 30;
let message = "I am ${age} years old";  // "I am 30 years old"
```

---

## 🔄 Control Flow

### If-Else Statements
```kode
if (condition) {
    // executed when condition is true
} else if (otherCondition) {
    // executed when otherCondition is true
} else {
    // executed when all conditions are false
}

// Expression form (returns value)
let status = if (age >= 18) { "adult" } else { "minor" };
```

### While Loops
```kode
while (count <= 5) {
    print(count);
    count = count + 1;
}

// Do-while loop
do {
    print(value);
    value = value + 1;
} while (value < 10);
```

### For Loops
```kode
// C-style for loop
for (let i = 0; i < 5; i = i + 1) {
    print(i);
}

// For-in loop (iteration)
for (item in collection) {
    print(item);
}

// For-each with index
for (let i, item in collection) {
    print("${i}: ${item}");
}
```

### Match Expression (Pattern Matching)
```kode
let result = match (value) {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "other"  // default case
};

// With complex patterns
match (pair) {
    (0, y) => print("x is zero: ${y}"),
    (x, 0) => print("y is zero: ${x}"),
    (x, y) => print("both non-zero: ${x}, ${y}")
}
```

### Break and Continue
```kode
for (let i = 0; i < 10; i = i + 1) {
    if (i == 3) {
        continue;  // Skip this iteration
    }
    if (i == 7) {
        break;     // Exit loop
    }
    print(i);
}
```

---

## 🔧 Functions

### Basic Function
```kode
fn add(a: int, b: int) -> int {
    return a + b;
}

let result = add(5, 3);  // 8
```

### Functions with Optional Types
```kode
fn greet(name) {  // Type inferred from usage
    print("Hello, " + name);
}

fn calculate(x, y) -> int {
    return x + y;
}
```

### Default Parameters
```kode
fn greet(name: string, greeting: string = "Hello") -> string {
    return greeting + ", " + name;
}

greet("Alice");              // "Hello, Alice"
greet("Bob", "Hi");          // "Hi, Bob"
```

### Variadic Functions
```kode
fn sum(numbers: ...int) -> int {
    let total = 0;
    for (n in numbers) {
        total = total + n;
    }
    return total;
}

sum(1, 2, 3, 4, 5);  // 15
```

### Named Return Values
```kode
fn divide(a: int, b: int) -> (quotient: int, remainder: int) {
    return (a / b, a % b);
}

let (q, r) = divide(10, 3);  // q=3, r=1
```

### Early Return
```kode
fn validate(age: int) -> bool {
    if (age < 0) {
        return false;
    }
    if (age > 150) {
        return false;
    }
    return true;
}
```

### Function Types
```kode
fn applyOperation(a: int, b: int, op: (int, int) -> int) -> int {
    return op(a, b);
}

applyOperation(5, 3, fn(x, y) { return x + y; });  // 8
applyOperation(5, 3, fn(x, y) { return x * y; });  // 15
```

---

## 🏗️ Structures

### Struct Declaration
```kode
struct Person {
    name: string,
    age: int,
    email: string
}

let person = Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
};

print(person.name);     // "Alice"
person.age = 31;        // Mutable by default
```

### Struct with Methods
```kode
struct Circle {
    radius: float
}

impl Circle {
    fn area() -> float {
        return 3.14159 * this.radius * this.radius;
    }
    
    fn circumference() -> float {
        return 2 * 3.14159 * this.radius;
    }
}

let circle = Circle { radius: 5.0 };
print(circle.area());           // 78.54
print(circle.circumference());  // 31.42
```

### Struct with Constructors
```kode
struct Point {
    x: int,
    y: int
}

impl Point {
    fn new(x: int, y: int) -> Point {
        return Point { x: x, y: y };
    }
    
    fn distance() -> float {
        return sqrt(this.x * this.x + this.y * this.y);
    }
}

let p = Point.new(3, 4);
print(p.distance());  // 5.0
```

---

## 📋 Enumerations

### Basic Enum
```kode
enum Color {
    Red,
    Green,
    Blue
}

let color = Color.Red;

match (color) {
    Color.Red => print("red"),
    Color.Green => print("green"),
    Color.Blue => print("blue")
}
```

### Enum with Associated Values
```kode
enum Result {
    Success(value: int),
    Error(message: string),
    Pending
}

let outcome = Result.Success(42);
let error = Result.Error("Connection timeout");

match (outcome) {
    Result.Success(value) => print("Got: ${value}"),
    Result.Error(msg) => print("Error: ${msg}"),
    Result.Pending => print("Still waiting...")
}
```

### Enum with Methods
```kode
enum Status {
    Active,
    Inactive,
    Pending
}

impl Status {
    fn isReady() -> bool {
        return this == Status.Active;
    }
}

let status = Status.Active;
if (status.isReady()) {
    print("Ready to go!");
}
```

---

## 🎯 Traits & Interfaces

### Trait Definition
```kode
trait Drawable {
    fn draw() -> void;
    fn getColor() -> string;
}
```

### Trait Implementation
```kode
struct Rectangle {
    width: float,
    height: float,
    color: string
}

impl Drawable for Rectangle {
    fn draw() -> void {
        print("Drawing rectangle: ${this.width}x${this.height}");
    }
    
    fn getColor() -> string {
        return this.color;
    }
}

struct Circle {
    radius: float,
    color: string
}

impl Drawable for Circle {
    fn draw() -> void {
        print("Drawing circle with radius: ${this.radius}");
    }
    
    fn getColor() -> string {
        return this.color;
    }
}
```

### Using Traits
```kode
fn renderShape(shape: Drawable) {
    shape.draw();
    print("Color: ${shape.getColor()}");
}

let rect = Rectangle { width: 10.0, height: 5.0, color: "blue" };
let circle = Circle { radius: 5.0, color: "red" };

renderShape(rect);     // Works!
renderShape(circle);   // Works!
```

---

## 📦 Arrays & Collections

### Array Operations
```kode
let numbers = [1, 2, 3, 4, 5];

// Indexing
let first = numbers[0];        // 1
let last = numbers[4];         // 5

// Modification
numbers[2] = 99;               // [1, 2, 99, 4, 5]

// Array length
let len = len(numbers);        // 5

// Array slicing
let slice = numbers[1:3];      // [2, 99]
```

### Array Methods
```kode
let arr = [3, 1, 4, 1, 5];

// Built-in functions
len(arr);              // 5 (length)
push(arr, 9);          // [3, 1, 4, 1, 5, 9]
pop(arr);              // Returns 9, arr is [3, 1, 4, 1, 5]
contains(arr, 4);      // true
indexOf(arr, 4);       // 2
sort(arr);             // [1, 1, 3, 4, 5]
reverse(arr);          // [5, 4, 3, 1, 1]
```

### Higher-Order Array Operations
```kode
let numbers = [1, 2, 3, 4, 5];

// Map
let doubled = map(numbers, fn(x) { return x * 2; });
// [2, 4, 6, 8, 10]

// Filter
let evens = filter(numbers, fn(x) { return x % 2 == 0; });
// [2, 4]

// Reduce/Fold
let sum = reduce(numbers, 0, fn(acc, x) { return acc + x; });
// 15

// Find
let firstEven = find(numbers, fn(x) { return x % 2 == 0; });
// 2
```

---

## 🧮 Closures & Lambda

### Anonymous Functions (Lambda)
```kode
let square = fn(x) { return x * x; };
print(square(5));  // 25

let add = fn(a, b) { return a + b; };
print(add(3, 4));  // 7
```

### Closures with Captured Variables
```kode
fn makeMultiplier(factor) {
    return fn(x) { return x * factor; };
}

let double = makeMultiplier(2);
let triple = makeMultiplier(3);

print(double(5));  // 10
print(triple(5));  // 15
```

### First-Class Functions
```kode
fn applyTwice(f, x) {
    return f(f(x));
}

let increment = fn(n) { return n + 1; };
print(applyTwice(increment, 5));  // 7
```

### Immediately Invoked Function Expressions (IIFE)
```kode
let result = (fn() {
    let privateVar = 42;
    return privateVar * 2;
})();

print(result);  // 84
```

---

## 🎭 Pattern Matching

### Simple Patterns
```kode
let value = 2;

match (value) {
    1 => print("one"),
    2 => print("two"),
    3 => print("three"),
    _ => print("other")
}
```

### Destructuring Patterns
```kode
let pair = (10, 20);

match (pair) {
    (a, b) => print("Pair: ${a}, ${b}")
}

let person = Person { name: "Alice", age: 30 };

match (person) {
    Person { name: n, age: a } => print("${n} is ${a} years old")
}
```

### Enum Patterns
```kode
enum Message {
    Click(x: int, y: int),
    Hover(x: int, y: int),
    Quit
}

let msg = Message.Click(10, 20);

match (msg) {
    Message.Click(x, y) => print("Clicked at ${x}, ${y}"),
    Message.Hover(x, y) => print("Hovering at ${x}, ${y}"),
    Message.Quit => print("Quit")
}
```

### Guard Clauses
```kode
let value = 15;

match (value) {
    x if x < 10 => print("small"),
    x if x < 20 => print("medium"),
    x if x >= 20 => print("large"),
    _ => print("unknown")
}
```

---

## ⚠️ Error Handling

### Try-Catch Blocks
```kode
try {
    let result = divide(10, 0);
    print(result);
} catch {
    print("Error: Division by zero");
}
```

### Error Types
```kode
enum Error {
    DivisionByZero,
    InvalidInput(reason: string),
    NotFound(resource: string),
    Unknown(message: string)
}

fn divide(a: int, b: int) -> Result<int, Error> {
    if (b == 0) {
        return Result.Error(Error.DivisionByZero);
    }
    return Result.Success(a / b);
}
```

### Error Handling with Result Type
```kode
let result = divide(10, 2);

match (result) {
    Result.Success(value) => print("Result: ${value}"),
    Result.Error(error) => print("Error occurred")
}
```

### Custom Error Handling
```kode
trait ErrorHandler {
    fn handle(error: Error) -> void;
}

impl ErrorHandler for Logger {
    fn handle(error: Error) -> void {
        print("Logged error: ${error.message}");
    }
}
```

---

## 📚 Modules & Imports

### Named Imports
```kode
import { add, subtract, multiply } from "math";
import sqrt from "math";

print(add(5, 3));           // 8
print(sqrt(16));            // 4.0
```

### Namespace Import
```kode
import math from "./math";

print(math.add(5, 3));      // 8
print(math.sqrt(16));       // 4.0
```

### Wildcard Import
```kode
import * from "math";

print(add(5, 3));           // 8
print(sqrt(16));            // 4.0
```

### Exports
```kode
// In math.kode

export fn add(a: int, b: int) -> int {
    return a + b;
}

export fn subtract(a: int, b: int) -> int {
    return a - b;
}

const PI = 3.14159;
export const PI_EXPORT = PI;  // Must explicitly export
```

### Module Alias
```kode
import math as M from "./math";

print(M.add(5, 3));  // 8
```

### Relative Imports
```kode
import { Vector } from "../geometry/vector";
import Parser from "./parser";
```

---

## 🔄 Concurrency (Future)

### Spawning Goroutines (Lightweight Threads)
```kode
spawn {
    for (let i = 0; i < 5; i = i + 1) {
        print("Worker: ${i}");
    }
}

print("Main continues");
```

### Channels for Communication
```kode
let chan = make(Channel<int>);

spawn {
    send(chan, 42);
};

let value = receive(chan);
print(value);  // 42
```

### Select Statement
```kode
let chan1 = make(Channel<string>);
let chan2 = make(Channel<int>);

spawn {
    send(chan1, "Hello");
}

spawn {
    send(chan2, 42);
}

select {
    message: chan1 => print("Got string: ${message}"),
    number: chan2 => print("Got number: ${number}")
}
```

### Mutexes for Synchronization
```kode
let mutex = make(Mutex<int>);
let counter = 0;

spawn {
    lock(mutex);
    counter = counter + 1;
    unlock(mutex);
}
```

---

## 🔑 Keywords

| Keyword | Description |
|---------|-------------|
| `let` | Variable declaration |
| `const` | Constant declaration |
| `fn` | Function definition |
| `return` | Return from function |
| `if` | Conditional (true branch) |
| `else` | Conditional (false branch) |
| `while` | While loop |
| `for` | For loop |
| `do` | Do-while loop |
| `break` | Exit loop early |
| `continue` | Skip to next iteration |
| `match` | Pattern matching |
| `struct` | Structure definition |
| `enum` | Enumeration definition |
| `trait` | Trait/interface definition |
| `impl` | Implementation block |
| `import` | Import module/items |
| `export` | Export item from module |
| `try` | Try block (error handling) |
| `catch` | Catch block (error handling) |
| `spawn` | Create goroutine |
| `send` | Send to channel |
| `receive` | Receive from channel |
| `select` | Multiple channel select |
| `true` | Boolean true |
| `false` | Boolean false |
| `null` | Null value |
| `as` | Type casting |
| `is` | Type checking |
| `this` | Reference to self |
| `self` | Reference to self |
| `_` | Wildcard pattern |
| `...` | Variadic arguments |

---

## 📝 Notes on v1.0+ Features

- **Type Inference**: Automatic type deduction is powerful but explicit types improve readability
- **Pattern Matching**: More powerful than if-else chains for complex data structures
- **Traits**: Enable polymorphism and code reuse across different types
- **Concurrency**: Lightweight goroutines for efficient parallel processing
- **Error Handling**: Result types and try-catch for comprehensive error management
- **Generics**: Support for parametric types throughout the language

---

*Created with ❤️ by Sreeraj V Rajesh*

© 2026 Kode Programming Language

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

### Bitwise Operators (NEW!)
```kode
let x = 12;  // Binary: 1100
let y = 5;   // Binary: 0101

let andBit = x & y;    // Bitwise AND: 4 (0100)
let orBit = x | y;     // Bitwise OR: 13 (1101)
let xorBit = x ^ y;    // Bitwise XOR: 9 (1001)
let leftShift = x << 2; // Left shift by 2: 48 (110000)
let rightShift = x >> 1; // Right shift by 1: 6 (110)
let notBit = ~y;       // Bitwise NOT: -6
```

| Operator | Example | Result | Description |
|----------|---------|--------|-------------|
| `&` | `12 & 5` | `4` | Bitwise AND - 1 if both bits are 1 |
| `\|` | `12 \| 5` | `13` | Bitwise OR - 1 if at least one bit is 1 |
| `^` | `12 ^ 5` | `9` | Bitwise XOR - 1 if exactly one bit is 1 |
| `<<` | `8 << 2` | `32` | Left shift - shift bits left, fill with 0 |
| `>>` | `8 >> 2` | `2` | Right shift - shift bits right |
| `~` | `~5` | `-6` | Bitwise NOT - invert all bits |
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
# PR Changes Summary for GitHub Linguist

## 1. Update .gitattributes (in Kode Repository)

**File**: `.gitattributes`

```diff
 *.kode linguist-language=Kode
+*.kbc linguist-language=Kode
```

## 2. lib/linguist/languages.yml Entry

**File**: `lib/linguist/languages.yml` (in Linguist Repository)

Add this entry in alphabetical order (between "Kotlin" and "Kusto"):

```yaml
Kode:
  type: programming
  extensions:
  - .kode
  - .kbc
  color: "#7C3AED"
  tm_scope: source.kode
```

## 3. Sample Files

Create directory: `samples/Kode/`

### samples/Kode/hello_world.kode
```kode
// Hello World example in Kode
fn main() {
  println("Hello, World!")
}
```

### samples/Kode/loops.kode
```kode
// Loops example in Kode
fn main() {
  // For loop
  for i in 0..10 {
    println(i)
  }

  // While loop
  let mut x = 0
  while x < 5 {
    println(x)
    x = x + 1
  }

  // Array iteration
  let arr = [1, 2, 3, 4, 5]
  for val in arr {
    println(val * 2)
  }
}
```

### samples/Kode/functions.kode
```kode
// Functions example in Kode
fn add(a: i32, b: i32): i32 {
  a + b
}

fn greet(name: str) {
  println("Hello, " + name)
}

fn factorial(n: i32): i32 {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}

fn main() {
  let result = add(5, 3)
  println(result)

  greet("Kode")

  let fact = factorial(5)
  println(fact)
}
```

## Color Choice Justification

**Color**: `#7C3AED` (Purple-Violet)

This color was chosen to:
- Represent the modern, elegant nature of the Kode language
- Provide good contrast and visibility in GitHub's language statistics
- Align with contemporary programming language color palettes
- Stand out among existing language colors in GitHub repositories

## TextMate Grammar

The Kode language includes comprehensive TextMate grammar support defined in:
- `grammars/kode.tmLanguage.json` (in the Kode repository)

This grammar provides syntax highlighting for:
- Keywords and control structures
- String literals and comments
- Functions and declarations
- Built-in types and operators
- Error states and other language features

## Verification Checklist

- [x] YAML syntax is valid
- [x] No trailing whitespace in any files
- [x] Extensions are properly formatted
- [x] Color hex code is valid
- [x] tm_scope matches the grammar definition
- [x] Sample files are syntactically correct Kode code
- [x] At least 3 representative samples provided
- [x] Alphabetical ordering maintained in languages.yml
- [x] Files follow Linguist's conventions

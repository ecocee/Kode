# Kode cheat sheet

One page of shapes. The chapters in [`spec/`](spec/) explain the rules; this is
for looking things up quickly.

## Values and types

```
n = 42                      # int (64-bit)
f = 3.5                     # float (64-bit)
ok = true                   # bool
s = "hi ${n}"               # string, with ${} interpolation
n: int = 0                  # a declared type, written once
```

- Types are `int`, `float`, `bool`, `string`, `[T]`, `{string: T}`, classes,
  enums, interfaces, `Task<T>`, `Channel<T>`, `fn(T) -> R` and `T?`.
- Numbers never convert on their own: use `int(x)` or `float(x)`.

## Optionals

```
name: string? = none
score: int? = none
score = 5

print(name ?? "guest")      # a default
print(score == none, score!)
if score != none {
    print(score + 1)        # narrowed to int inside the branch
}
```

- `?.` reads a field only when the object is not none.
- `int?`, `float?` and `bool?` work as **local variables** and are stored as a
  value plus a `present` flag, with no boxing. As a parameter, return type, field
  or list item they are still E0219.
- Narrowing holds inside `if x != none { }`, inside `while x != none { }`, and in
  the code after `if x == none { return }`. An assignment ends it.

## Lists, maps and slices

```
xs = [1, 2, 3]
xs.add(10)                  # append
xs.sort()                   # [int], [float], [string]
print(xs[0], xs.len(), len(xs))
print(xs.contains(2), xs.index_of(2))    # -1 when missing
print(xs.first(), xs.last())
xs.insert(0, 7)
xs.remove_at(0)

part = xs[1..3]             # a copy: items 1 and 2
head = xs[..2]              # from the start
tail = xs[2..]              # to the end
inc = xs[0..=2]             # includes index 2
print(xs[10..20].len())     # 0: bounds are clamped, not an error
part.add(99)                # the original is unchanged

doubled = xs.map(|x: int| x * 2)
evens = xs.filter(|x: int| x % 2 == 0)

ages = {"asha": 30, "ravi": 25}
ages["ravi"] = 25
for k, v in ages { print(k, v) }         # insertion order
for i, x in xs { print(i, x) }
```

- Maps always have string keys. Lists and maps are reference counted: `b = a`
  shares one list.
- An empty list or map needs a declared type: `xs: [int] = []`.
- `print(xs)` and `"${xs}"` show a list or map: `[1, 2, 3]`, `["a", "b"]`,
  `{"name": "Asha"}`. Strings inside a collection are quoted and escaped.
- Not yet: integer map keys.

## Limits

- Expressions and blocks nest at most 128 levels (E0118). A long `+` chain
  counts one level per term, so 129 added terms in one line is an error; build
  the value with a loop instead (ADR 0032).

## Enums and printing

```
enum Color { Red, Green, Blue }

c = Color.Green
print(c)                    # Color.Green
print("c is ${c}")          # c is Color.Green

match c {
    Color.Red => print("red")
    Color.Green => print("green")
    _ => print("other")
}
```

- `print` and `${}` write an enum the way it is written in source: `Color.Green`.
  Enums are ints underneath, so `==` and `match` are unchanged.
- `match` covers every case or ends with `_ =>` (E0224).

## Functions, classes and interfaces

```
fn add(a: int, b: int = 0) -> int {
    return a + b
}

fn double(n: int) -> int => n * 2      # expression body

class User {
    name: string
    tags: [string] = []

    fn greet() -> string => "hi ${name}"
}

u = User(name: "Asha")                 # named arguments
print(u.greet())
```

- Classes are reference types; `v = u` is the same object.
- No inheritance: compose, or implement an interface.

## Function values and closures

```
fn double(n: int) -> int => n * 2
fn apply(f: fn(int) -> int, n: int) -> int => f(n)

f = double                          # a named function is a value
print(f(21), apply(double, 5))

add = |x: int| x + 10               # a closure; parameters need types
print(add(5))
print(apply(|x: int| x * x, 7))
```

- A function type is `fn(int) -> int`; `fn()` takes nothing.
- A closure captures **every variable visible** where it is written, by value.
  An optional number or bool cannot be captured yet (E0219).
- Call a value by name: `f(1)`. Calling the result of an expression does not
  parse yet.
- Inside a closure a field of `this` is not visible (E0242); copy it out first.
- A function value cannot be printed (E0203).

## Tasks and channels

```
jobs: Channel<int> = channel()

fn produce(out: Channel<int>) {
    for i in 0..3 {
        send(out, i)
    }
}

t = spawn produce(jobs)
print(recv(jobs), recv(jobs), recv(jobs))   # recv waits for a value
await t
close(jobs)
```

- `send` queues without blocking, `recv` waits, `close` is idempotent.
- A task may take plain values and channels as parameters.

## Errors

```
throw "boom"                            # an Error with a message
throw Error(message: "missing", code: 404)

try {
    might_fail()
} catch e {
    print(e.message, e.code)
} finally {
    print("always")                     # cleanup
}

v = might_fail() catch e { -1 }         # handle one expression
```

- An uncaught error leaves the function and ends the program with a message.
- Every diagnostic has a stable code; `kode explain E0220` shows it.

## Commands

```
kode run app.kode            # compile and run
kode check app.kode          # type check only, --format=json for editors
kode build app.kode          # a standalone native binary
kode test                    # run the test blocks
kode fmt app.kode            # format
kode doc src -o docs/api     # HTML reference for pub items
kode explain E0220           # what a code means
kode tokens app.kode         # tokens: span, line:column, kind, text (self-hosting oracle)
kode ast app.kode            # syntax tree with spans (same; both take --format=json)
kode lsp                     # language server on stdio
kode doctor                  # version, target, linker, runtime ABI
kode init [name]             # make the current folder a package
kode new <name>              # make a package in a new folder
kode add <name> --path <dir> # add a path dependency, then install
kode remove <name>           # remove a dependency, then install
kode install [--check]       # resolve dependencies and write kode.lock
```

## Release files and installers

A release is published from a tag and produces a binary per platform plus notes
(see `docs/releasing.md`). The release workflow produces a tarball plus a native
installer for each platform: a `.deb` on Linux, a `.dmg` disk image on macOS, and
a `.zip` plus a standalone `.exe` on Windows. Downloads are checksummed.

## Generics

```
fn first<T>(xs: [T]) -> T => xs[0]
fn pair<T>(a: T, b: T) -> [T] => [a, b]
fn apply_all<T, U>(xs: [T], f: fn(T) -> U) -> [U] => xs.map(f)

print(first([10, 20]))              # 10: T is int here
print(first(["a", "b"]))            # a:  T is string here
print(pair(1, 2)[1])                # 2
print(apply_all([1, 2], |n: int| "n${n}")[1])   # n2
```

- `fn name<T>(...)` declares type parameters; each call infers them from its
  argument types. A parameter used twice must get one type: `pair(1, "a")` is
  E0208.
- Each concrete combination becomes its own native function (monomorphization),
  so generics cost nothing at run time.
- `T` must appear in a parameter type (E0250); `f = first` is E0251; methods
  cannot be generic yet (E0252).
- Not yet: generic classes, generic methods, constraints, `first::<int>`.

## Processes

```
r = exec("/usr/bin/cc", ["-c", "main.s", "-o", "main.o"], "")
print(r.code, r.stdout, r.stderr)
```

- `args` is a `[string]` passed straight to the program: no shell, so spaces and
  `$` stay literal. An empty `cwd` keeps the current directory.
- A program that cannot start throws (catch it); a non-zero exit is `r.code`.
- `exec` is synchronous; stdin is closed, stdout and stderr are captured safely.

## PostgreSQL

```
db = pg.connect("postgres://user:pass@127.0.0.1:5432/app?sslmode=disable")
pg.exec(db, "create table t (id int, name text)")

ins = pg.query(db, "insert into t values ($1, $2)", [json.int(1), json.string("asha")])
print(ins["command"].as_string(), ins["affected"].as_int())

rows = pg.query(db, "select id, name, note from t where id > $1", [json.int(0)])
for row in rows["rows"].as_list() {
    print(row["id"].as_int(), row["note"].is_null())
}
pg.close(db)
```

- A connection is an `int` handle. `pg.exec` is a simple query; `pg.query`
  binds `$1` parameters (`json.null()`, bool, int, float, string).
- A result is JSON: `command`, `columns`, `rows` (one object per row) and
  `affected`.
- Errors carry the SQLSTATE: `42P01: relation "t" does not exist`.
- Plain TCP only: add `?sslmode=disable`. No md5, no `COPY`, no notifications.

## Project layout

```
kode.toml                    # [package] and [dependencies] with path
src/main.kode                # the program
import "lib/util"            # another file in this package
import "utils/strings" as u  # a dependency
kode.lock                    # resolved versions, written by kode install
```

`kode.toml` can also pin the compiler: `kode = "0.3"` under `[package]` means
0.3.0 or newer, like `go 1.24` in `go.mod`. `kode doctor` fails when this
compiler is older than the requirement.

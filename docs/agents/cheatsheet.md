# Kode cheat sheet for agents (v0.3)

Agent-facing summary: dense shapes, no prose. The full cheat sheet is
[`docs/cheatsheet.md`](../cheatsheet.md).

Compiled language, one statement per line, no semicolons. Types: `int float bool string`,
`[T]` list, `{string: T}` map, `T?` optional (string, list, map, class), classes, enums.

```
// comment            /* block */
x = 5                 // declares; type is fixed; block-scoped
y: float = 2.5        // explicit type
const MAX = 100       // plain literal only
x += 1                // also -= *= /= %=
s = "Hi ${name}, ${x + 1}"        // interpolation; escapes \n \t \" \\ \$
a = [1, 2, 3]         a[0]   a.add(4)   a.len()   for x in a   for i, x in a
m = {"k": 1}          m["k"] m["z"] = 2   "k" in m   m.has("k")   m.keys()   for k, v in m
```

Operators: `+ - * / %`  `== != < <= > >=`  `and or not`  `??`  `?.`  `!`  `in`.
`int / int` truncates. Numbers never mix: use `int(x)`, `float(x)`. Conditions must be `bool`.

```
if a > b { ... } else if a == b { ... } else { ... }
while cond { ... }          for i in 0..10 { ... }     for i in 0..=10 { ... }
break   continue            match v { 1 => print("one")  _ => print("other") }

fn add(a: int, b: int) -> int { return a + b }      // return type required when it returns a value
fn sq(x: int) -> int => x * x                        // one-expression body
fn greet(name: string) { print("hi", name) }         // returns nothing
greet(name: "Asha")                                  // named arguments

class Account {
    owner: string
    balance: int = 0
    fn deposit(n: int) { balance += n }              // fields by plain name
}
acct = Account(owner: "Asha")   acct.deposit(5)   acct.balance

enum Color { Red, Green }       c = Color.Red        // compare with ==, use with match

u: Account? = none     u?.owner ?? "nobody"    u!.owner
throw "message"        throw Error(message: "x", code: 7)
v = f() catch e { print(e.message)  -1 }            // last line is the value
try { f() } catch e { print(e.message) } finally { close() }   // finally always runs

import "./util"        from "./util" import helper   // modules share one namespace
import "models/user" as users                        // package root from kode.toml
pub fn api() -> int => 1                              // visible to aliases and from imports
users.load(1)   users.LIMIT   users.User(1)           // qualified fn / const / class
test "name" { assert(1 + 1 == 2) }                   // run with: kode test
```

Tasks: `t = spawn work(1, "a")` (int/float/bool/string args)   `r = await t`   `Task<int>`
Library: `s.size() s.slice(a,b) s.split(",") s.find(t) s.upper() s.trim() s.to_int()`  `xs.sort() xs.pop()`  `words.join(",")`
More: `s.lines() s.char_at(i) s.is_empty() s.count(t)`  `xs.contains(v) xs.index_of(v) xs.reverse() xs.insert(i,v) xs.remove_at(i) xs.first() xs.last()`  `m.values()`
`read_file(p) write_file(p,t) args() env(n) exit(c) sleep(ms) now_ms() chr(n)`   `serve(8080, handler)`
Router: `routes = Router()`  `routes.get("/users/:id", handler)`  handlers are `fn name(req: Request) -> Response`
Request/Response: `req.params["id"] req.query req.headers req.body req.json()`  `Response.json(doc)` `Response.text(s)` `response(201, body, ctype)`
JSON: `j = json_parse(t)`  `j["k"] j[0]`  `j.as_string() as_int() as_float() as_bool() as_list() as_map()`  `j.is_null() j.len() j.has(k) j.encode()`
Build: `json_map({"a": json_int(1)})`  `json_list([...])`  `json_null/bool/int/float/string`
HTTP: `r = http_get(url)`  `r.status r.body r.headers["content-type"]`  `http_post(url, body, ctype)`  `http_request(m, url, {}, body)`  (plain http only)
Modules: `json.parse(t) json.stringify(v) json.object()`  `math.sqrt(2.0) math.pi`  `fs.read(p) fs.write(p,t)`  `time.now_ms() time.sleep(ms)`  `http.get(url)`
Bits: `a & b  a | b  a ^ b  ~a  a << n  a >> n`  `x &= y  x |= y  x ^= y  x <<= n  x >>= n`  (ints only; shifts of 64+ give 0)

Built-ins: `print(...)`, `len(x)`, `int(x)`, `float(x)`, `string(x)`, `assert(cond)`.
Commands: `kode run|build|check|test|fmt|explain|mir|lsp|doc|doctor`. Use `kode check --format=json` for errors
and `kode explain E0201` for help. Strict rules: no implicit conversions, no truthy values, no
inheritance, no semicolons, one statement per line.

# 13. Standard library (v0.3)

All of these are runtime functions listed in one table (`crates/kode-sema/src/std_fns.rs`).
Adding one means one table row and one `extern "C"` function in `crates/kode-rt/src/std_fns.rs`.

| Strings | |
|---|---|
| `s.size()` | bytes (`len(s)` counts characters) |
| `s.byte_at(i)` | byte at position `i` |
| `s.slice(a, b)` | bytes `a` to `b` (error if not on a character boundary) |
| `s.contains(t)` `s.starts_with(t)` `s.ends_with(t)` | bool |
| `s.find(t)` | position or -1 |
| `s.upper()` `s.lower()` `s.trim()` | new string |
| `s.replace(a, b)` `s.repeat(n)` | new string |
| `s.split(sep)` | `[string]` |
| `s.lines()` | `[string]`, split on `\n` with any trailing `\r` removed |
| `s.char_at(i)` | one character (not byte); raises an error when out of range |
| `s.is_empty()` | bool |
| `s.count(sub)` | non-overlapping occurrences |
| `s.to_int()` `s.to_float()` | raise an error if the text is not a number |
| `chr(n)` | the character with that code |

| Lists and maps | |
|---|---|
| `xs.sort()` | for `[int]`, `[float]`, `[string]` |
| `xs.pop()` `xs.clear()` `xs.reverse()` | not for `[bool]` |
| `xs.contains(v)` `xs.index_of(v)` | for `[int]`, `[float]`, `[string]`; `index_of` is -1 when missing |
| `xs.insert(i, v)` `xs.remove_at(i)` | raise an error when the index is out of range |
| `xs.first()` `xs.last()` | the item, or a catchable error on an empty list |
| `words.join(sep)` | for `[string]` |
| `m.remove(key)` | |
| `m.values()` | `[T]` in insertion order |

| Files, program, time | |
|---|---|
| `read_file(path)` `write_file(path, text)` | raise an error on failure |
| `args()` | program arguments: `kode run app.kode -- a b` gives `["a", "b"]` |
| `env(name)` | `string?` |
| `exit(code)` | |
| `sleep(ms)` `now_ms()` | |
| `exec(program, args, cwd)` | runs a program without a shell and returns `ProcessResult { code, stdout, stderr }`; an empty `cwd` inherits the current directory. A program that cannot start raises an error; a non-zero exit is a `code`, not an error. |

## Modules

The library is also reachable with Python-style names:

```
data = json.parse(text)
print(json.stringify(data))

print(math.sqrt(2.0), math.pi, math.floor(2.9))
now = time.now_ms()
text = fs.read("notes.txt")
res = http.get("http://localhost:8080/hello")
```

| Module | Functions |
|---|---|
| `json` | `parse`, `stringify`, `object`, `null`, `bool`, `int`, `float`, `string`, `list`, `map` |
| `http` | `get`, `post`, `request` |
| `fs` | `read`, `write` |
| `time` | `now_ms`, `sleep` |
| `math` | `sqrt`, `abs`, `floor`, `ceil`, `round`, `pow`, `min`, `max`, `sin`, `cos`, `tan`, `log`, `exp`; constants `pi` and `e` |

`math.min` and `math.max` take two floats. `floor`, `ceil` and `round` return floats, like
Python. A variable with the same name shadows a module.

## Release files and installers

A release is published from a tag and produces a binary per platform plus notes
(see `docs/releasing.md`). The release workflow produces a tarball plus a native
installer for each platform: a `.deb` on Linux, a `.dmg` disk image on macOS, and
a `.zip` plus a standalone `.exe` on Windows. Downloads are checksummed.

## Web server

```
fn handle(method: string, path: string, body: string) -> string {
    if path == "/hello" { return "Hello from Kode" }
    throw Error(message: "not found", code: 404)
}

serve(8080, handle)
```

`serve(port, handler)` runs an HTTP/1.1 server with keep-alive on a pool of threads. The handler
receives the method, the path (with the query) and the body, and returns the response text
(JSON is detected when it starts with `{` or `[`). An error with a `code` from 400 to 599 becomes
that status; any other error is a 500. Try it: `kode run tests/http/server.kode -- 8080`
and `curl localhost:8080/hello`.

For an API, `Router()` with `Request` and `Response` reads like Express or FastAPI:

```
fn get_user(req: Request) -> Response {
    user = json_obj()
    user["id"] = json_int(req.params["id"].to_int())
    user["name"] = json_string("Asha")
    return Response.json(json_map(user))
}

fn create_user(req: Request) -> Response {
    doc = req.json()
    return Response(status: 201, body: doc.encode(), content_type: "application/json")
}

routes = Router()
routes.get("/users/:id", get_user)
routes.post("/users", create_user)
serve(8080, routes)
```

`routes.get`, `post`, `put`, `patch` and `delete` take a path and the name of a function
`fn name(req: Request) -> Response`. A pattern matches one segment with `:name` and the rest
with `*name`; captured values are in `req.params`. `Request` has `method`, `path`, `params`,
`query`, `headers` (names lowercased) and `body`, plus `req.json()`, which parses the body and
raises a catchable error when it is not JSON.

`Response` has `status`, `body` and `content_type`. Build one with `Response.json(value)`,
`Response.text(text)`, `response(status, body, content_type)` or
`Response(status: ..., body: ..., content_type: ...)`; an empty content type is inferred from
the body. Unknown paths answer 404, and a known path with the wrong method answers 405. An
uncaught error answers with its `code` (400 to 599) or 500, as with the plain handler.
`json_obj()` returns an empty `{string: Json}` to fill key by key. Query and captured path
values are not percent-decoded yet.

Try it: `kode run examples/api.kode -- 8080`, then `curl localhost:8080/users/7`.

## JSON

```
doc = json_parse("{\"name\":\"Asha\",\"tags\":[\"ai\"]}")
print(doc["name"].as_string())
print(doc["tags"][0].as_string())
print(doc["tags"].len())
print(doc.encode())

out = json_map({
    "id": json_int(1),
    "tags": json_list([json_string("a"), json_string("b")]),
    "ok": json_bool(true),
    "none": json_null()
})
```

`json_parse(text)` returns a `Json` value and raises an error for invalid JSON. A `Json` value is
immutable and has a `kind: JsonKind` field (`Null`, `Bool`, `Int`, `Float`, `Str`, `List`, `Map`)
plus these methods: `is_null()`, `as_bool()`, `as_int()`, `as_float()`, `as_string()`, `as_list()`,
`as_map()`, `len()`, `has(key)` and `encode()`. An accessor raises an error when the value has a
different kind. `j["key"]` and `j[i]` read a child and raise an error when it is missing.

Build values with `json_null()`, `json_bool(b)`, `json_int(i)`, `json_float(f)`, `json_string(s)`,
`json_list([Json])` and `json_map({string: Json})`. Maps keep insertion order, so `encode()` output
is predictable. Integers must fit in `int`; floats must be finite. Nesting is limited to 256 levels.

## HTTP client

```
r = http_get("http://localhost:8080/hello")
print(r.status, r.headers["content-type"], r.body)

r = http_post("http://localhost:8080/users", "{\"name\":\"Asha\"}", "application/json")
r = http_request("DELETE", "http://localhost:8080/users/1", {}, "")
```

`HttpResponse` has `status: int`, `body: string` and `headers: {string: string}` (names lowercased).
Responses with `Content-Length` or chunked transfer encoding are supported. The client sends
`Connection: close`, waits up to 30 seconds, limits a body to 64 MiB, and does not follow redirects.
**Only plain `http://` is supported**: there is no TLS library in the runtime yet, so `https://`
raises a clear error instead of sending an unencrypted request.

Not yet: TLS, redirects, a database client.

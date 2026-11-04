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
| `s.to_int()` `s.to_float()` | raise an error if the text is not a number |
| `chr(n)` | the character with that code |

| Lists and maps | |
|---|---|
| `xs.sort()` | for `[int]`, `[float]`, `[string]` |
| `xs.pop()` `xs.clear()` | not for `[bool]` |
| `words.join(sep)` | for `[string]` |
| `m.remove(key)` | |

| Files, program, time | |
|---|---|
| `read_file(path)` `write_file(path, text)` | raise an error on failure |
| `args()` | program arguments: `kode run app.kode -- a b` gives `["a", "b"]` |
| `env(name)` | `string?` |
| `exit(code)` | |
| `sleep(ms)` `now_ms()` | |

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

Not yet: a JSON parser, an HTTP client, TLS, a database client.

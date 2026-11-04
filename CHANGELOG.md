# Changelog

## Unreleased
- JSON: `json_parse`, `json_null/bool/int/float/string/list/map`, a built-in `Json` class with
  `JsonKind` and `is_null`, `as_*`, `len`, `has`, `encode`, and read-only `j["key"]` / `j[i]` indexing.
  Strict parser with a depth limit; parse errors are catchable and do not leak.
- Web API layer: `Router()` with `get`, `post`, `put`, `patch` and `delete`, built-in `Request`
  (`method`, `path`, `params`, `query`, `headers`, `body`, `req.json()`) and `Response`
  (`status`, `body`, `content_type`), `Response.json` / `Response.text`, `response(status, body,
  content_type)` and `json_obj()`. `serve(port, router)` dispatches `:name` and `*name` patterns,
  answers 404/405, and keeps the old `serve(port, handler)` form working.
- HTTP client: `http_get`, `http_post`, `http_request`, built-in `HttpResponse`
  (`status`, `body`, `headers`). Plain `http://` only; `Content-Length` and chunked responses;
  30 s timeout; 64 MiB limit; redirects are not followed.
- New ADR 0017, spec updates, and tests: `tests/run-pass/json.kode`, two compile-fail cases, and an
  offline HTTP client integration test.

## 0.3.0 (M6, M7, performance)
- Tasks: `spawn f(args)`, `await t`, type `Task<T>`; atomic reference counts when threads exist; one error slot per thread.
- Standard library table: string, list, map, file, environment, time functions (see docs/spec/13-stdlib.md).
- `serve(port, handler)`: HTTP/1.1 server with keep-alive on a thread pool.
- The runtime library is embedded in the compiler (no stale runtime, `kode` works from any folder).
- `kode run file.kode -- args`; optimization is on by default (`--debug` turns it off).
- Faster code: constant division without checks, `x % 2 == 0` as a bit test, inlined list access and `add`,
  1-byte `[bool]` lists, loop rotation, conditions that branch directly, `if/else` to `select`, a small-function
  inliner, no frame pointers.
- Benchmark suite and harness in `benchmarks/` (Kode, C, C++, Go, JavaScript, Python; real peak memory).

## 0.2.0 (M2 to M5)
- Lists `[T]` and maps `{string: T}`: literals, indexing, `.add`, `.len`, `in`, `.keys`, `for x in list`, `for k, v in map`.
- Classes with fields, defaults, methods and named constructor arguments; enums; `match`.
- Optionals `T?` for heap types with `none`, `?.`, `??`, `!`.
- Errors: `throw`, `try / catch`, `expr catch e { ... }`, built-in `Error` class, errors that are freed on every path.
- Named arguments for every call.
- Imports across files (`import`, `from ... import`) with cycle and missing-file errors.
- `test` blocks and `kode test`; `kode fmt`; `kode explain`; `kode mir` shows field loads and stores.
- Runtime: one object header and one `release` for strings, lists, maps and objects; leak counter covers all of them.
- MIR: single exit block per function, error edges that release live temporaries.
- 19 run-pass programs (leak checked), 70 compile-fail programs, tests for `kode test`, `fmt`, `explain`.

## 0.1.0 (first working core)
- Rust rewrite: lexer, parser, type checker, MIR, Cranelift backend, runtime.
- Types: int, float, bool, string. Block-scoped variables, constants.
- Control flow: if / else if / else, while, for over ranges, break, continue.
- Functions: parameters, return values, recursion, expression bodies.
- Strings: reference counted, `+`, `${...}` interpolation, `len`, conversions.
- Diagnostics with stable codes, "did you mean" hints and JSON output.
- Conformance suite: 11 run-pass programs (leak checked) and 44 compile-fail programs.

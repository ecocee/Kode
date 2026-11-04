# ADR 0018: A router with `Request` and `Response` for JSON APIs

Status: accepted for v0.4

## Context

`serve(port, handler)` exposes the raw request: the handler receives the method, path
and body as strings and returns response text. That is usable, but every app repeats
the same routing, JSON parsing and status handling, and moving a route means editing
a chain of `if path == ...` branches.

Express and FastAPI solve this with route tables and a request/response pair. Two
constraints shape how much of that Kode can copy today:

- There are no function values or closures. A function name is a value only where a
  built-in asks for one (`serve`, `spawn`), and the compiler lowers it to its address.
- Built-in classes cannot carry field defaults (only `Error.code` has one), so a
  `Response` must be given its status, body and content type.

## Decision

1. Add three built-in classes: `Request` (`method`, `path`, `params`, `query`,
   `headers`, `body`), `Response` (`status`, `body`, `content_type`) and `Router`
   (`id`). Their layout lives in `kode-sema/src/check/mod.rs` and is mirrored by
   `kode-rt/src/router.rs`; both must change together.
2. `Router()` is a compiler-recognised constructor: it allocates the object and asks
   the runtime for an id. `router.get/post/put/patch/delete(path, handler)` is
   likewise checked by name. `handler` must be a top-level function
   `fn name(req: Request) -> Response`; anything else is E0233.
3. `serve(port, router)` selects the router server; the old
   `serve(port, handler)` signature keeps working, unchanged.
4. The runtime keeps the route table in a mutex, copying each method and pattern into
   Rust strings. It never retains a Kode object, so routes cannot create cycles.
   Patterns support `:name` (one segment) and `*name` (the rest); captured values and
   the parsed query string become string maps on the request. Header names are
   lowercased.
5. `Response.json(value)` and `Response.text(text)` are static-style helpers; the
   compiler maps them to the runtime functions `response_json` / `response_text`.
   `response(status, body, content_type)` and the `Response(...)` constructor cover
   other statuses. `req.json()` parses the body with the same strict parser as
   `json_parse`, so a malformed body is a catchable error.
6. A handler that throws answers with the error's `code` (400 to 599) or 500, exactly
   as the plain handler does. Unknown paths are 404; a path that exists for another
   method is 405.
7. `json_obj()` returns an empty `{string: Json}`, because a map literal cannot mix
   value types; build the object key by key, then wrap it with `json_map`.

## Consequences

- An API route is three lines: a handler, a `routes.get(...)` and `serve`. Moving or
  adding a route no longer changes the dispatch logic.
- Routing is a runtime table keyed by function address, so handlers are ordinary
  compiled functions; there is no new calling convention and no closure allocation.
- Because `Router` has no function-typed fields, a route cannot be built inline:
  handlers must be named top-level functions. Closures or function types would lift
  that restriction later.
- Response headers are not configurable yet; the content type is. Adding a `headers`
  field is a layout change and needs the runtime ABI bump described above.

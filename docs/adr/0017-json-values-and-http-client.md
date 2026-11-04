# ADR 0017: A dynamic `Json` value and a plain-HTTP client

Status: accepted for v0.4

## Context

Kode maps and lists are homogeneous and statically typed, so JSON — which mixes
strings, numbers, booleans, null, arrays and objects — cannot be represented by
them directly. The language has no generics, no `Any`, and enums carry no payload.
The HTTP server (`serve`) already exists, but callers need to parse request bodies
and call other services.

## Decision

1. Add one built-in class, `Json`, with a `kind: JsonKind` field and payload fields
   (`int_value`, `float_value`, `string_value`, `list_value`, `map_value`).
   `JsonKind` is a built-in enum: `Null`, `Bool`, `Int`, `Float`, `Str`, `List`, `Map`.
   The layout is defined in `kode-sema/src/check/mod.rs` and mirrored in
   `kode-rt/src/json.rs`; the two must change together.
2. Expose the value through functions and methods, not through its fields:
   `json_parse`, the `json_*` builders, `is_null`, `as_*`, `len`, `has`, `encode`.
   Accessors raise a normal Kode error when the kind is wrong, so they are catchable.
3. Allow `j["key"]` and `j[i]` as read-only indexing. A missing key or index raises an
   error; assigning into a `Json` value is a compile error (E0211), because the value
   is immutable.
4. Implement the parser in the runtime with strict RFC 8259 syntax, a 256-level depth
   limit, and cleanup of half-built containers on error so a caught parse error does
   not leak.
5. Add an HTTP client (`http_get`, `http_post`, `http_request`) in the runtime. It
   supports plain `http://` only: `kode-rt` has no dependencies (it is built with
   plain `rustc`), so there is no TLS stack. `https://` fails with a clear message.
   Responses support `Content-Length` and chunked transfer encoding, use a 30-second
   timeout and a 64 MiB body limit, and do not follow redirects.

## Consequences

- JSON is ergonomic for agents and services without adding generics or weakening the
  strict type rules: every dynamic step is an explicit accessor call.
- `Json` and `HttpResponse` are reserved names, like `Error`; user definitions are
  duplicate-name errors.
- The runtime layout and the type checker's field list are a single contract. Both
  sides document the field order, and any change needs a runtime ABI bump.
- TLS, redirects and a database client remain future work; the client API is shaped
  so TLS can be added underneath it without changing Kode source.

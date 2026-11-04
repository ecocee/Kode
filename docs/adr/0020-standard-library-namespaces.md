# ADR 0020: Standard-library namespaces

Status: accepted for v0.4

## Context

Every library function was a global name (`json_parse`, `http_get`, `read_file`). That
does not scale: adding `parse`, `get` or `read` to more than one area collides, and it
does not read like the Python the language is meant to feel like. Real modules and
packages are a larger project (qualified names, visibility, a manifest and a registry).

## Decision

1. Recognise a small set of namespace names in method position: `json`, `http`, `fs`,
   `time` and `math`. `json.parse(text)` is resolved before the receiver is evaluated,
   so no module object exists at runtime.
2. Each namespace entry is an alias for a row in `std_fns`. A table in
   `kode-sema/src/modules.rs` maps `(module, name)` to the existing function, so there is
   no new runtime code and no second implementation.
3. `math` also exposes the constants `pi` and `e`; a member expression `math.pi` has
   type `float` and is folded to a float constant in MIR.
4. A user variable with the same name shadows the namespace; the compiler checks the
   scope before treating an identifier as a module. Modules are not reserved words, so
   this does not break existing programs.
5. Math functions are floats only: `sqrt`, `abs`, `floor`, `ceil`, `round`, `pow`,
   `min`, `max`, `sin`, `cos`, `tan`, `log`, `exp`. `log` is the natural logarithm.
   `floor`, `ceil` and `round` return floats, matching Python.

## Consequences

- Library calls read like Python without a runtime module or a new value type, and the
  old global names keep working for existing code and tests.
- Real packages still need a module system. This table is deliberately a bridge, not a
  replacement: when `import` gains qualified names, the aliases can become the
  standard library's module definitions.
- Names are resolved statically, so a namespace call has no lookup cost.

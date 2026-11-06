# 0023 — `finally` blocks

Status: accepted

## Context

`try`/`catch` handles an error, but the language had no way to say "run this
whatever happens". File handles, locks and counters that must be released on
every path had to be repeated in the block, in the handler and before every
`return`.

## Decision

- `try { } catch e { } finally { }`; the `finally` block is optional, and a
  `try` still always needs a `catch`.
- `finally` runs when the block finishes normally, when it throws (after the
  handler), when the handler throws again, and before a `return`, `break` or
  `continue` that leaves the block.
- It cannot swallow an error: an error that is already travelling keeps
  travelling after `finally` runs.
- No new error code: `finally` uses the existing error model.

## Consequences

- The lowerer keeps a stack of pending `finally` blocks together with the loop
  depth at which they were entered (`cleanups` in `kode-mir/src/lower/mod.rs`).
  `return` lowers every entry; `break` and `continue` lower only the entries
  inside the loop being left.
- A handler is lowered with its own error target, so an error raised in the
  handler still runs `finally` before it leaves the statement.
- `finally` is lowered twice (once at the join point, once on the rethrow
  path), which grows the code a little, the same way other duplicated
  constructs do.
- Covered by `tests/run-pass/finally.kode`.

# 32. A nesting limit of 128 levels

Status: accepted (M9, production core).

## Context

`kode check` on a file with 300 nested parentheses, a 2,000-term `a + a + ...`
chain, or 500 nested `if` blocks aborted with

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

exit code 134, no diagnostic, no file name, no line. All three were found by the
v0.3 audit (`docs/STATUS.md`), which ran them against the debug compiler; a
release build has a bigger effective budget per frame but the failure mode is
the same, just further away.

The cause is that almost every pass is recursive over the syntax tree: the
parser's descent, `Drop` for the boxed tree, the type checker, MIR lowering, the
code generator and the formatter. Each level costs a few frames, so the depth
that is safe is a property of the stack, not of the language. Two different
shapes reach it:

- **Recursive shapes** (`((((1))))`, nested blocks): the parser's own recursion
  overflows while parsing.
- **Iterative shapes** (`1 + 1 + ...`): the parser builds the left-deep tree in a
  loop with constant depth, and the overflow happens later, in the checker or in
  `Drop`.

## Decision

A syntax tree may be at most **128 levels** deep, and both shapes are refused
with error `E0118` before any later pass runs.

1. The parser counts its own recursion in `parse_expr`, `parse_block` and
   `type_ann`, and returns `E0118` at the 129th level. The first such error is
   fatal: parsing stops instead of reporting the same error for every enclosing
   `}`.
2. After parsing succeeds, `crates/kode-syntax/src/depth.rs` walks the finished
   tree with an explicit `Vec` stack and returns the first node deeper than 128.
   This catches the trees a loop built, which the parser never recursed over.

Both use the one constant `MAX_NESTING`, so the limit means the same thing in
both places. Depth is counted in nodes a pass actually visits, so a nested `if`
costs three levels (block, statement, condition) and one `+` in a chain costs
one. `tests/run-pass/nesting_ok.kode` pins what is accepted: 30 nested `if`
blocks, a 100-term chain, 100 parentheses.

`kode fmt` and `kode lsp` parse through the same entry point, so they inherit the
check: a deep file is a diagnostic, never a crash.

## Alternatives

- **A larger stack.** Run the compiler on a thread with a 64 MB stack. It moves
  the cliff rather than removing it — unbounded input still aborts the compiler
  — and it hides the problem from a service that compiles user input. Worth
  doing in addition, not instead.
- **An iterative parser and iterative passes.** The correct long-term answer for
  `Drop` and the walkers, but it is a rewrite of every recursive function in the
  compiler and is not reviewable in one change for a maintainer at 10 hours a
  week.
- **No limit, catch the abort.** A crash handler cannot produce a useful
  diagnostic from a stack overflow, and it converts a clean refusal into a
  process that has already written partially to stderr.

## Consequences

- Nesting past 128 levels is now a compile error with a hint, on every platform,
  in every front end, instead of a crash. Programs with 129 or more chained
  binary operators are refused too; split them or use a loop.
- The limit is a language rule, so it is documented in
  `docs/spec/03-expressions.md` and `kode explain E0118`.
- The constant lives in one place. Raising it is a one-line change once the
  compiler runs on a bigger stack, and the two tests that pin the boundary
  (`deep_*.kode`, `nesting_ok.kode`) will say whether the new value is safe.
- Generated code that concatenates thousands of terms must be rewritten by the
  generator. That is the intended outcome: such a tree cannot be compiled
  safely today anyway.

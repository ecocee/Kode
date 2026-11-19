# The self-hosted compiler (Stage 1)

The Kode compiler written in Kode. Stage S1 — the lexer — is here and green
against the oracle; S2 slice 1 — the arena and a subset parser — is in as well.

```sh
./scripts/selfhost.sh stage0     # Stage 0 dumps the oracle over the corpus
./scripts/selfhost.sh stage1     # Stage 0 builds the port, then diffs token dumps
./scripts/selfhost.sh parse      # the S2 subset: tree dumps, case by case
```

| file | what it is |
|---|---|
| `tokens.kode` | the token set, the error type, the names the dump prints, and which tokens can start a value |
| `keywords.kode` | the keyword table |
| `bytes.kode` | UTF-8 width, `byte_at` and the character-class predicates |
| `strings.kode` | string literals and their `${...}` holes, as a pure scan |
| `chars.kode` | the byte values the scanners compare against |
| `numbers.kode` | number literals, including the base-prefixed forms |
| `lexer.kode` | the scan loop and punctuation: same tokens, spans, error codes and recovery as `lexer.rs`; strings and numbers delegate to their own modules |
| `dump.kode` | the `kode tokens` line format, escaping and line/column |
| `ast.kode` | the syntax tree: one arena of nodes with `int` child ids and one `int?` parent (ADR 0042), the `kode ast` line format, and the `kids`/`span_of`/`retype` helpers the parsers use |
| `cursor.kode` | the token cursor, the parse error type and the shared error list, plus where a statement ends (`end_statement`, `at_statement_end`), how a bad line is skipped (`recover_line`), how a group's closing token is demanded (`expect_close`), how a token is named in a message (`describe`), and the nesting level and fatal flag `depth.kode` reads |
| `depth.kode` | the 128-level nesting limit (ADR 0032): `guard` on entry to the expression, block and type productions (`E0118`, then the parse stops), and a stack walk over the finished tree for what a loop built, mirroring `crates/kode-syntax/src/depth.rs` |
| `report.kode` | the diagnostic block `kode ast` prints: code, position, source line, caret and hint, rendered byte for byte as `kode-diagnostics` does |
| `contracts.kode` | the `Blocks` and `Expressions` interfaces that let the statement, expression and interpolation grammars reach each other without a module cycle |
| `infix.kode` | the infix table: operator symbols and their binding powers |
| `expr.kode` | expressions: prefix (including `spawn`), postfix chains, `??`, `is`, `catch` |
| `collections.kode` | `[lists]` and `{maps}`, which parse whole expressions between their separators through `contracts.kode` |
| `closures.kode` | `\|x: int\| x * 2` and `await expr` |
| `interp.kode` | interpolated strings and the holes they re-lex |
| `stmt.kode` | statements: blocks, `if`/`while`/`for`, `try`/`throw`, `break`/`continue`, `return`, and the dispatch to expressions and assignments |
| `assign.kode` | assignments: `name = value`, `name: Type = value`, the eleven compound operators, and the `TargetVar`/`TargetIndex`/`TargetField` built from the left side |
| `matches.kode` | `match` statements and their arms, which reach statements and expressions through `contracts.kode` |
| `types.kode` | type annotations (`T`, `[T]`, `{string: T}`, `Box<T>`, `Task<T>`, `Channel<T>`, `fn(int) -> int`, `T?`) and `Param` nodes |
| `decls.kode` | items: `pub`/`fn`, `class`, `const` and `import` |
| `enums.kode` | `enum` declarations with payload variants, and `interface` signatures |
| `parser.kode` | the program loop: items one per line, each ending where the oracle says a line ends |
| `literals.kode` | what a literal means: base-2/8/16 ints, floats shaped like `{:?}`, strings split into text and `${...}` holes |
| `builtins.kode` | the names every program starts with — builtin functions, runtime free functions, module qualifiers, prelude classes — mirrored from `std_fns.rs` |
| `diag.kode` | one checker diagnostic (code, message, span, optional hint) and the constructors that add one |
| `scope.kode` | the checker's scope stack: one frame per block, each bound name with the type it holds |
| `typing.kode` | the checker's types: the primitives, `[T]`, `{string: T}`, `T?`, names and `any`, expression typing, and whether a value fits (`E0202`) |
| `annotations.kode` | what a written type means: `T`, `[T]`, `{string: T}`, `T?`, `Box<T>`, `fn(...)`, with an unknown name as `E0303` |
| `items.kode` | what a function or class declaration puts in scope: its type parameters, parameters, fields, methods and return type |
| `binding.kode` | what `name = value` does: the type the name takes (`E0202`), a constant as the target (`E0312`), and a value that is nothing at all (`E0205`) |
| `operands.kode` | which operands each operator accepts, and what `print`/`stderr` can show (`E0203`, `E0204`, `E0210`) |
| `signatures.kode` | what a function and a class constructor take and give back: parameters in order, the declared result, and the type parameters, collected program-wide before the walk |
| `functions.kode` | a call against that record (`E0207`, `E0215`, `E0216`, `E0208`) and a return against the declared result (`E0304`, `E0305`, `E0310`) |
| `names.kode` | the program's names and the rules about declaring one: `E0301`, `E0302`, `E0306`, `E0307`, `E0308` |
| `sema.kode` | the checker (S3): name resolution (`E0201`/`E0206`), alias members, `return`/`break`/`continue` outside their place (`E0311`/`E0314`), and the walk that ties the rules together |
| `namespace.kode` | the import namespace: a module alias may not repeat (`E0408`) or hide a declaration (`E0409`), and the first in load order is the one a member call resolves through |
| `modules.kode` | the module graph: `import`/`from` resolve, load once, and export (`E0401`/`E0402`/`E0403`/`E0405`/`E0411`), keeping every alias in load order |
| `main.kode` | the entry point: `kode-stage1 tokens <file>` / `ast <file>` / `check <file>` |
| `spec/lexer_cases.kode` | escapes, a 4-byte character, a brace inside a hole |
| `spec/parse/` | the cases `parse` compares tree by tree |

Verified by comparison, not by opinion: `stage1` diffs the port's token dump
against Stage 0's over `tests/run-pass`, `tests/compile-fail`, `examples/`,
`benchmarks/` and `selfhost/spec` — byte offsets, line/column and escaped text
included.

## What the port learned about the language

- `.len()` on a string counts characters, `.size()` counts bytes. Every offset
  in a compiler is a byte offset, so `.size()` is the one to reach for; using
  `.len()` there silently truncates any file with a multibyte character.
- `stderr(...)` takes the same arguments as `print` and writes them to stderr
  (WP-10, ADR 0045), so `main.kode` reports each diagnostic with
  `stderr(render(...))` and the blank line between reports comes from `render`'s
  own newline plus the one `stderr` adds.
- Constants *and free functions* share one namespace across a program's modules,
  and `match` is a statement, not an expression. Both are worked around here; both
  are worth remembering when the parser (S2) wants lookups and tables.
- That namespace bite is real: the parser's `digit_value` silently became the
  lexer method's callee, so the lexer stopped at the wrong answer for `0x6b` and
  looped forever. Free functions in `selfhost/` now carry a module prefix
  (`parse_int`, `parse_digit`, `parse_string`).

`stage1` proves the lexer and `parse` proves the tree, each by comparison with
Stage 0. `parse` compares the slice's own cases, the port's own files and the whole
`tests/compile-fail` corpus — the oracle's own list of what a parser must refuse — so
a recovery that starts reporting an extra code shows up here, not in a later slice.
Anything the slice does not parse is an `E0102` error naming the token it stopped at,
not a silent omission, so a green `parse` means the trees and diagnostics match on
the cases it covers and nothing more.

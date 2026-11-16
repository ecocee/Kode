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
| `ast.kode` | the syntax tree: one arena of nodes with `int` child ids (ADR 0042), the `kode ast` line format, and the `kids`/`span_of`/`retype` helpers the parsers use |
| `cursor.kode` | the token cursor, the parse error type and the shared error list, plus where a statement ends (`end_statement`, `at_statement_end`), how a bad line is skipped (`recover_line`), how a group's closing token is demanded (`expect_close`) and how a token is named in a message (`describe`) |
| `report.kode` | the diagnostic block `kode ast` prints: code, position, source line, caret and hint, rendered byte for byte as `kode-diagnostics` does |
| `contracts.kode` | the `Blocks` and `Expressions` interfaces that let the statement, expression and interpolation grammars reach each other without a module cycle |
| `infix.kode` | the infix table: operator symbols and their binding powers |
| `expr.kode` | expressions: prefix (including `spawn`), postfix chains, `??`, `is`, `catch`, `[lists]`, `{maps}` |
| `closures.kode` | `\|x: int\| x * 2` and `await expr` |
| `interp.kode` | interpolated strings and the holes they re-lex |
| `stmt.kode` | statements: `name = value`, an expression on its own line, blocks, `if`/`while`/`for`, `match`, `try`/`throw`, `break`/`continue`, `return` |
| `types.kode` | type annotations (`T`, `[T]`, `{string: T}`, `Box<T>`, `T?`) and `Param` nodes |
| `decls.kode` | items: `pub`/`fn`, `class`, `const` and `import` |
| `enums.kode` | `enum` declarations with payload variants, and `interface` signatures |
| `parser.kode` | the program loop: items one per line, each ending where the oracle says a line ends |
| `literals.kode` | what a literal means: base-2/8/16 ints, floats shaped like `{:?}`, strings split into text and `${...}` holes |
| `main.kode` | the entry point: `kode-stage1 tokens <file>` / `kode-stage1 ast <file>` |
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
- There is no stderr stream, so `main.kode` writes diagnostics to `/dev/stderr`
  through `write_file`. A `stderr(text)` builtin is the fix; until then this is
  a Unix-only path (see `docs/self-hosting.md`).
- Constants *and free functions* share one namespace across a program's modules,
  and `match` is a statement, not an expression. Both are worked around here; both
  are worth remembering when the parser (S2) wants lookups and tables.
- That namespace bite is real: the parser's `digit_value` silently became the
  lexer method's callee, so the lexer stopped at the wrong answer for `0x6b` and
  looped forever. Free functions in `selfhost/` now carry a module prefix
  (`parse_int`, `parse_digit`, `parse_string`).

`stage1` proves the lexer and `parse` proves the tree, each by comparison with
Stage 0. Anything the slice does not parse is an `E0102` error naming the token it
stopped at, not a silent omission, so a green `parse` means the trees match on the
cases it covers and nothing more.

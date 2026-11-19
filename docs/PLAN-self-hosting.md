# Plan: self-hosting Kode

Status: **executing**. This plan is the sequence, the work packets and the gates
it is built against — not a promise of a date. Every packet and every stage is
one branch, so `git log` reads like this file.

The plan is executable: `scripts/selfhost.sh` is its driver.

```
./scripts/selfhost.sh gates        # every work packet: target declared vs today
./scripts/selfhost.sh readiness    # the readiness level reached, and what is next
./scripts/selfhost.sh all          # Stage 0, Stage 1, then the work packets
./scripts/selfhost.sh stage0|stage1|stage2|compare
```

Companions: `docs/self-hosting.md` (what exists), `docs/PLAN-v1.md` (v1.0),
`docs/STATUS.md` (measured state), ADR 0030 (monomorphization), ADR 0038 (`weak`), ADR 0042 (the AST arena),
ADR 0039 (enum payloads), ADR 0040 (generic classes).

## 1. Definition of done

"Kode runs everything in self-host" means a three-stage bootstrap where the
compiler you use is a Kode program:

| stage | built by | what it is |
|---|---|---|
| **Stage 0** | hand | the Rust compiler, today's `kode` — the oracle, never removed |
| **Stage 1** | Stage 0 | a `kode` compiler written in Kode that compiles the whole corpus |
| **Stage 2** | Stage 1 | Stage 1 compiled by itself; must behave identically |

Self-hosting is done when all of the following hold, with no manual step:

1. `./scripts/selfhost.sh all` goes Stage 0 → Stage 1 → Stage 2 → `compare`.
2. Every file in `tests/run-pass/` prints its `.out` under Stage 1 and Stage 2,
   leak-clean with `KODE_LEAK_CHECK=1`.
3. Every file in `tests/compile-fail/` produces its documented code and the same
   `--format=json` diagnostics (code, message, span, hint, order) under both.
4. `kode ast` output is identical across stages on the whole corpus.
5. `benchmarks/report.md` records per-stage compile time, peak RSS and compiler
   size.
6. The `selfhost` CI job fails when any of the above stops holding.

Not part of done: a backend written in Kode (Stage 1 emits LLVM IR text and calls
`clang` through `exec`), deleting the Rust compiler, or matching Stage 0's speed.

## 2. Readiness levels

Levels are cumulative and machine-checked by `./scripts/selfhost.sh readiness`.
A level is reached when its gate is green, not when its code "mostly works".

| level | meaning | gate |
|---|---|---|
| **R0** | the oracle exists | `dumps/stage0` present and Stage 0 agrees with itself |
| **R1** | the lexer runs in Kode | `dumps/stage1` agrees with `dumps/stage0` on every token |
| **R2** | the language can express a compiler | every work packet in §3 is `met` |
| **R3** | the parser runs in Kode | `dumps/stage2` tree dumps agree with Stage 1's |
| **R4** | the checker runs in Kode | `check --format=json` parity on both corpora |
| **R5** | the front end interprets programs | every run-pass `.out` through the Kode evaluator |
| **R6** | bootstrapped | Stage 1 and Stage 2 agree on everything, leak-clean |

Measured today: **R1** — 270 token dumps byte-identical, including the work packet
probes, and the S2 parser is byte-identical on 51/51 cases and on all 25 files of
the port itself. R2 is open only on the two deferred packets: `gates` reports
11 met, 2 blocked (WP-07 `weak` and WP-08b `u8`/`i32`, both off the path).

## 3. Work packets

Each packet is one branch, one probe, one measurable gate. A probe lives in
`selfhost/spec/gates/` and declares its target in a comment:

```
// work packet: WP-03 enum payload construction
// target: ok
```

`target: ok` means "this program must compile and run". `target: E0417` means
"this must stay rejected with that code" — a guard, not a gap. `gates` compiles
each probe with Stage 0, records the observed result, and prints `met` or
`blocked`, so the table is the build's to-do list rather than an opinion.

| packet | goal | probe | today |
|---|---|---|---|
| **WP-01** | generic classes with inferred type arguments (ADR 0040) | `wp01-generic-classes` | **met** |
| **WP-02** | generic methods: `fn id<T>(x: T) -> T` | `wp02-generic-methods` | **met** |
| **WP-03** | construct an enum payload variant (ADR 0039 slice 1) | `wp03-enum-payload-construct` | **met** |
| **WP-04** | `match` a payload and bind its fields | `wp04-match-payload` | **met** |
| **WP-05** | `is` on a payload variant, with narrowing | `wp05-is-narrow` | **met** |
| **WP-06** | optional scalars as class fields (ADR 0041) | `wp06-optional-scalar-fields` | **met** |
| **WP-06b** | optional scalars as parameters and returns (ADR 0041 §5) | `wp06b-opt-params` | **met** |
| **WP-07** | `weak` back-edges (ADR 0038) | `wp07-weak` | E0114 — deferred by ADR 0042, not on the S2 path |
| **WP-08** | byte buffers and `0x`/`0b`/`0o` literals (ADR 0043) | `wp08-bytes` | **met** |
| **WP-08b** | `u8`/`i32` as numeric types (ADR 0043 §4) | `wp08b-fixed-width` | E0303 — deferred by ADR 0043, not needed to read bytes |
| **WP-09** | `StringBuilder` (ADR 0044) | `wp09-string-builder` | **met** |
| **WP-10** | a `stderr` stream (ADR 0045) | `wp10-stderr` | **met** |
| **WP-11** | the AST is an arena (ADR 0042) | `wp11-arena` | **met** |
| **WP-12** | streaming process output | `exec` on 10 MB without buffering | open |

Order: WP-03 → WP-04 → WP-05 (one representation, then the two ways to take it
apart), then WP-02, then WP-06 → WP-10 as the stage that needs them arrives.
WP-11 is decided before the AST is written rather than after (ADR 0042); that decision
takes WP-07 off the S2 path, so its probe stays red on purpose as the open language
feature rather than a port blocker. WP-08b is red for the same reason: `u8`/`i32`
are numeric types of their own and reading bytes does not need them (ADR 0043 §4).
WP-09 is met: `StringBuilder` is what S5 writes IR text through, and the gain is
measured in `benchmarks/report.md` (185.2 ms → 5.2 ms on 8,000 lines of text).
WP-10 is met too: `stderr(...)` takes `print`'s arguments on the diagnostic
stream, so the port no longer opens `/dev/stderr` (ADR 0045). What is left in R2
is the two deferred packets, not a blocker the port is waiting on: `readiness`
counts them apart, names the next packet in build order, and, when nothing on the
path is blocked, points at S3 and records that WP-12 (process streaming) stays
until S5.

Each packet lands with: its probe turning `met`, a `tests/run-pass/` program with
`.out` (or a `tests/compile-fail/` pair for guard cases), the codes it needs in
`docs/errors/index.md`, the ADR it belongs to, and `CHANGELOG.md`.

## 4. Stages

### S0 — bootstrap harness — **done**

`kode tokens` / `kode ast` are the oracles (`crates/kode-syntax/src/dump.rs`),
`stage0` is green on a clean checkout, and the harness checks Stage 0 against
itself first so a broken comparison cannot look like a port. Runs in CI.

Risk: the dumps are a compatibility surface. Mitigation: documented as debug
output; `--format=json` stability (AGENTS.md 9) applies from day one.

### S1 — lexer in Kode — **done**

`selfhost/{tokens,keywords,bytes,chars,numbers,strings,lexer,dump,main}.kode`
(~1,490 lines, each file under 500: `chars.kode` holds the byte values the scanners
share, `numbers.kode` and `strings.kode` the two scanners that are pure functions of
the source) plus `selfhost/spec/lexer_cases.kode`.

`stage1` builds the port with Stage 0 and reports **216 of 216 token dumps
byte-identical** (3.0 s / 6,736 KB for Stage 0 including trees, 2.3 s / 3,792 KB
for Stage 1 tokens).

Two findings carried forward: `.size()` is bytes and `.len()` is characters, so
every offset is a byte offset; and there is no stderr stream (WP-10).

### S2 — parser and spans (branch `selfhost/s2-parser`)

Opens when WP-03/04/05, WP-06 and WP-11 are `met` (ADR 0042).

Files: `selfhost/ast.kode`, `selfhost/parser.kode` (~2,500 lines estimated).
Acceptance: identical `kode ast` dumps on the corpus, including E01xx syntax
errors (message, span, hint). The AST is one arena with `int` ids (ADR 0042), so
freeing the tree frees every node, and the compiler's own leak check runs from here on.

**Slice 1 is in.** `selfhost/ast.kode` is the arena: `Node { kind, attrs, kids }`
with children as ids, no parent links, and `ast_lines` printing the oracle's format.
`selfhost/parser.kode` parses assignments of a literal or a name and call
statements whose arguments are literals or names, with `Int`/`Float`/`Str`/`Bool`/
`Ident`/`Call`/`Arg`/`Assign`/`TargetVar`/`ExprStmt` nodes. `kode-stage1 ast` prints
it, and `./scripts/selfhost.sh parse` diffs it with `kode ast` over
`selfhost/spec/parse` and `selfhost/spec/parse-errors` (48/48 cases: trees and
diagnostics byte-identical, `expr_count` included).

Slice 1b adds the expression grammar: every infix operator at the oracle's
binding powers (`or` 2, `and` 4, `|` 6, `^` 7, `&` 8, comparisons and `in` 10,
`<< >>` 12, `+ -` 16, `* / %` 20), prefix `-`, `~` and `not` (9), grouping
(parentheses leave no node, only the inner expression's span), and a line that
ends after an operator. Floats are re-read as values and printed the way Rust's
`{:?}` does, so `2.50` is `2.5` and `1e6` is `1000000.0`.

Slice 2 adds postfix expressions and the literals that hold other values:
`.name`, `.name(args)` with named and trailing-comma arguments, `?.name`, `[i]`,
`[a..b]`/`[..b]`/`[a..=b]`/`[..]`, `obj!`, `[lists]`, `{maps}` with `Entry` nodes,
`none`, and string interpolation (`Interp` of `Lit` text and `Hole` expressions,
the hole re-lexed with its absolute offset so inner spans stay true). It also
made the parser a set of modules rather than one file: `cursor.kode` (the shared
position and error list), `infix.kode` (the binding-power table), `expr.kode`
(expressions), `literals.kode` (values) and `parser.kode` (statements).

Slice 3 adds blocks and control flow: `{ one statement per line }` as a body,
`if`/`else`/`else if` (`Then`/`Else` wrappers, `else if` nested through an `Else`
block), `while`, both forms of `for` (a range with `..`/`..=` at `BP_ARITH`, and an
iteration with one or two variables), and `break`/`continue`. A statement ends at a
newline, the end of the file, or the `}` that closes its block — the oracle's rule,
which is why `else` has to follow `}` on the same line.

Slice 4 adds items and calls back out of a function: `fn` declarations (an optional
`pub`, type parameters, parameters with annotations, an optional return type, and
either a block or `=> expr`), `return` with or without a value, and type
annotations — names, `[T]`, `{string: T}`, `Box<T>` and the `T?` suffix. Two of the
oracle's spans are kept verbatim: a `Fn` spans only `fn` and its name, and a
`TypeList`/`TypeMap`/`TypeGeneric` spans only its opening token.

Slice 5 adds the two shapes that hold the rest of a program: `class` declarations
(an optional `pub`, type parameters, `implements` names, fields with their type and
an optional `= default`, and methods that dump as `Method` rather than `Fn`), and
`match` statements (`Scrutinee`, then `Arm`s whose pattern is `Wildcard` or a
`PatternValue` at `BP_ARITH` and whose body is always a `Block`, whether the source
wrote a block or one statement). A method's `pub` is accepted and dropped, which
is what the oracle does, and a `Class` node has no `span` attribute.

Slice 6 adds the remaining declarations: `enum` (variants with optional
`(name: type, ...)` payloads, separated by commas or newlines), `interface`
(signatures only; a body is `E0236`), `import` in all three forms (a path, a path
with `as`, and `from "..." import A, B`) and `const` (an optional annotation, then a
value). Enums carry no type parameters — `enum Maybe<T>` is `E0103` in the oracle,
which is left alone. An `import` only records its path; the file it names is not
read, because S2 parses one source at a time.

Slice 7 adds errors and the optional operators: `throw`, `try { } catch err { }
finally { }` (a `Try` node with a body `Block`, an optional `CatchVar` and `Handler`
and an optional `Finally`, and no span of its own), `a ?? b` at the loosest binding
power, `x is Shape.Circle` at comparison precedence, and `expr catch err { ... }`.

`catch` binds at binding power 0, which means it also appears inside parentheses,
argument lists, collections and interpolation holes. A `catch` body is a block of
statements, so the expression grammar needs the statement grammar while the
statement grammar already imports the expression grammar — and a module cycle is
`E0402`. The two meet through `blocks.kode`: a one-method `Blocks` interface that
`expr.kode` depends on and `Stmts` implements, handed to `Exprs` by whoever builds
it. A hole re-lexes into its own cursor and asks for the same parser over that
cursor (`with_cursor`), so a `catch` inside `${...}` parses its body from the hole's
tokens and every span stays absolute. The nesting is covered by
`selfhost/spec/parse/catch_expr.kode`.

Slice 7 pushed `stmt.kode` just over the size rule, so the line-end handling moved
to where it belongs: `Cursor.end_statement`, `Cursor.at_statement_end` and
`Cursor.recover_line` in `cursor.kode`, and `starts_value` next to the token set in
`tokens.kode`. No grammar changed and no tree moved — the 22 cases stayed
byte-identical, and the `E0101` report keeps its code, position and text.

Slice 8 closes the expression grammar: `|x: int| x * 2` (every parameter typed;
`E0241` otherwise, and `||` for no parameters) and `await expr`, which takes a unary
so `await f().g()` awaits the chain. A closure body is a whole expression, so `catch`
and `await` nest inside it. Both live in `closures.kode` behind the same
`Expressions` interface that `interp.kode` uses, which is also how `expr.kode` came
back under the size rule (453 lines): interpolation moved to `interp.kode`,
`kids`/`span_of` to the arena, and `Blocks` grew an `Expressions` sibling in
`contracts.kode`.

Slice 9 adds the last prefix form, `spawn f(1, 2)`. The oracle parses a whole postfix
chain after `spawn` and keeps it only if the chain ended as a plain call, so
`spawn obj.run(1)` (a method call) and `spawn f().g` are both `E0117` over the whole
chain – the port decides the same way, on the finished chain, and prints the same
code, message, span and help. `Spawn` holds a call's `callee`, `callee_span` and `Arg`
children under a wider span, so `Ast.retype` re-names the node the call built and
replaces its rendered `span` attribute instead of appending a second one; the oracle
allocates a second node there, so `expr_count` is bumped to match. `spawn` had to
join `starts_value` to be allowed at the start of a statement.

Slice 10 gives the port the oracle's *diagnostics*, not just its trees. `report.kode`
renders a `ParseError` exactly as `kode-diagnostics` does — `error[E0103]: ...`, the
`--> file:line:col` position, the source line, a caret whose width is the span's
character count (a zero-width span still shows one) and the `= help:` line — and the
columns count characters, so a caret lands under a multibyte character instead of
inside it.

Matching the text meant matching how a token is *named*: `Cursor.describe` wraps a
name, number or spelling in backticks, calls a string literal `a string` and the two
ends of the input `the end of the line` / `the end of the file`, which is why
`x = 1 +` now says `found the end of the file` rather than an empty set of
backticks. Codes, messages, hints and spans were then corrected site by site against
`crates/kode-syntax/src/parser/`: `E0103` (not `E0102`) closes groups and carries no
hint, `E0105` reports a `fn` in a block, `E0108` a two-variable range loop, `E0110` a
missing parameter or field name, `E0112` an unassignable target, `E0116` a
non-string import path, and the generic statement message is gone because the oracle
reports those through the expression parser (`expected a value, ...`). Recovery
follows the oracle too: a failed item, parameter or class member abandons the rest of
its line (`Parser::recover`) and the following line is reported on its own, so the
port emits the same *set* of errors in the same order.

`selfhost/spec/parse-errors` holds one case per code, and `check_parse` compares
stderr as well as stdout for every case in both directories.

Slice 11 adds the shape that had been missing: `x: int = 5`, a declaration that is
also an assignment. The `Assign` node carries the type between the target and the
value, exactly where the oracle puts it, and a missing `=` is `E0107` ("expected `=`
after the type", with the `count: int = 0` hint) over the token that was found. The
`for`-style target is *not* counted for `expr_count`: `x = 1` reuses the id of the
name it parsed, so its count stays, while a declaration never made one — counting it
made `expr_count` one too high until that was fixed against the oracle.

Slice 12 finishes the arena: every node now carries `parent: int?`, set when it is
attached, so a later pass can climb from a node to the statement that holds it without
a second tree or a set of reverse edges. The child edge stays the ordered `kids` list
(ADR 0042 sketches `first_child`/`next_sibling`; a list holds the same information in
the order the dump needs). Two kinds of node are deliberately parentless: the root,
and a node the parse built and then dropped — the key of a `{K: V}` annotation is
parsed to be checked and never dumped. A throwaway probe over every tree case checked
that each attached node names a parent that really lists it: no problems, and the only
orphans were those map keys.

The new `deep_tree` case (a generic class with methods, interpolation, a `match`, a
nested `for`/`if`, `spawn` and `await`) turned up three gaps at once. Two are recorded
below; the third was a diagnostic: a match arm without `=>` is `E0103` ("expected `=>`
after the pattern, found …") over the token found, and it ends the whole `match`, with
its `parse-errors` case. That pushed `stmt.kode` over the size rule, so `match`
statements moved to `matches.kode` behind a `Blocks` interface that now offers
`statement()` for a single-statement arm.

Slice 13 closes the first of those: compound assignment. The left side is parsed as a
whole expression and only then turned into a target — `Ident` to `TargetVar`, `Index`
to `TargetIndex`, `Member` to `TargetField` (never a `?.`, which is `E0112`) — which is
what makes `xs[0] = 1` and `obj.name += 2` work at all; the port had only ever
accepted `name = value`. The parsed expression stays in the arena, because the
oracle's `expr_count` still counts it, and for `op=` it is also the left side of the
`Binary` that becomes the value, so `x += 1` dumps `Assign(TargetVar,
Binary(Ident, Int))`. A `TargetIndex` shares the `Index` node's children, which the
dump prints twice. Assignments moved to `assign.kode` to keep `stmt.kode` the
dispatch.

The new `assign_targets` case covers all eleven operators plus index, map, field and
nested targets, and `parse-errors/e0112_optional_target.kode` pins `obj?.name += 1`.
Parsing the port's own source then exposed one unrelated dump bug: `""` was an
`Interp` with no parts where the oracle has `Str value=""`, fixed in `interp.kode`
with `empty_string.kode` as its case.

With that, **the port parses all 25 of its own files tree-identically**, and `parse`
now checks that on every run (`self-parse: 25/25`). 29 tree cases, 22 diagnostic
cases, 51/51 byte-identical.

Slice 14 closes the type grammar. `fn(int) -> int` was the one annotation the port
could not parse; the `|n: int| int` spelling written down here before is not Kode
syntax at all — the language spells a function type `fn(int) -> int`, as
`docs/spec/05-functions.md` has all along, so this was a port gap and not a missing
language feature. The `TypeFn` node spans only the `fn` keyword and holds its
parameter types, then its return type, which is also why `fn(int) -> int?` puts the
optional on the return. `Task<T>` and `Channel<T>` dump as their own kinds instead
of a generic named `Task`. Parsing the corpus's own malformed annotations also
showed three wrong diagnostics: a missing `]`, `}`, `>` or `->` is `E0103` over the
token found (the port said `E0102` in its own words), and a map key that is not
`string` is `E0115` over the key's first token, which the port did not check at
all. With that, 30 tree cases and 24 diagnostic cases are byte-identical — 54/54 —
and self-parse is still 25/25.

Risk: this is where a missing language feature becomes duplication. Stop and
report rather than invent a workaround.

### S3 — checker (branch `selfhost/s3-checker`)

Files: `selfhost/sema.kode`, `selfhost/types.kode` (~4,500 lines).

Acceptance: `kode check --format=json` identical to Stage 0 for the 107
compile-fail programs (code, message, span, hint, order) and clean for the 57
run-pass programs.

**The gate comes before the checker.** `./scripts/selfhost.sh check` runs the port's
`check` over both suites and compares the **set of codes** it reports with the code
each `.err` names; a run-pass program must report nothing. It is staged on purpose:
codes first, then message and span, then the JSON form with order, because a code
set is a number that can move from the first commit instead of only at the end.

**Slice 1 is in.** `selfhost/sema.kode` resolves names: top-level declarations,
imports, builtins, the runtime's free functions and modules (a table mirrored from
`std_fns.rs`, since the port cannot read it), the prelude classes, and block scopes
— parameters, assignment targets, `for` variables, `catch` variables (statement and
expression forms), match-arm payload bindings and closure parameters. It reports
`E0201` (unknown variable) and `E0206` (unknown function).

**Slice 2 is in: the module graph.** `selfhost/modules.kode` reads what `import` and
`from` name instead of only recording the path. A path resolves relative to the file
that wrote it, `.kode` is appended when the path has none, and one file is read once
however many modules reach it; a path that cannot be found is `E0401` and a file that
imports itself, directly or through others, is `E0402`, both pointing at the path
token of the import that asked, in the file that asked. A bare `x` is the package
root's (`kode.toml`: `root = "lib"`), which is what makes
`tests/run-pass/modules_alias.kode` resolve, and a module with top-level code is
`E0403`. What an import then reaches belongs to the same slice: a `from` name must be
declared (`E0405`) and `pub` (`E0411`, "does not export"), and a call through an
alias must be a declared (`E0410`, "has no method") `pub` top-level (`E0411`, "is
private to module"). Loader diagnostics render through `report.kode`, so they are
byte-identical to the oracle's, snippet and help included. No dependency's source
root is resolved, because nothing on the path reads a dependency yet.

**Slice 3 is in: the import namespace.** `selfhost/namespace.kode` closes the family
the oracle keeps in the checker (`collect_modules`): a module alias is a program-wide
name, so it may not repeat (`E0408`, "is imported twice") or hide a function,
constant, class, enum or interface (`E0409`, "hides another declaration", with the
oracle's hint to pick another name). The alias list is in load order — the imports of
a file are listed after the files they reach — so the *second* of two equal aliases is
the one reported, and an alias that hides a declaration is reported even when a later
file repeats it. Messages and hint are the oracle's, word for word. The first alias for
a name is the one that survives (the oracle's `module_index`), so a member call
resolves through it: `tests/compile-fail/e0408_duplicate_alias_across_files.kode`
proves the order from the outside, because its `E0410` can only appear when the alias
from the imported file — not the entry's duplicate — is the one in force.

The names every program starts with are their own file (`selfhost/builtins.kode`,
mirrored from `std_fns.rs`), so `sema.kode` has room for the next family.

**Slice 4 is in: types.** `selfhost/typing.kode` says what a value is — the
primitives, `[T]`, `{string: T}`, `T?`, a named class, and `any` for whatever this
slice cannot see through — types literals, locals, unary and binary expressions, list
and map literals and the calls it can name, and resolves annotations (an unknown name
is `E0303`, `u8` with the fixed-width hint). `selfhost/operands.kode` then holds the
two places a type is wrong in a way the program wrote out: which operands each
operator accepts (`E0203`, plus `E0210` for a mixed int and float) and what
`print`/`stderr` can show (`E0203`, which is what makes `print(users)` with a list of
class instances an error). An assignment keeps the type its target already had, so
`x = 1` then `x = "a"` is `E0202`, as is a value that does not fit an annotation.
`selfhost/scope.kode` is the frame stack those rules read, which is how `sema.kode`
stays under its size limit.

**Slice 5 is in: declarations.** `selfhost/names.kode` is the program's namespace and
the rules about declaring a name. `sema.kode` collects every file's declarations
before it walks any of them, which is what lets `stmt.kode` use `infix.kode`'s
constant without importing it, and the same walk reports what cannot stand — `E0301`
(a name defined twice, with the oracle's wording when the name is a class or the
built-in `Error`), `E0302` (a builtin redefined), `E0306` (a parameter or a field
twice), `E0307` (a constant that is not a plain value, hint included) and `E0308` (a
constant defined twice). The tree work that slice once held moves out as well:
`selfhost/annotations.kode` turns a written type into a `Type` (`E0303`),
`selfhost/items.kode` is what a function or class declaration puts in scope, and
`selfhost/binding.kode` what `name = value` does — `E0312` for a constant, `E0205` for
a value that is nothing at all, and `E0202` when it does not fit. A failed `E0202`
declares no name, which is why `tests/compile-fail/e0202_annotation.kode` reports
`E0201` on the next line, exactly as the oracle does. `sema.kode` also tracks whether
it is inside a function and how many loops deep, so `return` at the top level is
`E0311` and `break`/`continue` outside a loop `E0314`; the operator rules gained
`E0204` for a literal division by zero. Every new rule is silent when it cannot prove
the mistake, and the gate after it is **105/165** with **self-check 37/37**.

**Slice 6 is in: the nesting limit, and the corpus in the parse gate.**
`selfhost/depth.kode` ports ADR 0032 the way the oracle enforces it, in two places.
A guard on entry to the expression, block and type productions reports `E0118` at the
token the oracle points at and, once it has, stops the parse — the tree is unreliable
from there, so nothing else is reported. A walk over the finished tree, with an
explicit stack and the oracle's own push order, catches what a loop built: a long
`1 + 1 + ...` is left-deep and costs the type checker, MIR lowering and the code
generator one frame per term even though the parser never recursed. Both use one
`MAX_NESTING`; the walk runs only on a tree the parser produced without errors.
Recovery for four productions was wrong in the same way — it reported one error past
the one the oracle gives: a `fn` without a name and a `for` without a loop variable
now report `E0110` and skip the line, a type the oracle refused ends its typed
declaration instead of adding `E0107`, and an interface signature with a body ends
the interface after `E0236`. The `parse` gate now covers the whole compile-fail
corpus as well as the slice's own cases: **169/169** cases and **39/39** of the
port's own files, and the gate after it is **109/165**.

**Slice 7 is in: signatures, calls and returns.** `selfhost/signatures.kode` records
what every function and every class constructor takes and gives back — parameters and
their types, the declared result, and the type parameters an argument can bind — before
any file is walked, because the namespace is program-wide. A class's constructor is its
fields in declaration order; a field with a default value may be left out but keeps its
position, so `class A { x: int = 0, y: int }` still needs `A(1, ...)` for `y`.
`selfhost/functions.kode` then checks a call against that record: `E0207` for too many
arguments, `E0215` for an argument the callee has no parameter for, `E0216` for a
parameter left out, and `E0208` for the wrong type — with the oracle's second form when
the parameter is one of the callee's type parameters, bound at its first argument
(`T`, `[T]`, `{string: T}` and `T?` all bind) and compared at the rest. A return is
checked the same way: `E0305` when a function that declares a result can fall off its
last line (a `return` or `throw` at the end, both branches of an `if`, every arm of a
`match` — a `try` never counts), `E0310` for the wrong value or for a bare `return`, and
`E0304` for an `=>` body that gives a value the declaration never mentioned. A call the
checker refuses declares nothing, so the assigned name is unknown on the next line, as
in the oracle. Two type rules were wrong and are fixed: `int?` fits `int?`, and a type
parameter inside a container is recognised. The gate after it is **122/165** with
**self-check 42/42**.

**Slice 8 is in: builtins, runtime functions and methods.** `selfhost/stdlib.kode` is
the third signature table. It mirrors `crates/kode-sema/src/std_fns.rs` and the builtin
arms of the oracle's checker: the runtime functions a program calls by name (`exec`,
`read_file`, `args`, ...), the one-argument builtins with the words the oracle uses to
describe each (`len` "a string, a list or a map", `bytes` "a length"), and
`StringBuilder`'s `push` and `build`. A builtin takes kinds rather than types — `len`
accepts a list of anything — so a row carries the kinds one argument may be.
`signatures.kode` records a method as `Class.method` with the class's type parameters
in scope for its annotations, and keeps the bare name for return messages, because the
oracle says `m` there but `P.m` in a call. `functions.kode` resolves a method call
through its receiver's type (`StringBuilder()` types as its own class now, not `any`),
and the three message shapes are kept apart on purpose: a runtime function names its
arguments positionally ("argument 2 of `exec` should be `[string]`, found a string"), a
builtin describes the value it wants ("`bytes` expects a length, found a string"), and
a method of a class the program declares is a call like any other, help included. A
free builtin refuses a named argument once, at the first one (`len(x: 1)`); a builtin
method accepts a name the way a declared one does (`sb.push(s: "a")` is quiet,
`sb.push(x: 5)` reports the type) and `sb.build(1)` is the plain arity `E0207`.
A row also carries the call's result as a **spec** string (`"int"`, `"[string]"`,
`"string?"`, `"ProcessResult"`), not a `Type`: `typing.kode` owns `Type` and has to read
these rows, so the table cannot import it back; `typing.type_from_spec` reads one. That
gives an expression its type through the table, and the print rule then reports what the
oracle does: `print(exec(...))` is `E0203` ``cannot print `ProcessResult` yet``, a call
that returns nothing is `E0205` "this call returns nothing, so there is nothing to
print", and `print(env("HOME"))` is ``cannot print `string?` yet`` — an optional prints
only when it holds a scalar, and a bare `none` never does.

`Signature` gained `is_ctor`, because a constructor whose call is refused still builds
the instance (`a = A()` leaves `a` declared) and so does a builtin (`b = bytes("four")`
reports only the argument), while a refused function declares nothing. The gate after
it is **126/165** with **self-check 43/43**, and **130/169** once four fixtures cover
the new rules (`e0203_print_process_result`, `e0205_print_nothing`,
`e0203_print_optional_string` and the run-pass `optional_scalar_print`, which prints a
`none` `int?`, a `float?` and a `bool?`).

**Slice 9 is in: containers and conditions.** `selfhost/containers.kode` holds the rules
whose subject is a value's shape. A condition that is not a bool is `E0209` (the oracle's
`if` and `while` wording; an `else if` is another `If` inside the `Else`, so it is checked
by the same rule). Indexing is `E0211`/`E0212`: a list index is an int, a map key a string,
a `Json` value either, and everything else cannot be indexed at all — with the target form
saying why a `Json` value cannot be changed in place. A slice reads its bounds first (a bad
bound on a map is the bound's error alone) and then wants a list. A list or map literal
holds one type, `E0213`, and cannot hold `none` or nothing; a literal the checker refuses
declares no name, so the next line is `E0201` (`e0213_mixed_list`). `for ... in` over
anything but a list or a map is `E0214`, and the loop now binds the item (or the index and
the item), the key (or the key and the value) with the types they really have.
`selfhost/typing.kode` answers what an `Index` or `Slice` is worth and what a loop binds,
without reporting; `containers.kode` makes the same decision out loud. Two things it
needed: `none` fits only an optional (so `[1, none]` is `E0213`), and an enum prints, so a
loop over `[Color]` is silent. The gate after it is **138/169** with **self-check
44/44**. `sema.kode` stayed under 500 lines by moving the call-fails question to
`functions.call_fails` and the loop's binding to `containers.open_for_in`.

The gate itself changed what it measures. It compared the port's codes with the `.err`
file, which names what the Rust conformance test looks for and may leave out the
follow-on codes (`E0102` after a parser error, `E0201` after a failed call); a port that
agreed with Stage 0 could fail on that. It now compares with the oracle's own output, so
the parser fixtures and the named-argument ones became green by measuring the right
thing. A closure stays silent: its parameters are not in the table yet.

Every rule stays quiet when it cannot prove the mistake: an operand that is `any` (a
call, a method call, an index), an optional operand (narrowing is a later slice) and
two named types (this slice does not read `implements`, so a class stored as one of
its interfaces must stay legal) are all accepted. Preferring silence to a wrong
diagnostic is what keeps the run-pass suite clean while the families land.

Why the module graph comes before the other families: the namespace is program-wide,
so the checker cannot resolve `stmt.kode`'s `BP_ARITH` — a const in `infix.kode` that
`stmt.kode` does not import — without every file the program loads. `check` now loads
the graph and checks all of it, and `check_sema` checks the port's own sources as a
second gate: **self-check 37/37**, every file of the port clean under its own
checker. Measured: **95 of 165** programs report the expected code set, with the
split `38` compile-fail exact, `57` run-pass clean, `62` compile-fail with no
diagnostic yet (the families below), `6` where the port's parse errors still differ
from the oracle on the wider corpus, and `2` elsewhere.

A checker diagnostic still prints without a location — code, message and the
oracle's hint, but no file, line or caret. Loader diagnostics already render through
`report.kode` and are byte-identical; this is the spans-and-messages slice, and it is
what the JSON form needs before its order can be compared.

Next families, each one commit: types (resolution already gives the scope for it),
then arguments and the `E0208`/`E0210` family, then item signatures and generics,
then the spans and messages those diagnostics already carry, and the JSON form with
the oracle's order once the families stop moving.

Risk: wording is coupled to Rust. Mitigation: one diagnostic family per commit,
the JSON diff names the differing field.

### S4 — MIR and an interpreter (branch `selfhost/s4-interpreter`)

Files: `selfhost/mir.kode`, `selfhost/eval.kode` (~3,500 + 1,500 lines).

An interpreter is not a detour: it makes the front end testable before code
generation exists and runs every run-pass program against its `.out` without a
linker.

Acceptance: `kode mir` dumps match structurally; every run-pass `.out` matches.

### S5 — native code generation (branch `selfhost/s5-backend`)

Files: `selfhost/codegen.kode`, `selfhost/driver.kode` (~2,000 lines). Emits LLVM
IR text, writes it to a temp file, calls `clang`/`cc` through `exec`, and links
the existing runtime archive.

Acceptance: every run-pass program links and prints its `.out`; binary sizes
recorded, not asserted to the byte. This is the only stage that can produce a
wrong binary that still runs, so output parity plus `KODE_LEAK_CHECK=1` is the
gate.

### S6 — Stage 1 compiles the corpus (branch `selfhost/s6-stage1`)

Acceptance: Stage 1, running as a compiled Kode program, passes `stage1`: 46
run-pass outputs, 98 compile-fail codes, leak-clean, `kode fmt` unchanged. Rust
`cargo test --workspace` stays green throughout.

### S7 — Stage 2 and reproducibility (branch `selfhost/s7-stage2`)

Acceptance: `compare` reports parity between Stage 1 and Stage 2 on the whole
corpus; `benchmarks/report.md` gains a self-hosting section. Where objects are
byte-identical, say so; where not, record which bytes differ and why (embedded
paths are the expected cause).

## 5. The differential oracle

| stage | oracle | must match exactly | may differ |
|---|---|---|---|
| S1 lexer | `kode tokens` | token, text, byte span, line/column | internal ids |
| S2 parser | `kode ast` | tree shape, spans, E01xx diagnostics | node allocation order |
| S3 checker | `kode check --format=json` | codes, messages, hints, spans, order | nothing |
| S4 MIR | `kode mir` dump | structure and types | temporary numbering |
| S5 backend | stdout + leak check | output, exit code, zero leaks | binary size within a band |
| S6/S7 | all of the above, Stage 1 vs 2 | everything above | object bytes only if §7 gates it |

## 6. Build order

```
WP-03 → WP-04 → WP-05 ─┐
WP-01 (met) ───────────┼→ S2 parser → S3 checker → S4 MIR + interpreter
WP-06 → WP-07/11 ──────┤                              → S5 backend
WP-02 ─────────────────┘                              → S6 stage 1
WP-08, WP-09, WP-10, WP-12 → S5                        → S7 stage 2
WP-01..WP-10 all `met` → R2
```

Size, to be replaced by real numbers as stages land:

| stage | estimate (Kode lines) | basis |
|---|---:|---|
| S1 lexer | ~1,200 | done at 1,250 |
| S2 parser | ~2,500 | Rust parser + AST |
| S3 checker | ~4,500 | `kode-sema` 5,386 minus stdlib rows |
| S4 MIR + evaluator | ~5,000 | `kode-mir` 4,536 minus RC passes |
| S5 backend | ~2,000 | `kode-llvm` 1,364 + driver via `exec` |
| S6/S7 glue | ~1,000 | CLI, manifest, harness |

## 7. Risks

| risk | effect | mitigation |
|---|---|---|
| a prerequisite lands late | the stage it blocks cannot start | packets are ordered by what they block; `gates` shows the queue |
| the ported compiler is much slower | bootstrapping takes minutes | measure per stage; no optimization before S6 |
| diagnostics wording is unreachable | S3 never closes | one family per commit, JSON diff names the field |
| RC cycles in the AST | the compiler leaks | closed by ADR 0042: nodes are one flat list and every edge is an `int?`, so no cycle is expressible; the port's leak check runs in CI |
| platform-specific I/O (`/dev/stderr`, `exec`) | the port is Unix-only | WP-10, WP-12 |
| the port drifts into feature work | scope creep | the only accepted change is parity; a feature needs its own ADR |

Stop rule (AGENTS.md): if a gate still fails after two attempts, stop and report
what was measured instead of widening the change.

## 8. Non-goals

- A Kode-written backend replacing `clang`.
- Deleting the Rust compiler.
- Porting `kode lsp`, `fmt`, `doc`, `doctor` or the package resolver here; they
  get their own plan once the compiler itself is self-hosted.
- A faster-than-Stage-0 self-hosted compiler.
- Language features that exist only for the port: every work packet is a general
  feature whose first real user happens to be the compiler.

## 9. Approval gates

| gate | question | state |
|---|---|---|
| A | Are the work packets the right ordering? | accepted; executable as `gates` |
| B | Are `kode tokens` / `kode ast` permanent documented commands? | accepted (S0 shipped) |
| C | Interpreter before backend (S4 before S5)? | accepted |
| D | "Behaviour parity, byte parity where deterministic" as Stage 2's definition? | accepted |
| E | Arena for the AST instead of `weak`, if measurements favour it? | accepted 2026-11-13 (ADR 0042): the AST is an arena; `weak` (ADR 0038) stays proposed and off the S2 path |

Changing this plan means editing this file in the same commit as the code or
probe that changes, so the script and the document never disagree.

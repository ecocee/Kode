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

Measured today: **R1** — 283 token dumps byte-identical, including the work packet
probes, the S2 parser is byte-identical on 173/173 cases and on all 55 files of
the port itself, and S3 is done (170/170 reports byte-identical). R2 is open
only on the two deferred packets: `gates` reports
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

**Slice 10 is in: optionals and `none`.** `selfhost/optional.kode` holds the rules for a
value that may be none. `E0219` covers the shapes that are not supported yet — a list of
`int?`, a map of `int?`, an optional of something that is neither a reference nor a
scalar — and `none` written without a type; `E0222` an empty list or map with no type to
take (both of them report only for a *new* name, since `x = none` on an existing optional
is a plain assignment). `E0220` is reaching inside: `.` without `?.`, a method call, an
index (in `containers.kode`), and `!` on a value that cannot be none. `E0221` is the other
direction: `??` or `?.` on a value that cannot be none, and a default of the wrong type.
`E0419` is the payload-enum rule — `s.radius` is only valid where `s` is narrowed to
`Shape.Circle` — and the walker now carries a `Narrow` stack for the then branch of an `is`
test (ADR 0039 §4.4), which is what keeps `enum_is.kode` clean; an `else` branch narrows
nothing yet, so a field read there is still reported, exactly as the oracle does. Two
typing rules came with it: `??` keeps an optional default (`env("A") ?? env("B")` is a
`string?`), and an enum variant (`Shape.Circle`, `Shape.Circle(radius: 1.0)`) types as its
enum. The gate after it is **145/169** with **self-check 45/45**; `sema.kode` stayed at
500 lines by moving the whole assignment decision to `binding.bind_target`.

**Slice 11 is in: members.** `selfhost/members.kode` is the table a `.` reads: every
field and method a class or interface declares, in declaration order, plus the prelude
classes the language provides (`Json`, `ProcessResult`, `Error`, `Request`, `Response`,
`HttpResponse`, `Router`) whose fields the runtime defines. `E0217` is a name the class
does not have (with "did you mean" from `selfhost/suggest.kode`, a Levenshtein search
that `E0201`/`E0206`/`E0225` will reuse), `E0218` is a receiver with no fields at all
(an interface, with its own hint, or a number, a list, an enum), `E0417` is a payload
variant written as a value, `E0228` is a parameter that shares a field's name, and
`E0234`/`E0235` are `implements` obligations — a generic class is skipped because it is
checked per instantiation (ADR 0040). Two cascades came with it: a failed `.` declares
nothing (`x = a.z` makes the next `x` an `E0201`), and `?.` refuses a field that is not
a reference (`E0219`, "`?.` on an int is not supported yet"). Gate: **152/169**,
self-check **47/47**.

**Slice 12 is in: results.** `members.value_type` is the typed answer `typing.kode`
could not give: a function's or method's declared result, a constructor's instance, and a
class field's type — including `?.field`, which is an optional of the field. `operands`
(operators and `print`) and `binding` ask it, so a user call's result takes part in the
arithmetic rules (`f() + "a"` is `E0203`), `print(f())` on a function with no declared
result is `E0205`, and `y = f()` binds a typed name instead of `any`. Two shapes stay
`any` on purpose: a result written in the callee's own type parameters and a generic
class's fields (ADR 0030) — the gate holds at **152/169**.

**Slice 13 is in: the report.** `check` used to print a bare `error[CODE]: message` and
a hint; it now prints what the oracle prints — the position line, the offending source
line and the carets under the span, then the hint — because `report.kode` has one
renderer for a parse error and a checker diagnostic. That needed two small things: the
tree carries its file (`Ast.file`, set where a module is loaded) and a `Diag` names it,
so one program's diagnostics can render from several files. `./scripts/selfhost.sh check`
now also prints `check text: N/169`, which counts byte-identical output rather than code
sets; it is reported, not gated, because 32 programs still differ: the families below and
the missing hints (`E0201`). It is the number that will make the last of S3 honest.

**Slice 14 is in: hints.** `E0201`, `E0206` and `E0403` now carry the oracle's hint.
An unknown name is compared with every binding in scope and every constant: a module
alias says `use name.member`, an enum name says `enum values are written like
Enum.Variant`, a near miss gets `did you mean ...?` from `suggest.kode`, and the rest say
the name is created first. An unknown callee is compared with the program's functions,
its classes and every builtin, and a module that holds code says what it may contain.
`check text` moves **79 → 86/169**; the gate holds at **152/169**.

**Slice 15 is in: `match`.** `selfhost/matching.kode` walks an arm list twice: once to
check what the scrutinee may be and what each pattern is (`E0223`, which also catches a
pattern of the wrong type, with the oracle's second wording), then to see whether the
arms cover everything (`E0224`: every variant of an enum by name, both bools, the `_`
arm an int or a string needs). A payload pattern (`Shape.Circle(r)`) names its variant
without being a value, so it is read from the tree; a pattern's bool is an attribute
without quotes, which `attr_raw` reads. `E0225` (an enum has no such variant, with a
suggestion) and `E0420` (`==` on a payload enum, whose value is a reference) came with
them, and `names.kode` now records every variant and whether it carries payloads. Gate:
**156/169**, `check text` **90/169**.

**Slice 16 is in: throws, callbacks and the web layer.** `selfhost/throws.kode` carries
`E0226` (a `throw` of anything but a string or an `Error`) and `E0227` (a `catch` block
that does not answer with the expression's type, including one that ends in nothing at
all). The list callbacks (`xs.map`, `xs.filter`) check their function in
`containers.kode`: a closure's written parameter, or a named function's signature, has to
take one item of the list and give one back (`E0242`). `selfhost/server.kode` is the web
layer: a route needs `fn name(req: Request) -> Response` (`E0233`) and `serve`/`serve_tls`
needs a `Router` or `fn name(method: string, path: string, body: string) -> string`
(`E0232`), which is also why `Router()` now types as a `Router` rather than `any`. Gate:
161/169, `check text` 95/169.

**Slice 17 is in: warnings.** A `Diag` now carries a severity, so the report can say
`warning[W0001]` and `check` can exit 0 with nothing but warnings — the first output shape
the port could not produce before. `selfhost/unused.kode` keeps the names an assignment
declared and `sema.kode` marks the ones a read reached, so `x = 1` with no later use is
`W0001` with the oracle's hint; a name starting with `_` is exempt, and a program with
errors reports none of its warnings, exactly as the oracle does. Gate: **162/169**.

**Slice 18 is in: generics, and S3 closes.** `selfhost/generics.kode` answers one
question for a call: can the type parameters it needs be read off the arguments? A free
function's `T` has to appear in a parameter type (`E0250`), a generic class's `T` in one
of its fields — that is what the constructor has (`E0422`) — and a method of a generic
class already knows its class's parameters from the receiver, so only the ones it
declares itself (`own_params`) are asked for. A generic function used as a value is
`E0251`: there is no type to instantiate it at. A class whose method declares its own
type parameters is `E0252`, reported once at the method's name. `annotations.kode` reads
`Box<int>` (as many arguments as the class declares, `E0421` for the wrong count) and
refuses a bare `Box` (`E0423`); both leave the declaration unannotated, so the value is
reported next and the name stays undeclared, as the oracle does. Because a `fn(T) -> U`
parameter type is still `any`, the inference rule stays silent whenever a parameter type
is `any`: it cannot prove the parameter is missing. `E0206` gained the oracle's span (the
callee name). Gate: **169/169** — every compile-fail fixture and every run-pass program
report the oracle's own code set; `check text` **103/169**, `self-check` **52/52**.

**Slice 19 is in: the report is exact.** The remaining report diffs were spans, one
missing file name and one measurement. `E0215` underlines the argument's name, `E0305`
the function's name, `E0419` the field and `E0218` the receiver — each the token the
oracle's caret sits under. `Alias` now carries the file an `import ... as` is written in,
so `E0408`/`E0409` (recorded while the module graph loads, before any file is walked)
name it in the `-->` line. The last family was not the port's at all: the oracle prints
`checked in 0.38 ms` on success, so `check` drops exactly that line before comparing and
counts the run-pass reports too, which it had not been counting. That found a real
`W0001` false positive — a variable holding a closure and called (`greet("Bob")`) is
read, so three run-pass programs had been warning. `check text` is now a gate as well:
**169/169** reports byte-identical, and `check` exits nonzero if a case's codes, its
report or the port's self-check differs.

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

Files: `selfhost/mir.kode` (the model and the dump), `selfhost/lower.kode` (the walk
that builds it) and `selfhost/eval.kode` (~3,500 + 1,500 lines at plan time; the walk is
one class, so it stays over AGENTS' ~500-line guideline until `Builder` is broken up).

An interpreter is not a detour: it makes the front end testable before code
generation exists and runs every run-pass program against its `.out` without a
linker.

Acceptance: `kode mir` dumps match structurally; every run-pass `.out` matches.

**The plan for slice 1 — `print` of literals.** The first slice is the smallest
program shape whose MIR is still real: a straight line of `print(...)` calls over
scalar literals, and nothing else. It puts the three pieces of S4 in place — a MIR,
a dump that matches `kode mir`, and an evaluator that produces the program's stdout —
before any of them has to deal with locals, temporaries or reference counting.

- **Files.** `selfhost/mir.kode` is the builder and the printer: an arena of
  `Function` → `Block` → `Instr` (`call`, `goto`, `return`), a typed `Value` for an
  instruction's arguments (`int`, `float`, `bool`, `str`), `build(ast)` for a
  program whose statements are `print` calls of literals, and `dump(mir)` printing
  the oracle's text — `fn <main>() -> unit {  // entry`, `  bb0:`, four-space
  instructions, `goto bb1`, `bb1: return`, and the blank line the oracle ends with.
  `selfhost/eval.kode` walks that MIR: `rt::PrintStr`/`PrintInt`/`PrintFloat`/
  `PrintBool` append to a pending line, `rt::PrintSep` appends a space and
  `rt::PrintNl` prints the line and clears it, which is exactly the separator and
  newline rule `print` has.
- **CLI.** `kode-stage1 mir <file>`, `kode-stage1 run <file>` and `kode-stage1 report
  <file>` (which lists every unsupported construct instead of stopping at the first). A
  program outside
  the slice says so on stderr and exits 2, the way an unimplemented mode does today.
- **Fixtures first.** `selfhost/spec/run/` holds a program and its `.out` for each
  shape: one string, two scalars across two lines, several arguments with a
  separator and an empty `print()`. They are written before the code, and the
  harness proves the `.out` against the oracle as well as against the port.
- **Gates.** `./scripts/selfhost.sh mir` diffs `kode mir` with `kode-stage1 mir` on
  every `selfhost/spec/run/*.kode` (byte-identical; the oracle's temporary numbering
  is free to differ, but slice 1 emits no temporaries), and `run` compares three
  things per program: the oracle's `kode run` stdout with the `.out`, the port's
  stdout with the `.out`, and so the port with the oracle. `run` compares the **exit
  status** too: a fixture whose `kode run` does not exit 0 fails, and so does a port
  whose status differs from the oracle's, because right text followed by a different
  exit code is still a wrong program. Both gates join `all` after `check`.
- **Out of scope, deliberately.** Locals (`let _0: int`), temporaries, arithmetic,
  strings built at run time (and the `release`/`retain` lines they bring), control
  flow, calls to user functions, and a program's own `exit(...)` (the gate still
  requires the port's status to equal the oracle's, which for these fixtures is 0). The next slice adds a local and
  an `add`, which is where temporary numbering and the first comparison against
  `kode mir` structure (not text) begin.
- **Risk.** Float and string formatting is the runtime's, not the port's: a float is
  printed in its shortest form and a string literal is unescaped before it is
  printed. Mitigation: the values are read with the same helpers the parser and the
  dumps already use (`float_text`, `parse_string`), and the fixtures cover both.

**Slice 1 is in.** `selfhost/mir.kode` builds and prints the MIR and
`selfhost/eval.kode` runs it; `kode-stage1 mir` and `kode-stage1 run` are wired into
`main.kode`, and a program outside the slice says why on stderr and exits 2.
`./scripts/selfhost.sh mir` reports **3/3** dumps byte-identical and `run` **3/3**
cases with byte-identical output and the oracle's exit code, with each `.out` proven
against the oracle's `kode run` first. Both joined `all`. The one thing the plan let differ — temporary numbering —
does not arise yet, so the dump comparison starts out exact, and the next slice (a
local and an `add`) is where it becomes a structural comparison.

**The plan for slice 2 — a local and an `add`.** The shape it adds is
`x = <int expression>` and `print(<int expression>)`, where an expression is a
literal, a name or a `+` of two of those. That is the first slice with temporaries,
and the oracle's rules for them are exact, so the dump can stay byte-identical:

- **Allocation.** A variable and every intermediate `+` get one temporary, numbered in
  the order the program is walked; a string or bool printed directly gets none. The
  `let` lines come before `bb0:` (`    let _0: int  // x`, the variable's own name as
  a comment; a bare `    let _1: int` for an intermediate), then `bb0` opens with one
  `_N = 0` per temporary in index order, then the code.
- **Lowering.** A `+` lowers its left operand, then its right, then emits
  `_N = add a, b` where each operand prints as itself (`1`) or as its temporary
  (`_0`). Nested `1 + 2 + 3` is two `add`s, left one first. `print` follows with
  `call rt::PrintInt(_N)` and `rt::PrintNl`.
- **Files.** `selfhost/mir.kode` grows an `Operand` (a literal or a temporary), a
  `dest` on `Instr`, and a lowering state (name → temporary, next index); the `let`
  block is part of the function's dump. `selfhost/eval.kode` keeps one `int` per
  temporary and resolves `add` and the print primitives against it.
- **Fixtures first.** Four more programs in `selfhost/spec/run/`: a local,`1 + 2`,
  two locals summed, and `1 + 2 + 3`, each with its `.out`.
- **Gates.** Unchanged — `mir` byte-identical, `run` output and exit code. If the
  port's numbering ever drifts from the oracle's, the gate says so rather than
  hiding it, and the plan's allowance (numbering may differ) becomes a normaliser.
- **Out of scope.** Other operators, string temporaries (and their `retain`/
  `release`), typed annotations, reassignment (`x = x + 1` on an existing local),
  blocks, branches and calls to user functions.

**The plan for slice 3 — reassignment and the rest of the integer operators.** The
shapes it adds are `x = <int expression>` where `x` already exists, and `- * / %` on
ints. Probing the oracle settled two rules the slice must follow:

- **A declaration lowers its value first.** `x = 1 + 2` allocates the `add`'s
  temporary before the variable's (`_0 = add 1, 2`, then `_1 = _0`, and `let _1: int
  // x`), which slice 2 only got right by accident (`x = 41` has no value temporary).
- **Reassignment writes the existing slot, after the value.** `x = x + 1` is
  `_1 = add _0, 1`, then `_0 = _1` — no new temporary, so `let` has no second entry.

Operators are the oracle's mnemonics: `sub`, `mul`, `div`, `rem`, and `div` truncates
toward zero (`0 - 7 / 2` is `-3`), which is what Kode's own `/` does. Unary minus is
out of scope; `0 - x` is the shape that exists.

- **Files.** `selfhost/mir.kode`: `assign` lowers the value before it allocates or
  writes anything, `expr` maps the five operators to their instruction, and
  `instr_text` prints any `dest` instruction as `_N = <op> a, b`. `selfhost/eval.kode`
  gains `sub`/`mul`/`div`/`rem`.
- **Fixtures first.** Five more in `selfhost/spec/run/`: a declaration whose value is
  a sum, a reassignment, the four new operators, a sum of two locals multiplied, and
  a truncating division.
- **Gates.** Unchanged: `mir` byte-identical, `run` output and exit code.

**The plan for slice 4 — strings built at run time.** A string is a reference, so the
slice adds the retain/release lines the oracle emits. Probing `kode mir` pinned them:

- A string temporary is a `ref`: `    let _0: ref  // x`, zeroed to `_0 = null`.
- Storing into a string *variable* releases the old value first, always — including
  the first store over the null: `release _0`, then `_0 = "a"`. A reassignment is the
  same: `release _0`, `_0 = "b"`.
- An *intermediate* reference is released at its last use: `call rt::PrintStr(_1)`
  then `release _1`. A variable is not released after a print; every named string
  variable is released once in `bb1`, in index order, before `return`.
- `x + y` on strings is `_N = call rt::StrConcat(x, y)`; a literal argument is
  borrowed and never released, and a result stored into a variable is moved, not
  released (`_1 = call rt::StrConcat(_0, "b")`, `release _2`, `_2 = _1`).

- **Files.** `selfhost/mir.kode`: `Temp.kind` becomes real (`int`/`ref`), a `release`
  instruction and a `dest`-carrying `call`, `x + y` dispatches on the operand's type,
  and `assign` emits the release-before-store it learned. `selfhost/eval.kode` keeps a
  string per ref temporary.
- **Fixtures first.** Five more in `selfhost/spec/run/`: a string variable, two string
  variables concatenated, a variable plus a literal, a reassignment with a print in
  between, and `print("a" + "b")`.
- **Gates.** Unchanged: `mir` byte-identical, `run` output and exit code.
- **An interpolation is a value (slice 10).** `"a${y}b"` was only a `print` argument;
  now it is an operand, so a slot, a call argument and a `return` take one. The
  accumulator is what the caller owns: a single hole is the variable *retained* for it
  (`retain _0`, then `_1 = _0` — never a second retain in the store), two pieces are an
  intermediate the walk built. The oracle pins both: a call site releases an
  intermediate argument and a retained interpolation after the call, and a named
  variable argument still lives in its own slot. Fixtures: `interp_assign`,
  `interp_assign_sole`, `interp_assign_int`, `interp_arg`, `interp_return`; the corpus
  drops one construct (`errors.kode`'s `Interp`), 318 → 317.
- **The owning interpreter.** Ignoring `retain`/`release` would make the interpreter
  blind to the one thing the MIR exists for, so it keeps shadow counts instead. A
  temporary that holds a run-time string holds an *object*: `rt::StrConcat` and
  `rt::IntToStr` create one (count 1), a storage or return moves the object into the
  destination slot without touching the count, `retain` adds one and `release` takes
  one. A release with nothing to take, a use of an object whose count reached zero, and
  a count still standing when `<main>` returns are all failures: the run exits 1 with
  `kode-stage1: ownership: ...` on stderr. The `own` gate runs four stress cases
  (overwrite, a branch that reassigns, a reference that crosses two frames, a loop that
  reassigns and interpolates) and then proves the check can fail by building a mutant
  port whose every `retain` became a `release`: it must be caught, or "no violation" only
  means the check never ran.
- **Out of scope.** Interpolation (`"hi ${name}"` also releases its literal head —
  a different rule, its own slice), string comparison, `map`/`filter`, functions.

**The plan for slice 5 — interpolation.** `"hi ${name}"` is a `Interp` node whose kids
are `Lit text="hi "` and `Hole` (one kid, the expression). Probing `kode mir` shows it
is a concat chain with its own ownership rule — the `release "literal"` line is a
literal *head* being released as the accumulator:

- The pieces are the non-empty `Lit`s and the holes, in order. The head becomes the
  accumulator and is made owned: a literal head needs nothing (it is released after
  its first concat — `_2 = call rt::StrConcat("hi ", _0)`, `release "hi "`), while a
  head that is a variable is retained first (`retain _0`) and released after.
- Each following piece is `_t = call rt::StrConcat(acc, piece)`; then the old
  accumulator is released, then the piece if it is owned. A literal piece that is not
  the head is borrowed and never released (`", "` and `"!"` in
  `"hi ${name}, ${n}!"`); an int hole is owned, because it goes through
  `_t = call rt::IntToStr(_n)` first and is released right after the concat.
- A lone hole is no concat at all: `retain _0`, then the print, then `release _0`.
- The result is an owned reference, so the print that reads it releases it after.

- **Files.** `selfhost/mir.kode`: a `Piece` (operand and whether it is owned), the
  `interp`/`piece`/`hole` lowering, `retain` as an instruction, an `IntToStr` call with
  a destination, and interpolation as a `print` argument. `selfhost/eval.kode`:
  `retain` is a no-op like `release`, and `rt::IntToStr` renders an int.
- **Fixtures first.** Five more in `selfhost/spec/run/`: text with a string hole, text
  with an int hole, two holes with no text, a lone hole, and `"hi ${name}, ${n}!"`.
- **Gates.** Unchanged: `mir` byte-identical, `run` output and exit code.
- **Out of scope.** Interpolation as a value (`x = "hi ${n}"`), a hole that is not a
  name (an expression, a call, a nested interpolation), and interpolating floats and
  bools (their `FloatToStr`/`BoolToStr` calls are the same shape, another slice).

**The plan for slice 6 — comparisons and `if`.** The first slice with more than one
block. Probing `kode mir` gave the layout exactly:

- The function is `bb0` first, then `bb1` — the exit block with its releases and
  `return`, wherever it is branched to — and then the blocks the walk created, in
  creation order. An `if` creates its then-block, then the join, and then the else
  block when there is one (`branch _1, bb2, bb4`, `bb2` then, `bb3` join with the code
  after the `if`, `bb4` the else); a then-block and an else-block each end with
  `goto <join>`, and the block the walk ends in gets `goto bb1`.
- A condition is `_1 = lt _0, 2` into a `bool` temporary (`let _1: bool`, zeroed to
  `_1 = false`), then `branch _1, <then>, <else>`. Without an `else`, the else target is
  the join.
- Comparisons are `lt`, `le`, `gt`, `ge`, `eq`, `ne` on ints, and a `bool` prints
  through `rt::PrintBool`.

- **Files.** `selfhost/mir.kode`: blocks become a list the walk appends to (entry and
  exit created up front), `branch` carries its two labels, `if` lowers its blocks and
  joins them, comparisons allocate a `bool` temporary, and the zero block initialises a
  `bool` to `false`. `selfhost/eval.kode`: a `State` holding the pending line and the
  int, string and bool per temporary, and a program counter that follows `goto`,
  `branch` and `return` instead of walking one block — the first real control flow in
  the interpreter.
- **Fixtures first.** Five more in `selfhost/spec/run/`: the six comparisons printed,
  an `if` that runs, an `if` that is skipped, an `if`/`else`, and an `if` that assigns.
- **Gates.** Unchanged: `mir` byte-identical, `run` output and exit code.
- **Out of scope.** `while`/`for`, `and`/`or`/`not`, bool variables, string equality.

**The plan for slice 7 — `while`, `break` and `continue`.** Probing `kode mir` gave the
loop layout and one new rule about terminated blocks:

- A loop creates its condition block, its body block and its exit block in that order,
  and the block before it ends with `goto <cond>`. The condition block computes the
  condition (a `true` literal is passed straight to `branch true, bb3, bb4`) and
  branches to the body or the exit; the body ends with `goto <cond>`, and the code
  after the loop is the exit block. A nested loop is created inside the outer body, in
  the same order (`bb5` cond, `bb6` body, `bb7` exit).
- `continue` jumps to the condition block and `break` to the exit block.
- **A terminated block is never written again.** After `break`/`continue` ends a
  block, the next instruction — even the `goto <join>` an `if` emits for its arm —
  goes into a *new* block, which is why an `if` whose arm ends in `continue` produces a
  join and then an extra block that only `goto`s it (`bb5: goto bb2`, `bb6` the join,
  `bb7: goto bb6`).

- **Files.** `selfhost/mir.kode`: a single `emit` that starts a new block when the
  current one is terminated (every instruction goes through it), a stack of break and
  continue targets, `while_stmt`, and `break`/`continue` as gotos. `selfhost/eval.kode`
  needs nothing: its program counter already follows `goto` and `branch`.
- **Fixtures first.** Five more in `selfhost/spec/run/`: a counting loop, `while true`
  with a `break`, a loop with both `continue` and `break`, nested loops, and a loop
  whose body never runs.
- **Gates.** Unchanged: `mir` byte-identical, `run` output and exit code.
- **Out of scope.** `for` (both forms), `else` on a loop, and `break`/`continue` inside
  a `match` arm.

**Slice 7 is in.** Every instruction now goes through one `emit`, which starts a fresh
block when the current one already ends in a terminator — the rule that explains the
oracle's extra `goto`-only blocks after an arm that `break`s or `continue`s.
`while_stmt` creates its condition, body and exit blocks in that order, `break` and
`continue` jump to the exit and the condition through a stack of loop targets, and a
nested loop is created inside the outer body. `selfhost/eval.kode` needed nothing: its
program counter already followed `goto` and `branch`. Five fixtures came first in
`selfhost/spec/run/` (a counting loop, `while true` with `break`, `continue` with
`break`, nested loops, a body that never runs): the gates report **32/32** dumps
byte-identical and **32/32** runs with the oracle's output and exit code. `for` (both
forms), `else` on a loop and `break`/`continue` inside a `match` arm are still out.

Three cases the slice introduced had no fixture, and probing them settled where each
belongs. A loop inside an `if` arm is a run case (`while_in_if`, dump identical, prints
`0 1 2 3`). `break`/`continue` outside a loop and a `while` on a non-bool are
**checker** errors in the oracle (`E0314` for the first two, `E0209` for the third) and
never reach MIR, so they belong in `tests/compile-fail`, not `spec/run`; `continue`
outside a loop was the one missing (`e0314_continue_outside`), and the `check text` gate
now proves the port's report for it byte for byte. A `mir`-mode refusal cannot be a
`spec/run` fixture at all: the oracle prints its error on stderr, the port prints its
own stop reason, and `check_mir` compares stdout. Gates after these pins: parse
**173/173**, check **170/170** with **170/170** reports, mir **33/33**, run **33/33**.

**The walk is five files, not one.** `selfhost/lower_core.kode` holds the `Builder` that
every step shares (tree, function, blocks, names, refusals); `lower_stmt.kode` and
`lower_expr.kode` are the two walks; `ops.kode` is the operator tables; `lower.kode` is the
program — declared functions, `<main>` and the assembly of each. They form a one-way graph
(`lower` → `lower_stmt` → `lower_expr` → `lower_core`, plus `ops`), so no import cycle, and
every file is under 400 lines. It was a pure move: the gates did not move.

**Slice 9 is in — calls, and the corpus moves for the first time.** `name(args)` lowers
to `_N = call name(...)` with the arguments in order, the destination sized from the
callee's return kind (`unit` callees get no destination), and the port's own interpreter
gives each call a frame: arguments into parameters, the body to its `return`, the value
back to the caller. Recursion is just another frame (`fact(5)` is a fixture). The return
kinds are collected before any body is walked, because a recursive or forward call needs
its size while its callee's body has not been visited yet. `functions` and
`early_return_strings` now match the oracle dump for dump, output for output — the corpus
is **4/58**, and mir/run are **47/47**.

**Slice 8 is in — declarations and returns.** A `fn` is lowered as
its own function: parameters become `_0`, `_1`, … (declared by the header, so no `let` and
no zero init), the return temporary is allocated at the first `return` — after its operand,
which is the oracle's numbering — `bb1` is the return block and releases the named
references before it returns (`release _5` then `return _3`), a borrowed reference is
retained on the way into the return slot (`retain _1` then `_3 = _1`), and a block-body
function that returns something ends in `unreachable` — in the fallthrough block when the
walk did not end terminated, in a fresh block when it did, and never for the `=> expr`
sugar. Functions are dumped in declaration order, `<main>` last. Six run fixtures came
first (`fn_void_param`, `fn_return_int`, `fn_return_string`, `fn_sugar`, `fn_early_return`,
`retain_on_assign`), and the report fixtures show bodies being walked for the first time:
`fn_body_blocker` now reports the list inside the function instead of `statement \`Fn\``.
Gates: mir **39/39**, run **39/39**, report **5/5**, corpus 2/58 with **304** constructs in
56 programs. Calls are the next half of the slice.

**A divergence the fixtures had missed.** `b = a` between two string variables emitted
`release _1; _1 = _0` where the oracle emits `retain _0; release _1; _1 = _0`: a store
into a reference slot retains a borrowed value first. None of the thirty-three fixtures
assigned one string to another, so the byte-identical dump gate was green over a real
bug; `retain_on_assign` now pins it, and `borrowed()` is shared with the return walk.

**The walk reports every blocker, not just the first.** `kode-stage1 report <file>` walks
the whole program and prints each construct it cannot lower, once each, in the order it
meets them. `mir` and `run` still stop at the first refusal and exit 2, so the
byte-identical gates did not move. Collecting takes two pieces in `selfhost/lower.kode`:
`stop` records the reason instead of ending the walk, and every refusal hands back a
**poison** operand (`kind: "?"`) so the caller does not report the same failure again as
a wrong type (`b` holding a `any`, `==` on a `any`). A refused declaration also registers
its name, so reading it afterwards counts as a consequence rather than a second finding —
which also keeps the ledger stable when a variable is renamed. Three fixtures in
`selfhost/spec/report/` pin that behaviour: several blockers in one program, a loop body
that keeps being walked after a refusal, and a refused declaration that is then read.

**The corpus is the gate, and now it says how much is left.** `./scripts/selfhost.sh corpus`
runs the `mir` and `run` comparisons over `tests/run-pass` rather than the hand-written
fixtures. Today **2 of 58** programs lower — `hello` and `nesting_ok`, both byte-identical
and running — and the other 54 need **318 constructs** between them. The number keeps
growing as the walk sees more: 296 at first, 304 when function bodies became visible,
318 when call arguments did. That is the point of a generated ledger — coverage is
measured, never assumed. These
are not first-blocker floors any more: `report` walks past a refusal, so each program's
list is complete for the shapes the walk knows how to name.

| what is missing | constructs | programs |
|---|---|---|
| members and fields: `MethodCall`, `Member` | 25, 15 | 25, 15 |
| collections: `List`, `Map`, `Index` | 18, 5, 15 | 18, 5, 15 |
| loops beyond `while`: `statement \`For\``, `ForIn` | 13, 9 | 13, 9 |
| targets: an assignment to something other than a name | 9 | 9 |
| errors: `Catch`, `Try` | 8, 5 | 8, 5 |
| classes and enums: `Class`, `Enum` | 7, 5 | 7, 5 |
| optionals: `None`, `Coalesce`, `Unwrap` | 5, 5, 2 | 5, 5, 2 |
| `Unary`, `Closure`, `Spawn`, `Import`, `Match`, `Const`, `Interface` and the operators `&`/`|`/`^`/`<<`/`>>`/`and`/`or`/`in` | 20 | 20 |

The ledger is the per-program detail — `selfhost/spec/corpus-out-of-scope.txt`, one
`<program>\t<construct>` line per finding, generated from the port by
`./scripts/selfhost.sh ledger` and never edited by hand (`corpus` regenerates it and fails
when the file differs, so a program cannot quietly leave or join the list). The table above
regenerates with:

```sh
awk -F'\t' 'NF == 2 {print $2}' selfhost/spec/corpus-out-of-scope.txt | sort | uniq -c | sort -rn
```

So the next S4 slices, in the order the corpus asks for them: **members, method calls and
indexing** (25 + 24 + 15 programs — the largest remaining block, and the gate to anything
that holds a class instance), then list and map literals with `for`, then classes and
enums with `match`, then bool/float locals, optionals and the remaining operators. The
port's own sources agree on that order: `Import` 48, `Class` 36, `MethodCall` 35,
`ForIn` 25, `Member` 24 are its top blockers. That is a different order from "whatever MIR shape comes next", and it is the
one that moves this number.

**Known divergences — what green does not cover.** The port refuses shapes it cannot
lower with its own message and exit 2, where the oracle has already rejected the program
or dumped it: no gate compares that wording, and `check_mir` compares stdout only.
`release`/`retain` are no-ops in the interpreter — the dumps prove they are emitted, not
what they do. The dump comparison is byte-identical today, but the plan still allows the
oracle's temporary numbering to differ, which would need a structural comparison. WP-07
(`weak`), WP-08b (fixed-width ints) and WP-12 (streaming `exec`) are deferred on purpose.
The `report` vocabulary is the port's own: it is not compared with the oracle, which has
no equivalent list, so a wording change there is a ledger change and a review comment,
not a gate failure.

**Slice 6 is in.** `selfhost/mir.kode` keeps its blocks in a list: `bb0`, then the exit
`bb1`, then whatever the walk created, in creation order — exactly the oracle's dump.
`if` creates its then-block, its join and (when there is one) its else-block, branches
with `branch _1, bb2, bb4`, and both arms end in `goto <join>`; the block the walk ends
in gets `goto bb1`. Comparisons lower to `lt`/`le`/`gt`/`ge`/`eq`/`ne` on a `bool`
temporary, which `PrintBool` prints and which starts at `false`. `selfhost/eval.kode`
stopped walking one block: it holds a `State` (pending line, `int`/`string`/`bool` per
temporary) and a program counter that follows `goto`, `branch` and `return` — the
`goto`/`return` placeholders are gone. Five fixtures came first in
`selfhost/spec/run/` (the six comparisons, an `if` that runs, one that is skipped, an
`if`/`else`, an `if` that assigns): the gates report **27/27** dumps byte-identical and
**27/27** runs with the oracle's output and exit code. `while`, `and`/`or`/`not`, bool
variables, string equality and nested `if`s are still out, and `selfhost/mir.kode` is
now past 700 lines, so splitting it by idea (model, lowering, dump) is the next tidy-up.

**Slice 5 is in.** `selfhost/mir.kode` lowers interpolation as a `print` argument: the
pieces are lowered where they are concatenated, the head becomes the accumulator (a
literal head released after its first concat, a borrowed variable head retained), and
every further piece is `_t = call rt::StrConcat(acc, piece)` with the old accumulator
released and an owned int hole released too — `release "hi "` is exactly that literal
head. A lone hole is only `retain _0`, and the print releases the value it read.
`selfhost/eval.kode` adds `rt::IntToStr` and treats `retain` like `release`, as a no-op.
Five fixtures came first in `selfhost/spec/run/`: text with a string hole, text with an
int hole, two holes with no text, a lone hole, and `"hi ${name}, ${n}!"`. The gates
report **22/22** dumps byte-identical and **22/22** runs with the oracle's output and
exit code. Still out: interpolation as a value, holes that are not names, and float or
bool holes.

**Slice 4 is in.** `selfhost/mir.kode` now types its temporaries (`int` or `ref`), so
a string variable is `let _0: ref`, zeroed to `null`, released before every store, and
released once in `bb1`; an intermediate reference is released right after its last use
(`call rt::PrintStr(_2)`, then `release _2`), and `+` on strings is
`_N = call rt::StrConcat(a, b)` with borrowed literal arguments and a moved result.
`selfhost/eval.kode` keeps a string per reference temporary and treats `release` as a
no-op, because the interpreter has no counts to drop — the gate still checks the
instruction is emitted, byte for byte. Five fixtures came first in
`selfhost/spec/run/` (a string variable, two concatenated, a variable plus a literal, a
reassignment with prints in between, and `print("a" + "b")`): the gates report
**17/17** dumps byte-identical and **17/17** runs with the oracle's output and exit
code. Interpolation is the next slice, with its own literal-head release rule.

**Slice 3 is in.** `selfhost/mir.kode` lowers a declaration by lowering its value
first (`x = 1 + 2` is `_0 = add 1, 2` then `_1 = _0`, with `let _1: int  // x`), a
reassignment into the existing slot (`x = x + 1` is `_1 = add _0, 1` then `_0 = _1`,
no new temporary), and all five integer operators as the oracle's mnemonics (`add`,
`sub`, `mul`, `div`, `rem`). `selfhost/eval.kode` runs them, and `div`/`rem` truncate
toward zero like the language does (`0 - 7 / 2` is `-3`). Five fixtures came first in
`selfhost/spec/run/` (a sum as a declaration's value, a reassignment, the four new
operators, a product of two locals, and a truncating division): the gates report
**12/12** dumps byte-identical and **12/12** runs with the oracle's output and exit
code. Unary minus, comparison, `and`/`or` and the string operators are still out.

**Slice 2 is in.** `selfhost/mir.kode` now lowers a local (`x = 41`), a name read and
`+`, with the oracle's temporaries exactly: one per variable and per `add`, numbered
in walk order, `    let _N: int  // x` for a variable and a bare `let` for an
intermediate, one `_N = 0` per temporary at the top of `bb0`, and `_N = add a, b`.
`selfhost/eval.kode` keeps one `int` per temporary and resolves assignment, `add` and
`rt::PrintInt` against that store. Fixtures first: four more programs in
`selfhost/spec/run/` (a local, `1 + 2`, two locals summed, `1 + 2 + 3`), so the gates
report **7/7** dumps byte-identical and **7/7** runs with the oracle's output and exit
code. The numbering did not drift, so the dump comparison is still exact rather than
normalised. Next: reassignment and the other arithmetic operators, then strings built
at run time with their `retain`/`release`.

### S5 — native code generation (branch `selfhost/s5-backend`)

Files: `selfhost/codegen.kode`, `selfhost/driver.kode` (~2,000 lines). Emits LLVM
IR text, writes it to a temp file, calls `clang`/`cc` through `exec`, and links
the existing runtime archive.

Acceptance: every run-pass program links and prints its `.out`; binary sizes
recorded, not asserted to the byte. This is the only stage that can produce a
wrong binary that still runs, so output parity plus `KODE_LEAK_CHECK=1` is the
gate.

### S6 — Stage 1 compiles the corpus (branch `selfhost/s6-stage1`)

Acceptance: Stage 1, running as a compiled Kode program, passes `stage1` and
`corpus`: 58 run-pass outputs, 112 compile-fail codes, leak-clean, `kode fmt`
unchanged. Rust `cargo test --workspace` stays green throughout. (The corpus was
46 programs and 98 codes when this plan was written; it moves, so read the counts
from the gate, not from here.)

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

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

Measured today: **R1** — 216 token dumps byte-identical, including the twelve work
packet probes. R2 is open: `gates` reports 9 met, 4 blocked.

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
| **WP-09** | `StringBuilder` | `wp09-string-builder` | E0206 |
| **WP-10** | a `stderr` stream | `wp10-stderr` | E0206 |
| **WP-11** | the AST is an arena (ADR 0042) | `wp11-arena` | **met** |
| **WP-12** | streaming process output | `exec` on 10 MB without buffering | open |

Order: WP-03 → WP-04 → WP-05 (one representation, then the two ways to take it
apart), then WP-02, then WP-06 → WP-10 as the stage that needs them arrives.
WP-11 is decided before the AST is written rather than after (ADR 0042); that decision
takes WP-07 off the S2 path, so its probe stays red on purpose as the open language
feature rather than a port blocker. WP-08b is red for the same reason: `u8`/`i32`
are numeric types of their own and reading bytes does not need them (ADR 0043 §4).

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

`selfhost/{tokens,keywords,bytes,strings,lexer,dump,main}.kode` (~1,250 lines,
each file under 500) plus `selfhost/spec/lexer_cases.kode`.

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
`selfhost/spec/parse` (3/3 trees byte-identical, `expr_count` included).
Not yet: every other statement and expression, `E01xx` parity, and the `int?`
parent edges ADR 0042 allows. Those are the rest of S2's slices, and each adds
cases to `selfhost/spec/parse` rather than rewriting this one.

Risk: this is where a missing language feature becomes duplication. Stop and
report rather than invent a workaround.

### S3 — checker (branch `selfhost/s3-checker`)

Files: `selfhost/sema.kode`, `selfhost/types.kode` (~4,500 lines).

Acceptance: `kode check --format=json` identical to Stage 0 for the 98
compile-fail programs (code, message, span, hint, order) and clean for run-pass.

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

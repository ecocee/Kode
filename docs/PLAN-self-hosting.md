# Plan: self-hosting Kode

Status: **proposed** — this plan is the sequence and the gates, not a promise of a
date. Each stage is one branch and one commit set, so `git log` reads like the
plan, exactly as `docs/PLAN-v1.md` does for the release work.

Companion documents:
`docs/self-hosting.md` (what exists today),
`docs/PLAN-v1.md` (what ships in v1.0),
`docs/STATUS.md` (measured state).

## 1. Definition of done

A three-stage bootstrap, the same shape Go and Rust use:

| stage | built by | what it is |
|---|---|---|
| **Stage 0** | hand | the Rust compiler, today's `kode` |
| **Stage 1** | Stage 0 | a `kode` compiler written in Kode that compiles the whole corpus |
| **Stage 2** | Stage 1 | Stage 1 compiled by itself; must behave identically |

Self-hosting is done when:

1. `scripts/selfhost.sh` runs Stage 0 → Stage 1 → Stage 2 with no manual steps.
2. Every program in `tests/run-pass/` produces its `.out` under both Stage 1 and
   Stage 2, with `KODE_LEAK_CHECK=1`.
3. Every program in `tests/compile-fail/` produces its documented code and the
   same `--format=json` diagnostics under Stage 1 as under Stage 0.
4. Stage 1 and Stage 2 agree on the whole corpus: identical diagnostics,
   identical program output, and identical objects where the toolchain is
   deterministic (see §5).
5. The numbers are in `benchmarks/report.md`: compile time for the corpus, peak
   RSS, and the size of the Stage 1 compiler.

**Not** part of done: a backend written in Kode (Stage 1 emits LLVM IR text and
calls `clang` through `exec`, as `docs/self-hosting.md` already argues), removing
the Rust compiler, or matching Stage 0's speed.

## 2. Where we are (measured)

| item | value |
|---|---|
| compiler, `src` only | **27,606 lines** across 8 crates (31,081 with tests) |
| largest crates | `kode-rt` 8,466, `kode-sema` 5,386, `kode-mir` 4,536, `kode-cli` 4,591 |
| `tests/run-pass` | 46 programs with `.out` |
| `tests/compile-fail` | 98 programs with `.err` |
| standard library | 98 functions |
| error codes | 91 `E0xxx` + 1 warning, each with a hint and a `kode explain` page |
| runtime ABI | `kode_rt_abi_0004` |

Language surface that already exists and that the port depends on: classes,
interfaces with dynamic dispatch, closures and function values, generic
functions with monomorphization (ADR 0030), tag enums with `match`, lists,
string-keyed maps, `try`/`catch`/`finally`, `T?` for heap types, `spawn`/`await`,
`Channel<T>`, files, `exec` with `ProcessResult`, sorting, and `test` blocks.

## 3. Gate 0: the language must be able to express a compiler

Every item below blocks a specific stage. The port must not start a stage whose
prerequisites are missing, and the prerequisites must not be written *for* the
port as a special case — they are general language features from
`docs/PLAN-v1.md` M9 whose first real user happens to be the compiler.

| # | prerequisite | blocks | why the port cannot work around it | acceptance |
|---|---|---|---|---|
| 9.9 | generic classes, generic methods, constraints | S2 (parser), S3 (checker) | token buffers, AST child lists, symbol tables and diagnostic lists are all `Vec<T>`/`Map<K,V>`; without them every structure is duplicated per element type | `tests/run-pass/generic_classes.kode`; a generic container used by at least three different element types |
| 9.7 | enum payloads, `is`, safe downcast | S2 (AST), S3 | the AST is a tagged union; tag-only enums force N parallel fields plus a tag, which is the duplication the mission forbids | `tests/run-pass/enum_payloads.kode`, `is` on a payload, compile-fail for an unsafe cast |
| 9.6 | byte buffers, `u8`/`i32`, `StringBuilder` | S1 (lexer), S5 (IR text) | a lexer must look at bytes, not 64-bit ints; building IR text by `+` is quadratic | `tests/run-pass/bytes.kode`, `tests/run-pass/string_builder.kode`, and a measured linear build of a 1 MB string |
| 9.5 | `weak` references (ADR 0038) — **or** an arena | S2, S4 | an AST with parent pointers leaks every node under plain RC; either the back-edge is weak, or the whole tree lives in an arena freed at once | `benchmarks/leaks/run.sh` unchanged for strong cycles, plus a `weak` program at 0 leaks |
| 9.8 | optional scalars as fields and parameters | S2, S3 | spans, class ids and "no parent" are `int?` fields; today `int?` works only as a local | `tests/run-pass/int_optional_fields.kode` |
| — | process streaming | S5, S7 | compiling 144 programs must not buffer every compiler's output in memory | a `ProcessResult`-style streaming API with a test that reads 10 MB without buffering it |

Gate 0 is closed when all six rows are green in `docs/STATUS.md`. 9.5 and 9.6
are the long poles; 9.9 is the one that blocks the most code.

9.7 and 9.9 — the two rows S2 lives or dies on — now have designs (ADR 0039 for
enum payloads, ADR 0040 for generic classes) and an ordered slice list each. S2
opens when their first slices are green; generic classes go first, because they
change the checker only while payload enums change a value representation.

Measured correction from S1: byte *buffers* were not needed for the lexer.
`s.size()`, `s.byte_at(i)` and `s.slice(a, b)` read bytes and the token dump only
concatenates short lines, so the port shipped without 9.6. What 9.6 still blocks
is S5: building IR text by `+`, and `u8`/`i32` for object encoding.

## 4. Stages

### S0 — bootstrap harness (branch `selfhost/s0-harness`)

Status: **done**. `kode tokens` and `kode ast` print the stable dump below
(`crates/kode-syntax/src/dump.rs`), `scripts/selfhost.sh stage0` is green on a
clean checkout, and `stage1`/`stage2`/`compare` refuse to report success when
the program or the dump they would compare against does not exist yet.

Deliverable: `scripts/selfhost.sh`, plus the two debug dumps the later stages
compare against.

- `kode tokens <file>` and `kode ast <file>` — new Stage 0 commands that print a
  stable, diffable dump (`--format=json` included). They are the oracle for S1
  and S2, so they land with tests of their own, and `docs/cheatsheet.md` gains
  two rows.
- `scripts/selfhost.sh stage0|stage1|stage2|compare`:
  builds Stage 0, compiles `selfhost/main.kode` with it, compiles the corpus with
  the result, diffs against the oracles in §5, and prints a summary.
- A self-check: the harness compares Stage 0 against itself on the corpus first,
  so a broken comparison cannot masquerade as a working port. That run also
  records the baseline compile time and RSS.

Acceptance: `./scripts/selfhost.sh stage0` is green on a clean checkout;
`compare` refuses to claim success when Stage 1 does not exist yet.

Risk: the dumps become a compatibility surface. Mitigation: they are documented
as debug output in `docs/cheatsheet.md`, and `--format=json` stability rules
(AGENTS.md 9) apply to them from day one.

### S1 — lexer (branch `selfhost/s1-lexer`)

Status: **done**. `selfhost/{tokens,keywords,bytes,strings,lexer,dump,main}.kode`
(about 1,250 lines across seven files, each under 500) and
`selfhost/spec/lexer_cases.kode`.

Acceptance met: `./scripts/selfhost.sh stage1` builds the port with Stage 0 and
reports **184 of 184 token dumps byte-identical** — token text, byte spans,
line/column, escaped characters, and the exit status of the programs that must
fail to lex. The run is recorded in `target/selfhost/stage1.txt`: 2.27 s and
3,824 KB peak RSS for token dumps only, which is not comparable with Stage 0's
3.51 s because that run dumps trees as well.

Two findings carried into S2:

- `.len()` counts characters; `.size()` counts bytes. A compiler offset is a byte
  offset, so the port uses `.size()` everywhere — `.len()` there silently
  truncates any file containing a multibyte character. This is the measured cost
  of the 9.6 row: reading bytes needed nothing new (`byte_at`, `slice`, `size`),
  while a `StringBuilder` is still needed to build IR text in S5.
- There is no stderr stream. `main.kode` reports diagnostics through
  `write_file("/dev/stderr", ...)`, which is Unix-only; a `stderr(text)` builtin
  is the fix and is recorded in `docs/self-hosting.md`.

`ast` and later commands exit 2 with a "not implemented yet" message rather than
pretending, so a green `stage1` cannot be mistaken for a parser.

Risk: Unicode and escapes. Mitigation: `selfhost/spec/lexer_cases.kode` is part of
the corpus the harness dumps — every escape, a 4-byte character, a hole with a
brace inside a string, and the three-character operator runs (`<<=`, `>>=`, `..=`).

### S2 — parser and spans (branch `selfhost/s2-parser`)

Files: `selfhost/ast.kode`, `selfhost/parser.kode` (~2,500 lines estimated).

Acceptance: identical `kode ast` dumps on the same corpus, including the
E01xx syntax errors (message, span, hint). The AST is the first place the arena
decision is exercised: a `Program` owns its nodes and is freed as one unit.

Risk: this is where 9.7 and 9.9 are proven, and where a missing language feature
shows up as duplication. If the AST needs `interface`-typed collections that do
not exist yet, stop and report rather than inventing a workaround.

### S3 — checker (branch `selfhost/s3-checker`)

Files: `selfhost/sema.kode`, `selfhost/types.kode` (~4,500 lines estimated).

Acceptance: `kode check --format=json` output identical to Stage 0 for the 98
compile-fail programs (code, message, span, hint, in order) and for the 46
run-pass programs (no diagnostics). This is the strongest parity gate in the
plan, and the one that makes the port trustworthy.

Risk: the checker is the largest single stage, and error-message wording is
tightly coupled to the Rust code. Mitigation: port by diagnostic family (one
E0xxx group per commit), with the JSON diff as the test.

### S4 — MIR and an interpreter (branch `selfhost/s4-interpreter`)

Files: `selfhost/mir.kode`, `selfhost/eval.kode` (~3,500 + 1,500 lines estimated).

An interpreter is not a detour: it makes the whole front end testable long
before code generation exists (the order `docs/self-hosting.md` already
prescribes), and it runs the 46 run-pass programs against their `.out` files
without a linker.

Acceptance: `kode mir` dumps match Stage 0 structurally, and running every
run-pass program through the interpreter prints its `.out` exactly.

Risk: MIR is an internal contract with no stability promise; the dump format is
ours to define, but it must be stable within the port.

### S5 — native code generation (branch `selfhost/s5-backend`)

Files: `selfhost/codegen.kode` (~2,000 lines estimated), `selfhost/driver.kode`.

Emits textual LLVM IR identical in shape to `kode mir`'s lowering target, writes
it to a temp file, and calls `clang` then `cc` through `exec`, reusing the
existing runtime archive (`kode doctor` prints its path).

Acceptance: every run-pass program links and prints its `.out`; binary sizes are
within a stated band of Stage 0's (recorded, not asserted to the byte).

Risk: this is the only stage that can produce a *wrong* binary that still runs.
Mitigation: compare program output for the whole corpus, and run
`KODE_LEAK_CHECK=1` on every one of them.

### S6 — Stage 1 compiles the corpus (branch `selfhost/s6-stage1`)

Acceptance: Stage 1, running as a compiled Kode program, passes `stage1`:
46 run-pass outputs, 98 compile-fail codes, leak-clean, and `kode fmt` over the
same files unchanged. The Rust `cargo test --workspace` stays green throughout.

### S7 — Stage 2 and reproducibility (branch `selfhost/s7-stage2`)

Acceptance: `scripts/selfhost.sh compare` reports parity between Stage 1 and
Stage 2 on the whole corpus; `benchmarks/report.md` gains a self-hosting section
with per-stage compile time, peak RSS and compiler size. Where the object files
are byte-identical, say so; where they are not, record the difference and the
reason (embedded paths are the expected cause).

## 5. The differential oracle

The port is verified by comparison, not by opinion. Each stage has one oracle
and one rule for what must match:

| stage | oracle | must match exactly | may differ |
|---|---|---|---|
| S1 lexer | `kode tokens` | every token, its text, byte span, line/column | internal token ids |
| S2 parser | `kode ast` | tree shape, spans, E01xx diagnostics | node allocation order |
| S3 checker | `kode check --format=json` | codes, messages, hints, spans, order | nothing |
| S4 MIR | `kode mir` dump | structure and types | temporary numbering |
| S5 backend | program stdout + leak check | output, exit code, zero leaks | binary size within a band |
| S6/S7 | all of the above, Stage 1 vs Stage 2 | everything above | object bytes only if §7 gates it |

`--format=json` is a compatibility promise (AGENTS.md 9), which makes it the
right contract to port against: a change there is a change users can see.

## 6. Ordering and size

The order is forced by Gate 0 and by the oracles, not by preference:

```
9.5 weak/arena ─┐
9.6 bytes      ─┼─→ S0 harness → S1 lexer → S2 parser → S3 checker → S4 MIR
9.7 enums      ─┤                                            → S5 backend
9.8 optionals  ─┤                                            → S6 stage 1
9.9 generics   ─┘                                            → S7 stage 2
```

Estimated size, to be replaced by the first real number at the end of S1:

| stage | estimate (Kode lines) | basis |
|---|---:|---|
| S1 lexer | ~1,200 | Rust lexer 1,000 + dump format |
| S2 parser | ~2,500 | Rust parser + AST |
| S3 checker | ~4,500 | `kode-sema` 5,386, minus table-driven stdlib rows |
| S4 MIR + evaluator | ~5,000 | `kode-mir` 4,536, minus RC-insertion passes |
| S5 backend | ~2,000 | `kode-llvm` 1,364 + driver via `exec` |
| S6/S7 glue | ~1,000 | CLI, manifest, harness |

That is ~16k lines of Kode against 27.6k lines of Rust, which is an estimate with
a stated method (Rust line count, minus the passes a Kode compiler can express
directly, plus the runtime calls it must make). It is not a measurement.

## 7. Risks

| risk | effect | mitigation |
|---|---|---|
| Gate 0 lands late (9.5–9.9 are five ADR-sized branches) | the port cannot start | treat Gate 0 as its own milestone; do not open S1 early "to make progress" |
| the ported compiler is much slower than Rust | bootstrapping takes minutes | measure at every stage; no optimization before S6; parallelism in the harness |
| diagnostics parity is unreachable in wording | S3 never closes | port one code family per commit; the JSON diff names the field that differs |
| RC cycles in the AST | leaks in the compiler itself, caught only at exit | decide arena vs `weak` in S2, and run the compiler's own leak check in CI from S2 on |
| the port drifts into feature work | scope creep, no parity | the only accepted change is parity; a new feature needs its own ADR outside this plan |
| Stage 2 differs because of embedded paths | false failure | compare behavior first, bytes second, and record exactly which bytes differ |

Stop rule (AGENTS.md): if a stage's parity gate is still failing after two
attempts, stop and report what was measured rather than widening the change.

## 8. Non-goals

- Replacing `clang` with a Kode-written backend.
- Removing the Rust compiler from the repository.
- Porting `kode lsp`, `kode fmt`, `kode doc`, `kode doctor` or the package
  resolver in this plan; they stay Rust until the compiler itself is
  self-hosted, and get their own plan afterwards.
- A self-hosted compiler that is faster than Stage 0.
- New language features introduced *for* the port.

## 9. Gates that need approval

| gate | question |
|---|---|
| A | Is Gate 0 the right ordering: 9.5, 9.6, 9.7, 9.8, 9.9 before any port code? |
| B | Are `kode tokens` and `kode ast` acceptable as permanent, documented commands (S0), given they add a new stable-ish output surface? |
| C | Interpreter before backend (S4 before S5), as `docs/self-hosting.md` recommends? |
| D | Is "behavior parity, byte parity where deterministic" the right definition of Stage 2 success? |
| E | Arena for the AST (S2) instead of `weak`, if the measurements say the arena is simpler? That would make ADR 0038 the answer for user code and an arena the answer for the compiler. |

Work on any stage starts only after its gate is approved, and every stage ends
with: what changed, tests added, measurements, risks, and the next stage.

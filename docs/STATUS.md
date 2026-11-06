# Status: v0.3 audit (Phase 0)

Last verified: 2026-11-06, commit `336d98f` (branch `main`), macOS 15 / arm64
(Apple silicon), debug compiler unless stated.

To reproduce the numbers in this file:

```
cargo build --workspace && cargo test --workspace
cargo clippy --workspace
./target/debug/kode doctor
./target/debug/kode fmt examples tests/run-pass --check
ls tests/run-pass/*.kode | wc -l ; ls tests/compile-fail/*.kode | wc -l
find crates -name '*.rs' | xargs wc -l | tail -1
bash benchmarks/run.sh
```

Everything below was run or read in this pass. Where something could not be
verified from the repository, it says so instead of guessing.

## 1. Verdicts

**Production-ready: no.** It is a real, working compiler for single-binary CLIs
and small HTTP services on macOS/Linux, but it is not safe to offer as a
general-purpose language. Blockers, in the order they hurt:

1. No TLS. `https://` raises an error (ADR/spec 13). Every real service needs it.
2. A compiler crash on moderately deep input: **stack overflow (SIGABRT)** at
   ~300 nested parentheses, a 1,000–2,000 term `+` chain, and 500 nested `if`
   blocks. A compiler must diagnose, not abort (§7).
3. Reference-counting cannot break cycles, so two objects pointing at each
   other leak forever (no `weak`).
4. No byte buffers and only 64-bit `int`; no fixed-width integers, no binary
   protocol work.
5. No database client.
6. Release automation only builds Linux x86-64, while the docs promise macOS,
   Windows and aarch64 artifacts (§4.1). Windows has never been verified by hand.

**Self-hosting-ready: no, and far away.** There is no lexer, parser, checker or
backend written in Kode (§3, Phase 6). The language also lacks the three things
a compiler needs first: enum payloads with pattern matching, generic classes,
and a faster memory story for token/AST graphs. The backend question (Cranelift
is a Rust library) is undecided.

## 2. Baseline

| Item | Value |
|---|---|
| Rust source | 20,210 lines across 7 crates (`find crates -name '*.rs' \| xargs wc -l`) |
| Kode source (tests + examples) | 1,972 lines (`find examples tests -name '*.kode' \| xargs wc -l`) |
| `tests/run-pass` | 44 programs (`ls tests/run-pass/*.kode \| wc -l`), leak-checked |
| `tests/compile-fail` | 92 programs (`ls tests/compile-fail/*.kode \| wc -l`) |
| Rust tests | 49 (`cargo test --workspace`, summing the `test result` lines) |
| ADRs | 30 files (`docs/adr/0001` … `0031`; numbering has gaps) |
| Spec chapters | 13 (`docs/spec/01` … `13`, including `07-planned`) |
| `cargo build --workspace` | passes |
| `cargo test --workspace` | passes, 49/49, 0 failed |
| `cargo clippy --workspace` | **fails** (3 deny-by-default errors, 2 warnings), all pre-existing (§7) |
| `kode fmt examples tests/run-pass --check` | passes |
| `kode doctor` | all ok: kode 0.3.0, macos-aarch64, runtime ABI 0003, clang 2100 |

Version drift: `Cargo.toml` says `0.3.0`; tags reach `v0.3.5`; the public
releases page shows `v0.3.4-test2`; `dist/` holds `0.3.0` artifacts. Four
different "current versions" in one project.

Repository hygiene: `dist/` is **65 MB of committed binaries** (a `kode` binary,
a tarball, a Windows zip, `SHA256SUMS.txt`), and it is stale at 0.3.0. There is
no `LICENSE` file even though `README.md` says MIT and the GitHub repo declares
MIT; there is no `SECURITY.md`.

## 3. Phase status

Legend: **done** = code and tests exist; **partial** = some of it works;
**missing** = no implementation.

### Phase 1 — language core

| Item | State | Evidence |
|---|---|---|
| 1.1 Enum payloads, `match` destructuring, `is`, safe downcast | **missing** | probed: `Shape.Circle(1)` is rejected (with a poor message — `E0201 unknown variable Shape`); `docs/spec/07-planned.md`, `docs/self-hosting.md` ("Enums are tags only"); no test |
| 1.2 Generic classes, generic methods, constraints, explicit type args | **missing** | ADR 0030 "Limits"; E0251/E0252; only free functions |
| 1.3 `weak` references / cycle safety | **missing** | `docs/design/memory-model.md` "Known limits"; no `weak` token in the lexer |
| 1.4 Fixed-width ints, byte buffers, binary encode/decode | **missing** | probed: `x: u8 = 1` → `E0303 unknown type u8`; only `int` (64-bit) |
| 1.5 Checked arithmetic plus `wrapping_*` / `saturating_*` | **missing** | probed: `9223372036854775807 + 1` prints `-9223372036854775808` (silent wrap); ADR 0009 |
| 1.6 Optional scalars as fields, parameters, returns | **partial** | `int?`/`float?`/`bool?` work as **locals** (ADR 0024, `tests/run-pass/optional_numbers.kode`); E0219 elsewhere |
| 1.7 String formatting, `shared` classes, cancellation/timeouts, print everything | **partial** | interpolation, `string()`, `to_int`/`to_float`, collection printing (ADR 0031); no `printf`-style formatting, no `shared`, no cancellation; classes/enums inside a collection are still E0203 |

### Phase 2 — standard library

| Item | State | Evidence |
|---|---|---|
| 2.1 Unicode-aware text, regex, format helpers | **partial** | char counts and boundary-checked slicing exist (`kode_str_len`, `s.slice`); no UTF-8 validation API, no grapheme policy, **no regex**, no format helpers |
| 2.2 Time: dates, durations, zones, ISO 8601 | **missing** | only `now_ms()` and `sleep(ms)` (spec 13) |
| 2.3 Crypto/encoding: SHA-256, HMAC, RNG, UUID, base64, hex | **missing** | no such module in `crates/kode-rt/src/` |
| 2.4 Sets, deques, comparator sorting, iterators | **partial** | lists/maps with `sort`, `contains`, `index_of`, `insert`, `remove_at`, `first`, `last`, `values`, `keys`; `sort` only for `int`/`float`/`string`, no comparator; no sets/deques/iterators |
| 2.5 Logging, config, signals, graceful shutdown, structured errors | **partial** | `env`, `args`, `exec`, `read_file`, `write_file`, error codes; no logging levels/JSON logs, no signals, no graceful shutdown |

### Phase 3 — networking and data

| Item | State | Evidence |
|---|---|---|
| 3.1 TLS | **missing** | `docs/spec/13-stdlib.md` "Only plain `http://` is supported"; `https://` raises |
| 3.2 Raw TCP/UDP sockets | **missing** | no socket API outside the HTTP server |
| 3.3 Server hardening | **partial** | HTTP/1.1 keep-alive (`http.rs:103`); **no request size limit, no read timeout, no static files, no middleware, no graceful drain** (grep of `http.rs`/`router.rs`) |
| 3.4 PostgreSQL client | **missing** | no driver |
| 3.5 C FFI | **missing** | no `extern fn`; ADR not written |

### Phase 4 — tooling

| Item | State | Evidence |
|---|---|---|
| 4.1 DWARF, stack traces | **missing** | runtime errors carry line numbers; no debug info emitted, no backtrace decoder |
| 4.2 LSP: completion, definition, rename, references; VS Code packaging | **partial** | LSP covers diagnostics, hover, completion, symbols, format, definition, references, rename (unit tests in `kode-cli`); no packaged extension (the `kode-vscode-exection` submodule was removed), Neovim docs exist |
| 4.3 Coverage, `kode bench`, fuzzing | **missing** | no coverage, no bench command (`benchmarks/` is a shell harness), no fuzz targets |
| 4.4 Registry and publishing | **missing** | path dependencies only; ADR 0025 covers order, nothing implemented |
| 4.5 `fmt` stability, docs site in CI | **partial** | `kode fmt --check` is in CI; `docs/site/build.sh` exists but is **not** run by CI |

### Phase 5 — platforms and release

| Item | State | Evidence |
|---|---|---|
| 5.1 Windows conformance on a real runner | **partial** | `ci.yml` has a `windows-latest` job with MinGW-w64 + GNU toolchain; **no confirmation it is green** (no `gh`, no CI access from here), and `docs/windows.md` says linked binaries are "mostly not run on Windows by hand" |
| 5.2 Installers, signing, reproducibility | **partial** | `release.yml` builds **Linux x86-64 only** (tarball + `.deb` + checksums). No macOS, no Windows, no aarch64 in CI despite docs; no signing; no reproducibility notes |
| 5.3 Cross-compilation | **missing** | `docs/windows.md`: runtime archive is host-built, "not wired up yet" |
| 5.4 LLVM `--release` backend | **missing** | Cranelift only |
| 5.5 Security process | **missing** | no `SECURITY.md`, no `cargo deny`, no supply-chain policy, no runtime-patch procedure |

### Phase 6 — self-hosting

| Item | State | Evidence |
|---|---|---|
| 6.1 Backend decision | **missing** | not decided |
| 6.2–6.5 Lexer, parser, checker, codegen in Kode | **missing** | `docs/self-hosting.md`: "No lexer, parser, checker, MIR builder, evaluator or native backend exists in Kode" |
| 6.6 Bootstrap proof in CI | **missing** | no bootstrap scripts, no stage comparison |

Prerequisites that are absent: enum payloads (1.1), generic classes (1.2), and
a memory strategy for AST/token graphs that does not leak on cycles (1.3).

### Phase 7 — stability and v1.0

| Item | State | Evidence |
|---|---|---|
| 7.1 Spec freeze, editions, deprecation | **missing** | no versioned reference, no edition concept |
| 7.2 Dogfooding two services for four weeks | **missing** | no record in the repo |
| 7.3 RC checklist | **missing** | would fail today: compiler crashes (§7), all-platform installers absent |
| 7.4 v1.0 notes with honest limits | **missing** | — |

## 4. Doc-versus-code contradictions

These are the defects this audit is meant to expose. Each is a doc claim that
the code contradicts today.

1. **Release artifacts.** `docs/releasing.md` promises five platform groups
   (Linux x86-64/aarch64, macOS Intel/Apple silicon, Windows) with `.dmg`,
   `.deb`, `.zip`, `.exe` and per-platform checksums. `release.yml` has exactly
   one job — `Build Linux x86_64` — producing a tarball, a `.deb` and
   `SHA256SUMS-linux-x86_64.txt`. `docs/windows.md` and `docs/ROADMAP.md` repeat
   the promise.
2. **`README.md` "Next" line** says "function values and package dependencies,
   then budgets and AI features (M8), tooling and portability (M9)". Function
   values (ADR 0028, `tests/run-pass/fn_values.kode`, `closures.kode`) and path
   dependencies (`kode add/install`, `tests/packages.rs`) shipped; M8 is
   **skipped**; M9 is in progress.
3. **Two cheat sheets.** `README.md` links `docs/agents/cheatsheet.md` in the
   status paragraph and `docs/cheatsheet.md` at the end. Both exist, with
   different content and different command lists.
4. **Binary size.** `README.md` says "release binary about 330 KB".
   `benchmarks/report.md` says 440 KB, and `kode build --release` of
   `examples/hello.kode` here is 450,696 bytes, 373,336 bytes after `strip -x`.
5. **Memory model is atomic now.**
   `docs/design/memory-model.md` says "Counts are not atomic yet; concurrency
   (M6) will make them atomic". Channels set `THREADED` and the object header
   uses atomic counts (`crates/kode-rt/src/channel.rs`, `object.rs`).
6. **`PLAN-v1.md` gap column is stale**: it lists "channels, `int?` + narrowing,
   `weak` fields" as gaps. Channels shipped (ADR 0026, `channel_heap.kode`),
   `int?` works as a local (ADR 0024). `finally` is also listed as a gap in
   §1 while §4 records it as done.
7. **`README.md` names the wrong CI file**: "CI (test.yml) and release
   (release.yml)"; the workflow is `ci.yml`.
8. **`docs/site/build.sh` reads `COPYING.txt`**, which does not exist, so the
   "Project information" page and its sidebar link are dead. The script also
   has a hard-coded `generics.html` sidebar entry.
9. **No `LICENSE` file**, though `README.md` says "License: MIT" and GitHub
   records MIT. `docs/releasing.md` also tells readers to run `kode doctor` on
   an artifact, which is right, but no artifact exists that was produced by CI.
10. **GitHub repository metadata.** `git ls-remote origin` shows `HEAD` and
    `refs/heads/main` at `336d98f`, identical to local `main` (0 ahead, 0
    behind, no unpushed commits), and `main` is the Rust tree (`Cargo.toml`,
    `crates/`). The Go code is on other refs only: branches
    `migrate-to-go`, `v0.3.1-release`, `v0.3.2-dev`, `v0.3.2-development`,
    `v0.3.3`, `v0.3.3-development`, and tags `v0.2.0-alpha` through
    `v0.3.4-test2`. The repository description still says "concurrency-first"
    and the topics still include `golang` and `interpreter`, which contradict a
    Rust/Cranelift native compiler; those are live metadata and should be
    corrected.
11. **Version drift** (§2): Cargo 0.3.0, tags to v0.3.5, releases page
    v0.3.4-test2, `dist/` 0.3.0.
12. **`docs/spec/07-planned.md`** is now accurate for channels and `finally`,
    but `docs/site/content/generics.md` still heads a section "What is not
    implemented yet" without saying generic functions do work — a reader can
    conclude generics are unbuilt.

## 5. Budget table (PLAN-v1 §3), filled

Measured on macOS 15 / arm64 (Apple silicon, not the 1 vCPU / 1 GB Linux VM the
plan targets). Wall clock: best of 3; RSS: peak of one run. Compiler: debug
`kode`. Full data: `benchmarks/report.md`.

| Target | Goal | Measured | Verdict |
|---|---|---|---|
| `fib(35)` wall clock | within 2× of C `-O2` | 93 ms vs 58 ms = **1.62×** | met |
| `loop_mod` wall clock | within 2× of C `-O2` | 32 ms vs 14 ms = **2.40×** | **missed** |
| `sieve` wall clock | within 2× of C `-O2` | 30 ms vs 15 ms = **1.94×** | met (thin margin) |
| `mandel` wall clock | within 3× of C `-O2` | 17 ms vs 12 ms = **1.41×** | met |
| idle HTTP server RSS | under 3 MB above the binary | 2,144 KB total, binary 440 KB → ~1.7 MB above | met |
| echo server RSS at 1000 requests | flat | 2,144 → 2,432 KB at 1,000; 2,160 → 2,464 KB at 10,000 | provisionally met (+0.3 MB, no growth with load) |
| hello world binary | under 1 MB stripped | 373,336 bytes stripped | met |
| `kode check` | under 100 ms on 5k lines | 4,802-line file: **9–12 ms** | met |

Not measured, and why:

- **1 vCPU / 1 GB Linux VM**: no such machine in this pass (local arm64 laptop,
  plus this repo has no VM provisioning). All four compute rows must be re-run
  there before quoting them.
- **Requests per second vs Node/Python/Go**: no HTTP benchmark exists in
  `benchmarks/` at all; only compute programs. The ROADMAP go/no-go checkpoint
  cannot be evaluated until one is written.
- **RSS comparison with Node/Python** for the same server: not built.
- **100,000 requests** (the flat-RSS target): ran 10,000 instead; a 10k run takes
  minutes with `curl` per request and a keep-alive client is the right tool.
- **`fib(35)` in a `--release` build of the *program*** is included, but the
  harness uses the debug `kode` compiler (the compiler's own build profile does
  not change Cranelift's optimisation, but the report should say so explicitly).

## 6. CI status

- `ci.yml`: matrix `ubuntu-latest`, `macos-latest`, `windows-latest`; build,
  `cargo test --workspace` with `KODE_LEAK_CHECK=1`, a conformance smoke test,
  and `kode fmt tests/run-pass --check`. Windows installs MSYS2 UCRT64 gcc and
  switches rustup to `stable-x86_64-pc-windows-gnu`.
- **Not verified green**: `gh` is not installed here and the GitHub Actions API
  was not queried. Treat Ubuntu/Windows as unproven until a run is inspected.
  The previous known run (before these fixes) failed on Ubuntu
  (`list_map_filter`, `generics_nested`) and on Windows (CRLF and `-lc`), which
  is what the current changes address.
- **Gaps in CI**: no `cargo clippy`, no `kode fmt` check for `examples/`, no
  docs-site build, no benchmark regression check, no release-artifact smoke test
  for macOS/Windows.

## 7. Crashes and robustness

Fuzzing-style probes run against `kode check` in this audit:

- 200 random token-soup programs: **0 crashes** (every one produced a
  diagnostic or compiled).
- **Stack overflow (SIGABRT, rc -6)** on valid or near-valid input:
  - 300 nested parentheses crash; 200 is fine.
  - a 2,000-term `+` chain crashes; 1,000 is fine.
  - 500 nested `if` blocks crash; 400 is fine.
  These are recursion-depth limits in the parser/lowering with no guard. A
  generated program (long concatenations, machine-written literals, deep JSON)
  hits the `+` case realistically. Fix: a depth counter with a diagnostic
  (new error code), not a bigger stack.
- `cargo clippy --workspace` fails on pre-existing code, deny-by-default:
  `mut_from_ref` ×3 in `Raw::len/cap/ptr` (`crates/kode-rt/src/list.rs:14-20`),
  a redundant closure (`list.rs:299`), a manual char comparison
  (`router.rs:293`). Confirmed present at `HEAD` with `git show`.

## 8. What I recommend fixing first (awaiting GATE 0)

Doc-only, cheap, no behaviour change:

1. `docs/releasing.md`, `docs/windows.md`, `docs/ROADMAP.md`: describe what
   `release.yml` actually builds (Linux x86-64 today), and mark the rest as
   planned.
2. `README.md`: drop the stale "Next" sentence, fix the `test.yml` name, pick one
   cheat sheet, correct the binary-size claim, add collections/channels/printing
   to "Works today".
3. `docs/design/memory-model.md`: atomic counts are in; cycles are still a leak.
4. `PLAN-v1.md` §1 gap column: remove channels/`int?`/`finally`.
5. Add `LICENSE`; fix `docs/site/build.sh`'s `COPYING.txt` and `generics.html`
   references.
6. Un-track `dist/` (65 MB of stale binaries) and add it to `.gitignore`.

Code, in dependency order (each needs its own branch, tests, ADR where it
changes behaviour):

1. **Depth limit with a diagnostic** for parser/lowering (§7) — small, high
   safety value, new error code.
2. **Clippy clean** (the `Raw` borrow refactor) — the repo's own rule 5.
3. **TLS** (Phase 3.1) — the single biggest production blocker; needs a DECISION
   on `rustls` vs system OpenSSL with a binary-size measurement.
4. **Enum payloads** (1.1) — unblocks self-hosting, needs an ADR.
5. **`weak`** (1.3) — stops the cycle leak; needs an ADR and leak evidence.

## 9. Gates

**GATE 0: passed** (2026-11-06). No feature code was written in Phase 0.

## 10. Resolution log

### `chore/docs-hygiene`

Fixed: §4.1 (releasing/windows/ROADMAP now describe the single Linux x86-64 job
and mark the rest planned), 4.2 (README "Next" line, added channels, closures,
generics, dependencies and collection printing to "Works today"), 4.3 (one cheat
sheet link; the agent cheat sheet now points to the full one), 4.4 (binary size
cited from `benchmarks/report.md` and re-measured: 450,696 bytes, 373,336
stripped), 4.5 (atomic counts documented), 4.6 (`PLAN-v1` gap column), 4.7
(`ci.yml`), 4.8 (`build.sh` reads `CHANGELOG.md`, sidebar says "What's new";
verified by running the site build), 4.9 (`LICENSE` added), 4.11 (`Cargo.toml` is
`0.3.5`, matching the latest tag; CHANGELOG gains a 0.3.5 section), 4.12
(generics page intro). `SECURITY.md` added. `dist/` untracked and ignored;
`benchmarks/report.md` regenerated.

Still open, needs a human or approval:

- §4.10: repository description and topics (`golang`, `interpreter`) are GitHub
  settings, not files; they must be changed in the web UI or with the API.
- Deleting the `-test` releases needs `gh` (not installed here):
  `gh release delete v0.3.4-test2 --yes` and `gh release delete v0.3.4-test --yes`.
- History rewrite for `dist/` is measured but **not applied**. Command, on a
  fresh clone: `git filter-repo --path dist --invert-paths`, then force-push
  branches and tags. Scratch-clone measurement: `.git` 42M before, 11M after
  (filter-branch alone left it at 41M until `refs/original` was deleted, which is
  why filter-repo is the right tool).

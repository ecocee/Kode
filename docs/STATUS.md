# Status: v0.3 audit (Phase 0)

Last verified: 2026-11-09, commit `60f491d` (branch `chore/baseline-verify`), macOS 15 / arm64
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

1. Programs that use TLS or PostgreSQL pay for them in full: hello world is
   **983,376 bytes stripped** with the release compiler, under the 1 MB budget,
   but a plaintext server is 1,068,744 bytes and a TLS server 5,431,968 bytes
   (2026-11-09), because optional runtime components (`rustls`, `webpki-roots`,
   the PostgreSQL client and the server-side crypto) are linked as one unit
   (§2, §10). `examples/db/postgres.kode` builds to 3,088,800 bytes.
2. TLS servers pay one thread per connection (ADR 0034) and do not use the epoll
   loop, so 1,024 concurrent TLS connections is the practical ceiling and RSS
   grows with connections, not with request rate. Plaintext capacity is measured
   in ADR 0037 (60,374 req/s at c=100 on one vCPU); above 1,024 connections the
   server closes the socket with no response instead of answering 503.
3. Reference-counting cannot break cycles, so two objects pointing at each
   other leak forever (no `weak`; the design and its measured leak evidence are
   in ADR 0038).
4. No byte buffers and only 64-bit `int`; no fixed-width integers, no binary
   protocol work.
5. A PostgreSQL client exists (ADR 0033) with TLS (rustls) and a connection
   pool, but no md5, `COPY`, binary format or cancellation. The HTTP client and
   server still have no TLS at all: `https://` raises an error.
6. Release automation builds Linux x86-64 and macOS arm64 only; Linux aarch64,
   macOS Intel and Windows artifacts are promised in the docs but not built, and
   Windows has never been verified by hand (§4.1).

**Self-hosting-ready: no, and far away.** The stages that would get there, with
oracles and gates, are in [PLAN-self-hosting.md](PLAN-self-hosting.md). There is no lexer, parser, checker or
backend written in Kode (§3, Phase 6). The language also lacks the three things
a compiler needs first: enum payloads with pattern matching, generic classes,
and a faster memory story for token/AST graphs. The first two now have designs
(ADR 0039, ADR 0040); neither has a line of code. The backend question is settled: LLVM, which needs `clang` at build time.

## 2. Baseline

| Item | Value |
|---|---|
| Rust source | 27,789 lines across 7 crates (`find crates -name '*.rs' \| xargs wc -l`) |
| Kode source (tests + examples) | 3,174 lines (`find examples tests -name '*.kode' \| xargs wc -l`) |
| `tests/run-pass` | 46 programs (`ls tests/run-pass/*.kode \| wc -l`), leak-checked |
| `tests/compile-fail` | 98 programs (`ls tests/compile-fail/*.kode \| wc -l`) |
| Rust tests | 108 (`cargo test --workspace`, summing the `test result` lines) |
| ADRs | 33 files (`docs/adr/0001` … `0034`; numbering has gaps) |
| Spec chapters | 13 (`docs/spec/01` … `13`, including `07-planned`) |
| `cargo build --workspace` | passes |
| `cargo test --workspace` | passes, 98/98, 0 failed, no leaks |
| `cargo clippy --workspace --all-targets -- -D warnings` | passes |
| `kode fmt examples tests/run-pass --check` | passes |
| `kode check` on every example and run-pass program | passes, 54 files |
| `kode doctor` | all ok: kode 0.3.6, macos-aarch64, runtime ABI 0004, clang 2100 |
| hello world, default build | 1,094,776 bytes |
| hello world, `--release` | 1,073,176 bytes |
| hello world, `--release` + `strip -x` | **983,376 bytes — under the 1 MB budget** |
| plaintext benchmark server, stripped | 1,068,744 bytes |
| TLS benchmark server (`serve_tls`), stripped | 5,431,968 bytes (2026-11-09, debug compiler; not re-measured) |
| embedded runtime archive | 12,204 KB, ABI 0006 (`kode doctor`) |

The hello-world rows were re-measured on 2026-11-24 with the release compiler;
an earlier pass used a debug-built compiler, which embeds an unoptimized runtime
archive and quoted 1,077,600 bytes stripped — that is why this section used to
record the 1 MB budget as missed. What remains true is that the runtime's
optional components are linked as one unit: a program that opens a TLS or
PostgreSQL connection pays for all of it (a TLS server is 5.4 MB,
`examples/db/postgres.kode` 3.1 MB). That is the size work in §10, not a
hello-world problem.

Version history: `Cargo.toml` says `0.3.6`, and `v0.3.6` is the tag for that
commit. The earlier `v0.3.5` tag points at `ae2ce27`, a 0.3.0 tree, which is why
the release workflow used to build 0.3.0 while the tag said 0.3.5; it now checks
the tag against the tree first. `dist/` is no longer tracked and is git-ignored;
`LICENSE` and `SECURITY.md` exist.

## 3. Phase status

Legend: **done** = code and tests exist; **partial** = some of it works;
**missing** = no implementation.

### Phase 1 — language core

| Item | State | Evidence |
|---|---|---|
| 1.1 Enum payloads, `match` destructuring, `is`, safe downcast | **done** (ADR 0039) | payload variants construct (`Shape.Circle(radius: 1.5)`), print with field names, destructure positionally in `match`, and `is` narrows the true branch so `s.radius` compiles while an unnarrowed read is `E0419`. A payload enum is a class-layout object whose class-id bits hold the tag; tag-only enums stay `int`; `kode_enum_tag` is the only new symbol. Evidence: `tests/run-pass/{enum_payloads,enum_match_bindings,enum_is}` (0 leaks), `compile-fail/{e0417_payload_value,e0419_field_without_narrow,e0420_enum_eq}`; `E0253` is gone |
| 1.2 Generic classes, generic methods, constraints, explicit type args | **partial** (ADR 0040) | slices 1a-1c: `class Box<T>` instantiates per type-argument tuple (`Box.int`), fields and methods substitute, the constructor infers; a method may declare its own type parameters (`Util.id.int`, compiled with `this` first, `tests/run-pass/generic_methods`); `tests/run-pass/generic_classes`, `compile-fail/e0421`-`e0423`, `e0252_generic_class_method`. Constraints and explicit type args still missing |
| 1.3 `weak` references / cycle safety | **deferred** (design only) | ADR 0038 (`docs/adr/0038-weak-references.md`) with measured leak evidence in `benchmarks/leaks/report.md`; no `weak` token in the lexer yet. ADR 0042 takes it off the self-hosting path: the port's AST is an arena of nodes with `int?` edges, so its graph has no cycles to break. Still worth doing for user code |
| 1.4 Fixed-width ints, byte buffers, binary encode/decode | **partial** (ADR 0043) | byte buffers are done: `bytes(n)`, `size()`, `get(i)`, `set(i, v)` with the index and `0..=255` checked, and `0x`/`0b`/`0o` integer literals (`tests/run-pass/{bytes_buffers,hex_literals}`, 0 leaks, `wp08-bytes` met). `u8`/`i32` as *numeric* types are still missing (`x: u8 = 1` → `E0303`), tracked as WP-08b; no binary encode/decode API yet |
| 1.5 Checked arithmetic plus `wrapping_*` / `saturating_*` | **missing** | probed: `9223372036854775807 + 1` prints `-9223372036854775808` (silent wrap); ADR 0009 |
| 1.6 Optional scalars as fields, parameters, returns | **done** | `int?`/`float?`/`bool?` work as **locals** (ADR 0024), **class fields** (two slots: the value and its flag) and **parameters and returns** (the pair travels across a call boxed, `null` for none, and is decoded into the same two locals — ADR 0041). `tests/run-pass/{optional_fields,optional_params}.kode` cover each shape with 0 leaks. A list or map of an optional number is still E0219 |
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
| 3.1 TLS | **done, with limits** | ADR 0034: `https://` client verified against webpki roots, `serve_tls` with DER certificates; the TLS server is thread-per-connection, not epoll; costs in `benchmarks/http/tls.md` |
| 3.2 Raw TCP/UDP sockets | **missing** | no socket API outside the HTTP server |
| 3.3 Server hardening | **partial** | HTTP/1.1 keep-alive (`http.rs:103`); **no request size limit, no read timeout, no static files, no middleware, no graceful drain** (grep of `http.rs`/`router.rs`) |
| 3.4 PostgreSQL client | **partial** | `pg_wire.rs`/`pg.rs` (ADR 0033): opportunistic TLS, SCRAM-SHA-256, simple + extended protocol, JSON results, `pg.pool`; **no md5, no `COPY`, no binary format, no cancellation**, `sslmode=require` still rejected |
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
| 5.1 Windows conformance on a real runner | **partial** | `ci.yml` (B9) runs build, clippy, tests, conformance, fmt, release build and a hello-world run on `windows-latest` with MinGW-w64, GNU Rust and Chocolatey LLVM. Two real runs have built, linked and run programs on Windows through the conformance and LSP harnesses (including the three new applications), and they pass with the MinGW-target clang fix except for one now-fixed LSP harness regression on the latest run |
| 5.2 Installers, signing, reproducibility | **partial** | `release.yml` builds Linux x86-64 (tarball, `.deb`, checksums), macOS arm64 (tarball, checksums) and Windows x86_64 (tarball, checksums); the release job installs clang (B9) and smoke-tests the built artifact on macOS. No Windows release artifact has been uploaded yet, and there is no signing or reproducibility record |
| 5.3 Cross-compilation | **missing** | `docs/windows.md`: runtime archive is host-built, "not wired up yet" |
| 5.4 LLVM `--release` backend | **done** | LLVM is the only backend since B7; Cranelift removed in B8 |
| 5.5 Security process | **missing** | no `SECURITY.md`, no `cargo deny`, no supply-chain policy, no runtime-patch procedure |

### Phase 6 — self-hosting

| Item | State | Evidence |
|---|---|---|
| 6.1 Bootstrap harness (S0) | **done** | `scripts/selfhost.sh` and the `kode tokens` / `kode ast` dumps (`crates/kode-syntax/src/dump.rs`); `stage0` green on a clean checkout, `stage1`/`compare` refuse without a Stage 1. Branch `selfhost/s0-harness` |
| 6.1 Backend decision | **missing** | not decided |
| 6.2 Lexer in Kode (S1) | **done** | `selfhost/{tokens,keywords,bytes,strings,lexer,dump,main}.kode`; `./scripts/selfhost.sh stage1` reports **283/283** token dumps byte-identical (text, byte spans, line/column, and the status of the programs that must fail to lex), 2.53 s and 4,400 KB for token dumps only (the binary now also carries the parser). Branch `selfhost/s1-lexer` |
| 6.3 Parser in Kode (S2) | **partial** | slices 1-14: `selfhost/ast.kode` (the arena, ADR 0042) plus `cursor.kode`, `infix.kode`, `expr.kode`, `literals.kode`, `stmt.kode` and `parser.kode`, which parse assignments, typed local declarations (`x: int = 5`) and expression statements, the full infix/prefix operator grammar at the oracle's binding powers, postfix chains (`.name`, `.name(args)`, `?.name`, `[i]`, slices, `!`), `[lists]`, `{maps}`, `none`, interpolated strings with holes re-lexed at their absolute offsets, blocks, `if`/`else if`/`else`, `while`, both `for` forms, `break`/`continue`, `return`, `fn` declarations (type parameters, annotated parameters, return type, block or `=> expr`), type annotations (`T`, `[T]`, `{string: T}`, `Box<T>`, `Task<T>`, `Channel<T>`, `fn(int) -> int`, `T?`), `class` declarations (generics, `implements`, fields with defaults, methods), `match` statements (`Wildcard`/`PatternValue` arms with block or single-statement bodies), `enum` (with payload variants), `interface` signatures, all three `import` forms, `const`, `throw`, `try`/`catch`/`finally`, `??`, `is` and the `catch` expression at every binding power 0 position (`catch` bodies come through the `Blocks` interface in `contracts.kode`, which is how the expression grammar reaches statements without a module cycle). Modules: `types.kode` (annotations, params), `collections.kode` (`[lists]`, `{maps}`), `decls.kode` (items), `enums.kode` (enums, interfaces), `stmt.kode` (statements), `parser.kode` (the program loop). **`kode-stage1 ast` is byte-identical to `kode ast` on 61 spec cases, on all 108 `tests/compile-fail` programs and on all 55 of the port's own files** (`./scripts/selfhost.sh parse`). Branch `selfhost/s2-parser`. Also ported: closures and `await` (in `closures.kode`, which like `interp.kode` reaches the expression parser through `contracts.kode`), `spawn f(1, 2)` (only a plain call, `E0117` otherwise), the 128-level nesting limit (`depth.kode`: a guard on the expression, block and type productions plus a stack walk over the finished tree, `E0118`, ADR 0032) and the oracle's diagnostics — `report.kode` renders code, position, source line, caret and hint byte for byte, with `Cursor.describe` naming tokens the way the oracle does, so `selfhost/spec/parse-errors` (24 cases) compares stderr too. every node also carries the `int?` parent edge ADR 0042 allows (set when it is attached). all eleven assignment operators with index, map and field targets (`xs[0] += 1`, `obj.name = 1`, `obj?.name` is `E0112`). the whole type grammar, function types included (`fn(int) -> int`, nested, zero-argument, `fn(int) -> int?`), plus `Task<T>`/`Channel<T>` as their own dump kinds and the oracle's `E0103`/`E0115` wording for malformed annotations |
| 6.4 Checker in Kode (S3) | **done** | slices 1–18: `selfhost/sema.kode` resolves names (top-level names, builtins, runtime functions, prelude classes, block scopes, `for`/`catch`/match-arm/closure bindings) and `selfhost/modules.kode` loads the module graph (`E0401` missing file, `E0402` cycle, `E0403` code in a module, `E0405`/`E0410` unknown import/member, `E0411` non-`pub` import or alias member; bare paths use the package root); `selfhost/namespace.kode` then owns the alias namespace (`E0408` duplicate alias, `E0409` alias hiding a declaration, both with the oracle's wording and hint, and the first alias in load order is the one a member call resolves through); `selfhost/names.kode` the declarations themselves (`E0301` name twice, `E0302` builtin redefined, `E0306` parameter or field twice, `E0307` a constant that is not a plain value, `E0308` constant twice); and `selfhost/typing.kode` + `selfhost/annotations.kode` + `selfhost/items.kode` + `selfhost/binding.kode` + `selfhost/operands.kode` + `selfhost/scope.kode` the types: expression and annotation typing (`E0303`, with the fixed-width hint), what a declaration puts in scope, assignment (`E0202`, `E0205`, `E0312`), operand and printing rules (`E0203`, `E0204`, `E0210`), and `return`/`break`/`continue` outside their place (`E0311`, `E0314`). `selfhost/signatures.kode` records what each function and class constructor takes and gives back, and `selfhost/functions.kode` checks a call against it (`E0207` too many, `E0215` no such parameter, `E0216` missing, `E0208` wrong type, type parameters bound from the arguments) and a return (`E0304`, `E0305`, `E0310`); `selfhost/stdlib.kode` is the third table — the runtime functions called by name (`exec`), the one-argument builtins in the oracle's own words (`len`, `bytes`, `int`, `float`, `string`, `assert`) and `StringBuilder`'s `push`/`build` — and a method is recorded as `Class.method`, resolved through its receiver's type, so `P().m(1, 2)` reports `E0207` about `P.m` while its return messages keep the bare `m`; a refused constructor or builtin call still binds the name (`is_ctor`), a refused function does not, and a row's result type (a spec string, since `typing.kode` reads the table and the table cannot import it back) makes `print(exec(...))` `E0203` ``cannot print `ProcessResult` yet`` and a call that returns nothing `E0205`. `selfhost/containers.kode` adds the shape rules: a condition that is not a bool (`E0209`, `if`/`while`/`else if`), indexing and slicing (`E0211` not indexable or not sliceable, the `Json`-in-place form, `E0212` list index / map key / `Json` key / slice bound), list and map literals (`E0213` one item type, no `none` or nothing; a refused literal declares no name, so `E0201` follows) and `for ... in` over a list or a map (`E0214`), with the loop binding the item, index, key or value it really yields; optionals stay with `E0220` and range bounds with `E0203`. `selfhost/optional.kode` adds the optionals: `E0219` (a list or map of `int?`, an optional of something unsupported, `none` without a type), `E0222` (an empty list or map without one), `E0220` (`.`, a method, an index or `!` on a value that may be none), `E0221` (`??`/`?.` on something that cannot be none, a default of the wrong type), `E0313` (a type written twice) and `E0419` (a payload-enum field read without the `is` narrowing that makes it safe — the walker tracks the then branch of `x is E.V`); `??` keeps an optional default and an enum variant types as its enum. `selfhost/members.kode` adds the member rules: `E0217`/`E0218` for a `.` a class or interface does not have, `E0417` for a payload variant used as a value, `E0228` for a parameter named like a field, `E0234`/`E0235` for a class that does not fulfil `implements`, and the `?.`-on-a-scalar form of `E0219`; a failed `.` declares nothing, so the next use is `E0201`. `members.value_type` gives a user call's result, a method's result and a class field's type (through `?.` as well), so operators and `print` see them; a generic call's result and a generic class's fields stay `any` (ADR 0030). `selfhost/generics.kode` answers whether a call can infer its type parameters (`E0250` a free function's, `E0422` a generic class constructor's from its fields, `E0252` a generic class's method that declares its own, `E0251` a generic function used as a value), and `annotations.kode` reads `Box<int>` (`E0421` wrong count, `E0423` bare `Box`); both annotation refusals leave the name undeclared, so `E0219`/`E0201` follow as they do in the oracle. `check` renders the oracle's full report (position, source line, carets, hint) and the gate requires `check text: 170/170`: every case's report is byte-identical to the oracle's once the wall-clock `checked in ... ms` success line is dropped, and `check` exits nonzero if a case's codes, its report or the port's self-check differs. Warnings exist (`W0001` is an unused variable) and do not fail `check`. Closures stay silent (their signatures are not in the table yet). A rule stays quiet when it cannot prove the mistake (`any`, optionals, two named types), which is what keeps the run-pass suite clean. `selfhost/diag.kode` holds the diagnostic record and its constructors; the builtin/std name table is `selfhost/builtins.kode`. Gate first: `./scripts/selfhost.sh check` compares the port's code set with the oracle's own over `tests/compile-fail` and `tests/run-pass` — **170/170** today, with **170/170** reports byte-identical (the result-type and print rules have their own fixtures, `e0203_print_process_result`, `e0205_print_nothing`, `e0203_print_optional_string` and the run-pass `optional_scalar_print`), plus **self-check 64/64** over the port's own sources. Branch `selfhost/s3-text` | |
| 6.5 MIR, evaluator, codegen in Kode | **partial** | S4 slices 1–7: `selfhost/mir.kode` (the model and the dump) and `selfhost/lower.kode` (the walk, 645 lines because it is one class) lower `print(...)` of scalar literals into one `<main>` function of runtime print calls (`rt::PrintStr`/`PrintInt`/`PrintFloat`/`PrintBool`/`PrintSep`/`PrintNl`, `goto bb1`, `return`) and, in slice 2, a local (`x = 41`), a name read and `+` with the oracle's temporaries (one per variable and per `add`, `    let _N: int  // x` with the variable's name, one `_N = 0` each at the top of `bb0`, `_N = add a, b`), and in slice 3 a reassignment (the existing temporary, after the value), the value-before-variable order a declaration follows, and `- * / %` as `sub`/`mul`/`div`/`rem` (truncating, like the language), and in slice 4 string variables as `ref` temporaries with the oracle's retain/release lines (released before every store and once in `bb1`, an intermediate released right after its last use, `+` as `_N = call rt::StrConcat(a, b)`), and in slice 5 interpolation as a `print` argument with its `StrConcat` chain, literal-head release, `retain` for a variable head, owned `rt::IntToStr` int holes and the print's own release, and in slice 6 comparisons (`lt`/`le`/`gt`/`ge`/`eq`/`ne` into a `bool` temporary) and `if`/`else` as blocks (`bb0`, the exit `bb1`, then the blocks in creation order, `branch` at the split, `goto` to the join), and in slice 7 `while` loops (`bb` order: condition, body, exit) with `break`/`continue` through a stack of targets and a single `emit` that starts a fresh block when the current one is terminated; it prints all of that in `kode mir`'s layout, and `selfhost/eval.kode` runs it with a `State` (one `int`, string and `bool` per temporary, plus the pending line) and a program counter that follows `goto`, `branch` and `return`, and keeps shadow reference counts (`release` of what it does not own, use after release and a leak at the end fail the run with exit 1; `./scripts/selfhost.sh own` is **4/4** with the mutation — a port whose every `retain` became a `release` — caught), so `kode-stage1 mir` and `kode-stage1 run` exist and a program outside the slice exits 2 with a reason. Fixtures first in `selfhost/spec/run/` (thirty-three programs and their `.out`): `./scripts/selfhost.sh mir` is **67/67** dumps byte-identical and `run` **67/67** cases; and, in slice 10, an interpolation as a value — a slot's, a call's argument, a `return`'s — where the accumulator is the caller's ownership (a single hole is the variable retained for it, so a store moves instead of retaining twice, and a call releases the count it made), and, in slice 9, calls (`name(args)` as `_N = call name(...)`, arguments in order, the destination sized from a return-kind table built before any body is walked, and a frame per call in `selfhost/eval.kode`, so recursion runs), which takes the corpus to **6/58** with mir **54/54** and run **54/54**; a parameter counts as a named variable (its `Temp.name` is empty, so `print(msg)` must not release it and `return s` must retain it), and `(x % 2^k) == 0` masks (`bitand x, 2^k - 1`) the way the oracle does; and, in slice 8, `fn` declarations and `return` (`selfhost/spec/run/fn_*`, `retain_on_assign`: parameters in the header, the return temporary at the first `return`, `bb1` releasing named references before it returns, `unreachable` for a block body that returns something, not for `=>` sugar), with **54/54** dumps and **54/54** runs, and a store or return of a borrowed reference now retaining it first (a divergence the first thirty-three fixtures never reached); over the real corpus, `./scripts/selfhost.sh corpus` is **6/58** programs in scope and the other 52 need **305 constructs**, every one listed in `selfhost/spec/corpus-out-of-scope.txt`, which `./scripts/selfhost.sh ledger` regenerates from the port and `corpus` fails on when the committed file differs (`kode-stage1 report` lists every blocker per program, not just the first, and `./scripts/selfhost.sh report` pins that with **5/5** fixtures in `selfhost/spec/report/`) with byte-identical output **and** the oracle's exit code (a fixture that does not exit 0 fails, and so does a port whose status differs), each `.out` checked against the oracle's own `kode run`. and, in slice 14, bool locals (`let _0: bool`, `temp_kind` learns `bool`), short-circuit `and`/`or` (result temporary first, then a branch whose body block the right side fills), the bitwise operators (`bitand`/`bitor`/`bitxor`/`shl`/`shr`, `~x` as `bitnot`) and `const` inlining (a constant is its literal at every use, read from the file that declared it) — **63/63** dumps and runs, four fixtures, and **25** constructs retired (318 → 293); the walk's leaf helpers moved to `pieces.kode` to keep `lower_expr.kode` under budget, and, in slice 15, lists: a list literal is `rt::ListNew(kind)` plus one `rt::ListPush` per element (a borrowed element retained first), an index is `rt::ListGet(xs, i, line)` whose third operand is the **source line** (`selfhost/lists.kode`, `Ast.src`, and a byte-wise `line_at` — the first version mixed characters and bytes and crashed on an em dash), `len` is `rt::ListLen`/`rt::StrLen`, and `for x in xs` is four blocks over a copy of the list with the loop variable retained from a borrow and the copy released in `bb1` (`Builder.at_end`) — **67/67** dumps and runs, four fixtures, element ownership in the interpreter (`Owned.items`, dropped with the list), the `report` fixtures re-pointed at a map literal, and the ledger **up** from 293 to **305** constructs in the same 52 programs (56 retired, 68 newly reachable once a list stops masking what follows), with `eval.kode` split into `own.kode` (values, shadow counts, drops); and, in slice 13, whole programs: `selfhost/modules.kode`'s loader supplies the post-order file list (`Loaded.order`: a file after its imports, the entry last), names stay flat so one function/class table serves the program, a field's default is read from the file that declared it, and `./scripts/selfhost.sh modules` compares dumps and runs per case (**3/3**, `selfhost/spec/modules/{chain,two,class_import}`; alias-qualified calls are still refused), and, in slice 12, classes (`selfhost/classes.kode`; `rt::ObjNewClass(count, ref_mask, 7)`, fields at `24 + 8 * i`, `load`/`store` with the borrow-and-release order a reference field needs, methods as `Class.method` with `this` first; `eval.kode` grew objects with real field slots and drops a dead object's reference fields) with **59/59** dumps and runs and five fixtures plus `own_field`, and, in slice 11, unary `-` and `not` (`_N = neg x`, `_N = not b`, one operand, one temporary — this is what took the corpus from 4/58 to **6/58** and retired 6 constructs in one slice), so what is left is a class-typed parameter, field or return, map literals and `ListSet`/slices (so a member read only types a name bound from `P()` in the same function: `report` now says `a member read of X on a value this slice cannot type` 86 times over the port's own sources), `for`, `else` on a loop, bool variables, string equality, other string operators, an interpolation with a non-name, float or bool hole or a real reference-counting pass yet; no backend in Kode. `selfhost/mir.kode` holds the model and the dump; the walk is `lower_core.kode` (the builder), `lower_stmt.kode`, `lower_expr.kode` (512), `pieces.kode` (126), `lists.kode` (80), `ops.kode`, `classes.kode`, `lower.kode` (functions, classes and `main`, 377) and the module loader `modules.kode` (379), with `eval.kode` (468) and `own.kode` (130) for the program and its functions, in a one-way import graph. Branches `selfhost/s4-interpreter`, `selfhost/s4-locals`, `selfhost/s4-operators`, `selfhost/s4-strings`, `selfhost/s4-interp`, `selfhost/s4-if`, `selfhost/s4-while` |
| 6.6 Bootstrap proof in CI | **done** for S0-S2 | the `selfhost` job in `ci.yml` runs `stage0`, `stage1` and then `parse` on `ubuntu-latest`: 270 token dumps, then the tree and diagnostic dumps of the 61 spec cases, the port's own files and the whole compile-fail corpus (173/173), uploading both measurement files; `stage2` waits for the backend |

Prerequisites: enum payloads (1.1, done), generic classes and methods (1.2, done for
what S2 and S3 need), and a memory strategy for the AST (1.3 → ADR 0042: an arena of
nodes with `int?` edges). S1 needed none of them; S2's prerequisite set is now met, and
WP-08 is `met` (byte buffers and base-prefixed literals, ADR 0043); WP-09/WP-10 wait
for the stages that need them, and WP-08b (`u8`/`i32`) is not on the S2 path.

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
4. **Binary size.** Resolved: `README.md` and `benchmarks/report.md` quote one
   measurement, made with the release compiler — 1,073,176 bytes, 983,376 after
   `strip -x`, which the budget row below records as met. The 1.2 MB figure
   quoted earlier came from a debug-built compiler, which embeds an unoptimized
   runtime archive.
5. **Memory model is atomic everywhere.** Reference counts are atomic in every
   program, not only once a task or channel starts: the old `THREADED` switch
   let a running thread read the count non-atomically while another updated it
   atomically (`crates/kode-rt/src/object.rs`). No mode switch remains.
6. **`PLAN-v1.md` gap column is stale**: it lists "channels, `int?` + narrowing,
   `weak` fields" as gaps. Channels shipped (ADR 0026, `channel_heap.kode`),
   `int?` works as a local (ADR 0024). `finally` is also listed as a gap in
   §1 while §4 records it as done.
8. **`docs/site/build.sh` reads `COPYING.txt`**, which does not exist, so the
   "Project information" page and its sidebar link are dead. The script also
   has a hard-coded `generics.html` sidebar entry.
9. **No `LICENSE` file**, though `README.md` says "License: Apache-2.0" and GitHub
   records Apache-2.0. `docs/releasing.md` also tells readers to run `kode doctor` on
   an artifact, which is right, but no artifact exists that was produced by CI.
10. **GitHub repository metadata.** `git ls-remote origin` shows `HEAD` and
    `refs/heads/main` at `336d98f`, identical to local `main` (0 ahead, 0
    behind, no unpushed commits), and `main` is the Rust tree (`Cargo.toml`,
    `crates/`). The Go code is on other refs only: branches
    `migrate-to-go`, `v0.3.1-release`, `v0.3.2-dev`, `v0.3.2-development`,
    `v0.3.3`, `v0.3.3-development`, and tags `v0.2.0-alpha` through
    `v0.3.4-test2`. The repository description still says "concurrency-first"
    and the topics still include `golang` and `interpreter`, which contradict a
    Rust/LLVM native compiler; those are live metadata and should be
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
| `fib(38)` wall clock | within 2× of C `-O2` | **58.5 ms** with optimization off vs C `-O2` 61 ms = **0.96×**; like for like (`-O0` both sides) 58.5 vs 106.8 ms. The optimized pair is not a comparison: clang rewrites only the Kode build | met |
| `loop_mod` wall clock | within 2× of C `-O2` | 13 ms vs 12 ms = **1.13×** | met |
| `sieve` wall clock | within 2× of C `-O2` | 31 ms vs 12 ms = **2.64×** | **missed** |
| `mandel` wall clock | within 3× of C `-O2` | 13 ms vs 11 ms = **1.17×** | met |
| idle HTTP server RSS | under 3 MB above the binary | 2,080 KB total, binary 1,848 KB → **232 KB above** (macOS arm64, 2026-11-24) | met |
| echo server RSS at 1000 requests | flat | 2,080 → 2,336 KB at 1,000; 2,352 KB at 10,000 (+256 KB, then +16 KB for ten times the load) | met |
| hello world binary | under 1 MB stripped | **983,376 bytes** stripped, 1,073,176 unstripped (`kode build --release` with the release compiler at `461c406`, measured 2026-11-24) | met |
| `kode check` | under 100 ms on 5k lines | 4,802-line file: **9–12 ms** | met |

Re-measured 2026-11-24 with the release compiler at `461c406`
(`benchmarks/run.sh`, LLVM backend); every row above quotes that run, and
`benchmarks/report.md` holds the medians, the provenance and the method. `fib`
is the awkward one: clang rewrites the recursion in the Kode build and leaves
C's plain, so the optimized pair measures two different programs and only the
unoptimized pair is a comparison. The RSS rows are native macOS arm64 numbers —
the container harness (`benchmarks/http/run.sh`) needs Docker, which was not
available — so they are not directly comparable with the container runs quoted
in older notes.

Not measured, and why:

- **1 vCPU / 1 GB Linux VM**: no such machine in this pass (local arm64 laptop,
  plus this repo has no VM provisioning). All four compute rows must be re-run
  there before quoting them.
- **Requests per second vs Node/Python/Go**: the Kode side is now measured
  (ADR 0037, `benchmarks/http/report.md` "Stage 2"): 60,374 req/s for `/hello`
  at c=100 and 43,304 at c=1,000 on 1 vCPU / 1 GiB, 1,024 connections, ~20 MB.
  The other languages' rows come from a harness whose client source
  (`benchmarks/http/load.go`) is missing, so the comparison is **not** complete
  and the ROADMAP checkpoint still cannot be closed.
- **RSS comparison with Node/Python** for the same server: not built.
- **100,000 requests** (the flat-RSS target): ran 10,000 instead; a 10k run takes
  minutes with `curl` per request and a keep-alive client is the right tool.
- **`fib`** is measured twice: optimized, where clang rewrites the Kode build's
  recursion and leaves C's alone, so the ratio is not a language comparison; and
  unoptimized on both sides, which is. The report now records the compiler
  profile, the source commit and the build command, and the harness uses the
  release compiler — it stops with an error rather than falling back to the
  debug one.

## 6. CI status

- `ci.yml` (B9): one matrix, four native runners — `ubuntu-latest` (x86-64),
  `ubuntu-24.04-arm` (arm64), `macos-15` (arm64), `windows-latest` (x86-64).
  Each asserts `uname -m`, installs clang explicitly (apt, Chocolatey LLVM, or
  the verified system Command Line Tools) and runs: `cargo build --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` with `KODE_LEAK_CHECK=1`,
  `cargo test -p kode-cli --test llvm_conformance`,
  `kode fmt tests/run-pass examples --check`, `cargo build --release -p kode-cli`,
  then builds and runs `examples/hello.kode` with the release binary.
  No skip list, no `continue-on-error`. Details: `docs/platforms.md`.
- Two Windows runs have been inspected (the previous one and this one). They
  exposed four compiler/test fixes that are now in the tree:
    1. `-fPIC` was passed to MSVC-target clang and made every build fail
       (`unsupported option '-fPIC'`).
    2. large nested source crashed the compiler on the 1 MB Windows main-thread
       stack instead of reporting E0118.
    3. objects were emitted for the MSVC ABI while MinGW links them (a 256 KB frame
       probe showed `__chkstk` vs `___chkstk_ms`).
    4. the newer conformance harness did not honour `// skip-os: windows`, so
       `process_failures.kode` ran on Windows and failed.
  The fixes pass locally and turn around the two real failures seen in CI. The
  remaining difference in the latest run was one internal LSP test that asserted on
  exact byte-for-byte stdout; its assertion relaxed once that regress stopped, and
  the workspace is green now.
- **Gaps in CI**: no docs-site build in CI, no benchmark regression check, no
  Windows release artifact yet. The macOS smoke test in `release.yml` runs the
  artifact, so it is closer to "publish what the workflow built" than the Linux job,
  which still only smoke-tests the debug-compatible binary."


## 7. Crashes and robustness

Fuzzing-style probes run against `kode check` in this audit:

- 200 random token-soup programs: **0 crashes** (every one produced a
  diagnostic or compiled).
- **Stack overflow (SIGABRT, rc -6)** on valid or near-valid input:
  - 300 nested parentheses crash; 200 is fine.
  - a 2,000-term `+` chain crashes; 1,000 is fine.
  - 500 nested `if` blocks crash; 400 is fine.
  These were recursion-depth limits in the parser/lowering with no guard. A
  generated program (long concatenations, machine-written literals, deep JSON)
  hits the `+` case realistically. **Fixed on `fix/depth-limit`** with a guard
  and a new error code, not a bigger stack (§10).
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
5. **`weak`** (1.3) — makes the cycle leak avoidable for user graphs; ADR 0038 and the
   evidence exist. Not on the self-hosting path any more (ADR 0042 chose an arena for
   the port's AST), so it competes with the rest of the 1.x list rather than blocking S2.

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

### `ci/release-macos`

`release.yml` is now three jobs: Linux x86-64 (as before), macOS arm64
(`macos-15`, after the macOS 14 runner images were retired in November 2026),
and a publish job that merges the per-platform `SHA256SUMS-<platform>.txt` files
into one `SHA256SUMS.txt`. Each machine smoke-tests the artifact it built
(`kode doctor`, a `--version` that matches the tag, and `kode run` on hello
world) before packaging. The macOS steps were run by hand on this machine
(macOS 15 arm64): tarball 6.9 MB, checksummed, extracted binary runs. Releasing,
ROADMAP, README and `docs/windows.md` now describe exactly these two platforms.

### `fix/diagnostics-for-missing-features`

§3's two misleading errors are fixed without changing behavior: `Shape.Circle(1)`
is `E0253` ("enum payloads are not supported yet", hint to use a class, pointer
to `docs/ROADMAP.md`), and `x: u8 = 1` keeps
`E0303` but its hint lists the available types and says fixed-width numbers are
planned. The 15 codes that were missing from `docs/errors/index.md`
(`E0111`-`E0117`, `E0234`-`E0239`, `E0310`-`E0314`) now have rows, so `kode
explain` answers for every code the compiler emits. Tests:
`tests/compile-fail/enum_payload.kode`, `tests/compile-fail/fixed_width_int.kode`.

### `fix/clippy`

`cargo clippy --workspace --all-targets -- -D warnings` is now clean. The three
deny-by-default `mut_from_ref` errors in `Raw::len/cap/ptr` were real aliasing
UB (two `&mut i64` into one object), so they now return raw pointers; the eight
brand-new warnings that appeared once `kode-rt` compiled were fixed rather than
allowed, except `generic_call`'s eight parameters, which carries a comment.
CI gained the clippy and `kode fmt examples` steps.

### `fix/depth-limit`

§3 crash probes are fixed: 300 nested parentheses, a 2,000-term `+` chain and
500 nested `if` blocks now report `E0118` instead of aborting with exit code
134. The limit is 128 levels, enforced while parsing and again by an iterative
walk of the finished tree; `kode fmt` is line-based and never recursed, and
`kode lsp` parses through the same entry point. ADR 0032, tests
`tests/compile-fail/deep_{parens,chain,blocks}.kode` and
`tests/run-pass/nesting_ok.kode`.

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

### TLS: `feat/tls-transport`, `feat/http-client-https`, `feat/http-server-tls`, `perf/tls-budget`, `docs/tls`

Plan item 9.3, ADR 0034. `https://` works in the HTTP client (webpki roots and
host-name verification, failures name the host), `serve_tls` serves HTTPS from
DER certificate and key files with a handler or a `Router`, and one `Transport`
in `crates/kode-rt/src/tls.rs` is shared with the PostgreSQL client (whose
per-read `Mutex` is gone).

- Tests: fixture-certificate round trip, a certificate rejected for the wrong
  host, keep-alive over TLS in Rust, `serve_tls` verified with `curl` for both a
  handler and a `Router`, a missing-certificate startup error, and the
  compile-fail case `tests/compile-fail/serve_tls_bad_handler.kode` (E0232).
- Cost, `benchmarks/http/tls.md` (PRELIMINARY, one laptop): hello 1,077,600
  bytes stripped; plaintext server 1,484,136; TLS server 5,431,968; idle RSS
  2,080 KB vs 3,600 KB; keep-alive 140,734 vs 62,346 req/s; handshake ~1 ms.
- Not done: TLS on the epoll loop, PEM loading, certificate rotation, custom
  CAs, client certificates, HTTP/2. `sslmode=require` is still rejected for the
  PostgreSQL client, but the error now says that instead of claiming TLS is
  unimplemented.

### `chore/baseline-verify`

Re-verified the tree at `60f491d` (the `bench/http` line, 11 commits ahead of
`main`) before starting TLS work. Every number in §2 is from this pass.

- Cleared the last clippy failures: three `std::io::Error::new(ErrorKind::Other, …)`
  in `crates/kode-rt/src/pg_tls.rs` now use `std::io::Error::other`; a dead
  `TlsStream::plaintext`, a dead `pg::pool_slot`, an unused `std::net::Shutdown`
  import and a doc comment on a `thread_local!` (now a plain comment) are gone.
  `cargo clippy --workspace --all-targets -- -D warnings` is green.
- Gate: build, 98 tests with `KODE_LEAK_CHECK=1` (0 leaks), clippy, `kode fmt
  --check`, and `kode check` on all 54 example and run-pass programs.
- **Regression recorded, not fixed here:** hello world is 1,077,600 bytes
  stripped (debug 1,252,288, release 1,233,472). The PostgreSQL client brought
  `rustls` and `webpki-roots` into `kode-rt`, and they are linked into every
  program; the embedded runtime archive is 59,649 KB. The 1 MB budget is missed
  by about 29 KB. Fix with link-on-use (split the archive per subsystem) or
  change the budget deliberately, before A4 publishes any size claim.

## Self-host readiness (measured)

`./scripts/selfhost.sh readiness` prints the level and the blocked work packets;
`./scripts/selfhost.sh gates` prints the table itself. Today: **R1** — the lexer
runs in Kode with token-dump parity (283/283 dumps), and the S2 parser is
byte-identical on 173/173 cases and on all 55 files of the port. R2 (the language an
AST needs) is 11 met, 2 blocked: `StringBuilder` (WP-09, ADR 0044) landed for S5's
IR text and `stderr(...)` (WP-10, ADR 0045) replaced the port's `/dev/stderr`
write; only `weak` and `u8`/`i32` stay deferred (ADR 0042/0043). One probe per work
packet lives in `selfhost/spec/gates/`; the plan and the levels are in
`docs/PLAN-self-hosting.md`.

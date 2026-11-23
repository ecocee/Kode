# Contributing

Thanks for helping. Kode is a compiler, so the bar for a change is that the tests
prove it — not that it looks right.

Participation is covered by the [Code of Conduct](CODE_OF_CONDUCT.md). Report
security problems privately, per [SECURITY.md](SECURITY.md), never in a public
issue. The names "Kode" and "Ecocee" are not licensed with the code; see
[TRADEMARKS.md](TRADEMARKS.md).

## Set up

- Rust 1.91 or newer, and a C compiler (`cc`). The backend asks `clang` 15 or
  newer for an object file, or the program named in `KODE_CLANG`.
- `cargo build --workspace` builds the compiler and the runtime;
  `cargo test --workspace` runs the conformance suite with the leak checker on.
- `./target/debug/kode` is the compiler you just built; `cargo build --release`
  makes the fast one.

## Before you open a pull request

1. Add a test for what you change (`AGENTS.md`, rule 1). A feature starts as a
   `.kode` file with a `.out` beside it in `tests/run-pass/`; something that must
   be rejected starts as a `.kode` file with a `.err` holding the error code in
   `tests/compile-fail/`.
2. Keep a file under about 500 lines and split by idea.
3. A language change needs a short ADR in `docs/adr/`.
4. Update the docs the change touches in the same commit (`README.md`,
   `CHANGELOG.md`, `docs/PLAN-v1.md`, the relevant `docs/spec/` chapter).
5. Format and lint: `cargo fmt --all`, `cargo clippy --workspace`, and
   `./target/debug/kode fmt --check <the files you changed>`.
6. One idea per pull request. Ask before touching the memory model, the binary
   model or the backend: `AGENTS.md` lists what needs a maintainer decision.

## The self-hosted compiler

Much of the current work lives in `selfhost/`: a port of the compiler written in
Kode and compared with the Rust compiler byte for byte.
`docs/PLAN-self-hosting.md` is the plan.

```bash
./scripts/selfhost.sh all   # every gate: parse, check, mir, run, modules, own, report, corpus
./scripts/selfhost.sh mir   # just the MIR dump comparison
./target/debug/kode fmt selfhost/*.kode
```

A Rust change that alters what the compiler lowers must keep those gates green;
that is what keeps the port trustworthy.

## Review

- One maintainer, under ten hours a week: expect days, not minutes, and a small
  pull request is reviewed long before a large one.
- CI must be green: build, clippy with `-D warnings`, tests with
  `KODE_LEAK_CHECK=1`, conformance, `kode fmt --check`, the selfhost gates and
  the third-party notices check.
- There are no per-file license headers, and new files should not add any: the
  license is [LICENSE](LICENSE).
- AI assistance is welcome. The human who submits the change signs the DCO and is
  responsible for what the tool produced; an agent commits under its operator's
  identity, never as itself.
- A dependency change needs `python3 scripts/third-party-notices.py` run in the
  same commit, so the notices that ship in a release stay true.

## Licensing of contributions

Contributions are accepted under the Apache License 2.0, the license this project
already uses. Per [Section 5 of the license](LICENSE) ("Submission of
Contributions"), unless you explicitly state otherwise, any contribution you
intentionally submit for inclusion in this project is licensed under the terms
and conditions of Apache-2.0, with no additional terms or conditions.

You keep the copyright in your contribution. You grant the project the copyright
and patent licenses in Sections 2 and 3, on the same terms as everyone else.
There is no separate CLA to sign.

## Sign off every commit (DCO)

Every commit must carry a `Signed-off-by` line. It is the Developer Certificate
of Origin, version 1.1 (<https://developercertificate.org/>): by adding it you
certify that you wrote the change, or that you have the right to submit it under
the project's license.

Add the line with `git commit -s` (`-s` is short for `--signoff`), which appends:

```text
Signed-off-by: Your Name <you@example.com>
```

The name and email must match the commit author and must be a real identity you
can be reached at — no pseudonyms, no shared accounts, no `Signed-off-by` on
someone else's behalf.

For commits you have already made:

```bash
git commit --amend -s --no-edit     # the last commit
git rebase --signoff origin/main    # every commit on the branch
```

The `dco` workflow checks a pull request's commits and fails the job if the line
is missing; a sign-off naming someone other than the author is reported as a
warning so it can be explained in review.

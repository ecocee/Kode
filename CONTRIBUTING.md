# Contributing

1. Build and test: `cargo build --workspace && cargo test --workspace`.
2. Add a test for what you change (see `AGENTS.md`, rule 1).
3. Format and lint before committing: `cargo fmt --all` and `cargo clippy --workspace`.
4. Language changes need a short ADR in `docs/adr/`.
5. Open a pull request. CI runs on Linux and macOS.

## Branches

Two branches are long-lived:

- `main` — the Rust compiler that ships releases. Runtime, CLI, docs and release
  work targets it.
- `next/selfhost` — the compiler being written in Kode (`selfhost/`), with the
  Stage 0/1 gates in `scripts/selfhost.sh`. Self-hosting work targets it until the
  port can build itself.

Everything else is short-lived, named by intent and deleted after merge:
`feat/<feature>`, `fix/<issue>`, `perf/<optimisation>`, `test/<coverage>`,
`docs/<topic>`, `ci/<workflow>`, and `release/<version>` when a maintenance line
is actually needed. Branches are named after the work, not after a compiler stage.

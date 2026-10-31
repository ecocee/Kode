# Contributing

1. Build and test: `cargo build --workspace && cargo test --workspace`.
2. Add a test for what you change (see `AGENTS.md`, rule 1).
3. Format and lint before committing: `cargo fmt --all` and `cargo clippy --workspace`.
4. Language changes need a short ADR in `docs/adr/`.
5. Open a pull request. CI runs on Linux and macOS.

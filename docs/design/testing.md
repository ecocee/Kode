# Testing

`cargo test --workspace` runs `crates/kode-cli/tests/conformance.rs`:

- `tests/run-pass/*.kode` with `*.out`: the program must compile, run, print exactly the `.out`
  text, and pass the leak check (`KODE_LEAK_CHECK=1`).
- `tests/compile-fail/*.kode` with `*.err`: `kode check --strict --format=json` must fail with
  every error code in the `.err` file (one per line, or space separated when a case
  proves two rules at once; other diagnostics the program also reports are allowed).

The `.kode` files are the language specification in executable form. When the implementation
changes (even to another language), these files stay.

`cargo test` builds the runtime library automatically if it is missing.

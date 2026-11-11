# The self-hosted compiler (Stage 1)

This folder holds the Kode compiler written in Kode. It is empty of code on
purpose: Gate 0 in [PLAN-self-hosting.md](../docs/PLAN-self-hosting.md) still has
to close before the port starts, because the parser needs generic classes, enum
payloads, byte buffers and optional scalars in fields.

What exists today is the harness around it:

```sh
./scripts/selfhost.sh stage0     # Stage 0 dumps the oracles; green on a clean checkout
./scripts/selfhost.sh stage1     # needs main.kode, which arrives with S1
./scripts/selfhost.sh compare    # diffs Stage 1 and Stage 2 dumps
```

`main.kode` is the compiler's entry point, `lexer.kode` and `tokens.kode` arrive
with S1, and every stage is checked against `kode tokens` and `kode ast` rather
than against an opinion. See [docs/self-hosting.md](../docs/self-hosting.md) for
what the language can express today.

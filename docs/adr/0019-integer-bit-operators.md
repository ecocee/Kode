# ADR 0019: Integer bit operators

Status: accepted for v0.4

## Context

Kode could not express a bit mask, a flag set, a hash or a lexer's character classes,
which blocks systems code and self-hosting. The MIR already had an internal `BitAnd`
used by the `x % 2 == 0` optimisation, but the language surface had no bit operators.

## Decision

1. Add `&`, `|`, `^` (binary), `~` (unary), `<<` and `>>`, plus the compound
   assignments `&=`, `|=`, `^=`, `<<=` and `>>=`. They desugar to the existing
   assignment form, so no new statement is needed.
2. They accept two `int`s and give an `int`. Floats and bools are rejected with E0203;
   `not` remains the bool operator.
3. Precedence follows C: `* / %` > `+ -` > `<< >>` > comparisons > `&` > `^` > `|` >
   `and` > `or`. `not` sits between the comparisons and `&`.
4. `~x` is `-x - 1` (two's complement). `>>` is arithmetic: it keeps the sign.
5. A shift by 64 or more, or by a negative count, produces `0`. The generated code
   checks a variable count once; a constant count is folded. This is defined behaviour
   instead of the undefined or trapping behaviour of C and Rust debug builds.
6. Cranelift lowers the operators directly (`band`, `bor`, `bxor`, `bnot`, `ishl`,
   `sshr`); no runtime call is needed except for the shift guard described above.

## Consequences

- Programs can build masks, flags and simple hashes without dropping to another
  language, and the lexer/parser port becomes possible.
- The precedence change is observable for expressions that mixed `and`/`or` with
  comparisons; those keep their relative order, and the conformance suite covers the
  new table.
- No new error code is needed: wrong operand types already use E0203.

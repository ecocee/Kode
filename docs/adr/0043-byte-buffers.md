# 43. Byte buffers, and integers in bases 2, 8 and 16

- Status: accepted
- Date: 2026-11-13
- Work packet: WP-08

## Context

The port reads bytes: a UTF-8 string is a sequence of bytes with character
boundaries to respect, an object file is bytes, and the IR text a compiler writes
is built byte by byte. On Stage 0 that work is done with `s.byte_at(i)` returning an
`int`, and with no way to hold a mutable run of bytes at all.

Two gaps showed up at once when the WP-08 probe was written:

```
b = bytes(4)        # E0303: unknown type/function
b.set(0, 0x6b)      # E0006: `0x6b` is not a valid number
```

The lexer accepts only decimal integers (`42`, `1_000`), so even the byte value in
the probe could not be written. `docs/STATUS.md` §1.4 has recorded fixed-width
numbers as missing since the language was written; the error message for `u8` and
`i32` already says they are a planned feature, and `int` is 64-bit everywhere:
sema types, MIR `Ty::Int`, and an LLVM `i64`.

## Decision

### 1. Integer literals in base 2, 8 and 16

`0b1010`, `0o755` and `0x6b` are integers. The prefix letter is accepted in either
case (`0X`), `_` may separate digits, and the value must fit in an `int`
(E0006 when the digits are not valid for the base, or the value is too large). A
base-prefixed literal is still an `int`: there is no new token kind, so every
existing consumer of integer literals (constant folding, `match` arms, array
sizes, codegen) works unchanged.

Decimal floats are unchanged: `0x` is never part of a float, so `0x1.5` is
`0x1` followed by `.5`.

### 2. `bytes(n)` is a buffer of `n` bytes

```
b = bytes(4)          # four zero bytes
b.set(0, 0x6b)        # one byte, 0..=255
b.size()              # 4
b.get(0)              # 107
```

- `bytes(n)` allocates `n` zero bytes; a negative `n` is a runtime error.
- `size()` and `get(i)` return `int`; `get` is bounds-checked and reports the line.
- `set(i, v)` checks the index and that `v` is in `0..=255`. Out-of-range values are
  a runtime error rather than a truncation, because a silent `300 → 44` is exactly
  the class of bug the packet exists to remove.
- The type is `bytes`: it can be annotated, passed, returned, stored in a local and
  compared for identity, but it is not printable, not a `[int]`, not iterable and
  has no arithmetic. `bytes` is not a class, so it cannot be constructed with
  `bytes(...)` syntax other than the builtin.

### 3. Representation: the packed-list object

A `bytes` value is a packed-list object (`LIST | PACKED1`): one header, a
`Vec<u8>`. Retain, release, drop, `kode_list_len`, `kode_list_get8` and
`kode_list_set8` are the ones lists already use; only `kode_bytes_new` and
`kode_bytes_set` (the `0..=255` check) are new. This keeps the object graph the
reference counter sees unchanged, so a `bytes` value is freed exactly like a list
and the leak checks keep their meaning.

The consequence, and the reason it is written down: a `bytes` value and a `[bool]`
share a representation but not a type, so the type checker is the only thing that
distinguishes them. MIR therefore maps `bytes` to `Ty::Ref`, and `set`/`get` lower
to the packed accessors.

### 4. Fixed-width numbers stay out

`u8` and `i32` as *numeric types* would need new variants in `Type` and `Ty`,
promotion and mixing rules, LLVM `i8`/`i32` widths, overflow semantics and
printing. That is a language change large enough for its own ADR, and none of it is
needed to read bytes: `get` returns an `int` in `0..=255` and `set` accepts one.
The remaining half of the packet is tracked as **WP-08b** in
`docs/PLAN-self-hosting.md` and stays `E0303` on purpose.

## Consequences

- The WP-08 probe (`selfhost/spec/gates/wp08-bytes.kode`) is `met`, and
  `tests/run-pass/bytes_buffers.kode` covers zero-length buffers, a 100,000-byte
  buffer, and passing and returning a buffer, with 0 leaks.
- `0x`/`0b`/`0o` literals are covered by `tests/run-pass/hex_literals.kode`.
- Strings and byte buffers do not convert implicitly; a UTF-8 encoder will add
  `string → bytes` when a stage needs it.
- `bytes` is not printable: `print(b)` is E0203, the same as a class. Debugging
  prints bytes with a loop over `size()`/`get(i)` for now.

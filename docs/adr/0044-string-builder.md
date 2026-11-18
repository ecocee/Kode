# 44. `StringBuilder`: a growable buffer for building text

- Status: accepted, and implemented
- Date: 2026-11-18
- Work packet: WP-09

## Context

S5 emits LLVM IR as text. Written the only way the language allows today, every
line of output copies everything written before it:

```
text = text + line      # copies the whole text once per line
```

For *n* bytes of output that is about *n*²/2 bytes copied, so a 1 MB module moves
half a terabyte. The WP-09 probe was one line of the shape the backend needs, and
it failed at the first token:

```
sb = StringBuilder()    # E0206: unknown function `StringBuilder`
```

`docs/self-hosting.md` listed "no growable string builder" as a gap from the
first version of the plan; it is the last language feature S2–S5 need that is not
about diagnostics (WP-10, `stderr`) or process output (WP-12, streaming `exec`).

## Decision

`StringBuilder()` is a **builtin**, not a class: `Builtin::StringBuilder` in
`crates/kode-sema/src/builtins.rs`, typed `Type::Builder`. A prelude class would
have to be shipped with every program and would make constructor dispatch depend
on a library; a builtin is one enum arm, one type, and no user-visible name
resolution changes.

The value is one reference-counted runtime object
(`crates/kode-rt/src/buffer.rs`), laid out like a list — `[rc][kind = BUFFER]
[length][capacity][data pointer]` around a `Vec<u8>` — with three symbols:

| method | runtime | meaning |
|---|---|---|
| `StringBuilder()` | `kode_buffer_new` | an empty buffer |
| `sb.push(s)` | `kode_buffer_push` | copy the bytes of a string, linear amortised |
| `sb.build()` | `kode_buffer_build` | the text so far as one new string |

`push` **copies** the argument's bytes instead of retaining the string. That keeps
the builder independent of what it was given (no dangling pieces, no release loop
on drop), and it is what makes a builder a different object from the text it
builds. `build` copies out and leaves the builder usable, so it can be called
twice or in a loop; it does not hand out a view into the mutable buffer.

No new error code is needed: `sb.push(1)` is `E0208`, `sb.push()` is `E0207`,
`sb.pull()` is `E0217`, and `print(sb)` is `E0203` like any other non-printable
value. The methods are checked next to `bytes`' `size`/`get`/`set` in
`crates/kode-sema/src/check/expr.rs` rather than added to `std_fns.rs`, because
`std_fns` rows are typed by `StdTy` and cannot name a builder receiver; the MIR
side is a two-arm `match` in `crates/kode-mir/src/lower/data.rs`.

The runtime ABI marker moves to `kode_rt_abi_0005` (and `RT_ABI` in
`kode-cli/src/doctor.rs`), because the object kinds and the symbol list changed.

## Alternatives

- **A list of strings, joined at the end.** Zero new runtime code, and it does get
  the complexity right. Rejected because it retains every piece for as long as the
  builder lives and pays 8 bytes of pointer per push, which is exactly the cost
  the port is trying to lose; a byte buffer is the shape the output wants.
- **A class in a prelude.** Rejected above: constructor dispatch through a shipped
  library, for a type that is a language primitive.
- **`sb.add(s)` returning the builder for chaining.** Rejected as scope: chaining
  needs `push` to return `this` and changes ownership rules for no measured gain.

## Consequences

- Building text is linear. The port can write IR, dumps and diagnostics through a
  builder instead of `+=`.
- A builder is opaque: no `len()`, no indexing, no printing. That is deliberate —
  `build()` is the only way to read it, so there is one representation to keep
  true.
- The buffers it holds are freed through the ordinary `kode_release` path
  (`BUFFER` joins the drop dispatch in `object.rs`), so the leak check covers it.

## Evidence

`benchmarks/string_build/` builds the same 8,000-line IR-shaped text two ways and
prints its byte length. Recorded in `benchmarks/report.md`:

| build | median | peak RSS |
|---|---:|---:|
| `text = text + ...` | 185.2 ms | 4304 KB |
| `sb.push(...)` + `build()` | 5.2 ms | 2472 KB |

That is 36× faster at this size, and the gap grows with the text (the `+` shape is
quadratic, the builder linear). `tests/run-pass/string_builder.kode` covers an
empty builder, a loop of pushes and a string built from a variable, with no leaks;
`tests/compile-fail/e0208_builder_push.kode` pins `sb.push(1)`.

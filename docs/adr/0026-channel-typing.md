# 0026 — How channels are typed

Status: accepted

## Context

Kode has no user-written generics: `Task<T>` exists in the type system, but a
program never writes `Task<int>` because only `spawn` produces one, and it takes
the type from the function it starts. A channel is different — nothing at the
construction site says what will travel through it:

```
jobs = channel()      # Channel of what?
```

Two ways out were considered. An untyped channel (`send` accepts anything,
`recv` gives back something of unknown type) is easy to build and useless to a
type checker: every receive would need a cast, which is exactly the kind of
run-time surprise the error model exists to avoid. Inferring the payload from the
first `send` would make a variable's type depend on statement order and break the
moment two call sites disagree.

## Decision

**`Channel<T>` is a declared type, written like `Task<T>`.** The payload comes
from the declaration, the same way an empty list gets its element type from one
(E0222):

```
jobs: Channel<int> = channel()
names: Channel<string> = channel()
```

- `channel()` on its own has payload type `none`. That state is legal only until
  a declared type meets it, exactly like `[]`. Using it before then is E0240:
  `this channel has no type yet; declare it, for example `ch: Channel<int> = channel()``.
- The payload may be any type a list item may be: `int`, `float`, `bool`,
  `string`, a class, a list, a map. It is carried in one 64-bit slot. Ownership
  is **transferred** on `send`: the lowering retains a borrowed argument before
  the call, the channel stores that one reference, and `recv` hands it to the
  receiver, which releases it like any temporary. Queued values are released
  when the channel is dropped, so a channel can outlive its sender.
  (Implementing `send` with a second retain in the runtime leaked one object per
  heap payload; `tests/run-pass/channel_heap.kode` covers the transfer.)
- `send(ch, v)` does not block (the queue is unbounded in v1), `recv(ch)` blocks
  until a value or a close. `close(ch)` is idempotent. Receiving from a closed,
  empty channel and sending after a close are catchable errors, not crashes.
- A channel is a reference-counted object, so it can be closed and then dropped;
  queued values are released with it.
- Errors do not travel through a channel. A task reports failure through
  `await`, as it does today; a producer that wants to report one closes the
  channel and stores the reason elsewhere, or sends an `Error` value.
- Bounded channels (`channel(16)`) are a later addition; the size argument is
  reserved, not implemented, and `send` on a full bounded channel would block.

## Consequences

- `Type::Chan(Box<Type>)` mirrors `Type::Task`, including `name()`, `is_ref()`
  and the empty-payload rule in `assignable`.
- The parser's `Task<T>` branch in `type_ann` gains a `Channel<T>` sibling; no
  other syntax changes.
- Three new builtins (`channel`, `send`, `recv`, `close`) and four runtime
  functions. The object layout gains one kind (`CHAN`), and no existing layout
  changes, so the ABI marker stays `kode_rt_abi_0002`.
- E0240 is added to `docs/errors/index.md`.
- Covered by `tests/run-pass/channels.kode`: a producer/consumer pair over two
  channels, a close, and a receive after close that is caught.

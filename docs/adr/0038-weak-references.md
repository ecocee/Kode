# ADR 0038: `weak` references

Status: **proposed** — design and leak evidence only. No compiler or runtime
change has been made; each step below is a separate, test-first change and needs
the maintainer's approval, because it touches the memory model and the object
layout (AGENTS.md "Ask the maintainer first").
Date: 2026-11-11.
Plan item: `docs/PLAN-v1.md` 9.5 (`weak` references), `docs/STATUS.md` 1.3.
Evidence: `benchmarks/leaks/` (`run.sh`, `report.md`).

## 1. Context

Every heap value is reference counted (ADR-free by now, documented in
`docs/design/memory-model.md`). The compiler inserts `retain`/`release`; counts
cannot see a cycle, so two objects that point at each other keep each other's
count above zero and are never freed. `docs/design/memory-model.md` tells users
to "keep back-links out of the object graph or break them by hand
(`a.next = none`)", which is exactly the failure mode `weak` is meant to remove.

The rule this ADR has to satisfy is AGENTS.md: a memory-model change needs an
ADR **and** leak evidence. So the evidence comes first, measured on the current
compiler (`benchmarks/leaks/report.md`):

| program | shape | `KODE_LEAK_CHECK=1` result |
|---|---|---|
| `nocycle.kode` | two objects, back-edge cleared with `b.next = none` | `a b`, exit **0**, no leak reported |
| `cycle.kode` | two objects, `a.next = b` and `b.next = a` | `a b`, exit **3**, `kode: memory leak: 2 object(s) were never freed` |
| `ring.kode` | ring of 10,000 objects, each owning the next | `10000`, exit **3**, `kode: memory leak: 10000 object(s) were never freed` |

The leak check is the existing one (`crates/kode-rt/src/object.rs`): with
`KODE_LEAK_CHECK=1`, `kode_rt_finish` reports the live-object count and exits 3.
Nothing in the test suite covers cycles, and none of these three programs may be
added to `tests/run-pass` while the leak is real.

## 2. What `weak` is, and what it is not

A `weak` field is a **non-owning** reference. It lets a child point back at its
parent without keeping the parent alive:

```kode
class Dir {
    name: string
    parent: weak Dir? = none
    children: [Dir] = []
}

class Tree {
    root: Dir
    fn add(parent: Dir, name: string) -> Dir {
        d = Dir(name: name, parent: parent)
        parent.children.add(d)
        return d
    }
}

t = Tree(root: Dir(name: "/"))
etc = t.add(t.root, "etc")
print(etc.name, etc.parent!.name)
```

`weak` is **not** a cycle collector. A cycle built entirely from strong fields
still leaks after this ADR, exactly as `cycle.kode` and `ring.kode` do today.
What changes is that a user can now *express* a back-edge without creating the
cycle. Any claim that "9.5 fixes cycle leaks" is wrong and must not be written
in the docs; the honest statement is "9.5 makes cycles avoidable".

## 3. Syntax

`weak` is a **type modifier allowed only on class fields**:

```
FieldDecl: name ":" Type
Type      : "weak"? TypeName ("?" )?
```

- `weak Parent?` is the only well-formed use: the target must be a class type
  and it must be optional, because a weak field is always allowed to be `none`
  and reading one that has been destroyed yields `none`.
- Parsed in `crates/kode-syntax/src/parser/` where a field's type annotation is
  parsed (`parse_class` in `parser/item.rs`), so `FieldDecl` gains one bool, and
  `TypeAnn` gains `is_weak: bool`.
- Not allowed on locals, parameters, return types, list elements, map values or
  generic arguments. `weak [Dir]?`, `weak string?`, `fn f(p: weak Dir?)` and
  `x: weak Dir? = none` inside a body are compile errors.

Two codes are **reserved** here and land with the implementation, together with
their rows in `docs/errors/index.md` so `kode explain` works:

| code | condition | hint |
|---|---|---|
| `E0415` | `weak` on a type that is not an optional class | "`weak` needs a class type, like `weak Parent?`" |
| `E0416` | `weak` outside a class field | "a `weak` reference is a field; locals and parameters are normal references" |

## 4. Semantics

1. **Storing** into a weak field does not retain; **clearing** it (`field = none`)
   drops the weak link.
2. **Reading** a weak field produces an ordinary `T?` value that the reader owns.
   If the target was already destroyed, the read yields `none`. This is the only
   observable difference from a strong field, and it is why `weak` needs no new
   type in the checker: `weak T?` and `T?` read the same.
3. **The holder is unaffected** by the target's lifetime; a destroyed target
   never writes through the weak link.
4. **The target's memory outlives its destruction** as long as a weak link
   exists. "Destroyed" means: strong count reached zero, fields released, the
   instance marked dead (refcount 0). The shell is freed when the last weak link
   disappears. This is what makes `none` observable without a use-after-free.
5. **Upgrade is atomic against destruction.** In threaded programs the reader
   must not resurrect an object between "count is positive" and "retain"; see §6.

## 5. Runtime layout

Current class instance (`crates/kode-rt/src/object.rs`):

```
[rc: i64][kind: i64][refmask: i64][field0][field1]...
                         ^ kind = OBJ | class_id << 8 | nfields << 16
                         ^ size = 24 + 8 * nfields
```

Proposed:

```
[rc][kind][refmask][weakmask][weak_rc][field0][field1]...
                    ^ new      ^ new
                    size = 40 + 8 * nfields  (+16 bytes per instance)
```

- `weakmask` parallels `refmask`: bit *i* set means field *i* is a weak link.
  `drop_obj` releases strong fields through `refmask` and drops weak links
  through `weakmask` (no `release`, just a `weak_rc` decrement).
- `weak_rc` counts the weak links pointing at this instance. It lives in the
  instance, not the shared header, so strings, lists and maps pay nothing —
  `weak` is class-only by design.
- New runtime entry points, registered in `kode-mir/src/ir.rs` (`RtFn`) and
  `crates/kode-llvm/src/lower.rs` (`RT`), per AGENTS.md:

| symbol | job |
|---|---|
| `kode_weak_set(field, value)` | drop the old link, take a weak link on `value`, store the raw pointer |
| `kode_weak_get(field)` | upgrade: retain and return the pointer if `rc > 0`, else return null |
| `kode_weak_drop(field)` | clear a link (used by `drop_obj` and by assignment of `none`) |

`kode_obj_new` and `kode_obj_new_class` gain a `weakmask` argument, which is an
ABI change: bump `kode_rt_abi_0004` to `kode_rt_abi_0005` and `RT_ABI` in
`kode-cli/src/doctor.rs`, so an old embedded runtime can never be linked by
mistake.

## 6. Destruction and upgrade, precisely

`kode_release` today frees the object when the count reaches zero, in the same
call. With weak links the last strong release must instead:

1. release every strong field (`refmask`), recursively,
2. mark the instance dead by leaving `rc == 0` (it already is),
3. free the memory **only if** `weak_rc == 0`; otherwise leave the shell for the
   last weak link to free.

`kode_weak_drop` decrements `weak_rc` and frees the shell when it reaches zero
**and** `rc == 0`.

`kode_weak_get` must not be racy in threaded programs (`THREADED` is already set
by the first task or server thread). The upgrade is a compare-and-swap loop on
`rc`: succeed only if the value read is greater than zero, otherwise return
`null`. A plain load followed by `kode_retain` would be a use-after-free: the
object can be destroyed between the two.

## 7. Compiler changes

| crate | change |
|---|---|
| `kode-syntax` | `weak` token (`token.rs`), `FieldDecl.is_weak`, `TypeAnn.is_weak`, field-type parsing |
| `kode-sema` | reject `E0415`/`E0416`; record each class's weak fields for the layout and for field access |
| `kode-mir` | pass `weakmask` to the constructor; store to a weak field emits `kode_weak_set`; load emits `kode_weak_get`; `drop_obj` path uses `weakmask` |
| `kode-llvm` | `RT` entries for the three new runtime symbols |
| `kode-rt` | `object.rs` layout, `weak_rc`, the three entry points, `weakmask` in `drop_obj` |

Ownership rules do not change for strong values; a weak field is simply never
retained, which is one fewer rule to get wrong than the manual
`a.next = none` convention it replaces.

## 8. Tests (written before the implementation, per AGENTS.md)

`tests/run-pass/` (each with a `.out`, all run under `KODE_LEAK_CHECK=1`):

| file | asserts |
|---|---|
| `weak_parent.kode` | parent/child tree with a `weak` back-link; reading `child.parent!.name` works; **0 leaks** |
| `weak_after_free.kode` | child outlives parent: `child.parent` reads `none`, no crash, 0 leaks |
| `weak_clear.kode` | assigning `none` and re-pointing a weak field keeps counts right (0 leaks) |
| `weak_in_list.kode` | a list holds children, children hold `weak` parents, list is freed normally |

`tests/compile-fail/`:

| file | code |
|---|---|
| `e0415_weak_not_class.kode` + `.err` | `E0415` (`weak [int]?`) |
| `e0416_weak_parameter.kode` + `.err` | `E0416` (`fn f(p: weak Dir?)`) |

Unit tests in `crates/kode-rt/src/object.rs`:

- `weak_shell_survives_until_the_last_link_drops` — with the crate's `counting()`
  guard, the live count is unchanged after a full strong-then-weak teardown.
- `weak_get_after_drop_is_null`.
- `weak_upgrade_never_resurrects` — two threads, one upgrading in a loop while
  the other drops the last strong reference; the upgraded pointer is either a
  valid object or null, and a leak check at the end must show zero.

`benchmarks/leaks/report.md` stays the record that the **strong** cycle still
leaks after 9.5: `cycle.kode` and `ring.kode` are expected to keep exiting 3.
Only a new program using a `weak` back-edge is expected to exit 0, and that
program is added to the benchmark set when the feature lands.

## 9. Alternatives considered

- **Cycle collector (Bacon–Rajan trial deletion).** Collects real cycles, so
  `cycle.kode` and `ring.kode` would stop leaking without user changes. Rejected
  for v1.0: a collector needs a write barrier on every field store, a
  collection pass with pauses, and a second way to free objects — a much larger
  change to the one ownership model the compiler already implements, for a
  problem `weak` makes avoidable. Revisit after v1.0 if real programs still
  build strong cycles.
- **Manual breaking (`a.next = none`), the status quo.** Free, and already
  documented. It fails exactly where cycles are natural (trees, observers,
  caches): forgetting one assignment leaks the whole group — measured as 2
  objects in `cycle.kode` and 10,000 in `ring.kode`, with no error at the point
  of the mistake.
- **A raw, untracked pointer (`raw T?`).** Zero runtime cost, but reading one
  after the target is freed is a use-after-free; unacceptable as the safe
  default. Could be offered later as an explicitly unsafe escape hatch.
- **Arena / region allocation for the AST-style graphs.** The right answer for a
  compiler's own data structures, and probably what self-hosting needs
  (`docs/self-hosting.md` step 4). It is a different memory model for a
  different purpose; it does not replace a back-edge in an ordinary object
  graph.
- **Weak links that store `none` by rewriting the referrer's fields at free
  time** (eager clearing). No shell, no `weak_rc`, but freeing thread must write
  into objects it does not own, which needs a lock or an intrusive list per
  object and makes the free path unbounded. Rejected: the shell costs 8 bytes,
  eager clearing costs a synchronization design.

## 10. Costs and consequences

- **+16 bytes per class instance** (`weakmask` and `weak_rc`), nothing for
  strings, lists or maps. Binary size: no new crate, three small runtime
  functions; the binary-size effect should be measured before merging, per
  AGENTS.md rule 8.
- One CAS on a weak read; no cost for programs that never use `weak`.
- A destroyed-but-referenced object keeps its fields' memory until the last weak
  link goes; a program that keeps weak links to dead objects forever keeps their
  shells. Documented, and visible in `KODE_LEAK_CHECK` only if the shells
  outlive the program.
- `weak` does not stop strong cycles. `docs/PLAN-v1.md` and `docs/ROADMAP.md`
  used to promise "cycle-safe memory"; they now say "cycle-avoiding", because
  this design does not collect a cycle that strong fields already built.
- No change to pending/error paths, no change to how lists or maps own items.

## 11. Open questions for the maintainer

1. Approve the layout change (+16 bytes per class instance, ABI 0005) rather
   than a separate control block, or prefer a control block that keeps the
   instance at 24 bytes but adds an allocation per weakly-referenced object?
2. Should `weak` also work for interface-typed fields (`weak Shape?`), or is
   class-only enough for v1.0?
3. Is a cycle collector worth planning for after v1.0, or is `weak` plus arenas
   for the compiler's own graphs the whole answer?

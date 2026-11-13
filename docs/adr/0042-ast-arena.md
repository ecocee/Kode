# ADR 0042: the AST is an arena, not a `weak` graph

Status: **accepted**. The port's AST is a flat list of nodes with `int?` links;
`weak` is not on the S2 path.
Date: 2026-11-13.
Plan item: `docs/PLAN-self-hosting.md` WP-11 (and WP-07 by consequence),
`docs/STATUS.md` 1.3.
Related: ADR 0038 (`weak` references, still proposed), ADR 0039 (payload enums),
ADR 0040 (generic classes), ADR 0041 (optional scalar fields).

## 1. Context

S2 writes `selfhost/ast.kode` and `selfhost/parser.kode`, and the choice of AST
representation is hard to change afterwards: the parser, the dumper and the
checker all touch it. The plan left two candidates open: an arena, or `weak`
parent links (ADR 0038).

The AST is a tree with back-edges. A parser needs the child list; later passes and
diagnostics want a parent and a sibling. With plain reference counting, `parent`
as an owning field is a cycle: every node keeps its parent alive, the parent keeps
its children, and nothing is ever freed — exactly the leak `benchmarks/leaks`
measures (10,000 nodes, exit 3). `weak` would make the up edge non-owning, at the
cost described in ADR 0038 §5: a new object header (`weakmask`, `weak_rc`, +16
bytes per instance), three new runtime symbols, an atomic upgrade path for
threads, a `weak` type modifier in the lexer and parser, two reserved codes
(`E0415`, `E0416`), and a memory-model change that AGENTS.md requires the
maintainer to approve. That is the largest single item on the S2 list, and S2
would be paying it to solve a problem it does not have to create.

In the year since ADR 0038 was written the language gained what an arena needs:
lists of class instances, payload enums with positional bindings (ADR 0039),
`is` narrowing, generic classes and methods (ADR 0040), and optional numbers as
class fields and parameters (ADR 0041). A node id can therefore be an `int?`
field, and "no parent" can be `none` rather than a sentinel.

## 2. Decision

The AST is **one flat arena of nodes**, owned by a single `Ast` object. Every edge
is an index, never a reference:

```kode
enum Kind {
    Root
    Dir(name: string)
    File(name: string)
}

class Node {
    kind: Kind        // payload enum: the node's shape and its data
    span_start: int
    span_end: int
    parent: int?      // none for the root; never a reference
    first_child: int?
    next_sibling: int?
}

class Ast {
    nodes: [Node]

    fn add(kind: Kind, parent: int?) -> int {
        nodes.add(Node(kind: kind, parent: parent))
        return nodes.len() - 1
    }
}
```

`selfhost/spec/gates/wp11-arena.kode` is this shape compiled and run by the gate
harness.

1. **One owner.** `Ast` owns `nodes`; releasing the `Ast` releases every node. The
   compiler's leak check therefore has one AST-sized unit to account for, and the
   pointer graph inside the arena is acyclic by construction.
2. **Cycles are not expressible.** An edge is an `int?`; an `int` cannot own a
   node, so neither a parent link nor a sibling link can create a cycle. This is a
   property of the representation, not a discipline the passes must remember.
3. **Ids are stable and dense.** `add` appends and returns `nodes.len() - 1`, so an
   id is its position, ids are never reused, and an id is a good map key (the
   checker's per-expression tables can be lists indexed the same way, not maps
   hashed by object identity).
4. **Access is checked.** `ast.nodes[id]` bounds-checks; a bad id is a language
   runtime error with a line, not memory corruption. S2 keeps the accessor small
   (`nodes[id]`) so a later inlining of bounds checks stays a local change.
5. **Spans are values on the node.** `span_start`/`span_end` are byte offsets, as
   S1 established; there is no span object per node and therefore no second
   lifetime to reason about.
6. **`weak` stays proposed.** ADR 0038 is unchanged and still worth doing for user
   code — it makes cycles avoidable for anyone building a graph. It is simply not
   a prerequisite for the port, and the plan's WP-07 probe stays red on purpose so
   the gate table keeps showing the open language feature rather than hiding it.

## 3. Alternatives

- **`weak` parent links.** The plan's original pairing. It solves the cycle for
  user graphs and would let the AST use real references end to end. Rejected for
  S2: it is a memory-model and object-layout change with new runtime symbols,
  threading-visible upgrade semantics and two new codes, and it still leaves node
  lifetime as per-node reference counting, where the arena has one owner and no
  counts at all. It is the right change for a different problem, later.
- **A bare RC tree with no back-edges.** No arena, no `weak`: children own their
  nodes and every pass that needs an ancestor takes it as a parameter. Cheapest on
  the runtime, but the checker and the diagnostic rendering genuinely need "where
  did this node come from", and threading a parent stack through every visitor is
  the kind of duplication S2 exists to avoid; it also makes node identity
  unstable for tables.
- **A runtime bump allocator with raw pointers.** What a systems compiler would
  do; Kode has no raw pointers and no `unsafe`-style escape, and adding one is a
  language change larger than `weak`. Rejected.

## 4. Consequences

- An AST access costs an index, a bounds check and one extra indirection. No
  performance claim is made here; when the parser lands, `benchmarks/report.md`
  gets Stage 0 versus Stage 1 numbers before anything is called fast (AGENTS.md
  rule 8).
- The arena grows by appending; nothing is ever removed, so a "delete this node"
  pass is not possible and is not wanted — the parser is append-only.
- Structure-changing passes must go through `Ast` methods, which is where the
  invariants (parent set once, siblings appended in order) can live.
- The AST dump and the leak check both walk `nodes` in id order, which makes their
  output deterministic without sorting.
- The port no longer needs `weak` before S3, so WP-07 moves off the critical path;
  the S2 gate becomes WP-08/WP-09/WP-10 plus this probe.
- If a future pass turns out to want object identity (a set of nodes), it can use
  a `[bool]` parallel to `nodes` indexed by id instead of a set of references.

## 5. Gate

`selfhost/spec/gates/wp11-arena.kode` builds a three-node tree with a payload-enum
kind, `int?` parent links and one owning list, walks it, and prints the parent of
each node. It compiles, runs, and is leak-free under `KODE_LEAK_CHECK=1` on Stage 0
today, which is what "the representation is expressible" means for this decision.

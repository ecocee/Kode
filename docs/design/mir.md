# MIR

MIR is a control-flow graph. A function is a list of **locals** and **basic blocks**.
A block is a list of simple instructions and one terminator.

```
fn fib(_0: int) -> int {
    let _1: bool
  bb0:
    _1 = lt _0, 2
    branch _1, bb1, bb2
  bb1:
    return _0
  bb2:
    ...
}
```

Print it with `kode mir file.kode`.

| Instruction | Meaning |
|---|---|
| `dst = use x`, unary, binary, cast | arithmetic and moves on locals and constants |
| `dst = call f(args)` | call a user function or a runtime function (`rt::...`) |
| `dst = load base+offset` / `store base+offset, x` | read and write a field of a class instance |
| `retain x` / `release x` | change the owner count of a heap value |

| Terminator | Meaning |
|---|---|
| `goto bb` | jump |
| `branch c, bbA, bbB` | conditional jump |
| `return [x]` | leave the function |
| `unreachable` | cannot happen (after a diverging call) |

`and`, `or`, `??`, `if`, `while`, `for`, `match`, `try` / `catch` and all error handling are lowered to blocks and jumps here.
A value of type `ref` is any reference-counted heap value.
`kode_mir::verify` checks the graph after lowering; a failure is a compiler bug.

Why a MIR: memory management, control flow and (later) optimizations are far easier on
simple blocks than on a tree, and any backend (LLVM today, WebAssembly later) can read it.

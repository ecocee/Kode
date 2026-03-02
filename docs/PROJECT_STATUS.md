# Kode — Project Status (snapshot)

Generated: March 1, 2026

## Short status
- Build and CLI are functional. The project now targets a single runtime: the bytecode VM. The old tree-walking runtime and Go/LLVM backends have been removed.
- You can build and run source and bytecode: `kode run <file>.kode` and `kode <file>.kbc`.

## Implemented features
- Frontend: lexer & parser producing `ast.Program` supporting `let`, `func` (expr/block bodies), `if`, `for`, `while`, arrays, calls, member access, structs, enums.
- Type checking: basic inference, constraint solving, function hoisting, built-ins (`print`, `input`) in `internal/typer`.
- IR: simplified IR with functions, blocks, and instructions (`pkg/ir`).
- Backend: bytecode emitter (`pkg/bytecode/compiler.go`) producing `Program{Instructions, Constants, Globals}` and serializer/deserializer (`pkg/bytecode/bytecode.go`).
- VM: stack-based VM (`pkg/bytecode/vm.go`) implementing arithmetic, logic, arrays, structs/enums, builtins, and basic control flow.
- CLI: `build`, `run`, direct-file execution; `exec` and non-VM backends removed.

## Architecture highlights
- Clear pipeline: Source → Lexer → Parser → AST → Typer → IR → Bytecode → VM.
- Bytecode format is self-contained (constants + globals + instructions) and serializable to `.kbc` files.
- VM uses Go `interface{}` values, native Go memory and GC.

## Limitations & gaps
- Function call frames are not fully implemented: multi-frame call stack, saved `pc`/`bp`, and proper return handling are limited; many user functions are inlined instead.
- `OpReturn`/`OpReturnValue` and loop control (`OpBreak`/`OpContinue`) behavior is basic and may not handle nested frames or complex control flow robustly.
- Concurrency constructs exist in AST but are not implemented at VM level (no scheduler/channels runtime).
- Runtime diagnostics are limited: stack traces mapping bytecode to source are minimal.
- Performance: boxed `interface{}` values and simple VM design are not optimized for speed.
- Module loading/linking and a richer standard library are incomplete.

## Short list of recommended next steps
1. Implement full call frames: push frame, save `pc`/`bp`, allocate locals, restore on return.
2. Improve loop control semantics and maintain a loop-target stack for `break`/`continue`.
3. Add source mapping from bytecode PCs to source lines and improve runtime errors/stack traces.
4. Expand builtin/stdlib and add `.kbc` module loading/linking.
5. Add targeted tests for recursion, nested calls, and error reporting.

## Where to look in the repo
- Parser: `internal/parser/parser.go`
- AST: `pkg/ast/ast.go`
- Typer: `internal/typer/typer.go`
- Compiler (AST→IR): `internal/compiler/compiler.go`
- IR: `pkg/ir/ir.go`
- Bytecode emitter & format: `pkg/bytecode/compiler.go`, `pkg/bytecode/bytecode.go`
- VM: `pkg/bytecode/vm.go`
- CLI: `internal/cli/` and `cmd/kode/main.go`

---

If you want, I can now:
- Produce an annotated bytecode dump for `examples/basic.kode` (constants, globals, instruction list), or
- Start implementing proper call-frame support in `pkg/bytecode/vm.go` (I will make a focused patch and run tests), or
- Add source ↔ bytecode mapping and minimal runtime stack traces.

Which should I do next?

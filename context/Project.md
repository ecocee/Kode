# KODE — Development Status, Roadmap & TODO

**Project:** KODE programming language  
**Version:** 0.3.0  
**Current implementation language:** Rust  
**Current development platform:** macOS  
**Intended deployment platforms:** Linux and macOS  
**Native backend direction:** Cranelift  
**Document status:** Working roadmap — update as implementation changes

---

## 1. Project vision

KODE is a general-purpose programming language being developed in Rust.

The long-term goal is to create a practical, high-performance language suitable for:

- Backend applications and APIs
- Systems and infrastructure software
- Networking and database-related applications
- Command-line tools
- Production software
- Eventually, cross-platform native applications

KODE should have its own syntax, compiler, type system, standard library, runtime and developer tooling. It should not remain only a syntax layer over Rust or an interpreter.

### Design principles

- Clear, readable syntax.
- Predictable language semantics.
- Strong static typing as the language matures.
- Native compilation for performance.
- A modular compiler architecture.
- Useful diagnostics and developer tools.
- Cross-platform support, initially macOS and Linux.
- Keep the language implementation in Rust.
- Keep the KODE IR independent from Cranelift so other backends can be added later.

---

## 2. Current development status

### 2.1 Confirmed project state

The following is based on the source code and terminal output shared so far.

| Area | Current status | Notes |
|---|---|---|
| Rust project | Working | `cargo check` succeeds |
| CLI entry point | Implemented | `src/main.rs` |
| Lexer | Present | `lexer::lex()` is called |
| Parser | Present | `parser::parse()` is called |
| AST | Present | Statements and expressions are represented |
| Interpreter | Present | `kode run` executes through Rust interpreter |
| Syntax checking | Implemented | `kode check` runs lexer and parser |
| KODE IR | Initial structure exists | `src/compiler/ir.rs` |
| Cranelift dependencies | Installed | Version 0.125.4 |
| AST-to-IR lowering | Not implemented/connected yet | Needs implementation |
| Cranelift code generation | Not implemented/connected yet | Dependencies alone do not compile KODE |
| Object-file generation | Not implemented/connected yet | Future backend milestone |
| Native executable linking | Not implemented/connected yet | Requires system linker integration |
| Static type checker | Not confirmed | Must be designed and integrated |
| Standard library | Not confirmed | Needs a defined design |
| Package manager | Not implemented | Long-term item |
| Automated language test suite | Not confirmed | Must be established |

### 2.2 Current CLI

The current `src/main.rs` supports:

```bash
kode --version
kode check hello.kode
kode run hello.kode
```

Current execution flow:

```text
KODE source file
      |
      v
    Lexer
      |
      v
    Parser
      |
      v
     AST
      |
      +----------------------+
      |                      |
      v                      v
   check                 Interpreter
                             |
                             v
                       Program output
```

**Important:** `kode run` currently uses the Rust interpreter. It does not use Cranelift.

### 2.3 Cranelift dependencies

The project successfully installed these dependencies at version `0.125.4`:

- `cranelift-codegen`
- `cranelift-frontend`
- `cranelift-module`
- `cranelift-object`

The successful `cargo check` and `cargo tree | grep cranelift` output confirm that the dependencies are present.

They do **not** confirm that KODE generates native machine code yet.

### 2.4 Current compiler architecture

The initial KODE IR is in:

```text
src/compiler/ir.rs
```

The module is declared from `src/main.rs`:

```rust
mod compiler;
```

The IR currently has foundational structures such as:

- `ValueId`, `BlockId`, `FunctionId`
- `Type`
- `Constant`
- `BinaryOp`, `UnaryOp`
- `Value`, `Parameter`
- `InstructionKind`, `Instruction`
- `Terminator`
- `BasicBlock`
- `Function`, `Module`
- `Builder`

This is an initial foundation, not yet a complete or verified compiler IR. It needs validation rules, proper control-flow representation and integration with the AST.

---

## 3. Current source layout

The confirmed files include:

```text
kode/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── ast.rs
    ├── diagnostics/
    ├── lexer/
    ├── parser/
    ├── runtime/
    │   ├── mod.rs
    │   ├── value.rs
    │   └── interpreter.rs
    └── compiler/
        ├── mod.rs
        └── ir.rs
```

The exact contents of `Cargo.toml`, `compiler/mod.rs`, and the full current IR should be checked before implementing the backend. Do not assume that future modules already exist.

---

## 4. Language syntax direction

The intended syntax direction is:

### Variables

```kode
name = "KODE"
age = 23

val language = "KODE"
```

- `name = ...` declares or assigns a mutable variable, according to the final scope rules.
- `val` declares an immutable variable.

### Output

```kode
print("Hello, KODE!")
print(10 + 20)
```

Parentheses are required for `print`.

### Comments

```kode
// Single-line comment

/*
   Multi-line comment
*/
```

### Conditions

```kode
if age >= 18 {
    print("Adult")
} else {
    print("Minor")
}
```

### Loops

```kode
for i in 0..10 {
    print(i)
}

for i in 1..=5 {
    print(i)
}

while condition {
    print("Running")
}
```

### Functions

Current intended direction:

```kode
fn add(a: int, b: int) -> int {
    return a + b
}

result = add(10, 20)
print(result)
```

Typed parameters and return types are a future parser, AST, type-checker and compiler task unless they have since been implemented.

### Arrays

```kode
numbers = [10, 20, 30]
print(numbers[0])
```

### Basic types

Initial type mapping proposal:

| KODE type | Intended representation |
|---|---|
| `int` | `i64` |
| `uint` | `u64` |
| `small` | `i32` |
| `byte` | `u8` |
| `float` | `f64` |
| `bool` | Boolean |
| `string` | String |

This is a proposed language design, not a claim that all these types are currently implemented.

---

# 5. Development TODO

Priority guide:

- **P0 — Required next:** blocks the first native compiler milestone.
- **P1 — Core language:** required for a useful compiled language.
- **P2 — Production maturity:** reliability, tooling and ecosystem.
- **P3 — Long-term:** advanced capabilities.

## Phase 0 — Stabilise the current project

- [x] Create Rust project.
- [x] Implement initial lexer.
- [x] Implement initial parser.
- [x] Create AST representation.
- [x] Implement initial interpreter.
- [x] Add `kode check`.
- [x] Add `kode run`.
- [x] Add `kode --version`.
- [x] Create initial KODE IR structures.
- [x] Add Cranelift dependencies.
- [x] Confirm `cargo check` succeeds with Cranelift dependencies.
- [ ] Add a `README.md` describing the actual current features.
- [ ] Add a `tests/` directory and baseline language tests.
- [ ] Add a compiler/interpreter error test suite.
- [ ] Record the exact supported syntax in a language specification.
- [ ] Resolve existing compiler warnings by using the code or removing genuinely unnecessary structures; do not hide warnings globally.

## Phase 1 — Complete the KODE IR

**Priority: P0**

- [ ] Review the existing `src/compiler/ir.rs`.
- [ ] Define and document the IR type system.
- [ ] Define a consistent function and module representation.
- [ ] Define basic-block ownership and control-flow rules.
- [ ] Add block parameters or another explicit SSA merge mechanism.
- [ ] Define how variables map to IR values and storage.
- [ ] Define function parameters and return values.
- [ ] Define integer and floating-point constants.
- [ ] Define arithmetic and comparison instructions.
- [ ] Define boolean operations.
- [ ] Define calls and returns.
- [ ] Define memory operations.
- [ ] Define branches and jumps.
- [ ] Add an IR verifier.
- [ ] Verify that every block has a valid terminator.
- [ ] Verify that referenced blocks and values exist.
- [ ] Verify that types match instruction requirements.
- [ ] Add readable IR printing/dumping for debugging.
- [ ] Add unit tests for IR construction and verification.

**Completion condition:** KODE can represent and verify simple functions and control flow without depending on Cranelift APIs.

## Phase 2 — AST-to-IR lowering

**Priority: P0**

Create:

```text
src/compiler/lower.rs
```

Tasks:

- [ ] Lower integer literals.
- [ ] Lower floating-point literals.
- [ ] Lower boolean literals.
- [ ] Lower variables.
- [ ] Lower variable declarations and assignments.
- [ ] Lower arithmetic operators.
- [ ] Lower comparison operators.
- [ ] Lower unary operators.
- [ ] Lower `print`.
- [ ] Lower function definitions.
- [ ] Lower function calls.
- [ ] Lower return statements.
- [ ] Lower `if` / `else`.
- [ ] Lower `while`.
- [ ] Lower `for` ranges.
- [ ] Lower `break` and `continue`.
- [ ] Add source-location tracking to IR instructions for diagnostics.
- [ ] Add tests comparing expected IR with generated IR.

**Completion condition:** A supported KODE AST can be converted into valid KODE IR without executing the program.

## Phase 3 — First Cranelift backend

**Priority: P0**

Suggested structure:

```text
src/compiler/
├── mod.rs
├── ir.rs
├── lower.rs
└── cranelift/
    ├── mod.rs
    ├── codegen.rs
    └── object.rs
```

Tasks:

- [ ] Confirm the exact Cranelift dependency versions in `Cargo.toml`.
- [ ] Create a Cranelift module and target configuration.
- [ ] Map KODE integer types to Cranelift types.
- [ ] Map KODE boolean types.
- [ ] Map KODE floating-point types.
- [ ] Translate KODE constants.
- [ ] Translate arithmetic instructions.
- [ ] Translate comparison instructions.
- [ ] Translate basic blocks and branches.
- [ ] Translate function parameters and returns.
- [ ] Implement function calls.
- [ ] Define the initial native runtime/ABI.
- [ ] Implement output support for `print`.
- [ ] Generate an object file.
- [ ] Link the object file using the host system linker.
- [ ] Report backend and linker errors clearly.
- [ ] Add tests for generated native programs.

**First native test program:**

```kode
a = 10
b = 20
result = a + b
print(result)
```

Expected output:

```text
30
```

**Completion condition:** KODE source is lowered to KODE IR, translated into Cranelift IR, emitted as a native object file and linked into an executable that runs correctly.

## Phase 4 — Add the native build command

**Priority: P0**

Update the CLI to support:

```bash
kode check hello.kode
kode run hello.kode
kode build hello.kode
```

Tasks:

- [ ] Preserve the current interpreter command.
- [ ] Add `build` command parsing.
- [ ] Add output path handling, for example `-o hello`.
- [ ] Add target selection.
- [ ] Add debug and release build modes.
- [ ] Add clear compiler-stage errors.
- [ ] Ensure failed builds do not leave misleading output executables.
- [ ] Add an integration test that builds and runs a native program.

**Completion condition:** `kode build hello.kode -o hello` creates a runnable native executable.

## Phase 5 — Static semantics and type checking

**Priority: P1**

Create a dedicated semantic-analysis layer, for example:

```text
src/semantic/
    mod.rs
    symbols.rs
    types.rs
    checker.rs
```

Tasks:

- [ ] Define lexical scope rules.
- [ ] Define variable declaration rules.
- [ ] Enforce immutable variable rules.
- [ ] Detect undefined variables.
- [ ] Detect duplicate declarations where prohibited.
- [ ] Define type inference policy.
- [ ] Implement explicit type annotations.
- [ ] Check assignment compatibility.
- [ ] Check arithmetic operand types.
- [ ] Check comparison operand types.
- [ ] Check boolean conditions.
- [ ] Check function argument count and types.
- [ ] Check function return types.
- [ ] Detect invalid return statements.
- [ ] Validate `break` and `continue` placement.
- [ ] Define integer overflow behaviour.
- [ ] Define numeric conversion rules.
- [ ] Add semantic diagnostics with source locations.

**Completion condition:** Invalid programs are rejected before code generation with useful errors.

## Phase 6 — Complete core language features

**Priority: P1**

### Functions

- [ ] Typed parameters.
- [ ] Typed return values.
- [ ] Void/unit return type.
- [ ] Recursion.
- [ ] Nested scopes.
- [ ] Function overloading decision.
- [ ] Closures decision.

### Data structures

- [ ] Arrays.
- [ ] Array mutation.
- [ ] Array bounds checking.
- [ ] Strings.
- [ ] Struct definitions.
- [ ] Struct construction.
- [ ] Struct field access.
- [ ] Struct field mutation.
- [ ] Enums.
- [ ] Tuples decision.
- [ ] Classes decision.
- [ ] Methods.
- [ ] Generics design.

### Control flow

- [ ] Nested conditions.
- [ ] Nested loops.
- [ ] Correct `break` and `continue` semantics.
- [ ] Range iteration.
- [ ] Match/switch expression design.
- [ ] Error-handling design.

### Operators

- [ ] Complete arithmetic operators.
- [ ] Complete comparison operators.
- [ ] Logical operators.
- [ ] Assignment operators.
- [ ] Bitwise operators.
- [ ] Operator precedence specification.
- [ ] Short-circuit semantics.

**Design note:** Prefer a simple first implementation. Do not add inheritance, complex generics or advanced metaprogramming before the core language and compiler are stable.

## Phase 7 — Runtime and standard library

**Priority: P1**

Suggested structure:

```text
src/
├── runtime/
│   ├── mod.rs
│   ├── value.rs
│   ├── memory.rs
│   └── builtins.rs
└── stdlib/
    ├── mod.rs
    ├── io/
    ├── string/
    ├── collections/
    ├── math/
    ├── filesystem/
    ├── networking/
    └── time/
```

Tasks:

- [ ] Decide which features are compiler intrinsics and which belong to the standard library.
- [ ] Define the native calling convention and ABI.
- [ ] Define string representation and ownership.
- [ ] Define array and collection representation.
- [ ] Define memory allocation strategy.
- [ ] Decide ownership/borrowing model or another memory-safety strategy.
- [ ] Define error handling.
- [ ] Implement basic input/output.
- [ ] Implement string operations.
- [ ] Implement collections.
- [ ] Implement math functions.
- [ ] Implement filesystem operations.
- [ ] Implement time utilities.
- [ ] Plan networking APIs.
- [ ] Document runtime compatibility.

Do not assume the interpreter's `Value` representation is automatically suitable for native execution. The native runtime needs its own deliberate representation and ABI.

## Phase 8 — Diagnostics and developer experience

**Priority: P1**

- [ ] Standardise lexer, parser, semantic and backend errors.
- [ ] Show filename, line and column.
- [ ] Show source snippets and error locations.
- [ ] Add helpful suggestions where practical.
- [ ] Add warning support.
- [ ] Add compiler diagnostic codes.
- [ ] Add `kode fmt` design.
- [ ] Add `kode doc` design.
- [ ] Add readable compiler output.
- [ ] Add optional IR dump.
- [ ] Add optional Cranelift IR dump.
- [ ] Add verbose build mode.

Example desired diagnostic:

```text
error[KD1001]: undefined variable `total`
  --> main.kode:4:7
   |
 4 | print(total)
   |       ^^^^^
   |
help: declare `total` before using it
```

## Phase 9 — Cross-platform native compilation

**Priority: P2**

Initial platforms:

- [ ] macOS on Apple Silicon.
- [ ] macOS on Intel, if supported by the selected toolchain.
- [ ] Linux x86-64.
- [ ] Linux ARM64.

Tasks:

- [ ] Separate host platform from compilation target.
- [ ] Define target triples.
- [ ] Add target-specific linker selection.
- [ ] Test object formats and linking.
- [ ] Test runtime behaviour on each target.
- [ ] Add cross-compilation documentation.
- [ ] Add CI builds for supported targets.

Potential later targets:

- [ ] Windows.
- [ ] WebAssembly.
- [ ] Embedded targets, if the language/runtime design supports them.

## Phase 10 — Tooling and package ecosystem

**Priority: P2**

- [ ] Design `kode.toml`.
- [ ] Add project initialisation: `kode new`.
- [ ] Add package/dependency management.
- [ ] Add lockfile.
- [ ] Add module and import system.
- [ ] Add local package support.
- [ ] Add build profiles.
- [ ] Add a test runner: `kode test`.
- [ ] Add formatter: `kode fmt`.
- [ ] Add language server protocol support.
- [ ] Add editor syntax highlighting.
- [ ] Add debugger strategy.
- [ ] Add documentation generator.
- [ ] Define package registry strategy.

## Phase 11 — Performance and optimisation

**Priority: P2**

- [ ] Establish interpreter benchmarks.
- [ ] Establish native compilation benchmarks.
- [ ] Benchmark startup time.
- [ ] Benchmark compile time.
- [ ] Benchmark runtime performance.
- [ ] Benchmark memory usage.
- [ ] Add optimisation levels.
- [ ] Add constant folding.
- [ ] Add dead-code elimination.
- [ ] Add common-subexpression elimination where appropriate.
- [ ] Add inlining strategy.
- [ ] Add profile-guided optimisation research.
- [ ] Compare KODE against equivalent Rust and C programs using fair, reproducible tests.

Do not advertise performance superiority without repeatable benchmarks and equivalent workloads.

## Phase 12 — Compiler architecture expansion

**Priority: P3**

Keep KODE IR independent from the machine-code backend.

Potential architecture:

```text
                 KODE SOURCE
                      |
                    Lexer
                      |
                    Parser
                      |
                     AST
                      |
             Semantic Analysis
                      |
                   KODE IR
                      |
          +-----------+-----------+
          |           |           |
          v           v           v
       Cranelift    LLVM         WASM
       backend     backend      backend
          |           |           |
          v           v           v
       Native      Native       Web/
       machine     machine      WASM
       code        code         module
```

Tasks:

- [ ] Stabilise KODE IR before adding another backend.
- [ ] Define backend capability requirements.
- [ ] Research LLVM backend feasibility.
- [ ] Research WebAssembly backend feasibility.
- [ ] Keep backend-specific code outside the language frontend.
- [ ] Add backend conformance tests.

Cranelift is the planned first backend. LLVM is a possible later backend, not a requirement for the first native compiler.

## Phase 13 — Security, reliability and production readiness

**Priority: P2/P3**

- [ ] Define memory safety guarantees.
- [ ] Define unsafe/foreign-function interface policy.
- [ ] Validate array and pointer operations.
- [ ] Define stack and recursion limits where needed.
- [ ] Review integer overflow and division-by-zero behaviour.
- [ ] Review file and network APIs.
- [ ] Fuzz lexer and parser.
- [ ] Fuzz IR verifier.
- [ ] Add regression tests for every fixed bug.
- [ ] Add dependency auditing.
- [ ] Add reproducible release builds.
- [ ] Document supported platforms and limitations.
- [ ] Define language and compiler version compatibility.

---

# 6. Recommended immediate implementation order

Do not implement every feature at once. Follow this sequence:

1. [ ] Inspect and stabilise the current IR.
2. [ ] Add an IR verifier and tests.
3. [ ] Implement AST-to-IR lowering for integer literals, variables and arithmetic.
4. [ ] Implement the first Cranelift backend.
5. [ ] Generate an object file.
6. [ ] Link a native executable using the macOS system linker.
7. [ ] Implement native integer output.
8. [ ] Add `kode build`.
9. [ ] Test the native executable independently of the interpreter.
10. [ ] Add functions and control flow.
11. [ ] Add semantic analysis and static type checking.
12. [ ] Expand the runtime and standard library.
13. [ ] Add Linux builds and CI.
14. [ ] Add package manager and developer tooling.

---

# 7. First native compiler acceptance tests

The first backend should support only a deliberately small subset.

### Test 1 — Integer constants and output

```kode
print(42)
```

Expected:

```text
42
```

### Test 2 — Arithmetic

```kode
a = 10
b = 20
print(a + b)
```

Expected:

```text
30
```

### Test 3 — Operator precedence

```kode
print(2 + 3 * 4)
```

Expected:

```text
14
```

### Test 4 — Variables and reassignment

```kode
count = 10
count = count + 1
print(count)
```

Expected:

```text
11
```

### Test 5 — Comparison and conditional

```kode
if 10 < 20 {
    print(1)
} else {
    print(0)
}
```

Expected:

```text
1
```

### Test 6 — Function call

```kode
fn add(a: int, b: int) -> int {
    return a + b
}

print(add(10, 20))
```

Expected:

```text
30
```

Each test must run through the native build path, not silently fall back to the interpreter.

---

# 8. CLI target

The intended CLI can grow towards:

```bash
# Version
kode --version

# Validate source
kode check main.kode

# Interpret source
kode run main.kode

# Compile native executable
kode build main.kode

# Choose output name
kode build main.kode -o app

# Run tests
kode test

# Create a project
kode new my_project

# Format source
kode fmt

# Show compiler information
kode info
```

Only `--version`, `check` and `run` are confirmed in the current `main.rs`. The other commands are planned.

---

# 9. Architecture rules

Keep these rules throughout development:

1. Do not put the entire compiler into `main.rs`.
2. Keep lexer, parser, AST, semantic analysis, IR, backend and runtime separate.
3. Do not make the KODE IR depend directly on Cranelift types.
4. Keep the interpreter available as a separate execution mode.
5. Do not silently execute interpreted code when native compilation fails.
6. Do not claim a feature is supported until it has tests.
7. Do not claim native compilation until a generated executable runs independently.
8. Keep source locations available for diagnostics.
9. Define language semantics before implementing complicated features.
10. Prefer small, testable compiler milestones over a large rewrite.
11. Keep dependency versions aligned and tested.
12. Use reproducible benchmarks before making performance claims.

---

# 10. Current known gaps and decisions

These points still need explicit confirmation or design decisions:

- [ ] Full current `Cargo.toml` configuration.
- [ ] Full current `src/compiler/mod.rs`.
- [ ] Full current `src/compiler/ir.rs` review.
- [ ] Exact lexer and parser feature coverage.
- [ ] Exact interpreter feature coverage.
- [ ] Whether semantic/type checking already exists elsewhere.
- [ ] Variable declaration and shadowing rules.
- [ ] Function type and return semantics.
- [ ] String and array memory representation.
- [ ] Memory management strategy.
- [ ] Structs versus classes and their final semantics.
- [ ] Module/import system.
- [ ] Error-handling model.
- [ ] Native runtime and ABI.
- [ ] Supported target triples and linker requirements.

---

# 11. Definition of a successful first native release

KODE reaches its first native compiler milestone when all of the following are true:

- [ ] A `.kode` source file can be parsed.
- [ ] The AST can be lowered to verified KODE IR.
- [ ] The Cranelift backend can translate the supported IR.
- [ ] The compiler emits a valid native object file.
- [ ] The system linker creates an executable.
- [ ] The executable runs without invoking the KODE interpreter.
- [ ] Arithmetic and output tests pass.
- [ ] Build failures produce useful diagnostics.
- [ ] The same source produces repeatable results.

**First release scope:** a small, reliable native-compiled subset is better than a large list of partially working language features.

---

## 12. Progress log

Record completed milestones here.

| Date | Milestone | Result |
|---|---|---|
| 2026-10-31 | Rust project and current CLI checked | `cargo check` succeeds |
| 2026-10-31 | Cranelift 0.125.4 dependencies installed | Confirmed using `cargo tree` |
| 2026-10-31 | Initial KODE IR exists | Not yet connected to code generation |
| 2026-10-31 | Interpreter execution confirmed | `kode run` uses Rust interpreter |

Add future entries with the exact command, test and result.

---

**Current overall status:** KODE has a working early-stage Rust-based language frontend and interpreter, plus an initial IR and installed Cranelift dependencies. The next major engineering task is to connect AST → KODE IR → Cranelift → object file → system linker, then prove that a standalone native executable works.
"""
path = Path("/mnt/data/KODE_DEVELOPMENT_TODO.md")
path.write_text(content, encoding="utf-8")
print(f"Created: {path}")
print(f"Size: {path.stat().st_size} bytes")
from pathlib import Path

content = r"""# KODE — Development Status, Roadmap & TODO

**Version:** 0.3.0  
**Implementation language:** Rust  
**Current development platform:** macOS  
**Planned native backend:** Cranelift 0.125.4  
**Status:** Working roadmap based on the code and terminal output shared on 31 October 2026.

---

## 1. Vision

KODE is a general-purpose programming language implemented in Rust. Its long-term goal is to support backend applications, APIs, networking, database-related software, command-line tools, systems software and production applications.

Core principles:
- Readable, consistent syntax.
- A well-defined type system and language semantics.
- Native compilation and predictable performance.
- Modular compiler architecture.
- Useful diagnostics and developer tooling.
- Cross-platform support, initially macOS and Linux.
- KODE's intermediate representation (IR) should remain independent of any one machine-code backend.

## 2. Current development status

This status is based only on files and terminal output shared so far. Items marked “not confirmed” must be inspected before being treated as implemented.

| Component | Status | Details |
|---|---|---|
| Rust project | Working | `cargo check` succeeds |
| CLI | Implemented | `src/main.rs` |
| Lexer | Present | `lexer::lex()` |
| Parser | Present | `parser::parse()` |
| AST | Present | `src/ast.rs` |
| Interpreter | Implemented | `kode run` uses Rust interpreter |
| Syntax check | Implemented | `kode check` |
| KODE IR | Initial foundation | `src/compiler/ir.rs` |
| Cranelift dependencies | Installed | Version 0.125.4 |
| AST-to-IR lowering | Not connected/implemented yet | Next compiler stage |
| Cranelift backend | Not connected/implemented yet | Dependencies alone do not compile KODE |
| Object-file generation | Not implemented yet | Backend milestone |
| Native executable linking | Not implemented yet | Requires system linker integration |
| Static semantic/type checker | Not confirmed | Needs review/design |
| Standard library | Not confirmed | Needs design |
| Package manager | Not implemented | Long-term |
| Automated test suite | Not confirmed | Must be established |

### Current CLI

Confirmed commands in the supplied `main.rs`:

```bash
kode --version
kode check hello.kode
kode run hello.kode
```

Current execution flow:

```text
KODE source -> Lexer -> Parser -> AST
                              |
                   +----------+----------+
                   |                     |
                   v                     v
                 check              Interpreter
                                         |
                                         v
                                    Program output
```

**Important:** `kode run` currently executes through the Rust interpreter. Cranelift is installed but is not yet in the execution pipeline.

### Cranelift dependencies

Confirmed in `cargo tree`:

- `cranelift-codegen v0.125.4`
- `cranelift-frontend v0.125.4`
- `cranelift-module v0.125.4`
- `cranelift-object v0.125.4`

This confirms dependency installation, not native code generation.

### Initial IR

`src/compiler/ir.rs` contains foundational types including:

- `ValueId`, `BlockId`, `FunctionId`
- `Type`, `Constant`
- `BinaryOp`, `UnaryOp`
- `Value`, `Parameter`
- `InstructionKind`, `Instruction`
- `Terminator`, `BasicBlock`
- `Function`, `Module`, `Builder`

The IR is an initial foundation. It still needs verification, control-flow/SSA design, tests and AST lowering.

## 3. Current project layout

Confirmed or previously shared structure:

```text
kode/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── ast.rs
    ├── diagnostics/
    ├── lexer/
    ├── parser/
    ├── runtime/
    │   ├── mod.rs
    │   ├── value.rs
    │   └── interpreter.rs
    └── compiler/
        ├── mod.rs
        └── ir.rs
```

Check the actual repository before creating or renaming files. The full current `Cargo.toml`, `compiler/mod.rs` and `compiler/ir.rs` have not yet been reviewed in this conversation.

## 4. Intended language syntax

These are design targets; not every feature is confirmed as implemented.

### Variables

```kode
name = "KODE"
age = 23

val language = "KODE"
```

Intended: ordinary assignment/declaration is mutable; `val` is immutable. Final declaration and shadowing rules must be specified.

### Output and comments

```kode
print("Hello, KODE!")
print(10 + 20)

// Single-line comment

/*
   Multi-line comment
*/
```

`print(...)` requires parentheses.

### Conditions and loops

```kode
if age >= 18 {
    print("Adult")
} else {
    print("Minor")
}

for i in 0..10 {
    print(i)
}

for i in 1..=5 {
    print(i)
}

while condition {
    print("Running")
}
```

### Functions

```kode
fn add(a: int, b: int) -> int {
    return a + b
}

print(add(10, 20))
```

Typed parameters and return types need parser, AST, semantic and backend support if not already present.

### Arrays

```kode
numbers = [10, 20, 30]
print(numbers[0])
```

### Proposed initial type mapping

| KODE type | Intended representation |
|---|---|
| `int` | `i64` |
| `uint` | `u64` |
| `small` | `i32` |
| `byte` | `u8` |
| `float` | `f64` |
| `bool` | Boolean |
| `string` | String |

These are design proposals, not a claim that all types currently compile natively.

---

# 5. Complete development TODO

Priority:
- **P0:** required for the first native compiler.
- **P1:** core language completeness.
- **P2:** production quality and ecosystem.
- **P3:** longer-term expansion.

## Phase 0 — Stabilise the existing language

- [x] Create Rust project.
- [x] Implement initial lexer.
- [x] Implement initial parser.
- [x] Create AST.
- [x] Implement initial interpreter.
- [x] Add `kode check`.
- [x] Add `kode run`.
- [x] Add `kode --version`.
- [x] Create initial KODE IR structures.
- [x] Add Cranelift dependencies.
- [x] Confirm `cargo check` succeeds.
- [ ] Document exactly which syntax currently works.
- [ ] Add a language feature test suite.
- [ ] Add lexer and parser error tests.
- [ ] Add interpreter regression tests.
- [ ] Add a README with accurate current capabilities.
- [ ] Review and resolve compiler warnings; do not globally suppress dead-code warnings.

## Phase 1 — Finish and verify KODE IR [P0]

- [ ] Review `src/compiler/ir.rs`.
- [ ] Define IR types and their semantics.
- [ ] Define functions, parameters and return values.
- [ ] Define basic-block ownership and control-flow rules.
- [ ] Add block parameters or another explicit SSA merge mechanism.
- [ ] Define variable storage and value mapping.
- [ ] Define constants and arithmetic instructions.
- [ ] Define comparisons and boolean operations.
- [ ] Define calls and returns.
- [ ] Define memory operations.
- [ ] Define branches and jumps.
- [ ] Add an IR verifier.
- [ ] Verify block terminators.
- [ ] Verify referenced values and blocks exist.
- [ ] Verify instruction type constraints.
- [ ] Add an IR text dump for debugging.
- [ ] Add IR construction and verifier tests.

**Done when:** KODE can represent and verify simple functions and control flow without depending on Cranelift types.

## Phase 2 — AST-to-IR lowering [P0]

Create `src/compiler/lower.rs`.

- [ ] Lower integer, float and boolean literals.
- [ ] Lower variables and assignments.
- [ ] Lower arithmetic and comparison operators.
- [ ] Lower unary and boolean operators.
- [ ] Lower `print`.
- [ ] Lower function definitions and calls.
- [ ] Lower returns.
- [ ] Lower `if` / `else`.
- [ ] Lower `while`.
- [ ] Lower `for` ranges.
- [ ] Lower `break` and `continue`.
- [ ] Preserve source locations for diagnostics.
- [ ] Add expected-IR tests.

**Done when:** supported AST programs become valid KODE IR without being executed.

## Phase 3 — First Cranelift backend [P0]

Suggested structure:

```text
src/compiler/
├── mod.rs
├── ir.rs
├── lower.rs
└── cranelift/
    ├── mod.rs
    ├── codegen.rs
    └── object.rs
```

- [ ] Confirm dependency versions in `Cargo.toml`.
- [ ] Configure Cranelift target and module.
- [ ] Map KODE integer types to Cranelift types.
- [ ] Map booleans and floating-point types.
- [ ] Translate constants.
- [ ] Translate arithmetic and comparisons.
- [ ] Translate basic blocks and branches.
- [ ] Translate function parameters and returns.
- [ ] Implement function calls.
- [ ] Define native calling convention and runtime ABI.
- [ ] Implement native output support for `print`.
- [ ] Generate an object file.
- [ ] Link the object file using the host system linker.
- [ ] Report code-generation and linker errors.
- [ ] Test generated native programs.

**First native test:**

```kode
a = 10
b = 20
result = a + b
print(result)
```

Expected output:

```text
30
```

**Done when:** KODE source becomes verified KODE IR, then Cranelift code, then a linked native executable that runs correctly.

## Phase 4 — Add `kode build` [P0]

- [ ] Preserve `kode run` as interpreter mode.
- [ ] Add `kode build <file.kode>`.
- [ ] Add output selection, such as `-o app`.
- [ ] Add target selection.
- [ ] Add debug and release modes.
- [ ] Add useful stage-specific errors.
- [ ] Avoid leaving a misleading executable after a failed build.
- [ ] Add an integration test that builds and executes a native program.

Target CLI:

```bash
kode check hello.kode
kode run hello.kode
kode build hello.kode
kode build hello.kode -o hello
./hello
```

Only the first three current commands (`--version`, `check`, `run`) are confirmed. `build` is planned.

## Phase 5 — Semantic analysis and type checking [P1]

Suggested structure:

```text
src/semantic/
├── mod.rs
├── symbols.rs
├── types.rs
└── checker.rs
```

- [ ] Define lexical scope rules.
- [ ] Define declarations, assignment and shadowing.
- [ ] Enforce immutable variable rules.
- [ ] Detect undefined variables.
- [ ] Detect prohibited duplicate declarations.
- [ ] Define type inference.
- [ ] Implement type annotations.
- [ ] Check assignment compatibility.
- [ ] Check arithmetic and comparison operands.
- [ ] Check boolean conditions.
- [ ] Check function argument count and types.
- [ ] Check return types.
- [ ] Detect invalid returns.
- [ ] Validate `break` and `continue` placement.
- [ ] Define numeric conversions.
- [ ] Define integer overflow behaviour.
- [ ] Produce source-located semantic diagnostics.

## Phase 6 — Core language completeness [P1]

### Functions
- [ ] Typed parameters and returns.
- [ ] Unit/void return.
- [ ] Recursion.
- [ ] Nested scopes.
- [ ] Decide on function overloading.
- [ ] Decide on closures.

### Data structures
- [ ] Arrays and mutation.
- [ ] Bounds checking.
- [ ] Strings.
- [ ] Struct definitions and construction.
- [ ] Struct fields and mutation.
- [ ] Enums.
- [ ] Decide on tuples.
- [ ] Decide on classes and methods.
- [ ] Design generics.

### Control flow and operators
- [ ] Nested conditions and loops.
- [ ] Correct `break` / `continue`.
- [ ] Range semantics.
- [ ] Design match/switch.
- [ ] Define error handling.
- [ ] Complete arithmetic, comparison and logical operators.
- [ ] Define assignment and bitwise operators.
- [ ] Specify precedence and short-circuit behaviour.

Keep the first implementation simple. Avoid inheritance, advanced generics and metaprogramming until the core compiler is stable.

## Phase 7 — Runtime and standard library [P1]

Possible structure:

```text
src/
├── runtime/
│   ├── mod.rs
│   ├── value.rs
│   ├── memory.rs
│   └── builtins.rs
└── stdlib/
    ├── mod.rs
    ├── io/
    ├── string/
    ├── collections/
    ├── math/
    ├── filesystem/
    ├── networking/
    └── time/
```

- [ ] Decide compiler intrinsics versus standard-library functions.
- [ ] Define native ABI and calling convention.
- [ ] Define string representation and ownership.
- [ ] Define array and collection representation.
- [ ] Choose memory allocation strategy.
- [ ] Choose and document memory-safety strategy.
- [ ] Define error handling.
- [ ] Implement I/O.
- [ ] Implement string operations.
- [ ] Implement collections.
- [ ] Implement math.
- [ ] Implement filesystem APIs.
- [ ] Implement time APIs.
- [ ] Plan networking APIs.
- [ ] Document runtime compatibility.

Do not assume the interpreter's `Value` enum is automatically suitable for native execution. The native runtime needs a deliberate representation and ABI.

## Phase 8 — Diagnostics and developer experience [P1]

- [ ] Standardise lexer, parser, semantic and backend errors.
- [ ] Show filename, line and column.
- [ ] Show source snippets and error spans.
- [ ] Add helpful suggestions.
- [ ] Add warning support and diagnostic codes.
- [ ] Design `kode fmt`.
- [ ] Design `kode doc`.
- [ ] Add readable compiler output.
- [ ] Add optional KODE IR dump.
- [ ] Add optional Cranelift IR dump.
- [ ] Add verbose build mode.

Example desired diagnostic:

```text
error[KD1001]: undefined variable `total`
  --> main.kode:4:7
   |
 4 | print(total)
   |       ^^^^^
   |
help: declare `total` before using it
```

## Phase 9 — Cross-platform compilation [P2]

Initial targets:

- [ ] macOS Apple Silicon.
- [ ] macOS Intel, subject to toolchain support.
- [ ] Linux x86-64.
- [ ] Linux ARM64.

- [ ] Separate host from target platform.
- [ ] Define target triples.
- [ ] Select platform linkers.
- [ ] Test object formats and linking.
- [ ] Test runtime behaviour per target.
- [ ] Document cross-compilation.
- [ ] Add CI for supported targets.

Later possibilities: Windows, WebAssembly and embedded targets if the runtime design supports them.

## Phase 10 — Tooling and package ecosystem [P2]

- [ ] Design `kode.toml`.
- [ ] Add `kode new`.
- [ ] Add package/dependency management.
- [ ] Add lockfile.
- [ ] Add module/import system.
- [ ] Add local packages.
- [ ] Add build profiles.
- [ ] Add `kode test`.
- [ ] Add `kode fmt`.
- [ ] Add language-server support (LSP).
- [ ] Add editor syntax highlighting.
- [ ] Decide debugger strategy.
- [ ] Add documentation generation.
- [ ] Decide package registry strategy.

## Phase 11 — Performance and optimisation [P2]

- [ ] Establish interpreter benchmarks.
- [ ] Establish native benchmarks.
- [ ] Measure startup and compile time.
- [ ] Measure runtime and memory use.
- [ ] Add optimisation levels.
- [ ] Add constant folding.
- [ ] Add dead-code elimination.
- [ ] Add common-subexpression elimination where appropriate.
- [ ] Design inlining.
- [ ] Research profile-guided optimisation.
- [ ] Compare against equivalent Rust and C programs with fair, reproducible workloads.

Do not claim performance superiority without repeatable benchmarks.

## Phase 12 — Multiple compiler backends [P3]

Keep KODE IR independent from backend-specific APIs.

```text
                 KODE SOURCE
                      |
                    Lexer
                      |
                    Parser
                      |
                     AST
                      |
             Semantic Analysis
                      |
                   KODE IR
                      |
          +-----------+-----------+
          |           |           |
          v           v           v
       Cranelift    LLVM         WASM
       backend     backend      backend
          |           |           |
          v           v           v
       Native      Native       Web/
       machine     machine      WASM
       code        code         module
```

- [ ] Stabilise KODE IR.
- [ ] Define backend capability requirements.
- [ ] Research LLVM as a later backend.
- [ ] Research WebAssembly backend.
- [ ] Keep backend code separate from the frontend.
- [ ] Add backend conformance tests.

Cranelift is the planned first backend. LLVM is optional future work, not a prerequisite for the first native compiler.

## Phase 13 — Security and production readiness [P2/P3]

- [ ] Document memory-safety guarantees.
- [ ] Define unsafe and foreign-function interface policy.
- [ ] Validate pointer and array operations.
- [ ] Define stack/recursion limits where appropriate.
- [ ] Test overflow and division-by-zero behaviour.
- [ ] Review filesystem and networking APIs.
- [ ] Fuzz lexer and parser.
- [ ] Fuzz IR verifier.
- [ ] Add regression tests for fixed bugs.
- [ ] Audit dependencies.
- [ ] Create reproducible release builds.
- [ ] Document supported platforms and limitations.
- [ ] Define language/compiler version compatibility.

---

# 6. Recommended immediate sequence

Work in this order. Do not start all phases simultaneously.

1. [ ] Review the actual IR and module files.
2. [ ] Add IR verification and tests.
3. [ ] Implement AST-to-IR lowering for integer literals, variables and arithmetic.
4. [ ] Implement the first Cranelift backend.
5. [ ] Generate a native object file.
6. [ ] Link it using the macOS system linker.
7. [ ] Implement native integer output.
8. [ ] Add `kode build`.
9. [ ] Prove the executable runs without the interpreter.
10. [ ] Add functions and control flow.
11. [ ] Add semantic analysis and type checking.
12. [ ] Expand runtime and standard library.
13. [ ] Add Linux builds and CI.
14. [ ] Add package management and developer tooling.

## 7. First native compiler acceptance tests

The initial backend should support a small, explicit subset.

**Test 1 — Constant output**

```kode
print(42)
```

Expected: `42`

**Test 2 — Arithmetic**

```kode
a = 10
b = 20
print(a + b)
```

Expected: `30`

**Test 3 — Precedence**

```kode
print(2 + 3 * 4)
```

Expected: `14`

**Test 4 — Reassignment**

```kode
count = 10
count = count + 1
print(count)
```

Expected: `11`

**Test 5 — Conditional**

```kode
if 10 < 20 {
    print(1)
} else {
    print(0)
}
```

Expected: `1`

**Test 6 — Function**

```kode
fn add(a: int, b: int) -> int {
    return a + b
}

print(add(10, 20))
```

Expected: `30`

Every test must use the native build path. It must not silently fall back to the interpreter.

## 8. Target CLI

Long-term command design:

```bash
kode --version
kode check main.kode
kode run main.kode
kode build main.kode
kode build main.kode -o app
kode test
kode new my_project
kode fmt
kode info
```

Only `--version`, `check` and `run` are confirmed in the current `main.rs`. The others are planned.

## 9. Architecture rules

1. Do not put the whole compiler in `main.rs`.
2. Keep lexer, parser, AST, semantic analysis, IR, backend and runtime modular.
3. Do not make KODE IR depend on Cranelift types.
4. Keep the interpreter as a separate execution mode.
5. Never silently fall back to interpretation when native compilation fails.
6. Do not mark a feature complete until it has tests.
7. Do not claim native compilation until a generated executable runs independently.
8. Preserve source locations for diagnostics.
9. Define language semantics before implementing complex features.
10. Prefer small, testable milestones over a large rewrite.
11. Keep dependency versions aligned and tested.
12. Benchmark before making performance claims.

## 10. Decisions and files still to review

- [ ] Full current `Cargo.toml`.
- [ ] Full current `src/compiler/mod.rs`.
- [ ] Full current `src/compiler/ir.rs`.
- [ ] Exact lexer/parser feature coverage.
- [ ] Exact interpreter feature coverage.
- [ ] Whether semantic analysis already exists.
- [ ] Variable declaration and shadowing rules.
- [ ] Function and return semantics.
- [ ] String and array representation.
- [ ] Memory management strategy.
- [ ] Struct versus class semantics.
- [ ] Module/import system.
- [ ] Error-handling model.
- [ ] Native runtime and ABI.
- [ ] Supported target triples and linker requirements.

## 11. First native release definition

The first native compiler milestone is complete only when:

- [ ] A `.kode` source file parses successfully.
- [ ] AST lowers into verified KODE IR.
- [ ] Cranelift translates the supported IR.
- [ ] A valid native object file is emitted.
- [ ] The system linker creates an executable.
- [ ] The executable runs without invoking the KODE interpreter.
- [ ] Arithmetic and output tests pass.
- [ ] Build failures produce useful diagnostics.
- [ ] Builds are repeatable.

A small, reliable native-compiled subset is better than many partially working features.

## 12. Progress log

| Date | Milestone | Result |
|---|---|---|
| 2026-10-31 | Rust project and current CLI checked | `cargo check` succeeds |
| 2026-10-31 | Cranelift 0.125.4 dependencies installed | Confirmed by `cargo tree` |
| 2026-10-31 | Initial KODE IR exists | Not connected to code generation |
| 2026-10-31 | Interpreter execution confirmed | `kode run` uses Rust interpreter |

Add future entries with the exact implementation, test command and result.

---

**Current overall status:** KODE has an early-stage Rust-based frontend and interpreter, an initial IR foundation, and Cranelift dependencies installed. The next major engineering task is to connect AST → KODE IR → Cranelift → object file → system linker and prove that a standalone native executable works.
"""
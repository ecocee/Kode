# Changelog

Release notes for the `kode` compiler. Versions follow the binary, not the
language: a patch release fixes bugs, a minor release may add syntax.

## Unreleased

### Added

- **S4 slice 5: string interpolation, including its literal-head release.** A `print`
  argument that is an interpolation lowers to a `StrConcat` chain: the head becomes the
  accumulator (a literal head is released after its first concat — the `release "hi "`
  line — and a variable head is retained), each further piece releases the old
  accumulator, an int hole is an owned `rt::IntToStr` temporary released right after its
  concat, and a lone hole is only a `retain`. The print releases the value it read
  (`selfhost/mir.kode`); the interpreter renders ints and treats `retain` as a no-op
  (`selfhost/eval.kode`). Five fixtures came first in `selfhost/spec/run/`, so the gates
  report **22/22** dumps byte-identical and **22/22** runs with the oracle's output and
  exit code. Interpolation as a value, non-name holes and float/bool holes are not in.

- **S4 slice 4: strings built at run time, with the oracle's retain/release lines.**
  MIR temporaries now have a type, so a string is a `ref`: `let _0: ref`, zeroed to
  `null`, released before every store (including the first one) and once more in `bb1`;
  an intermediate is released right after its last use; `+` on strings lowers to
  `_N = call rt::StrConcat(a, b)` with borrowed literals and a moved result
  (`selfhost/mir.kode`). The interpreter keeps a string per reference temporary and
  treats `release` as a no-op, since it has no counts to drop (`selfhost/eval.kode`).
  Five fixtures came first in `selfhost/spec/run/`, so the gates report **17/17** dumps
  byte-identical and **17/17** runs with the oracle's output and exit code.
  Interpolation is not in yet: it also releases its literal head, a rule of its own.

- **S4 slice 3: reassignment and the rest of the integer operators.** A declaration
  now lowers its value before it allocates the variable (`x = 1 + 2` is
  `_0 = add 1, 2` then `_1 = _0`, which slice 2 only got right by accident), a
  reassignment writes the existing temporary (`x = x + 1` is `_1 = add _0, 1` then
  `_0 = _1`), and `- * / %` lower to the oracle's `sub`/`mul`/`div`/`rem` alongside
  `add` (`selfhost/mir.kode`, `selfhost/eval.kode`). `div` and `rem` truncate toward
  zero, as the language does: `0 - 7 / 2` is `-3`. Five fixtures came first in
  `selfhost/spec/run/`, so the gates report **12/12** dumps byte-identical and
  **12/12** runs with the oracle's output and exit code.

- **S4 slice 2: a local, a name read and `+`, with the oracle's temporaries.** The
  self-hosted MIR now lowers `x = <int expression>` and `print(<int expression>)`
  (`selfhost/mir.kode`, learning the oracle's rules: one temporary per variable and per
  `add`, numbered in walk order; the `let` block before `bb0`, with the variable's name
  as a comment; one `_N = 0` per temporary at the top of `bb0`; `_N = add a, b` for a
  sum). `selfhost/eval.kode` keeps one `int` per temporary and resolves assignment,
  `add` and `rt::PrintInt` against it. Four fixtures came first in
  `selfhost/spec/run/` (a local, `1 + 2`, two locals summed, `1 + 2 + 3`), and the
  gates report **7/7** dumps byte-identical and **7/7** runs with the oracle's output
  and exit code. Out of scope still: other operators, reassignment, string temporaries
  and their retain/release, blocks and user calls.

- **S4 has begun: a MIR, a dump and an interpreter for `print` of literals.** The
  self-hosted compiler's first S4 slice lowers a program whose statements are
  `print(...)` calls over scalar literals into one function, `<main>`, with `bb0`
  calling the runtime print primitives (`rt::PrintStr`, `PrintInt`, `PrintFloat`,
  `PrintBool`, `PrintSep`, `PrintNl`) and `goto bb1; return` after them
  (`selfhost/mir.kode`), prints that MIR in `kode mir`'s own layout, and runs it
  (`selfhost/eval.kode`), so `kode-stage1 mir` and `kode-stage1 run` are new modes.
  A program outside the slice stops with a reason on stderr and exit 2 rather than
  dumping MIR that would not run. The fixtures came first: `selfhost/spec/run/` holds
  three programs and their `.out`, and `./scripts/selfhost.sh mir` reports **3/3**
  dumps byte-identical while `run` reports **3/3** cases whose output is
  byte-identical and whose exit code equals the oracle's — each `.out` is checked
  against the oracle's own `kode run`, and a run that does not exit 0 fails the gate,
  so printing the right text and then leaving differently is caught.

- **`check text` is a gate, and it is green: 169/169.** The oracle prints
  `checked in 0.38 ms` on success, which is a wall-clock measurement rather than
  diagnostic text, so `./scripts/selfhost.sh check` drops that one line (and only it)
  before comparing, and now compares the rendered report for the run-pass programs too,
  not just the compile-fail ones. Every case's report — code, message, position, source
  line, carets, hint, order — is byte-identical to the oracle's, and `check` exits
  nonzero when a case's codes, its report or the port's self-check differs. Measuring the
  clean programs turned up one false warning: a variable holding a closure that is
  called (`greet("Bob")`, `f(21)`) is read, so it no longer says `W0001` — which had made
  `closures.kode`, `fn_values.kode` and `optional_params.kode` warn. `check text`
  **111 → 169/169**.

- **`E0408` and `E0409` name their file.** Both are recorded while the module graph
  loads, before any file is walked, so the `import ... as` alias now carries the file it
  is written in (`Alias.file`) and the report's `-->` line no longer reads `--> :2:25`.
  Compile-fail reports byte-identical: 110 → **111/169**.

- **Diagnostics point where the oracle points.** Four rules were underlining more than
  the oracle does: `E0215` (a named argument, or any named argument to a builtin) now
  underlines the argument's name, `E0305` (a function whose body can fall off its end)
  the function's name rather than its whole signature, `E0419` (a payload field read
  without the narrowing that makes it safe) the field rather than the whole `s.radius`,
  and `E0218` (`x.y` on a value with no fields) the receiver. The interface form of
  `E0218` keeps pointing at the field, because there the field is the mistake. `check
  text` over the compile-fail corpus: 103 → **110/169**.

- **Generic calls and generic annotations are checked, and S3 is complete.** The gate now
  reports **169/169** programs with the oracle's own code set — every compile-fail fixture
  and every run-pass program. `selfhost/generics.kode` infers: a free function's type
  parameter has to appear in a parameter type (`E0250`), a generic class's in one of its
  fields at the constructor (`E0422`), and a method of a generic class already has its
  class's parameters from the receiver, so only the ones it declares itself are asked for
  — which is also why a generic class's method may not declare any of its own yet
  (`E0252`, in `names.kode`). A generic function used as a value is `E0251`. Annotations
  follow: `Box<int, int>` is `` `Box` takes 1 type argument, found 2`` (`E0421`) and a
  bare `Box` is `` `Box` is a generic class, so it needs type arguments`` (`E0423`), both
  of which leave the declaration unannotated so the value is reported next, exactly as
  the oracle does. A parameter whose type the slice cannot read (`fn(T) -> U`) silences
  the inference rule rather than guessing. `E0206` now points at the callee name, not the
  whole call. `check text` moves 95 → **103/169**; `self-check` 51 → **52/52**.

- **Warnings exist, and `W0001` is the first.** A `Diag` now says whether it is an error
  or a warning: it renders as `warning[W0001]: ...` and `check` exits 0 when nothing but
  warnings was found. `selfhost/unused.kode` records every name an assignment declared and
  `sema.kode` marks the ones read (a name starting with `_` is exempt), so an unused
  variable says `variable `x` is assigned but never used` with the oracle's hint. The
  oracle suppresses warnings for a program that has errors, and so does the port. Gate:
  161 → **162/169**; `tests/compile-fail/w0001_unused.kode` is identical but for the
  oracle's timing line.

- **`throw`, `catch`, the list callbacks and the web layer are checked.**
  `selfhost/throws.kode` refuses a `throw` that is neither a string nor an `Error`
  (`E0226`), and a `catch` block that does not answer with the expression's type
  (`E0227`) — including a block that ends in nothing at all. `xs.map(f)` and
  `xs.filter(f)` demand a callback that takes one item and gives one back (`E0242`),
  whether it is a closure with a written parameter or a named function.
  `selfhost/server.kode` knows the two shapes of a handler: a route's
  `fn name(req: Request) -> Response` (`E0233`) and `serve`/`serve_tls`'s `Router` or
  `fn name(method: string, path: string, body: string) -> string` (`E0232`), and a
  `Router()` value now has that name instead of `any`. Gate: 156 → **161/169**,
  `check text` **95/169**.

- **`match` is checked.** What may be matched, what a pattern has to be, and what a
  `match` has to cover: `E0223` refuses a value that is not an int, a string, a bool or an
  enum (and a pattern of the wrong type), `E0224` counts the missing cases — an enum's
  variants by name, both bools, or the `_` arm an int or a string needs — and `E0225`
  reports a variant an enum does not declare, with the closest one it does. `E0420`
  refuses `==` and `!=` on a payload enum, whose value is a reference. `selfhost/names.kode`
  now records every variant (and whether it carries payloads), which is what all four
  read. Gate: 152 → **156/169**, `check text` **90/169**.

- **The port's hints match the oracle's.** An unknown name (`E0201`) now says which
  mistake it probably is: a module needs `.member`, an enum value is written
  `Enum.Variant`, a near miss gets `did you mean ...?` (the same Levenshtein search the
  member rules use), and anything else is created first. `E0206` suggests a declared
  function, class or builtin, or says how to define one, and `E0403` says what a module
  may contain. `check text` (byte-identical reports) moves **79 → 86/169**.

- **The port renders a diagnostic the way the oracle does.** `check` now prints the
  whole report — `error[CODE]: message`, the `--> file:line:column` line, the offending
  source line with its carets, and the hint — instead of the bare code and message. The
  tree carries its file (`Ast.file`), a `Diag` names it, and `report.render_diag` shares
  one renderer with the parser's errors. `./scripts/selfhost.sh check` reports how many
  programs are byte-identical (`check text`), so a wrong span or missing hint is visible
  before it becomes a gate: **79/111** today, the rest being the wording gaps and the
  families S3 has not reached.

- **A user call and a class field now have a type.** `members.value_type` answers what
  `typing.kode` cannot on its own: a function's or method's declared result, a class
  constructor's instance, and a field's type — through `?.` too. The operators and
  `print` ask it, so `f() + "a"` is `E0203`, `print(f())` on a function that declares no
  result is `E0205`, and `y = f()` binds a typed `y`. A result written in the callee's
  own type parameters (`fn id<T>() -> T`) stays `any`, as do a generic class's fields
  (ADR 0030), so no rule guesses. Gate holds at **152/169**, self-check **47/47**.

- **The port's checker knows what a class has.** `selfhost/members.kode` collects every
  field and method a class or interface declares (and the fields of the prelude classes
  `Json`, `ProcessResult`, `Error`, `Request`, `Response`, `HttpResponse` and `Router`),
  so a `.` on a class is checked instead of guessed: an unknown name is `E0217` with the
  oracle's "did you mean" hint (from the new `selfhost/suggest.kode`), and a receiver
  with no fields at all — an interface, a number, a list, an enum — is `E0218`.
  `E0417` reports a payload variant used as a value instead of called, `E0228` a
  method parameter that shares a field's name, and `E0234`/`E0235` a class that does not
  implement its interfaces' methods with exactly the same parameter and result types.
  `?.` now also refuses a field that is not a reference (`E0219`), and a failed `.`
  declares nothing, so the next use of the name is `E0201` as in the oracle. Gate:
  145 → **152/169**, self-check **47/47**.

- **The port's checker knows what may be none.** `selfhost/optional.kode` ports the
  optional rules: `E0219` for the shapes that are not supported yet (a list of `int?`, a
  map of `int?`, an optional of something that is neither a reference nor a scalar) and
  for `none` written without a type, `E0222` for an empty list or map that has no type to
  take, `E0220` for reaching inside a value that may be none (`.` without `?.`, a method
  call, an index, and `!` on something that cannot be none), `E0221` for going the other
  way (`??` or `?.` on a value that cannot be none, and a default of the wrong type), and
  `E0313` for writing a type on a name that already exists. `E0419` is the payload-enum
  rule: reading `s.radius` is only valid where `s` is narrowed to `Shape.Circle`, which
  the walker now tracks for the then branch of an `is` test (ADR 0039 §4.4), so
  `tests/run-pass/enum_is.kode` stays clean. `??` keeps an optional default
  (`env("A") ?? env("B")` is a `string?`), an enum variant types as its enum, and a `Json`
  value or an optional says what it is. Gate: 138 → **145/169**, self-check **45/45**.

- **The port's checker knows what may be indexed, sliced, collected and looped over.**
  `selfhost/containers.kode` ports the rules whose subject is a value's shape:
  `E0209` for a condition that is not a bool (the oracle's `if` and `while` wording, the
  `else if` form included, since it is another `If` inside the `Else`), `E0211` for a
  value that cannot be indexed (`a string cannot be indexed`, "`Json` cannot be changed in
  place" for an index target) or sliced (bounds first, so a bad bound on a map is the
  bound's error alone), `E0212` for an index, key or slice bound of the wrong type —
  a list index is an int, a map key a string, a `Json` value either — `E0213` for a list
  item or map value that is not the type of the first one (and a list that cannot hold
  `none` or nothing at all), and `E0214` for a `for ... in` over something that is not a
  list or a map. A refused literal declares no name, so the next line is `E0201` as in
  the oracle, and `for` now binds a list's item (or index and item), a map's key (or key
  and value) with the types they really have rather than `any`. Optionals are left to
  `E0220`'s own family, and a range loop's bounds to `E0203`'s. Two supporting rules
  were fixed: `none` fits only an optional (so `[1, none]` is `E0213`), and an enum is
  printable. Gate: 130 → **138/169**, self-check **44/44**.

- **The port's checker knows the builtins, the runtime functions and methods.**
  `selfhost/stdlib.kode` mirrors the oracle's tables: the runtime functions a program
  calls by name (`exec` and its `(string, [string], string)` among them), the
  one-argument builtins with the words the oracle uses for each (`len` "a string, a
  list or a map", `bytes` "a length", `int`, `float`, `string`, `assert`), and
  `StringBuilder`'s `push`/`build`. `selfhost/signatures.kode` records every method as
  `Class.method`, with its class's type parameters in scope, and keeps the bare name
  for a return message — the oracle prints `` `m` `` there but `` `P.m` `` in a call.
  `selfhost/functions.kode` resolves a method call through its receiver's type and
  reports the three shapes the oracle uses: a runtime function names its arguments
  positionally ("argument 2 of `exec` should be `[string]`, found a string"), a
  builtin describes the value it wants ("`bytes` expects a length, found a string"),
  and a method of a class the program declares is a call like any other, named
  parameter help included. Every row also carries what the call answers, so an
  expression typed through it is checked like any other value: `print(exec(...))` is
  `E0203` ``cannot print `ProcessResult` yet``, `print(assert(x))` is `E0205` "this call
  returns nothing", and `print(env("HOME"))` is ``cannot print `string?` yet`` — the
  print rule now tells a scalar optional (`int?`, `float?`, `bool?` print fine) from the
  rest, and a bare `none`, as the oracle does. Named arguments separate the two kinds of
  builtin: a free
  one refuses them once, at the first (`len(x: 1)` is `E0215`), while a builtin method
  takes a name the way a declared one does — `sb.push(s: "a")` is clean and
  `sb.push(x: 5)` reports only the type — and `sb.build(1)` is the plain arity
  `E0207`. A call that fails still binds the name when it is a
  builtin or a constructor (`b = bytes("four")` reports only the argument), which the
  oracle confirms; a refused function still declares nothing. The wrong-number-width
  hint ("numbers never convert automatically") now also appears on a user call.
  Gate: 122 → **126/165**, self-check **43/43**. Four fixtures keep the result-type and
  print rules honest: `tests/compile-fail/e0203_print_process_result.kode`,
  `e0205_print_nothing.kode` and `e0203_print_optional_string.kode`, plus
  `tests/run-pass/optional_scalar_print.kode` (an `int?`, a `float?` and a `bool?`, none
  included, print as `none`, `4`, `true`), so the gate is **130/169** with the same
  self-check.

- **The port's checker knows what a function promises.** `selfhost/signatures.kode`
  records every function's parameters and result, and every class's constructor — its
  fields in declaration order, the ones with a default value optional but still taking
  their position — before any file is walked, because a program's namespace is shared
  across modules. `selfhost/functions.kode` then checks a call against it: `E0207` for
  too many arguments (with the oracle's singular and plural), `E0215` for an argument
  the callee has no parameter for, `E0216` for a parameter the call leaves out,
  `E0208` for the wrong type — including the oracle's second form for a type
  parameter, which is bound at its first argument (`T`, `[T]`, `{string: T}`, `T?`) and
  has to agree at the rest — and `E0304`/`E0305`/`E0310` for a body that gives nothing
  where a result was declared, a function that can fall off its last line, and a
  `return` of the wrong type. A failed call declares nothing, so the name it was
  assigned to is unknown on the next line, as in the oracle. Two type rules grew the
  fix the families needed: `int?` is assignable to `int?` (it was not), and a type
  parameter nested in a container is recognised. Gate: 109 → **122/165**, self-check
  **42/42**. Methods, builtins and closures are silent: their signatures are not in the
  table yet.

  The `check` gate now compares the port's codes with the **oracle's own**, not with
  the `.err` file: a `.err` names the codes the Rust conformance test looks for and may
  leave out the follow-on ones (`E0102` after a parser error, `E0201` after a failed
  call), so the old comparison failed a port that agreed with Stage 0. Six parser
  fixtures and the named-argument ones became green by measuring the right thing.

- **The port's parser refuses what the oracle refuses.** `selfhost/depth.kode`
  enforces the 128-level nesting limit (ADR 0032) the way the oracle does, in two
  places: a guard on entry to the expression, block and type productions, which
  catches `((((1))))` and a tower of `if`s at the token the oracle points at, and a
  walk over the finished tree with an explicit stack (mirroring
  `crates/kode-syntax/src/depth.rs`, push order and transparent wrappers included),
  which catches what a loop built — `1 + 1 + ...` is left-deep and costs the later
  passes one frame per term. Both report `E0118`, and the walk only runs on a clean
  tree. Recovery now matches the oracle on four productions: `fn` without a name and
  `for` without a loop variable report `E0110` and skip the line (which is why the
  closing `}` is then a statement, `E0102`); a type the oracle refused ends a typed
  declaration instead of reporting `E0107` on the `:` it stopped at; and a signature
  with a body ends the whole interface after `E0236`. The `parse` gate now compares
  the entire `tests/compile-fail` corpus, not only the slice's own cases, so a
  recovery that drifts is caught where it happens: **169/169** trees and
  diagnostics byte-identical, **39/39** of the port's own files. CI runs it too:
  the `selfhost` job now ends with `./scripts/selfhost.sh parse` after `stage1`,
  which adds 1.8 s (416 processes over 208 programs) on the development machine.

- **The port's checker learns declarations.** `selfhost/names.kode` collects every
  name a program declares — functions, constants, classes, enums, interfaces, `from`
  names — before any file is walked, and reports the declarations that cannot stand:
  `E0301` (a name defined twice, with the oracle's wording for a class and for the
  built-in `Error`), `E0302` (a builtin redefined), `E0306` (a parameter or field
  twice), `E0307` (a constant that is not a plain value, with the `${}` hint) and
  `E0308` (a constant defined twice). `selfhost/annotations.kode` turns a written type
  into a `Type`, `selfhost/items.kode` is what a function or class declaration puts in
  scope, and `selfhost/binding.kode` what `name = value` does — `E0312` (a constant
  assigned), `E0205` (the value of `print`, which returns nothing) and, in the
  operator rules, `E0204` (a literal division by zero). The walk tracks whether it is
  inside a function and how many loops deep, so `return` at the top level is `E0311`
  and `break`/`continue` outside a loop `E0314`. A value that does not fit its
  annotation is `E0202` *and* declares no name, which is why
  `tests/compile-fail/e0202_annotation.kode` reports `E0201` as well, like the oracle.
  Gate: 95 → **105/165**, self-check **37/37**.

- **The port's checker learns types.** `selfhost/typing.kode` says what a value is —
  the primitives, `[T]`, `{string: T}`, `T?`, a named class, `any` — and types
  literals, locals, unary and binary expressions, list and map literals and the calls
  it can name; it also resolves annotations (`E0303`, with the fixed-width hint `u8`
  gets). `selfhost/operands.kode` holds the operator and printing rules (`E0203`,
  `E0210`), and an assignment keeps its target's type, so `x = 1` then `x = "a"` is
  `E0202`. `selfhost/scope.kode` is the frame stack, which keeps `sema.kode` under its
  size limit. A rule only fires when the mistake can be proved: an `any` operand, an
  optional operand (narrowing is a later slice) and two named types (no `implements`
  yet) are accepted. Gate: 85 → **95/165**, self-check **33/33**.

- **The port's import namespace is complete.** `selfhost/namespace.kode` adds the two
  alias rules the oracle keeps in the checker: a module alias may not repeat
  (`E0408`, "the module name `x` is imported twice") or hide a declaration
  (`E0409`, "the module name `x` hides another declaration", with the oracle's hint).
  The alias list is kept in load order, so the *second* of two equal aliases is the
  one reported, matching `crates/kode-sema/src/check/item.rs::collect_modules`, and
  the first is the one that survives there — a member call resolves through it, so
  `tests/compile-fail/e0408_duplicate_alias_across_files.kode` proves the order from
  the outside: its `E0410` only appears when the alias from the imported file is the
  one in force. A `.err` file may now list several codes (one per line or space
  separated), which is what lets that case state both, and both conformance suites
  check each one. `selfhost/diag.kode` holds the diagnostic record both passes
  produce, and the builtin/std name table moved to `selfhost/builtins.kode`.
  Gate: 82 → **85/165**, self-check **30/30**.

- **The port's checker loads the module graph.** `selfhost/modules.kode` makes
  `import`/`from` read what they name instead of only recording the path: paths
  resolve relative to the file that wrote them (a bare path against the package root
  from `kode.toml`), `.kode` is appended when absent, each file is read once, and the
  oracle's module diagnostics are reproduced byte-for-byte — `E0401` missing file,
  `E0402` cycle, `E0403` code in a module. What an import reaches is enforced too:
  `E0405` a name the module does not define, `E0410` a member an alias does not have,
  `E0411` a name or member that is not `pub`. `kode-stage1 check` therefore checks the
  whole program: the port's own sources are clean under its own checker
  (`self-check: 27/27`, a second gate in `./scripts/selfhost.sh check`), and the
  corpus gate moves 75 → 82/164.

- **The S3 checker starts (name resolution).** `selfhost/sema.kode` resolves every
  name a program uses against top-level declarations, imports, builtins, the
  runtime's free functions and modules, the prelude classes, and block scopes
  (parameters, assignment targets, `for` and `catch` variables, match-arm payloads,
  closure parameters), reporting `E0201`/`E0206`; `kode-stage1 check <file>` is the
  new entry point. The gate comes first: `./scripts/selfhost.sh check` compares the
  port's code set with the code each program's `.err` names, so progress is a number
  — **75/164** today (18 compile-fail exact, 57 run-pass clean, 80 with no
  diagnostic yet, 6 where the wider corpus still parses differently). `readiness`
  now reports the checker level, and `all` runs the new pass.

- **Function types in annotations (S2 slice 14).** The port parses the whole type
  grammar now, `fn(int) -> int` included (in a parameter, a field, a local, a return
  type, nested, zero-argument, after `?`), and `Task<T>`/`Channel<T>` dump as their
  own kinds instead of a generic named `Task`. Malformed annotations also match the
  oracle: a missing `]`, `}`, `>` or `->` is `E0103` over the token found (the port
  had been saying `E0102` in its own words) and a map key that is not `string` is
  `E0115`, which the port had not checked at all. `selfhost/spec/parse` is 30 tree
  cases and 24 diagnostic cases, **54/54 byte-identical**, self-parse 25/25.
  `./scripts/selfhost.sh readiness` also counts deferred packets apart and, with
  nothing on the path blocked, names S3 as next and records that WP-12 stays until
  S5 instead of pointing at the deliberately red WP-07.

- **`stderr(...)` (WP-10, ADR 0045).** The same arguments and formatting as
  `print`, written to stderr: variadic, space-separated, one line per call, with
  the existing `E0203`/`E0205` rules. The port now reports diagnostics with
  `stderr(render(...))` instead of opening `/dev/stderr` through `write_file`, so
  it no longer has a Unix-only path. Runtime ABI marker `0006`. Evidence:
  `tests/run-pass/stderr_stream.kode` (+ `.err`, compared by `llvm_conformance`),
  `tests/compile-fail/e0203_stderr_bytes.kode`, and WP-10 met in the gates table.

- **`StringBuilder` (WP-09, ADR 0044).** `sb = StringBuilder()` with `push(s)` and
  `build()`: one growable byte buffer instead of a new string per `+`, which is what
  S5 needs to emit IR text in linear time. `push` copies the bytes and does not
  retain the argument; `build` returns the text so far as a new string and leaves the
  builder usable. No new error code (`E0208`, `E0207`, `E0217`, `E0203` cover it).
  Evidence: `tests/run-pass/string_builder.kode` (0 leaks),
  `tests/compile-fail/e0208_builder_push.kode`, and the measured 185.2 ms → 5.2 ms
  in `benchmarks/report.md`.

### Docs

- **ADR 0045: a `stderr` stream.** `stderr(...)` mirrors `print` on the
  diagnostic stream with six new runtime symbols; the port uses it for parse
  errors, which removes its last platform-specific workaround.

- **ADR 0044: `StringBuilder`.** The last language feature S2–S5 need that is not a
  diagnostic stream: a reference-counted `BUFFER` object with `push`/`build`, typed
  as the builtin `Type::Builder`, with the ABI marker bumped to `0005`.

- **ADR 0043: byte buffers and base-2/8/16 literals.** `bytes(n)` is a fixed-size
  buffer of bytes backed by the packed-list object, so retain/release/drop and the
  index accessors are the existing ones; `u8`/`i32` as numeric types stay out
  (WP-08b) because reading bytes needs no new numeric type.

- **ADR 0042: the AST is an arena.** The port's syntax tree is one list of nodes
  owned by the tree, with children and parents as `int?` ids, so no cycle is
  expressible and one release frees everything. `weak` (ADR 0038) stays proposed and
  off the self-hosting path; the new `wp11-arena` gate probe compiles and runs the
  shape on Stage 0.

### Added

- **The parser, slice 1 (S2).** `selfhost/ast.kode` is the syntax tree as one arena
  of nodes with `int` child ids (ADR 0042), and `selfhost/parser.kode` parses
  assignments of a literal or a name and call statements whose arguments are
  literals or names. `kode-stage1 ast` prints the oracle's format;
  `./scripts/selfhost.sh parse` diffs it with `kode ast` over
  `selfhost/spec/parse` and reports 7/7 trees byte-identical, `expr_count`
  included. Every other statement and expression is an `E0102` error until its
  slice lands.

- **Operators in the port's parser (S2 slice 1b).** Every infix operator at the
  oracle's binding powers (`or` 2 … `* / %` 20), prefix `-`, `~` and `not`,
  grouping (parentheses add no node), and a line that ends after an operator.
  Float literals are re-read as values and printed like Rust's `{:?}`, so `2.50`
  is `2.5`. Cases: `selfhost/spec/parse/{binary,parens,unary,line_break}.kode`,
  all byte-identical to `kode ast`.

- **Postfix expressions and value literals in the port's parser (S2 slice 2).**
  `.name`, `.name(args)` (named and trailing-comma arguments), `?.name`, `[i]`,
  every slice form, `obj!`, `[lists]`, `{maps}`, `none`, and string interpolation:
  `Interp` of `Lit` text and `Hole` expressions whose inner tokens are re-lexed
  with the hole's absolute offset, so inner spans match the oracle. The parser is
  now five modules (`cursor`, `infix`, `expr`, `literals`, `parser`) so each stays
  under the size rule. Cases: `selfhost/spec/parse/{postfix,collections,interp}.kode`,
  **10/10 trees byte-identical**.

- **Blocks and control flow in the port's parser (S2 slice 3).** `if`/`else`/
  `else if`, `while`, both `for` forms (`0..n`, `0..=n`, `for x in xs`,
  `for k, v in m`), `break`/`continue`, and a block as the body of each: one
  statement per line, ending at a newline, the end of the file, or the closing
  `}` — the oracle's rule, so `else` must follow `}` on its line. Cases:
  `selfhost/spec/parse/{control_flow,blocks}.kode`, **12/12 trees
  byte-identical**.

- **Items and `return` in the port's parser (S2 slice 4).** `fn` declarations with
  an optional `pub`, type parameters, annotated parameters, an optional return type
  and either a block or `=> expr`; `return` with or without a value; and type
  annotations (`T`, `[T]`, `{string: T}`, `Box<T>`, `T?`). The statement grammar
  moved to `selfhost/stmt.kode` so `parser.kode` is the item level. The oracle's
  odd spans are reproduced exactly: a `Fn` spans `fn` and its name, a `TypeList`/
  `TypeMap`/`TypeGeneric` spans only its opening token. Cases:
  `selfhost/spec/parse/{functions,returns}.kode`, **14/14 trees byte-identical**.

- **Classes and `match` in the port's parser (S2 slice 5).** `class` declarations
  with an optional `pub`, type parameters, `implements` names, fields with their
  type and an optional `= default`, and methods; and `match` statements with
  `Wildcard` or value arms (`PatternValue`, parsed at `BP_ARITH`) whose body is
  always a `Block`. Two oracle quirks are reproduced: a method dumps as `Method`
  with `pub=false` even when the source says `pub fn`, and a `Class` node carries
  no `span`. Cases: `selfhost/spec/parse/{classes,matches}.kode`, **16/16 trees
  byte-identical**.

- **Enums, interfaces, imports and consts in the port's parser (S2 slice 6).**
  `enum` with payload variants (`Circle(radius: float)`), `interface` signatures
  (a body is `E0236`), all three `import` forms and `const` with an optional
  annotation. The item grammar was split into `types.kode`, `decls.kode` and
  `enums.kode` (a case's `kode fmt` output indents enum bodies deeply; that is the
  formatter's existing behaviour, unchanged here). Cases:
  `selfhost/spec/parse/{enums,interfaces,imports}.kode`, **19/19 trees
  byte-identical**.

- **`catch` now parses anywhere the oracle allows it.** The nested positions
  (parentheses, argument lists, collections, interpolation holes) were the one gap
  left in slice 7, because a `catch` body is a block of statements and the statement
  grammar already imports the expression grammar. A one-method `Blocks` interface in
  `selfhost/blocks.kode` breaks the cycle: `Exprs` carries a block parser, and a hole
  re-lexes into its own cursor and takes the same parser over that cursor. `catch`
  inside `${...}`, `f(a catch e { 1 })`, `[a catch e { 1 }]`, `(a catch e { 1 })` and
  nested `((a catch x { 1 }) catch y { 2 })` are all covered by
  `selfhost/spec/parse/catch_expr.kode`, and `parse` is still **22/22 trees
  byte-identical**.

- **Compound assignment in the port's parser (S2 slice 13).** `x += 1` and the other
  ten operators now parse, and so do `xs[0] = 1`, `m["k"] += 2` and `obj.a.b = 3`: the
  left side is parsed as an expression and then turned into a `TargetVar`,
  `TargetIndex` or `TargetField`, with `obj?.name = 1` reported as `E0112`. The parsed
  expression stays counted (the oracle's `expr_count`) and becomes the left side of the
  `Binary` for `op=`, so `x += 1` dumps an `Assign` around a `Binary`. Assignments moved
  to `selfhost/assign.kode`. Parsing the port's own source also found `""` dumped as an
  empty `Interp` instead of `Str value=""`, now fixed. New cases `assign_targets`,
  `empty_string` and `e0112_optional_target`.
- **The port parses its own source.** With the above, `kode-stage1 ast` is
  byte-identical to `kode ast` on all 25 files of `selfhost/`, and `./scripts/selfhost.sh
  parse` checks it every run (`self-parse: 25/25`) next to **51/51** cases.

- **The port's arena carries its parent edges (S2 slice 12).** Every node has
  `parent: int?`, set when it is attached, so a later pass can climb from a node to the
  statement that holds it without another tree or reverse edges; the child edge stays
  the ordered `kids` list. Only the root and nodes built and then dropped (the key of a
  `{K: V}` annotation, which is checked and never dumped) have no parent, which a
  throwaway probe over every case confirmed node by node. New `deep_tree` case (generic
  class, methods, interpolation, `match`, nested loops, `spawn`, `await`).
- **A match arm without `=>` reports `E0103`** ("expected `=>` after the pattern,
  found …") over the token found, and ends the whole `match`, as the oracle does; new
  `parse-errors/e0103_match_arm.kode`. That work pushed `stmt.kode` past the size rule,
  so `match` moved to `selfhost/matches.kode`, with `Blocks` offering `statement()` for
  a single-statement arm. **48/48 cases byte-identical**, stage1 261/261 dumps.

- **Typed local declarations in the port's parser (S2 slice 11).** `x: int = 5` now
  parses, with the type node between the target and the value exactly where the oracle
  dumps it, and any annotation the type grammar accepts (`[int]`, `{string: int}`,
  `Box<int>`, `User?`). A missing `=` reports `E0107` ("expected `=` after the type")
  with the `count: int = 0` hint, over the token that turned up. The target is not
  counted for `expr_count`, because `x = 1` reuses the id of the name it parsed while a
  declaration never made one. New cases `selfhost/spec/parse/typed_decl.kode` and
  `parse-errors/e0107_typed_decl.kode`: **46/46 byte-identical**, stage1 259/259 dumps.

- **The port's parser reports errors the way the oracle does (S2 slice 10).**
  `selfhost/report.kode` renders a diagnostic as `kode-diagnostics` does: the code and
  message, `--> file:line:col`, the source line, a caret as wide as the span (one
  character at minimum, columns counted in characters) and the `= help:` line, so
  `kode-stage1 ast` stderr is now byte-identical to `kode ast`'s. Token names follow
  the oracle too (`Cursor.describe`: names and spellings in backticks, `a string`,
  `the end of the file`), and codes, messages, hints and spans were corrected against
  the Rust parser: `E0103` for an unclosed group (no hint), `E0105` for `fn` in a
  block, `E0108` for a two-variable range loop, `E0110` for a missing parameter or
  field name, `E0112` for an unassignable target, `E0116` for a non-string import
  path, and generic statements now report through the expression parser. Recovery
  matches too: a failed item abandons its line, so the same set of errors comes out in
  the same order. New `selfhost/spec/parse-errors` with one case per code (19), and
  `check_parse` compares stderr as well as stdout: **44/44 byte-identical**. Known
  gap: typed local declarations (`x: int = 5`, `E0107`) are still unparsed.

- **`spawn` in the port's parser (S2 slice 9).** `spawn f(1, 2)` dumps as `Spawn`
  with the call's `callee`, `callee_span` and `Arg` children and a span from the
  keyword. The oracle keeps the chain only if it ended as a plain call, so
  `spawn obj.run(1)` and `spawn f().g` are `E0117` ("`spawn` needs a function call")
  over the whole chain, with the documented help text; the port matches code,
  message, span and help on all such probes. `Ast.retype` lets the call node become
  a `Spawn` without a second allocation and rewrites its rendered `span` attribute,
  and `expr_count` is bumped because the oracle does allocate two nodes. Case:
  `selfhost/spec/parse/spawn.kode`, **25/25 trees byte-identical**. With this the
  port parses every prefix form the oracle has, so `spawn` leaves the plan's
  not-yet list.

- **Closures and `await` in the port's parser (S2 slice 8).** `|x: int| x * 2` with
  typed parameters (`E0241` otherwise) and `||` for none, plus `await expr` at unary
  precedence. A closure body is a whole expression, so `catch` and `await` nest in it.
  Both live in `selfhost/closures.kode` and reach the expression parser through an
  `Expressions` interface in `contracts.kode`; interpolation moved to `interp.kode`
  behind the same interface, `kids`/`span_of` moved to the arena, and `expr.kode` is
  back to 453 lines. Cases: `selfhost/spec/parse/{closures,await}.kode`, **24/24 trees
  byte-identical**.

- **Errors and optionals in the port's parser (S2 slice 7).** `throw`,
  `try`/`catch`/`finally`, `a ?? b`, `x is Shape.Circle` and `expr catch err { }`.
  One documented gap: the oracle attaches `catch` at binding power 0, so it can sit
  inside parentheses or an argument list; a `catch` body is a block of statements and
  the statement grammar already imports the expression grammar (module cycles are
  `E0402`), so the port parses `catch` only where a statement's expression starts.
  Cases: `selfhost/spec/parse/{try_catch,optionals,catch_expr}.kode`, **22/22 trees
  byte-identical**.

- **`selfhost/stmt.kode` back under the size rule (437 lines).** The statement-end
  handling moved to the cursor (`end_statement`, `at_statement_end`,
  `recover_line`) and `starts_value` next to the token set, since a statement's end
  is a property of the token stream. No grammar changed: all 22 parse cases stayed
  byte-identical and the `E0101` report keeps its code, position and text.

- **`selfhost/lexer.kode` split by scanner (322 lines, was 558).** The byte values
  moved to `chars.kode` (one definition, imported everywhere, since modules share
  one constant namespace), number literals to `numbers.kode` and string literals to
  `strings.kode`. Both scanners are now pure functions of `src` and an offset that
  return where they stopped plus the token or the errors, so the lexer keeps the
  only mutable cursor. No dump changed: 235 corpus token dumps are byte-identical,
  and the pre-split and post-split binaries agree on stdout, stderr and exit status
  for both `tokens` and `ast` across all 262 `.kode` files in the repository.

- **Byte buffers** (ADR 0043). `bytes(n)` gives `n` zero bytes, with `size()`,
  `get(i)` and `set(i, v)`. The index is bounds-checked and `set` rejects values
  outside `0..=255` instead of truncating them; the object is a packed list, so a
  buffer is freed exactly like a list. `tests/run-pass/bytes_buffers.kode` covers an
  empty buffer, a 100,000-byte buffer, and passing and returning one. The
  `wp08-bytes` gate probe is `met`.

- **Integer literals in bases 2, 8 and 16.** `0b1010`, `0o755` and `0x6b` are
  `int` literals (prefix in either case, `_` between digits); a value that does not
  fit an `int` is E0006. `tests/run-pass/hex_literals.kode`.

- **Generic methods** (ADR 0040 slice 1c). A method may declare its own type
  parameters: `fn id<T>(x: T) -> T => x` inside `class Util` compiles one instance
  per call's type arguments (`Util.id.int`, `Util.id.str`), each an ordinary method
  with `this` first, so a body can read fields and its own parameters together. A
  method of a generic class still cannot declare its own parameters (E0252).

- **Optional numbers as parameters and returns** (ADR 0041 §5). `int?`, `float?`
  and `bool?` cross a call as one boxed pair (`null` for none), so no calling
  convention, function value, closure or interface dispatch changed: the callee
  decodes it into the two locals a pair already uses, and calls through a function
  value box and decode the same way. A list or map of an optional number is still
  E0219. `tests/compile-fail/e0219_optional_int.kode` is gone with the restriction.

- **Optional numbers as class fields** (ADR 0041). `int?`, `float?` and `bool?` are
  no longer locals only: a field takes two slots, the value and its `present` flag,
  and construction, assignment, `this`-reads and `?.` move the two together. So a
  span can hold "no parent" and "line if known" without a second `bool` field. A
  parameter, a return type, or a list of optional numbers is still E0219 (WP-06b).

- **`is` and narrowing** (ADR 0039 §4.4). `s is Shape.Circle` is one tag compare
  and a bool, so it composes with `and`/`not`; inside `if s is Shape.Circle { ... }`
  the value is narrowed and `s.radius` may be read, while an unnarrowed field read
  is `E0419` with the narrowing to write. That closes ADR 0039: payload enums
  construct, print, destructure and narrow, with `E0253` long gone.

- **Pattern bindings on payload enums** (ADR 0039 §4.3). `Shape.Circle(r)` binds
  the variant's fields in declaration order, positionally, as ordinary locals
  inside the arm: `match v { Value.Pair(l, r) => ... }` handles a recursive tree
  with no leaks. The arm still compares the tag, so a binding pattern is one tag
  compare plus one field load per name; a wrong number of names is `E0207`, a
  non-name in a binding position `E0201`, a duplicate name `E0306`. `is` and
  narrowing are the last slice of ADR 0039.

- **Payload enums are values** (ADR 0039, slice 1). `enum Shape { Circle(radius: float) }`
  builds with `Shape.Circle(radius: 1.5)`, prints as it is written
  (`Shape.Circle(radius: 1.5)`, `Shape.Empty`), and `match` compares the variant
  tag. An enum with any payload variant stores every variant in the class layout
  (tag in the object header), so tag-only enums keep the `int` representation and
  the ABI is unchanged; `kode_enum_tag` is the one new runtime symbol. `E0417`
  refuses a payload variant used as a value, `E0420` refuses `==`, and `E0253` is
  gone. Binding fields in a pattern (`Shape.Circle(r)`) and `is` narrowing are the
  next slices.

- **The self-host plan is executable.** `./scripts/selfhost.sh gates` compiles one
  probe per work packet (`selfhost/spec/gates/`) and prints `met`/`blocked` against
  the target each probe declares; `readiness` turns that into the R0-R6 level and
  names the next packet; `all` runs Stage 0, Stage 1 and the gates. The plan in
  `docs/PLAN-self-hosting.md` is the same table, so the script and the document
  cannot drift apart.

- **Generic classes work** (ADR 0040). `class Box<T>` is instantiated once per
  set of type arguments, inferred at the constructor (`Box(value: 1)` is
  `Box.int`); fields and methods are substituted and each method compiles as an
  ordinary function. `E0421`-`E0423` report arity, inference and bare-name
  mistakes. Generic methods, constraints and explicit type arguments are still
  missing.
- **The self-hosting harness runs in CI.** A `selfhost` job records the Stage 0
  oracle, builds the Kode lexer with it and compares 184 token dumps on every
  push and pull request, uploading both measurement files (`.github/workflows/ci.yml`).
- **Enum payload variants parse.** A variant may declare fields,
  `Circle(radius: float)` (ADR 0039, slice 1a): the AST, `kode ast` (text and
  JSON) and the nesting-limit walker carry them. Construction, printing and
  tag-based `match` follow in slice 1 (see the bullet above); destructuring and
  `is` are later slices.
- **Designs for the two language features S2 needs** (ADRs 0039 and 0040): enum
  payloads with `match` destructuring, `is` and narrowing, plus generic classes by
  monomorphization. Design and slice order only — no compiler or runtime change
  yet; each slice stays its own reviewable commit.
- **A lexer written in Kode** (S1 of `docs/PLAN-self-hosting.md`).
  `selfhost/lexer.kode` scans source into tokens with the same spans, escapes,
  error codes and recovery as `crates/kode-syntax/src/lexer.rs`;
  `./scripts/selfhost.sh stage1` builds it with Stage 0 and reports **184 of 184**
  token dumps byte-identical over the corpus plus `selfhost/spec`. `ast` and the
  later commands exit 2 until the parser lands (S2).
- **The self-hosting harness and its oracles** (S0 of
  `docs/PLAN-self-hosting.md`). `kode tokens` and `kode ast` print a stable,
  diffable dump — token kind, text, byte span, line and column, or the tree with
  its spans — in text or with `--format=json`, and `scripts/selfhost.sh
  stage0|stage1|stage2|compare` dumps the whole corpus through them, checks
  Stage 0 against itself first, records wall clock and peak RSS, and refuses to
  compare against a Stage 1 that does not exist yet.

## 0.3.6

The LLVM backend becomes the only code generator, the release workflow learns to
check the tag it is building, and Windows joins CI.

### Changed

- **LLVM is the only backend** (ADR 0035). Code generation goes through textual
  LLVM IR and `clang` (15 or newer, or `KODE_CLANG`); a missing clang is a clear
  error, never a silent fallback. The Cranelift backend, its crates and the
  `--backend=cranelift` option are removed; `--backend=llvm` is still accepted as
  a no-op, and `cargo build` no longer compiles a second code generator.
- **`exec` failures are reported reliably.** A child that cannot be started
  (missing program, bad interpreter) now raises the documented error instead of
  exiting 127 silently on some platforms.
- **Windows builds use the right object format and flags.** `-fPIC` is no longer
  passed there (MSVC-target clang rejects it), clang is asked for
  `x86_64-w64-mingw32` objects so stack probes match MinGW's `___chkstk_ms`, and
  the compiler runs on a 64 MB thread so deeply nested source reports E0118
  instead of overflowing the 1 MB Windows main-thread stack.
- **Start failures are reported as start failures.** A missing or non-executable
  clang, linker driver or `cc` now prints the install hint and exits 1 instead of
  claiming the IR was rejected with status 127. A failing linker still reports
  its own exit status, and a failed build leaves no output file.
- **`kode-codegen` is now `kode-link`**, which is all it does: hand an object
  file and the runtime archive to the platform linker.
- **`cargo build --release` works from a clean tree.** The embedded runtime was
  looked up only in `target/debug`/`target/release`, but a `staticlib` built as a
  build dependency lands in `deps/libkode_rt-<hash>.a`. The build script now
  searches the matching profile's `deps/` folder and honours `CARGO_TARGET_DIR`.

### Added

- **Packages can require a compiler version.** `kode = "0.3"` under `[package]`
  in `kode.toml` states the minimum compiler, like `go 1.24` in `go.mod`;
  `kode doctor` reports and exits 1 when this compiler is older (ADR 0036).
  `kode init` writes the current major.minor into new packages.
- **Three application examples.** `examples/tasks.kode` is a task tracker with a
  JSON store on disk (add/list/done/stats, real exit codes),
  `examples/ledger.kode` turns a CSV ledger into a checked trial balance, and
  `examples/worker.kode` runs a job queue with bounded workers and retries. All
  three are compared line by line by the conformance suite.
- **CI on four platforms.** `.github/workflows/ci.yml` builds, lints, tests,
  format-checks, release-builds and runs a hello world on native Linux x86-64,
  Linux arm64, macOS arm64 and Windows x86-64, installing clang and the linker
  explicitly and asserting each runner's architecture. See `docs/platforms.md`.
- **`--backend=llvm`** (ADR 0035): the flag that introduced the backend in B6 and
  is now also the default.
- **A PostgreSQL client** (ADR 0033): `pg.connect`, `pg.exec`, `pg.query` and
  `pg.close` speak the PostgreSQL wire protocol from the runtime, with
  SCRAM-SHA-256 or cleartext authentication, the extended query protocol with
  `$1` parameters, and JSON results (`command`, `columns`, `rows`, `affected`)
  that keep ints, floats, bools and NULLs typed. Server errors carry the
  SQLSTATE. `sslmode=prefer` (the default) upgrades to TLS when the server
  offers it; `sslmode=require` is still rejected, as are md5, `COPY`,
  notifications and the binary format. A pool is available through `pg.pool`.
  The runtime ABI marker moves to `kode_rt_abi_0004`.
- **HTTPS** (ADR 0034): the HTTP client speaks `https://`, verifying the
  certificate against the webpki roots and the host name, and
  `serve_tls(port, cert, key, handler)` serves HTTPS with DER certificate and
  key files (a handler or a `Router`, like `serve`). A failed handshake names
  the host. The TLS server runs one thread per connection instead of the epoll
  loop; the ADR records why and what it costs.
- **One TLS transport** for both clients: `kode-rt`'s `tls.rs` owns the rustls
  configuration, the root store and the handshake, and the PostgreSQL client
  uses it too. The per-read `Mutex` in the old `pg_tls.rs` is gone.

### Fixed

- **`cargo test --workspace` works from a clean tree.** `kode-llvm`'s
  development binary looked for the runtime archive only in
  `target/<profile>/libkode_rt.a`, but a `staticlib` built as a build
  dependency lands in `deps/libkode_rt-<hash>.a` and is never uplifted, so on a
  fresh checkout every test that compiled a program failed with "cannot find
  libkode_rt.a"; locally a previous `cargo build` hid it. It now searches the
  matching `deps/` folder and takes the newest archive, the same fallback
  `kode-cli`'s build script already had.
- **The Linux release could not produce checksums.** The `.deb` was written as
  `kode-0.3.6-linux-x86_64.deb` while the tarball and the checksum step use the
  tag, `kode-v0.3.6-...`, so `sha256sum` failed after packaging and the release
  job stopped. Artifact names now follow the tag; only the version recorded
  inside the package drops the `v`, because dpkg requires a leading digit.
- **macOS builds would not start.** The macOS 14 runner images were retired
  on 2 November 2026, so the release's macOS job failed before its first step
  with `The configuration 'macos-14-arm64-us-default' is not supported`, which
  skipped `Publish GitHub Release` while Linux and Windows had already produced
  artifacts. The release job and the `ci.yml` macOS entry now use `macos-15`,
  which is still arm64.
- Two diagnostics that sent readers looking in the wrong place now say what is
  actually missing. `Shape.Circle(1)` reports `E0253` "enum payloads are not
  supported yet" with a class-based example instead of "unknown variable
  `Shape`", and `x: u8 = 1` reports `E0303` with the list of real types and a
  note that fixed-width numbers are planned.
- `kode explain` no longer answers "unknown error code" for 15 codes that the
  compiler emits (`E0111`-`E0117`, `E0234`-`E0239`, `E0310`-`E0314`): they were
  missing from `docs/errors/index.md`.
- Deeply nested input no longer crashes the compiler. 300 nested parentheses, a
  2,000-term `+` chain and 500 nested `if` blocks used to abort with a stack
  overflow and exit code 134; they now report `E0118` with a hint. The limit is
  128 levels (ADR 0032), checked both while parsing and by an iterative walk of
  the finished tree that also covers `kode fmt` and `kode lsp`.

## 0.3.5

The `version` in `Cargo.toml` now matches the latest tag. Tags `v0.3.1` through
`v0.3.4` were cut without changelog entries; their changes are described under
0.3.0 and here.

### Fixed

- **Heap values sent through a channel leaked.** `kode_chan_send` retained the
  payload a second time even though the lowering already transfers one
  reference, so every string, list, map or object sent through a channel was
  never freed. `tests/run-pass/channel_heap.kode` covers the transfer.

### Changed

- **Examples refreshed and extended.** `examples/tour.kode` (now the whole
  language, not v0.1) and `examples/bank.kode` use current syntax; new
  `report.kode` (CSV to report), `wordcount.kode` (CLI over a file),
  `pipeline.kode` (tasks and channels), `config.kode` (JSON plus `env`
  override), and `api.kode` gains a `delete` route. `examples/README.md` indexes
  them, and `kode doc examples` has `pub` items to document.

## 0.3.0

### Added

- **Generic functions.** `fn first<T>(xs: [T]) -> T` with type arguments inferred
  at each call and monomorphization: every distinct combination of type
  arguments becomes its own native function, shared by identical calls. Generic
  classes, generic methods, constraints and explicit type arguments are not
  implemented yet (ADR 0030).
- **Language server.** `kode lsp` speaks JSON-RPC on stdio: diagnostics as you
  type, hover, completion, document symbols and formatting, with an unsaved
  buffer read from the editor instead of disk. See `docs/editors.md`.
- **Packages.** `[dependencies]` with a `path`, transitive resolution, a
  `kode.lock`, and `kode init`, `kode new`, `kode add`, `kode remove`,
  `kode install`. Bare imports find a dependency's source root.
- **`finally`.** `try { } catch e { } finally { }`, with `catch` optional when
  only cleanup is needed. It runs on both paths and before `return`, `break` or
  `continue` leaves the block (ADR 0023).
- **Standard library.** Lists gain `contains`, `index_of`, `reverse`, `insert`,
  `remove_at`, `first` and `last`; maps gain `values()`; strings gain `lines`,
  `char_at`, `is_empty` and `count`.
- **`kode doctor`** reports the compiler, target, embedded runtime, linker and
  package/lockfile state.
- **`kode doc <folder> -o <dir>`** writes an HTML reference for `pub` items, and
  `docs/site/build.sh` assembles the spec, ADRs, error index and API page.
- **Windows.** MinGW-w64 linking, a `KODE_CC` override and `docs/windows.md`.
- **Colour** in diagnostics and status lines, following `NO_COLOR`, `TERM=dumb`
  and `CLICOLOR_FORCE`; piped and JSON output stay byte-identical.
- **Benchmarks.** `benchmarks/run.sh` measures wall clock and peak RSS against
  C, Go, Node and Python and writes `benchmarks/report.md`.

- **Channels.** `Channel<T>`, `channel()`, `send`, `recv` and `close`, with
  blocking receive, catchable errors on a closed channel, and tasks that take a
  channel as a parameter (ADR 0026).

- **Enum printing.** `print(color)` and `"${color}"` write `Color.Red`, the way
  the value is written in source (ADR 0027).
- **List slicing.** `items[1..3]`, `items[..2]`, `items[2..]` and `items[0..=2]`
  copy a range of a list; bounds past the ends are clamped (ADR 0027).
- **Function values and closures.** `fn(int) -> int` types, `|x: int| x * 2`
  literals, calling a value (`f(1)`), and passing or returning functions. A
  closure captures the variables in scope by value; it is one heap object holding
  the code address and its captures (ADR 0028).
- **`exec`.** `exec(program, args, cwd)` runs another program without a shell,
  capturing its exit code, stdout and stderr in a `ProcessResult`. A program that
  fails to start throws a catchable error; a non-zero exit is a result, not an
  error (ADR 0029). This is what a self-hosted compiler needs to call `cc` and
  the linker.
- **`map` and `filter`.** `xs.map(|x: int| x * 2)` and
  `xs.filter(|x: int| x % 2 == 0)` on lists of ints, floats, strings, bools and
  reference types, compiled to an inline loop.

- **Optional numbers.** `int?`, `float?` and `bool?`, represented as a value plus a
  `present` flag (no boxing), with `??`, `!`, `== none`, printing, interpolation and
  narrowing inside `if`/`while` branches and after an early return (ADR 0024).

### Fixed

- Writing an item of a `[bool]` list, and iterating one with `for x in list`,
  used the 8-byte accessors on a one-byte-per-item buffer. Small lists returned
  wrong results and a ten-million-element sieve crashed.
- `"abc".char_at(9)` in a `catch` no longer leaks the empty string it returned
  on the error path.

### Changed

- Runtime errors that stop a program (`!` on `none`, division by zero) now say
  what to write instead.
- `kode check` and `kode build` report how long they took.

# Changelog

Release notes for the `kode` compiler. Versions follow the binary, not the
language: a patch release fixes bugs, a minor release may add syntax.

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

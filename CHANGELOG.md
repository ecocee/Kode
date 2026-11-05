# Changelog

Release notes for the `kode` compiler. Versions follow the binary, not the
language: a patch release fixes bugs, a minor release may add syntax.

## 0.3.0

### Added

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

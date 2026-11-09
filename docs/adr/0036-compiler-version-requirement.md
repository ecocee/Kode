# ADR 0036: A package declares the compiler version it needs

Status: accepted

## Context

`kode.lock` pins package dependencies, but nothing records the compiler a project
expects. Kode is still pre-1.0: a minor release may add syntax, and an older
compiler then fails somewhere deep in a build with a parse or type error that
does not mention the real cause. Users install a tarball and have no supported
way to say "this repository needs 0.3 or newer", so CI and teammates silently
run whatever `kode` happens to be on `PATH`.

Go and Rust both solved this in the project file: `go 1.24` in `go.mod` and
`[toolchain] channel` in `rust-toolchain.toml`. Kode already walks up to
`kode.toml` for every build, so the manifest is the natural place.

## Decision

1. **`kode = "0.3"` under `[package]` is the minimum compiler version.** It is a
   minimum, like `go 1.24`: `"0.3"` means 0.3.0 or newer, `"0.3.5"` means 0.3.5
   or newer. There is no upper bound, because a language change that breaks a
   package is a language-version problem, not a range-checking problem.
2. **The value is one to three numbers, digits only.** `0`, `0.3`, `0.3.5` are
   accepted; `0.3.x`, `v0.3` and `0.3.5.1` are not (E0406), so a typo never turns
   into "requirement always satisfied".
3. **`kode doctor` enforces it.** A package whose requirement this compiler does
   not meet fails the check with the required version, the current version and a
   hint to upgrade. The command exits 1 for that, like a missing linker.
4. **`kode init`/`kode new` write the current major.minor.** New packages start
   with the compiler that created them, and the line can be raised deliberately
   when the package starts using newer syntax.
5. **A normal build does not read it yet.** `doctor` is the supported check;
   wiring it into every `kode build` would surprise people building a package
   made on another machine, and it belongs with the toolchain manager work.

## Alternatives

- **An exact pin (`rust-toolchain.toml` style).** Too strict for a language whose
  patch releases are bug fixes: every patch would force every project to edit a
  file, which is why Go chose a minimum.
- **A semver range (`>=0.3, <0.5`).** The compiler is meant to be backward
  compatible within a major version, and the pre-1.0 rule is already "minor may
  add syntax". A range adds a resolver and a failure mode without buying
  correctness.
- **A separate file (`kode-toolchain.toml`).** One more file to find, copy and
  document; the manifest is already located by walking up from the source.
- **Checking in every command.** It would make `kode check` in a package made by
  a newer compiler fail even when the code being checked does not use the new
  syntax.

## Consequences

- A stale toolchain now fails with "needs kode 0.3 or newer, this is 0.2.9;
  upgrade the compiler" instead of a confusing diagnostic, provided the project
  runs `kode doctor` (CI should, and `docs/releasing.md` already asks for it).
- The field is metadata for a future `kodeup`/toolchain manager: it can select a
  compiler from the same line without a second file format.
- `kode doctor` now also exits 1 for a stale `kode.lock` and for an unreadable
  manifest; before, those rows said "no" while the exit code stayed 0.
- Old manifests without the key keep working: the requirement is optional.

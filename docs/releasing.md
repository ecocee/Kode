# Releasing

The compiler is one self-contained binary, so a release is a set of those
binaries plus notes. Nothing is installed at run time.

## Cut a release

1. **Green suite on all three platforms.** `.github/workflows/ci.yml` runs
   `cargo build --workspace`, `cargo test --workspace` with `KODE_LEAK_CHECK=1`,
   and a conformance smoke test on Linux, macOS and Windows (MinGW-w64).
2. **Version.** Bump `version` in the workspace `Cargo.toml`. A patch release
   fixes bugs; a minor release may add syntax. Language changes also need an ADR
   in `docs/adr/`.
3. **Changelog.** Add a section to `CHANGELOG.md` with `Added`, `Changed` and
   `Fixed`. It becomes the release body.
4. **Tag.** `git tag v0.3.0 && git push origin v0.3.0`. The release workflow
   builds five artifacts:

   | Artifact | Runner | Notes |
   |---|---|---|
   | `kode-linux-x86_64.tar.gz` | `ubuntu-latest` | glibc |
   | `kode-linux-aarch64.tar.gz` | `ubuntu-24.04-arm` | glibc |
   | `kode-macos-x86_64.tar.gz` | `macos-13` | Intel |
   | `kode-macos-aarch64.tar.gz` | `macos-latest` | Apple silicon |
   | `kode-windows-x86_64.zip` | `windows-latest` | MinGW-w64 build |

5. **Check one artifact by hand.** Unpack it, run `kode --version`,
   `kode doctor`, and compile a program that uses the standard library. A
   release is published only after that.

## Why one binary per platform, not one universal download

`kode` is native code that links a native runtime archive, so a binary is tied to
its operating system and architecture. That is the same trade the Go and Zig
toolchains make: no VM, no shared runtime, nothing to install alongside it.

## What a release does not include

- A package registry. Dependencies are `path` dependencies today; see
  `docs/adr/0025-git-dependencies.md` for the order of work.
- A cross-compilation sysroot. Build on the platform you ship, as the matrix does.
- Windows binaries built with MSVC. MinGW-w64 is the supported path
  (`docs/windows.md`).

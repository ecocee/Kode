# Releasing

The compiler is one self-contained binary, so a release is a set of those
binaries plus notes. Nothing is installed at run time.

## Cut a release

1. **Green suite on all three platforms.** `.github/workflows/ci.yml` runs
   `cargo build --workspace`, `cargo test --workspace` with `KODE_LEAK_CHECK=1`,
   and a conformance smoke test on Linux, macOS and Windows (MinGW-w64).
   The Windows job has **never been confirmed green**; check it before a release
   and say so in the notes if it is not.
2. **Version.** Bump `version` in the workspace `Cargo.toml`. A patch release
   fixes bugs; a minor release may add syntax. Language changes also need an ADR
   in `docs/adr/`.
3. **Changelog.** Add a section to `CHANGELOG.md` with `Added`, `Changed` and
   `Fixed`. It becomes the release body.
4. **Tag.** `git tag v<version> && git push origin v<version>`. The release
   workflow (`.github/workflows/release.yml`) has **one job today**: Linux
   x86-64. It also pins the source files (README, CHANGELOG, the two editor
   docs) next to the binary.

   | Platform | Release files | Status |
   |---|---|---|
   | Linux x86-64 | `kode-<version>-linux-x86_64.tar.gz`, `kode-<version>-linux-x86_64.deb` | **built by CI** |
   | Linux aarch64 | `kode-<version>-linux-aarch64.tar.gz`, `kode-<version>-linux-aarch64.deb` | planned |
   | macOS Intel | `kode-<version>-macos-x86_64.tar.gz`, `kode-<version>-macos-x86_64.dmg` | planned |
   | macOS Apple silicon | `kode-<version>-macos-aarch64.tar.gz`, `kode-<version>-macos-aarch64.dmg` | planned |
   | Windows x86-64 | `kode-<version>-windows-x86_64.zip`, `kode-<version>-windows-x86_64.exe` | planned, unverified |

   The Linux files are checksummed and published as
   `SHA256SUMS-linux-x86_64.txt`. Per-platform checksum files for the other
   rows are part of the same planned work.

5. **Check one artifact by hand.** Unpack it, run `kode --version`,
   `kode doctor`, and compile a program that uses the standard library. A
   release is published only after that.

   On Linux you can also install the `.deb` and run the compiler directly:
   `sudo dpkg -i kode_<version>_<arch>.deb`, or via
   `apt install ./kode_<version>_<arch>.deb`.

## Why installers, not one universal download

`kode` is native code that links a native runtime archive, so a binary is tied to
its operating system and architecture. That is the same trade the Go and Zig
toolchains make: no VM, no shared runtime, nothing to install alongside it.

Installers are the same shape other native toolchains ship: a tarball you unpack,
a `.deb` you install on Linux, a `.dmg` you drop into `/Applications` on macOS, and
a `.zip` plus a standalone `.exe` on Windows. Only the Linux shapes are produced
by the workflow today; the others are planned and are listed as such above.

## Checksums

Every release file is accompanied by `SHA256SUMS-<platform>.txt` with one
`sha256sum` line per file. The release workflow also publishes a combined
`SHA256SUMS.txt` for the whole release. Find a specific file with
`grep kode-<version>-<platform> SHA256SUMS.txt`.

## What a release does not include

- A package registry. Dependencies are `path` dependencies today; see
  `docs/adr/0025-git-dependencies.md` for the order of work.
- A cross-compilation sysroot. Build on the platform you ship, as the matrix does.
- Windows binaries built with MSVC. MinGW-w64 is the supported path
  (`docs/windows.md`).

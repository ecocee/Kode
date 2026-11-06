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
   workflow (`.github/workflows/release.yml`) builds on two machines — Linux
   x86-64 and macOS arm64 — and a third job publishes both sets with one merged
   checksum file. Each artifact also carries the source files (README, CHANGELOG,
   LICENSE) next to the binary.

   | Platform | Release files | Status |
   |---|---|---|
   | Linux x86-64 | `kode-<version>-linux-x86_64.tar.gz`, `kode-<version>-linux-x86_64.deb` | **built by CI** |
   | macOS Apple silicon (arm64) | `kode-<version>-macos-arm64.tar.gz` | **built by CI** |
   | Linux aarch64 | `kode-<version>-linux-aarch64.tar.gz`, `kode-<version>-linux-aarch64.deb` | planned |
   | macOS Intel | `kode-<version>-macos-x86_64.tar.gz`, `kode-<version>-macos-x86_64.dmg` | planned |
   | Windows x86-64 | `kode-<version>-windows-x86_64.zip`, `kode-<version>-windows-x86_64.exe` | planned, unverified |

   Every published binary is smoke-tested **before** it is packaged, on the
   machine that built it: `kode doctor`, a `--version` that matches the tag, and
   `kode run` on a hello-world program. The macOS tarball is also a plain
   `tar.gz` of the same layout as Linux, so `tar -xzf` is the whole install.

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

Each build machine writes `SHA256SUMS-<platform>.txt` for the files it produced
(`sha256sum` on Linux, `shasum -a 256` on macOS), and the publish job merges
them into one `SHA256SUMS.txt` for the release. Find a specific file with
`grep kode-<version>-<platform> SHA256SUMS.txt`, then verify with
`shasum -a 256 -c SHA256SUMS.txt` on macOS or `sha256sum -c` on Linux.

## What a release does not include

- A package registry. Dependencies are `path` dependencies today; see
  `docs/adr/0025-git-dependencies.md` for the order of work.
- A cross-compilation sysroot. Build on the platform you ship, as the matrix does.
- Windows binaries built with MSVC. MinGW-w64 is the supported path
  (`docs/windows.md`).

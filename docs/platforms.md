# Platforms

What Kode runs on, what each platform needs, and how each claim was checked.
`docs/windows.md` has the Windows-specific setup; this page is the summary.

## Supported targets

| OS | Architecture | Status | How it was validated |
|---|---|---|---|
| macOS 14+ | arm64 | **supported** | native, macOS 27.2 arm64 (M-series), Apple clang 21.0.0 |
| Linux (glibc) | arm64 | **supported** | native, Docker on the same arm64 host, Debian clang 19.1.7 |
| Linux (glibc) | x86-64 | **supported** | **emulated** only (Docker Desktop + Rosetta on arm64), Debian clang 19.1.7 |
| Windows | x86-64 | **preview**, not released | CI target only; has never been run by hand on a Windows machine |

"Emulated" means the compiler and every test ran inside a Linux x86-64 container
whose instructions are translated by Rosetta. It is not the same as a native
x86-64 machine: timing numbers from it are meaningless, and a green run there
does not prove a native run is green. Native x86-64 validation is the main gap,
tracked in `docs/STATUS.md` (5.1, 5.2).

macOS x86-64 (Intel) and Linux-musl are not targets. Cross-compilation is not
supported: `kode build` produces a binary for the host only.

## Required tools

| Tool | Requirement | Override |
|---|---|---|
| Rust | 1.91 or newer (workspace `rust-version`), stable | — |
| clang | **15 or newer**, see `clang --version` | `KODE_CLANG`, then `CLANG`, then `clang` on `PATH` |
| C compiler / linker driver | `cc` (macOS Command Line Tools, gcc on Linux), MinGW-w64 `gcc` on Windows | `KODE_CC`, then `CC` |
| `strip` | optional; only `kode build --release` uses it on Unix | — |

`kode` shells out to clang for every build and to the C compiler for linking; it
links no LLVM libraries into itself, which is why the binary stays around 1 MB.
`kode doctor` prints the resolved clang and linker driver with their versions.

### macOS

```bash
xcode-select --install          # clang, cc, strip
rustup toolchain install stable
cargo build --release -p kode-cli
```

Apple clang 15+ is supported; 21.0.0 is what the current numbers were measured
with. Homebrew's LLVM also works: `KODE_CLANG="$(brew --prefix llvm)/bin/clang"`.

### Linux

```bash
sudo apt-get install -y clang gcc libc6-dev    # Debian/Ubuntu
rustup toolchain install stable
cargo build --release -p kode-cli
```

Any distribution clang 15 or newer works; Debian's clang 19.1.7 is what CI
installs. `gcc` (or another `cc`) is needed for linking only.

### Windows

Windows is a preview target: the compiler builds and links through MinGW-w64, but
no released artifact exists and the result has not been run on real hardware.
Follow `docs/windows.md`:

```powershell
winget install BrechtSanders.WinLibs.POSIX.UCRT   # gcc + linker
winget install LLVM.LLVM                          # clang 15+
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build --release -p kode-cli
```

The GNU Rust toolchain is required: the runtime archive is linked by gcc, and an
MSVC-built archive references symbols MinGW cannot resolve (`__chkstk`,
`__CxxFrameHandler3`).

## Continuous integration

`.github/workflows/ci.yml` runs on every push to `main` and every pull request:

| Job | Runner | Actual architecture |
|---|---|---|
| `test (linux-x86_64)` | `ubuntu-latest` | x86-64, native |
| `test (linux-aarch64)` | `ubuntu-24.04-arm` | arm64, native |
| `test (macos-aarch64)` | `macos-14` | arm64, native |
| `test (windows-x86_64)` | `windows-latest` | x86-64, native |

Each job asserts `uname -m` matches the architecture it claims before building,
installs clang explicitly (apt on Linux, Chocolatey LLVM on Windows, the system
Command Line Tools on macOS after verifying they exist), and then runs the same
gates in the same order:

1. `cargo build --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace` with `KODE_LEAK_CHECK=1`
4. `cargo test -p kode-cli --test llvm_conformance` (run-pass output, compile-fail codes)
5. `kode fmt tests/run-pass examples --check`
6. `cargo build --release -p kode-cli`
7. build `examples/hello.kode` with the release binary, run it, compare output, `kode doctor`

There is no skip list and no `continue-on-error`: a failing platform is a red
build. The one environmental constraint is `ubuntu-24.04-arm`, which GitHub only
offers to public repositories; in a private fork that job will not start, and the
arm64 Linux gate has to run on a self-hosted arm64 runner instead.

## Known limitations

- **Windows** has not been exercised end to end on real hardware. `ci.yml` runs
  it, but a workflow that has never executed is not evidence. Treat Windows as
  preview until a green run is recorded.
- **Linux x86-64** was validated under Rosetta emulation only; see above.
- `exec` and the compiler's own clang/linker calls force the fork/exec path so a
  failed `exec` is reported as an error. On Windows there is no fork; `std`'s
  normal error reporting applies.
- Signals: a program killed by a signal reports `code = -1` from `exec`.
- `kode build` writes the `.ll` and object files next to the output, so the
  output directory must be writable even for `kode run` (which uses a temporary
  directory).
- Old clang versions (14 and older) are untested and may reject the generated IR.

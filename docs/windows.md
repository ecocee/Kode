# Windows

Kode builds Windows executables through **MinGW-w64**, the same way Go and Zig
start: a C toolchain and linker, no Visual Studio project files.

## Status

| Step | macOS / Linux | Windows |
|---|---|---|
| Compile to an object | works | works (same LLVM backend) |
| Link | `cc` | `gcc` from MinGW-w64, with `-static` and the Windows system libraries |
| Run the result | works | expected to work; see "What is not tested" |
| Paths in diagnostics | `/` | `\` and `/` both accepted |

The linker settings live in `crates/kode-link/src/link.rs`
(`platform_args`), and the driver can be overridden:

```
KODE_CC=C:\msys64\mingw64\bin\gcc.exe kode build hello.kode
```

`kode doctor` prints which driver it will use and its version, so a broken
setup is visible before the first build.

## Setting up MinGW-w64

1. Install one of:
   - `winget install BrechtSanders.WinLibs.POSIX.UCRT` (standalone, no MSYS),
   - MSYS2, then `pacman -S mingw-w64-ucrt-x86_64-gcc`, or
   - the LLVM-MinGW release archive.
2. Put the folder that contains `gcc.exe` on `PATH`.
3. Use the GNU Rust toolchain too. The runtime library is linked by `gcc`, and an
   MSVC toolchain (`stable-x86_64-pc-windows-msvc`, the default on Windows)
   builds a runtime MinGW cannot link: it references MSVC-only symbols such as
   `__chkstk` and `__CxxFrameHandler3`.

   ```
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

4. Check: `gcc --version`, then `kode doctor`.

MSVC (`cl.exe`) is **not** supported: it needs a different object format setup
and a `.lib` runtime. The error message says so instead of failing with a linker
dump.

## What is not tested

- A `windows-latest` CI job exists (GNU toolchain + MinGW-w64) and runs the full
  suite, so the compiler and linker are exercised there. **That job has never
  been confirmed green**, and the linked binaries are still mostly not run on
  Windows by hand. Treat Windows as unverified until a green run is linked in
  `docs/releasing.md`.
- Cross-compiling *to* Windows from macOS/Linux needs a MinGW-w64 cross
  toolchain (`x86_64-w64-mingw32-gcc`) and `KODE_CC` set to it. The runtime
  archive is built for the host, so a cross build is not wired up yet.
- Console colour: `kode` relies on the terminal reporting itself as a TTY;
  Windows Terminal does, older `cmd.exe` may not.
- Paths longer than 260 characters are not handled specially.

## Releases on Windows

**Windows artifacts are planned, not built.** `.github/workflows/release.yml`
builds Linux x86-64 and macOS arm64, so a release currently contains no Windows
file. The planned shape is:

- `kode-<version>-windows-x86_64.zip` — the `kode.exe` binary plus README,
  CHANGELOG, the two editor docs.
- `kode-<version>-windows-x86_64.exe` — the `kode.exe` binary by itself, so it
  can be downloaded and dropped in a folder without unzipping.

The binary would be a single MinGW-w64 build, statically linked by default,
needing no C runtime beside it. `dist/` in the repository holds an old 0.3.0
build from before this was automated; it is not a release artifact.

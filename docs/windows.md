# Windows

Kode builds Windows executables through **MinGW-w64**, the same way Go and Zig
start: a C toolchain and linker, no Visual Studio project files.

## Status

| Step | macOS / Linux | Windows |
|---|---|---|
| Compile to an object | works | works (same Cranelift backend) |
| Link | `cc` | `gcc` from MinGW-w64, with `-static` and the Windows system libraries |
| Run the result | works | expected to work; see "What is not tested" |
| Paths in diagnostics | `/` | `\` and `/` both accepted |

The linker settings live in `crates/kode-codegen/src/link.rs`
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

- CI runs the test suite on `windows-latest` with the GNU toolchain and
  MinGW-w64, so the compiler and the linker are exercised; the linked binaries
  are still mostly not run on Windows by hand.
- Cross-compiling *to* Windows from macOS/Linux needs a MinGW-w64 cross
  toolchain (`x86_64-w64-mingw32-gcc`) and `KODE_CC` set to it. The runtime
  archive is built for the host, so a cross build is not wired up yet.
- Console colour: `kode` relies on the terminal reporting itself as a TTY;
  Windows Terminal does, older `cmd.exe` may not.
- Paths longer than 260 characters are not handled specially.

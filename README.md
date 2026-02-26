# Kode

> A concurrency‑first, statically typed compiled language for backend & distributed systems  
> Compiles to idiomatic Go **or** bytecode. Modern tooling suite. *(2026 edition)*

---

## 🚀 What is Kode?

Kode is an open-source programming language designed from the ground up for high‑throughput
servers, microservices and distributed applications. It provides:

- **Static typing** with sophisticated inference
- **Built‑in concurrency** modeled after CSP using goroutines & channels
- **Bytecode compilation** with Go-independent stack-based VM (default)
- **Go code generation** option for native performance and ecosystem compatibility
- **Modern tooling**: Colored CLI, project generators, formatter and more

## 🎯 Why choose Kode?

- Familiar syntax that's easy to read and write
- Seamless interop with Go and its package ecosystem
- Lightweight runtime with preemptive scheduler
- Focus on predictable performance and developer productivity
- Ideal for cloud services, data pipelines, and systems programming

## 🌟 Key Features

- **Static type system** with Hindley‑Milner inference
- **First‑class concurrency** primitives (`go`, `chan`, `select`)
- **Bytecode VM execution** - portable, no Go compiler needed (default)
- **Full operator support** - arithmetic, bitwise, logical, comparison
- **Complete control flow** - if/else, for/while loops, functions
- **Modern CLI** with colored output and helpful error messages
- **AST → IR → Bytecode** compilation pipeline (or legacy Go backend)
- **Cross‑platform support** (Windows, macOS, Linux)

## 📅 Roadmap & Phases

Kode is released under a semantic‑versioned roadmap:

1. **v0.2 – Core language** (2024‑2025): lexer, parser, typer, IR, Go backend, runtime scheduler
2. **v0.3 – Concurrency & stdlib** (current, 2026): full channel select, HTTP, collections
3. **v0.4 – Optimization & packages**: LLVM backend option, package manager, tooling
4. **Future phases**: JIT, actors, cloud‑native SDKs, web playground

For details see [docs/roadmap.md](./docs/roadmap.md).

## 📦 Installation

```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
```

## 🛠️ Getting Started

```bash
kode new myproject      # scaffold a new project
cd myproject
kode build .            # generate Go code and compile
./myproject             # run binary
```

## 📂 Example

```kode
fn main() {
    let ch: chan<int> = chan.new();
    go fn() {
        ch <- 42;
    }();
    let x = <-ch;
    print x;
}
```

### CLI Reference

```bash
kode run path/to/file.kode           # compile & execute
kode build path/to/file.kode         # compile to .kbc bytecode (default)
kode build --go path/to/file.kode    # compile to Go binary (legacy)
kode file.kbc                        # shorthand to execute bytecode
kode exec file.kbc                   # explicit bytecode execution
kode fmt path/to/file.kode           # format source
kode check path/to/file.kode         # type check only
kode clean                           # remove generated artifacts
kode version                         # show version
```

> Full CLI documentation: [docs/cli.md](./docs/cli.md)

## 📚 Documentation

Detailed guides live under `docs/`:

- [Syntax & grammar](./docs/syntax.md)
- [Architecture overview](./docs/ARCHITECTURE.md)
- [Roadmap & phases](./docs/roadmap.md)
- [Bytecode format](./docs/bytecode.md)
- [Complete wiki](./docs/wiki.md)

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before
getting started.

## 📊 Project Status

As of **2026**, Kode is in beta (v0.3.0) and actively developed. See
[CHANGELOG.md](CHANGELOG.md) for recent updates.

## 📄 License

MIT License © 2026 ECOCEE

---

*Created with ❤️ by the Kode team*

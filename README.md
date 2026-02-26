# Kode

Kode is a statically typed, concurrency-first compiled language for backend and distributed systems development.

## Features

- Static typing with type inference
- Built-in concurrency model with goroutines and channels
- HTTP server primitives
- Clean, idiomatic Go-based compilation

## Installation

```bash
go install github.com/ecocee/kode-go/cmd/kode@latest
```

## Usage

### Run a Kode program

```bash
kode run examples/concurrency.kode
```

### Build to Go code

```bash
kode build examples/concurrency.kode
```

### Create a new project

```bash
kode new myproject
```

## Architecture

Kode compiles to Go code via an AST → IR → Go code generation pipeline. The runtime provides a lightweight scheduler and channel abstractions.

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Roadmap

See [ROADMAP.md](ROADMAP.md).

```bash
kode run hello.kode
```

### Interactive REPL

```bash
kode repl

> let x = 5;
> let y = 10;
> print x + y;
15
```

## 📚 Documentation

For more detailed information, check out these resources:

- [Language Syntax](./docs/syntax.md)
- [CLI Reference](./docs/cli.md)
- [Complete Wiki](./docs/wiki.md)
- [Development Roadmap](./docs/roadmap.md)
- [Bytecode Format](./docs/bytecode.md)

## 🤝 Contributing

Contributions are welcome! Please check our [contribution guidelines](CONTRIBUTING.md) before getting started.

## 📊 Project Status

Kode is currently in beta (v0.1.0). See the [changelog](CHANGELOG.md) for recent updates and our [roadmap](./docs/roadmap.md) for future plans.

## 📄 License

Kode is released under the MIT License.

---

*Created with ❤️ by ECOCEE*

© 2025 Kode Programming Language

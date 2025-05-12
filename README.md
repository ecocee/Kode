# 🚀 Kode Programming Language

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-beta-orange)
[![Kode on Product Hunt](https://api.producthunt.com/widgets/embed-image/v1/featured.svg?post_id=964499&theme=light)](https://www.producthunt.com/posts/kode-3?utm_source=badge-featured&utm_medium=badge&utm_souce=badge-kode-3)

> A modern, interpretable programming language with C-style syntax and functional capabilities

**Created by Sreeraj V Rajesh**

## 📋 Overview

Kode is a lightweight, interpreted programming language designed with readability and simplicity in mind. It features a clean C-like syntax with influences from JavaScript, Python, and Rust, making it approachable for developers from various backgrounds.

### Key Features

- **Familiar Syntax**: C-style syntax that feels natural to most programmers
- **Dynamic Typing**: Flexible variable handling without type declarations
- **First-Class Functions**: Supports closures and function passing
- **Bytecode Compilation**: Compile to `.kdc` bytecode for faster execution
- **Interactive REPL**: Experiment with code in real-time
- **Module System**: Import and use code from other files
- **Error Handling**: Built-in try/catch mechanism

## 🛠️ Installation

```bash
# Clone repository
git clone https://github.com/cyberkutti-iedc/kode
cd kode

# Build with Cargo
cargo build --release

# Run a sample program
./target/release/kode run examples/hello.kode
```

## 🚀 Getting Started

### Hello World

```kode
fn main() {
    print "Hello, World!";
}
```

### Run Your First Program

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

*Created with ❤️ by Sreeraj V Rajesh*

© 2025 Kode Programming Language

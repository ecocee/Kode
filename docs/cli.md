# 📄 Kode CLI Commands Reference

This document provides usage details for all CLI commands available in the `kode` binary.

## 🔧 Commands

| Command               | Description                                |
|------------------------|--------------------------------------------|
| `kode run <file>`     | Runs a `.kode` source file (type checks & executes) |
| `kode build <file>`   | Compiles `.kode` into Go and builds a binary      |
| `kode version`        | Prints the version number and build info         |
| `kode help`           | Displays CLI usage instructions                 |

## ⚙️ Options

| Flag           | Description                                 |
|----------------|---------------------------------------------|
| `--verbose`    | Prints additional internal debug information|
| `--optimize`   | Enables code optimization (if supported)    |
| `--no-run`     | Only compile, do not execute                |
| `--time`       | Shows execution time                        |

## Examples

```bash
kode run examples/hello.kode --verbose
kode build examples/main.kode --no-run

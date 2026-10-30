# test

A [Kode](https://github.com/ecocee/kode) project.

## Usage

```bash
# Run directly (interpreted)
kode run

# Build to bytecode
kode build

# Execute bytecode
kode target/main.kbc
```

## Project structure

```
test/
├── main.kode       # entry point
├── src/            # additional source files
├── target/         # build output (.kbc files)
├── kode.toml       # project configuration
├── .gitignore
└── README.md
```

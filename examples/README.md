# Examples

Small programs, each about one idea. Every one runs from the repository root with
the compiler built (`cargo build --workspace`).

| File | What it shows | Run it |
|---|---|---|
| `hello.kode` | the smallest program | `kode run examples/hello.kode` |
| `tour.kode` | the whole language in one file: values, control flow, functions, generics, interfaces, collections, optionals, errors | `kode run examples/tour.kode` |
| `fib.kode` | recursion and a loop | `kode run examples/fib.kode` |
| `bank.kode` | classes, enums with `match`, maps, errors that leave state untouched | `kode run examples/bank.kode` |
| `report.kode` | turning CSV text into a report: maps, `keys`, `sort`, string methods, printing a map | `kode run examples/report.kode` |
| `wordcount.kode` | a command-line tool: `args()`, `read_file`, error handling with `exit` | `kode run examples/wordcount.kode -- README.md` |
| `pipeline.kode` | tasks and channels: fan out to three workers, collect and sort the results | `kode run examples/pipeline.kode` |
| `config.kode` | JSON config, an `env()` override, validation with error codes, `finally` | `kode run examples/config.kode` |
| `api.kode` | a JSON HTTP API with `Router`, `Request` and `Response` | `kode run examples/api.kode -- 8080` |

`api.kode` keeps running until you stop it (`Ctrl+C`). The others print and exit.

Notes that save time:

- A Kode program's entry point is its **top-level code**, not a `fn main()`. A file
  that defines `main` and nothing else does nothing.
- Add `-- a b` after the file to pass program arguments: `args()` is `["a", "b"]`.
- `print(xs)` and `"${xs}"` show lists and maps (`[1, 2]`, `{"name": "Asha"}`); see
  `docs/adr/0031-printing-collections.md`.
- `kode fmt examples` keeps these files in the project's style.
- `kode doc examples -o /tmp/api-docs` generates an HTML reference from `pub` items.

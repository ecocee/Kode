# Contributing to Kode

## Development Setup

1. Clone the repository
2. Install Go 1.21+
3. Run `go mod tidy`
4. Build with `make build`
5. Test with `make test`

## Code Style

- Follow Go conventions
- Use `go fmt`
- Pass `go vet` and `golangci-lint`

## Testing

- Add unit tests for new features
- Use table-driven tests
- Ensure integration tests pass

## Submitting Changes

1. Create a feature branch
2. Write tests
3. Ensure CI passes
4. Submit PR

2. Make your changes and commit them:

```bash
git add .
git commit -m "Add feature: my new feature"
```

3. Push to your fork:

```bash
git push origin feature/my-new-feature
```

4. Open a pull request on GitHub.

## 🧼 Code Style

* Use **Rust idioms** and follow the [Rust style guide](https://doc.rust-lang.org/1.0.0/style/style/index.html)
* Format code with `cargo fmt`
* Lint with `cargo clippy` if possible
* Use `snake_case` for function and variable names
* Write descriptive comments for non-obvious code
* Keep functions small and focused on a single task

## 🧭 Where to Contribute

* **Bug fixes**: Check the [issues](https://github.com/ecoceevr/kode/issues) for reported bugs
* **Documentation**: Improve README, docs, or inline code comments
* **Examples**: Add example programs to showcase language features
* **Language features**: Implement items from the [roadmap](docs/roadmap.md)
* **Standard library**: Help build the standard function library
* **Performance**: Optimize the interpreter or bytecode execution
* **Testing**: Add more unit and integration tests

## 📜 Licensing

By submitting your code, you agree to license your contribution under the [MIT License](LICENSE).

## 🙋 Need Help?

Open an issue with the `question` label or reach out to the maintainers for guidance.

---

Thanks again for your interest in contributing to Kode! 🎉

*Document maintained by Sreeraj V Rajesh*
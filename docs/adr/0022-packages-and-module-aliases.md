# ADR 0022: Packages use `kode.toml` and qualified module aliases

Status: accepted for v0.6

## Context

Imports already load files and merge every declaration into one namespace. That is easy to
understand but gives packages neither a root nor a boundary: `import "models/user"` is only a
relative path, and there is no way to say which names a module exposes.

A full package manager (dependencies, versions, a registry, lockfiles) is a larger project and
needs network and supply-chain decisions. v0.6 takes the part that a compiler can own: a
package is a folder with a manifest, and modules can be reached by a qualified name.

## Decision

1. **A package is a folder with `kode.toml`.** The compiler walks up from the entry file and
   uses the nearest manifest:

   ```toml
   [package]
   name = "shop"
   version = "0.1.0"
   root = "src"
   ```

   `name` and `version` are metadata; `root` (default `src`) is the source folder.
2. **Bare imports are package-relative; `./` and `../` are file-relative.** The syntax already
   distinguishes the two, so no existing program changes meaning. A missing root is E0407.
3. **`import "x" as name` is a qualifier, not a value.** `name.fn(...)`, `name.CONST` and
   `name.Class(...)` resolve to declarations in that module. Because modules share one namespace,
   aliases do too: they are program-wide names that may not hide a declaration (E0409) or repeat
   (E0408). A local variable with the same name shadows the alias.
4. **`pub` gates module access.** The loader records which declarations are public. Qualified
   access and `from ... import` only see public names (E0411); a name the file never defines is
   E0405 or E0410. `pub` is tracked on functions, constants, classes, interfaces and enums.
5. **The shared namespace stays.** `import "./x"` still merges every declaration, as ADR 0011
   describes. Making non-public items file-private needs module-local name resolution and is
   deliberately left for a later milestone; enforcing it now would silently break existing code.

## Consequences

- A package works with no registry, no lockfile and no network access.
- Qualified access reads like the standard-library namespaces (`json.parse`), so the language has
  one spelling for both.
- A module's private helpers remain reachable by an unqualified name today. The documented rule
  is that `pub` controls import surfaces, not the shared namespace; the next step is to give each
  file its own scope.
- Dependencies, versions that affect resolution, and publishing are still out of scope.

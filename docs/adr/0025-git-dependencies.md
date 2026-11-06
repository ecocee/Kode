# 0025 — Git dependencies first, a registry later

Status: accepted, scheduled

## Context

`kode.toml` supports `path` dependencies and `kode.lock` (ADR 0022). That makes
one repository work, but nothing can be reused across machines: every dependency
has to be cloned by hand and pointed at. A registry is the long-term answer and
also the largest piece of infrastructure the project could build, so it must not
come first.

## Decision

Three steps, each useful on its own:

1. **Git dependencies.** `utils = { git = "https://...", tag = "v1.2.0" }` or
   `rev = "<sha>"`. `kode install` clones into `~/.cache/kode/git/<name>-<sha>/`
   and records the resolved commit in `kode.lock`. Offline installs work once the
   cache is warm. A `branch` is allowed but resolves to a commit in the lock, so
   builds are reproducible.
2. **Tarball dependencies.** `url = "https://.../pkg-1.2.0.tar.gz"` with a
   `sha256` that must be present. This is what a registry would serve, so it
   validates the format before the registry exists.
3. **A registry** only when step 1 has real users. It serves the same tarball
   format plus metadata; publishing is `kode publish` with a token from the
   environment, never from `kode.toml`.

Rules that apply from step 1:

- Resolution stays deterministic: the lockfile records the exact commit or hash,
  and `kode install --check` fails when it is stale.
- The dependency key must equal the package's own name (E0414), and every
  dependency must be a package with a `kode.toml`.
- No code runs during resolution: cloning, extracting and checking a hash only.
- The compiler stays dependency-free. Cloning and hashing use the runtime the
  standard library already has; if TLS is not available, git dependencies are
  documented as requiring `git` on `PATH` (a deliberate, visible trade).

## Consequences

- Git dependencies need either TLS or an external `git`. Choosing `git` on `PATH`
  for step 1 keeps the compiler dependency-free and is honest about it in
  `kode doctor`.
- The lockfile grows a `source` field per dependency (`path`, `git`, `url`), with
  the resolved revision. Old lockfiles stay valid because `path` is the default.
- A registry is out of scope for v1.0; this ADR exists so that step 1 does not
  paint the format into a corner.

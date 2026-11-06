# Security policy

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Email **security@ecocee.in** with:

- the Kode version (`kode --version`) and platform,
- a small program that reproduces it, if one exists,
- what you expected and what happened,
- whether you are willing to be credited.

You can also use GitHub's private vulnerability reporting on
`github.com/ecocee/Kode` (Security → Report a vulnerability). Either channel is
private; email is preferred if GitHub is unavailable to you.

We aim to acknowledge within 7 days and to agree on a disclosure date within 30.
Please give us that window before publishing details.

## What counts as a security issue

Kode compiles programs to native binaries and embeds the runtime in each one, so
the attack surface is unusually concrete:

- **The compiler**: crashes, stack exhaustion, or wrong code generation from
  untrusted `.kode` input; a program that should fail to compile but does not.
  A crash is a denial of service when a service compiles user-supplied source.
- **The runtime**: memory-safety bugs in strings, lists, maps, classes, channels,
  the HTTP server/client, JSON parsing, or `exec`; reference-count over-release
  or under-release that a program can trigger.
- **The standard library**: path traversal in file APIs, unbounded reads, unsafe
  parsing of network input, header or request-smuggling bugs in the HTTP server.
- **Distribution**: a release artifact that does not match its checksum, or a
  build that embeds a different runtime than the one in the tag.

Not security issues: a program that loops forever, an out-of-memory condition
caused by a program the user wrote, or a compiler abort on pathological nesting
(that is a robustness bug — see `docs/STATUS.md` — still worth reporting, but it
is not a vulnerability).

## Supported versions

Only the latest release tag receives fixes. Because programs embed the runtime,
a runtime fix means **rebuilding the program**; there is no shared-library patch
channel. Security releases say which tag contains the fix.

## Scope of this policy

This covers the compiler, runtime and tooling in this repository. Programs
written in Kode, and third-party libraries they use, are the responsibility of
their authors.

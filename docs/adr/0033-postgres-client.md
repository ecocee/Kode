# ADR 0033: A PostgreSQL client in the runtime

## Context

Plan item 9.4 says a language without a data store cannot back a real service.
Kode had no way to reach any database: no driver, no `extern fn` to call libpq,
no raw sockets in the language, no byte buffers, and no TLS. Every backend in
`benchmarks/database/` other language has a driver; Kode was reported
UNSUPPORTED.

What the runtime already had: TCP (`std::net::TcpStream`) for the HTTP client
and server, JSON values with a parser, reference-counted strings/lists/maps, and
a single table (`kode-sema/src/std_fns.rs`) that tells the compiler about every
runtime function.

## Decision

Add a PostgreSQL protocol client to the runtime, not to the compiler and not as
an external process:

- `pg_wire.rs` speaks protocol version 3 over plain TCP: startup, cleartext and
  SCRAM-SHA-256 authentication, the simple query protocol, and the extended
  query protocol with text parameters and text results.
- `pg.rs` exposes it to Kode as four free functions, plus `pg.*` aliases:

  ```kode
  db = pg.connect("postgres://user:pass@host:5432/app?sslmode=disable")
  pg.exec(db, "create table t (id int)")            // simple query
  rows = pg.query(db, "select * from t where id = $1", [json.int(1)])
  pg.close(db)
  ```

- A connection is an `int` handle into a table of open connections. Each call
  locks only its own connection, so two tasks can use two connections and a
  shared connection is never used by two queries at once.
- Results are one JSON object: `command`, `columns`, `rows` (an object per row,
  keyed by column name) and `affected`. `bool`, `int2/4/8`, `oid`, `float4/8`
  map to JSON bool/int/float; NULL maps to JSON null; everything else stays the
  text PostgreSQL sent, so `numeric`, `uuid`, `timestamp` and `bytea` round-trip
  without losing precision. Parameters are JSON values sent in text format,
  `json.null()` binding SQL NULL.
- Server errors become Kode `Error`s whose message starts with the SQLSTATE
  (`42P01: relation "x" does not exist`), so `try`/`catch` can match on it, and
  the connection stays usable.
- **No third-party crates.** SCRAM needs SHA-256, HMAC and PBKDF2, so
  `sha256.rs` implements them from FIPS 180-4, RFC 2104 and RFC 8018, pinned to
  the published vectors (including the RFC 7677 SCRAM exchange). `base64.rs` is
  the RFC 4648 codec SCRAM needs. Neither is used for encryption or for
  verifying untrusted input.

The runtime ABI marker moves to `kode_rt_abi_0004` because the runtime gained
functions.

## Alternatives

- **`extern fn` + libpq (C FFI).** The language has no FFI, and adding one is a
  bigger change than the driver (ADR needed, platform headers, linking).
- **A Kode-written driver.** Impossible today: no sockets, no byte buffers, no
  fixed-width integers in the language. This is exactly why the driver lives in
  the runtime.
- **An HTTP gateway in front of PostgreSQL.** Adds a hop and measures the
  gateway, not Kode.
- **`exec("psql", ...)` per query.** Measures process creation; rejected as a
  benchmark strategy (kept in `benchmarks/database/kode/probe.sh` only as a
  feasibility probe).
- **Adding RustCrypto (`sha2`, `hmac`) as a dependency.** Would be the usual
  choice, but the runtime links no third-party crates by design; the request to
  change that belongs to the maintainer, not to a feature branch.

## Consequences

- Programs that do not use `pg` link no driver code: `examples/hello.kode`
  still builds to 373,344 bytes stripped with zero `kode_pg_*` symbols. A
  program that does use it is 534,448 bytes stripped (+161 KB), which includes
  the socket, SCRAM and JSON result paths.
- **No TLS.** `sslmode=prefer` (the default) sends an SSLRequest and, if the
  server offers TLS, fails with a message telling the user to add
  `sslmode=disable`. `sslmode=require` is rejected up front. Connecting to a
  managed database therefore needs a local TLS-terminating proxy until the TLS
  item (9.3) lands.
- **No md5 authentication**: PostgreSQL 14+ defaults to SCRAM, and md5 is
  rejected with an explanation. SASLprep is not applied to the password, so
  non-ASCII passwords must be avoided for now.
- Not supported yet: `COPY`, the binary format, notifications, `LISTEN`/
  `NOTIFY`, cancellation, statement-level timeouts, connection pooling, and
  arrays/composite types as parameters.
- Ownership is explicit: a connection is closed by `pg.close`, not when a value
  goes out of scope (the language has no destructors). An unclosed connection is
  closed by the OS at exit; the leak checker does not track it.
- The benchmark report can now run a Kode database client instead of reporting
  it unsupported.

## Measurement

`examples/hello.kode` (no `pg`) and `examples/db/postgres.kode` (uses `pg`),
built with the same debug compiler and stripped with `strip -x`:

| Binary | Bytes |
|---|---|
| hello, no driver linked | 373,336 |
| postgres example | 534,440 |
| difference | 161,104 |

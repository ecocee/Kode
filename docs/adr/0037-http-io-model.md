# ADR 0037: The HTTP server I/O model

Status: **proposed** — the measurements are done and recorded below; the decisions
in §4 need the maintainer's approval before any of them is treated as settled.
Date: 2026-11-11.
Plan item: `docs/plan-fix-http-concurrency.md`, Stage 2.
Evidence: `benchmarks/http/report.md` ("Stage 2 — I/O model") and
`benchmarks/http/stage2.sh`.

## 1. Context

`serve(port, handler)` and `serve_tls(port, cert, key, handler)` are the only
long-running server paths in the runtime. Everything a service does at scale
depends on how those two loops accept, read, write and shed connections.

When Stage 2 was written, the plan assumed one model existed
(thread-per-connection) and asked for an ADR plus measurements to choose among
three options. Since then the epoll loop landed on Linux without an ADR, so this
record has to do two jobs: document what actually ships, and decide what is
still open. `crates/kode-rt/src/http_wire.rs` contains:

| Path | Lines | Where | Model |
|---|---:|---|---|
| `mod epoll` | 435 | `cfg(target_os = "linux")` | readiness loop, non-blocking sockets, one epoll instance per worker thread, `EPOLLEXCLUSIVE` accept, `write` per response |
| `threaded_loop` | 35 | macOS, Windows | blocking, one thread per accepted connection |
| `tls_event_loop` | 48 | every OS | blocking, one thread per connection, 5 s handshake timeout (ADR 0034) |

Shared by all three: HTTP/1.1 parsing with limits (`8 KiB` request line,
`32 KiB` headers, `100` headers, `8 MiB` body), keep-alive up to 10,000 requests
per connection, `MAX_ACTIVE_CONNECTIONS = 1024`, and workers from
`KODE_HTTP_WORKERS` (default `max(4, available_parallelism)`).

Constraints that shaped this decision:

- One self-contained binary per program. A new runtime crate is added to *every*
  compiled program, so it needs approval and a binary-size number.
- No platform TLS, no async runtime, no `tokio`. The runtime is blocking code
  with hand-written readiness where it matters.
- Windows has no `epoll`/`kqueue`; the equivalent is IOCP, which shares no code
  with either loop.
- A TLS session is a state machine, so it cannot join the fd-level epoll loop
  without making the parser transport-agnostic (ADR 0034 decided against it).

## 2. What Stage 2 asked for

> **Stage 2 (ADR + measurements, no commitment)**
> Compare on 1 vCPU / 1 GB server container, load gen in separate container:
> thread-per-connection (bounded), event loop (epoll/kqueue) + worker pool,
> event loop with `mio`. Measure RSS and peak RSS at 100/1000/10000 live idle
> keep-alive connections, RPS and p50/p99/p99.9 at c=100 and c=1000, CPU time
> per request, binary size delta, LOC, Windows story. Recommend one with
> rationale. ADR required.

## 3. Measurements

Protocol: server in its own container, `--cpus=1 --memory=1g`, load generator in
a second container on the same network, `oha` 1.16.0 and `wrk` 4.2.0 as the two
independent generators. Full tables, machine details and the exact command are
in `benchmarks/http/report.md`; the numbers that drive this decision are:

**Linux, epoll loop, 1 vCPU, 1 GiB, `KODE_HTTP_WORKERS=1`, `/hello`:**

| generator | connections | requests/s | p50 | p99 | p99.9 | CPU µs/request |
|---|---:|---:|---:|---:|---:|---:|
| oha | 100 | 60,374 | 1.53 ms | 4.16 ms | 20.25 ms | 16.6 |
| wrk | 100 | 91,417 | 0.96 ms | 7.18 ms | — | — |
| oha | 1000 | 43,304 | 21.67 ms | 41.18 ms | 53.39 ms | 23.1 |
| wrk | 1000 | 61,247 | 15.62 ms | 31.69 ms | — | — |

**Idle keep-alive connections (same server):**

| attempted | served | closed before a response | peak container memory |
|---:|---:|---:|---:|
| 100 | 100 | 0 | 18.4 MB |
| 1,000 | 1,000 | 0 | 22.0 MB |
| 10,000 | 1,024 | 8,976 | 20.2 MB |

**macOS, thread-per-connection fallback** (host machine, no CPU limit, so the
request rates are *not* comparable to the container above; the shape is):

| measurement | value |
|---|---|
| `oha` c=100 | 123,145 req/s, p50 0.61 ms, p99 6.10 ms |
| `oha` c=1000 | 121,151 req/s, p50 6.63 ms, p99 41.05 ms |
| 500 simultaneous idle connections | 395 served (rest closed before a response), **396 threads**, RSS 1.9 MB → 11.9 MB |
| 10,000 simultaneous connections | 1,024 served, 178 dial failures |
| threads after all clients close | back to 1 |

**Binary size.** The epoll loop costs nothing: it is in the runtime archive the
program already links. `mio` 1.x with `os-poll` and `net`, linked into a
`staticlib` that keeps every dependency object, adds **122,952 bytes** to the
archive; a real program would strip some of that at final link, so ~120 KB is an
upper bound.

## 4. Decision

1. **Linux keeps the hand-written epoll loop.** It is measured, it needs no
   crate, and it is the only model that holds 1,024 idle connections at ~20 MB
   of container memory. `mio` would add ~120 KB to every program that touches
   HTTP for an abstraction the runtime does not otherwise need, and the loop is
   435 lines that already exist and are tested.
2. **macOS and Windows keep the capped thread-per-connection fallback for
   v1.0.** A second readiness backend (`kqueue`, IOCP) is a new 400-line module
   per platform with its own tests, and the measured fallback already serves
   ~120k req/s on one host and 1,024 connections. Deferring it is a scope
   decision, not a claim that threads scale.
3. **The capacity contract is 1,024 concurrent connections per process**, for
   plaintext and TLS alike, because `MAX_ACTIVE_CONNECTIONS` is shared and the
   TLS path is thread-per-connection everywhere. This becomes documented
   behaviour rather than a constant nobody knows.
4. **Shedding must be honest.** Today a connection above the cap is accepted and
   then closed with no response: the 10,000-connection run served 1,024 and gave
   the other 8,976 a bare close. Stage 1 of the plan promised `503` with
   `Connection: close`. That is the next code change (with a test that opens
   `cap + 1` connections), and it is deliberately **not** part of this ADR.
5. **No `mio`.** Rejected on size and on the fact that option 2a already exists.
6. **Revisit only on a measured requirement**: more than 1,024 concurrent
   connections, or more than ~60k req/s per vCPU, or a workload where TLS
   connections must exceed the cap. The TLS event loop is the first candidate,
   because HTTPS is currently the weakest path.

`MAX_ACTIVE_CONNECTIONS` stays a constant in this change. Making it
configurable (`KODE_HTTP_MAX_CONNECTIONS`) needs its own test that the cap is
enforced and that exceeding it answers 503, not a silent close.

## 5. Alternatives considered

- **Blocking thread-per-connection everywhere.** Simplest code, and fine on
  macOS at 1,024 connections, but on the 1 vCPU container it would put a
  blocked thread on every idle connection: the measured fallback used ~25 KB
  RSS per idle connection on macOS, and thread stacks reserve far more virtual
  address space on Windows (1 MB each by default), so 10,000 connections is not
  reachable. It is also the model that cannot be measured above 1,024 without
  changing the cap.
- **`mio` with a worker pool (option 2b).** Cross-platform, would give Windows
  IOCP for free. Rejected for v1.0: ~120 KB per binary, one more crate in the
  runtime, and it duplicates a loop that is already working on the platform
  where services run.
- **`kqueue` + IOCP written by hand.** Best long-term shape on macOS/Windows,
  but two new modules with platform-specific unsafe FFI, and no measured
  workload currently justifies it. This is what decision 6 reopens.
- **Event loop for TLS.** Would raise HTTPS capacity, but makes the parser
  transport-agnostic (ADR 0034 §4 explains why it was not done), so it is a
  larger change than the plaintext loop it would replace.

## 6. Consequences

- The ROADMAP go/no-go checkpoint ("Kode must clearly beat Python and Node on
  requests per second and RAM, and be close to Go") can now be evaluated with
  numbers instead of the in-repo Go client, whose source
  (`benchmarks/http/load.go`) is missing from the tree and cannot be rebuilt.
- A service can state its capacity: ~60k req/s for a small plaintext handler on
  one vCPU at c=100, degrading to ~43k at c=1,000 as queueing, not throughput,
  becomes the limit; 1,024 connections; ~10–20 MB of container memory.
- Above the cap, clients currently see a closed socket. Until decision 4 lands,
  that is documented behaviour and a load balancer health check will read it as
  a failed request, not as backpressure.
- Windows inherits both the thread-per-connection model and the `1 MB` default
  thread stack, so its practical ceiling is lower than macOS at the same cap.
  This is the strongest argument for IOCP, and it is why decision 6 exists.
- No new dependency, no new unsafe code, no change to the binary model.

## 7. Open questions for the maintainer

1. Approve decision 4 (`503` + `Connection: close` above the cap) as the next
   code change, with the `cap + 1` test?
2. Is more than 1,024 concurrent connections a v1.0 requirement? If yes, that is
   a `kqueue`/IOCP branch before release, not a constant change.
3. Should `MAX_ACTIVE_CONNECTIONS` become configurable before v1.0, or is a fixed
   documented cap acceptable?

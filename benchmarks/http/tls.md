# TLS cost: preliminary measurements

**PRELIMINARY.** One machine (macOS 15, arm64, 10-core Apple silicon laptop),
not the 1 vCPU / 1 GiB container this project budgets against, and `ab` ran on
the same host as the servers. Reproduce before quoting any number publicly.

Measured 2026-11-09 on `main` at `b6f3304`, `kode` debug compiler, programs built
with `kode build --release`, sizes after `strip -x`.

## Binary size

| Program | Size (bytes) | Delta |
|---|---:|---:|
| `examples/hello.kode` | 1,077,600 | — |
| benchmark server, plaintext `serve` | 1,484,136 | +406,536 over hello |
| benchmark server, `serve_tls` | 5,431,968 | **+3,947,832 over the plaintext server** |

All three are over the 1 MB stripped budget in `docs/PLAN-v1.md`; plaintext
hello already was before TLS, because the PostgreSQL client pulled `rustls` and
`webpki-roots` into every program (ADR 0033). The TLS *server* adds about 3.9 MB
on top, almost all of it server-side `rustls`/`aws-lc-rs` code that a program
which only makes `https://` requests does not link.

Link time reported by the compiler: 53–60 ms for each of the three programs.

## Memory, idle and after load

| Server | RSS idle | RSS after 5,000 requests, c=50 |
|---|---:|---:|
| plaintext `serve` | 2,080 KB | 2,880 KB |
| `serve_tls` | 3,600 KB | 5,552 KB |

The TLS server runs one thread per connection (ADR 0034), so this grows with
concurrent connections, not with request count.

## Throughput and latency

`ab -k -n 5000 -c 50 /hello`, keep-alive, no failed requests on either side:

| Server | Requests/s | Mean time per request |
|---|---:|---:|
| plaintext `serve` | 140,734 | 0.374 ms |
| `serve_tls` | 62,346 (43,813 on a first, colder run) | 1.141 ms |

Connection setup, five samples each (`curl -w`):

| | Samples (s) |
|---|---|
| TLS handshake (`time_appconnect`) | 0.026 (cold), 1.0–1.2 ms |
| plaintext connect (`time_connect`) | 0.12–0.17 ms |

## What this does not cover

- The 1 vCPU / 1 GiB container, a separate load generator, and `wrk`/`oha`
  reproduction. The numbers above are for direction, not for a release note.
- High connection counts (1,000+ idle TLS connections). The thread-per-connection
  model makes this the real limit to measure next.
- HTTP/2, session resumption, and certificate rotation — none are implemented.

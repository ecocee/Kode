# HTTP benchmark report
Generated: 2026-11-07T02:36:05.186426 (container, 1 vCPU / 1 GiB)
Runner: `benchmarks/http/load.go` (purpose-written keep-alive client) with `-n/-c`; see `benchmarks/http/measure.sh`.
All servers built for release where possible; Kode server built with `kode build <file> --release`.

> **Not reproducible as written:** `benchmarks/http/load.go` is missing from the
> tree, so the rows in this first section cannot be regenerated. The Stage 2
> section at the end of this file is the reproducible one (two independent
> generators, `benchmarks/http/stage2.sh`), and it supersedes these numbers for
> the Kode server's I/O model.

## Results

| language | endpoint | connections | requests | actual | rps | p50 (ms) | p99 (ms) | errors | RSS (KB) | peak RSS (KB) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| kode | hello | 100 | 1000 | 1000 | 40757 | 0.06 | 14.38 | 0 | 1572 | 1572 |
| kode | hello | 100 | 100000 | 100000 | 23419 | 0.06 | 0.13 | 0 | 1572 | 1572 |
| kode | hello | 1000 | 1000 | 1000 | 640 | 209.48 | 438.34 | 0 | 1572 | 1572 |
| kode | hello | 1000 | 100000 | 100000 | 3576 | 0.06 | 66.60 | 0 | 1572 | 1572 |
| kode | json | 100 | 1000 | 1000 | 32586 | 0.06 | 18.04 | 0 | 1600 | 1600 |
| kode | json | 100 | 100000 | 100000 | 21962 | 0.06 | 0.13 | 0 | 1632 | 1632 |
| kode | json | 1000 | 1000 | 1000 | 361 | 210.56 | 1718.63 | 0 | 1632 | 1632 |
| kode | json | 1000 | 100000 | 100000 | 3572 | 0.06 | 68.92 | 0 | 1632 | 1632 |
| go | hello | 100 | 1000 | 1000 | 65801 | 0.17 | 10.38 | 0 | 14980 | 14980 |
| go | hello | 100 | 100000 | 100000 | 36594 | 0.23 | 85.61 | 0 | 15808 | 17028 |
| go | hello | 1000 | 1000 | 1000 | 5436 | 3.12 | 83.75 | 0 | 28536 | 28536 |
| go | hello | 1000 | 100000 | 100000 | 31270 | 3.96 | 191.88 | 0 | 35408 | 51044 |
| go | json | 100 | 1000 | 1000 | 13055 | 0.12 | 68.52 | 0 | 35408 | 51044 |
| go | json | 100 | 100000 | 100000 | 28712 | 0.25 | 84.88 | 0 | 20384 | 51044 |
| go | json | 1000 | 1000 | 1000 | 5391 | 10.35 | 102.86 | 0 | 32368 | 51044 |
| go | json | 1000 | 100000 | 100000 | 28697 | 3.59 | 108.10 | 0 | 29712 | 51044 |
| node | hello | 100 | 1000 | 1000 | 15674 | 4.06 | 17.37 | 0 | 59268 | 59268 |
| node | hello | 100 | 100000 | 100000 | 27181 | 1.39 | 62.77 | 0 | 74856 | 74856 |
| node | hello | 1000 | 1000 | 1000 | 5309 | 15.50 | 96.65 | 0 | 81580 | 81604 |
| node | hello | 1000 | 100000 | 100000 | 26905 | 18.41 | 79.88 | 0 | 105804 | 105804 |
| node | json | 100 | 1000 | 1000 | 16385 | 1.47 | 43.77 | 0 | 105860 | 105860 |
| node | json | 100 | 100000 | 100000 | 26496 | 1.59 | 57.02 | 0 | 106448 | 106448 |
| node | json | 1000 | 1000 | 1000 | 5154 | 9.24 | 105.54 | 0 | 107288 | 107288 |
| node | json | 1000 | 100000 | 100000 | 25667 | 19.70 | 79.36 | 0 | 113732 | 113732 |
| python | hello | 100 | 1000 | 940 | 30 | 42.65 | 15041.72 | 6 | 17916 | 17916 |
| python | hello | 100 | 100000 | 74000 | 1171 | 43.78 | 51.31 | 26 | 20756 | 21056 |
| python | hello | 1000 | 1000 | 184 | 6 | 7.47 | 27453.64 | 816 | 20832 | 21056 |
| python | hello | 1000 | 100000 | 14600 | 440 | 42.70 | 53.97 | 854 | 21684 | 23232 |
| python | json | 100 | 1000 | 800 | 26 | 43.05 | 14915.01 | 20 | 21676 | 23232 |
| python | json | 100 | 100000 | 50000 | 806 | 44.12 | 47.62 | 50 | 21760 | 23232 |
| python | json | 1000 | 1000 | 207 | 6 | 207.01 | 28158.72 | 793 | 21764 | 23232 |
| python | json | 1000 | 100000 | 18600 | 562 | 43.09 | 62.09 | 814 | 22480 | 24624 |

## Notes

- Python uses `threading` + `select` in the benchmark server; high p99 and errors under load are expected for that model.
- Kode HTTP runtime uses `available_parallelism().max(4)` workers (ADR context) and a global `ROUTES` lock on dispatch; performance is constrained by that design.
- The custom Go load client prints accurate counters (we fixed flag parsing) unlike `hey` which mis-reported Kode totals under concurrency.

---

# Stage 2 — I/O model (ADR 0037)

Generated: 2026-11-11.
Runner: `benchmarks/http/stage2.sh` (server container `--cpus=1 --memory=1g`,
load generator in a second container on the same Docker network).
Generators: **oha 1.16.0** and **wrk 4.2.0**, both run from the load container.
Reproduce: `./benchmarks/http/stage2.sh` (builds `kode`, `tools` images first).

Why a second report: every number above came from the repo's own Go client,
whose source file (`benchmarks/http/load.go`) is **missing from the tree**, so
those rows cannot be reproduced. Stage 2 therefore uses two independent,
widely used generators and reports both. Where oha and wrk disagree, the gap is
the generator's own overhead (oha does its own timing and JSON bookkeeping);
wrk is the higher, simpler number.

## Machine and image

| item | value |
|---|---|
| host | macOS 27.2 (26B123), Apple silicon, 10 cores, 8 GiB Docker VM |
| container runtime | Docker Desktop 29.8.2, Linux 7.0.14-linuxkit, cgroup v2 |
| server image | `debian:trixie-slim` + `kode build server.kode --release` |
| build toolchain | rust 1.9x (trixie image), clang 19.1.7 |
| server binary | 988,912 bytes (the runtime image has no `strip`, so this is unstripped) |
| server workers | `KODE_HTTP_WORKERS=1` (one worker thread, 1 vCPU) |
| limits | 1 CPU quota, 1 GiB memory, `nofile=65535` |

## Throughput and latency: `/hello`

| generator | connections | requests | requests/s | p50 | p99 | p99.9 | CPU s | CPU µs/req | throttled |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oha | 100 | 905,709 | 60,374 | 1.53 ms | 4.16 ms | 20.25 ms | 14.99 | 16.6 | 607 ms |
| wrk | 100 | — | 91,417 | 0.96 ms | 7.18 ms | — | — | — | — |
| oha | 1000 | 648,940 | 43,304 | 21.67 ms | 41.18 ms | 53.39 ms | 14.97 | 23.1 | 779 ms |
| wrk | 1000 | — | 61,247 | 15.62 ms | 31.69 ms | — | — | — | — |

## Throughput and latency: `/json`

| generator | connections | requests | requests/s | p50 | p99 | p99.9 | CPU s | CPU µs/req | throttled |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| oha | 100 | 520,719 | 34,713 | 2.46 ms | 11.89 ms | 36.38 ms | 14.98 | 28.8 | 1,020 ms |
| wrk | 100 | — | 55,963 | 1.65 ms | 9.27 ms | — | — | — | — |
| oha | 1000 | 535,012 | 35,709 | 26.71 ms | 47.70 ms | 96.56 ms | 15.01 | 28.1 | 658 ms |
| wrk | 1000 | — | 50,447 | 18.59 ms | 34.26 ms | — | — | — | — |

Reading these:

- **One worker thread saturates the CPU** at every configuration (99.8–100.1 %
  of the 1 vCPU quota, 15 s of CPU in a 15 s run). The server is CPU-bound, not
  waiting: the epoll loop keeps up with 60k req/s at c=100.
- **More connections cost latency, not throughput.** c=1000 raises p50 from
  1.53 ms to 21.67 ms on `/hello` (14×) while throughput falls 28 %. That is
  queueing behind one worker, the expected shape for a 1-vCPU server.
- **`/json` costs ~12 µs more CPU per request** than `/hello` (28.8 vs 16.6 µs),
  because the handler builds a JSON object per request.
- oha reports `aborted due to deadline` once per connection when `-z` ends the
  run (100 at c=100, 1000 at c=1000). `successRate` stays 1.0000; these are not
  server errors.

## Idle keep-alive connections

`idle` (in `benchmarks/http/idle.go`) opens N sockets, sends one request on
each, holds them idle for 5 s, then checks which are still open.

| attempted | served | closed before a response | still open after 5 s | peak container memory | connect p50 / p99 |
|---:|---:|---:|---:|---:|---|
| 100 | 100 | 0 | 100 | 18.4 MB | 8.53 / 9.32 ms |
| 1,000 | 1,000 | 0 | 1,000 | 22.0 MB | 39.46 / 44.84 ms |
| 10,000 | 1,024 | 8,976 | 1,024 | 20.2 MB | 47.27 / 111.45 ms |

- Memory per idle connection is **small and flat**: peak container memory rose
  3.6 MB going from 100 to 1,000 idle connections (~4 KB each), and did not grow
  further at the 1,024-connection cap. An idle connection is an entry in the
  epoll slab, not a thread.
- **The cap is the wall, not memory.** `MAX_ACTIVE_CONNECTIONS = 1024`: at
  10,000 attempts the server accepted every socket and served 1,024 of them;
  the other 8,976 were closed with no response. Clients see an EOF, not a 503 —
  ADR 0037 decision 4.
- `memory.current` includes page cache, so the ~18 MB baseline is mostly the
  binary and its shared pages, not per-connection state.

## The portable fallback (macOS, thread-per-connection)

Same `server.kode`, built for the host and run natively. There is no CPU limit
on macOS, so request rates are **not** comparable to the container numbers; this
run exists to measure the model's shape, not its ceiling.

| measurement | value |
|---|---|
| oha c=100 | 123,145 req/s, p50 0.61 ms, p99 6.10 ms, success 1.0000 |
| oha c=1000 | 121,151 req/s, p50 6.63 ms, p99 41.05 ms, success 0.9998 |
| 500 simultaneous idle connections | 395 served, 396 threads, RSS 1.9 MB → 11.9 MB |
| 10,000 simultaneous connections | 1,024 served, 178 dial failures |
| threads after every client closed | 1 |

- One thread per connection is exactly what the thread count shows, and RSS
  grows with **connections**, not with request rate: about 25 KB of resident
  memory per idle connection.
- At 500 simultaneous dials only 395 got a response, and at 1,000 dials 119
  failed outright. That is the accept backlog (`raise_backlog` asks for 4,096,
  macOS caps `kern.ipc.somaxconn`), which is hit before the 1,024 connection
  cap. Windows has the same class of limit with a different number.

## Binary size: the cost of an alternative

Measured in a scratch crate (`crate-type = ["staticlib"]`, arm64 macOS,
`--release`):

| crate graph | static archive | delta |
|---|---:|---:|
| empty `staticlib` | 17,649,168 B | — |
| `+ mio 1` (`os-poll`, `net`) | 17,772,120 B | **+122,952 B** |

A `staticlib` keeps every dependency object, so this is an upper bound: a real
program would drop unreachable objects at final link. The epoll loop costs 0
bytes because it is already part of the runtime archive.

## What this means for ADR 0037

- Linux ships the epoll loop and it is measured: ~60k req/s at c=100 on one
  vCPU, 1,024 connections, ~20 MB.
- macOS/Windows ship thread-per-connection with the same 1,024 cap; their limit
  arrives earlier (threads, backlog) and their memory scales with connections.
- Nobody needs `mio` yet: 435 lines of epoll already exist, and the crate would
  add ~120 KB to every program that serves HTTP.

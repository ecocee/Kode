# Plan: fix/http-concurrency

Branch: `fix/http-concurrency` (open after approval). No code changes until this plan is approved.

## Goals
Stop the outage (starvation, unbounded blocking, no limits), make concurrency model explicit (ADR + measurements), remove hot-path contention. No public performance/RAM claims until fixed and independently reproduced (wrk/oha). TLS decision memo must account for chosen I/O model.

## Stage 1 (small, safe, ship alone)
- [ ] **Read timeouts**: header-read timeout, body-read timeout, idle keep-alive timeout (env/configurable, sensible defaults). Close connections on timeout.
- [ ] **Size limits**: max request-line, header count/total bytes, body bytes. Return proper 400/431/413 with close or per-spec behavior.
- [ ] **Connection cap**: global cap (configurable). When exceeded, return 503 with appropriate headers/close (no silent queueing forever).
- [ ] **Regression tests** (deterministic worker count via `KODE_HTTP_WORKERS`): N workers, open N idle keep-alive conns → fresh request succeeds within bounded time (idle timeout or cap must release). Slowloris-style (headers one byte at a time). Timeouts/limits exercised.
- [ ] **Docs**: update behavior notes.

## Stage 2 (ADR + measurements, no commitment)
Compare on 1 vCPU / 1 GB server container, load gen in separate container:

| Option | I/O | Handler execution | Notes |
|---|---|---|---|
| 1 | Blocking thread-per-connection (small stacks), bounded by cap | Blocking OK if pooled | Simple; RSS grows with conns; need to cap. |
| 2a | Event loop (epoll/Linux, kqueue/macOS) + worker pool for handlers | Blocking handler won't stall I/O | Scalable; need to handle blocking handlers correctly. |
| 2b | 2a with vetted crate `mio` vs direct libc | same | Crate needs explicit approval + size impact. |

Measure for each:
- RSS and peak RSS at 100, 1000, 10000 live idle keep-alive connections
- RPS, p50/p99/p99.9 at c=100, c=1000 (wrk or oha)
- CPU time per request (user+sys)
- Binary size delta
- LOC, Windows story (epoll/kqueue no Windows equiv; propose solution)

Recommend one with rationale. If adding `mio`, present as question with size impact. ADR required.

## Stage 3 (contention)
- [ ] Replace global ROUTES lock on hot path with immutable snapshot (built at startup) or `RwLock`. Measure before/after.

## Cross-cutting requirements
- [ ] Handlers keep existing refcount/`THREADED` rules; leak-checked test with concurrent requests.
- [ ] Keep-alive, `Connection: close`, `Content-Length` compatible with current tests.
- [ ] Explain why `hey` misreported (check `Content-Length`/connection-close with `curl -v` and raw sockets). If server protocol bug → test + fix.
- [ ] Re-run full benchmark after each stage; keep before/after in `benchmarks/http/report.md`. PRELIMINARY banner until independently reproduced with `wrk` or `oha`.
- [ ] No public perf/RAM claims. 1.6MB RSS figure not valid (only handful live).
- [ ] TLS memo must state interaction with each I/O model (rustls works with blocking/non-blocking; needs per model).

## Testing strategy
- Deterministic: `KODE_HTTP_WORKERS=N` for all concurrency tests.
- Starvation regression: N workers + N idle keep-alive → new request succeeds within timeout.
- Slowloris: partial headers one byte at a time → timeout/400, doesn't block others.
- Overload: over cap → 503.
- Leak check under concurrency.

## Acceptance
All stages done, benchmarks reproduced with wrk/oha, ADR for model choice, before/after tables, no regressions in existing tests.

# PostgreSQL database benchmark

A real PostgreSQL benchmark: one shared server, one schema, one seed dataset and
one set of SQL statements, executed by six native drivers and by Kode.

Kode cannot take part. It has no PostgreSQL client, no C FFI, no raw socket API
and no TLS, so every Kode case is reported **UNSUPPORTED** instead of being
replaced by a substitute implementation. `results/reports/kode-capability.md`
holds the evidence, produced by `kode/probe.sh` from the compiler itself.

The workload is not simulated. Every measurement comes from a PostgreSQL 17
container, and every case is verified before its numbers are kept.

## Quick start

```bash
# start PostgreSQL, apply the schema, seed 10,000 rows, exit
bash benchmarks/database/run.sh --reset

# one workload, one language
bash benchmarks/database/run.sh --workload insert --language python --no-build

# smoke test: 1,000 rows, 1 repetition, everything
bash benchmarks/database/run.sh --quick --build

# the published run
bash benchmarks/database/run.sh --all --rows 10000 --repetitions 5 --build
```

`run.sh` needs `docker` (with the daemon running) and `python3`. Everything else
runs inside the client images.

## Options

| Flag | Meaning |
|---|---|
| `--all` | every workload and every language (same as no `--workload`/`--language`) |
| `--workload insert,select` | comma-separated workloads: `insert`, `select`, `update`, `delete`, `join`, `transactions`, `concurrency`, `connections`, `application` |
| `--language rust,go` | comma-separated languages: `rust`, `go`, `node`, `java`, `python`, `csharp`, `kode` |
| `--rows 100000` | dataset size for both the read set (`bench_events`) and the write set (`bench_writes`) |
| `--repetitions 5` | recorded repetitions per (language, workload); one warm-up is discarded first |
| `--workers 10,100` | which concurrency cases to run |
| `--ops 200` | operations per concurrent worker |
| `--timeout 600` | seconds before one client invocation is killed and reported TIMEOUT |
| `--build` | build client images before running |
| `--build-only` | build client images and exit |
| `--no-build` | use the images that already exist |
| `--no-warmup` | skip the discarded warm-up invocation |
| `--quick` | 1,000 rows, 50 ops, 1 repetition |
| `--reset` | start PostgreSQL, apply schema and seed, then exit |
| `--down` | stop the PostgreSQL container at the end |

The runner exits non-zero if any selected case ends FAIL or TIMEOUT.

## Layout

```
benchmarks/database/
  docker-compose.yml        PostgreSQL 17 + one service per client
  Dockerfile                one target per language, pinned driver versions
  schema/schema.sql         tables, indexes, reset functions, grants
  schema/seed.sql           deterministic dataset
  schema/init-roles.sql     non-superuser role `bench`
  workloads/cases.json      the canonical SQL, parameters and iterations
  workloads/invariants.json driver-independent end-of-workload checks
  common/config.py          paths, language list, driver versions
  common/monitoring.py      psql access, resets, pg_stat_statements, cgroup, EXPLAIN
  common/validation.py      parameter formulas, expectations, comparison
  common/report.py          results.json / results.csv / summary.md
  harness/runner.py         orchestration, warm-up, rotations, invalidation
  rust/ go/ node/ java/ python/ csharp/   one client per language
  kode/                     capability evidence and the psql feasibility probe
  results/raw/              raw per-repetition JSON, never edited
  results/reports/          summary.md, kode-capability.md
```

## Fairness rules enforced by the harness

* **One server.** Every language connects to the same PostgreSQL 17 container
  over the same Docker network with the same credentials, `max_connections` and
  `default_transaction_isolation` (read committed).
* **One schema, one seed.** `schema/schema.sql` and `schema/seed.sql` are applied
  by the harness, not by clients. Seeding is deterministic: no random data, no
  clock.
* **One statement list.** Clients read `workloads/cases.json` at run time. They
  do not contain their own SQL, their own iteration counts or their own checks,
  so the workload is identical by construction.
* **One client at a time.** Languages never run concurrently, so they never
  compete for the server.
* **Fresh state per invocation.** Write workloads re-seed before every
  invocation; read-only workloads re-seed at least once per round.
* **Release builds.** Rust `cargo build --release`, Go `-trimpath -ldflags='-s -w'`,
  .NET `dotnet publish -c Release`, Java shaded jar with a JRE, Node plain ESM,
  Python with the binary wheel of psycopg 3.
* **Idiomatic drivers.** Prepared statements in loops, the driver's own pool for
  pooled cases, positional binding native to each driver. No hand-written
  protocol client and no deliberately slow implementation.
* **Warm-up then repetitions.** One discarded invocation per (language, workload),
  then `--repetitions` recorded invocations; the report shows the median.
* **Verification gates the numbers.** A case whose check fails is excluded from
  the medians and the invocation is reported FAIL.

## What is measured, and where

| Metric | Source | Owner |
|---|---|---|
| `ops/s`, `wall_ms` | client process, per case | client driver |
| `p50_ms`, `p95_ms`, `p99_ms` | per-operation timings inside the client | client driver + runtime |
| `rows_affected`, `rows_fetched`, `transactions` | driver return values | client driver |
| `errors`, `error_sample` | driver exceptions | client driver |
| `cpu_ms` | process CPU time (user+system) during the case | client |
| `peak_rss_kb` | `VmHWM` of the client container | client |
| `db_total_ms` | `pg_stat_statements.total_exec_time` delta around the invocation | PostgreSQL |
| `db_exec_ms` | `EXPLAIN (ANALYZE, TIMING OFF, FORMAT JSON)` of one representative query, rolled back | PostgreSQL |
| `db_conns_max`, `db_mem_mb`, `db_cpu_s` | sampled `pg_stat_activity`, cgroup of the server container | PostgreSQL |
| `deadlocks` | `pg_stat_database.deadlocks` delta | PostgreSQL |

`db exec` is PostgreSQL's own execution time. `client ms/op` is `p50 - db exec`:
driver, protocol, serialization and runtime overhead. For concurrent cases the
server time is summed across parallel sessions, so that residual can go negative;
it is a diagnostic, not a language ranking.

## Correctness

Every case carries `checks` in `cases.json`, executed by the client immediately
after the case runs, while the state it asserts is current. Example checks: the
inserted row count equals the iteration count, an update left no negative stock,
a rollback left zero rows, a join returned the expected number of orders.

`workloads/invariants.json` adds independent checks that run through `psql`
after each workload, so a driver that silently returned wrong data cannot make
the workload pass.

`expect_errors` marks the two cases where an error is the point (a duplicate key
inside a transaction, a terminated backend). Their failures are counted in
`errors` and shown in the report; they do not invalidate the case.

## Limits, stated plainly

* **Kode has no database numbers.** There is no driver, no FFI, no socket API and
  no TLS in Kode 0.3. `kode/probes/psql_probe.kode` measures the only path that
  exists (spawning `psql` per query) and is labelled a feasibility probe, not a
  benchmark result. Adding a proxy or an in-memory substitute would have compared
  a different architecture, so it was not done.
* **Docker Desktop on macOS.** PGDATA lives on a tmpfs inside the container so
  commits do not pay the userspace file-sharing latency of the host filesystem.
  `fsync` and `synchronous_commit` stay on. Absolute throughput is therefore
  lower than bare-metal Linux, but every language pays the same cost.
* **One host, one server, one dataset size per run.** Latency figures include the
  Docker network hop, which is identical for all clients.
* **`db exec` is one representative query,** not a full workload trace; it is an
  attribution aid, not a per-operation measurement.
* **No server tuning sweep.** PostgreSQL runs with its stock image settings except
  `shared_buffers` and `max_connections`, which are recorded in the report.
* **Timings are wall clock.** The host is shared with whatever else the machine is
  doing; the report records repetitions and medians so a single noisy run cannot
  decide a comparison.

## Kode evidence

```bash
bash benchmarks/database/kode/probe.sh     # writes results/raw/kode-probe.txt
```

The probe compiles three small programs that ask for a database API, a raw
socket and an `extern fn` declaration, records the compiler's answer, and greps
the front end for any database surface. `results/reports/kode-capability.md` is
generated from that log plus `kode/unsupported.json`.

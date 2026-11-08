# Kode and PostgreSQL

**Kode has a PostgreSQL client** (ADR 0033). This directory contains the benchmark
client `client.kode` that runs the shared database workloads.

## What is implemented

The Kode runtime (`crates/kode-rt/src/pg*.rs`) includes a PostgreSQL wire protocol
client with:

- `pg.connect(url)` / `pg.close(conn)` - direct connections
- `pg.exec(conn, sql)` - simple query protocol
- `pg.query(conn, sql, params)` - extended query protocol with parameters
- `pg.pool(url, min, max, idle_ms)` / `pg.pool_close(pool)` - connection pooling
- `pg.pool_connect(pool)` / `pg.pool_release(conn)` - manual pool acquire/release
- `pg.pool_exec(pool, sql)` / `pg.pool_query(pool, sql, params)` - pool convenience methods

Authentication: cleartext and SCRAM-SHA-256. No TLS (use `sslmode=disable`).

## Running the benchmark

```bash
docker compose -f benchmarks/database/docker-compose.yml up -d postgres
./target/debug/kode run benchmarks/database/kode/client.kode -- \
  --workload insert --cases benchmarks/database/workloads/cases.json \
  --host 127.0.0.1 --port 55432 --rows 10000 --ops 200
```

## Configuration

Connection string: `postgres://user:password@host:port/database?sslmode=disable`

Environment variables: `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`

## Limitations

- Plain TCP only (no TLS)
- No md5 authentication (SCRAM-SHA-256 and cleartext only)
- No COPY, binary format, notifications, or arrays as parameters

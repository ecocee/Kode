# Workload definitions

`cases.json` is the single source of truth for what every language runs. Each
client reads it from `/bench/workloads/cases.json`, filters by `--workload` and
runs the cases in file order. Nothing about the SQL, the iteration counts or the
checks lives in a client, so all six drivers execute an identical workload.

## Case fields

| Field | Meaning |
|---|---|
| `id` | stable identifier, also the CSV/JSON case column |
| `workload` | one of `insert select update delete join transactions concurrency connections application` |
| `shape` | how the client drives the case (see below) |
| `sql` | the statement; `$n` placeholders are bound by number |
| `params` | parameter generator names, in `$n` order |
| `iterations` | operation count, or the string `rows` for the dataset size |
| `setup` | SQL run before the case, through the same connection |
| `checks` | queries with expected values, run by the client after the case |
| `expect_errors` | the case is expected to raise errors (they are counted, not fatal) |
| `scenario`, `mode`, `workers`, `per_txn` | shape-specific settings |
| `note` | one line the report quotes when explaining the case |

## Shapes

| Shape | What the client does |
|---|---|
| `stmt_loop` | prepare once, bind new values, execute once per iteration |
| `query_loop` | prepare once, execute, read every row and every column |
| `chunk_insert` | multi-row `INSERT` with `chunk` (500) rows per statement, autocommit |
| `chunk_insert_txn` | the same, inside one transaction |
| `batch_update` | `UPDATE ... FROM (VALUES ...)` with `chunk` pairs per statement |
| `batch_delete` | `DELETE ... WHERE id BETWEEN $1 AND $2` with `chunk` ids per statement |
| `txn_loop` | one transaction wrapping every iteration |
| `txn_case` | per-iteration `BEGIN`/`COMMIT`, `ROLLBACK`, multi-statement, constraint failure and recovery scenarios |
| `conc` | one connection per worker, all workers started before the timer, `ops` operations each |
| `conn` | connection establishment, reuse, pool check-out, pool exhaustion and backend termination |
| `app_order_txn` | order insert plus stock decrement in one transaction, rolled back when stock is short |

## Parameter formulas

The same formulas exist in every client (`common/validation.py` is the reference)
so the rows a driver asks for are identical. `i` is the operation index, `rows`
the dataset size.

| Name | Value |
|---|---|
| `id` | `i + 1` |
| `k` | `(i * 37) % rows` |
| `k_hi` | `((i * 37) % rows) + 25` |
| `pick_id` | `((i * 7) % rows) + 1` |
| `pick_product` | `((i * 7) % 1000) + 1` |
| `offset` | `(i * 100) % 1000` |
| `qty` | `(i % 5) + 1` |
| `order_id` | `((i * 997) % 100000) + 1` |
| `app_product_id`, `app_customer_id` | `100000 + i` |
| `app_order_id` | `1000000 + i` |
| `payload` / `payload2` | `payload-<i>` / `updated-<i>` |
| `tag` | `t<i % 10>` |
| `status` | `new, paid, shipped, cancelled` by `i % 4` |
| `price` | `19.99` (bound as text, cast in SQL) |
| `sku`, `name`, `email` | `SKU-NEW-<i>`, `name-<i>`, `user<i>@example.com` |
| `city` | `Bengaluru, Berlin, Lisbon, Osaka` by `i % 4` |
| `category` | `books, games, tools, music` by `i % 4` |
| `name_like` | `name-%` |

Numeric columns are cast to `text` in `SELECT`s and bound as `$n::text::numeric`
in writes, so all six drivers decode the same types. Timestamps are not part of
any case for the same reason.

## Expectations

`checks[].expect` may be a literal (`"0"`, `"1200"`) or an expression using
`rows`, `iters`, `per_txn`, `ops`, `workers`, `chunk`:

* `rows` — the dataset size for this run;
* `iters` — the case's iteration count;
* `workers*ops` — rows written by a concurrent write case;
* `workers*60` — writes in a mixed case (30% of 200 operations per worker).

An expectation that is a value rather than a count is written as `lit:<text>`
(for example `lit:updated-0`), which the harness compares verbatim instead of
evaluating.

`workloads/invariants.json` holds the additional per-workload checks that run
through `psql` after the invocation, independently of any driver.

## Adding a case

1. Add the case to `cases.json` with an `id`, `shape`, `sql`, `params`, `setup`
   and `checks`. Keep the SQL valid on PostgreSQL 17 and free of driver-specific
   syntax.
2. Add any new parameter generator to **every** client and to
   `common/validation.py`, or reuse an existing one.
3. Add a driver-independent check to `invariants.json` if the case changes data.
4. Run it for one language first:
   `bash benchmarks/database/run.sh --workload <workload> --language python --no-build`.
5. Then run every language before publishing numbers from it.

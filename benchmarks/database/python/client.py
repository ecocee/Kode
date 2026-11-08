#!/usr/bin/env python3
"""PostgreSQL benchmark client using psycopg 3 (the established Python driver).

Reads workloads/cases.json, runs every case of one workload, and prints one JSON
object per case on stdout. The harness resets the database between invocations,
so a client never has to care about state left by another language.

Metric split:
  * wall_ms + p50/p95/p99 come from this process (client-side driver cost).
  * server-side execution time is measured by the harness from pg_stat_statements,
    never guessed here.
  * cpu_ms is process user+system CPU, peak_rss_kb is VmHWM, so both include the
    driver and the interpreter, which is what a language comparison needs.
"""

import argparse
import json
import math
import os
import re
import sys
import threading
import time

import psycopg
from psycopg_pool import ConnectionPool

LANGUAGE = "python"
STATUSES = ["new", "paid", "shipped", "cancelled"]
CATEGORIES = ["books", "games", "tools", "music"]
CITIES = ["Bengaluru", "Berlin", "Lisbon", "Osaka"]


# ------------------------------------------------------------------ utilities
def percentile(sorted_values, p):
    if not sorted_values:
        return 0.0
    pos = (len(sorted_values) - 1) * p
    lo, hi = int(math.floor(pos)), int(math.ceil(pos))
    if lo == hi:
        return float(sorted_values[lo])
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (pos - lo)


def peak_rss_kb():
    try:
        with open("/proc/self/status") as f:
            for line in f:
                if line.startswith("VmHWM:"):
                    return int(line.split()[1])
    except OSError:
        pass
    return 0


def cpu_ms():
    t = time.process_time()
    return t * 1000.0


def summarise(latencies, wall_ms):
    lat = sorted(latencies)
    ops = len(lat)
    return {
        "ops": ops,
        "wall_ms": round(wall_ms, 3),
        "throughput": round(ops / (wall_ms / 1000.0), 2) if wall_ms > 0 else 0.0,
        "p50_ms": round(percentile(lat, 0.50), 3),
        "p95_ms": round(percentile(lat, 0.95), 3),
        "p99_ms": round(percentile(lat, 0.99), 3),
        "min_ms": round(lat[0], 3) if lat else 0.0,
        "max_ms": round(lat[-1], 3) if lat else 0.0,
    }


# ---------------------------------------------------------- parameter formulas
# These formulas are duplicated verbatim in every client and documented in
# workloads/README.md. If one changes, all of them change.
def gen(name, i, rows, worker):
    if name == "id":
        return i + 1
    if name == "k":
        return (i * 37) % rows
    if name == "k_hi":
        return ((i * 37) % rows) + 25
    if name == "pick_id":
        return ((i * 7) % rows) + 1
    if name == "pick_product":
        return ((i * 7) % 1000) + 1
    if name == "offset":
        return (i * 100) % 1000
    if name == "qty":
        return (i % 5) + 1
    if name == "order_id":
        return ((i * 997) % 100000) + 1
    if name == "app_product_id" or name == "app_customer_id":
        return 100000 + i
    if name == "app_order_id":
        return 1000000 + i
    if name == "worker_id":
        return worker * 1000000 + i + 1
    if name == "payload":
        return "payload-" + str(i)
    if name == "payload2":
        return "updated-" + str(i)
    if name == "tag":
        return "t" + str(i % 10)
    if name == "status":
        return STATUSES[i % 4]
    if name == "price":
        return "19.99"
    if name == "sku":
        return "SKU-NEW-" + str(i)
    if name == "name":
        return "name-" + str(i)
    if name == "email":
        return "user" + str(i) + "@example.com"
    if name == "city":
        return CITIES[i % 4]
    if name == "category":
        return CATEGORIES[i % 4]
    if name == "name_like":
        return "name-%"
    raise ValueError("unknown parameter generator: " + name)


def params_for(names, i, rows, worker=0):
    return [gen(n, i, rows, worker) for n in names]


PLACEHOLDER = re.compile(r"\$(\d+)")


def compile_sql(sql):
    """Rewrite $n to psycopg's %s and record the binding order.

    $n placeholders are matched by number, not by position, so a statement such
    as `SET payload = $2 WHERE id = $1` must bind its values in appearance order.
    """
    order = []

    def replace(match):
        order.append(int(match.group(1)) - 1)
        return "%s"

    return PLACEHOLDER.sub(replace, sql), order


def values_for(order, names, i, rows, worker=0):
    raw = params_for(names, i, rows, worker)
    return [raw[k] for k in order]


def connect(conninfo, autocommit=True):
    return psycopg.connect(conninfo, autocommit=autocommit, application_name="kode-bench-" + LANGUAGE)


def is_deadlock(exc):
    return getattr(exc, "sqlstate", None) == "40P01"


# ------------------------------------------------------------------ case shapes
def run_stmt_loop(conn, case, rows, iterations):
    sql, order = compile_sql(case["sql"])
    names = case.get("params", [])
    lat, affected, errors, deadlocks, sample = [], 0, 0, 0, ""
    with conn.cursor() as cur:
        start = time.perf_counter()
        for i in range(iterations):
            t0 = time.perf_counter()
            try:
                cur.execute(sql, values_for(order, names, i, rows), prepare=True)
                affected += cur.rowcount if cur.rowcount and cur.rowcount > 0 else 0
            except psycopg.Error as exc:
                errors += 1
                deadlocks += 1 if is_deadlock(exc) else 0
                sample = sample or str(exc).splitlines()[0][:200]
            lat.append((time.perf_counter() - t0) * 1000.0)
        wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_affected=affected, errors=errors,
                deadlocks=deadlocks, error_sample=sample)


def run_query_loop(conn, case, rows, iterations):
    sql, order = compile_sql(case["sql"])
    names = case.get("params", [])
    lat, fetched, errors, sample = [], 0, 0, ""
    with conn.cursor() as cur:
        start = time.perf_counter()
        for i in range(iterations):
            t0 = time.perf_counter()
            try:
                cur.execute(sql, values_for(order, names, i, rows), prepare=True)
                fetched += len(cur.fetchall())
            except psycopg.Error as exc:
                errors += 1
                sample = sample or str(exc).splitlines()[0][:200]
            lat.append((time.perf_counter() - t0) * 1000.0)
        wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_fetched=fetched, errors=errors,
                error_sample=sample)


def chunk_values(names, start, count, rows):
    """Flatten parameters for `count` rows starting at `start`."""
    flat = []
    for i in range(start, start + count):
        flat.extend(params_for(names, i, rows))
    return flat


def run_chunk_insert(conn, case, rows, iterations, chunk, in_txn):
    names = case.get("params", [])
    width = len(names)
    lat, affected, errors, sample = [], 0, 0, ""
    start = time.perf_counter()
    try:
        if in_txn:
            conn.autocommit = False
        chunks = [(s, min(chunk, iterations - s)) for s in range(0, iterations, chunk)]
        for s, n in chunks:
            placeholders = ",".join("(" + ",".join(["%s"] * width) + ")" for _ in range(n))
            sql = case["sql"] + placeholders
            t0 = time.perf_counter()
            with conn.cursor() as cur:
                cur.execute(sql, chunk_values(names, s, n, rows), prepare=False)
                affected += cur.rowcount if cur.rowcount > 0 else 0
            lat.append((time.perf_counter() - t0) * 1000.0)
        if in_txn:
            conn.commit()
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
        if in_txn:
            conn.rollback()
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_affected=affected, errors=errors,
                error_sample=sample, batches=len(range(0, iterations, chunk)))


def run_batch_update(conn, case, rows, iterations, chunk):
    names = case.get("params", [])
    lat, affected, errors, sample = [], 0, 0, ""
    start = time.perf_counter()
    try:
        for s in range(0, iterations, chunk):
            n = min(chunk, iterations - s)
            values = ",".join("(%s::bigint,%s::text)" for _ in range(n))
            sql = case["sql"] + values + ") AS v(id, payload) WHERE bench_writes.id = v.id"
            t0 = time.perf_counter()
            with conn.cursor() as cur:
                cur.execute(sql, chunk_values(names, s, n, rows), prepare=False)
                affected += cur.rowcount if cur.rowcount > 0 else 0
            lat.append((time.perf_counter() - t0) * 1000.0)
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_affected=affected, errors=errors,
                error_sample=sample, batches=len(range(0, iterations, chunk)))


def run_batch_delete(conn, case, rows, iterations, chunk):
    lat, affected, errors, sample = [], 0, 0, ""
    sql, order = compile_sql(case["sql"])
    start = time.perf_counter()
    try:
        for s in range(0, iterations, chunk):
            n = min(chunk, iterations - s)
            t0 = time.perf_counter()
            bounds = {0: s + 1, 1: s + n}  # order is 0-based: $1 start, $2 end
            with conn.cursor() as cur:
                cur.execute(sql, [bounds[k] for k in order], prepare=True)
                affected += cur.rowcount if cur.rowcount > 0 else 0
            lat.append((time.perf_counter() - t0) * 1000.0)
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_affected=affected, errors=errors,
                error_sample=sample, batches=len(range(0, iterations, chunk)))


def run_txn_loop(conn, case, rows, iterations):
    sql, order = compile_sql(case["sql"])
    names = case.get("params", [])
    lat, affected, errors, sample = [], 0, 0, ""
    conn.autocommit = False
    start = time.perf_counter()
    try:
        with conn.cursor() as cur:
            for i in range(iterations):
                t0 = time.perf_counter()
                cur.execute(sql, values_for(order, names, i, rows), prepare=True)
                affected += cur.rowcount if cur.rowcount and cur.rowcount > 0 else 0
                lat.append((time.perf_counter() - t0) * 1000.0)
        conn.commit()
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
        conn.rollback()
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), rows_affected=affected, errors=errors,
                error_sample=sample, transactions=1)


def row_params(i, rows):
    return (i + 1, (i * 37) % rows, "payload-" + str(i), "t" + str(i % 10))


def run_txn_case(conn, case, rows, iterations):
    insert = "INSERT INTO bench_writes (id, k, payload, tag) VALUES (%s,%s,%s,%s)"
    per_txn = case.get("per_txn", 1)
    scenario = case["scenario"]
    txns, errors, affected, lat, sample = 0, 0, 0, [], ""
    conn.autocommit = False
    start = time.perf_counter()
    try:
        with conn.cursor() as cur:
            for i in range(iterations):
                t0 = time.perf_counter()
                cur.execute("BEGIN")
                if scenario in ("commit", "rollback"):
                    cur.execute(insert, row_params(i, rows))
                elif scenario == "multi":
                    for j in range(per_txn):
                        cur.execute(insert, row_params(i * per_txn + j, rows))
                elif scenario in ("constraint", "recovery"):
                    cur.execute(insert, row_params(i, rows))
                    try:
                        cur.execute(insert, row_params(i, rows))
                    except psycopg.errors.UniqueViolation:
                        errors += 1
                        cur.execute("ROLLBACK")
                        if scenario == "recovery":
                            cur.execute("BEGIN")
                            cur.execute(insert, row_params(1_000_000 + i, rows))
                            cur.execute("COMMIT")
                            txns += 1
                            affected += 1
                        lat.append((time.perf_counter() - t0) * 1000.0)
                        continue
                if scenario == "rollback":
                    cur.execute("ROLLBACK")
                else:
                    cur.execute("COMMIT")
                txns += 1
                affected += cur.rowcount if cur.rowcount and cur.rowcount > 0 else 0
                lat.append((time.perf_counter() - t0) * 1000.0)
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
        conn.rollback()
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), transactions=txns, rows_affected=affected,
                errors=errors, error_sample=sample)


def run_app_order_txn(conn, case, rows, iterations):
    update = "UPDATE products SET stock = stock - %s WHERE id = %s AND stock >= %s"
    insert = ("INSERT INTO orders (id, customer_id, product_id, qty, unit_price, status) "
              "VALUES (%s,%s,%s,%s,%s::numeric,%s)")
    lat, errors, committed, rolled_back, sample = [], 0, 0, 0, ""
    conn.autocommit = False
    start = time.perf_counter()
    try:
        with conn.cursor() as cur:
            for i in range(iterations):
                product = ((i * 7) % 1000) + 1
                qty = 1
                t0 = time.perf_counter()
                cur.execute("BEGIN")
                cur.execute(update, (qty, product, qty))
                if cur.rowcount == 1:
                    cur.execute(insert, (2_000_000 + i, product, product, qty, "19.99", "new"))
                    cur.execute("COMMIT")
                    committed += 1
                else:
                    cur.execute("ROLLBACK")
                    rolled_back += 1
                lat.append((time.perf_counter() - t0) * 1000.0)
    except psycopg.Error as exc:
        errors += 1
        sample = str(exc).splitlines()[0][:200]
        conn.rollback()
    wall_ms = (time.perf_counter() - start) * 1000.0
    return dict(summarise(lat, wall_ms), transactions=committed + rolled_back,
                rows_affected=committed, rolled_back=rolled_back, errors=errors,
                error_sample=sample)


def run_conc(conninfo, case, rows, ops):
    mode = case["mode"]
    workers = case["workers"] or 10
    lat_all, errors_all, deadlocks_all = [], 0, 0

    t0 = time.perf_counter()
    pool = ConnectionPool(conninfo, min_size=workers, max_size=workers, timeout=60,
                          kwargs={"application_name": "kode-bench-" + LANGUAGE,
                                  "autocommit": True})
    pool.wait()
    setup_ms = (time.perf_counter() - t0) * 1000.0

    lock = threading.Lock()
    samples = []

    def work(worker):
        nonlocal errors_all, deadlocks_all
        local_lat, local_err, local_dead = [], 0, 0
        sample = ""
        with pool.connection() as conn:
            with conn.cursor() as cur:
                for j in range(ops):
                    write = mode == "write" or (mode == "mixed" and j % 10 >= 7)
                    t = time.perf_counter()
                    try:
                        if write:
                            cur.execute(
                                "INSERT INTO bench_writes (id, k, payload, tag) VALUES (%s,%s,%s,%s)",
                                (worker * 1_000_000 + j + 1, (j * 37) % max(rows, 1),
                                 "payload-w" + str(worker), "t" + str(j % 10)),
                                prepare=True)
                        else:
                            cur.execute("SELECT id, payload FROM bench_events WHERE id = %s",
                                        (((worker * ops + j) * 7) % rows + 1,), prepare=True)
                            cur.fetchall()
                    except psycopg.Error as exc:
                        local_err += 1
                        local_dead += 1 if is_deadlock(exc) else 0
                        sample = sample or str(exc).splitlines()[0][:200]
                    local_lat.append((time.perf_counter() - t) * 1000.0)
        with lock:
            lat_all.extend(local_lat)
            errors_all += local_err
            deadlocks_all += local_dead
            if sample:
                samples.append(sample)

    start = time.perf_counter()
    threads = [threading.Thread(target=work, args=(w,)) for w in range(workers)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    wall_ms = (time.perf_counter() - start) * 1000.0
    pool.close()

    result = dict(summarise(lat_all, wall_ms), errors=errors_all, deadlocks=deadlocks_all,
                  workers=workers, connections=workers, setup_ms=round(setup_ms, 3),
                  error_sample=samples[0] if samples else "")
    return result


def run_conn(conninfo, case, rows, iterations):
    scenario = case["scenario"]
    lat, errors, sample = [], 0, ""
    setup_ms = 0.0
    connections = 0

    if scenario == "connect":
        for _ in range(iterations):
            t0 = time.perf_counter()
            try:
                c = connect(conninfo)
                with c.cursor() as cur:
                    cur.execute("SELECT 1")
                    cur.fetchone()
                c.close()
            except psycopg.Error as exc:
                errors += 1
                sample = sample or str(exc).splitlines()[0][:200]
            lat.append((time.perf_counter() - t0) * 1000.0)
        connections = 1
    elif scenario == "reuse":
        t0 = time.perf_counter()
        conn = connect(conninfo)
        setup_ms = (time.perf_counter() - t0) * 1000.0
        with conn.cursor() as cur:
            for i in range(iterations):
                t = time.perf_counter()
                try:
                    cur.execute("SELECT id, payload FROM bench_events WHERE id = %s",
                                (((i * 7) % rows) + 1,), prepare=True)
                    cur.fetchone()
                except psycopg.Error as exc:
                    errors += 1
                    sample = sample or str(exc).splitlines()[0][:200]
                lat.append((time.perf_counter() - t) * 1000.0)
        conn.close()
        connections = 1
    elif scenario == "pool":
        t0 = time.perf_counter()
        pool = ConnectionPool(conninfo, min_size=10, max_size=10, timeout=60,
                              kwargs={"application_name": "kode-bench-" + LANGUAGE,
                                      "autocommit": True})
        pool.wait()
        setup_ms = (time.perf_counter() - t0) * 1000.0
        for i in range(iterations):
            t = time.perf_counter()
            try:
                with pool.connection() as conn:
                    with conn.cursor() as cur:
                        cur.execute("SELECT id, payload FROM bench_events WHERE id = %s",
                                    (((i * 7) % rows) + 1,), prepare=True)
                        cur.fetchone()
            except psycopg.Error as exc:
                errors += 1
                sample = sample or str(exc).splitlines()[0][:200]
            lat.append((time.perf_counter() - t) * 1000.0)
        pool.close()
        connections = 10
    elif scenario == "exhaust":
        t0 = time.perf_counter()
        pool = ConnectionPool(conninfo, min_size=4, max_size=4, timeout=60,
                              kwargs={"application_name": "kode-bench-" + LANGUAGE,
                                      "autocommit": True})
        pool.wait()
        setup_ms = (time.perf_counter() - t0) * 1000.0
        lock = threading.Lock()

        def worker(w):
            nonlocal errors, sample
            local = []
            for j in range(iterations):
                t = time.perf_counter()
                try:
                    with pool.connection() as conn:
                        with conn.cursor() as cur:
                            cur.execute("SELECT id FROM bench_events WHERE id = %s",
                                        (((w * iterations + j) % rows) + 1,), prepare=True)
                            cur.fetchone()
                except psycopg.Error as exc:
                    with lock:
                        errors += 1
                        sample = sample or str(exc).splitlines()[0][:200]
                local.append((time.perf_counter() - t) * 1000.0)
            with lock:
                lat.extend(local)

        threads = [threading.Thread(target=worker, args=(w,)) for w in range(20)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        pool.close()
        connections = 4
    elif scenario == "recover":
        control = connect(conninfo)
        for i in range(iterations):
            t0 = time.perf_counter()
            try:
                victim = connect(conninfo)
                with victim.cursor() as cur:
                    cur.execute("SELECT pg_backend_pid()")
                    pid = cur.fetchone()[0]
                with control.cursor() as cur:
                    cur.execute("SELECT pg_terminate_backend(%s)", (pid,))
                    cur.fetchone()
                try:
                    with victim.cursor() as cur:
                        cur.execute("SELECT 1")
                        cur.fetchone()
                except psycopg.Error:
                    errors += 1  # expected: the backend was terminated
                victim.close()
                with control.cursor() as cur:
                    cur.execute("SELECT count(*) FROM bench_events WHERE id = 1")
                    cur.fetchone()
            except psycopg.Error as exc:
                errors += 1
                sample = sample or str(exc).splitlines()[0][:200]
            lat.append((time.perf_counter() - t0) * 1000.0)
        control.close()
        connections = 2
    else:
        raise ValueError("unknown connection scenario: " + scenario)

    wall_ms = sum(lat)
    return dict(summarise(lat, wall_ms), errors=errors, error_sample=sample,
                setup_ms=round(setup_ms, 3), connections=connections,
                reconnect_ok=scenario == "recover" and errors == iterations)


# ------------------------------------------------------------------ verification
def run_checks(conn, case, rows, workers, ops, iterations):
    out = []
    for check in case.get("checks", []):
        names = check.get("params", [])
        sql, order = compile_sql(check["sql"])
        with conn.cursor() as cur:
            cur.execute(sql, values_for(order, names, 0, rows))
            row = cur.fetchone() or ()
            out.append([render(v) for v in row])
    return out


def render(value):
    if value is None:
        return "NULL"
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value)


# ------------------------------------------------------------------ main
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--workload", required=True)
    ap.add_argument("--cases", required=True)
    ap.add_argument("--rows", type=int, default=10000)
    ap.add_argument("--ops", type=int, default=200)
    ap.add_argument("--host", default=os.environ.get("PGHOST", "postgres"))
    ap.add_argument("--port", default=os.environ.get("PGPORT", "5432"))
    ap.add_argument("--user", default=os.environ.get("PGUSER", "bench"))
    ap.add_argument("--password", default=os.environ.get("PGPASSWORD", "bench"))
    ap.add_argument("--db", default=os.environ.get("PGDATABASE", "kode_bench"))
    ap.add_argument("--only", default="")
    args = ap.parse_args()

    with open(args.cases) as f:
        spec = json.load(f)
    chunk = int(spec.get("chunk", 500))
    cases = [c for c in spec["cases"] if c["workload"] == args.workload]
    if args.only:
        wanted = set(args.only.split(","))
        cases = [c for c in cases if c["id"] in wanted]
    if not cases:
        print(json.dumps({"language": LANGUAGE, "workload": args.workload, "error": "no cases"}), flush=True)
        return 2

    conninfo = (f"host={args.host} port={args.port} user={args.user} "
                f"password={args.password} dbname={args.db}")
    conn = connect(conninfo)
    conn.autocommit = True

    for case in cases:
        # Concurrency cases have no iteration count: each worker runs --ops.
        if case["shape"] == "conc":
            iterations = args.ops * case["workers"]
        else:
            declared = case.get("iterations", "rows")
            iterations = args.rows if declared == "rows" else int(declared)
        if case.get("setup"):
            conn.autocommit = True
            with conn.cursor() as cur:
                cur.execute(case["setup"].replace("{rows}", str(args.rows)))
        started_cpu = cpu_ms()
        t0 = time.perf_counter()
        shape = case["shape"]
        if shape == "stmt_loop":
            res = run_stmt_loop(conn, case, args.rows, iterations)
        elif shape == "query_loop":
            res = run_query_loop(conn, case, args.rows, iterations)
        elif shape == "chunk_insert":
            res = run_chunk_insert(conn, case, args.rows, iterations, chunk, False)
        elif shape == "chunk_insert_txn":
            res = run_chunk_insert(conn, case, args.rows, iterations, chunk, True)
        elif shape == "batch_update":
            res = run_batch_update(conn, case, args.rows, iterations, chunk)
        elif shape == "batch_delete":
            res = run_batch_delete(conn, case, args.rows, iterations, chunk)
        elif shape == "txn_loop":
            res = run_txn_loop(conn, case, args.rows, iterations)
        elif shape == "txn_case":
            res = run_txn_case(conn, case, args.rows, iterations)
        elif shape == "app_order_txn":
            res = run_app_order_txn(conn, case, args.rows, iterations)
        elif shape == "conc":
            res = run_conc(conninfo, case, args.rows, args.ops)
            iterations = args.ops * case["workers"]
        elif shape == "conn":
            res = run_conn(conninfo, case, args.rows, iterations)
        else:
            raise ValueError("unknown shape: " + shape)
        case_cpu = cpu_ms() - started_cpu
        if not conn.closed:
            conn.autocommit = True
        checks = run_checks(conn, case, args.rows, case.get("workers", 0), args.ops, iterations)

        record = {
            "language": LANGUAGE,
            "workload": args.workload,
            "case": case["id"],
            "shape": shape,
            "rows": args.rows,
            "iterations": iterations,
            "expect_errors": bool(case.get("expect_errors", False)),
            "checks": checks,
            "cpu_ms": round(case_cpu, 3),
            "peak_rss_kb": peak_rss_kb(),
            "elapsed_ms": round((time.perf_counter() - t0) * 1000.0, 3),
            "setup_ms": res.pop("setup_ms", 0.0),
            "error_sample": res.pop("error_sample", ""),
            "status": "ok",
        }
        if "throughput" not in res:
            res["throughput"] = round(res["ops"] / (res["wall_ms"] / 1000.0), 2) if res["wall_ms"] > 0 else 0.0
        record.update(res)
        print(json.dumps(record), flush=True)

    conn.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())

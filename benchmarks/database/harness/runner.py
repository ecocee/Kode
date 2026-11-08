#!/usr/bin/env python3
"""Benchmark orchestrator: one shared PostgreSQL, one client invocation at a time.

Design rules implemented here:
  * one database, one schema, one dataset for every language;
  * the database is re-seeded before every invocation, so no language inherits
    another language's leftovers;
  * one client invocation at a time (no parallel languages), so they do not
    compete for the same server;
  * warm-up before recording, repetitions for every case, median reported;
  * every case is verified twice (through the client's driver and through psql)
    before its numbers are kept;
  * raw records are written to results/raw/<run-id>/ before anything is
    summarised.

Kode runs like every other language: its client (kode/client.kode) is compiled
into a container image and uses the PostgreSQL client built into the Kode runtime.
Cases a client cannot run (Kode has no connection pool) are reported as
"unsupported" by the client itself and shown as UNSUPPORTED in the report.
"""

import argparse
import datetime
import json
import os
import platform
import shutil
import subprocess
import sys
import time
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
DB_DIR = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(DB_DIR, "common"))

import config          # noqa: E402
import monitoring      # noqa: E402
import report          # noqa: E402
import validation      # noqa: E402

# Workloads that write to the canonical tables must start from fresh data for
# every single invocation. The read-only ones only need one reset per round.
WRITE_WORKLOADS = {"insert", "update", "delete", "transactions", "concurrency", "application"}


def log(message):
    print(f"[{datetime.datetime.now().strftime('%H:%M:%S')}] {message}", flush=True)


def human_bytes(value):
    if not value:
        return "unknown"
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if value < 1024 or unit == "TB":
            return f"{value:.1f} {unit}" if unit != "B" else f"{int(value)} B"
        value /= 1024.0


def sh(*args, timeout=60):
    try:
        proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired):
        return ""
    if proc.returncode != 0:
        return ""
    return proc.stdout.decode("utf-8", "replace").strip()


def cpu_brand():
    text = sh("sysctl", "-n", "machdep.cpu.brand_string")
    if text:
        return text
    try:
        with open("/proc/cpuinfo") as f:
            for line in f:
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or "unknown"


def host_memory():
    text = sh("sysctl", "-n", "hw.memsize")
    if text.isdigit():
        return int(text)
    try:
        with open("/proc/meminfo") as f:
            for line in f:
                if line.startswith("MemTotal:"):
                    return int(line.split()[1]) * 1024
    except OSError:
        pass
    return 0


def load_cases():
    with open(config.CASES_FILE) as f:
        return json.load(f)


def load_invariants():
    path = os.path.join(os.path.dirname(config.CASES_FILE), "invariants.json")
    with open(path) as f:
        return json.load(f).get("invariants", {})


def check_invariants(db, invariants, workload, ctx):
    """Independent end-of-workload verification through psql."""
    for item in invariants.get(workload, []):
        expected = [[validation.evaluate(v, ctx) for v in item.get("expect", [])]]
        out = db.psql(item["sql"])
        if not out:
            return False, f"{workload} invariant returned nothing: {item['sql'][:80]}"
        actual = [out.splitlines()[0].split("|")]
        ok, message = validation.compare(actual, expected, f"{workload} invariant", "psql")
        if not ok:
            return False, message
    return True, ""


def image_size_mb(service):
    text = sh("docker", "image", "inspect", "-f", "{{.Size}}", f"kode-db-bench-{service}")
    try:
        return round(int(text) / 1048576.0, 1)
    except ValueError:
        return None


def build_images(languages):
    sizes = {}
    for lang in languages:
        log(f"building client image for {lang}")
        started = time.time()
        proc = subprocess.run(
            ["docker", "compose", "-f", config.COMPOSE_FILE, "build", f"client-{lang}"],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        if proc.returncode != 0:
            tail = proc.stdout.decode("utf-8", "replace")[-3000:]
            raise SystemExit(f"image build failed for {lang}:\n{tail}")
        sizes[lang] = {"build_s": round(time.time() - started, 1),
                       "image_mb": image_size_mb(f"client-{lang}")}
        log(f"built {lang} in {sizes[lang]['build_s']}s ({sizes[lang]['image_mb']} MB)")
    return sizes


def invoke(db, lang, workload, rows, ops, case_ids, timeout, run_id, repetition, warmup=False):
    """Run one client container, sample the server while it runs, return a record."""
    name = f"kode-db-bench-run-{run_id}-{os.getpid()}-{repetition}-{lang}-{workload}".lower()
    cmd = [
        "docker", "compose", "-f", config.COMPOSE_FILE, "run", "--rm", "-T", "--name", name,
        f"client-{lang}",
        "--workload", workload,
        "--rows", str(rows),
        "--ops", str(ops),
        "--cases", "/bench/workloads/cases.json",
    ]
    if case_ids:
        cmd += ["--only", ",".join(case_ids)]

    sampler = monitoring.Sampler(db)
    sampler.start()
    started = time.time()
    timed_out = False
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                start_new_session=True)
        try:
            out, err = proc.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            try:
                os.killpg(os.getpgid(proc.pid), 9)
            except OSError:
                pass
            try:
                out, err = proc.communicate(timeout=30)
            except subprocess.TimeoutExpired:
                out, err = b"", b""
            sh("docker", "rm", "-f", name, timeout=60)
    except OSError as exc:
        sampler.stop()
        return {"status": config.STATUS_FAIL, "reason": f"cannot start docker: {exc}",
                "cases": [], "stdout": "", "stderr": ""}
    elapsed = time.time() - started
    sampler.stop()
    resources = sampler.summary()

    stdout = out.decode("utf-8", "replace") if out else ""
    stderr = err.decode("utf-8", "replace") if err else ""
    records = []
    for line in stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError:
            pass

    if timed_out:
        status = config.STATUS_TIMEOUT
        reason = f"exceeded {timeout}s ({elapsed:.0f}s elapsed)"
    elif proc.returncode != 0:
        status = config.STATUS_FAIL
        reason = f"exit {proc.returncode}: {(stderr.strip().splitlines() or ['no stderr'])[-1][:300]}"
    elif not records:
        status = config.STATUS_FAIL
        reason = "client produced no JSON records"
    else:
        status = config.STATUS_PASS
        reason = ""

    return {
        "status": status,
        "reason": reason,
        "cases": records,
        "stdout": stdout,
        "stderr": stderr,
        "elapsed_s": round(elapsed, 2),
        "resources": resources,
        "warmup": warmup,
    }


def verify_case(db, case, ctx, record):
    """Compare the client's own post-case checks and measure server time.

    Per-case checks are executed by the client immediately after the case, while
    the state it asserts is current. Driver-independent verification lives in
    workloads/invariants.json and runs once per workload through psql.
    """
    expected = validation.expected(case, ctx)
    checks_ok, checks_msg = validation.compare(record.get("checks"), expected, case["id"], "client")
    if not checks_ok:
        return config.STATUS_FAIL, checks_msg, None

    errors = record.get("errors") or 0
    if errors and not case.get("expect_errors"):
        sample = (record.get("error_sample") or "").strip()
        return config.STATUS_FAIL, f"{errors} query errors: {sample[:200]}", None

    db_exec = None
    # INSERT is excluded: the case has already written that key, and EXPLAIN
    # ANALYZE cannot re-run the statement even inside a rolled-back transaction.
    # Shapes that build their own SQL (chunks, concurrency, connections, the
    # application order transaction) carry no `sql` field at all.
    statement = (case.get("sql") or "").lstrip().upper()
    measurable = any(statement.startswith(verb) for verb in ("SELECT", "UPDATE", "DELETE"))
    if case["shape"] in ("stmt_loop", "query_loop", "txn_loop") and measurable:
        params = validation.params_for(case.get("params", []), 0, ctx["rows"])
        samples = [db.explain_ms(case["sql"], params) for _ in range(3)]
        samples = [s for s in samples if s is not None]
        db_exec = round(sorted(samples)[len(samples) // 2], 4) if samples else None
    return config.STATUS_PASS, "", db_exec


def unsupported_case(case_id, reason):
    """Record for a case the client itself reported as unsupported."""
    return {
        "case": case_id,
        "status": "unsupported",
        "reason": reason,
        "throughput": None, "p50_ms": None, "p95_ms": None, "p99_ms": None,
        "ops": 0, "cpu_ms": None, "peak_rss_kb": None, "setup_ms": None,
        "db_exec_ms": None, "db_total_ms": None, "errors": 0,
        "db_conns_max": 0, "db_mem_mb": 0.0, "db_cpu_s": 0.0,
    }


def main():
    ap = argparse.ArgumentParser(description="Kode database benchmark")
    ap.add_argument("--all", action="store_true", help="run every workload and language")
    ap.add_argument("--workload", default="", help="comma separated workloads (default: all)")
    ap.add_argument("--language", default="", help="comma separated languages (default: all)")
    ap.add_argument("--rows", type=int, default=config.DEFAULT_ROWS, help="dataset size")
    ap.add_argument("--repetitions", type=int, default=config.DEFAULT_REPETITIONS)
    ap.add_argument("--workers", default="", help="concurrency worker counts, e.g. 10,50,100")
    ap.add_argument("--ops", type=int, default=config.DEFAULT_OPS, help="operations per concurrent worker")
    ap.add_argument("--timeout", type=int, default=config.DEFAULT_TIMEOUT_S, help="seconds per client invocation")
    ap.add_argument("--build", action="store_true", help="build client images and run")
    ap.add_argument("--build-only", action="store_true", help="build client images and exit")
    ap.add_argument("--no-build", action="store_true", help="do not build images")
    ap.add_argument("--reset", action="store_true", help="start PostgreSQL, apply schema and seed, then exit")
    ap.add_argument("--down", action="store_true", help="stop the PostgreSQL container after the run")
    ap.add_argument("--quick", action="store_true", help="small smoke run: 1,000 rows, 1 repetition")
    ap.add_argument("--no-warmup", action="store_true", help="skip the discarded warm-up invocation")
    args = ap.parse_args()

    if not shutil.which("docker"):
        raise SystemExit("docker is required")
    if args.quick:
        args.rows = min(args.rows, 1000) if args.rows != config.DEFAULT_ROWS else 1000
        args.repetitions = 1
        args.ops = min(args.ops, 50)

    languages = [l.strip() for l in args.language.split(",") if l.strip()] or list(config.ALL_LANGUAGES)
    unknown = [l for l in languages if l not in config.ALL_LANGUAGES]
    if unknown:
        raise SystemExit(f"unknown language(s): {', '.join(unknown)}")
    # Every language, Kode included, now has a client container.
    driver_languages = list(languages)

    workloads = [w.strip() for w in args.workload.split(",") if w.strip()] or list(config.WORKLOADS)
    unknown = [w for w in workloads if w not in config.WORKLOADS]
    if unknown:
        raise SystemExit(f"unknown workload(s): {', '.join(unknown)}")

    workers = [int(w) for w in args.workers.split(",") if w.strip()] or list(config.DEFAULT_WORKERS)

    spec = load_cases()
    all_cases = spec["cases"]
    invariants = load_invariants()

    db = monitoring.Database()
    log("starting PostgreSQL 17 (shared instance for every language)")
    db.up()
    info = db.server_info()
    log(f"PostgreSQL {info['version']} ready, fsync={info['fsync']}, "
        f"max_connections={info['max_connections']}, isolation={info['isolation']}")
    db.apply_schema()

    if args.reset:
        db.reset(args.rows)
        log(f"database reset and seeded with {args.rows} rows per dataset")
        return 0

    build_info = {}
    if args.build_only or (not args.no_build):
        build_info = build_images(driver_languages)
    if args.build_only:
        return 0
    if args.no_build:
        for lang in driver_languages:
            if image_size_mb(f"client-{lang}") is None:
                raise SystemExit(f"image for {lang} is missing; run without --no-build")

    run_id = datetime.datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + uuid.uuid4().hex[:6]
    run_dir = os.path.join(config.RAW_DIR, run_id)
    os.makedirs(run_dir, exist_ok=True)
    os.makedirs(config.REPORTS_DIR, exist_ok=True)

    notes = [
        "One PostgreSQL 17 instance per host run; every language connects to it over the "
        "same Docker network with the same credentials and transaction isolation.",
        "PGDATA is a tmpfs inside the container. Docker Desktop on macOS forwards fsync "
        "through a userspace filesystem, which adds tens of milliseconds of variance per "
        "COMMIT unrelated to the client language. fsync and synchronous_commit stay on.",
        "Every invocation re-seeds the database (schema/seed.sql) before the client starts; "
        "writes from one language can never reach another language's measurement.",
        "Concurrency cases use each driver's own pool sized to the worker count; "
        "connection cases measure setup separately from steady state.",
        "Latency percentiles are computed inside the client from per-operation timings. "
        "p50 is the median operation, not the median of the repetitions.",
        "Server execution time (`db exec`) comes from EXPLAIN (ANALYZE) of one "
        "representative query, run in a rolled-back transaction after the measurement "
        "window. It is PostgreSQL time, not client time.",
        "Warm-up: one discarded invocation per (language, workload) before the recorded "
        "repetitions.",
        "Kode uses the PostgreSQL client built into its runtime (plain TCP, no connection "
        "pool, no prepared-statement caching). Connection-pool cases are reported "
        "unsupported for Kode, and its concurrency cases use one connection per task.",
    ]
    if args.rows >= 100000:
        notes.append("The 100,000-row sweep uses 3 repetitions in the published run; "
                     "see the reproducing command in README.md.")

    environment = {
        "Host": platform.platform(),
        "CPU": cpu_brand(),
        "Cores": os.cpu_count(),
        "Host memory": human_bytes(host_memory()),
        "Docker": sh("docker", "version", "--format", "{{.Server.Version}}") or "unknown",
        "Docker Compose": sh("docker", "compose", "version", "--short") or "unknown",
        "PostgreSQL": info["version"],
        "Database size": info["database_size"],
        "max_connections": info["max_connections"],
        "shared_buffers": info["shared_buffers"],
        "fsync / isolation": f"{info['fsync']} / {info['isolation']}",
        "Dataset rows": args.rows,
        "Repetitions": args.repetitions,
        "Client images": ", ".join(f"{k} {v['image_mb']} MB" for k, v in build_info.items()) or "prebuilt",
    }

    invocations = []
    total_invocations = len(driver_languages) * len(workloads) * args.repetitions
    started_run = time.time()
    done = 0

    for repetition in range(1, args.repetitions + 1):
        offset = (repetition - 1) % max(len(driver_languages), 1)
        ordered = driver_languages[offset:] + driver_languages[:offset]
        for workload in workloads:
            workload_cases = [c for c in all_cases if c["workload"] == workload]
            case_ids = None
            if workload == "concurrency":
                keep = [c["id"] for c in workload_cases if c.get("workers") in workers]
                if not keep:
                    log(f"concurrency: no cases for workers {workers}; skipping")
                    continue
                case_ids = keep
                workload_cases = [c for c in workload_cases if c["id"] in keep]

            for index, lang in enumerate(ordered):
                done += 1
                # Read-only workloads cannot change the dataset, so one reset per
                # round is enough for them. Anything that writes resets every time,
                # so no language inherits another language's state.
                need_reset = workload in WRITE_WORKLOADS or index == 0
                if need_reset:
                    reset_ms = db.reset(args.rows)
                else:
                    reset_ms = 0.0

                if not args.no_warmup:
                    warm = invoke(db, lang, workload, args.rows, args.ops, case_ids,
                                  args.timeout, run_id, repetition, warmup=True)
                    if warm["status"] != config.STATUS_PASS:
                        log(f"warning: warm-up for {lang}/{workload} ended {warm['status']}: {warm['reason']}")
                    # The warm-up wrote to the tables; restore the seed state so the
                    # recorded run starts from exactly the same data as every other.
                    if workload in WRITE_WORKLOADS:
                        reset_ms = db.reset(args.rows)

                baseline = db.stats()
                result = invoke(db, lang, workload, args.rows, args.ops, case_ids,
                                args.timeout, run_id, repetition)
                after = db.stats()
                resources = result.pop("resources", {})
                result["db_total_ms"] = round(after["db_total_ms"] - baseline["db_total_ms"], 2)
                result["deadlocks"] = max(after["deadlocks"] - baseline["deadlocks"], 0)
                result.update(resources)
                result.update({
                    "language": lang,
                    "workload": workload,
                    "rows": args.rows,
                    "repetition": repetition,
                    "reset_ms": round(reset_ms, 1),
                    "started_at": datetime.datetime.now().isoformat(timespec="seconds"),
                })

                invariant_ctx = {"rows": args.rows, "ops": args.ops,
                                 "workers": max(workers), "iters": 0}
                invariants_ok, invariant_reason = (True, "")
                if result["status"] == config.STATUS_PASS:
                    invariants_ok, invariant_reason = check_invariants(
                        db, invariants, workload, invariant_ctx)
                    if not invariants_ok:
                        result["status"] = config.STATUS_FAIL
                        result["reason"] = invariant_reason

                records_by_id = {r["case"]: r for r in result["cases"]}
                verified = []
                for case in workload_cases:
                    record = records_by_id.get(case["id"])
                    if result["status"] != config.STATUS_PASS or record is None:
                        verified.append({"case": case["id"], "status": result["status"],
                                         "reason": result["reason"] or "case missing from client output"})
                        continue
                    # A client may declare a case it cannot run (Kode has no
                    # connection pool). That is reported, never faked, and does
                    # not fail the invocation.
                    if record.get("status") == "unsupported":
                        verified.append(unsupported_case(case["id"], record.get("error_sample", "")))
                        continue
                    if case["shape"] == "conc":
                        iterations = args.ops * case["workers"]
                    else:
                        declared = case.get("iterations", "rows")
                        iterations = args.rows if declared == "rows" else int(declared)
                    ctx = validation.context(args.rows, case, iterations, args.ops)
                    status, reason, db_exec = verify_case(db, case, ctx, record)
                    record["db_exec_ms"] = db_exec
                    record["status"] = "ok" if status == config.STATUS_PASS else "bad"
                    record.update({k: v for k, v in result.items()
                                   if k in ("resources", "db_total_ms", "deadlocks", "reset_ms")})
                    verified.append(record)
                    if status != config.STATUS_PASS:
                        result["status"] = config.STATUS_FAIL
                        result["reason"] = (result["reason"] + "; " if result["reason"] else "") + reason
                result["cases"] = verified
                invocations.append(result)

                with open(os.path.join(run_dir, f"{lang}-{workload}-r{repetition}.json"), "w") as f:
                    json.dump(result, f, indent=2)

                elapsed = time.time() - started_run
                eta = elapsed / done * (total_invocations - done)
                log(f"[{done}/{total_invocations}] {lang}/{workload} rep {repetition}: "
                    f"{result['status']} in {result['elapsed_s']}s "
                    f"(db {result['db_total_ms']:.0f} ms, {result['db_conns_max']} conns, "
                    f"{result['db_mem_mb']} MB; reset {reset_ms:.0f} ms; ETA {eta/60:.1f} min)")
                if result["status"] != config.STATUS_PASS:
                    log(f"    reason: {result['reason'][:300]}")

    # Kode now runs like every other language, so there is no separate
    # "unsupported" block and no capability note to write.
    kode_info = {}

    results = report.aggregate(invocations)
    results_doc = {
        "run": {
            "id": run_id,
            "started_at": datetime.datetime.now().isoformat(timespec="seconds"),
            "invocations": len(invocations),
            "languages": languages,
            "workloads": workloads,
            "workers": workers,
            "rows": args.rows,
            "repetitions": args.repetitions,
            "notes": notes,
        },
        "environment": environment,
        "drivers": config.LANGUAGE_VERSIONS,
        "kode": kode_info,
        "results": results,
    }
    report.write_results(results_doc, os.path.join(config.RESULTS_DIR, "results.json"))
    report.write_csv(results, os.path.join(config.RESULTS_DIR, "results.csv"))
    report.write_summary(results_doc, os.path.join(config.REPORTS_DIR, "summary.md"))

    write_kode_capability(kode_info, args.rows)

    passed = sum(1 for r in results if r["status"] == config.STATUS_PASS)
    failed = sum(1 for r in results if r["status"] == config.STATUS_FAIL)
    timeouts = sum(1 for r in results if r["status"] == config.STATUS_TIMEOUT)
    unsupported = sum(1 for r in results if r["status"] == config.STATUS_UNSUPPORTED)
    log(f"done in {(time.time() - started_run)/60:.1f} min: "
        f"{passed} PASS, {failed} FAIL, {timeouts} TIMEOUT, {unsupported} UNSUPPORTED")
    log(f"results: {config.RESULTS_DIR}/results.json, results.csv, reports/summary.md")
    log(f"raw: {run_dir}")

    if args.down:
        db.down()
    return 0 if failed == 0 and timeouts == 0 else 1


def write_kode_capability(kode_info, rows):
    """Write the Kode capability note next to the reports (only if one is provided)."""
    if not kode_info:
        return
    path = os.path.join(config.REPORTS_DIR, "kode-capability.md")
    lines = ["# Kode: database integration capabilities", ""]
    lines.append(kode_info.get("summary", ""))
    lines.append("")
    lines.append("**Verdict:** " + kode_info.get("verdict", "UNSUPPORTED"))
    lines.append("")
    lines.append("## What Kode has today")
    lines.append("")
    lines.append("| Capability | State | Evidence |")
    lines.append("|---|---|---|")
    for row in kode_info.get("capabilities", []):
        lines.append(f"| {row['name']} | {row['state']} | {row['evidence']} |")
    lines.append("")
    lines.append("## What was investigated for PostgreSQL access")
    lines.append("")
    for item in kode_info.get("investigations", []):
        lines.append(f"- **{item['path']}**: {item['result']}")
    lines.append("")
    lines.append("## Compiler evidence")
    lines.append("")
    probe = os.path.join(config.RAW_DIR, "kode-probe.txt")
    if os.path.exists(probe):
        with open(probe) as f:
            lines.append("```")
            lines.append(f.read().strip())
            lines.append("```")
    else:
        lines.append("Run `bash benchmarks/database/kode/probe.sh` to capture the compiler's "
                     "response to a database API.")
    lines.append("")
    lines.append("## Not done, and why")
    lines.append("")
    lines.append(kode_info.get("not_done", ""))
    lines.append("")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
"""Aggregate per-run measurements and write results.json / results.csv / summary.md."""

import csv
import json
import os
import statistics

import config

MEDIAN_METRICS = [
    "throughput", "wall_ms", "p50_ms", "p95_ms", "p99_ms", "ops", "cpu_ms",
    "peak_rss_kb", "setup_ms", "db_exec_ms", "db_total_ms", "errors",
    "db_conns_max", "db_mem_mb", "db_cpu_s", "rows_affected", "rows_fetched",
]

CSV_COLUMNS = [
    "language", "workload", "case", "rows", "status", "runs", "pass_runs",
    "ops", "throughput", "wall_ms", "p50_ms", "p95_ms", "p99_ms",
    "cpu_ms", "cpu_ms_per_op", "peak_rss_mb", "db_exec_ms", "client_ms_per_op",
    "db_conns_max", "db_mem_mb", "db_cpu_s", "errors", "reason",
]


def median_of(values):
    values = [v for v in values if v is not None]
    return round(statistics.median(values), 3) if values else None


def aggregate(invocations):
    """Group per-invocation records by (language, workload, case, rows)."""
    groups = {}
    for inv in invocations:
        key_base = (inv["language"], inv["workload"], inv["rows"])
        for record in inv.get("cases", []):
            key = key_base + (record["case"],)
            groups.setdefault(key, []).append((inv, record))

    rows = []
    for (language, workload, dataset_rows, case), items in groups.items():
        passes = [(inv, rec) for inv, rec in items if inv["status"] == config.STATUS_PASS and rec.get("status") == "ok"]
        statuses = [inv["status"] for inv, _ in items]
        status = config.STATUS_PASS if passes else (statuses[0] if statuses else config.STATUS_NOT_RUN)
        reason = next((inv.get("reason", "") for inv, _ in items if inv.get("reason")), "")
        entry = {
            "language": language,
            "workload": workload,
            "case": case,
            "rows": dataset_rows,
            "status": status,
            "reason": reason,
            "runs": len(items),
            "pass_runs": len(passes),
        }
        for metric in MEDIAN_METRICS:
            entry[metric] = median_of([rec.get(metric) for _, rec in passes])
        # Derived per-operation figures. A median of ratios is preferred over a
        # ratio of medians so a single slow run cannot move the whole number.
        cpu_per_op = [rec["cpu_ms"] / rec["ops"] for _, rec in passes if rec.get("ops")]
        client_per_op = []
        for _, rec in passes:
            if not rec.get("ops") or rec.get("db_exec_ms") is None or rec.get("p50_ms") is None:
                continue
            client_per_op.append(rec["p50_ms"] - rec["db_exec_ms"])
        entry["cpu_ms_per_op"] = median_of(cpu_per_op)
        entry["client_ms_per_op"] = median_of(client_per_op)
        entry["peak_rss_mb"] = round(entry["peak_rss_kb"] / 1024.0, 1) if entry["peak_rss_kb"] else None
        entry["per_run"] = [
            {
                "repetition": inv.get("repetition"),
                "status": inv["status"],
                "record": {k: rec.get(k) for k in MEDIAN_METRICS if k in rec},
            }
            for inv, rec in items
        ]
        rows.append(entry)

    rows.sort(key=lambda r: (config.WORKLOADS.index(r["workload"])
                             if r["workload"] in config.WORKLOADS else 99,
                             r["case"], r["language"]))
    return rows


def write_results(doc, path):
    with open(path, "w") as f:
        json.dump(doc, f, indent=2, sort_keys=False)
        f.write("\n")


def write_csv(rows, path):
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_COLUMNS, extrasaction="ignore")
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


# --------------------------------------------------------------------- markdown
def fmt(value, digits=1):
    if value is None:
        return "—"
    if isinstance(value, float):
        return f"{value:,.{digits}f}"
    if isinstance(value, int):
        return f"{value:,}"
    return str(value)


def write_summary(doc, path):
    lines = []
    add = lines.append
    add("# PostgreSQL benchmark: Kode against Rust, Go, Node.js, Java, Python and C#")
    add("")
    add(f"Run `{doc['run']['id']}` started {doc['run']['started_at']}.")
    add("")
    add("Every number below comes from the same PostgreSQL instance, the same schema,")
    add("the same seed data and the same SQL. Each language client reads")
    add("`workloads/cases.json`, so query text and iteration counts are shared by")
    add("construction. Throughput and latency are medians over passing repetitions.")
    add("")

    add("## Environment")
    add("")
    add("| Item | Value |")
    add("|---|---|")
    for key, value in doc["environment"].items():
        add(f"| {key} | {value} |")
    add("")

    add("## Languages")
    add("")
    add("| Language | Runtime | Driver | Pool | Build |")
    add("|---|---|---|---|---|")
    for lang in config.DRIVER_LANGUAGES:
        v = config.LANGUAGE_VERSIONS[lang]
        add(f"| {lang} | {v['runtime']} | {v['driver']} | {v['pool']} | {v['build']} |")
    add("")
    add("Kode is listed separately: it has no PostgreSQL client, no C FFI and no raw")
    add("socket API, so its cases are UNSUPPORTED rather than benchmarked against a")
    add("substitute implementation. Evidence is in `results/reports/kode-capability.md`.")
    add("")

    # ---------------------------------------------------------- status matrix
    add("## Status matrix")
    add("")
    add("PASS = every repetition ran and every correctness check agreed. FAIL = a")
    add("client error or a wrong result. TIMEOUT = the harness killed it. UNSUPPORTED")
    add("= the language has no supported way to run the workload. NOT RUN = excluded")
    add("from this run.")
    add("")
    langs = sorted({r["language"] for r in doc["results"]},
                   key=lambda l: (config.ALL_LANGUAGES.index(l) if l in config.ALL_LANGUAGES else 99, l))
    add("| Workload | " + " | ".join(langs) + " |")
    add("|---|" + "|".join(["---"] * len(langs)) + "|")
    for workload in config.WORKLOADS:
        cells = []
        for lang in langs:
            cases = [r for r in doc["results"] if r["language"] == lang and r["workload"] == workload]
            if not cases:
                cells.append("NOT RUN")
                continue
            counts = {}
            for r in cases:
                counts[r["status"]] = counts.get(r["status"], 0) + 1
            if set(counts) == {config.STATUS_PASS}:
                cells.append(f"PASS ({counts[config.STATUS_PASS]})")
            elif config.STATUS_PASS in counts:
                others = ", ".join(f"{k}:{v}" for k, v in sorted(counts.items()) if k != config.STATUS_PASS)
                cells.append(f"PARTIAL {others}")
            else:
                cells.append(next(iter(counts)))
        add(f"| {workload} | " + " | ".join(cells) + " |")
    add("")

    # ---------------------------------------------------------- per workload
    for workload in config.WORKLOADS:
        cases = [r for r in doc["results"] if r["workload"] == workload]
        if not cases:
            continue
        add(f"## {workload}")
        add("")
        add("`ops/s` and latency are client-side; `db exec` is PostgreSQL's own")
        add("execution time for one representative query of the case (EXPLAIN ANALYZE,")
        add("rolled back); `client` is p50 minus `db exec`, i.e. driver plus runtime")
        add("overhead. For concurrent cases server time is summed across parallel")
        add("sessions, so `client` can be negative and should be read as a residual.")
        add("")
        add("| Case | Rows | Language | Status | ops/s | p50 ms | p95 ms | p99 ms | db exec ms | client ms | CPU ms/op | Peak RSS MB | DB conns |")
        add("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        for case_id in sorted({r["case"] for r in cases}):
            group = [r for r in cases if r["case"] == case_id]
            group.sort(key=lambda r: (r["throughput"] is None, -(r["throughput"] or 0)))
            for r in group:
                add("| {case} | {rows} | {lang} | {status} | {tput} | {p50} | {p95} | {p99} | {db} | {client} | {cpu} | {rss} | {conns} |".format(
                    case=case_id,
                    rows=fmt(r["rows"]),
                    lang=r["language"],
                    status=r["status"],
                    tput=fmt(r["throughput"], 0) if r["status"] == config.STATUS_PASS else "—",
                    p50=fmt(r["p50_ms"], 3) if r["status"] == config.STATUS_PASS else "—",
                    p95=fmt(r["p95_ms"], 3) if r["status"] == config.STATUS_PASS else "—",
                    p99=fmt(r["p99_ms"], 3) if r["status"] == config.STATUS_PASS else "—",
                    db=fmt(r["db_exec_ms"], 3),
                    client=fmt(r["client_ms_per_op"], 3),
                    cpu=fmt(r["cpu_ms_per_op"], 3),
                    rss=fmt(r["peak_rss_mb"], 1),
                    conns=fmt(r["db_conns_max"], 0),
                ))
        add("")

    # ---------------------------------------------------------- failures
    add("## Failures and unsupported cases")
    add("")
    problems = [r for r in doc["results"] if r["status"] != config.STATUS_PASS]
    if not problems:
        add("None: every selected language and workload passed.")
        add("")
    else:
        add("| Language | Case | Status | Reason |")
        add("|---|---|---|---|")
        seen = set()
        for r in problems:
            key = (r["language"], r["workload"], r["status"], r["reason"])
            if key in seen:
                continue
            seen.add(key)
            add(f"| {r['language']} | {r['workload']} | {r['status']} | {r['reason'] or '—'} |")
        add("")

    add("## Method and limits")
    add("")
    for note in doc["run"].get("notes", []):
        add(f"- {note}")
    add("")
    add("Raw per-repetition records: `results/raw/`. Machine-readable results:")
    add("`results/results.json` and `results/results.csv`.")
    add("")

    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")

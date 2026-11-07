#!/usr/bin/env python3
"""Summarize results.tsv: median over repetitions per configuration.

usage: summarize.py results.tsv summary.tsv [build_times.tsv]
"""
import csv
import statistics
import sys
from collections import defaultdict

GROUP = ["endpoint", "connections", "target_requests", "cpus", "memory", "threads"]
METRICS = [
    "rps", "p50_ms", "p95_ms", "p99_ms", "duration_s",
    "cpu_avg_pct", "cpu_max_pct", "cpu_seconds", "throttled_ms",
    "mem_avg_mb", "mem_max_mb", "mem_peak_mb", "idle_mem_mb",
    "startup_ms", "image_mb",
]


def num(row, key):
    try:
        return float(row[key])
    except (KeyError, ValueError):
        return 0.0


def med(values):
    return statistics.median(values) if values else 0.0


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    src, dst = sys.argv[1], sys.argv[2]
    build_file = sys.argv[3] if len(sys.argv) > 3 else None

    with open(src, newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t"))

    cells = defaultdict(list)
    for r in rows:
        cells[tuple(r[k] for k in GROUP) + (r["language"],)].append(r)

    out = []
    for key, rs in cells.items():
        ok = [r for r in rs if r["status"] == "ok"]
        rec = dict(zip(GROUP + ["language"], key))
        rec["runs"] = len(rs)
        rec["ok_runs"] = len(ok)
        rec["status"] = "ok" if ok else (rs[0]["status"] if rs else "none")
        for m in METRICS:
            rec[m] = med([num(r, m) for r in ok])
        # Efficiency: work done per resource consumed
        rec["req_per_cpu_s"] = med([
            num(r, "successful_requests") / num(r, "cpu_seconds")
            for r in ok if num(r, "cpu_seconds") > 0
        ])
        rec["rps_per_100mb"] = med([
            num(r, "rps") / num(r, "mem_max_mb") * 100
            for r in ok if num(r, "mem_max_mb") > 0
        ])
        out.append(rec)

    best = defaultdict(float)
    for rec in out:
        k = tuple(rec[g] for g in GROUP)
        best[k] = max(best[k], rec["rps"])
    for rec in out:
        b = best[tuple(rec[g] for g in GROUP)]
        rec["rps_vs_best_pct"] = rec["rps"] / b * 100 if b > 0 else 0.0

    def sort_key(rec):
        return (rec["endpoint"], int(rec["connections"]), int(rec["target_requests"]),
                float(rec["cpus"]), rec["memory"], rec["threads"], -rec["rps"])

    out.sort(key=sort_key)

    cols = (GROUP + ["language", "runs", "ok_runs", "status"] + METRICS
            + ["req_per_cpu_s", "rps_per_100mb", "rps_vs_best_pct"])
    with open(dst, "w", newline="") as f:
        w = csv.writer(f, delimiter="\t")
        w.writerow(cols)
        for rec in out:
            w.writerow([f"{rec[c]:.3f}" if isinstance(rec[c], float) else rec[c] for c in cols])

    # Console table
    head = f"{'endpoint':8} {'conn':>5} {'reqs':>7} {'cpus':>5} {'mem':>6} {'thr':>4} {'lang':8} " \
           f"{'ok':>5} {'rps':>10} {'p50ms':>7} {'p99ms':>8} {'cpu%':>6} {'memMB':>7} " \
           f"{'req/cpu-s':>10} {'vs best':>8}"
    print()
    print(head)
    print("-" * len(head))
    last = None
    for rec in out:
        grp = tuple(rec[g] for g in GROUP)
        if last is not None and grp != last:
            print()
        last = grp
        print(f"{rec['endpoint']:8} {rec['connections']:>5} {rec['target_requests']:>7} "
              f"{rec['cpus']:>5} {rec['memory']:>6} {rec['threads']:>4} {rec['language']:8} "
              f"{str(rec['ok_runs']) + '/' + str(rec['runs']):>5} {rec['rps']:>10.0f} "
              f"{rec['p50_ms']:>7.2f} {rec['p99_ms']:>8.2f} {rec['cpu_avg_pct']:>6.0f} "
              f"{rec['mem_max_mb']:>7.1f} {rec['req_per_cpu_s']:>10.0f} "
              f"{rec['rps_vs_best_pct']:>7.0f}%"
              + ("" if rec["status"] == "ok" else f"  [{rec['status']}]"))

    # Static facts per language: start-up time, idle memory, image size, build time
    if build_file:
        try:
            with open(build_file, newline="") as f:
                builds = {r["target"]: r for r in csv.DictReader(f, delimiter="\t")}
        except OSError:
            builds = {}
        langs = sorted({rec["language"] for rec in out})
        print("\nPer language (median over all ok runs)")
        print(f"{'lang':8} {'startup ms':>11} {'idle MB':>8} {'image MB':>9} {'build s':>8}")
        for lang in langs:
            ok = [r for r in rows if r["language"] == lang and r["status"] == "ok"]
            b = builds.get(lang, {})
            print(f"{lang:8} {med([num(r, 'startup_ms') for r in ok]):>11.0f} "
                  f"{med([num(r, 'idle_mem_mb') for r in ok]):>8.1f} "
                  f"{med([num(r, 'image_mb') for r in ok]):>9.1f} "
                  f"{float(b.get('build_seconds', 0) or 0):>8.1f}")
    print()


if __name__ == "__main__":
    main()
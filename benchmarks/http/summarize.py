#!/usr/bin/env python3

import csv
import statistics
import sys
from collections import defaultdict

if len(sys.argv) != 3:
    print("Usage: summarize.py results.tsv summary.tsv")
    sys.exit(1)

source, destination = sys.argv[1], sys.argv[2]

def status_for(row):
    successful = int(row["successful_requests"])
    failed = int(row["failed_requests"])
    unattempted = int(row["unattempted_requests"])
    target = int(row["target_requests"])
    return "ok" if successful == target and failed == 0 and unattempted == 0 else "fail"

groups = defaultdict(list)

with open(source, newline="") as file:
    reader = csv.DictReader(file, delimiter="\t")

    for row in reader:
        row["status"] = status_for(row)
        key = (
            row["language"],
            row["endpoint"],
            row["connections"],
            row["target_requests"],
        )
        groups[key].append(row)

columns = [
    "language",
    "endpoint",
    "connections",
    "target_requests",
    "runs",
    "successful_runs",
    "median_rps",
    "median_p50_ms",
    "median_p95_ms",
    "median_p99_ms",
    "median_process_rss_kb",
    "median_container_current_kb",
    "max_container_peak_kb",
    "total_failed_requests",
    "total_unattempted_requests",
]


def median(rows, key):
    values = [float(row[key]) for row in rows]
    return round(statistics.median(values), 3)

with open(destination, "w", newline="") as file:
    writer = csv.DictWriter(
        file,
        fieldnames=columns,
        delimiter="\t",
    )
    writer.writeheader()

    for key in sorted(groups):
        rows = groups[key]

        successful = sum(row["status"] == "ok" for row in rows)

        output = {
            "language": key[0],
            "endpoint": key[1],
            "connections": key[2],
            "target_requests": key[3],
            "runs": len(rows),
            "successful_runs": successful,
            "median_rps": median(rows, "rps"),
            "median_p50_ms": median(rows, "p50_ms"),
            "median_p95_ms": median(rows, "p95_ms"),
            "median_p99_ms": median(rows, "p99_ms"),
            "median_process_rss_kb": median(
                rows, "server_process_rss_kb"
            ),
            "median_container_current_kb": median(
                rows, "container_current_kb"
            ),
            "max_container_peak_kb": max(
                int(row["container_peak_kb"]) for row in rows
            ),
            "total_failed_requests": sum(
                int(row["failed_requests"]) for row in rows
            ),
            "total_unattempted_requests": sum(
                int(row["unattempted_requests"]) for row in rows
            ),
        }

        writer.writerow(output)

print(f"Summary written to {destination}")

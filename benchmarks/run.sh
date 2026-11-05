#!/usr/bin/env bash
# Run the benchmarks and write benchmarks/report.md.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
python3 "$here/run.py" "$@"

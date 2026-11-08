#!/usr/bin/env bash
# Kode database benchmark runner.
#
# Examples
#   bash benchmarks/database/run.sh --all
#   bash benchmarks/database/run.sh --workload insert --rows 1000 --repetitions 3
#   bash benchmarks/database/run.sh --language kode,go,rust
#   bash benchmarks/database/run.sh --workload concurrency --workers 10,100
#   bash benchmarks/database/run.sh --quick
#   bash benchmarks/database/run.sh --reset
#   bash benchmarks/database/run.sh --down
#
# Every option is documented in benchmarks/database/README.md and by
# `bash benchmarks/database/run.sh --help`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

command -v docker >/dev/null 2>&1 || { echo "ERROR: docker is required" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "ERROR: python3 is required" >&2; exit 1; }
docker info >/dev/null 2>&1 || { echo "ERROR: cannot talk to the Docker daemon" >&2; exit 1; }

exec python3 "$ROOT/benchmarks/database/harness/runner.py" "$@"

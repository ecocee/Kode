#!/usr/bin/env bash
# ============================================================
# ADR 0044 evidence: the same text built with `+` and with StringBuilder.
#
#   ./benchmarks/string_build/run.sh
#
# Prints the table that `benchmarks/run.sh` puts in report.md as the
# "String building (WP-09)" section. Both programs print the byte length of
# the text they built, so a wrong result is visible next to the time.
# ============================================================
set -euo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
python3 "$here/../run.py" --string-build "$@"

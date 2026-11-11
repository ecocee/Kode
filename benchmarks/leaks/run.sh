#!/usr/bin/env bash
# ============================================================
# ADR 0038 evidence: the cycle leak, measured on the current compiler.
#
# Runs three small programs with KODE_LEAK_CHECK=1 and checks that the observed
# state is still the one benchmarks/leaks/report.md records:
#
#   nocycle.kode  back-edge cleared by hand  -> exit 0, 0 leaked objects
#   cycle.kode    two objects, strong cycle  -> exit 3, 2 leaked objects
#   ring.kode     10,000 objects, strong ring -> exit 3, 10,000 leaked objects
#
# These must NOT live in tests/run-pass: they are supposed to fail the leak
# check until the language changes. If a number moves, update report.md in the
# same commit — that is the whole point of this script.
#
#   ./benchmarks/leaks/run.sh
#   KODE=./target/release/kode ./benchmarks/leaks/run.sh
# ============================================================
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

KODE="${KODE:-./target/debug/kode}"
[[ -x "$KODE" ]] || { echo "ERROR: no compiler at $KODE (build it first)" >&2; exit 2; }

fail=0
printf 'program\texpected_exit\texit\tleaked_objects\texpected_leaked\toutput\n'

# check <file> <expected-exit> <expected-leaked>
check() {
    local file="$1" want_exit="$2" want_leak="$3"
    local out err rc leaked
    out="$(KODE_LEAK_CHECK=1 "$KODE" run "benchmarks/leaks/$file" 2>/tmp/leaks-stderr.$$)"
    rc=$?
    err="$(cat /tmp/leaks-stderr.$$)"; rm -f /tmp/leaks-stderr.$$
    leaked="$(sed -n 's/.*memory leak: \([0-9]*\) object.*/\1/p' <<< "$err")"
    leaked="${leaked:-0}"

    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$file" "$want_exit" "$rc" "$leaked" "$want_leak" \
        "$(tr '\n' ' ' <<< "$out" | sed 's/ $//')"

    if [[ "$rc" != "$want_exit" || "$leaked" != "$want_leak" ]]; then
        echo "  MISMATCH: $file exited $rc (wanted $want_exit) with $leaked leaked objects (wanted $want_leak)" >&2
        echo "  stderr: $err" >&2
        fail=1
    fi
}

check nocycle.kode 0 0
check cycle.kode 3 2
check ring.kode 3 10000

if [[ "$fail" -ne 0 ]]; then
    echo
    echo "The leak behaviour changed. Update benchmarks/leaks/report.md (and ADR 0038)" >&2
    echo "if that is intended, then commit both together." >&2
    exit 1
fi

echo
echo "OK: strong cycles still leak, exactly as ADR 0038 records."

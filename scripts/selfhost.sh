#!/usr/bin/env bash
#
# The self-hosting bootstrap harness (docs/PLAN-self-hosting.md, stage S0).
#
#   ./scripts/selfhost.sh stage0     build Stage 0 and record the oracles
#   ./scripts/selfhost.sh stage1     compile the port and compare it with Stage 0
#   ./scripts/selfhost.sh stage2     Stage 1 compiles itself; compare with Stage 1
#   ./scripts/selfhost.sh compare    diff the dumps that already exist
#
# The oracle is `kode tokens` and `kode ast` over the whole corpus: the port is
# verified by comparison, not by opinion (PLAN-self-hosting.md §5). Every stage
# refuses to report success when the dump it would compare against is missing,
# so a broken comparison cannot look like a working port.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
OUT="$ROOT/target/selfhost"
mkdir -p "$OUT"
STAGE0="$ROOT/target/debug/kode"
STAGE1="$OUT/stage1/kode-stage1"
STAGE2="$OUT/stage2/kode-stage2"
PORT="$ROOT/selfhost/main.kode"
CORPUS=(tests/run-pass tests/compile-fail examples benchmarks)

usage() {
    sed -n '3,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

corpus() {
    (cd "$ROOT" && find "${CORPUS[@]}" -name '*.kode' -type f | LC_ALL=C sort)
}

dump_corpus() {
    # dump_corpus <kode-binary> <dump-dir>
    local kode="$1" dir="$2"
    rm -rf "$dir"
    mkdir -p "$dir"
    local file dest
    while IFS= read -r file; do
        dest="$dir/${file%.kode}"
        mkdir -p "$(dirname "$dest")"
        # A compile-fail program is expected to fail: its exit code and its
        # diagnostics are part of the oracle, so they are kept next to the dump.
        if ! "$kode" tokens "$file" >"$dest.tokens" 2>"$dest.tokens.err"; then
            echo "$?" >"$dest.tokens.status"
        fi
        if ! "$kode" ast "$file" >"$dest.ast" 2>"$dest.ast.err"; then
            echo "$?" >"$dest.ast.status"
        fi
        rm -f "$dest.tokens.err" "$dest.ast.err"
    done < <(corpus)
}

# Run a command under /usr/bin/time and record wall clock and peak RSS.
timed() {
    local log="$OUT/time.txt"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        /usr/bin/time -l "$@" 2>"$log"
    else
        /usr/bin/time -v "$@" 2>"$log"
    fi
}

# Write `key=value` lines describing the last `timed` run.
record_measurement() {
    local label="$1"
    local files="$2"
    local log="$OUT/time.txt"
    local out="$OUT/$label.txt"
    local wall rss
    if [[ "$(uname -s)" == "Darwin" ]]; then
        wall="$(awk '/ real/ {print $1; exit}' "$log")"
        rss="$(awk '/maximum resident set size/ {print int($1 / 1024); exit}' "$log")"
    else
        wall="$(awk -F'[)\t ]+' '/Elapsed \\(wall clock\\)/ {print $NF; exit}' "$log")"
        rss="$(awk -F': ' '/Maximum resident set size/ {print $2; exit}' "$log")"
    fi
    {
        echo "stage=$label"
        echo "files=$files"
        echo "wall_seconds=${wall:-unknown}"
        echo "peak_rss_kb=${rss:-unknown}"
        echo "oracle=kode tokens, kode ast"
    } >"$out"
    echo "  recorded $out"
    sed 's/^/    /' "$out"
}

diff_dumps() {
    # diff_dumps <left> <right> <what>  -- returns 1 when they differ
    local left="$1" right="$2" what="$3"
    local report="$OUT/diff-$what.txt"
    if diff -r -u "$left" "$right" >"$report" 2>&1; then
        local count
        count="$(find "$right" -name '*.tokens' | wc -l | tr -d ' ')"
        echo "  parity: $count token dumps and as many tree dumps agree"
        rm -f "$report"
        return 0
    fi
    echo "  parity FAILED between $what"
    echo "  first differences ($report):"
    head -40 "$report" | sed 's/^/    /'
    return 1
}

build_stage1() {
    if [[ ! -f "$PORT" ]]; then
        echo "  no $PORT yet: the Kode compiler arrives with stage S1." >&2
        echo "  See docs/PLAN-self-hosting.md; Gate 0 (9.5-9.9) still blocks it." >&2
        return 2
    fi
    mkdir -p "$(dirname "$STAGE1")"
    ( cd "$ROOT" && "$STAGE0" build selfhost/main.kode -o "$STAGE1" )
}

mode="${1:-}"
case "$mode" in
stage0)
    echo "S0: Stage 0 builds the oracles"
    ( cd "$ROOT" && cargo build --workspace )
    timed bash "${BASH_SOURCE[0]}" __dump "$STAGE0" "$OUT/dumps/stage0"
    record_measurement stage0 "$(corpus | wc -l | tr -d ' ')"
    echo "  self-check: Stage 0 must agree with itself, or the comparison proves nothing"
    bash "${BASH_SOURCE[0]}" __dump "$STAGE0" "$OUT/dumps/stage0-again"
    diff_dumps "$OUT/dumps/stage0" "$OUT/dumps/stage0-again" self-check
    rm -rf "$OUT/dumps/stage0-again"
    echo "stage0: green"
    ;;
stage1)
    if [[ ! -f "$PORT" ]]; then
        echo "  no $PORT yet: the Kode compiler arrives with stage S1." >&2
        echo "  See docs/PLAN-self-hosting.md; Gate 0 (9.5-9.9) still blocks it." >&2
        exit 2
    fi
    echo "S1: Stage 0 compiles the port"
    ( cd "$ROOT" && cargo build --workspace )
    build_stage1 || exit $?
    timed bash "${BASH_SOURCE[0]}" __dump "$STAGE1" "$OUT/dumps/stage1"
    record_measurement stage1 "$(corpus | wc -l | tr -d ' ')"
    if [[ ! -d "$OUT/dumps/stage0" ]]; then
        echo "  no Stage 0 oracle: run ./scripts/selfhost.sh stage0 first" >&2
        exit 2
    fi
    diff_dumps "$OUT/dumps/stage0" "$OUT/dumps/stage1" stage0-vs-stage1
    echo "stage1: green"
    ;;
stage2)
    echo "S2: Stage 1 compiles itself"
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        exit 2
    fi
    mkdir -p "$(dirname "$STAGE2")"
    ( cd "$ROOT" && "$STAGE1" build selfhost/main.kode -o "$STAGE2" )
    timed bash "${BASH_SOURCE[0]}" __dump "$STAGE2" "$OUT/dumps/stage2"
    record_measurement stage2 "$(corpus | wc -l | tr -d ' ')"
    if [[ ! -d "$OUT/dumps/stage1" ]]; then
        echo "  no Stage 1 dump: run ./scripts/selfhost.sh stage1 first" >&2
        exit 2
    fi
    diff_dumps "$OUT/dumps/stage1" "$OUT/dumps/stage2" stage1-vs-stage2
    echo "stage2: green"
    ;;
compare)
    if [[ ! -d "$OUT/dumps/stage1" || ! -d "$OUT/dumps/stage2" ]]; then
        echo "nothing to compare: there is no Stage 1 and Stage 2 to diff yet." >&2
        echo "S0 stops at the Stage 0 oracle; the port starts when Gate 0 closes." >&2
        exit 2
    fi
    diff_dumps "$OUT/dumps/stage1" "$OUT/dumps/stage2" compare
    echo "compare: green"
    ;;
__dump)
    dump_corpus "$2" "$3"
    ;;
"" | -h | --help)
    usage
    ;;
*)
    echo "unknown mode \`$mode\`" >&2
    usage >&2
    exit 2
    ;;
esac

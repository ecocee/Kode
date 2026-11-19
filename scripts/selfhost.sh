#!/usr/bin/env bash
#
# The self-hosting bootstrap harness (docs/PLAN-self-hosting.md, stage S0).
#
#   ./scripts/selfhost.sh stage0     build Stage 0 and record the oracles
#   ./scripts/selfhost.sh stage1     compile the port and compare it with Stage 0
#   ./scripts/selfhost.sh stage2     Stage 1 compiles itself; compare with Stage 1
#   ./scripts/selfhost.sh compare    diff the dumps that already exist
#   ./scripts/selfhost.sh parse      the S2 subset and the port's own source, case by case
#   ./scripts/selfhost.sh check      the S3 checker: its codes against the oracle, over tests/{compile-fail,run-pass}
#   ./scripts/selfhost.sh gates      every work packet: target declared vs today
#   ./scripts/selfhost.sh readiness  the readiness level reached, and what is next
#   ./scripts/selfhost.sh all        Stage 0, Stage 1, then the work packets
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
# `selfhost/spec` holds the cases the port needs but the language suite does not
# cover: every escape, a 4-byte character, a hole with a brace in a string.
CORPUS=(tests/run-pass tests/compile-fail examples benchmarks selfhost/spec)

usage() {
    grep '^#   \./scripts/selfhost.sh' "${BASH_SOURCE[0]}" | sed 's/^# *//'
}

GATES_DIR="$ROOT/selfhost/spec/gates"
# The order PLAN-self-hosting.md §6 builds in: what unblocks the most, first.
# `readiness` names the next packet from this list, not from file order.
PACKET_ORDER=(WP03 WP04 WP05 WP02 WP06 WP08 WP09 WP10)
# Red on purpose: the plan defers these, so `next` must not name them.
DEFERRED=(WP07 WP08b)

# Compile every work-packet probe and compare what Stage 0 does today with the
# target the probe declares. Writes `target/selfhost/gates.txt` (packet, probe,
# target, observed, state) and prints the table. The table is the build's
# to-do list: `blocked` rows are what still needs code (PLAN-self-hosting §3).
check_gates() {
    if [[ ! -d "$GATES_DIR" ]]; then
        echo "  no work-packet probes in $GATES_DIR" >&2
        return 2
    fi
    local out="$OUT/gates.txt" probe packet target observed state met=0 blocked=0
    : >"$out"
    for probe in "$GATES_DIR"/*.kode; do
        packet="$(sed -n 's|^// work packet: \([^ ]*\).*|\1|p' "$probe" | head -1)"
        target="$(sed -n 's|^// target: \(.*\)|\1|p' "$probe" | head -1)"
        if "$STAGE0" check "$probe" >"$OUT/gate.log" 2>&1; then
            observed=ok
        else
            observed="$(grep -o 'E0[0-9][0-9][0-9]' "$OUT/gate.log" | head -1)"
            observed="${observed:-error}"
        fi
        if [[ "$observed" == "$target" ]]; then
            state=met
            met=$((met + 1))
        else
            state=blocked
            blocked=$((blocked + 1))
        fi
        printf '%s\t%s\t%s\t%s\t%s\n' "$packet" "$(basename "$probe" .kode)" \
            "$target" "$observed" "$state" >>"$out"
        printf '  %-6s %-34s target %-6s today %-6s %s\n' \
            "$packet" "$(basename "$probe" .kode)" "$target" "$observed" "$state"
    done
    rm -f "$OUT/gate.log"
    echo "  work packets: $met met, $blocked blocked ($out)"
}

# Compare `kode ast` with the port's parser: trees in `selfhost/spec/parse`, trees
# plus diagnostics in `selfhost/spec/parse-errors` (where stderr is the point), and
# the whole `tests/compile-fail` corpus. The corpus is there to catch drift: those
# programs are the oracle's own list of what a parser must refuse, so a recovery
# that starts reporting an extra or a different code changes the count here, not
# months later in a slice. All three are compared byte for byte, stdout and stderr.
# A case is the acceptance test for the slice it came with; this refuses to report
# success when there are no cases at all.
check_parse() {
    local dirs=("$ROOT/selfhost/spec/parse" "$ROOT/selfhost/spec/parse-errors" "$ROOT/tests/compile-fail")
    local out="$OUT/parse" total=0 bad=0 dir file
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    rm -rf "$out"
    mkdir -p "$out"
    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            continue
        fi
        while IFS= read -r file; do
            total=$((total + 1))
            "$STAGE0" ast "$file" >"$out/expected.txt" 2>"$out/expected.err"
            "$STAGE1" ast "$file" >"$out/actual.txt" 2>"$out/actual.err"
            if ! diff -u "$out/expected.txt" "$out/actual.txt" >"$out/diff.txt"; then
                bad=$((bad + 1))
                echo "  $(basename "$file"): tree differs"
                head -20 "$out/diff.txt" | sed 's/^/    /'
            elif ! diff -u "$out/expected.err" "$out/actual.err" >"$out/diff.txt"; then
                bad=$((bad + 1))
                echo "  $(basename "$file"): diagnostics differ"
                head -20 "$out/diff.txt" | sed 's/^/    /'
            fi
        done < <(find "$dir" -name '*.kode' -type f | LC_ALL=C sort)
    done
    if [[ "$total" -eq 0 ]]; then
        echo "  no parse cases in ${dirs[*]}" >&2
        return 2
    fi
    echo "  parse cases: $((total - bad))/$total trees and diagnostics byte-identical"
    # Then the port's own source: the strongest case there is, because every file in
    # `selfhost/` is a program the compiler already builds.
    local own=0 ownbad=0
    while IFS= read -r file; do
        own=$((own + 1))
        "$STAGE0" ast "$file" >"$out/expected.txt" 2>"$out/expected.err"
        "$STAGE1" ast "$file" >"$out/actual.txt" 2>"$out/actual.err"
        if ! diff -u "$out/expected.txt" "$out/actual.txt" >"$out/diff.txt"; then
            ownbad=$((ownbad + 1))
            echo "  $(basename "$file"): own-source tree differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        fi
    done < <(find "$ROOT/selfhost" -maxdepth 1 -name '*.kode' -type f | LC_ALL=C sort)
    echo "  self-parse: $((own - ownbad))/$own of the port's own files tree-identical"
    [[ "$bad" -eq 0 && "$ownbad" -eq 0 ]]
}

# The level reached (PLAN-self-hosting.md §2), from the artefacts on disk and the
# packet table. This reports; it never pretends a stage exists.
# The S3 gate: the port's code set must be the oracle's, and a run-pass program must
# report nothing. The expectation is the oracle itself, not the `.err` file: a `.err`
# names the codes the Rust conformance test looks for and may leave out the follow-on
# ones (`E0102` after a parser error, `E0201` after a failed call), so comparing with
# it would fail a port that agrees with Stage 0. Codes are compared as sets; message,
# span and order (the JSON form) are the next step of S3, not this one, so a family
# lands when this count moves.
check_sema() {
    local total=0 pass=0 file expected got name
    local log="$OUT/check.log" oracle="$OUT/oracle.log" mismatches=""
    if [[ ! -x "$STAGE0" ]]; then
        echo "  no Stage 0 oracle: run ./scripts/selfhost.sh stage0 first" >&2
        return 2
    fi
    for file in "$ROOT"/tests/compile-fail/*.kode; do
        total=$((total + 1))
        name="$(basename "${file%.kode}")"
        "$STAGE0" check "$file" >/dev/null 2>"$oracle" || true
        expected="$(grep -o '[EW]0[0-9][0-9][0-9]' "$oracle" 2>/dev/null | sort -u | tr '\n' ' ' || true)"
        "$STAGE1" check "$file" >/dev/null 2>"$log" || true
        got="$(grep -o '[EW]0[0-9][0-9][0-9]' "$log" 2>/dev/null | sort -u | tr '\n' ' ' || true)"
        if [[ "$got" == "$expected" ]]; then
            pass=$((pass + 1))
        else
            mismatches+="    $name: wanted [$expected] got [$got]"$'\n'
        fi
    done
    for file in "$ROOT"/tests/run-pass/*.kode; do
        total=$((total + 1))
        name="$(basename "${file%.kode}")"
        if "$STAGE1" check "$file" >/dev/null 2>"$log"; then
            pass=$((pass + 1))
        else
            got="$(grep -o '[EW]0[0-9][0-9][0-9]' "$log" 2>/dev/null | sort -u | tr '\n' ' ' || true)"
            mismatches+="    $name: wanted [] got [$got]"$'\n'
        fi
    done
    echo "  check cases: $pass/$total report the expected code set"
    if [[ "$pass" -ne "$total" ]]; then
        echo "  first mismatches (S3 is in progress, one family per commit):"
        printf '%s' "$mismatches" | head -n 8
    fi
    # The port's own sources are the one program big enough to need the module
    # graph, so they are checked too: a name the checker cannot resolve in
    # `selfhost/` is a bug in the checker, not a missing family.
    local self_total=0 self_pass=0 self_bad=""
    for file in "$ROOT"/selfhost/*.kode; do
        self_total=$((self_total + 1))
        if "$STAGE1" check "$file" >/dev/null 2>"$log"; then
            self_pass=$((self_pass + 1))
        else
            self_bad+="    $(basename "$file"): $(head -n 1 "$log")"$'\n'
        fi
    done
    echo "  self-check: $self_pass/$self_total of the port's own files are clean"
    if [[ "$self_pass" -ne "$self_total" ]]; then
        printf '%s' "$self_bad" | head -n 8
    fi
    printf 'check_pass=%s\ncheck_total=%s\nself_pass=%s\nself_total=%s\n' \
        "$pass" "$total" "$self_pass" "$self_total" >"$OUT/check.txt"
    return 0
}

readiness() {
    echo "Self-host readiness (docs/PLAN-self-hosting.md)"
    if [[ -d "$OUT/dumps/stage0" ]]; then
        echo "  R0 oracle: present"
    else
        echo "  R0 oracle: missing - run ./scripts/selfhost.sh stage0"
    fi
    if [[ -d "$OUT/dumps/stage1" ]]; then
        echo "  R1 lexer: present and compared"
    else
        echo "  R1 lexer: missing - run ./scripts/selfhost.sh stage1"
    fi
    check_gates || true
    local blocked next deferred onpath packet
    blocked="$(awk -F'\t' '$5 == "blocked"' "$OUT/gates.txt" | wc -l | tr -d ' ')"
    # A deferred packet is red on purpose, so it is counted apart from the
    # packets the port is actually waiting on.
    deferred=0
    for packet in "${DEFERRED[@]}"; do
        if awk -F'\t' -v p="$packet" '$1 == p && $5 == "blocked" {found = 1} END {exit !found}' \
            "$OUT/gates.txt"; then
            deferred=$((deferred + 1))
        fi
    done
    onpath=$((blocked - deferred))
    if [[ "$onpath" -eq 0 ]]; then
        echo "  R2 language: every packet on the path is met, $deferred deferred (${DEFERRED[*]})"
    else
        echo "  R2 language: $onpath work packet(s) blocked on the path, $deferred deferred"
    fi
    if [[ -d "$OUT/dumps/stage2" ]]; then
        echo "  R3 parser: present"
    elif [[ -d "$ROOT/selfhost/spec/parse" ]]; then
        echo "  R3 parser: subset green (./scripts/selfhost.sh parse)"
    else
        echo "  R3 parser: not started"
    fi
    if [[ -f "$OUT/check.txt" ]]; then
        echo "  R4 checker: $(sed -n 's/^check_pass=//p' "$OUT/check.txt")/$(sed -n 's/^check_total=//p' "$OUT/check.txt") cases, name resolution and modules"
    else
        echo "  R4 checker: not run (./scripts/selfhost.sh check)"
    fi
    {
        echo "gates_blocked=$blocked"
        echo "gates_blocked_on_path=$onpath"
        echo "gates_deferred=$deferred"
        awk -F'\t' '$5 == "blocked" {printf "blocked=%s %s (target %s, today %s)\n", $1, $2, $3, $4}' \
            "$OUT/gates.txt"
    } >"$OUT/readiness.txt"
    next=""
    for packet in "${PACKET_ORDER[@]}"; do
        if [[ " ${DEFERRED[*]} " == *" $packet "* ]]; then
            continue
        fi
        if awk -F'\t' -v p="$packet" '$1 == p && $5 == "blocked" {found = 1} END {exit !found}' \
            "$OUT/gates.txt"; then
            next="$packet"
            break
        fi
    done
    # A packet added to the probes but not yet to the order still gets named,
    # unless the plan defers it.
    if [[ -z "$next" ]]; then
        next="$(awk -F'\t' -v skip=" ${DEFERRED[*]} " \
            'index(skip, " " $1 " ") == 0 && $5 == "blocked" {print $1; exit}' "$OUT/gates.txt")"
    fi
    if [[ -n "$next" ]]; then
        echo "  next: $next (docs/PLAN-self-hosting.md §3)"
    else
        echo "  next: S3 checker (R4) - every work packet on the path is met"
        echo "  WP-12 process streaming stays until S5 (docs/PLAN-self-hosting.md §3)"
    fi
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
    # diff_dumps <left> <right> <what> [tokens|all] -- returns 1 when they differ
    #
    # Stage S1 ships the lexer only, so `tokens` compares token dumps and leaves
    # the tree dumps to S2. `all` (the default) compares both.
    local left="$1" right="$2" what="$3" only="${4:-all}"
    local report="$OUT/diff-$what.txt"
    local excludes=()
    if [[ "$only" == "tokens" ]]; then
        excludes=(-x '*.ast' -x '*.ast.status')
    fi
    if diff -r -u ${excludes[@]+"${excludes[@]}"} "$left" "$right" >"$report" 2>&1; then
        local count
        count="$(find "$right" -name '*.tokens' | wc -l | tr -d ' ')"
        if [[ "$only" == "tokens" ]]; then
            echo "  parity: $count token dumps agree (tree dumps arrive with S2)"
        else
            echo "  parity: $count token dumps and as many tree dumps agree"
        fi
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
    diff_dumps "$OUT/dumps/stage0" "$OUT/dumps/stage1" stage0-vs-stage1 tokens
    echo "stage1: green (lexer)"
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
gates)
    echo "Self-host work packets (docs/PLAN-self-hosting.md §3)"
    [[ -x "$STAGE0" ]] || ( cd "$ROOT" && cargo build --workspace )
    check_gates
    ;;
readiness)
    [[ -x "$STAGE0" ]] || ( cd "$ROOT" && cargo build --workspace )
    readiness
    ;;
parse)
    echo "S2: the port's parser against the oracle (the parse subset)"
    check_parse || exit $?
    ;;
check)
    echo "S3: the port's checker against the expected codes"
    check_sema || exit $?
    ;;
all)
    bash "${BASH_SOURCE[0]}" stage0
    bash "${BASH_SOURCE[0]}" stage1
    bash "${BASH_SOURCE[0]}" parse
    bash "${BASH_SOURCE[0]}" check
    bash "${BASH_SOURCE[0]}" gates
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

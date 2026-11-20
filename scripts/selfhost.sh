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
#   ./scripts/selfhost.sh mir        the S4 MIR dump against `kode mir`, over selfhost/spec/run
#   ./scripts/selfhost.sh run        the S4 interpreter's output against every .out
#   ./scripts/selfhost.sh own        the owning interpreter: shadow counts, silent on the
#                                    stress cases and loud on a lowering that drops a retain
#   ./scripts/selfhost.sh report     the walk lists every unsupported construct, not just the first
#   ./scripts/selfhost.sh corpus     the same two checks over tests/run-pass, with the
#                                    out-of-scope programs recorded and re-asserted
#   ./scripts/selfhost.sh ledger     regenerate selfhost/spec/corpus-out-of-scope.txt
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
# `tests/run-pass` programs the S4 walk cannot lower yet, with the port's reason. The
# corpus gate keeps this list honest in both directions.
CORPUS_OUT_OF_SCOPE="$ROOT/selfhost/spec/corpus-out-of-scope.txt"

# `timeout` is not in a stock macOS userland, so use it when it is there and run
# without it otherwise (a hang then shows up as a stuck job, not a wrong result).
with_timeout() {
    if command -v timeout >/dev/null 2>&1; then
        timeout 30 "$@"
    else
        "$@"
    fi
}

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
# The oracle prints `checked in 0.38 ms` on success. That line is a wall-clock
# measurement, not diagnostic text, so it is dropped before the two reports are compared:
# a clean program's report is otherwise identical.
strip_timing() {
    sed -E '/^checked in [0-9.]+ ms$/d' "$1"
}

# The S3 gate: the port's code set must be the oracle's, and a run-pass program must
# report nothing. The expectation is the oracle itself, not the `.err` file: a `.err`
# names the codes the Rust conformance test looks for and may leave out the follow-on
# ones (`E0102` after a parser error, `E0201` after a failed call), so comparing with
# it would fail a port that agrees with Stage 0. Codes are compared as sets, so a
# family lands when that count moves; the rendered text is gated too ("check text"),
# because the port is now expected to print the oracle's message, span, carets and hint
# byte for byte, in the oracle's order, with the wall-clock line dropped.
check_sema() {
    local total=0 pass=0 file expected got name text_ok=0 rc=0
    local log="$OUT/check.log" oracle="$OUT/oracle.log" mismatches="" text_bad=""
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
        if diff -q <(strip_timing "$oracle") <(strip_timing "$log") >/dev/null 2>&1; then
            text_ok=$((text_ok + 1))
        else
            text_bad+="    $name"$'\n'
        fi
        if [[ "$got" == "$expected" ]]; then
            pass=$((pass + 1))
        else
            mismatches+="    $name: wanted [$expected] got [$got]"$'\n'
        fi
    done
    for file in "$ROOT"/tests/run-pass/*.kode; do
        total=$((total + 1))
        name="$(basename "${file%.kode}")"
        "$STAGE0" check "$file" >/dev/null 2>"$oracle" || true
        rc=0
        "$STAGE1" check "$file" >/dev/null 2>"$log" || rc=$?
        if diff -q <(strip_timing "$oracle") <(strip_timing "$log") >/dev/null 2>&1; then
            text_ok=$((text_ok + 1))
        else
            text_bad+="    $name (run-pass)"$'\n'
        fi
        if [[ "$rc" -eq 0 ]]; then
            pass=$((pass + 1))
        else
            got="$(grep -o '[EW]0[0-9][0-9][0-9]' "$log" 2>/dev/null | sort -u | tr '\n' ' ' || true)"
            mismatches+="    $name: wanted [] got [$got]"$'\n'
        fi
    done
    echo "  check cases: $pass/$total report the expected code set"
    echo "  check text: $text_ok/$total reports byte-identical"
    if [[ "$pass" -ne "$total" ]]; then
        echo "  cases whose code set differs:"
        printf '%s' "$mismatches" | head -n 8
    fi
    if [[ "$text_ok" -ne "$total" ]]; then
        echo "  cases whose report differs (message, span, carets, hint or order):"
        printf '%s' "$text_bad" | head -n 8
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
    printf 'check_pass=%s\ncheck_total=%s\ntext_pass=%s\nself_pass=%s\nself_total=%s\n' \
        "$pass" "$total" "$text_ok" "$self_pass" "$self_total" >"$OUT/check.txt"
    [[ "$pass" -eq "$total" && "$text_ok" -eq "$total" && "$self_pass" -eq "$self_total" ]]
}

# The S4 slice 1 gate: `kode mir` on the programs a straight line of `print` calls
# over literals builds. Compared byte for byte, because slice 1 emits no temporaries
# — the one thing PLAN-self-hosting §5 lets differ is numbering, and there is none.
check_mir() {
    local dir="$ROOT/selfhost/spec/run" out="$OUT/mir" total=0 bad=0 file
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    rm -rf "$out"
    mkdir -p "$out"
    for file in "$dir"/*.kode; do
        total=$((total + 1))
        "$STAGE0" mir "$file" >"$out/expected.txt" 2>"$out/expected.err"
        "$STAGE1" mir "$file" >"$out/actual.txt" 2>"$out/actual.err"
        if ! diff -u "$out/expected.txt" "$out/actual.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): dump differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        fi
    done
    if [[ "$total" -eq 0 ]]; then
        echo "  no S4 cases in $dir" >&2
        return 2
    fi
    echo "  mir cases: $((total - bad))/$total dumps byte-identical"
    [[ "$bad" -eq 0 ]]
}

# The owning interpreter (S4): the interpreter keeps shadow reference counts, so a
# release that had nothing to release, a use after release, or a count still standing at
# the end fails the run. The cases in `selfhost/spec/own` are the stress paths, and the
# gate also builds a mutant port whose every `retain` became a `release` — a check that
# never fails would pass the first half while proving nothing.
check_own() {
    local dir="$ROOT/selfhost/spec/own" out="$OUT/own" total=0 bad=0 file expected
    local oracle_rc=0 port_rc=0 caught=0 mut="$OUT/own/mutation"
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    rm -rf "$out"
    mkdir -p "$out"
    for file in "$dir"/*.kode; do
        total=$((total + 1))
        expected="${file%.kode}.out"
        oracle_rc=0
        port_rc=0
        "$STAGE0" run "$file" >"$out/oracle.txt" 2>/dev/null || oracle_rc=$?
        "$STAGE1" run "$file" >"$out/actual.txt" 2>"$out/actual.err" || port_rc=$?
        if [[ "$oracle_rc" -ne 0 ]]; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): kode run exited $oracle_rc, not 0"
        elif ! diff -u "$expected" "$out/oracle.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): the fixture's .out is not what kode run prints"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        elif [[ "$port_rc" -ne 0 ]]; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): the ownership check refused a correct program (exit $port_rc)"
            head -3 "$out/actual.err" | sed 's/^/    /'
        elif ! diff -u "$expected" "$out/actual.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): the interpreter's output differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        fi
    done
    # The mutation: a copy of the port whose lowers turn every retain into a release.
    mkdir -p "$mut/src"
    cp "$ROOT"/selfhost/*.kode "$mut/src/"
    python3 - "$mut/src/lower_stmt.kode" <<'MUTATE'
import sys
path = sys.argv[1]
text = open(path).read()
text = text.replace('Instr(kind: "retain"', 'Instr(kind: "release"')
open(path, "w").write(text)
MUTATE
    if ! "$STAGE0" build "$mut/src/main.kode" -o "$mut/kode-mutant" >"$mut/build.txt" 2>&1; then
        echo "  the mutant port did not build: the mutation no longer applies" >&2
        head -5 "$mut/build.txt" | sed 's/^/    /' >&2
        return 2
    fi
    for file in "$ROOT"/selfhost/spec/run/*.kode "$dir"/*.kode; do
        port_rc=0
        "$mut/kode-mutant" run "$file" >/dev/null 2>"$mut/err.txt" || port_rc=$?
        if [[ "$port_rc" -ne 0 ]] && grep -q "ownership:" "$mut/err.txt"; then
            caught=$((caught + 1))
        fi
    done
    if [[ "$total" -eq 0 ]]; then
        echo "  no ownership cases in $dir" >&2
        return 2
    fi
    if [[ "$caught" -eq 0 ]]; then
        echo "  the mutation (retain as release) was not caught: the check never fires" >&2
        return 1
    fi
    echo "  own cases: $((total - bad))/$total silent on correct programs, mutation caught on $caught"
    [[ "$bad" -eq 0 ]]
}

# The S4 slice 1 interpreter: the port's output has to be the fixture's `.out`, and
# the fixture has to be the oracle's own `kode run` output — so a wrong `.out` cannot
# hide a wrong interpreter.
check_run() {
    local dir="$ROOT/selfhost/spec/run" out="$OUT/run" total=0 bad=0 file expected
    local oracle_rc=0 port_rc=0
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    rm -rf "$out"
    mkdir -p "$out"
    for file in "$dir"/*.kode; do
        total=$((total + 1))
        expected="${file%.kode}.out"
        oracle_rc=0
        port_rc=0
        "$STAGE0" run "$file" >"$out/oracle.txt" 2>/dev/null || oracle_rc=$?
        "$STAGE1" run "$file" >"$out/actual.txt" 2>"$out/actual.err" || port_rc=$?
        if [[ "$oracle_rc" -ne 0 ]]; then
            # A run-pass fixture has to exit 0, or the .out proves nothing about a
            # successful run.
            bad=$((bad + 1))
            echo "  $(basename "$file"): kode run exited $oracle_rc, not 0"
        elif ! diff -u "$expected" "$out/oracle.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): the fixture's .out is not what kode run prints"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        elif [[ "$port_rc" -ne "$oracle_rc" ]]; then
            # An interpreter that prints the right text and then exits differently is
            # still wrong: the gate compares the status too, not only stdout.
            bad=$((bad + 1))
            echo "  $(basename "$file"): exit $port_rc where the oracle exits $oracle_rc"
        elif ! diff -u "$expected" "$out/actual.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $(basename "$file"): the interpreter's output differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        fi
    done
    if [[ "$total" -eq 0 ]]; then
        echo "  no S4 cases in $dir" >&2
        return 2
    fi
    echo "  run cases: $((total - bad))/$total outputs byte-identical and exit codes equal"
    [[ "$bad" -eq 0 ]]
}


# The generated ledger: a header, then `<program>\t<reason>` once per unsupported
# construct, in the order the walk meets them. The port produces it, so the list
# cannot drift from what the port actually does.
generate_ledger() {
    cat <<'HEADER'
# `tests/run-pass` programs the S4 walk cannot lower yet: one line per program and
# unsupported construct, `<program>\t<reason>`, in the order the walk meets them.
#
# Generated from the port, never edited by hand:
#
#     ./scripts/selfhost.sh ledger
#
# `corpus` regenerates it and fails when the file differs, so a program cannot
# quietly leave the list (it lowers now) or join it (it stopped lowering) without
# the commit saying so. `report` is the per-program view behind it.
HEADER
    local file name reasons reason
    for file in "$ROOT"/tests/run-pass/*.kode; do
        name="$(basename "$file")"
        reasons="$(with_timeout "$STAGE1" report "$file" 2>/dev/null || true)"
        [[ -n "$reasons" ]] || continue
        while IFS= read -r reason; do
            printf '%s\t%s\n' "$name" "$reason"
        done <<<"$reasons"
    done
}

# `selfhost/spec/report/*.kode` next to a `.report`: `kode-stage1 report` must list
# exactly those constructs, in that order. The fixtures pin the walk's behaviour on
# a program that needs several things (not just the first) and on one whose refused
# declaration is then read (which is a consequence, not a second finding).
check_report() {
    local dir="$ROOT/selfhost/spec/report" total=0 bad=0 file name expected
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    for file in "$dir"/*.kode; do
        [[ -e "$file" ]] || continue
        total=$((total + 1))
        name="$(basename "$file")"
        expected="${file%.kode}.report"
        if [[ ! -f "$expected" ]]; then
            bad=$((bad + 1))
            echo "  $name: no .report next to it"
            continue
        fi
        if ! with_timeout "$STAGE1" report "$file" >"$OUT/report.txt" 2>"$OUT/report.err"; then
            bad=$((bad + 1))
            echo "  $name: report exited nonzero"
            sed 's/^/    /' "$OUT/report.err"
            continue
        fi
        if ! diff -u "$expected" "$OUT/report.txt" >"$OUT/report.diff"; then
            bad=$((bad + 1))
            echo "  $name: the report differs"
            head -20 "$OUT/report.diff" | sed 's/^/    /'
        fi
    done
    if [[ "$total" -eq 0 ]]; then
        echo "  no report cases in $dir" >&2
        return 2
    fi
    echo "  report cases: $((total - bad))/$total unsupported constructs listed exactly"
    [[ "$bad" -eq 0 ]]
}

# The same two comparisons as `mir` and `run`, over the real corpus in
# `tests/run-pass` instead of the hand-written fixtures. Nothing is skipped: a
# program the walk cannot lower is in `selfhost/spec/corpus-out-of-scope.txt` (which
# this gate regenerates and compares), and an in-scope program's dump, output and
# exit code must match the oracle's.
check_corpus() {
    local dir="$ROOT/tests/run-pass" out="$OUT/corpus" total=0 bad=0 listed=0 file
    local expected got oracle_rc port_rc name reasons
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        return 2
    fi
    rm -rf "$out"
    mkdir -p "$out"
    generate_ledger >"$out/ledger.txt"
    if ! diff -u "$CORPUS_OUT_OF_SCOPE" "$out/ledger.txt" >"$out/ledger.diff"; then
        bad=$((bad + 1))
        echo "  the out-of-scope ledger is stale: ./scripts/selfhost.sh ledger"
        head -20 "$out/ledger.diff" | sed 's/^/    /'
    fi
    for file in "$dir"/*.kode; do
        name="$(basename "$file")"
        reasons="$(awk -F'\t' -v n="$name" '$1 == n {print $2}' "$CORPUS_OUT_OF_SCOPE")"
        if [[ -n "$reasons" ]]; then
            listed=$((listed + 1))
            # Listed means it must really still refuse at the first blocker.
            got=0
            with_timeout "$STAGE1" mir "$file" >/dev/null 2>"$out/refusal.err" || got=$?
            if [[ "$got" -ne 2 ]]; then
                bad=$((bad + 1))
                echo "  $name: listed as out of scope, but mir exited $got instead of 2"
            fi
            continue
        fi
        total=$((total + 1))
        expected="${file%.kode}.out"
        if ! diff -u <(with_timeout "$STAGE0" mir "$file" 2>/dev/null) \
            <(with_timeout "$STAGE1" mir "$file" 2>/dev/null) >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $name: dump differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
            continue
        fi
        oracle_rc=0
        port_rc=0
        with_timeout "$STAGE0" run "$file" >"$out/oracle.txt" 2>/dev/null || oracle_rc=$?
        with_timeout "$STAGE1" run "$file" >"$out/actual.txt" 2>"$out/actual.err" || port_rc=$?
        if [[ "$oracle_rc" -ne 0 ]]; then
            bad=$((bad + 1))
            echo "  $name: kode run exited $oracle_rc, not 0"
        elif ! diff -u "$expected" "$out/oracle.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $name: the corpus .out is not what kode run prints"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        elif [[ "$port_rc" -ne "$oracle_rc" ]]; then
            bad=$((bad + 1))
            echo "  $name: exit $port_rc where the oracle exits $oracle_rc"
        elif ! diff -u "$expected" "$out/actual.txt" >"$out/diff.txt"; then
            bad=$((bad + 1))
            echo "  $name: the interpreter's output differs"
            head -20 "$out/diff.txt" | sed 's/^/    /'
        fi
    done
    if [[ "$total" -eq 0 && "$listed" -eq 0 ]]; then
        echo "  no corpus programs found in $dir" >&2
        return 2
    fi
    echo "  corpus cases: $((total - bad))/$total in scope, $listed programs recorded out of scope ($(awk -F'\t' 'NF == 2' "$CORPUS_OUT_OF_SCOPE" | wc -l | tr -d ' ') constructs)"
    [[ "$bad" -eq 0 ]]
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
mir)
    echo "S4: the port's MIR dump against the oracle's"
    check_mir || exit $?
    ;;
run)
    echo "S4: the port's interpreter against every .out"
    check_run || exit $?
    ;;
own)
    echo "S4: the owning interpreter's shadow reference counts"
    check_own || exit $?
    ;;
report)
    echo "S4: the walk's list of unsupported constructs, not just the first"
    check_report || exit $?
    ;;
corpus)
    echo "S4: the real corpus in tests/run-pass, with its out-of-scope list"
    check_corpus || exit $?
    ;;
ledger)
    if [[ ! -x "$STAGE1" ]]; then
        echo "  no Stage 1 compiler: run ./scripts/selfhost.sh stage1 first" >&2
        exit 2
    fi
    generate_ledger >"$CORPUS_OUT_OF_SCOPE"
    echo "ledger: $(awk -F'\t' 'NF == 2' "$CORPUS_OUT_OF_SCOPE" | wc -l | tr -d ' ') constructs in $(awk -F'\t' 'NF == 2 {print $1}' "$CORPUS_OUT_OF_SCOPE" | sort -u | wc -l | tr -d ' ') programs"
    ;;
all)
    bash "${BASH_SOURCE[0]}" stage0
    bash "${BASH_SOURCE[0]}" stage1
    bash "${BASH_SOURCE[0]}" parse
    bash "${BASH_SOURCE[0]}" check
    bash "${BASH_SOURCE[0]}" mir
    bash "${BASH_SOURCE[0]}" run
    bash "${BASH_SOURCE[0]}" own
    bash "${BASH_SOURCE[0]}" report
    bash "${BASH_SOURCE[0]}" corpus
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

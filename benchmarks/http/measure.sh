#!/usr/bin/env bash
# ============================================================
# measure.sh: resource measurement helpers for a running container.
#
# Source it (run.sh does) or use it standalone:
#   ./measure.sh snapshot <container>            one-shot CPU / memory / OOM state
#   ./measure.sh watch    <container> [seconds]  sample N seconds, print summary
#   ./measure.sh time     <command...>           wall-clock any command
#
# Reads cgroup v2 files inside the container (memory.current, memory.peak,
# cpu.stat). On cgroup v1 hosts the values come back as 0.
# Sampling uses `docker exec`, so it costs a few ms of the container's own
# CPU per sample; raise SAMPLE_INTERVAL if you test at 0.25 CPU.
# ============================================================

now_ms() {
    local t
    t="$(date +%s%3N 2>/dev/null || true)"
    if [[ "$t" =~ ^[0-9]+$ ]]; then
        echo "$t"
    else
        python3 -c 'import time; print(int(time.time()*1000))'
    fi
}

# cg_read <container> <cgroup-file>
cg_read() {
    docker exec "$1" cat "/sys/fs/cgroup/$2" 2>/dev/null || true
}

# Appends: "<ts_ms> <mem_bytes> <cpu_usec> <nr_throttled> <throttled_usec>"
measure_snapshot() {
    local container="$1" file="$2" out
    out="$(docker exec "$container" sh -c \
        'cat /sys/fs/cgroup/memory.current; awk "/^(usage_usec|nr_throttled|throttled_usec) /{print \$2}" /sys/fs/cgroup/cpu.stat' \
        2>/dev/null)" || return 0
    # shellcheck disable=SC2086
    echo "$(now_ms) $(echo $out)" >> "$file"
}

MEASURE_PID=""
MEASURE_CONTAINER=""
MEASURE_FILE=""

# measure_start <container> <samples-file> [interval-seconds]
measure_start() {
    MEASURE_CONTAINER="$1"
    MEASURE_FILE="$2"
    local interval="${3:-1}"

    : > "$MEASURE_FILE"
    measure_snapshot "$MEASURE_CONTAINER" "$MEASURE_FILE"   # exact starting counters

    (
        while :; do
            sleep "$interval"
            measure_snapshot "$MEASURE_CONTAINER" "$MEASURE_FILE"
        done
    ) &
    MEASURE_PID=$!
    disown "$MEASURE_PID" 2>/dev/null || true
}

measure_stop() {
    if [[ -n "${MEASURE_PID:-}" ]]; then
        kill "$MEASURE_PID" >/dev/null 2>&1 || true
        MEASURE_PID=""
        # exact ending counters, so even very short runs get a correct CPU delta
        measure_snapshot "$MEASURE_CONTAINER" "$MEASURE_FILE"
    fi
}

# measure_summary <samples-file> [wall-seconds]
# Prints key=value pairs. cpu_avg_pct: 100 = one full core.
measure_summary() {
    local file="$1" wall="${2:-}"

    if [[ ! -s "$file" ]]; then
        echo "cpu_seconds=0 cpu_avg_pct=0 cpu_max_pct=0 mem_avg_mb=0 mem_max_mb=0 throttled_periods=0 throttled_ms=0 samples=0"
        return 0
    fi

    sort -n "$file" | awk -v wall="$wall" '
    NR == 1 { t0 = $1; c0 = $3; n0 = $4; h0 = $5; pt = $1; pc = $3 }
    {
        mem = $2 / 1048576
        sum += mem; cnt++
        if (mem > mx) mx = mem
        if (NR > 1) {
            dt = ($1 - pt) * 1000                      # ms -> usec
            if (dt > 0) { p = ($3 - pc) / dt * 100; if (p > pm) pm = p }
        }
        pt = $1; pc = $3
        lt = $1; lc = $3; ln = $4; lh = $5
    }
    END {
        cpu_s = (lc - c0) / 1e6
        span  = (wall != "" && wall + 0 > 0) ? wall + 0 : (lt - t0) / 1000
        avg   = span > 0 ? cpu_s / span * 100 : 0
        if (pm < avg) pm = avg
        printf "cpu_seconds=%.3f cpu_avg_pct=%.1f cpu_max_pct=%.1f mem_avg_mb=%.1f mem_max_mb=%.1f throttled_periods=%d throttled_ms=%.0f samples=%d\n",
               cpu_s, avg, pm, sum / cnt, mx, ln - n0, (lh - h0) / 1000, cnt
    }'
}

# container_oom <container>: exit 0 if the kernel OOM-killed it
container_oom() {
    [[ "$(docker inspect -f '{{.State.OOMKilled}}' "$1" 2>/dev/null || echo false)" == "true" ]]
}

# ------------------------------------------------------------
# Standalone mode
# ------------------------------------------------------------
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    set -euo pipefail

    cmd="${1:-}"
    shift || true

    case "$cmd" in
        snapshot)
            c="${1:?usage: measure.sh snapshot <container>}"
            cur="$(cg_read "$c" memory.current)"
            peak="$(cg_read "$c" memory.peak)"
            cpu="$(cg_read "$c" cpu.stat | awk '/^usage_usec/{print $2}')"
            thr="$(cg_read "$c" cpu.stat | awk '/^throttled_usec/{print $2}')"
            rss="$(docker exec "$c" awk '/VmRSS:/{print $2}' /proc/1/status 2>/dev/null || true)"
            oom="$(docker inspect -f '{{.State.OOMKilled}}' "$c")"
            awk -v cur="${cur:-0}" -v peak="${peak:-0}" -v cpu="${cpu:-0}" -v thr="${thr:-0}" \
                -v rss="${rss:-0}" -v oom="$oom" 'BEGIN {
                printf "mem_current_mb=%.1f mem_peak_mb=%.1f pid1_rss_mb=%.1f cpu_seconds=%.3f throttled_ms=%.0f oom_killed=%s\n",
                       cur/1048576, peak/1048576, rss/1024, cpu/1e6, thr/1000, oom }'
            ;;
        watch)
            c="${1:?usage: measure.sh watch <container> [seconds]}"
            secs="${2:-10}"
            f="$(mktemp)"
            trap 'rm -f "$f"' EXIT
            measure_start "$c" "$f" "${SAMPLE_INTERVAL:-1}"
            sleep "$secs"
            measure_stop
            measure_summary "$f" "$secs"
            ;;
        time)
            [[ $# -gt 0 ]] || { echo "usage: measure.sh time <command...>" >&2; exit 2; }
            t0="$(now_ms)"
            rc=0
            "$@" || rc=$?
            t1="$(now_ms)"
            awk -v a="$t0" -v b="$t1" -v rc="$rc" 'BEGIN { printf "wall_seconds=%.3f exit_code=%d\n", (b-a)/1000, rc }'
            exit "$rc"
            ;;
        *)
            sed -n '3,12p' "$0" | sed 's/^# \{0,1\}//'
            exit 2
            ;;
    esac
fi
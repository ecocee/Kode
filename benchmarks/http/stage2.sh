#!/usr/bin/env bash
# ============================================================
# stage2.sh: the measurements Stage 2 of
# docs/plan-fix-http-concurrency.md asks for, and the evidence behind
# ADR 0037 (the HTTP I/O model).
#
# Protocol (from the plan):
#   * server in its own container limited to 1 vCPU / 1 GiB
#   * load generator in a separate container on the same network
#   * two independent generators, `oha` and `wrk` (not the in-repo client)
#
# What it measures:
#   1. requests/s and p50/p99/p99.9 at c=100 and c=1000, /hello and /json
#   2. container CPU seconds per request (user+sys) and throttling
#   3. server RSS while N keep-alive connections sit idle (N=100, 1000, 10000)
#   4. what the server does above its connection cap
#
# Output: one TSV row per run on stdout (and a report-ready block at the end).
#
#   ./benchmarks/http/stage2.sh                 # full run, ~4 minutes
#   READY=0 ./benchmarks/http/stage2.sh         # reuse running images
#   DURATION=5s CONNS="100" ./benchmarks/http/stage2.sh
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
# shellcheck source=benchmarks/http/measure.sh
source "$ROOT/benchmarks/http/measure.sh"

PREFIX="${PREFIX:-kode-http-stage2}"
NETWORK="$PREFIX-net"
SERVER="$PREFIX-server"
LOADCT="$PREFIX-load"
# Named HTTP_PORT because PORT is set in some shells and CI environments.
HTTP_PORT="${HTTP_PORT:-8080}"
CPUS="${CPUS:-1}"
MEMORY="${MEMORY:-1g}"
WORKERS="${WORKERS:-1}"
ENDPOINTS="${ENDPOINTS:-hello json}"
CONNS="${CONNS:-100 1000}"
DURATION="${DURATION:-15s}"
IDLE_CONNS="${IDLE_CONNS:-100 1000 10000}"
IDLE_HOLD="${IDLE_HOLD:-5s}"
TAG="${TAG:-kode}"
READY="${READY:-1}"

log() { printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*" >&2; }

# kv "<key=value ...>" <key>: read one value out of a measure_summary line
kv() {
    awk -v want="$2" -v line="$1" 'BEGIN { n = split(line, a, " "); for (i = 1; i <= n; i++) {
        split(a[i], p, "="); if (p[1] == want) { print p[2]; exit } } }'
}
die() { echo "ERROR: $*" >&2; exit 1; }

cleanup() {
    local rc=$?
    set +e
    docker rm -f "$SERVER" "$LOADCT" >/dev/null 2>&1
    exit "$rc"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

build_images() {
    local target
    for target in kode tools; do
        log "building image $PREFIX-$target"
        DOCKER_BUILDKIT=1 docker build --target "$target" \
            -f benchmarks/http/Dockerfile -t "$PREFIX-$target" . \
            > "/tmp/stage2-build-$target.log" 2>&1 \
            || { tail -n 30 "/tmp/stage2-build-$target.log" >&2; die "image build failed: $target"; }
    done
}

start_load_container() {
    # oha, wrk and idle in one container: they are tools, not services.
    docker rm -f "$LOADCT" >/dev/null 2>&1 || true
    docker run -d --name "$LOADCT" --network "$NETWORK" \
        --ulimit nofile=65535:65535 \
        --entrypoint sh "$PREFIX-tools" -c 'tail -f /dev/null' >/dev/null
}

start_server() {
    docker rm -f "$SERVER" >/dev/null 2>&1 || true
    docker run -d --name "$SERVER" --network "$NETWORK" \
        --cpus="$CPUS" --memory="$MEMORY" --memory-swap="$MEMORY" \
        --ulimit nofile=65535:65535 \
        -e "KODE_HTTP_WORKERS=$WORKERS" \
        "$PREFIX-$TAG" "$HTTP_PORT" >/dev/null

    local i
    for i in $(seq 1 240); do
        if docker exec "$LOADCT" curl -fsS --max-time 1 "http://$SERVER:$HTTP_PORT/hello" >/dev/null 2>&1; then
            log "server ready"
            return 0
        fi
        sleep 0.25
    done
    docker logs --tail 20 "$SERVER" >&2 || true
    die "server did not become ready"
}

# notify_exec <tool> <args...>: run a load tool from the load container, with
# the server's CPU and memory sampled for the duration.
run_oha() {
    local endpoint="$1" conns="$2"
    local out samples m cpu_s cpu_pct mem_max thr_ms
    samples="$(mktemp)"

    measure_start "$SERVER" "$samples" 0.5
    out="$(docker exec "$LOADCT" /bench/bin/oha --output-format json -c "$conns" -z "$DURATION" \
        "http://$SERVER:$HTTP_PORT/$endpoint")"
    measure_stop
    m="$(measure_summary "$samples" "$DURATION_SECONDS")"
    rm -f "$samples"

    cpu_s="$(kv "$m" cpu_seconds)"
    cpu_pct="$(kv "$m" cpu_avg_pct)"
    mem_max="$(kv "$m" mem_max_mb)"
    thr_ms="$(kv "$m" throttled_ms)"

    printf '%s' "$out" | python3 -c '
import json,sys
d=json.load(sys.stdin)
s=d["summary"]; lp=d["latencyPercentiles"]
count=sum(d.get("statusCodeDistribution",{}).values())
errors=sum(d.get("errorDistribution",{}).values())
cpu_s=float(sys.argv[1] or 0)
per=cpu_s*1e6/count if count else 0
print("oha\t%s\t%s\t%.1f\t%d\t%.3f\t%.2f\t%s\t%s\t%.2f\t%.2f\t%.2f\t%s\t%.4f\t%d" % (
    sys.argv[2], sys.argv[3], s["requestsPerSec"], count, cpu_s, per,
    sys.argv[4] or 0, sys.argv[5], round(lp["p50"]*1000,2), round(lp["p99"]*1000,2),
    round(lp["p99.9"]*1000,2), sys.argv[6] or 0, s["successRate"], errors))' \
        "$cpu_s" "$endpoint" "$conns" "$cpu_pct" "$mem_max" "$thr_ms"
}

run_wrk() {
    local endpoint="$1" conns="$2"
    local threads=2 out
    [[ "$conns" -ge 1000 ]] && threads=4
    out="$(docker exec "$LOADCT" /bench/bin/wrk -t"$threads" -c"$conns" -d"${DURATION}" --latency \
        "http://$SERVER:$HTTP_PORT/$endpoint" 2>&1 || true)"
    printf '%s' "$out" | awk -v ep="$endpoint" -v c="$conns" -v t="$threads" '
        function ms(v) {
            if (v ~ /us$/) return substr(v, 1, length(v) - 2) / 1000
            if (v ~ /ms$/) return substr(v, 1, length(v) - 2)
            if (v ~ /s$/)  return substr(v, 1, length(v) - 1) * 1000
            return v
        }
        /Requests\/sec:/ { rps = $2 }
        /Socket errors:/ { errs = $0 }
        /^ *50%/        { p50 = ms($2) }
        /^ *99%/        { p99 = ms($2) }
        END {
            printf "wrk\t%s\t%s\t%.1f\t\t\t\t\t\t%.2f\t%.2f\t\t\t\t%s\n",
                   ep, c, rps, p50, p99, "-"
        }'
}

run_idle() {
    local conns="$1"
    local out samples peak mb
    samples="$(mktemp)"
    measure_start "$SERVER" "$samples" 0.5
    out="$(docker exec "$LOADCT" /bench/bin/idle -url "http://$SERVER:$HTTP_PORT/hello" \
        -conns "$conns" -hold "$IDLE_HOLD" 2>/dev/null)"
    measure_stop
    peak="$(awk '{ if ($2 > m) m = $2 } END { printf "%.1f", m/1048576 }' "$samples")"
    rm -f "$samples"
    printf '%s\t%s\n' "$peak" "$out"
}

main() {
    command -v docker >/dev/null || die "docker is required"
    docker info >/dev/null 2>&1 || die "docker daemon is not running"
    DURATION_SECONDS="$(printf '%s' "$DURATION" | awk '{ n=substr($0,1,length($0)-1); u=substr($0,length($0)); if (u=="s") print n; else if (u=="m") print n*60; else print n }')"

    if [[ "$READY" == "1" ]]; then
        build_images
    fi

    docker network inspect "$NETWORK" >/dev/null 2>&1 || docker network create "$NETWORK" >/dev/null
    start_load_container
    start_server

    log "server config: cpus=$CPUS memory=$MEMORY KODE_HTTP_WORKERS=$WORKERS"

    # Fixed warmup so the first measured run is not the first request ever.
    docker exec "$LOADCT" /bench/bin/oha -c 10 -n 2000 "http://$SERVER:$HTTP_PORT/hello" >/dev/null 2>&1 || true

    printf 'tool\tendpoint\tconnections\trps\trequests\tcpu_seconds\tcpu_us_per_request\tcpu_pct\tmem_max_mb\tp50_ms\tp99_ms\tp99.9_ms\tthrottled_ms\tsuccess_rate\taborted_at_deadline\n'

    local endpoint conns
    for endpoint in $ENDPOINTS; do
        for conns in $CONNS; do
            run_oha "$endpoint" "$conns"
            run_wrk "$endpoint" "$conns"
        done
    done

    echo
    printf 'idle_connections\tpeak_rss_mb\testablished\tdial_failed\tclosed_early\tstill_open\tconnect_p50_ms\tconnect_p99_ms\n'
    local peak rest
    local out
    for conns in $IDLE_CONNS; do
        out="$(run_idle "$conns")"
        peak="${out%%$'\t'*}"
        rest="${out#*$'\t'}"
        local est dialf closed open p50 p99
        est="$(sed -n 's/.*established=\([0-9]*\).*/\1/p' <<< "$rest")"
        dialf="$(sed -n 's/.*dial_failed=\([0-9]*\).*/\1/p' <<< "$rest")"
        closed="$(sed -n 's/.*closed_early=\([0-9]*\).*/\1/p' <<< "$rest")"
        open="$(sed -n 's/.*still_open=\([0-9]*\).*/\1/p' <<< "$rest")"
        p50="$(sed -n 's/.*connect_ms_p50=\([0-9.]*\).*/\1/p' <<< "$rest")"
        p99="$(sed -n 's/.*connect_ms_p99=\([0-9.]*\).*/\1/p' <<< "$rest")"
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$conns" "$peak" "$est" "$dialf" "$closed" "$open" "$p50" "$p99"
    done
}

main "$@"

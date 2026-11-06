#!/usr/bin/env bash
# HTTP benchmark: hello and JSON, at 100 and 1,000 keep-alive connections, for
# 1,000 and 100,000 requests. Runs inside the image built from ./Dockerfile on a
# 1 vCPU / 1 GB container; every number is written to results.tsv.
#
#   docker build -f benchmarks/http/Dockerfile -t kode-http-bench .
#   docker run --rm --cpus=1 --memory=1g -v "$PWD/benchmarks/http:/bench" \
#       kode-http-bench bash /bench/measure.sh
set -uo pipefail

KODE=${KODE:-/src/target/release/kode}
PORT=${PORT:-8080}
RESULTS=/bench/results.tsv
CONNECTIONS=(100 1000)
REQUESTS=(1000 100000)
ENDPOINTS=(hello json)

printf 'language\tendpoint\tconnections\trequests\tactual_requests\trps\tp50_ms\tp99_ms\terrors\trss_after_kb\tpeak_rss_kb\tstatus\n' > "$RESULTS"

log() { printf '%s\n' "$*" >&2; }

# --- build the servers and the load client -----------------------------------
mkdir -p /bench/bin
log "building the load client"
go build -o /bench/bin/load /bench/load.go || { log "load client failed to build"; exit 1; }
log "building the Kode server with --release"
if ! "$KODE" build /bench/server.kode --release -o /bench/bin/kode_server >/tmp/kbuild.log 2>&1; then
    log "kode build failed:"; cat /tmp/kbuild.log; exit 1
fi
log "building the Go server (go build is optimized by default)"
go build -o /bench/bin/go_server /bench/server.go || { log "go build failed"; exit 1; }

start_server() {
    case "$1" in
        kode)   /bench/bin/kode_server "$PORT" >/tmp/srv.log 2>&1 & echo $! ;;
        go)     /bench/bin/go_server "$PORT" >/tmp/srv.log 2>&1 & echo $! ;;
        node)   node /bench/server.js "$PORT" >/tmp/srv.log 2>&1 & echo $! ;;
        python) python3 /bench/server.py "$PORT" >/tmp/srv.log 2>&1 & echo $! ;;
    esac
}

ready() {
    for _ in $(seq 1 150); do
        if curl -sf -m 1 "http://127.0.0.1:$PORT/hello" >/dev/null 2>&1; then return 0; fi
        sleep 0.1
    done
    return 1
}

rss_kb()  { awk '/^VmRSS:/ {print $2}' "/proc/$1/status" 2>/dev/null || echo 0; }
peak_kb() { awk '/^VmHWM:/ {print $2}' "/proc/$1/status" 2>/dev/null || echo 0; }

measure() {  # measure <language> <endpoint> <connections> <requests>
    local lang=$1 endpoint=$2 conns=$3 reqs=$4
    local out
    out=$(/bench/bin/load -url "http://127.0.0.1:$PORT/$endpoint" -n "$reqs" -c "$conns" 2>/dev/null)
    local actual rps p50 p99 errs status
    actual=$(sed -n 's/.*requests=\([0-9]*\).*/\1/p' <<<"$out")
    rps=$(sed -n 's/.*rps=\([0-9.]*\).*/\1/p' <<<"$out")
    p50=$(sed -n 's/.*p50_ms=\([0-9.]*\).*/\1/p' <<<"$out")
    p99=$(sed -n 's/.*p99_ms=\([0-9.]*\).*/\1/p' <<<"$out")
    errs=$(sed -n 's/.*errors=\([0-9]*\).*/\1/p' <<<"$out")
    : "${actual:=0}" "${rps:=0}" "${p50:=0}" "${p99:=0}" "${errs:=0}"
    status=ok
    [ "$actual" = "0" ] && status=failed
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$lang" "$endpoint" "$conns" "$reqs" "$actual" "$rps" "$p50" "$p99" "$errs" \
        "$(rss_kb "$SERVER_PID")" "$(peak_kb "$SERVER_PID")" "$status" >> "$RESULTS"
    log "  $lang $endpoint c=$conns n=$reqs -> $actual reqs, $rps rps, p99 $p99 ms, $errs errors"
}

for lang in kode go node python; do
    SERVER_PID=$(start_server "$lang")
    if ! ready; then
        log "$lang: server did not answer"
        printf '%s\t-\t-\t-\t0\t0\t0\t0\t0\t0\t0\tno-server\n' "$lang" >> "$RESULTS"
        kill "$SERVER_PID" 2>/dev/null; continue
    fi
    log "$lang: up (pid $SERVER_PID, rss $(rss_kb "$SERVER_PID") KB idle)"
    for endpoint in "${ENDPOINTS[@]}"; do
        for conns in "${CONNECTIONS[@]}"; do
            for reqs in "${REQUESTS[@]}"; do
                measure "$lang" "$endpoint" "$conns" "$reqs"
            done
        done
    done
    kill "$SERVER_PID" 2>/dev/null
    wait "$SERVER_PID" 2>/dev/null
done

log "done: $RESULTS"
cat "$RESULTS" >&2

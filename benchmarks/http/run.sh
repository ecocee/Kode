#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Kode HTTP Benchmark
# Languages: Kode, Go, Node.js, Java, PHP, Python, C#
#
# Every server runs in its own Docker container with its own CPU / memory /
# thread limits. All images and containers are removed when the run ends
# (success, failure or Ctrl-C) unless KEEP_IMAGES=1.
#
# Examples
#   ./benchmarks/http/run.sh                                  # defaults: 1 CPU, 1 GB, auto threads
#   PRESET=quick ./benchmarks/http/run.sh                     # fast smoke test
#   CPUS_LIST="0.25 0.5 1 2 4" ./benchmarks/http/run.sh       # CPU sweep
#   MEM_LIST="100m 256m 512m 1g 4g" ./benchmarks/http/run.sh  # memory sweep
#   CPUS_LIST=4 THREADS_LIST="1 2 4 8" ./benchmarks/http/run.sh
#   LANGS="kode go java" REPETITIONS=3 ./benchmarks/http/run.sh
#
# Settings (environment variables)
#   LANGS            languages to run          (kode go node java php python csharp)
#   CPUS_LIST        CPU limits, 0.25 .. 4     (1)
#   MEM_LIST         memory limits, 100m .. 4g (1g)
#   THREADS_LIST     "auto" or integers        (auto = ceil(cpus))
#   CONNECTIONS_LIST / REQUESTS_LIST / ENDPOINTS_LIST
#   REPETITIONS      repeats per config        (5)
#   PRESET           quick | cpu-sweep | mem-sweep | thread-sweep
#   MIN_CPUS MAX_CPUS MIN_MEM_MB MAX_MEM_MB MAX_THREADS   allowed ranges
#   SERVER_CPUSET / LOAD_CPUSET   pin server / load generator to host cores, e.g. "0-3" / "4-7"
#   SAMPLE_INTERVAL  resource sampling seconds (1)
#   READY_TIMEOUT_S  server start timeout      (60)
#   KEEP_IMAGES=1    do not delete images afterwards
#   PRUNE_CACHE=0    do not run `docker builder prune` afterwards
# ============================================================

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

HERE="benchmarks/http"
# shellcheck source=benchmarks/http/measure.sh
source "$ROOT/$HERE/measure.sh"

# ------------------------------------------------------------
# Presets (only fill in what the caller did not set)
# ------------------------------------------------------------
case "${PRESET:-}" in
    quick)        : "${REPETITIONS:=1}" "${CONNECTIONS_LIST:=100}" "${REQUESTS_LIST:=1000}" ;;
    cpu-sweep)    : "${CPUS_LIST:=0.25 0.5 1 2 4}" "${REPETITIONS:=3}" ;;
    mem-sweep)    : "${MEM_LIST:=100m 256m 512m 1g 2g 4g}" "${REPETITIONS:=3}" ;;
    thread-sweep) : "${THREADS_LIST:=1 2 4 8}" "${REPETITIONS:=3}" ;;
    "")           ;;
    *)            echo "ERROR: unknown PRESET '${PRESET}'" >&2; exit 1 ;;
esac

PREFIX="${PREFIX:-kode-http-bench}"
NETWORK="${NETWORK:-$PREFIX-net}"
LABEL="$PREFIX=1"
SERVER="${SERVER:-kode-http-server}"
LOAD="$SERVER-load"
PORT="${PORT:-8080}"

REPETITIONS="${REPETITIONS:-5}"
read -ra LANGUAGES    <<< "${LANGS:-kode go node java php python csharp}"
read -ra CPU_VALUES   <<< "${CPUS_LIST:-1}"
read -ra MEM_VALUES   <<< "${MEM_LIST:-1g}"
read -ra THREAD_VALUES <<< "${THREADS_LIST:-auto}"
read -ra CONNECTIONS  <<< "${CONNECTIONS_LIST:-100 1000}"
read -ra REQUESTS     <<< "${REQUESTS_LIST:-1000 100000}"
read -ra ENDPOINTS    <<< "${ENDPOINTS_LIST:-hello json}"

MIN_CPUS="${MIN_CPUS:-0.25}"
MAX_CPUS="${MAX_CPUS:-4}"
MIN_MEM_MB="${MIN_MEM_MB:-100}"
MAX_MEM_MB="${MAX_MEM_MB:-4096}"
MAX_THREADS="${MAX_THREADS:-256}"

SERVER_CPUSET="${SERVER_CPUSET:-}"
LOAD_CPUSET="${LOAD_CPUSET:-}"
SAMPLE_INTERVAL="${SAMPLE_INTERVAL:-1}"
READY_TIMEOUT_S="${READY_TIMEOUT_S:-60}"
KEEP_IMAGES="${KEEP_IMAGES:-0}"
PRUNE_CACHE="${PRUNE_CACHE:-1}"

RESULT_DIR="$HERE"
RESULTS="$RESULT_DIR/results.tsv"
SUMMARY="$RESULT_DIR/summary.tsv"
BUILD_TIMES="$RESULT_DIR/build_times.tsv"
SAMPLES="$RESULT_DIR/.samples.$$"

CURRENT_SERVER=""
STARTUP_MS=0
RUN_NO=0
TOTAL_RUNS=0
RUN_START_MS=0

HOST_CPUS="$(docker info --format '{{.NCPU}}' 2>/dev/null || echo 1)"

# ============================================================
# Logging / small helpers
# ============================================================

log() { printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"; }
die() { echo "ERROR: $*" >&2; exit 1; }

fmt_duration() {
    awk -v s="$1" 'BEGIN {
        s = int(s + 0.5); h = int(s / 3600); m = int((s % 3600) / 60); sec = s % 60
        if (h > 0) printf "%dh%02dm%02ds", h, m, sec
        else if (m > 0) printf "%dm%02ds", m, sec
        else printf "%ds", sec }'
}

# mem_to_mb 512m -> 512 ; 4g -> 4096 ; -1 if unparseable
mem_to_mb() {
    awk -v m="$1" 'BEGIN {
        if (match(m, /^[0-9.]+/) == 0) { print -1; exit }
        n = substr(m, RSTART, RLENGTH) + 0
        u = tolower(substr(m, RLENGTH + 1))
        if (u == "" || u == "m" || u == "mb")      print n
        else if (u == "g" || u == "gb")            print n * 1024
        else if (u == "k" || u == "kb")            print n / 1024
        else                                       print -1 }'
}

# resolve_threads <cpus> <auto|N>
resolve_threads() {
    if [[ "$2" == "auto" ]]; then
        awk -v c="$1" 'BEGIN { t = int(c); if (t < c) t++; if (t < 1) t = 1; print t }'
    else
        echo "$2"
    fi
}

# Exact key=value parser for the load generator output
get_value() {
    local key="$1" output="$2"
    awk -v key="$key" '
    {
        for (i = 1; i <= NF; i++) {
            sep = index($i, "=")
            if (sep > 0 && substr($i, 1, sep - 1) == key) { print substr($i, sep + 1); exit }
        }
    }' <<< "$output"
}

# ============================================================
# Validation
# ============================================================

validate_config() {
    [[ "$REPETITIONS" =~ ^[1-9][0-9]*$ ]] || die "REPETITIONS must be a positive integer"

    local l c m t mb
    for l in "${LANGUAGES[@]}"; do
        case "$l" in kode|go|node|java|php|python|csharp) ;; *) die "unknown language '$l'" ;; esac
    done

    for c in "${CPU_VALUES[@]}"; do
        awk -v c="$c" -v lo="$MIN_CPUS" -v hi="$MAX_CPUS" -v host="$HOST_CPUS" \
            'BEGIN { exit !(c ~ /^[0-9]*\.?[0-9]+$/ && c + 0 >= lo && c + 0 <= hi && c + 0 <= host) }' \
            || die "CPU value '$c' must be between $MIN_CPUS and $MAX_CPUS (this Docker host has $HOST_CPUS CPUs)"
    done

    for m in "${MEM_VALUES[@]}"; do
        mb="$(mem_to_mb "$m")"
        awk -v v="$mb" -v lo="$MIN_MEM_MB" -v hi="$MAX_MEM_MB" 'BEGIN { exit !(v >= lo && v <= hi) }' \
            || die "memory value '$m' must be between ${MIN_MEM_MB}m and ${MAX_MEM_MB}m"
    done

    for t in "${THREAD_VALUES[@]}"; do
        if [[ "$t" != "auto" ]]; then
            [[ "$t" =~ ^[1-9][0-9]*$ ]] && (( t <= MAX_THREADS )) \
                || die "threads value '$t' must be 'auto' or an integer from 1 to $MAX_THREADS"
        fi
    done
}

# ============================================================
# Docker setup / cleanup
# ============================================================

remove_containers() {
    local id
    for id in $(docker ps -aq --filter "label=$LABEL" 2>/dev/null); do
        docker rm -f "$id" >/dev/null 2>&1 || true
    done
}

remove_images() {
    local id
    for id in $(docker image ls -q --filter "label=$LABEL" 2>/dev/null | sort -u); do
        docker rmi -f "$id" >/dev/null 2>&1 || true
    done
    docker image prune -f >/dev/null 2>&1 || true
    if [[ "$PRUNE_CACHE" == "1" ]]; then
        docker builder prune -f >/dev/null 2>&1 || true
    fi
}

cleanup() {
    local rc=$?
    set +e
    measure_stop 2>/dev/null
    rm -f "$SAMPLES"
    log "Cleaning up Docker containers, network and images"
    remove_containers
    docker network rm "$NETWORK" >/dev/null 2>&1
    if [[ "$KEEP_IMAGES" == "1" ]]; then
        log "KEEP_IMAGES=1: images kept"
    else
        remove_images
        log "Docker images removed"
    fi
    exit "$rc"
}

trap cleanup EXIT
trap 'exit 130' INT TERM

ensure_network() {
    docker network inspect "$NETWORK" >/dev/null 2>&1 || docker network create "$NETWORK" >/dev/null
}

build_images() {
    printf 'target\tbuild_seconds\timage_mb\n' > "$BUILD_TIMES"

    local target t0 t1 size
    for target in "${LANGUAGES[@]}" load; do
        log "Building image: $target"
        t0="$(now_ms)"

        if ! DOCKER_BUILDKIT=1 docker build \
                --target "$target" \
                --label "$LABEL" \
                -f "$HERE/Dockerfile" \
                -t "$PREFIX-$target" . \
                > "$RESULT_DIR/build-$target.log" 2>&1; then
            tail -n 40 "$RESULT_DIR/build-$target.log" >&2
            die "image build failed for '$target' (full log: $RESULT_DIR/build-$target.log)"
        fi

        t1="$(now_ms)"
        size="$(docker image inspect -f '{{.Size}}' "$PREFIX-$target")"
        printf '%s\t%s\t%s\n' "$target" \
            "$(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.1f", (b - a) / 1000 }')" \
            "$(awk -v s="$size" 'BEGIN { printf "%.1f", s / 1048576 }')" >> "$BUILD_TIMES"
        log "Built $target in $(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.1f", (b - a) / 1000 }')s, image $(awk -v s="$size" 'BEGIN { printf "%.0f", s / 1048576 }') MB"
    done
}

image_mb() {
    docker image inspect -f '{{.Size}}' "$PREFIX-$1" 2>/dev/null \
        | awk '{ printf "%.1f", $1 / 1048576 }'
}

start_load_generator() {
    docker rm -f "$LOAD" >/dev/null 2>&1 || true

    local args=(run -d --name "$LOAD" --network "$NETWORK" --label "$LABEL")
    if [[ -n "$LOAD_CPUSET" ]]; then
        args+=(--cpuset-cpus="$LOAD_CPUSET")
    fi
    args+=("$PREFIX-load")

    docker "${args[@]}" >/dev/null
}

# ============================================================
# Server lifecycle
# ============================================================

# start_server <language> <cpus> <memory> <threads>   (sets STARTUP_MS)
start_server() {
    local language="$1" cpus="$2" memory="$3" threads="$4"
    CURRENT_SERVER="$SERVER-$language"

    docker rm -f "$CURRENT_SERVER" >/dev/null 2>&1 || true

    local args=(run -d
        --name "$CURRENT_SERVER"
        --network "$NETWORK"
        --label "$LABEL"
        --cpus="$cpus"
        --memory="$memory"
        --memory-swap="$memory"
        -e "THREADS=$threads"
        -e "GOMAXPROCS=$threads"
        -e "KODE_HTTP_WORKERS=$threads"
        -e "DOTNET_PROCESSOR_COUNT=$threads")
    if [[ -n "$SERVER_CPUSET" ]]; then
        args+=(--cpuset-cpus="$SERVER_CPUSET")
    fi
    args+=("$PREFIX-$language" "$PORT")

    local t0
    t0="$(now_ms)"
    docker "${args[@]}" >/dev/null || return 1

    local ticks=$(( READY_TIMEOUT_S * 4 )) i
    for ((i = 0; i < ticks; i++)); do
        if [[ "$(docker inspect -f '{{.State.Running}}' "$CURRENT_SERVER" 2>/dev/null || echo false)" != "true" ]]; then
            break
        fi
        if docker exec "$LOAD" curl -fsS --max-time 1 "http://$CURRENT_SERVER:$PORT/hello" >/dev/null 2>&1; then
            STARTUP_MS=$(( $(now_ms) - t0 ))
            return 0
        fi
        sleep 0.25
    done

    echo "ERROR: $language server failed to start (cpus=$cpus memory=$memory threads=$threads)." >&2
    docker logs --tail 20 "$CURRENT_SERVER" >&2 || true
    return 1
}

stop_server() {
    if [[ -n "${CURRENT_SERVER:-}" ]]; then
        docker rm -f "$CURRENT_SERVER" >/dev/null 2>&1 || true
        CURRENT_SERVER=""
    fi
}

warmup() {
    log "Warming up $1"
    docker exec "$LOAD" /bench/bin/load \
        -url "http://$CURRENT_SERVER:$PORT/hello" -c 10 -n 1000 >/dev/null 2>&1 || true
}

# ============================================================
# Results
# ============================================================

write_header() {
    printf '%s\n' \
        $'language\trepetition\tcpus\tmemory\tthreads\tendpoint\tconnections\ttarget_requests\tsuccessful_requests\tfailed_requests\tunattempted_requests\trps\tp50_ms\tp95_ms\tp99_ms\tduration_s\tcpu_avg_pct\tcpu_max_pct\tcpu_seconds\tthrottled_ms\tmem_avg_mb\tmem_max_mb\tmem_peak_mb\tidle_mem_mb\tserver_process_rss_kb\tstartup_ms\timage_mb\tstatus' \
        > "$RESULTS"
}

# write_failed_row <lang> <rep> <cpus> <mem> <threads> <status>
write_failed_row() {
    local IFS=$'\t'
    local row=("$1" "$2" "$3" "$4" "$5" "-" 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 "$(image_mb "$1")" "$6")
    printf '%s\n' "${row[*]}" >> "$RESULTS"
}

# Returns 0 normally, 2 if the server died (OOM / crash) so the caller stops.
run_load() {
    local language="$1" repetition="$2" cpus="$3" memory="$4" threads="$5"
    local endpoint="$6" connections="$7" requests="$8" idle_mb="$9"

    local url="http://$CURRENT_SERVER:$PORT/$endpoint"
    local output rc=0

    measure_start "$CURRENT_SERVER" "$SAMPLES" "$SAMPLE_INTERVAL"
    output="$(docker exec "$LOAD" /bench/bin/load -url "$url" -c "$connections" -n "$requests" 2>&1)" || rc=$?
    measure_stop

    local successful failed unattempted rps p50 p95 p99 duration
    successful="$(get_value requests "$output")"
    failed="$(get_value failed_requests "$output")"
    unattempted="$(get_value unattempted_requests "$output")"
    rps="$(get_value rps "$output")"
    p50="$(get_value p50_ms "$output")"
    p95="$(get_value p95_ms "$output")"
    p99="$(get_value p99_ms "$output")"
    duration="$(get_value seconds "$output")"

    local m cpu_avg cpu_max cpu_s thr_ms mem_avg mem_max
    m="$(measure_summary "$SAMPLES" "${duration:-0}")"
    cpu_s="$(get_value cpu_seconds "$m")"
    cpu_avg="$(get_value cpu_avg_pct "$m")"
    cpu_max="$(get_value cpu_max_pct "$m")"
    thr_ms="$(get_value throttled_ms "$m")"
    mem_avg="$(get_value mem_avg_mb "$m")"
    mem_max="$(get_value mem_max_mb "$m")"

    local rss peak_mb
    rss="$(docker exec "$CURRENT_SERVER" awk '/VmRSS:/ {print $2}' /proc/1/status 2>/dev/null || true)"
    peak_mb="$(cg_read "$CURRENT_SERVER" memory.peak | awk '{ printf "%.1f", $1 / 1048576 }')"

    local status="ok"
    if container_oom "$CURRENT_SERVER"; then
        status="oom"
    elif [[ "$(docker inspect -f '{{.State.Running}}' "$CURRENT_SERVER" 2>/dev/null || echo false)" != "true" ]]; then
        status="crashed"
    elif [[ -z "$rps" ]]; then
        status="no_output"
    elif [[ "${failed:-0}" != "0" || "${unattempted:-0}" != "0" || "$rc" -ne 0 ]]; then
        status="errors"
    fi

    local IFS=$'\t'
    local row=(
        "$language" "$repetition" "$cpus" "$memory" "$threads" "$endpoint" "$connections" "$requests"
        "${successful:-0}" "${failed:-0}" "${unattempted:-0}" "${rps:-0}" "${p50:-0}" "${p95:-0}" "${p99:-0}"
        "${duration:-0}" "${cpu_avg:-0}" "${cpu_max:-0}" "${cpu_s:-0}" "${thr_ms:-0}"
        "${mem_avg:-0}" "${mem_max:-0}" "${peak_mb:-0}" "$idle_mb" "${rss:-0}" "$STARTUP_MS" "$(image_mb "$language")" "$status"
    )
    printf '%s\n' "${row[*]}" >> "$RESULTS"

    RUN_NO=$(( RUN_NO + 1 ))
    local elapsed eta
    elapsed="$(awk -v a="$RUN_START_MS" -v b="$(now_ms)" 'BEGIN { printf "%.0f", (b - a) / 1000 }')"
    eta="$(awk -v e="$elapsed" -v n="$RUN_NO" -v t="$TOTAL_RUNS" 'BEGIN { printf "%.0f", e / n * (t - n) }')"

    log "[$RUN_NO/$TOTAL_RUNS] $language cpu=$cpus mem=$memory thr=$threads $endpoint c=$connections n=$requests rep=$repetition: ${rps:-0} RPS, p99=${p99:-0}ms, cpu=${cpu_avg:-0}%, mem<=${mem_max:-0}MB, ${duration:-0}s, status=$status (elapsed $(fmt_duration "$elapsed"), ETA $(fmt_duration "$eta"))"

    if [[ "$status" == "oom" || "$status" == "crashed" ]]; then
        return 2
    fi
    return 0
}

# Runs every endpoint x connections x requests combo against the live server.
run_matrix() {
    local language="$1" repetition="$2" cpus="$3" memory="$4" threads="$5" idle_mb="$6"
    local endpoint connections requests rc

    for endpoint in "${ENDPOINTS[@]}"; do
        for connections in "${CONNECTIONS[@]}"; do
            for requests in "${REQUESTS[@]}"; do
                rc=0
                run_load "$language" "$repetition" "$cpus" "$memory" "$threads" \
                         "$endpoint" "$connections" "$requests" "$idle_mb" || rc=$?
                if [[ "$rc" -eq 2 ]]; then
                    log "$language died under load (OOM or crash); skipping its remaining runs for this config"
                    return 0
                fi
            done
        done
    done
}

# ============================================================
# Main
# ============================================================

main() {
    command -v docker >/dev/null  || die "docker is required"
    command -v python3 >/dev/null || die "python3 is required"
    docker info >/dev/null 2>&1   || die "cannot talk to the Docker daemon"

    validate_config

    TOTAL_RUNS=$(( REPETITIONS * ${#CPU_VALUES[@]} * ${#MEM_VALUES[@]} * ${#THREAD_VALUES[@]} \
                 * ${#LANGUAGES[@]} * ${#ENDPOINTS[@]} * ${#CONNECTIONS[@]} * ${#REQUESTS[@]} ))

    log "Kode HTTP benchmark"
    log "Languages:   ${LANGUAGES[*]}"
    log "CPUs:        ${CPU_VALUES[*]}   Memory: ${MEM_VALUES[*]}   Threads: ${THREAD_VALUES[*]}"
    log "Repetitions: $REPETITIONS   Planned load runs: $TOTAL_RUNS   (host CPUs: $HOST_CPUS)"

    mkdir -p "$RESULT_DIR"

    # clear leftovers from any interrupted earlier run
    remove_containers
    ensure_network
    build_images
    start_load_generator
    write_header

    RUN_START_MS="$(now_ms)"

    local repetition cpus memory tspec threads language offset i idle_mb
    for ((repetition = 1; repetition <= REPETITIONS; repetition++)); do
        for cpus in "${CPU_VALUES[@]}"; do
            for memory in "${MEM_VALUES[@]}"; do
                for tspec in "${THREAD_VALUES[@]}"; do

                    threads="$(resolve_threads "$cpus" "$tspec")"

                    log "============================================"
                    log "Repetition $repetition/$REPETITIONS  cpus=$cpus memory=$memory threads=$threads"
                    log "============================================"

                    # Rotate language order to reduce ordering bias.
                    offset=$(( (repetition - 1) % ${#LANGUAGES[@]} ))
                    local ordered=()
                    for ((i = 0; i < ${#LANGUAGES[@]}; i++)); do
                        ordered+=("${LANGUAGES[$(( (i + offset) % ${#LANGUAGES[@]} ))]}")
                    done

                    for language in "${ordered[@]}"; do
                        log "Starting $language server"

                        if start_server "$language" "$cpus" "$memory" "$threads"; then
                            log "$language ready in ${STARTUP_MS} ms"
                            warmup "$language"
                            idle_mb="$(cg_read "$CURRENT_SERVER" memory.current | awk '{ printf "%.1f", $1 / 1048576 }')"
                            run_matrix "$language" "$repetition" "$cpus" "$memory" "$threads" "${idle_mb:-0}"
                        else
                            local why="start_failed"
                            if container_oom "$CURRENT_SERVER"; then why="oom_at_start"; fi
                            write_failed_row "$language" "$repetition" "$cpus" "$memory" "$threads" "$why"
                            RUN_NO=$(( RUN_NO + ${#ENDPOINTS[@]} * ${#CONNECTIONS[@]} * ${#REQUESTS[@]} ))
                        fi

                        stop_server
                    done
                done
            done
        done
    done

    local total_s
    total_s="$(awk -v a="$RUN_START_MS" -v b="$(now_ms)" 'BEGIN { printf "%.0f", (b - a) / 1000 }')"

    log "Generating summary"
    python3 "$HERE/summarize.py" "$RESULTS" "$SUMMARY" "$BUILD_TIMES"

    log "============================================"
    log "Benchmark completed in $(fmt_duration "$total_s") (load phase)"
    log "Raw results:  $RESULTS"
    log "Summary:      $SUMMARY"
    log "Build times:  $BUILD_TIMES"
    log "============================================"
}

main "$@"
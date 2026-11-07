
#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Kode HTTP Benchmark
# Languages: Kode, Go, Node.js
# ============================================================

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

IMAGE="${IMAGE:-kode-http-bench}"
NETWORK="${NETWORK:-kode-http-bench-net}"
SERVER="${SERVER:-kode-http-server}"
PORT="${PORT:-8080}"

REPETITIONS="${REPETITIONS:-5}"
KODE_HTTP_WORKERS="${KODE_HTTP_WORKERS:-}"

CONNECTIONS=(100 1000)
REQUESTS=(1000 100000)
LANGUAGES=(kode go node)
ENDPOINTS=(hello json)

RESULT_DIR="benchmarks/http"
RESULTS="$RESULT_DIR/results.tsv"
SUMMARY="$RESULT_DIR/summary.tsv"

CURRENT_SERVER=""

# ============================================================
# Logging
# ============================================================

log() {
    printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"
}

# ============================================================
# Cleanup
# ============================================================

cleanup() {
    if [[ -n "${CURRENT_SERVER:-}" ]]; then
        docker rm -f "$CURRENT_SERVER" >/dev/null 2>&1 || true
    fi

    docker rm -f "$SERVER-load" >/dev/null 2>&1 || true
}

trap cleanup EXIT INT TERM

# ============================================================
# Docker setup
# ============================================================

ensure_network() {
    if ! docker network inspect "$NETWORK" >/dev/null 2>&1; then
        docker network create "$NETWORK" >/dev/null
    fi
}

ensure_image() {
    if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
        log "Building benchmark Docker image"

        docker build \
            -f benchmarks/http/Dockerfile \
            -t "$IMAGE" .
    fi
}

# ============================================================
# Server commands
# ============================================================

server_command() {
    local language="$1"

    case "$language" in
        kode)
            echo "/bench/bin/kode_server $PORT"
            ;;
        go)
            echo "/bench/bin/go_server $PORT"
            ;;
        node)
            echo "node /src/benchmarks/http/server.js $PORT"
            ;;
        *)
            echo "Unknown language: $language" >&2
            return 1
            ;;
    esac
}

# ============================================================
# Start server
# ============================================================

start_server() {
    local language="$1"
    local command

    command="$(server_command "$language")"

    CURRENT_SERVER="${SERVER}-${language}"

    log "Starting $language server"

    # Kode can receive an optional worker override.
    # Go and Node.js use their normal configurations.
    if [[ "$language" == "kode" && -n "$KODE_HTTP_WORKERS" ]]; then

        log "Kode HTTP workers: $KODE_HTTP_WORKERS"

        docker run -d \
            --name "$CURRENT_SERVER" \
            --network "$NETWORK" \
            --cpus=1 \
            --memory=1g \
            -e "KODE_HTTP_WORKERS=$KODE_HTTP_WORKERS" \
            "$IMAGE" \
            bash -lc "exec $command" >/dev/null

    else

        docker run -d \
            --name "$CURRENT_SERVER" \
            --network "$NETWORK" \
            --cpus=1 \
            --memory=1g \
            "$IMAGE" \
            bash -lc "exec $command" >/dev/null

    fi

    # Wait for server readiness.
    local ready=0

    for _ in $(seq 1 40); do

        if docker exec "$CURRENT_SERVER" \
            curl -fsS "http://127.0.0.1:$PORT/hello" \
            >/dev/null 2>&1; then

            ready=1
            break
        fi

        sleep 0.5
    done

    if [[ "$ready" -ne 1 ]]; then
        echo "ERROR: $language server failed to start."

        docker logs "$CURRENT_SERVER" || true

        return 1
    fi

    log "$language server is ready"
}

# ============================================================
# Stop server
# ============================================================

stop_server() {
    if [[ -n "${CURRENT_SERVER:-}" ]]; then
        docker rm -f "$CURRENT_SERVER" >/dev/null 2>&1 || true
        CURRENT_SERVER=""
    fi
}

# ============================================================
# Start load generator
# ============================================================

start_load_generator() {
    docker rm -f "$SERVER-load" >/dev/null 2>&1 || true

    docker run -d \
        --name "$SERVER-load" \
        --network "$NETWORK" \
        "$IMAGE" \
        sleep infinity >/dev/null
}

# ============================================================
# Exact key-value parser
# ============================================================

get_value() {
    local key="$1"
    local output="$2"

    awk -v key="$key" '
    {
        for (i = 1; i <= NF; i++) {

            separator = index($i, "=")

            if (separator > 0) {

                field = substr($i, 1, separator - 1)

                if (field == key) {
                    print substr($i, separator + 1)
                    exit
                }
            }
        }
    }
    ' <<< "$output"
}

# ============================================================
# Run load test
# ============================================================

run_load() {
    local language="$1"
    local endpoint="$2"
    local connections="$3"
    local requests="$4"
    local repetition="$5"

    local url="http://${SERVER}-${language}:${PORT}/${endpoint}"
    local output

    output="$(
        docker exec "$SERVER-load" \
            /bench/bin/load \
            -url "$url" \
            -c "$connections" \
            -n "$requests"
    )"

    local successful
    local failed
    local unattempted
    local rps
    local p50
    local p95
    local p99

    successful="$(get_value "requests" "$output")"
    failed="$(get_value "failed_requests" "$output")"
    unattempted="$(get_value "unattempted_requests" "$output")"
    rps="$(get_value "rps" "$output")"
    p50="$(get_value "p50_ms" "$output")"
    p95="$(get_value "p95_ms" "$output")"
    p99="$(get_value "p99_ms" "$output")"

    local process_rss
    local container_current
    local container_peak

    process_rss="$(
        docker exec "$CURRENT_SERVER" \
            awk '/VmRSS:/ {print $2}' /proc/1/status
    )"

    container_current="$(
        docker exec "$CURRENT_SERVER" \
            cat /sys/fs/cgroup/memory.current
    )"

    container_peak="$(
        docker exec "$CURRENT_SERVER" \
            cat /sys/fs/cgroup/memory.peak
    )"

    process_rss="${process_rss:-0}"
    container_current="${container_current:-0}"
    container_peak="${container_peak:-0}"

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$language" \
        "$repetition" \
        "$endpoint" \
        "$connections" \
        "$requests" \
        "${successful:-0}" \
        "${failed:-0}" \
        "${unattempted:-0}" \
        "${rps:-0}" \
        "${p50:-0}" \
        "${p95:-0}" \
        "${p99:-0}" \
        "$process_rss" \
        "$container_current" \
        "$container_peak" \
        >> "$RESULTS"

    log "$language $endpoint c=$connections n=$requests rep=$repetition: ${rps:-0} RPS, failed=${failed:-0}, unattempted=${unattempted:-0}"
}

# ============================================================
# Warmup
# ============================================================

warmup() {
    local language="$1"

    log "Warming up $language"

    docker exec "$SERVER-load" \
        /bench/bin/load \
        -url "http://${SERVER}-${language}:${PORT}/hello" \
        -c 10 \
        -n 1000 \
        >/dev/null
}

# ============================================================
# Results header
# ============================================================

write_header() {
    printf '%s\n' \
        $'language\trepetition\tendpoint\tconnections\ttarget_requests\tsuccessful_requests\tfailed_requests\tunattempted_requests\trps\tp50_ms\tp95_ms\tp99_ms\tserver_process_rss_kb\tcontainer_current_kb\tcontainer_peak_kb' \
        > "$RESULTS"
}

# ============================================================
# Benchmark execution
# ============================================================

main() {

    log "Kode HTTP benchmark"
    log "Repetitions: $REPETITIONS"

    if ! [[ "$REPETITIONS" =~ ^[1-9][0-9]*$ ]]; then
        echo "ERROR: REPETITIONS must be a positive integer."
        exit 1
    fi

    if [[ -n "$KODE_HTTP_WORKERS" ]]; then

        if ! [[ "$KODE_HTTP_WORKERS" =~ ^[1-9][0-9]*$ ]]; then
            echo "ERROR: KODE_HTTP_WORKERS must be a positive integer."
            exit 1
        fi

        log "Kode worker override: $KODE_HTTP_WORKERS"

    else

        log "Kode worker override: default runtime setting"

    fi

    mkdir -p "$RESULT_DIR"

    ensure_network
    ensure_image

    start_load_generator
    write_header

    local repetition
    local language
    local endpoint
    local connections
    local requests

    for ((repetition = 1; repetition <= REPETITIONS; repetition++)); do

        log "============================================"
        log "Repetition $repetition / $REPETITIONS"
        log "============================================"

        # Rotate language order to reduce ordering bias.
        local offset=$(((repetition - 1) % ${#LANGUAGES[@]}))
        local ordered_languages=()

        for ((i = 0; i < ${#LANGUAGES[@]}; i++)); do
            ordered_languages+=("${LANGUAGES[$(((i + offset) % ${#LANGUAGES[@]}))]}")
        done

        for language in "${ordered_languages[@]}"; do

            start_server "$language"
            warmup "$language"

            for endpoint in "${ENDPOINTS[@]}"; do

                for connections in "${CONNECTIONS[@]}"; do

                    for requests in "${REQUESTS[@]}"; do

                        run_load \
                            "$language" \
                            "$endpoint" \
                            "$connections" \
                            "$requests" \
                            "$repetition"

                    done
                done
            done

            stop_server

        done
    done

    log "Generating summary"

    python3 benchmarks/http/summarize.py \
        "$RESULTS" \
        "$SUMMARY"

    log "============================================"
    log "Benchmark completed"
    log "Raw results: $RESULTS"
    log "Summary: $SUMMARY"
    log "============================================"
}

main "$@"

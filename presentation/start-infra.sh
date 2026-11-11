#!/usr/bin/env bash
# presentation/start-infra.sh — bring up all Lesson 14 infrastructure with one command.
#
#   presentation/start-infra.sh              # start everything (safe to re-run; skips healthy services)
#   presentation/start-infra.sh status       # show what is up/down
#   presentation/start-infra.sh stop         # stop everything
#   presentation/start-infra.sh restart      # stop, then start
#   presentation/start-infra.sh logs tutor   # tail one log: tutor|harrier|postgres|mlflow
#
# Services (documented commands from README "Commands to run before the workshops"):
#   tutor     Qwen3.6-35B-A3B tutor model   http://127.0.0.1:8080  (provider JSON base_url)
#   harrier   Harrier 0.6B embedding model  http://127.0.0.1:9999  (Lesson 12-14 retrieval)
#   postgres  pgvector with the test corpus baked into the image, 127.0.0.1:55432
#   mlflow    MLflow v3.17.1 tracking server, http://127.0.0.1:5000 (volume gdk-dev-mlflow)
#
# PID files and model logs live in GDK_INFRA_STATE_DIR (default /tmp/gdk-presentation-infra).
# Notes:
#   - The postgres container is created with --rm, so `stop` removes the container and the
#     next `start` recreates it from the image with the same baked-in corpus. Its data
#     lives in the image, so nothing is lost.
#   - MLflow data lives in the named volume gdk-dev-mlflow and survives stop/start.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
STATE_DIR=${GDK_INFRA_STATE_DIR:-/tmp/gdk-presentation-infra}
ENGINE=${CONTAINER_ENGINE:-podman}

MLFLOW_IMAGE=ghcr.io/mlflow/mlflow:v3.17.1@sha256:65aa7a75fe37554bd7dd9e539b58e1ec130dac99f27fef9756df316a272dfee2
PG_IMAGE=ghcr.io/thesteve0/gdk-harrier-embeddings-pg18:harrier-llamacpp-test-511-v1
PG_NAME=gdk-harrier-postgres
MLFLOW_NAME=gdk-dev-mlflow

TUTOR_PORT=8080
HARRIER_PORT=9999
PG_PORT=55432
MLFLOW_PORT=5000

TUTOR_WAIT=${TUTOR_WAIT:-420}     # seconds to wait for the 35B tutor to finish loading
HARRIER_WAIT=${HARRIER_WAIT:-120}
POSTGRES_WAIT=${POSTGRES_WAIT:-60}
MLFLOW_WAIT=${MLFLOW_WAIT:-120}

mkdir -p "$STATE_DIR"
chmod 700 "$STATE_DIR" 2>/dev/null || true

say()  { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; }
die()  { fail "$*"; exit 1; }

# --- small health helpers -----------------------------------------------------

# HTTP 200 check that ignores any proxy configuration.
http_ok() {
    curl --noproxy '*' --silent --max-time 5 -o /dev/null -w '%{http_code}' "$1" 2>/dev/null | grep -q '^200$'
}

# Plain TCP probe (used for PostgreSQL, which is not HTTP).
tcp_up() {
    (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null
}

container_exists()   { "$ENGINE" container inspect "$1" >/dev/null 2>&1; }
container_running()  { [ "$("$ENGINE" container inspect -f '{{.State.Running}}' "$1" 2>/dev/null)" = "true" ]; }

pid_alive() {
    local pid=$1
    [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null || return 1
    # Guard against PID reuse after a reboot: only trust processes named llama*.
    [ -r "/proc/$pid/comm" ] && grep -qi '^llama' "/proc/$pid/comm"
}

wait_http() {
    local name=$1 url=$2 timeout=$3 elapsed=0
    until http_ok "$url"; do
        elapsed=$((elapsed + 2))
        if [ "$elapsed" -ge "$timeout" ]; then
            fail "$name did not become healthy within ${timeout}s"
            return 1
        fi
        printf '.'
        sleep 2
    done
    say " ready (${elapsed}s)"
}

wait_tcp() {
    local name=$1 port=$2 timeout=$3 elapsed=0
    until tcp_up "$port"; do
        elapsed=$((elapsed + 2))
        if [ "$elapsed" -ge "$timeout" ]; then
            fail "$name did not accept connections within ${timeout}s"
            return 1
        fi
        printf '.'
        sleep 2
    done
    say " ready (${elapsed}s)"
}

# --- process helpers ----------------------------------------------------------

# Start a long-running command in the background with nohup, logging to a file.
start_background() {
    local name=$1; shift
    local logfile="$STATE_DIR/$name.log"
    local pidfile="$STATE_DIR/$name.pid"
    : >"$logfile"
    nohup "$@" >>"$logfile" 2>&1 &
    echo $! >"$pidfile"
}

stop_pidfile() {
    local name=$1 pidfile="$STATE_DIR/$1.pid" pid
    [ -f "$pidfile" ] || return 0
    pid=$(cat "$pidfile" 2>/dev/null || true)
    if pid_alive "$pid"; then
        kill -TERM "$pid" 2>/dev/null || true
        local i=0
        while pid_alive "$pid" && [ "$i" -lt 20 ]; do sleep 0.5; i=$((i + 1)); done
        if pid_alive "$pid"; then
            fail "$name did not exit after SIGTERM; sending SIGKILL"
            kill -KILL "$pid" 2>/dev/null || true
        fi
        say "  $name stopped (pid $pid)"
    else
        say "  $name was not running (stale pid file)"
    fi
    rm -f "$pidfile"
}

# --- per-service checks used by `status` --------------------------------------

service_tutor() {
    if pid_alive "$(cat "$STATE_DIR/tutor.pid" 2>/dev/null || true)" || http_ok "http://127.0.0.1:${TUTOR_PORT}/health"; then
        if http_ok "http://127.0.0.1:${TUTOR_PORT}/health"; then
            say "  tutor     UP    http://127.0.0.1:${TUTOR_PORT}  (qwen3.6-35b-a3b)"
            return 0
        fi
        say "  tutor     STARTING (process alive, /health not ready yet)"
        return 1
    fi
    say "  tutor     DOWN  http://127.0.0.1:${TUTOR_PORT}"
    return 1
}

service_harrier() {
    if pid_alive "$(cat "$STATE_DIR/harrier.pid" 2>/dev/null || true)" || http_ok "http://127.0.0.1:${HARRIER_PORT}/health"; then
        if http_ok "http://127.0.0.1:${HARRIER_PORT}/health"; then
            say "  harrier   UP    http://127.0.0.1:${HARRIER_PORT}  (harrier-oss-v1-0.6b embeddings)"
            return 0
        fi
        say "  harrier   STARTING (process alive, /health not ready yet)"
        return 1
    fi
    say "  harrier   DOWN  http://127.0.0.1:${HARRIER_PORT}"
    return 1
}

service_postgres() {
    local status=1
    if tcp_up "$PG_PORT"; then
        if container_running "$PG_NAME"; then
            say "  postgres  UP    127.0.0.1:${PG_PORT}  (container $PG_NAME)"
        else
            say "  postgres  UP    127.0.0.1:${PG_PORT}  (port served outside $PG_NAME)"
        fi
        status=0
    else
        say "  postgres  DOWN  127.0.0.1:${PG_PORT}  (container $PG_NAME)"
    fi
    return "$status"
}

service_mlflow() {
    if http_ok "http://127.0.0.1:${MLFLOW_PORT}/health"; then
        say "  mlflow    UP    http://127.0.0.1:${MLFLOW_PORT}  (UI: http://127.0.0.1:${MLFLOW_PORT})"
        return 0
    fi
    say "  mlflow    DOWN  http://127.0.0.1:${MLFLOW_PORT}  (container $MLFLOW_NAME)"
    return 1
}

# --- start --------------------------------------------------------------------

start_postgres() {
    printf '[1/4] PostgreSQL pgvector (port %s)   ' "$PG_PORT"
    if tcp_up "$PG_PORT"; then
        say "already serving on ${PG_PORT}; leaving it untouched"
    elif container_running "$PG_NAME"; then
        say "container reported running; waiting for readiness"
        wait_tcp postgres "$PG_PORT" "$POSTGRES_WAIT"
    elif container_exists "$PG_NAME"; then
        "$ENGINE" start "$PG_NAME" >/dev/null
        wait_tcp postgres "$PG_PORT" "$POSTGRES_WAIT"
    else
        # Data is baked into this image (both draft collections, 511+511 passages).
        if ! "$ENGINE" run -d --rm --name "$PG_NAME" \
            -p "127.0.0.1:${PG_PORT}:5432" \
            "$PG_IMAGE" >"$STATE_DIR/postgres-create.log" 2>&1; then
            fail "could not create container $PG_NAME (see $STATE_DIR/postgres-create.log)"
            return 1
        fi
        wait_tcp postgres "$PG_PORT" "$POSTGRES_WAIT"
    fi
    if container_running "$PG_NAME"; then
        local counts
        counts=""
        local attempt=0
        while [ -z "$counts" ] && [ "$attempt" -lt 10 ]; do
            counts=$(timeout 15 "$ENGINE" exec "$PG_NAME" psql -X -U retrieval_reader -d harrier -tAc \
                "SELECT string_agg(collection_id || '=' || chunk_count, ', ' ORDER BY collection_id) FROM retrieval.collections;" 2>/dev/null || true)
            [ -n "$counts" ] || { attempt=$((attempt + 1)); sleep 1; }
        done
        if [ -n "$counts" ]; then
            say "      corpus in database: $counts"
            case "$counts" in
                *511*) : ;;
                *) fail "expected the 511-passage test collections; verify the database contents" ;;
            esac
        else
            fail "could not query the corpus with the retrieval_reader role"
        fi
    fi
}

start_mlflow() {
    printf '[2/4] MLflow tracking server (port %s)  ' "$MLFLOW_PORT"
    if http_ok "http://127.0.0.1:${MLFLOW_PORT}/health"; then
        say "already running"
        return 0
    fi
    if container_exists "$MLFLOW_NAME"; then
        "$ENGINE" start "$MLFLOW_NAME" >/dev/null
        say "started existing container $MLFLOW_NAME"
    else
        # Same pinned image, loopback port and named volume as the course MLflow setup.
        "$ENGINE" run -d --name "$MLFLOW_NAME" \
            --label gdk.presentation-infra=v1 \
            -p "127.0.0.1:${MLFLOW_PORT}:5000" \
            -v gdk-dev-mlflow:/mlflow \
            "$MLFLOW_IMAGE" \
            mlflow server --host 0.0.0.0 --port 5000 --workers 1 \
            --backend-store-uri sqlite:////mlflow/mlflow.db \
            --artifacts-destination /mlflow/artifacts --serve-artifacts \
            >"$STATE_DIR/mlflow-create.log" 2>&1
        say "created container $MLFLOW_NAME (volume gdk-dev-mlflow)"
    fi
    wait_http mlflow "http://127.0.0.1:${MLFLOW_PORT}/health" "$MLFLOW_WAIT"
}

start_tutor() {
    printf '[3/4] Tutor model qwen3.6-35b-a3b (port %s) ' "$TUTOR_PORT"
    if http_ok "http://127.0.0.1:${TUTOR_PORT}/health"; then
        say "already running"
        return 0
    fi
    local pidfile="$STATE_DIR/tutor.pid"
    if pid_alive "$(cat "$pidfile" 2>/dev/null || true)"; then
        say "process already loading; waiting for /health"
    else
        local llama_bin
        llama_bin=$(command -v llama) || die "llama (llama.cpp CLI) not found on PATH"
        start_background tutor "$llama_bin" serve \
            -hf unsloth/Qwen3.6-35B-A3B-GGUF:UD-Q6_K_XL \
            --alias qwen3.6-35b-a3b \
            --host 127.0.0.1 --port "$TUTOR_PORT" \
            --jinja --reasoning-format deepseek \
            --temp 1.0 --top-p 0.95 --top-k 20 --min-p 0.0 \
            --presence-penalty 1.5 --repeat-penalty 1.0 \
            --gpu-layers all --kv-offload --flash-attn auto \
            --ctx-size 131072 --parallel 1 \
            --batch-size 2048 --ubatch-size 512 \
            --cache-prompt --metrics
        say "launched (pid $(cat "$pidfile"), log $STATE_DIR/tutor.log)"
    fi
    wait_http tutor "http://127.0.0.1:${TUTOR_PORT}/health" "$TUTOR_WAIT" || {
        fail "see $STATE_DIR/tutor.log for the tutor error output"
        return 1
    }
}

start_harrier() {
    printf '[4/4] Harrier embedding model (port %s) ' "$HARRIER_PORT"
    if http_ok "http://127.0.0.1:${HARRIER_PORT}/health"; then
        say "already running"
        return 0
    fi
    local pidfile="$STATE_DIR/harrier.pid"
    if pid_alive "$(cat "$pidfile" 2>/dev/null || true)"; then
        say "process already loading; waiting for /health"
    else
        local server_bin
        server_bin=$(command -v llama-server) || die "llama-server not found on PATH"
        start_background harrier "$server_bin" \
            -hf TheSteve0/harrier-oss-v1-0.6b-GGUF \
            --embeddings --pooling last \
            -ngl 99 -c 16384 -np 8 -fa on -b 2048 -ub 2048 \
            --host 0.0.0.0 --port "$HARRIER_PORT"
        say "launched (pid $(cat "$pidfile"), log $STATE_DIR/harrier.log)"
    fi
    wait_http harrier "http://127.0.0.1:${HARRIER_PORT}/health" "$HARRIER_WAIT" || {
        fail "see $STATE_DIR/harrier.log for the harrier error output"
        return 1
    }
}

cmd_start() {
    command -v curl >/dev/null || die "curl is required"
    command -v "$ENGINE" >/dev/null || die "container engine '$ENGINE' not found"
    say "Starting Lesson 14 infrastructure (state dir: $STATE_DIR)"
    local failures=0
    start_postgres || failures=$((failures + 1))
    start_mlflow || failures=$((failures + 1))
    start_tutor || failures=$((failures + 1))
    start_harrier || failures=$((failures + 1))
    say ""
    if [ "$failures" -eq 0 ]; then
        say "All Lesson 14 infrastructure is up."
        say "  Tutor provider JSON : custom_aa_llama_qwen3_6-35b.json (base_url http://127.0.0.1:${TUTOR_PORT})"
        say "  MLflow UI           : http://127.0.0.1:${MLFLOW_PORT} (pick/create an experiment, then export MLFLOW_EXPERIMENT_ID)"
        say "  Logs                : $STATE_DIR/{tutor,harrier}.log"
        say "  Stop everything     : presentation/start-infra.sh stop"
    else
        fail "$failures service(s) failed to start; run 'presentation/start-infra.sh status' and check $STATE_DIR/*.log"
        return 1
    fi
}

# --- stop ---------------------------------------------------------------------

cmd_stop() {
    say "Stopping Lesson 14 infrastructure"
    stop_pidfile harrier
    stop_pidfile tutor
    for name in "$PG_NAME" "$MLFLOW_NAME"; do
        if container_exists "$name" && container_running "$name"; then
            "$ENGINE" stop -t 10 "$name" >/dev/null
            say "  container $name stopped"
        else
            say "  container $name not running"
        fi
    done
    # gdk-harrier-postgres runs with --rm: stopping it also removes it (data is in the image).
    say "Done. Re-run presentation/start-infra.sh to start everything again."
}

# --- status -------------------------------------------------------------------

cmd_status() {
    local failures=0
    say "Lesson 14 infrastructure status:"
    service_postgres || failures=$((failures + 1))
    service_mlflow || failures=$((failures + 1))
    service_tutor || failures=$((failures + 1))
    service_harrier || failures=$((failures + 1))
    if [ "$failures" -ne 0 ]; then
        fail "$failures service(s) down"
        return 1
    fi
}

# --- logs ---------------------------------------------------------------------

cmd_logs() {
    local name=${1:-}
    case "$name" in
        tutor | harrier)
            [ -f "$STATE_DIR/$name.log" ] || die "no log for '$name' yet (started? logs live in $STATE_DIR)"
            tail -n 60 "$STATE_DIR/$name.log"
            say "(full log: $STATE_DIR/$name.log)"
            ;;
        postgres)
            container_exists "$PG_NAME" || die "container $PG_NAME does not exist"
            "$ENGINE" logs --tail 60 "$PG_NAME"
            ;;
        mlflow)
            container_exists "$MLFLOW_NAME" || die "container $MLFLOW_NAME does not exist"
            "$ENGINE" logs --tail 60 "$MLFLOW_NAME"
            ;;
        *)
            die "usage: $0 logs tutor|harrier|postgres|mlflow"
            ;;
    esac
}

# --- entry point ---------------------------------------------------------------

case "${1:-start}" in
    start) cmd_start ;;
    stop) cmd_stop ;;
    status) cmd_status ;;
    restart) cmd_stop; cmd_start ;;
    logs) shift; cmd_logs "${1:-}" ;;
    *) die "usage: $0 [start|stop|status|restart|logs [tutor|harrier|postgres|mlflow]]" ;;
esac

#!/usr/bin/env bash
# Docker-compatible local PostgreSQL lifecycle; CONTAINER_ENGINE=podman also supported.
set -euo pipefail
umask 077
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)
ENGINE=${CONTAINER_ENGINE:-docker}
NAME=gdk-harrier-postgres
IMAGE=docker.io/pgvector/pgvector@sha256:60cddfb66e0c6c077f33a81c974ad12f688f2d26e055d38cd696004af7911cd8
PRIVATE="$ROOT/day_trading/embeddings/postgres-local"
ENVFILE="$PRIVATE/container.env"
PORT=${PGVECTOR_PORT:-55432}
command -v "$ENGINE" >/dev/null

check_container() {
    local label
    label=$("$ENGINE" inspect --format '{{ index .Config.Labels "gdk.harrier.preparation" }}' "$NAME")
    [[ "$label" == v1 ]] || { echo 'Refusing unrelated container with matching name' >&2; exit 1; }
}

case "${1:-up}" in
    up)
        [[ ! -L "$PRIVATE" ]] || { echo 'Refusing symlink private directory' >&2; exit 1; }
        mkdir -p "$PRIVATE"
        if [[ ! -e "$ENVFILE" ]]; then
            ADMIN=$(openssl rand -hex 24)
            READER=$(openssl rand -hex 24)
            printf 'POSTGRES_USER=harrier_admin\nPOSTGRES_DB=harrier\nPOSTGRES_PASSWORD=%s\nREADER_PASSWORD=%s\n' "$ADMIN" "$READER" > "$ENVFILE"
            printf 'export DATABASE_URL=postgresql://harrier_admin:%s@127.0.0.1:%s/harrier\nexport READER_DATABASE_URL=postgresql://retrieval_reader:%s@127.0.0.1:%s/harrier\n' "$ADMIN" "$PORT" "$READER" "$PORT" > "$PRIVATE/connection.env"
        fi
        [[ -f "$ENVFILE" && ! -L "$ENVFILE" ]] || { echo 'Invalid private environment file' >&2; exit 1; }
        if "$ENGINE" container inspect "$NAME" >/dev/null 2>&1; then
            check_container
            "$ENGINE" start "$NAME" >/dev/null
        else
            "$ENGINE" run -d --name "$NAME" --label gdk.harrier.preparation=v1 \
                --env-file "$ENVFILE" -p "127.0.0.1:$PORT:5432" \
                -v gdk-harrier-pg18:/var/lib/postgresql \
                --health-cmd='pg_isready -U harrier_admin -d harrier' \
                --health-interval=2s --health-timeout=3s --health-retries=30 \
                "$IMAGE" >/dev/null
        fi
        READY=false
        for ((i=0; i<60; i++)); do
            if "$ENGINE" exec "$NAME" pg_isready -U harrier_admin -d harrier >/dev/null 2>&1; then READY=true; break; fi
            sleep 1
        done
        "$READY" || { echo 'PostgreSQL readiness timeout; inspect container logs locally' >&2; exit 1; }
        "$ENGINE" exec -i "$NAME" psql -X -v ON_ERROR_STOP=1 -U harrier_admin -d harrier < "$HERE/sql/schema.sql"
        # Resolve the reader secret inside the container without printing it or argv disclosure.
        "$ENGINE" exec -i "$NAME" psql -X -q -v ON_ERROR_STOP=1 -U harrier_admin -d harrier <<'SQL'
\getenv reader_password READER_PASSWORD
SELECT format('ALTER ROLE retrieval_reader PASSWORD %L', :'reader_password') \gexec
-- PUBLIC TEMP permits writes even for a table-reader role; keep the supplied store read-only.
REVOKE TEMPORARY ON DATABASE harrier FROM PUBLIC;
GRANT CONNECT ON DATABASE harrier TO retrieval_reader;
SQL
        echo "PostgreSQL ready on 127.0.0.1:$PORT. Source private connection.env to use Rust CLI."
        ;;
    stop) check_container; "$ENGINE" stop "$NAME" ;;
    status) check_container; "$ENGINE" exec "$NAME" psql -X -U harrier_admin -d harrier -c 'SELECT version(); SELECT extversion FROM pg_extension WHERE extname = '\''vector'\''; SELECT collection_id, chunk_count, model, dimensions FROM retrieval.collections;' ;;
    sql) check_container; "$ENGINE" exec -i "$NAME" psql -X -v ON_ERROR_STOP=1 -U harrier_admin -d harrier "${@:2}" ;;
    *) echo 'Usage: postgres.sh {up|stop|status|sql [psql arguments]}' >&2; exit 2 ;;
esac

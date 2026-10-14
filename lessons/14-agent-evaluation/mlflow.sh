#!/usr/bin/env bash
# Supplied infrastructure only. Run from any directory; never touches retrieval services.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
ENGINE=${CONTAINER_ENGINE:-podman}
IMAGE=ghcr.io/mlflow/mlflow:v3.17.1@sha256:65aa7a75fe37554bd7dd9e539b58e1ec130dac99f27fef9756df316a272dfee2
NAME=gdk-mlflow
DATA="$ROOT/.evaluation/mlflow"
case "${1:-start}" in
  start)
    mkdir -p "$DATA"
    if "$ENGINE" container inspect "$NAME" >/dev/null 2>&1; then
      "$ENGINE" start "$NAME"
    else
      "$ENGINE" run -d --name "$NAME" \
        -p 127.0.0.1:5000:5000 -v "$DATA:/mlflow:Z" \
        "$IMAGE" mlflow server --host 0.0.0.0 --port 5000 --workers 1 \
        --backend-store-uri sqlite:////mlflow/mlflow.db \
        --artifacts-destination /mlflow/artifacts --serve-artifacts
    fi
    ;;
  stop) "$ENGINE" stop "$NAME" ;;
  restart) "$ENGINE" restart "$NAME" ;;
  logs) "$ENGINE" logs "$NAME" ;;
  remove) "$ENGINE" rm -f "$NAME" ;; # leaves all SQLite/artifacts intact
  *) echo 'Usage: mlflow.sh start|stop|restart|logs|remove' >&2; exit 2 ;;
esac

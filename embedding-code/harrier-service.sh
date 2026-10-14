#!/usr/bin/env bash
# Local reference runtime. Foreground server: stop with Ctrl-C; no remote inference.
set -euo pipefail
umask 077
HERE=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)
RUNTIME="$ROOT/day_trading/model-preparation/harrier-runtime"
case "${1:-serve}" in
    install)
        command -v uv >/dev/null
        if [[ ! -d "$RUNTIME" ]]; then uv venv --python 3.11 "$RUNTIME"; fi
        uv pip install --python "$RUNTIME/bin/python3" --index-strategy unsafe-best-match \
            --extra-index-url https://download.pytorch.org/whl/cpu \
            -r "$HERE/requirements-harrier.txt"
        ;;
    serve)
        [[ -x "$RUNTIME/bin/python3" ]] || { echo 'Run harrier-service.sh install first' >&2; exit 1; }
        export HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 HF_HUB_DISABLE_TELEMETRY=1
        exec "$RUNTIME/bin/python3" "$HERE/harrier_server.py" "${@:2}"
        ;;
    *) echo 'Usage: harrier-service.sh {install|serve [server options]}' >&2; exit 2 ;;
esac

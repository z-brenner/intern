#!/usr/bin/env bash
# Starts llama-server with the app's flags plus extra ones, replays captured requests against
# it (replay-requests.py), and stops it: one server-side speed measurement whose replies can be
# compared byte for byte with a reference run.
#
# Laboratory tooling; the app does not call it. Usage:
#   LLAMA_SERVER=... MODEL=... WIRE="dir [dir ...]" [THREADS=4] [PORT=18120] [CONCURRENCY=1]
#   [REFERENCE=dir-or-json] [LIMIT=n] [EVERY=k] [STRIP_LOGPROBS=1] [CTX=8192] [PARALLEL=1] \
#     scripts/bonsai-eval/replay-server.sh OUT_DIR [extra llama-server flags ...]
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${1:?output directory}"; shift
: "${LLAMA_SERVER:?}" "${MODEL:?}" "${WIRE:?}"
THREADS="${THREADS:-4}"; PORT="${PORT:-18120}"; CONCURRENCY="${CONCURRENCY:-1}"
CTX="${CTX:-8192}"; PARALLEL="${PARALLEL:-1}"
mkdir -p "$OUT"
# The app's launch (crates/intern-engine/src/server.rs arguments_for + MEASURED_FLAGS), no API key.
ARGS=(--host 127.0.0.1 --port "$PORT" --model "$MODEL" --parallel "$PARALLEL" --ctx-size "$CTX"
  --n-gpu-layers 0 --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui
  --cache-ram 0 -ub 256 --prio -1 --poll 0 --no-mmproj "$@")
echo "$LLAMA_SERVER ${ARGS[*]}" > "$OUT/server-command.txt"
"$LLAMA_SERVER" "${ARGS[@]}" > "$OUT/llama-server.log" 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null; wait $SERVER_PID 2>/dev/null || true' EXIT
for _ in $(seq 1 240); do
  curl -sf "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 && break
  kill -0 $SERVER_PID 2>/dev/null || { echo "server exited; see $OUT/llama-server.log" >&2; exit 1; }
  sleep 0.5
done
WIRE_ARGS=(); for w in $WIRE; do WIRE_ARGS+=(--wire "$w"); done
# Each capture's first request is its warm-up: it is skipped, and replayed untimed once per slot
# so every slot starts with the instructions cached, as in an app that has read a document.
python3 -I "$REPO/scripts/bonsai-eval/replay-requests.py" --server "http://127.0.0.1:$PORT" \
  "${WIRE_ARGS[@]}" --skip-first --warm-up --every "${EVERY:-1}" ${STRIP_LOGPROBS:+--strip-logprobs} \
  --concurrency "$CONCURRENCY" ${LIMIT:+--limit "$LIMIT"} \
  ${REFERENCE:+--reference "$REFERENCE"} --out "$OUT/replay.json"
grep -E "draft acceptance|statistics" "$OUT/llama-server.log" | tail -3 || true

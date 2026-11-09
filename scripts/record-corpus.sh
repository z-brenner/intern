#!/usr/bin/env bash
# Records the fixture corpus live on Linux, as scripts/record-corpus.ps1 does
# on Windows: starts llama-server with the flags the desktop app uses (one
# slot, CPU only, 8,192-token context, the model's own chat template, no web
# UI, no projector), waits for it to be healthy, runs `intern-evaluate` over
# fixtures/generated with the token probabilities the review gate reads
# (`--token-confidence`), records every reply, writes the baseline at the
# accurate model's gate (`--min-token-confidence 0.7`, as CI replays it), and
# stops the server however the run ends. See docs/evaluation.md.
#
# Usage:
#   MODEL=/path/to/model.gguf LLAMA_SERVER=/path/to/llama-server \
#   INTERN_RUNTIME_DIR=/path/to/runtime scripts/record-corpus.sh [OUT_DIR]
#
# Environment (all optional except MODEL and LLAMA_SERVER):
#   MODEL               the GGUF the app downloads (required)
#   LLAMA_SERVER        the llama-server binary (required)
#   INTERN_RUNTIME_DIR  what the worker reads PDFium, PP-OCR and tessdata from
#   INTERN_WORKER       the parser worker (default: target/release/intern-worker)
#   INTERN_EVALUATE     the evaluator (default: built with cargo --release)
#   THREADS             model threads (default: the app's, half the logical cores, 2..12)
#   PORT                server port (default: 18091)
#   NOTE                appended to the recording's note
#   LLAMA_SERVER_ARGS   extra llama-server flags appended after the app's own
#                       (laboratory use); unset, the server starts as the app starts it
#   SERVER_MEMORY_FILE  if set, the server's VmHWM and VmRSS lines are written there
#   OUT_DIR             where corpus-recording.json, corpus-baseline.json,
#                       report.json and the server log go (default: target/corpus-recording)
#
# The fixtures are generated again first (npm run fixtures), so what is
# recorded is what fixtures/manifest.json describes. Commit the recording and
# the baseline only from a run of the pinned model; the server's API key is
# generated here and never printed.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-${OUT_DIR:-$REPO/target/corpus-recording}}"
MODEL="${MODEL:?set MODEL to the GGUF model file}"
LLAMA_SERVER="${LLAMA_SERVER:?set LLAMA_SERVER to the llama-server binary}"
WORKER="${INTERN_WORKER:-$REPO/target/release/intern-worker}"
PORT="${PORT:-18091}"
if [ -z "${THREADS:-}" ]; then
  LOGICAL="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 8)"
  THREADS=$(( LOGICAL / 2 ))
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 12 ] && THREADS=12
fi

[ -f "$MODEL" ] || { echo "no model at $MODEL" >&2; exit 1; }
[ -x "$LLAMA_SERVER" ] || { echo "no llama-server at $LLAMA_SERVER" >&2; exit 1; }
[ -x "$WORKER" ] || { echo "no worker at $WORKER (cargo build --release --locked -p intern-worker --features windows-native)" >&2; exit 1; }

if [ -z "${INTERN_EVALUATE:-}" ]; then
  (cd "$REPO" && cargo build --release --locked -p intern-engine --bin intern-evaluate)
  TARGET="${CARGO_TARGET_DIR:-target}"
  case "$TARGET" in /*) ;; *) TARGET="$REPO/$TARGET" ;; esac
  INTERN_EVALUATE="$TARGET/release/intern-evaluate"
fi
(cd "$REPO" && npm run --silent fixtures)
mkdir -p "$OUT_DIR"

read -r -a EXTRA_SERVER_ARGS <<< "${LLAMA_SERVER_ARGS:-}"
KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
"$LLAMA_SERVER" --host 127.0.0.1 --port "$PORT" --api-key "$KEY" --model "$MODEL" \
  --parallel 1 --ctx-size 8192 --n-gpu-layers 0 \
  --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui --no-mmproj \
  --cache-ram 0 -ub 256 --prio -1 --poll 0 \
  ${EXTRA_SERVER_ARGS[@]+"${EXTRA_SERVER_ARGS[@]}"} \
  > "$OUT_DIR/llama-server.log" 2>&1 &
SERVER=$!
trap 'kill "$SERVER" 2>/dev/null || true; wait "$SERVER" 2>/dev/null || true' EXIT

for _ in $(seq 1 90); do
  if curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"'; then break; fi
  kill -0 "$SERVER" 2>/dev/null || { echo "llama-server exited; see $OUT_DIR/llama-server.log" >&2; exit 1; }
  sleep 2
done
curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"' || { echo "llama-server not healthy on port $PORT" >&2; exit 1; }

DESCRIPTION="$(basename "$MODEL") (sha256 $(sha256sum "$MODEL" | cut -c1-8)), llama-server $THREADS threads, 8192 context${LLAMA_SERVER_ARGS:+, $LLAMA_SERVER_ARGS}, $(uname -sm); routing worker; evidence pipeline, recorded with --token-confidence${NOTE:+; $NOTE}"
status=0
(cd "$REPO" && "$INTERN_EVALUATE" \
  --fixtures fixtures/generated --expected fixtures/expected.json \
  --worker "$WORKER" --endpoint "http://127.0.0.1:$PORT/v1/chat/completions" --api-key "$KEY" \
  --model-id intern-local --token-confidence --min-token-confidence 0.7 \
  --record "$OUT_DIR/corpus-recording.json" --write-baseline "$OUT_DIR/corpus-baseline.json" \
  --output "$OUT_DIR/report.json" --note "$DESCRIPTION") || status=$?
if [ -n "${SERVER_MEMORY_FILE:-}" ]; then
  grep -E '^VmHWM|^VmRSS' "/proc/$SERVER/status" > "$SERVER_MEMORY_FILE" 2>/dev/null || true
fi
echo "recording: $OUT_DIR/corpus-recording.json (baseline: $OUT_DIR/corpus-baseline.json, report: $OUT_DIR/report.json)" >&2
exit "$status"

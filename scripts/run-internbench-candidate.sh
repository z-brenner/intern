#!/usr/bin/env bash
# Runs InternBench live against a candidate model/runtime: like
# scripts/run-internbench.sh (same app server flags, same corpus checks), plus
# what a model qualification needs and the app does not do:
#
#   LLAMA_SERVER_ARGS  extra llama-server flags appended after the app's own,
#                      e.g. "--cache-ram 0 -fa on -ub 256"
#   BENCH_PROFILE      a --bench-profile JSON (needs intern-bench built with
#                      --features benchmark-transport); the wire captures go
#                      to OUT_DIR/wire
#   NICE               niceness for llama-server (default 0)
#   SAMPLE_RSS=1       sample the RSS of llama-server, intern-worker and
#                      tesseract every 100 ms into OUT_DIR/rss.json
#
# Laboratory tooling: the app does not call it. Usage:
#   MODEL=... LLAMA_SERVER=... INTERN_RUNTIME_DIR=... INTERN_BENCH=... \
#     scripts/run-internbench-candidate.sh OUT_DIR [-- intern-bench run args...]
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:?output directory}"
shift
if [ $# -gt 0 ] && [ "$1" = "--" ]; then
  shift
fi

MODEL="${MODEL:?set MODEL to the GGUF model file}"
LLAMA_SERVER="${LLAMA_SERVER:?set LLAMA_SERVER to the llama-server binary}"
INTERN_BENCH="${INTERN_BENCH:?set INTERN_BENCH to an intern-bench binary}"
WORKER="${INTERN_WORKER:-$REPO/target/release/intern-worker}"
PORT="${PORT:-18090}"
CORPUS="${CORPUS:-$REPO/bench/generated}"
GOLD="${GOLD:-$REPO/bench/gold.json}"
MANIFEST="${MANIFEST-$REPO/bench/manifest.json}"
NICE="${NICE:-0}"
if [ -z "${THREADS:-}" ]; then
  LOGICAL="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 8)"
  THREADS=$(( LOGICAL / 2 ))
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 12 ] && THREADS=12
fi
[ -f "$MODEL" ] || { echo "no model at $MODEL" >&2; exit 1; }
[ -x "$LLAMA_SERVER" ] || { echo "no llama-server at $LLAMA_SERVER" >&2; exit 1; }
[ -x "$WORKER" ] || { echo "no worker at $WORKER" >&2; exit 1; }
[ -x "$INTERN_BENCH" ] || { echo "no intern-bench at $INTERN_BENCH" >&2; exit 1; }
[ -e "$OUT_DIR" ] && { echo "$OUT_DIR exists; use a new directory" >&2; exit 1; }
mkdir -p "$OUT_DIR"

read -r -a EXTRA <<< "${LLAMA_SERVER_ARGS:-}"
KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
SERVER_ARGS=(--host 127.0.0.1 --port "$PORT" --api-key "$KEY" --model "$MODEL"
  --parallel 1 --ctx-size 8192 --n-gpu-layers 0
  --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui --no-mmproj
  ${EXTRA[@]+"${EXTRA[@]}"})
printf '%q ' "$LLAMA_SERVER" "${SERVER_ARGS[@]}" | sed "s/$KEY/<key>/" > "$OUT_DIR/server-command.txt"
echo >> "$OUT_DIR/server-command.txt"

SAMPLER=""
if [ "${SAMPLE_RSS:-0}" = 1 ]; then
  python3 -I "$REPO/scripts/rss-sampler.py" "$OUT_DIR/rss.json" \
    llama-server,intern-worker,tesseract,intern-bench &
  SAMPLER=$!
fi
START=$(date +%s.%N)
nice -n "$NICE" "$LLAMA_SERVER" "${SERVER_ARGS[@]}" > "$OUT_DIR/llama-server.log" 2>&1 &
SERVER=$!
cleanup() {
  kill "$SERVER" 2>/dev/null || true
  wait "$SERVER" 2>/dev/null || true
  if [ -n "$SAMPLER" ]; then kill "$SAMPLER" 2>/dev/null || true; wait "$SAMPLER" 2>/dev/null || true; fi
}
trap cleanup EXIT

for _ in $(seq 1 3000); do  # up to ~5 min: a cold 4B load from a slow disk takes > 60 s
  if curl -sf --noproxy '*' "http://127.0.0.1:$PORT/health" | grep -q '"ok"'; then break; fi
  kill -0 "$SERVER" 2>/dev/null || { echo "llama-server exited; see $OUT_DIR/llama-server.log" >&2; exit 1; }
  sleep 0.1
done
curl -sf --noproxy '*' "http://127.0.0.1:$PORT/health" | grep -q '"ok"' || { echo "llama-server not healthy" >&2; exit 1; }
echo "startup_s $(echo "$(date +%s.%N) - $START" | bc)" > "$OUT_DIR/startup.txt"

PROFILE_ARGUMENTS=()
if [ -n "${BENCH_PROFILE:-}" ]; then
  PROFILE_ARGUMENTS=(--bench-profile "$BENCH_PROFILE" --wire-output "$OUT_DIR/wire")
fi
MANIFEST_ARGUMENTS=()
if [ -n "$MANIFEST" ]; then
  MANIFEST_ARGUMENTS=(--manifest "$MANIFEST")
fi

status=0
"$INTERN_BENCH" run --corpus "$CORPUS" --gold "$GOLD" ${MANIFEST_ARGUMENTS[@]+"${MANIFEST_ARGUMENTS[@]}"} \
  --worker "$WORKER" --endpoint "http://127.0.0.1:$PORT/v1/chat/completions" --api-key "$KEY" \
  --model-id intern-local --model-path "$MODEL" --server-pid "$SERVER" \
  --note "llama-server $THREADS threads, 8192 context, ${LLAMA_SERVER_ARGS:-app flags}, $(uname -sm)" \
  ${PROFILE_ARGUMENTS[@]+"${PROFILE_ARGUMENTS[@]}"} \
  --record "$OUT_DIR/recording.json" --output "$OUT_DIR/report.json" --markdown "$OUT_DIR/report.md" \
  "$@" || status=$?
grep -E '^VmHWM|^VmRSS' "/proc/$SERVER/status" > "$OUT_DIR/server-memory.txt" 2>/dev/null || true
echo "report: $OUT_DIR/report.md (exit $status)" >&2
exit "$status"

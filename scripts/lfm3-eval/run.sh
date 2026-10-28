#!/usr/bin/env bash
# LFM3 evaluation for Intern, end to end on a Linux CPU host:
#   1. builds llama.cpp b11946 (the first release with LFM3, LFM3-Omni and d1)
#      from source, CPU only, the way a release archive dispatches CPU variants;
#   2. live InternBench, same runtime and app flags for every model:
#      Qwen3.5-2B (production file; the baseline), LFM3-Omni-2B (the candidate
#      replacement, text only) and LFM3-1.7B (Omni's text backbone, as a control);
#   3. d1-2B as a review gate over the names the Qwen run marked ready.
# Each step is resumable and skipped when its output exists.
#
# Laboratory tooling; the app does not call it. Usage:
#   S=/scratch scripts/lfm3-eval/run.sh
# $S/cache/lfm3/*.gguf and $S/cache/qwen-prod/*.gguf must be present
# (see cache/lfm3/fetch.sh for the pinned, verified downloads).
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
S="${S:?set S to the scratch directory}"
THREADS="${THREADS:-4}"
TAG=b11946
COMMIT=8ca4060b7e30e76baaa5670ee1ec1b760b7f8552
SRC="$S/build/llama-lfm3"
BUILD="$S/build/$TAG-portable"
OUT="$S/results/lfm3"
mkdir -p "$OUT"

# 1. runtime
if [ ! -x "$BUILD/bin/llama-server" ]; then
  [ -d "$SRC/.git" ] || git clone --filter=blob:none https://github.com/ggml-org/llama.cpp "$SRC"
  git -C "$SRC" fetch -q origin tag "$TAG" || true
  git -C "$SRC" checkout -q --detach "$COMMIT"
  [ "$(git -C "$SRC" rev-parse HEAD)" = "$COMMIT" ]
  TARGETS="llama-server llama-bench" "$REPO/scripts/build-llama-cpu-linux.sh" "$SRC" "$BUILD" portable
fi
SERVER="$BUILD/bin/llama-server"
"$SERVER" --version 2>&1 | tail -1 | tee "$OUT/runtime-version.txt"

# 2. live InternBench, one model at a time (shares the CPU with nothing else)
APP_FLAGS="--cache-ram 0 -ub 256"
run_bench() { # name model extra-flags
  local name=$1 model=$2 extra=$3
  [ -f "$OUT/$name/report.json" ] && { echo "$name: done"; return; }
  MODEL="$model" LLAMA_SERVER="$SERVER" INTERN_BENCH="$REPO/target/release/intern-bench" \
    INTERN_RUNTIME_DIR="$S/runtime" THREADS="$THREADS" SAMPLE_RSS=1 \
    LLAMA_SERVER_ARGS="$APP_FLAGS $extra" \
    "$REPO/scripts/bonsai-eval/internbench-chunked.sh" "$OUT/$name"
}
QWEN="$(ls "$S"/cache/qwen-prod/*.gguf | head -1)"
run_bench qwen "$QWEN" ""
run_bench lfm3-omni-2b "$S/cache/lfm3/LFM3-Omni-2B-Q4_K_M.gguf" "--reasoning off"
run_bench lfm3-1.7b "$S/cache/lfm3/LFM3-1.7B-Q4_K_M.gguf" "--reasoning off"

# 3. d1 over the Qwen run's ready names (the production model's own proposals)
D1="${D1:-$S/cache/lfm3/d1-2B-Q8_0.gguf}"
D1_NAME="$(basename "$D1" .gguf)"
if [ ! -f "$OUT/$D1_NAME.md" ]; then
  PORT=18091
  "$SERVER" --host 127.0.0.1 --port $PORT --model "$D1" --ctx-size 16384 -b 8192 -ub 8192 \
    --threads "$THREADS" --threads-batch "$THREADS" --no-webui --no-mmproj > "$OUT/$D1_NAME-server.log" 2>&1 &
  D1PID=$!
  trap 'kill $D1PID 2>/dev/null || true' EXIT
  for _ in $(seq 1 600); do curl -sf --noproxy '*' "http://127.0.0.1:$PORT/health" | grep -q ok && break; sleep 0.5; done
  python3 -I "$REPO/scripts/lfm3-eval/d1_verifier.py" ask --server "http://127.0.0.1:$PORT" \
    --recording "$OUT/qwen/recording.json" --report "$OUT/qwen/report.json" --out "$OUT/$D1_NAME-answers.jsonl"
  grep -E '^VmHWM|^VmRSS' "/proc/$D1PID/status" > "$OUT/$D1_NAME-memory.txt" || true
  kill $D1PID; wait $D1PID 2>/dev/null || true
  python3 -I "$REPO/scripts/lfm3-eval/d1_verifier.py" analyze --answers "$OUT/$D1_NAME-answers.jsonl" \
    --report "$OUT/qwen/report.json" --markdown "$OUT/$D1_NAME.md"
fi
for name in qwen lfm3-omni-2b lfm3-1.7b; do
  echo "== $name"; grep -m3 -E 'Filename|unsafe|Sent to review' "$OUT/$name/report.md" || true
done

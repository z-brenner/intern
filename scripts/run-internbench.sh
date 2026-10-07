#!/usr/bin/env bash
# Runs InternBench live: starts llama-server with the flags the desktop app
# uses (one slot, CPU only, 8,192-token context, the model's own chat
# template, no web UI, no projector), waits for it to be healthy, runs
# `intern-bench run` over bench/generated with a recording, and stops the
# server however the run ends. See docs/internbench.md.
#
# Usage:
#   MODEL=/path/to/model.gguf LLAMA_SERVER=/path/to/llama-server \
#   INTERN_RUNTIME_DIR=/path/to/runtime scripts/run-internbench.sh [OUT_DIR] [-- intern-bench run args...]
#
# Environment (all optional except MODEL and LLAMA_SERVER):
#   MODEL               the GGUF the app downloads (required)
#   LLAMA_SERVER        the llama-server binary (required)
#   INTERN_RUNTIME_DIR  what the worker reads PDFium, Tesseract and tessdata from
#   INTERN_WORKER       the parser worker (default: target/release/intern-worker)
#   INTERN_BENCH        the benchmark (default: built with cargo --release)
#   THREADS             model threads (default: the app's, half the logical cores, 2..12)
#   PORT                server port (default: 18090)
#   CORPUS, GOLD        the corpus and its gold (default: bench/generated, bench/gold.json;
#                       the default corpus is generated again unless every file in it
#                       matches the committed bench/manifest.json, so a missing, partial
#                       (--only) or out-of-date corpus is never run)
#   MANIFEST            the manifest the documents are checked against (default:
#                       bench/manifest.json for the default corpus, none for another;
#                       set it empty to check nothing)
#   OUT_DIR             where report.json, report.md, recording.json and the
#                       server log go (default: target/internbench)
#
# Extra arguments after `--` go to `intern-bench run`, for example
# `-- --only invoice-date-in-table,scan-clean-lease-2p --baseline bench/baseline.json`.
# The server's API key is generated here and never printed.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${OUT_DIR:-$REPO/target/internbench}"
if [ $# -gt 0 ] && [ "$1" != "--" ]; then
  OUT_DIR="$1"
  shift
fi
if [ $# -gt 0 ] && [ "$1" = "--" ]; then
  shift
fi

MODEL="${MODEL:?set MODEL to the GGUF model file}"
LLAMA_SERVER="${LLAMA_SERVER:?set LLAMA_SERVER to the llama-server binary}"
WORKER="${INTERN_WORKER:-$REPO/target/release/intern-worker}"
PORT="${PORT:-18090}"
CORPUS="${CORPUS:-$REPO/bench/generated}"
GOLD="${GOLD:-$REPO/bench/gold.json}"
# The committed manifest vouches for the default corpus only; a corpus of
# your own brings its own, or none.
if [ -z "${MANIFEST+set}" ] && [ "$CORPUS" = "$REPO/bench/generated" ] && [ -f "$REPO/bench/manifest.json" ]; then
  MANIFEST="$REPO/bench/manifest.json"
fi
if [ -z "${THREADS:-}" ]; then
  LOGICAL="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 8)"
  THREADS=$(( LOGICAL / 2 ))
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 12 ] && THREADS=12
fi

[ -f "$MODEL" ] || { echo "no model at $MODEL" >&2; exit 1; }
[ -x "$LLAMA_SERVER" ] || { echo "no llama-server at $LLAMA_SERVER" >&2; exit 1; }
[ -x "$WORKER" ] || { echo "no worker at $WORKER (cargo build --release --locked -p intern-worker --features windows-native)" >&2; exit 1; }

if [ -z "${INTERN_BENCH:-}" ]; then
  (cd "$REPO" && cargo build --release --locked -p intern-bench)
  INTERN_BENCH="${CARGO_TARGET_DIR:-$REPO/target}/release/intern-bench"
fi
# Regenerate when any file differs from the committed manifest, not only
# when the generated manifest does: a file edited by hand leaves that
# manifest as it was. The runner refuses to start on a mismatch either way.
corpus_matches_manifest() {
  node -e '
    const { createHash } = require("node:crypto");
    const { existsSync, readFileSync } = require("node:fs");
    const { join } = require("node:path");
    const [corpus, manifest] = process.argv.slice(1);
    for (const { file, sha256 } of JSON.parse(readFileSync(manifest, "utf8")).files) {
      const path = join(corpus, file);
      if (!existsSync(path) || createHash("sha256").update(readFileSync(path)).digest("hex") !== sha256) process.exit(1);
    }
  ' "$CORPUS" "$REPO/bench/manifest.json"
}
if [ "$CORPUS" = "$REPO/bench/generated" ] && ! corpus_matches_manifest; then
  (cd "$REPO" && node bench/generate.mjs)
fi
mkdir -p "$OUT_DIR"

KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
"$LLAMA_SERVER" --host 127.0.0.1 --port "$PORT" --api-key "$KEY" --model "$MODEL" \
  --parallel 1 --ctx-size 8192 --n-gpu-layers 0 \
  --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui --no-mmproj \
  > "$OUT_DIR/llama-server.log" 2>&1 &
SERVER=$!
trap 'kill "$SERVER" 2>/dev/null || true; wait "$SERVER" 2>/dev/null || true' EXIT

for _ in $(seq 1 90); do
  if curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"'; then break; fi
  kill -0 "$SERVER" 2>/dev/null || { echo "llama-server exited; see $OUT_DIR/llama-server.log" >&2; exit 1; }
  sleep 2
done
curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"' || { echo "llama-server not healthy on port $PORT" >&2; exit 1; }

MANIFEST_ARGUMENTS=()
if [ -n "${MANIFEST:-}" ]; then
  MANIFEST_ARGUMENTS=(--manifest "$MANIFEST")
fi

status=0
"$INTERN_BENCH" run --corpus "$CORPUS" --gold "$GOLD" ${MANIFEST_ARGUMENTS[@]+"${MANIFEST_ARGUMENTS[@]}"} \
  --worker "$WORKER" --endpoint "http://127.0.0.1:$PORT/v1/chat/completions" --api-key "$KEY" \
  --model-id intern-local --model-path "$MODEL" --server-pid "$SERVER" \
  --note "llama-server $THREADS threads, 8192 context, $(uname -sm)" \
  --record "$OUT_DIR/recording.json" --output "$OUT_DIR/report.json" --markdown "$OUT_DIR/report.md" \
  "$@" || status=$?
echo "report: $OUT_DIR/report.md (JSON: $OUT_DIR/report.json, recording: $OUT_DIR/recording.json)" >&2
exit "$status"

#!/usr/bin/env bash
# Runs InternBench live: starts llama-server with the flags the desktop app
# uses (one slot, CPU only, 8,192-token context, the model's own chat
# template, no web UI, no projector, the app's thread count, and the flags
# the app adds for the model's tier, `--no-repack` for either), waits for it
# to be healthy, runs `intern-bench run` over bench/generated with a
# recording, and stops the server however the run ends. The one flag of the
# app's left out is its slot folder (`--slot-save-path`), where it keeps the
# instructions' state between launches. See docs/internbench.md.
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
#   THREADS             model threads (default: the app's, every logical processor,
#                       at least 2 and at most 8: crates/intern-engine/src/server.rs
#                       default_threads)
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
#   TIER_SERVER_FLAGS   the flags the app adds for the model's tier
#                       (tiers::tier_server_flags in crates/intern-engine/src/tiers.rs),
#                       after its own (default: --no-repack, which both tiers
#                       carry); set it empty to launch without them, e.g. to
#                       measure repacked weights against the app's launch
#   LLAMA_SERVER_ARGS   extra llama-server flags appended after the app's own
#                       and the tier's, for a trial of a flag the app does not
#                       set (laboratory use, as scripts/run-internbench-candidate.sh
#                       takes it); llama-server (b11946) takes the last of
#                       `--no-repack` and `--repack`, so `--repack` here also
#                       measures repacked weights. Unset, the server starts
#                       with the app's flags
#   SERVER_MEMORY_FILE  if set, the server's VmHWM and VmRSS lines from
#                       /proc are written there once the run is over and
#                       before the server is stopped
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
# A path with its directory resolved, so `bench/generated`, a trailing slash
# or a symlinked checkout all name the default corpus the same way.
canonical() {
  local parent
  parent="$(cd "$(dirname "$1")" 2>/dev/null && pwd -P)" || { printf '%s\n' "$1"; return; }
  printf '%s/%s\n' "$parent" "$(basename "$1")"
}
DEFAULT_CORPUS="$(canonical "$REPO/bench/generated")"
CORPUS="$(canonical "${CORPUS:-$DEFAULT_CORPUS}")"
GOLD="${GOLD:-$REPO/bench/gold.json}"
# The committed manifest vouches for the default corpus only; a corpus of
# your own brings its own, or none.
if [ -z "${MANIFEST+set}" ] && [ "$CORPUS" = "$DEFAULT_CORPUS" ] && [ -f "$REPO/bench/manifest.json" ]; then
  MANIFEST="$REPO/bench/manifest.json"
fi
# The app's rule (server.rs default_threads): every logical processor, at
# least 2 and at most 8, and 4 when the count cannot be read.
if [ -z "${THREADS:-}" ]; then
  THREADS="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)"
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 8 ] && THREADS=8
fi
# The flags the app adds for the model's tier (tiers::tier_server_flags).
# Both tiers carry the same ones, so one default serves either model.
if [ -z "${TIER_SERVER_FLAGS+set}" ]; then
  TIER_SERVER_FLAGS="--no-repack"
fi

[ -f "$MODEL" ] || { echo "no model at $MODEL" >&2; exit 1; }
[ -x "$LLAMA_SERVER" ] || { echo "no llama-server at $LLAMA_SERVER" >&2; exit 1; }
[ -x "$WORKER" ] || { echo "no worker at $WORKER (cargo build --release --locked -p intern-worker --features windows-native)" >&2; exit 1; }

if [ -z "${INTERN_BENCH:-}" ]; then
  (cd "$REPO" && cargo build --release --locked -p intern-bench)
  # Cargo resolves a relative target directory from where it ran: here.
  TARGET="${CARGO_TARGET_DIR:-target}"
  case "$TARGET" in /*) ;; *) TARGET="$REPO/$TARGET" ;; esac
  INTERN_BENCH="$TARGET/release/intern-bench"
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
if [ "$CORPUS" = "$DEFAULT_CORPUS" ] && ! corpus_matches_manifest; then
  (cd "$REPO" && node bench/generate.mjs)
fi
mkdir -p "$OUT_DIR"

# The tier's flags follow the app's own, as the app adds them, and nothing
# more unless LLAMA_SERVER_ARGS is set, so by default the command line is the
# app's (its slot folder aside).
read -r -a TIER_FLAGS <<< "$TIER_SERVER_FLAGS"
read -r -a EXTRA_SERVER_ARGS <<< "${LLAMA_SERVER_ARGS:-}"
KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
"$LLAMA_SERVER" --host 127.0.0.1 --port "$PORT" --api-key "$KEY" --model "$MODEL" \
  --parallel 1 --ctx-size 8192 --n-gpu-layers 0 \
  --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui --no-mmproj \
  --cache-ram 0 -ub 256 --prio -1 --poll 0 \
  ${TIER_FLAGS[@]+"${TIER_FLAGS[@]}"} \
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

MANIFEST_ARGUMENTS=()
if [ -n "${MANIFEST:-}" ]; then
  MANIFEST_ARGUMENTS=(--manifest "$MANIFEST")
fi

status=0
"$INTERN_BENCH" run --corpus "$CORPUS" --gold "$GOLD" ${MANIFEST_ARGUMENTS[@]+"${MANIFEST_ARGUMENTS[@]}"} \
  --worker "$WORKER" --endpoint "http://127.0.0.1:$PORT/v1/chat/completions" --api-key "$KEY" \
  --model-id intern-local --model-path "$MODEL" --server-pid "$SERVER" \
  --note "llama-server $THREADS threads, 8192 context${TIER_SERVER_FLAGS:+, $TIER_SERVER_FLAGS}${LLAMA_SERVER_ARGS:+, $LLAMA_SERVER_ARGS}, $(uname -sm)" \
  --record "$OUT_DIR/recording.json" --output "$OUT_DIR/report.json" --markdown "$OUT_DIR/report.md" \
  "$@" || status=$?
# The server's peak, read while it is still running: the EXIT trap stops it.
if [ -n "${SERVER_MEMORY_FILE:-}" ]; then
  grep -E '^VmHWM|^VmRSS' "/proc/$SERVER/status" > "$SERVER_MEMORY_FILE" 2>/dev/null || true
fi
echo "report: $OUT_DIR/report.md (JSON: $OUT_DIR/report.json, recording: $OUT_DIR/recording.json)" >&2
exit "$status"

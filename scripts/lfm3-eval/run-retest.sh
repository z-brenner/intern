#!/usr/bin/env bash
# The LFM3 retest: run.sh measured LFM3-1.7B with thinking off, but it is a
# hybrid reasoning model whose card runs it with thinking on. This pass gives
# it (and Qwen3.5-2B, as the control) a bounded thinking block, and checks
# whether 4-bit quantization is what holds LFM3 back:
#   - thinking: think1024-profile.json turns thinking on with a 1,024-token
#     budget (llama-server then forces </think>) and wraps Intern's grammar so
#     the reply follows the thinking block (intern-bench's lab transport,
#     feature benchmark-transport). Unbounded thinking ran past 8,192 tokens
#     on both documents probed, so the budget is what makes it usable at all.
#   - quantization: LiquidAI's own QAD-Q4_0 and Q8_0 files, thinking off,
#     with the few-shot profile (LFM3's best thinking-off result).
#
# Laboratory tooling; the app does not call it. Run after run.sh, which builds
# the runtime; the binary must be built with the lab transport:
#   cargo build --release -p intern-bench --features benchmark-transport
# Usage: S=/scratch scripts/lfm3-eval/run-retest.sh [name ...]
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
S="${S:?set S to the scratch directory}"
SERVER="$S/build/b11946-portable/bin/llama-server"
OUT="$S/results/lfm3-retest"
PROFILE="$REPO/scripts/lfm3-eval/think1024-profile.json"
QWEN="$(ls "$S"/cache/qwen-prod/*.gguf | head -1)"
[ -x "$SERVER" ] || { echo "build the runtime first (scripts/lfm3-eval/run.sh)" >&2; exit 1; }
run_bench() { # name model server-flags profile bench-args
  local name=$1 model=$2 flags=$3 profile=$4 args=$5
  [ -f "$OUT/$name/report.json" ] && { echo "$name: done"; return; }
  MODEL="$model" LLAMA_SERVER="$SERVER" INTERN_BENCH="$REPO/target/release/intern-bench" \
    INTERN_RUNTIME_DIR="$S/runtime" THREADS="${THREADS:-4}" \
    LLAMA_SERVER_ARGS="--cache-ram 0 -ub 256 $flags" BENCH_PROFILE="$profile" BENCH_ARGS="$args" \
    "$REPO/scripts/bonsai-eval/internbench-chunked.sh" "$OUT/$name"
}
want() { [ $# -eq 0 ] || [[ " $* " == *" $NAME "* ]]; }
for NAME in lfm3-q4km-think1024 lfm3-qad-q4_0-fewshot lfm3-q8_0-fewshot qwen-think1024; do
  want "$@" || continue
  case $NAME in
    lfm3-q4km-think1024) run_bench "$NAME" "$S/cache/lfm3/LFM3-1.7B-Q4_K_M.gguf" "" "$PROFILE" "" ;;
    lfm3-qad-q4_0-fewshot) run_bench "$NAME" "$S/cache/lfm3/LFM3-1.7B-QAD-Q4_0.gguf" "--reasoning off" "" "--prompt-profile bonsai_2" ;;
    lfm3-q8_0-fewshot) run_bench "$NAME" "$S/cache/lfm3/LFM3-1.7B-Q8_0.gguf" "--reasoning off" "" "--prompt-profile bonsai_2" ;;
    qwen-think1024) run_bench "$NAME" "$QWEN" "" "$PROFILE" "" ;;
  esac
done

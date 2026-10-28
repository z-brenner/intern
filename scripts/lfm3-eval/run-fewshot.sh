#!/usr/bin/env bash
# Second pass for the LFM3 models: the same live InternBench run as run.sh,
# with the few-shot prompt profile Intern gives its quick-tier model
# (`bonsai_2`: the production instructions word for word plus one neutral
# worked exchange and a fixed seed). Without it both LFM3 models copied the
# instructions' own words into document_type ("what the document is",
# "Document") - the failure that profile was built for in Bonsai 2B.
#
# Laboratory tooling; the app does not call it. Run after run.sh, which builds
# the runtime. Usage: S=/scratch scripts/lfm3-eval/run-fewshot.sh
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
S="${S:?set S to the scratch directory}"
SERVER="$S/build/b11946-portable/bin/llama-server"
OUT="$S/results/lfm3"
[ -x "$SERVER" ] || { echo "build the runtime first (scripts/lfm3-eval/run.sh)" >&2; exit 1; }
run_bench() { # name model
  local name=$1 model=$2
  [ -f "$OUT/$name/report.json" ] && { echo "$name: done"; return; }
  MODEL="$model" LLAMA_SERVER="$SERVER" INTERN_BENCH="$REPO/target/release/intern-bench" \
    INTERN_RUNTIME_DIR="$S/runtime" THREADS="${THREADS:-4}" SAMPLE_RSS=1 \
    LLAMA_SERVER_ARGS="--cache-ram 0 -ub 256 --reasoning off" BENCH_ARGS="--prompt-profile bonsai_2" \
    "$REPO/scripts/bonsai-eval/internbench-chunked.sh" "$OUT/$name"
}
run_bench lfm3-1.7b-fewshot "$S/cache/lfm3/LFM3-1.7B-Q4_K_M.gguf"
run_bench lfm3-omni-2b-fewshot "$S/cache/lfm3/LFM3-Omni-2B-Q4_K_M.gguf"

#!/usr/bin/env bash
# Resumable orchestration of the Bonsai evaluation (each step skips finished work).
set -uo pipefail
export S=${S:?set S to the scratch directory}
W=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
R=$S/results/bonsai
export LOCK=$S/bench.lock INTERN_RUNTIME_DIR=$S/runtime INTERN_BENCH=$W/target/release/intern-bench
export INTERN_WORKER=$W/target/release/intern-worker
P26=$S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc
PROF=$W/bench/reports/bonsai-native-windows-2026-10-18/plans
ib() { # name model profile threads extra-args confidence
  local name=$1
  if [ -f "$R/internbench/$name/report.json" ]; then echo "$name done"; return; fi
  MODEL=$2 BENCH_PROFILE=$3 THREADS=$4 LLAMA_SERVER_ARGS="$5" INTERN_BENCH_TOKEN_CONFIDENCE=$6 \
    LLAMA_SERVER=$P26/llama-server SAMPLE_RSS=1 PORT=18${4}9${RANDOM:0:1} \
    bash $W/scripts/bonsai-eval/internbench-chunked.sh "$R/internbench/$name" > "$R/logs/ib-$name.log" 2>&1
  echo "$name: $(tail -1 $R/logs/ib-$name.log)"
}
ib 2b-t4-app $S/cache/bonsai/Ternary-Bonsai-2-2B-PQ2_0.gguf $PROF/profile.json 4 "" 1
ib 4b-t4-app $S/cache/bonsai/Ternary-Bonsai-2-4B-PQ2_0.gguf $PROF/bonsai4-profile-17.json 4 "" 1
bash $W/scripts/bonsai-eval/phase1.sh >> $R/logs/phase1.log 2>&1
bash $W/scripts/bonsai-eval/phase2a.sh >> $R/logs/phase2a.log 2>&1
echo PIPELINE-STAGE1-DONE

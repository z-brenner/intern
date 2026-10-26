#!/usr/bin/env bash
# Phase 1: runtime/build/CPU-variant/quantisation matrix with llama-bench.
set -uo pipefail
S=${S:?set S to the scratch directory}
W=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
export LOCK=$S/bench.lock
SW=$W/scripts/bonsai-eval/llama-bench-sweep.sh
OUT=$S/results/bonsai/microbench/runtime-matrix.jsonl
C=$S/cache/bonsai
B=$S/build
P26=$B/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc
P77=$B/prebuilt-prism-b10777/llama-prism-b10777-e3de715
A=(-p 512 -n 64 -t 2,4 -r 3)
for sz in 2B 4B; do
  M=$C/Ternary-Bonsai-2-$sz-PQ2_0.gguf
  $SW $OUT C-$sz-PQ2_0-b10826-prebuilt-auto $P26 $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10777-prebuilt-auto $P77 $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10777-source-portable-auto $B/prism-b10777-portable/bin $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10777-source-native $B/prism-b10777-native/bin $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10826-prebuilt-haswell $B/prebuilt-prism-b10826-only-haswell $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10826-prebuilt-alderlake $B/prebuilt-prism-b10826-only-alderlake $M "${A[@]}"
  $SW $OUT C-$sz-PQ2_0-b10777-source-haswell $B/prism-b10777-portable-only-haswell $M "${A[@]}"
  for q in PTQ1_0 Q2_0; do
    M=$C/Ternary-Bonsai-2-$sz-$q.gguf
    $SW $OUT C-$sz-$q-b10826-prebuilt-auto $P26 $M "${A[@]}"
    $SW $OUT C-$sz-$q-b10826-prebuilt-haswell $B/prebuilt-prism-b10826-only-haswell $M "${A[@]}"
    $SW $OUT C-$sz-$q-b10777-prebuilt-auto $P77 $M "${A[@]}"
  done
  if [ -x $B/upstream-b11912-avx2/bin/llama-bench ]; then
    $SW $OUT C-$sz-Q2_0-upstream-b11912-avx2 $B/upstream-b11912-avx2/bin $C/Ternary-Bonsai-2-$sz-Q2_0.gguf "${A[@]}"
  fi
done
echo PHASE1-DONE

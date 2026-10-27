#!/usr/bin/env bash
# Phase 2a: llama-bench flag sweep on the candidate runtime (Prism b10826 prebuilt).
set -uo pipefail
S=${S:?set S to the scratch directory}
W=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
export LOCK=${LOCK-$S/bench.lock}  # LOCK="" when the caller holds the lock
SW=$W/scripts/bonsai-eval/llama-bench-sweep.sh
OUT=$S/results/bonsai/microbench/flag-sweep.jsonl
C=$S/cache/bonsai
P26=$S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc
H26=$S/build/prebuilt-prism-b10826-only-haswell
for sz in 2B 4B; do
  M=$C/Ternary-Bonsai-2-$sz-PQ2_0.gguf
  # prefill vs micro-batch (-b 2048) at an Intern-sized prompt
  $SW $OUT F-$sz-ub-auto $P26 $M -p 2048 -n 0 -b 2048 -ub 256,512,1024,2048 -t 4 -r 2
  $SW $OUT F-$sz-ub-auto-t2 $P26 $M -p 2048 -n 0 -b 2048 -ub 512,1024 -t 2 -r 2
  $SW $OUT F-$sz-ub-haswell $H26 $M -p 2048 -n 0 -b 2048 -ub 256,512,1024 -t 4 -r 2
  # KV cache type and flash attention: prefill 2048, decode after 1024 tokens of context
  $SW $OUT F-$sz-kv-fa1-f16 $P26 $M -p 2048 -n 32 -d 0,1024 -fa 1 -t 4 -r 2
  $SW $OUT F-$sz-kv-fa1-q8 $P26 $M -p 2048 -n 32 -d 0,1024 -fa 1 -ctk q8_0 -ctv q8_0 -t 4 -r 2
  $SW $OUT F-$sz-kv-fa0-f16 $P26 $M -p 2048 -n 32 -d 0,1024 -fa 0 -t 4 -r 2
  # thread scaling (1 and 3; 2 and 4 are in the runtime matrix)
  $SW $OUT F-$sz-threads $P26 $M -p 512 -n 32 -t 1,3 -r 2
done
echo PHASE2A-DONE

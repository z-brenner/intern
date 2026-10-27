#!/usr/bin/env bash
# Phase 2c: micro-batch size on an AVX2-only CPU variant (what most laptops load), where the
# flag sweep showed -ub 256 prefilling faster than the default 512. llama-bench, then end to end.
set -uo pipefail
S=${S:?set S to the scratch directory}
W=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
export LOCK=${LOCK-$S/bench.lock}  # LOCK="" when the caller holds the lock
SW=$W/scripts/bonsai-eval/llama-bench-sweep.sh
OUT=$S/results/bonsai/microbench/flag-sweep.jsonl
C=$S/cache/bonsai
H26=$S/build/prebuilt-prism-b10826-only-haswell
P26=$S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc
$SW $OUT H-2B-ub-haswell-t2 $H26 $C/Ternary-Bonsai-2-2B-PQ2_0.gguf -p 1024 -n 0 -b 2048 -ub 128,256,512 -t 2 -r 2
$SW $OUT H-4B-ub-haswell $H26 $C/Ternary-Bonsai-2-4B-PQ2_0.gguf -p 1024 -n 0 -b 2048 -ub 128,256,512 -t 2,4 -r 2
$SW $OUT H-2B-ub-auto-t2 $P26 $C/Ternary-Bonsai-2-2B-PQ2_0.gguf -p 1024 -n 0 -b 2048 -ub 128,256,512 -t 2 -r 2
# end to end on the haswell variant, 6 real requests, 2 threads (app default here)
Q=$S/work/req2
REQ6=$Q/real-assignment-assumption.json,$Q/real-capital-call-notice.json,$Q/real-invoice-right-aligned.json,$Q/real-promissory-note.json,$Q/real-scan-lease-25p.json,$Q/real-separation-agreement.json
SOUT=$S/results/bonsai/microbench/server.jsonl
sbh() { local label=$1 model=$2 extra=$3
  if grep -q "\"label\": \"$label\"" "$SOUT" 2>/dev/null; then echo "$label skipped"; return; fi
  ${LOCK:+flock -x "$S/bench.turnstile" flock -x "$LOCK"} python3 -I "$W/scripts/bonsai-eval/serverbench.py" \
    --server "$H26/llama-server" --model "$model" --threads 2 --label "$label" --out "$SOUT" \
    --requests "$REQ6" --extra "--cache-ram 0 $extra" --warmup "$Q/real-scan-noisy-statement.json"
}
sbh S-2B-t2-haswell-ub512 $C/Ternary-Bonsai-2-2B-PQ2_0.gguf ""
sbh S-2B-t2-haswell-ub256 $C/Ternary-Bonsai-2-2B-PQ2_0.gguf "-ub 256"
sbh S-4B-t2-haswell-ub512 $C/Ternary-Bonsai-2-4B-PQ2_0.gguf ""
sbh S-4B-t2-haswell-ub256 $C/Ternary-Bonsai-2-4B-PQ2_0.gguf "-ub 256"
echo PHASE2C-DONE

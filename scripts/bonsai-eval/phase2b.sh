#!/usr/bin/env bash
# Phase 2b: llama-server end-to-end on Intern-shaped requests (one server per label,
# each run one exclusive-lock measurement; labels already in OUT are skipped).
set -uo pipefail
S=${S:?set S to the scratch directory}
W=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
OUT=$S/results/bonsai/microbench/server.jsonl
Q=$S/work/req2
C=$S/cache/bonsai
P26=$S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc
REAL=$(ls $Q/real-*.json | paste -sd,)
LONG=$Q/long-2000.json,$Q/long-4000.json,$Q/long-6500.json
WARM=$Q/real-scan-noisy-statement.json
sb() { # label model threads requests extra [serverbench options...]
  local label=$1 model=$2 th=$3 reqs=$4 extra=$5; shift 5
  if [ -f "$OUT" ] && grep -q "\"label\": \"$label\"" "$OUT"; then echo "$label skipped"; return; fi
  flock -x "$S/bench.turnstile" flock -x "$S/bench.lock" python3 -I "$W/scripts/bonsai-eval/serverbench.py" --server "$P26/llama-server" \
    --model "$model" --threads "$th" --label "$label" --out "$OUT" --requests "$reqs" --extra "$extra" "$@"
}
M2=$C/Ternary-Bonsai-2-2B-PQ2_0.gguf
M4=$C/Ternary-Bonsai-2-4B-PQ2_0.gguf
M4MTP=$C/Ternary-Bonsai-2-4B-PQ2_0-MTP-Q8_0.gguf
MTP="--spec-type draft-mtp --spec-draft-n-min 0 --spec-draft-p-min 0"
for m in 2B 4B; do
  M=$C/Ternary-Bonsai-2-$m-PQ2_0.gguf
  # cold start (model evicted from the page cache), long prompts, idle CPU after
  sb S-$m-t2-long $M 2 "$LONG" "" --warmup "$WARM" --evict --idle 10
  sb S-$m-t4-long $M 4 "$LONG" "" --warmup "$WARM" --evict --idle 10
  # the 12 real Windows-qualification requests: app flags, then variations
  sb S-$m-t4-real12 $M 4 "$REAL" "" --warmup "$WARM"
  sb S-$m-t2-real12 $M 2 "$REAL" "" --warmup "$WARM"
  sb S-$m-t4-real12-logprobs $M 4 "$REAL" "" --warmup "$WARM" --logprobs
  sb S-$m-t4-real12-cram0 $M 4 "$REAL" "--cache-ram 0" --warmup "$WARM"
  sb S-$m-t4-real12-ub1024 $M 4 "$REAL" "-ub 1024" --warmup "$WARM"
  sb S-$m-t2-real12-bg $M 2 "$REAL" "--cache-ram 0 --poll 0 --prio -1" --warmup "$WARM" --idle 10 --nice 10
done
sb S-4B-t4-real12-mtp2 $M4MTP 4 "$REAL" "$MTP --spec-draft-n-max 2" --warmup "$WARM"
sb S-4B-t2-real12-mtp2 $M4MTP 2 "$REAL" "$MTP --spec-draft-n-max 2" --warmup "$WARM"
sb S-4B-t4-real12-mtp1 $M4MTP 4 "$REAL" "$MTP --spec-draft-n-max 1" --warmup "$WARM"
sb S-4B-t4-real12-mtp3 $M4MTP 4 "$REAL" "$MTP --spec-draft-n-max 3" --warmup "$WARM"
sb S-4B-t4-real12-mtpfile-nospec $M4MTP 4 "$REAL" "" --warmup "$WARM"
sb S-4B-t2-long-mtp2 $M4MTP 2 "$LONG" "$MTP --spec-draft-n-max 2" --warmup "$WARM" --evict --idle 10
echo PHASE2B-DONE

#!/usr/bin/env bash
# d1 shaped the way Intern could ship it: the one question that separated
# wrong names in the full run (party_main), asked only about names already
# marked ready, from the smaller Q4_K_M file, at the app's thread count
# (half the logical cores, 2..12). Records time and memory per document.
#
# Laboratory tooling; the app does not call it. Run after run.sh.
# Usage: S=/scratch [D1=...gguf] [THREADS=n] scripts/lfm3-eval/run-d1-gate.sh
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
S="${S:?set S to the scratch directory}"
SERVER="$S/build/b11946-portable/bin/llama-server"
OUT="$S/results/lfm3"
D1="${D1:-$S/cache/lfm3/d1-2B-Q4_K_M.gguf}"
if [ -z "${THREADS:-}" ]; then
  THREADS=$(( $(getconf _NPROCESSORS_ONLN) / 2 ))
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 12 ] && THREADS=12
fi
NAME="$(basename "$D1" .gguf)-party_main-t$THREADS"
[ -f "$OUT/$NAME.md" ] && { echo "$NAME: done"; exit 0; }
PORT=18093
# -b/-ub hold one question with its state: 8,000 characters is ~2.3k tokens.
"$SERVER" --host 127.0.0.1 --port $PORT --model "$D1" --ctx-size 8192 -b 4096 -ub 4096 \
  --threads "$THREADS" --threads-batch "$THREADS" --no-webui --no-mmproj > "$OUT/$NAME-server.log" 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT
for _ in $(seq 1 600); do curl -sf --noproxy '*' "http://127.0.0.1:$PORT/health" | grep -q ok && break; sleep 0.5; done
python3 -I "$REPO/scripts/lfm3-eval/d1_verifier.py" ask --server "http://127.0.0.1:$PORT" --only-ready \
  --questions party_main --recording "$OUT/qwen/recording.json" --report "$OUT/qwen/report.json" \
  --out "$OUT/$NAME-answers.jsonl"
grep -E '^VmHWM|^VmRSS' "/proc/$PID/status" > "$OUT/$NAME-memory.txt" || true
python3 -I "$REPO/scripts/lfm3-eval/d1_verifier.py" analyze --answers "$OUT/$NAME-answers.jsonl" \
  --report "$OUT/qwen/report.json" --markdown "$OUT/$NAME.md"

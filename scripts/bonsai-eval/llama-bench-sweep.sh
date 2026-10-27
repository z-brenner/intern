#!/usr/bin/env bash
# One llama-bench invocation as one exclusive-lock measurement; JSONL results
# are appended to OUT_JSONL, each line tagged with LABEL (see lb_append.py).
# Laboratory tooling for the Bonsai 2 CPU evaluation; the app does not call it.
#
# Usage: LOCK=/path/bench.lock llama-bench-sweep.sh OUT_JSONL LABEL BIN_DIR MODEL [llama-bench args...]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT=$1 LABEL=$2 BIN=$3 MODEL=$4
shift 4
mkdir -p "$(dirname "$OUT")"
# Resumable: a label already in OUT is skipped (delete its lines to re-measure).
if [ -f "$OUT" ] && grep -q "\"label\": \"$LABEL\"" "$OUT"; then
  echo "$LABEL: already measured, skipped"
  exit 0
fi
TMP=$(mktemp "$(dirname "$OUT")/.lb.XXXXXX")
ERR="$(dirname "$OUT")/llama-bench.$LABEL.stderr.log"
if [ -n "${LOCK:-}" ]; then
  # turnstile: while this measurement waits, new shared holders queue behind it
  flock -x "$(dirname "$LOCK")/bench.turnstile" flock -x "$LOCK" "$BIN/llama-bench" -m "$MODEL" -o jsonl "$@" > "$TMP" 2> "$ERR"
else
  "$BIN/llama-bench" -m "$MODEL" -o jsonl "$@" > "$TMP" 2> "$ERR"
fi
grep -h 'load_backend' "$ERR" | sed "s/^/$LABEL: /" || true
python3 -I "$HERE/lb_append.py" "$LABEL" "$TMP" "$OUT"
rm -f "$TMP"

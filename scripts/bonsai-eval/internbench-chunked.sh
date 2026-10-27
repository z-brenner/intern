#!/usr/bin/env bash
# Resumable live InternBench run for a candidate model: the 77 documents in
# chunks of CHUNK (default 8), each chunk one run of
# scripts/run-internbench-candidate.sh (fresh llama-server, warm-up, its own
# recording); the whole run holds the exclusive LOCK. A chunk whose report.json exists is
# skipped, so a run killed by a restart resumes where it stopped. When every
# chunk is done the recordings are merged (intern-bench merge-recordings) and
# replayed into one report whose timings are the live ones.
#
# Laboratory tooling; the app does not call it. Usage:
#   MODEL=... LLAMA_SERVER=... INTERN_BENCH=... INTERN_RUNTIME_DIR=... \
#   [BENCH_PROFILE=...] [THREADS=n] [LLAMA_SERVER_ARGS=...] [LOCK=bench.lock] \
#   [INTERN_BENCH_TOKEN_CONFIDENCE=1] [SAMPLE_RSS=1] [ONLY=id,id] \
#     scripts/bonsai-eval/internbench-chunked.sh RUN_DIR
set -uo pipefail
# Hold the exclusive lock for the whole run (the run is one measurement), not per
# chunk: with other agents' shared holders coming and going, re-queueing for every
# chunk starves. A restart still loses at most the chunk in progress.
if [ -n "${LOCK:-}" ] && [ -z "${HOLDING_LOCK:-}" ]; then
  HOLDING_LOCK=1 exec flock -x "$LOCK" "$0" "$@"
fi
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUN="${1:?run directory}"
CHUNK="${CHUNK:-8}"
mkdir -p "$RUN/chunks"
if [ -n "${ONLY:-}" ]; then
  IDS=$(echo "$ONLY" | tr ',' '\n')
else
  IDS=$(python3 -I -c 'import json,sys; [print(d["id"]) for d in json.load(open(sys.argv[1]))["documents"]]' "$REPO/bench/gold.json")
fi
mapfile -t ALL <<< "$IDS"
n=${#ALL[@]}
i=0; c=0
while [ $i -lt $n ]; do
  ids=$(IFS=,; echo "${ALL[*]:$i:$CHUNK}")
  dir="$RUN/chunks/$(printf %02d $c)"
  if [ -f "$dir/report.json" ]; then
    echo "chunk $c done, skipped"
  else
    rm -rf "$dir"
    echo "chunk $c: $ids  ($(date -u +%H:%M:%S))"
    "$REPO/scripts/run-internbench-candidate.sh" "$dir" -- --only "$ids"
    echo "chunk $c exit $? ($(date -u +%H:%M:%S))"
  fi
  i=$((i + CHUNK)); c=$((c + 1))
done
for d in "$RUN"/chunks/*; do [ -f "$d/report.json" ] || { echo "incomplete: $d" >&2; exit 1; }; done
first=1
for d in "$RUN"/chunks/*; do
  if [ $first = 1 ]; then cp "$d/recording.json" "$RUN/recording.json"; first=0; continue; fi
  "$INTERN_BENCH" merge-recordings --base "$RUN/recording.json" --add "$d/recording.json" \
    --output "$RUN/recording.merged.json" --gold "$REPO/bench/gold.json" && mv "$RUN/recording.merged.json" "$RUN/recording.json"
done
"$INTERN_BENCH" run --corpus "${CORPUS:-$REPO/bench/generated}" --gold "$REPO/bench/gold.json" \
  --manifest "$REPO/bench/manifest.json" ${ONLY:+--only "$ONLY"} --replay "$RUN/recording.json" \
  --output "$RUN/report.json" --markdown "$RUN/report.md"
echo "merged report: $RUN/report.md"

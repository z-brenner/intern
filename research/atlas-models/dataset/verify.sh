#!/usr/bin/env bash
# Laboratory tooling; the app does not call it.
#
#   verify.sh CORPUS_DIR WORK_DIR
#
# Turns a generated corpus (generate.mjs) into verified SFT examples:
#   1. runs intern-bench over the corpus against fake_server.py to capture the exact request
#      (system turn, evidence lines, grammar) the engine sends for each document;
#   2. for variants 0..3, builds a reply from the gold and the captured evidence lines
#      (build_targets.py), serves it, and lets intern-bench parse, validate, compose and score it;
#      a document is kept the first time the composed name is scored right;
#   3. exports the kept documents to WORK_DIR/sft.jsonl.
# Nothing here runs a model. Needs INTERN_RUNTIME_DIR (PDFium) and target/release/{intern-bench,intern-worker}.
set -euo pipefail
corpus="$(cd "$1" && pwd)"
work="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
bench="${INTERN_BENCH:-$repo/target/release/intern-bench}"
worker="${INTERN_WORKER:-$repo/target/release/intern-worker}"
port="${PORT:-18091}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
rm -f "$work/capture.jsonl" "$work/passed.json"

serve() {
  python3 -I "$here/fake_server.py" --port "$port" --capture "$work/capture.jsonl" "$@" &
  server=$!
  for _ in $(seq 50); do curl -sf "http://127.0.0.1:$port/health" >/dev/null && return 0; sleep 0.1; done
  echo "fake server did not start" >&2; return 1
}
bench_run() {
  "$bench" run --corpus "$corpus" --gold "$corpus/gold.json" --worker "$worker" \
    --endpoint "http://127.0.0.1:$port" --api-key unused --no-warmup --output "$1" > "$1.log" 2>&1 || true
}

serve
bench_run "$work/capture-report.json"
kill "$server"; wait "$server" 2>/dev/null || true
cp "$work/capture.jsonl" "$work/requests.jsonl"

# Variants 0-2 try three role orders and keep only names joined as the gold's own party set;
# 3-5 retry them accepting the gold's alternative sets; 6-8 also give the type in the gold's case.
for variant in 0 1 2 3 4 5 6 7 8; do
  if [ -f "$work/passed.json" ] && python3 -I -c "import json,sys; sys.exit(0 if len(json.load(open(sys.argv[1]))) >= len(json.load(open(sys.argv[2]))['documents']) else 1)" "$work/passed.json" "$corpus/gold.json"; then
    break
  fi
  strict=""; [ "$variant" -lt 3 ] && strict="--strict"
  python3 -I "$here/build_targets.py" answers --gold "$corpus/gold.json" --capture "$work/requests.jsonl" \
    --variant "$variant" --passed "$work/passed.json" --out "$work/answers-$variant.json"
  serve --answers "$work/answers-$variant.json"
  bench_run "$work/report-$variant.json"
  kill "$server"; wait "$server" 2>/dev/null || true
  python3 -I "$here/build_targets.py" record --gold "$corpus/gold.json" --capture "$work/requests.jsonl" \
    --report "$work/report-$variant.json" --answers "$work/answers-$variant.json" --passed "$work/passed.json" $strict
done
python3 -I "$here/build_targets.py" export --gold "$corpus/gold.json" --capture "$work/requests.jsonl" \
  --passed "$work/passed.json" --out "$work/sft.jsonl"

#!/usr/bin/env bash
# Laboratory tooling for .github/workflows/atlas-teacher.yml; the app does not call it.
#
#   bench_through_fake.sh WORK_DIR ANSWERS PORT
#
# Runs intern-bench over CORPUS/GOLD against dataset/fake_server.py at the app's review gate
# (GATE, default 0.7). Every request the engine sends is captured to WORK_DIR/capture.jsonl.
# With ANSWERS empty, every reply is an empty object the engine rejects, so this only captures.
# With ANSWERS set (teacher/answer.py output, .json or .json.gz), each document gets the reply
# and token probabilities a real model gave to that exact request, and WORK_DIR/report.json is
# scored as the live run would have been. Needs INTERN_RUNTIME_DIR and target/release builds.
set -euo pipefail
work="$1"
answers="$2"
port="$3"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
bench="${INTERN_BENCH:-$repo/target/release/intern-bench}"
worker="${INTERN_WORKER:-$repo/target/release/intern-worker}"
mkdir -p "$work"
rm -f "$work/capture.jsonl"
extra=()
[ -n "$answers" ] && extra=(--answers "$answers")
python3 -I "$here/../dataset/fake_server.py" --port "$port" --capture "$work/capture.jsonl" "${extra[@]}" &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT
for _ in $(seq 50); do curl -sf "http://127.0.0.1:$port/health" >/dev/null && break; sleep 0.2; done
status=0
"$bench" run --corpus "${CORPUS:?}" --gold "${GOLD:?}" --worker "$worker" \
  --endpoint "http://127.0.0.1:$port/v1/chat/completions" --api-key unused --no-warmup \
  --min-token-confidence "${GATE:-0.7}" --output "$work/report.json" > "$work/report.json.log" 2>&1 || status=$?
tail -n 20 "$work/report.json.log"
# A capture-only pass ends with every reply rejected, which intern-bench may report as failure.
if [ -n "$answers" ] && [ "$status" -ne 0 ] && [ ! -f "$work/report.json" ]; then exit "$status"; fi
exit 0

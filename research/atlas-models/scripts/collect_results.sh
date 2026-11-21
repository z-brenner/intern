#!/usr/bin/env bash
# Laboratory tooling for .github/workflows/atlas-research.yml; the app does not call it.
#
#   collect_results.sh OUT_DIR DEST_DIR
#
# Copies what is worth keeping from one InternBench run directory (OUT_DIR, as
# scripts/run-internbench.sh writes it) into DEST_DIR: the report compressed, its Markdown, the
# memory and wall-time files, the CPU, and the recording's exchanges without the extracted
# documents (the documents are public and refetched by pin; the exchanges are what a later
# re-gating or teacher study needs). Nothing else, and never a model file.
set -euo pipefail
out="$1"
dest="$2"
mkdir -p "$dest"
[ -f "$out/report.json" ] && gzip -9 -c "$out/report.json" > "$dest/report.json.gz"
for name in report.md server-memory.txt wall-seconds.txt lscpu.txt model.json; do
  [ -f "$out/$name" ] && cp "$out/$name" "$dest/$name"
done
if [ -f "$out/run.log" ]; then
  grep -E '^(filename |unsafe ready |total per document)' "$out/run.log" > "$dest/headline.txt" || true
fi
if [ -f "$out/recording.json" ]; then
  python3 -I - "$out/recording.json" "$dest/exchanges.json.gz" <<'EOF'
import gzip, json, sys
recording = json.load(open(sys.argv[1], encoding="utf-8"))
for document in recording.get("documents", []):
    document["extraction"] = {"outcome": "dropped_for_size"}
    for exchange in document.get("exchanges", []):
        exchange.pop("token_trace", None)
        if isinstance(exchange.get("reply"), dict):
            exchange["reply"].pop("token_trace", None)
with gzip.open(sys.argv[2], "wt", encoding="utf-8") as file:
    json.dump(recording, file)
EOF
fi
ls -la "$dest"

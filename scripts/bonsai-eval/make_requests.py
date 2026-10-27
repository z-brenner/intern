"""Build microbenchmark request bodies from the committed Windows Bonsai captures.

real-*.json : the exact forwarded (few-shot greedy profile) request of each of the
              12 Windows heldout cases (last model request per case).
long-NNNN.json : the promissory-note request with its evidence replaced by
              [pN.bM] lines from the 25-page scanned lease (and other captures) until
              roughly NNNN characters, to reach ~4k and ~6.5k prompt tokens.
Usage: python3 -I make_requests.py RAW_JSONL OUT_DIR
"""
import json
import sys
from pathlib import Path

raw, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
cases = [json.loads(line) for line in raw.open()]
pages_text = []
for case in cases:
    cid = case["captured_document"]["id"]
    req = case["wire_records"][-1]["request"]["forwarded"]
    (out / f"real-{cid}.json").write_text(json.dumps(req))
    src = case["captured_document"]["extraction"].get("source") or {}
    for page in src.get("pages", []):
        pages_text.append((cid, page["page_number"], page.get("text", "")))

base = next(c for c in cases if c["captured_document"]["id"] == "promissory-note")
base_req = base["wire_records"][-1]["request"]["forwarded"]
lines = []
for n, (cid, page, text) in enumerate(sorted(pages_text, key=lambda t: (t[0] != "scan-lease-25p", t[0], t[1]))):
    for b, line in enumerate([l.strip() for l in text.splitlines() if l.strip()], 1):
        lines.append(f"[p{page}.b{b}] {line}")

# ~3.6 chars/token for this English legal text plus ids; the prefix is ~630 tokens.
for target_tokens in (2000, 4000, 6500):
    budget = int((target_tokens - 700) * 3.5)
    body, size = [], 0
    for line in lines:
        if size + len(line) > budget:
            break
        body.append(line)
        size += len(line) + 1
    user = ("Excerpts from a 25-page document follow.\n--- BEGIN EVIDENCE ---\n"
            + "\n".join(body) + "\n--- END EVIDENCE ---\n\nJSON only.")
    req = json.loads(json.dumps(base_req))
    req["messages"][-1]["content"] = user
    (out / f"long-{target_tokens}.json").write_text(json.dumps(req))
print(len(cases), "cases;", len(lines), "evidence lines available")

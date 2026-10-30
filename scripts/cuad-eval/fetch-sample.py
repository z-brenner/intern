"""Fetches the pinned CUAD sample (sample.json) and writes it as an InternBench corpus.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/cuad-eval/fetch-sample.py OUT_DIR

writes OUT_DIR/corpus/cuad-NN.pdf (each checked against its pinned size and SHA-256) and
OUT_DIR/gold.json, a minimal InternBench gold from CUAD's own answers. A live run then records
it like any corpus:

    CORPUS=OUT_DIR/corpus GOLD=OUT_DIR/gold.json MANIFEST= MODEL=... LLAMA_SERVER=... \\
      scripts/run-internbench.sh OUT_DIR/run -- --min-token-confidence 0.5

and scripts/cuad-eval/score.py scores its report.json against CUAD's answers. InternBench's
own filename score is no use here: CUAD's answers are verbatim spans, typos and blanks included,
not filenames.

The contracts are SEC filings collected by The Atticus Project (CUAD v1, CC BY 4.0); they are
fetched, never committed.
"""
import hashlib
import json
import os
import re
import sys
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from score import parse_date  # noqa: E402

BASE = "https://huggingface.co/datasets/theatticusproject/cuad/resolve"


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: fetch-sample.py OUT_DIR")
    out = sys.argv[1]
    sample = json.load(open(os.path.join(HERE, "sample.json"), encoding="utf-8"))
    corpus = os.path.join(out, "corpus")
    os.makedirs(corpus, exist_ok=True)
    documents = []
    for contract in sample["contracts"]:
        path = os.path.join(corpus, f"{contract['id']}.pdf")
        if not (os.path.exists(path) and matches(path, contract)):
            url = f"{BASE}/{sample['revision']}/{urllib.parse.quote(contract['path'])}"
            with urllib.request.urlopen(url, timeout=120) as response, open(path, "wb") as file:
                file.write(response.read())
            if not matches(path, contract):
                os.remove(path)
                sys.exit(f"{contract['id']}: {contract['path']} does not match its pinned size and SHA-256")
        documents.append(gold_document(contract))
    gold = {
        "schema_version": 1,
        "suite": "cuad-sample",
        "notice": f"{sample['source']}, {sample['license']}: real commercial contracts filed with the SEC.",
        "documents": documents,
    }
    with open(os.path.join(out, "gold.json"), "w", encoding="utf-8") as file:
        json.dump(gold, file, indent=1, ensure_ascii=False)
    print(f"{len(documents)} contracts in {corpus}; gold in {os.path.join(out, 'gold.json')}")


def matches(path, contract):
    data = open(path, "rb").read()
    return len(data) == contract["size"] and hashlib.sha256(data).hexdigest() == contract["sha256"]


def gold_document(contract):
    """InternBench gold from CUAD's answers: the type, the first full date, the named parties."""
    dates = [d for d in (parse_date(contract["agreement_date"]), parse_date(contract["effective_date"])) if d]
    names = []
    for part in contract["parties"].split(";"):
        name = re.sub(r"\([^)]*\)?", " ", part)
        name = " ".join(re.sub(r"[\"“”]", " ", name).split()).strip(" ,")
        if re.search(r"[A-Za-z]{2}", name):
            names.append(name)
    gold = {"document_type": " ".join(contract["document_name"].split()) or None}
    if dates:
        gold["document_date"] = dates[0]
        gold["acceptable_dates"] = dates[1:]
    if names:
        gold["parties"] = names[:2]
        gold["party_relation"] = "between" if len(names) >= 2 else "with"
    return {
        "id": contract["id"],
        "file": f"{contract['id']}.pdf",
        "title": os.path.basename(contract["path"]),
        "kind": "agreement",
        "format": "pdf",
        "gold": gold,
    }


if __name__ == "__main__":
    main()

"""Fetches the pinned real-document sample (sample.json) and writes it as an InternBench corpus.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/realdocs-eval/fetch.py OUT_DIR [SAMPLE.json]

writes OUT_DIR/corpus/<id>.<ext>, each checked against its pinned size and SHA-256, and
OUT_DIR/gold.json, the sample's own gold in InternBench's schema. SAMPLE.json is sample.json, the
first set, by default; sample-heldout.json, the second; sample-heldout2.json, the third, which
shares no document with the other two and was gathered after the rules written from them; or
sample-heldout3.json, the fourth, 58 modern business documents gathered after the rules written
from all three, for the model qualification of November 2026; or sample-heldout4.json, the fifth,
57 letters, notices, invoices, forms, memoranda and minutes, mostly under open licences, gathered
after every rule written so far from collections none of the other four used and from issuers
none of them used but the IRS (a sample notice in the first, a 1982 records form in the fifth);
its report lists the issuers that earlier gold names as recipients or traps. A live
run then records and scores it like any corpus, with InternBench's own filename score:

    CORPUS=OUT_DIR/corpus GOLD=OUT_DIR/gold.json MANIFEST= MODEL=... LLAMA_SERVER=... \\
      scripts/run-internbench.sh OUT_DIR/run -- --min-token-confidence 0.5

The documents are real letters, notices, invoices and receipts from public records (US and UK
government sites, federal courts via CourtListener, broadcast stations' FCC political files)
and public-domain or openly licensed archives; sample.json names each one's source and licence.
They are fetched from their sources, never committed. A file already in OUT_DIR/corpus is kept
when its bytes match the pins, so a document whose site refuses scripted downloads (ftc.gov
answers 403) can be saved there by hand first.
"""
import hashlib
import json
import os
import sys
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
USER_AGENT = "Intern document-naming evaluation (https://github.com/z-brenner/intern)"


def main():
    if len(sys.argv) not in (2, 3):
        sys.exit("usage: fetch.py OUT_DIR [SAMPLE.json]")
    out = sys.argv[1]
    name = sys.argv[2] if len(sys.argv) == 3 else "sample.json"
    sample = json.load(open(os.path.join(HERE, os.path.basename(name)), encoding="utf-8"))
    corpus = os.path.join(out, "corpus")
    os.makedirs(corpus, exist_ok=True)
    documents, failed = [], []
    for document in sample["documents"]:
        path = os.path.join(corpus, document["file"])
        source = document["source"]
        if not (os.path.exists(path) and matches(open(path, "rb").read(), source)):
            try:
                data = get(source["url"])
            except Exception as error:  # noqa: BLE001 - one source down must not stop the rest
                failed.append(f"{document['id']}: {error} ({source['url']})")
                continue
            if not matches(data, source):
                failed.append(f"{document['id']}: does not match its pinned size and SHA-256")
                continue
            with open(path, "wb") as file:
                file.write(data)
        documents.append({key: value for key, value in document.items() if key != "source"})
    gold = {
        "schema_version": 1,
        "suite": "realdocs",
        "notice": sample["notice"],
        "documents": documents,
    }
    with open(os.path.join(out, "gold.json"), "w", encoding="utf-8") as file:
        json.dump(gold, file, indent=1, ensure_ascii=False)
    print(f"{len(documents)} documents in {corpus}; gold in {os.path.join(out, 'gold.json')}")
    if failed:
        sys.exit("not fetched (save each by hand into the corpus and run again):\n  " + "\n  ".join(failed))


def get(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def matches(data, source):
    return len(data) == source["size"] and hashlib.sha256(data).hexdigest() == source["sha256"]


if __name__ == "__main__":
    main()

"""Fetches the pinned real-document sample (sample.json) and writes it as an InternBench corpus.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/realdocs-eval/fetch.py OUT_DIR

writes OUT_DIR/corpus/<id>.<ext>, each checked against its pinned size and SHA-256, and
OUT_DIR/gold.json, the sample's own gold in InternBench's schema. A live run then records and
scores it like any corpus, with InternBench's own filename score:

    CORPUS=OUT_DIR/corpus GOLD=OUT_DIR/gold.json MANIFEST= MODEL=... LLAMA_SERVER=... \\
      scripts/run-internbench.sh OUT_DIR/run -- --min-token-confidence 0.5

The documents are real letters, notices, invoices and receipts from public records (SEC EDGAR,
federal courts via CourtListener, US and UK government sites) and openly licensed datasets;
sample.json names each one's source and licence. They are fetched, never committed. A source is
either a direct URL or a Hugging Face dataset row, which is fetched through the dataset viewer's
rows API because its image URLs are signed and expire.

SEC EDGAR serves only requests whose User-Agent names a contact email (its fair-access policy).
Its documents are marked "needs_contact"; they are fetched only when INTERN_EVAL_CONTACT holds
your own contact address, which is sent to the SEC and nowhere else, and skipped otherwise.
"""
import hashlib
import json
import os
import sys
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
USER_AGENT = "Intern document-naming evaluation (https://github.com/z-brenner/intern)"
# SEC EDGAR refuses requests whose User-Agent names no contact email; whoever fetches supplies one.
CONTACT = os.environ.get("INTERN_EVAL_CONTACT", "").strip()
ROWS = "https://datasets-server.huggingface.co/rows"


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: fetch.py OUT_DIR")
    out = sys.argv[1]
    sample = json.load(open(os.path.join(HERE, "sample.json"), encoding="utf-8"))
    corpus = os.path.join(out, "corpus")
    os.makedirs(corpus, exist_ok=True)
    documents, failed, skipped = [], [], []
    for document in sample["documents"]:
        path = os.path.join(corpus, document["file"])
        source = document["source"]
        cached = os.path.exists(path) and matches(open(path, "rb").read(), source)
        if source.get("needs_contact") and not CONTACT and not cached:
            skipped.append(document["id"])
            continue
        if not cached:
            try:
                data = download(source)
            except Exception as error:  # noqa: BLE001 - one source down must not stop the rest
                failed.append(f"{document['id']}: {error}")
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
    if skipped:
        print(f"skipped {len(skipped)} SEC documents (set INTERN_EVAL_CONTACT to fetch them): "
              + ", ".join(skipped))
    if failed:
        sys.exit("not fetched:\n  " + "\n  ".join(failed))


def download(source):
    if "url" in source:
        return get(source["url"])
    query = urllib.parse.urlencode({
        "dataset": source["dataset"], "config": source["config"], "split": source["split"],
        "offset": source["row"], "length": 1,
    })
    row = json.loads(get(f"{ROWS}?{query}"))["rows"][0]["row"]
    return get(row[source["column"]]["src"])


def get(url):
    agent = USER_AGENT
    if urllib.parse.urlparse(url).hostname in ("www.sec.gov", "sec.gov"):
        agent = f"Intern document-naming evaluation {CONTACT}"
    request = urllib.request.Request(url, headers={"User-Agent": agent})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def matches(data, source):
    return len(data) == source["size"] and hashlib.sha256(data).hexdigest() == source["sha256"]


if __name__ == "__main__":
    main()

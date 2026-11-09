"""Fetches the pinned real-document sample (sample.json) and writes it as an InternBench corpus.

Laboratory tooling; the app does not call it. Usage:

    python3 -I scripts/realdocs-eval/fetch.py [--gold-only] [--recording RECORDING.json] OUT_DIR [SAMPLE.json]

writes OUT_DIR/corpus/<id>.<ext>, each checked against its pinned size and SHA-256, and
OUT_DIR/gold.json, the sample's own gold in InternBench's schema. SAMPLE.json is sample.json, the
first set, by default; sample-heldout.json, the second; sample-heldout2.json, the third, which
shares no document with the other two and was gathered after the rules written from them; or
sample-heldout3.json, the fourth, 58 modern business documents gathered after the rules written
from all three, for the model qualification of November 2026. A live
run then records and scores it like any corpus, with InternBench's own filename score:

    CORPUS=OUT_DIR/corpus GOLD=OUT_DIR/gold.json MANIFEST= MODEL=... LLAMA_SERVER=... \\
      scripts/run-internbench.sh OUT_DIR/run -- --min-token-confidence 0.5

--gold-only writes OUT_DIR/gold.json and fetches nothing: a replay of a recording needs the gold,
never the documents (bench/realdocs/README.md). --recording keeps only the documents RECORDING.json
holds, so a replay of a recording that lacks some of the sample's documents (the committed
recordings hold only the documents whose licence allows it) scores every document of its gold.

The documents are real letters, notices, invoices and receipts from public records (US and UK
government sites, federal courts via CourtListener, broadcast stations' FCC political files)
and public-domain or openly licensed archives; sample.json names each one's source and licence.
They are fetched from their sources, never committed. A file already in OUT_DIR/corpus is kept
when its bytes match the pins, so a document whose site refuses scripted downloads (ftc.gov
answers 403) can be saved there by hand first. A source that answers 429 (too many requests) or
a 5xx, or whose connection fails, resets or times out, is asked again, three tries in all, about
20 seconds apart; any other refusal (a 403, a 404) is final.
"""
import hashlib
import http.client
import json
import os
import sys
import time
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
USER_AGENT = "Intern document-naming evaluation (https://github.com/z-brenner/intern)"
USAGE = "usage: fetch.py [--gold-only] [--recording RECORDING.json] OUT_DIR [SAMPLE.json]"
TRIES = 3
RETRY_SECONDS = 20


def main():
    arguments = sys.argv[1:]
    gold_only = "--gold-only" in arguments
    arguments = [argument for argument in arguments if argument != "--gold-only"]
    recording = None
    if "--recording" in arguments:
        at = arguments.index("--recording")
        if at + 1 >= len(arguments):
            sys.exit(USAGE)
        recording = arguments[at + 1]
        del arguments[at : at + 2]
    if len(arguments) not in (1, 2) or any(argument.startswith("--") for argument in arguments):
        sys.exit(USAGE)
    out = arguments[0]
    name = arguments[1] if len(arguments) == 2 else "sample.json"
    sample = json.load(open(os.path.join(HERE, os.path.basename(name)), encoding="utf-8"))
    selected = sample["documents"]
    if recording is not None:
        recorded = {document["id"] for document in json.load(open(recording, encoding="utf-8"))["documents"]}
        unknown = sorted(recorded - {document["id"] for document in selected})
        if unknown:
            sys.exit(f"{recording} holds documents {name} does not: {', '.join(unknown)}")
        selected = [document for document in selected if document["id"] in recorded]
    os.makedirs(out, exist_ok=True)
    corpus = os.path.join(out, "corpus")
    if not gold_only:
        os.makedirs(corpus, exist_ok=True)
    documents, failed = [], []
    for document in selected:
        if not gold_only:
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
    if gold_only:
        print(f"{len(documents)} documents' gold in {os.path.join(out, 'gold.json')}; nothing fetched")
    else:
        print(f"{len(documents)} documents in {corpus}; gold in {os.path.join(out, 'gold.json')}")
    if failed:
        sys.exit("not fetched (save each by hand into the corpus and run again):\n  " + "\n  ".join(failed))


def get(url, tries=TRIES, wait=RETRY_SECONDS, sleep=time.sleep, timeout=120):
    """The body at url. A 429 or a 5xx, or a connection that fails, resets or times out
    (runners see these now and then), is asked again, `tries` times in all, `wait` seconds apart
    (or what a 429's or 5xx's Retry-After asks, up to a minute); any other HTTP answer is
    raised at once."""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(1, tries + 1):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if attempt == tries or not (error.code == 429 or 500 <= error.code < 600):
                raise
            delay = retry_after(error.headers.get("Retry-After"), wait)
            what = f"HTTP {error.code}"
        except (urllib.error.URLError, TimeoutError, ConnectionError, http.client.HTTPException) as error:
            # Refused, reset, timed out or cut short: the network, not the source's answer.
            if attempt == tries:
                raise
            delay = wait
            what = f"{type(error).__name__}: {getattr(error, 'reason', error)}"
        print(f"{url}: {what}, try {attempt} of {tries}; asking again in {delay} s", file=sys.stderr)
        sleep(delay)
    raise AssertionError("unreachable")


def retry_after(header, default):
    """Seconds a Retry-After header asks for, when it gives a number of at most 60; else default."""
    try:
        seconds = int(header)
    except (TypeError, ValueError):
        return default
    return seconds if 0 <= seconds <= 60 else default


def matches(data, source):
    return len(data) == source["size"] and hashlib.sha256(data).hexdigest() == source["sha256"]


if __name__ == "__main__":
    main()

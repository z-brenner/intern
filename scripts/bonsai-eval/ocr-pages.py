"""Time two parser workers over the same corpus, alternated on one machine, and check that they
read every document the same: what docs/ocr.md's "OCR concurrency" measures, with the worker of
one build (--base) against another's (--head).

Each run starts a worker, as the app does, and sends it every document of the gold file in order
over the worker's own protocol (crates/intern-worker/src/protocol.rs), one at a time, timing
each from the parse command to the parsed document. Runs alternate base, head, head, base, ...
so a machine that slows down or speeds up mid-job shifts both the same way. For every document
the script keeps a SHA-256 of what the worker returned that the rest of Intern reads - each
page's number, text, source and OCR confidence, and the document's warnings and truncation - and
the summary says whether every run of every worker returned the same for every document. A
worker's peak resident memory over a run is its `VmHWM` from /proc just before it is stopped.

The worker reads PDFium, ONNX Runtime and the OCR models from INTERN_RUNTIME_DIR, which must be
set. Whether it reads two pages at once depends on the machine's logical processors and on the
memory available when each document starts (paddle::thread_budget, paddle::pages_at_once); the
summary records both.

Laboratory tooling; the app does not call it. Linux, standard library only.
  python3 -I ocr-pages.py --base WORKER --head WORKER --corpus DIR --gold GOLD.json --out DIR
      [--rounds 2]
"""
import argparse
import hashlib
import json
import os
import statistics
import subprocess
import time

PROTOCOL_VERSION = 1
# Documents that put OCR to work, read by name in the summary: the longest scan and a mid-sized one.
NAMED = ("scan-lease-25p", "scan-agreement-10p")


def mem_available_bytes():
    with open("/proc/meminfo", encoding="utf-8") as meminfo:
        for line in meminfo:
            if line.startswith("MemAvailable:"):
                return int(line.split()[1]) * 1024
    return None


def peak_resident_mb(pid):
    try:
        with open(f"/proc/{pid}/status", encoding="utf-8") as status:
            for line in status:
                if line.startswith("VmHWM:"):
                    return round(int(line.split()[1]) / 1000, 1)
    except OSError:
        pass
    return None


class Worker:
    def __init__(self, executable):
        self.process = subprocess.Popen(
            [executable], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            text=True, encoding="utf-8", bufsize=1)
        self.sent = 0
        hello = self.request({"type": "hello"})
        if hello["event"]["type"] != "hello":
            raise RuntimeError(f"no hello from {executable}: {hello}")

    def request(self, command):
        self.sent += 1
        request_id = f"ocr-pages-{self.sent}"
        self.process.stdin.write(json.dumps(
            {"protocol_version": PROTOCOL_VERSION, "request_id": request_id, "command": command}) + "\n")
        self.process.stdin.flush()
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError("the worker exited")
            response = json.loads(line)
            if response.get("request_id") != request_id:
                continue
            if response["event"]["type"] in ("hello", "parsed", "error"):
                return response

    def stop(self):
        peak = peak_resident_mb(self.process.pid)
        try:
            self.process.stdin.write(json.dumps(
                {"protocol_version": PROTOCOL_VERSION, "request_id": "shutdown", "command": {"type": "shutdown"}}) + "\n")
            self.process.stdin.flush()
            self.process.wait(timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            self.process.kill()
            self.process.wait()
        return peak


def reading(document):
    """What the rest of Intern reads of a parsed document, hashed."""
    pages = [[page["page_number"], page["text"], page["source"], page.get("ocr_confidence")]
             for page in document["pages"]]
    kept = {"pages": pages, "warnings": document["warnings"], "truncated": document["truncated"]}
    return hashlib.sha256(json.dumps(kept, sort_keys=True, ensure_ascii=False).encode()).hexdigest()


def run(executable, corpus, documents):
    started = time.monotonic()
    worker = Worker(executable)
    rows = []
    for document in documents:
        path = os.path.abspath(os.path.join(corpus, document["file"]))
        available = mem_available_bytes()
        began = time.monotonic()
        response = worker.request({"type": "parse", "path": path})
        seconds = time.monotonic() - began
        event = response["event"]
        row = {"id": document["id"], "seconds": round(seconds, 3),
               "mem_available_gib": None if available is None else round(available / 2**30, 2)}
        if event["type"] == "parsed":
            parsed = event["document"]
            row["sha256"] = reading(parsed)
            row["pages"] = len(parsed["pages"])
            # Read by OCR, or native text with OCR'd regions merged in: the pages OCR worked on.
            row["ocr_pages"] = sum(1 for page in parsed["pages"]
                                   if page["source"] == "ocr" or page.get("ocr_confidence") is not None)
            row["sources"] = sorted({page["source"] for page in parsed["pages"]})
        else:
            row["error"] = event.get("code")
        rows.append(row)
        print(f"  {document['id']}: {seconds:.2f} s {row.get('sources', row.get('error'))}", flush=True)
    peak = worker.stop()
    return {"wall_s": round(time.monotonic() - started, 2), "worker_peak_mb": peak, "documents": rows}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--corpus", required=True)
    parser.add_argument("--gold", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--rounds", type=int, default=2)
    args = parser.parse_args()
    if not os.environ.get("INTERN_RUNTIME_DIR"):
        parser.error("set INTERN_RUNTIME_DIR to the runtime the worker reads its OCR models from")
    with open(args.gold, encoding="utf-8") as gold:
        documents = json.load(gold)["documents"]
    os.makedirs(args.out, exist_ok=True)

    order = []
    for round_ in range(1, args.rounds + 1):
        order += [("base", round_), ("head", round_)] if round_ % 2 else [("head", round_), ("base", round_)]
    runs = []
    for which, round_ in order:
        print(f"{which} round {round_}", flush=True)
        result = run(args.base if which == "base" else args.head, args.corpus, documents)
        result.update({"worker": which, "round": round_})
        runs.append(result)
        with open(os.path.join(args.out, f"{which}-r{round_}.json"), "w", encoding="utf-8") as file:
            json.dump(result, file, indent=1)

    ocr_ids = sorted({row["id"] for result in runs for row in result["documents"] if row.get("ocr_pages")})
    readings = {}
    for result in runs:
        for row in result["documents"]:
            readings.setdefault(row["id"], set()).add(row.get("sha256"))
    differing = sorted(document_id for document_id, hashes in readings.items() if len(hashes) > 1)
    failed = sorted({row["id"] for result in runs for row in result["documents"] if "error" in row})

    def totals(which):
        mine = [result for result in runs if result["worker"] == which]
        return {
            "wall_s": [result["wall_s"] for result in mine],
            "documents_s": [round(sum(row["seconds"] for row in result["documents"]), 1) for result in mine],
            "ocr_documents_s": [round(sum(row["seconds"] for row in result["documents"] if row["id"] in ocr_ids), 1)
                                for result in mine],
            "named_s": {name: [next(row["seconds"] for row in result["documents"] if row["id"] == name)
                               for result in mine]
                        for name in NAMED if any(d["id"] == name for d in documents)},
            "worker_peak_mb": [result["worker_peak_mb"] for result in mine],
        }

    def change(key):
        before = statistics.mean(base[key])
        return None if before == 0 else round(statistics.mean(head[key]) / before - 1, 4)

    base, head = totals("base"), totals("head")
    summary = {
        "logical_processors": os.cpu_count(),
        "mem_available_gib_min": min(row["mem_available_gib"] for result in runs for row in result["documents"]
                                     if row["mem_available_gib"] is not None),
        "documents": len(documents),
        "ocr_documents": len(ocr_ids),
        "ocr_pages": sum(row.get("ocr_pages", 0) for row in runs[0]["documents"]),
        "base": base,
        "head": head,
        "ocr_documents_change": change("ocr_documents_s"),
        "documents_change": change("documents_s"),
        "readings_differing": differing,
        "failed": failed,
    }
    with open(os.path.join(args.out, "ocr-pages.json"), "w", encoding="utf-8") as file:
        json.dump({"summary": summary, "order": [f"{which}-r{round_}" for which, round_ in order]}, file, indent=1)
    print(json.dumps(summary, indent=1))
    return 1 if differing or failed else 0


if __name__ == "__main__":
    raise SystemExit(main())

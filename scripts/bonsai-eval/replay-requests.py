"""Replay captured llama-server requests at a chosen concurrency and compare the replies.

The requests are the exact bodies Intern sent, as `intern-bench --bench-profile ... --wire-output DIR`
captures them (`NNNN-model-wire.json`). Each is posted to a running llama-server's
/v1/chat/completions; the reply's content, finish reason, token counts and server timings are
recorded. With --reference, every reply is compared byte for byte with the reply the same request
got before (`NNNN-response.json` in that directory, or a previous replay's JSON): greedy decoding
with the same weights should give the same reply, so a change of speed setting that changes a reply
changes accuracy too and has to be scored by a full InternBench run.

Laboratory tooling; the app does not call it. Standard library only.
  python3 -I replay-requests.py --server http://127.0.0.1:PORT --wire DIR [--wire DIR ...]
      [--concurrency N] [--limit N] [--skip-first] [--reference DIR|FILE] --out OUT.json
"""
import argparse
import glob
import json
import os
import statistics
import threading
import time
import urllib.request


def load_requests(dirs, skip_first):
    requests = []
    for directory in dirs:
        files = sorted(glob.glob(os.path.join(directory, "*-model-wire.json")))
        if skip_first:
            files = files[1:]
        # Keyed like the reference replies: the chunk directory a `wire/` capture sits in.
        name = os.path.basename(directory.rstrip("/"))
        if name == "wire":
            name = os.path.basename(os.path.dirname(directory.rstrip("/")))
        for path in files:
            requests.append((f"{name}/{os.path.basename(path)[:4]}", path))
    return requests


def load_reference(source):
    if not source:
        return {}
    if os.path.isfile(source):
        return {r["key"]: r["content"] for r in json.load(open(source))["results"]}
    reference = {}
    for path in sorted(glob.glob(os.path.join(source, "**", "*-response.json"), recursive=True)):
        parent = os.path.basename(os.path.dirname(os.path.dirname(path)))
        body = json.load(open(path))
        reference[f"{parent}/{os.path.basename(path)[:4]}"] = body["choices"][0]["message"].get("content")
    return reference


def post(server, body, api_key, timeout):
    data = json.dumps(body).encode()
    request = urllib.request.Request(f"{server}/v1/chat/completions", data=data, method="POST")
    request.add_header("Content-Type", "application/json")
    if api_key:
        request.add_header("Authorization", f"Bearer {api_key}")
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--server", required=True)
    parser.add_argument("--wire", action="append", required=True)
    parser.add_argument("--concurrency", type=int, default=1)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--skip-first", action="store_true", help="skip each directory's first capture (a warm-up)")
    parser.add_argument("--reference")
    parser.add_argument("--api-key", default="")
    parser.add_argument("--timeout", type=float, default=600)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    requests = load_requests(args.wire, args.skip_first)
    if args.limit:
        requests = requests[: args.limit]
    reference = load_reference(args.reference)
    results = [None] * len(requests)
    lock = threading.Lock()
    cursor = [0]

    def worker():
        while True:
            with lock:
                index = cursor[0]
                cursor[0] += 1
            if index >= len(requests):
                return
            key, path = requests[index]
            body = json.load(open(path))
            started = time.monotonic()
            try:
                reply = post(args.server, body, args.api_key, args.timeout)
                error = None
            except Exception as exc:  # recorded, not raised: one failure must not end the run
                reply, error = None, repr(exc)
            ended = time.monotonic()
            choice = (reply or {}).get("choices", [{}])[0]
            results[index] = {
                "key": key,
                "start_s": started,
                "end_s": ended,
                "latency_s": ended - started,
                "error": error,
                "content": (choice.get("message") or {}).get("content"),
                "finish_reason": choice.get("finish_reason"),
                "usage": (reply or {}).get("usage"),
                "timings": (reply or {}).get("timings"),
            }

    began = time.monotonic()
    threads = [threading.Thread(target=worker) for _ in range(max(1, args.concurrency))]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    wall = time.monotonic() - began

    done = [r for r in results if r and not r["error"]]
    latencies = [r["latency_s"] for r in done]
    predicted = sum((r["timings"] or {}).get("predicted_n", 0) for r in done)
    draft_n = sum((r["timings"] or {}).get("draft_n", 0) or 0 for r in done)
    draft_ok = sum((r["timings"] or {}).get("draft_n_accepted", 0) or 0 for r in done)
    compared = [r for r in done if r["key"] in reference]
    same = sum(r["content"] == reference[r["key"]] for r in compared)
    summary = {
        "requests": len(requests),
        "errors": len(requests) - len(done),
        "concurrency": args.concurrency,
        "wall_s": round(wall, 1),
        "docs_per_min": round(60 * len(done) / wall, 2) if wall else None,
        "latency_p50_s": round(statistics.median(latencies), 2) if latencies else None,
        "latency_mean_s": round(statistics.mean(latencies), 2) if latencies else None,
        "prompt_ms_p50": round(statistics.median((r["timings"] or {}).get("prompt_ms", 0) for r in done), 1) if done else None,
        "predicted_ms_p50": round(statistics.median((r["timings"] or {}).get("predicted_ms", 0) for r in done), 1) if done else None,
        "generated_tokens": predicted,
        "decode_tok_per_s_p50": round(statistics.median((r["timings"] or {}).get("predicted_per_second", 0) for r in done), 2) if done else None,
        "draft_tokens": draft_n,
        "draft_accepted": draft_ok,
        "compared_with_reference": len(compared),
        "identical_to_reference": same,
        "differing": [r["key"] for r in compared if r["content"] != reference[r["key"]]],
    }
    json.dump({"summary": summary, "results": results}, open(args.out, "w"), indent=1)
    print(json.dumps(summary))


if __name__ == "__main__":
    main()

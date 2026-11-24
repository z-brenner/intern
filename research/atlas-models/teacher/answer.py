#!/usr/bin/env python3
"""Answers captured ATLAS requests with a real llama-server. Laboratory tooling; the app does not call it.

    answer.py --endpoint http://127.0.0.1:8080 --requests requests.jsonl[.gz] --out answers.json \
        [--api-key KEY] [--parallel 4] [--timeout 600]

--requests is what dataset/fake_server.py --capture writes: one {"key", "body"} per line, where
body is the exact chat-completions request the engine sent (system turn, evidence, grammar,
temperature 0, thinking off, logprobs on). Each body is sent unchanged to
ENDPOINT/v1/chat/completions, so the model sees byte for byte what the app would send it.

--out maps each key to {"content", "logprobs", "timings"} from the server's reply. Served back
by fake_server.py --answers, the engine parses, gates and scores the reply exactly as it would
have live, so a model can be answered on rented GPU hardware and scored for free elsewhere.
A request that fails is retried twice and then left out; nothing is invented for it.

Python standard library only, so it runs in any image that has python3.
"""
import argparse
import concurrent.futures
import gzip
import json
import sys
import time
import urllib.request


def read_requests(path):
    opener = gzip.open if path.endswith(".gz") else open
    seen = {}
    with opener(path, "rt", encoding="utf-8") as file:
        for line in file:
            if line.strip():
                record = json.loads(line)
                seen.setdefault(record["key"], record["body"])
    return seen


def ask(endpoint, api_key, body, timeout):
    data = json.dumps(body).encode("utf-8")
    request = urllib.request.Request(endpoint.rstrip("/") + "/v1/chat/completions", data=data, method="POST",
                                     headers={"Content-Type": "application/json",
                                              "Authorization": f"Bearer {api_key}"})
    started = time.monotonic()
    with urllib.request.urlopen(request, timeout=timeout) as response:
        reply = json.load(response)
    choice = reply["choices"][0]
    logprobs = choice.get("logprobs")
    if isinstance(logprobs, dict) and isinstance(logprobs.get("content"), list):
        # The engine's gate reads only each chosen token's own probability, so the alternatives
        # are dropped to keep the file small.
        logprobs = {"content": [{"token": t.get("token"), "logprob": t.get("logprob"), "bytes": t.get("bytes"),
                                 "top_logprobs": []} for t in logprobs["content"]]}
    return {"content": choice["message"].get("content") or "",
            "logprobs": logprobs,
            "timings": reply.get("timings"),
            "wall_seconds": round(time.monotonic() - started, 3)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--requests", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--api-key", default="unused")
    parser.add_argument("--parallel", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=600)
    args = parser.parse_args()

    requests = read_requests(args.requests)
    answers, failed = {}, []
    started = time.monotonic()

    def one(key):
        for attempt in range(3):
            try:
                return key, ask(args.endpoint, args.api_key, requests[key], args.timeout)
            except Exception as error:  # noqa: BLE001 - reported, then retried
                print(f"{key[:12]} attempt {attempt + 1} failed: {error}", file=sys.stderr, flush=True)
                time.sleep(5 * (attempt + 1))
        return key, None

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.parallel)) as pool:
        for done, (key, answer) in enumerate(pool.map(one, list(requests)), 1):
            if answer is None:
                failed.append(key)
            else:
                answers[key] = answer
            if done % 10 == 0 or done == len(requests):
                rate = done / max(time.monotonic() - started, 1e-6) * 60
                print(f"{done}/{len(requests)} answered, {len(failed)} failed, {rate:.1f} per minute", flush=True)

    with open(args.out, "w", encoding="utf-8") as file:
        json.dump(answers, file)
    print(json.dumps({"requests": len(requests), "answered": len(answers), "failed": len(failed),
                      "wall_seconds": round(time.monotonic() - started, 1)}))
    sys.exit(1 if failed and not answers else 0)


if __name__ == "__main__":
    main()

"""Measure the first request a freshly started llama-server answers, with and without the
system-turn state restored from a slot file, as the app restores it at every model load
(docs/architecture.md, "The model server").

A fresh server has nothing cached, so its first document prefills the fixed system turn (477
tokens for Intern's evidence request) before the document's own text; every later document finds
that state in the context checkpoint the server takes at the start of the user turn. A slot file
saved after one document and restored into the next fresh server carries that checkpoint across
the restart. This script runs the whole sequence on one machine:

1. a fresh server reads the --save-with request unwarmed, and its slot is saved to a file;
2. each round, a fresh server reads the --measure request unwarmed (no restore), and another
   fresh server restores the file and then reads the same request;
3. in round 1 the unrestored server then reads --between and --measure again: the steady-state
   reply every later document of a server lifetime gets.

The restored reply must equal the steady-state one byte for byte: the restore only puts back
the state the server would otherwise compute again. Every server runs with the app's flags
(crates/intern-engine/src/server.rs) plus --slot-save-path and an API key, which the slot
endpoints need like every other one.

Laboratory tooling; the app does not call it. Standard library only.
  python3 -I cold-start.py --server-bin PATH --model PATH --wire DIR --out DIR [--threads N]
      [--save-with 0000] [--measure 0015] [--between 0016] [--rounds 2] [--port 18130]
"""
import argparse
import json
import os
import secrets
import shutil
import statistics
import subprocess
import time
import urllib.error
import urllib.request

SLOT_FILE = "cold-start.bin"


def app_arguments(model, port, threads, key, slots, ctx):
    """The app's launch (server.rs arguments_for + MEASURED_FLAGS), with a slot directory."""
    return [
        "--host", "127.0.0.1", "--port", str(port), "--api-key", key, "--model", model,
        "--parallel", "1", "--ctx-size", str(ctx), "--n-gpu-layers", "0",
        "--threads", str(threads), "--threads-batch", str(threads), "--jinja", "--no-webui",
        "--cache-ram", "0", "-ub", "256", "--prio", "-1", "--poll", "0", "--no-mmproj",
        "--slot-save-path", slots,
    ]


def call(port, key, path, body=None, timeout=1800):
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(f"http://127.0.0.1:{port}{path}", data=data,
                                     method="GET" if body is None else "POST")
    request.add_header("Authorization", f"Bearer {key}")
    if data is not None:
        request.add_header("Content-Type", "application/json")
    started = time.monotonic()
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            reply, status = json.load(response), response.status
    except urllib.error.HTTPError as error:
        reply, status = {"error": error.read().decode(errors="replace")[:500]}, error.code
    return reply, status, time.monotonic() - started


class Server:
    def __init__(self, args, label, key):
        self.label, self.key, self.port = label, key, args.port
        self.dir = os.path.join(args.out, label)
        os.makedirs(self.dir, exist_ok=True)
        command = [args.server_bin, *app_arguments(args.model, args.port, args.threads, key, args.slots, args.ctx)]
        with open(os.path.join(self.dir, "server-command.txt"), "w", encoding="utf-8") as file:
            file.write(" ".join(part if part != key else "<key>" for part in command) + "\n")
        self.log = open(os.path.join(self.dir, "llama-server.log"), "w", encoding="utf-8")
        started = time.monotonic()
        self.process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                        stderr=self.log, cwd=os.path.dirname(args.server_bin))
        deadline = started + 600
        while True:
            if self.process.poll() is not None:
                raise SystemExit(f"{label}: llama-server exited; see {self.dir}/llama-server.log")
            try:
                _, status, _ = call(self.port, key, "/health", timeout=2)
                if status == 200:
                    break
            except OSError:
                pass
            if time.monotonic() > deadline:
                raise SystemExit(f"{label}: llama-server never became healthy")
            time.sleep(0.25)
        self.load_s = round(time.monotonic() - started, 2)

    def read(self, key, path):
        body = json.load(open(path, encoding="utf-8"))
        reply, status, latency = call(self.port, self.key, "/v1/chat/completions", body)
        timings = reply.get("timings") or {}
        choice = (reply.get("choices") or [{}])[0]
        return {
            "request": key,
            "status": status,
            "latency_s": round(latency, 2),
            "cache_n": timings.get("cache_n"),
            "prompt_n": timings.get("prompt_n"),
            "prompt_ms": timings.get("prompt_ms"),
            "predicted_n": timings.get("predicted_n"),
            "predicted_ms": timings.get("predicted_ms"),
            "content": (choice.get("message") or {}).get("content"),
            "error": reply.get("error"),
        }

    def slot(self, action):
        reply, status, latency = call(self.port, self.key, f"/slots/0?action={action}", {"filename": SLOT_FILE})
        return {"status": status, "wall_ms": round(latency * 1000, 1), **reply}

    def stop(self):
        self.process.terminate()
        try:
            self.process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.log.close()


def mean(values):
    values = [value for value in values if value is not None]
    return round(statistics.mean(values), 1) if values else None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--server-bin", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--wire", required=True, help="a capture directory of NNNN-model-wire.json requests")
    parser.add_argument("--out", required=True)
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--ctx", type=int, default=8192)
    parser.add_argument("--port", type=int, default=18130)
    parser.add_argument("--save-with", default="0000")
    parser.add_argument("--measure", default="0015")
    parser.add_argument("--between", default="0016")
    parser.add_argument("--rounds", type=int, default=2)
    args = parser.parse_args()
    # The server runs in its own folder, as the app starts it, so every path it is given is absolute.
    args.server_bin, args.model = os.path.abspath(args.server_bin), os.path.abspath(args.model)
    os.makedirs(args.out, exist_ok=True)
    args.slots = os.path.join(os.path.abspath(args.out), "slots")
    shutil.rmtree(args.slots, ignore_errors=True)
    os.makedirs(args.slots)
    request = {name: os.path.join(args.wire, f"{name}-model-wire.json") for name in (args.save_with, args.measure, args.between)}
    for path in request.values():
        if not os.path.isfile(path):
            raise SystemExit(f"no capture {path}")
    key = secrets.token_hex(32)
    result = {"threads": args.threads, "save_with": args.save_with, "measure": args.measure, "between": args.between}

    server = Server(args, "save", key)
    try:
        result["save_first_read"] = server.read(args.save_with, request[args.save_with])
        result["save"] = server.slot("save")
        result["save"]["file_bytes"] = os.path.getsize(os.path.join(args.slots, SLOT_FILE))
        result["save"]["load_s"] = server.load_s
    finally:
        server.stop()

    result["rounds"] = []
    for round_ in range(1, args.rounds + 1):
        row = {"round": round_}
        server = Server(args, f"cold-r{round_}", key)
        try:
            row["cold_load_s"] = server.load_s
            row["cold"] = server.read(args.measure, request[args.measure])
            if round_ == 1:
                server.read(args.between, request[args.between])
                result["steady"] = server.read(args.measure, request[args.measure])
        finally:
            server.stop()
        server = Server(args, f"restored-r{round_}", key)
        try:
            row["restored_load_s"] = server.load_s
            row["restore"] = server.slot("restore")
            row["restored"] = server.read(args.measure, request[args.measure])
        finally:
            server.stop()
        result["rounds"].append(row)

    steady = result.get("steady", {}).get("content")
    cold = [row["cold"] for row in result["rounds"]]
    restored = [row["restored"] for row in result["rounds"]]
    restores = [row["restore"] for row in result["rounds"]]
    cold_prompt = mean(read["prompt_ms"] for read in cold)
    restored_prompt = mean(read["prompt_ms"] for read in restored)
    restore_ms = mean(entry.get("wall_ms") for entry in restores)
    summary = {
        "slot_file_bytes": result["save"].get("file_bytes"),
        "saved_tokens": result["save"].get("n_saved"),
        "save_ms": (result["save"].get("timings") or {}).get("save_ms"),
        "restore_ms": mean((entry.get("timings") or {}).get("restore_ms") for entry in restores),
        "restore_wall_ms": restore_ms,
        "restore_ok": all(entry.get("status") == 200 for entry in restores),
        "load_s": mean([row["cold_load_s"] for row in result["rounds"]] + [row["restored_load_s"] for row in result["rounds"]]),
        "cold_cache_n": [read["cache_n"] for read in cold],
        "restored_cache_n": [read["cache_n"] for read in restored],
        "steady_cache_n": result.get("steady", {}).get("cache_n"),
        "cold_prompt_n": mean(read["prompt_n"] for read in cold),
        "restored_prompt_n": mean(read["prompt_n"] for read in restored),
        "cold_prompt_ms": cold_prompt,
        "restored_prompt_ms": restored_prompt,
        "steady_prompt_ms": result.get("steady", {}).get("prompt_ms"),
        "cold_latency_s": mean(read["latency_s"] for read in cold),
        "restored_latency_s": mean(read["latency_s"] for read in restored),
        "first_document_saving_s": (round((cold_prompt - restored_prompt - (restore_ms or 0)) / 1000, 1)
                                    if cold_prompt is not None and restored_prompt is not None else None),
        "restored_identical_to_steady": [read["content"] == steady for read in restored] if steady is not None else None,
        "cold_identical_to_steady": [read["content"] == steady for read in cold] if steady is not None else None,
        "errors": sum(read["status"] != 200 for read in cold + restored),
    }
    result["summary"] = summary
    with open(os.path.join(args.out, "cold-start.json"), "w", encoding="utf-8") as file:
        json.dump(result, file, indent=1)
    shutil.rmtree(args.slots, ignore_errors=True)
    print(json.dumps(summary))


if __name__ == "__main__":
    main()

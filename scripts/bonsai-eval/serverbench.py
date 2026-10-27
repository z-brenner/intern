"""Start llama-server with given flags, time start-up, send Intern-shaped requests,
and record server timings, wall time, outputs and memory.

python3 -I serverbench.py --server BIN --model GGUF --threads N --label L --out OUT.jsonl
       [--extra "flags ..."] [--requests a.json,b.json] [--repeat 1] [--nice 0]
       [--logprobs] [--idle SECONDS] [--evict]

Mirrors the app's launch (crates/intern-engine/src/server.rs): one slot, CPU only,
8192 context, --jinja, --no-webui, --no-mmproj, threads == threads-batch.
Peak RSS is the kernel's VmHWM for the server process; RSS is also sampled every 50 ms.
"""
import argparse
import json
import os
import secrets
import shlex
import subprocess
import threading
import time
import urllib.request
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--server", required=True)
ap.add_argument("--model", required=True)
ap.add_argument("--threads", type=int, default=2)
ap.add_argument("--label", required=True)
ap.add_argument("--out", required=True)
ap.add_argument("--extra", default="")
ap.add_argument("--requests", required=True)
ap.add_argument("--repeat", type=int, default=1)
ap.add_argument("--nice", type=int, default=0)
ap.add_argument("--port", type=int, default=18311)
ap.add_argument("--ctx", type=int, default=8192)
ap.add_argument("--logprobs", action="store_true")
ap.add_argument("--warmup", default="")
ap.add_argument("--idle", type=float, default=0.0,
                help="seconds to stay idle after the requests, measuring the server's idle CPU")
ap.add_argument("--evict", action="store_true",
                help="drop the model file from the page cache first (cold start)")
args = ap.parse_args()

opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
key = secrets.token_hex(16)
cmd = [args.server, "--host", "127.0.0.1", "--port", str(args.port), "--api-key", key,
       "--model", args.model, "--parallel", "1", "--ctx-size", str(args.ctx), "--n-gpu-layers", "0",
       "--threads", str(args.threads), "--threads-batch", str(args.threads),
       "--jinja", "--no-webui", "--no-mmproj"] + shlex.split(args.extra)
if args.evict:
    fd = os.open(args.model, os.O_RDONLY)
    os.posix_fadvise(fd, 0, 0, os.POSIX_FADV_DONTNEED)
    os.close(fd)
log_path = Path(args.out).with_suffix(f".{args.label}.server.log")
log = open(log_path, "w")
t0 = time.perf_counter()
proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT,
                        preexec_fn=(lambda: os.nice(args.nice)) if args.nice else None)


def status(field):
    try:
        for line in open(f"/proc/{proc.pid}/status"):
            if line.startswith(field + ":"):
                return int(line.split()[1]) / 1024.0  # MiB
    except OSError:
        return None


def cpu_s():
    """user+system CPU seconds of the server process (all threads)."""
    try:
        f = open(f"/proc/{proc.pid}/stat").read().rsplit(")", 1)[1].split()
        return (int(f[11]) + int(f[12])) / os.sysconf("SC_CLK_TCK")
    except OSError:
        return None


samples = []
stop = threading.Event()


def sampler():
    while not stop.is_set():
        r = status("VmRSS")
        if r is not None:
            samples.append((time.perf_counter() - t0, r, status("RssAnon"), status("RssFile")))
        time.sleep(0.05)


threading.Thread(target=sampler, daemon=True).start()


def get(path):
    req = urllib.request.Request(f"http://127.0.0.1:{args.port}{path}",
                                 headers={"Authorization": f"Bearer {key}"})
    with opener.open(req, timeout=5) as r:
        return r.status, r.read()


startup = None
while time.perf_counter() - t0 < 300:
    if proc.poll() is not None:
        break
    try:
        code, body = get("/health")
        if code == 200 and b"ok" in body:
            startup = time.perf_counter() - t0
            break
    except Exception:
        pass
    time.sleep(0.05)

result = {"label": args.label, "cmd": [c if c != key else "<key>" for c in cmd], "nice": args.nice,
          "threads": args.threads, "startup_s": startup}
if startup is None:
    result["error"] = "server did not become healthy"
    proc.kill()
    stop.set()
    with open(args.out, "a") as f:
        f.write(json.dumps(result) + "\n")
    raise SystemExit(1)
result["rss_after_load_mib"] = status("VmRSS")
result["rss_anon_after_load_mib"] = status("RssAnon")


def post(body):
    data = json.dumps(body).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{args.port}/v1/chat/completions", data=data,
                                 headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
    t = time.perf_counter()
    with opener.open(req, timeout=1800) as r:
        payload = json.loads(r.read())
    return time.perf_counter() - t, payload


runs = []
files = [f for f in args.requests.split(",") if f]
if args.warmup:
    body = json.loads(Path(args.warmup).read_text())
    wall, payload = post(body)
    result["warmup_wall_s"] = wall
for rep in range(args.repeat):
    for f in files:
        body = json.loads(Path(f).read_text())
        if args.logprobs:
            body["logprobs"] = True
            body["top_logprobs"] = 1
        rss_before = status("VmRSS")
        cpu_before = cpu_s()
        wall, payload = post(body)
        cpu_used = cpu_s() - cpu_before
        msg = payload["choices"][0]["message"]
        entry = {"request": Path(f).name, "rep": rep, "wall_s": wall,
                 "timings": payload.get("timings"), "usage": payload.get("usage"),
                 "finish_reason": payload["choices"][0].get("finish_reason"),
                 "content": msg.get("content"), "rss_before_mib": rss_before,
                 "rss_after_mib": status("VmRSS"), "server_cpu_s": cpu_used}
        if args.logprobs:
            entry["logprobs"] = payload["choices"][0].get("logprobs")
        runs.append(entry)
if args.idle:
    c0 = cpu_s()
    time.sleep(args.idle)
    result["idle_s"] = args.idle
    result["idle_cpu_s"] = cpu_s() - c0
stop.set()
result["peak_rss_mib"] = status("VmHWM")
result["rss_end_mib"] = status("VmRSS")
result["rss_anon_end_mib"] = status("RssAnon")
result["max_sampled_rss_mib"] = max((s[1] for s in samples), default=None)
result["max_sampled_rss_anon_mib"] = max((s[2] or 0 for s in samples), default=None)
result["max_sampled_rss_file_mib"] = max((s[3] or 0 for s in samples), default=None)
result["evicted_before_start"] = args.evict
result["runs"] = runs
proc.terminate()
try:
    proc.wait(timeout=30)
except subprocess.TimeoutExpired:
    proc.kill()
with open(args.out, "a") as f:
    f.write(json.dumps(result) + "\n")
s = [r["timings"] for r in runs if r.get("timings")]
print(f"{args.label}: startup {startup:.2f}s rss_load {result['rss_after_load_mib']:.0f} peak {result['peak_rss_mib']:.0f} "
      f"anon_peak {result['max_sampled_rss_anon_mib']:.0f} end {result['rss_end_mib']:.0f} MiB idle_cpu {result.get('idle_cpu_s')}")
for r in runs:
    t = r["timings"] or {}
    print(f"  {r['request']:40s} wall {r['wall_s']:7.2f}s  pp {t.get('prompt_n')}@{t.get('prompt_per_second', 0):.1f}/s  "
          f"tg {t.get('predicted_n')}@{t.get('predicted_per_second', 0):.2f}/s  draft {t.get('draft_n')}/{t.get('draft_n_accepted')}  "
          f"cache {t.get('cache_n')}  cpu {r['server_cpu_s']:.1f}s")

#!/usr/bin/env python3
"""ATLAS teacher job: answers captured requests with one GGUF model on a rented GPU. Laboratory tooling.

Runs inside ghcr.io/ggml-org/llama.cpp:full-cuda-b11946 (the app's own llama.cpp build, with
CUDA), started by Hugging Face Jobs. It is the only part of the teacher study that needs a GPU;
capture and scoring run free on GitHub runners (.github/workflows/atlas-teacher.yml). It:

  1. fetches teacher/answer.py and each requests file from the public repo at ATLAS_COMMIT;
  2. downloads MODEL_REPO/MODEL_FILE at MODEL_REVISION and checks MODEL_SHA256 when given;
  3. starts llama-server on the GPU and answers every request through answer.py;
  4. prints each answers file, gzip and base64, between ATLAS-ANSWERS markers in the job log,
     so nothing is written to any account; teacher/unpack_log.py turns the log back into files.

Environment: ATLAS_COMMIT (40 hex), ATLAS_REQUESTS (comma-separated repo paths of
requests.jsonl.gz), MODEL_REPO, MODEL_FILE, MODEL_REVISION (default main), MODEL_SHA256
(optional), PARALLEL (default 4), CTX_PER_SLOT (default 8192), STOP_AFTER_MINUTES (default 0:
none; the job's own timeout is the hard stop). Sends no document anywhere but the local server.
"""
import base64
import gzip
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.request

REPO = "z-brenner/intern"
AGENT = "Intern ATLAS research (https://github.com/z-brenner/intern)"


def log(message):
    print(time.strftime("%H:%M:%S"), message, flush=True)


def fetch(url, path):
    request = urllib.request.Request(url, headers={"User-Agent": AGENT})
    with urllib.request.urlopen(request, timeout=120) as response, open(path, "wb") as file:
        while chunk := response.read(1 << 20):
            file.write(chunk)


def main():
    commit = os.environ["ATLAS_COMMIT"]
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        sys.exit("ATLAS_COMMIT must be a full commit hash")
    requests = [p for p in os.environ["ATLAS_REQUESTS"].split(",") if p]
    for path in requests:
        if not re.fullmatch(r"research/atlas-models/results/teacher/[a-z0-9._/-]+\.jsonl\.gz", path):
            sys.exit(f"refusing requests path {path!r}")
    repo, file = os.environ["MODEL_REPO"], os.environ["MODEL_FILE"]
    revision = os.environ.get("MODEL_REVISION", "main")
    expected = os.environ.get("MODEL_SHA256", "")
    parallel = int(os.environ.get("PARALLEL", "4"))
    ctx = int(os.environ.get("CTX_PER_SLOT", "8192"))
    work = "/tmp/atlas"
    os.makedirs(work, exist_ok=True)
    started = time.monotonic()

    raw = f"https://raw.githubusercontent.com/{REPO}/{commit}"
    fetch(f"{raw}/research/atlas-models/teacher/answer.py", f"{work}/answer.py")
    local = []
    for path in requests:
        target = f"{work}/{path.replace('/', '__')}"
        fetch(f"{raw}/{path}", target)
        local.append((path, target))
    log(f"fetched answer.py and {len(local)} requests files at {commit}")

    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        api = urllib.request.Request(f"https://huggingface.co/api/models/{repo}/revision/{revision}",
                                     headers={"User-Agent": AGENT})
        revision = json.load(urllib.request.urlopen(api, timeout=60))["sha"]
    model = f"{work}/model.gguf"
    url = f"https://huggingface.co/{repo}/resolve/{revision}/{file}"
    log(f"downloading {url}")
    subprocess.run(["curl", "--fail", "--location", "--silent", "--show-error", "--retry", "3",
                    "--user-agent", AGENT, "--output", model, url], check=True)
    digest = hashlib.sha256()
    with open(model, "rb") as handle:
        while chunk := handle.read(1 << 24):
            digest.update(chunk)
    actual = digest.hexdigest()
    if expected and actual != expected:
        sys.exit(f"SHA-256 mismatch: expected {expected}, got {actual}")
    info = {"repo": repo, "revision": revision, "file": file, "sha256": actual,
            "bytes": os.path.getsize(model), "pinned": bool(expected)}
    log("model " + json.dumps(info))
    subprocess.run(["nvidia-smi", "--query-gpu=name,memory.total", "--format=csv"], check=False)

    key = hashlib.sha256(os.urandom(16)).hexdigest()
    server = subprocess.Popen(["/app/llama-server", "--host", "127.0.0.1", "--port", "8080", "--api-key", key,
                               "--model", model, "--parallel", str(parallel), "--ctx-size", str(ctx * parallel),
                               "--n-gpu-layers", "999", "--jinja", "--no-webui", "--no-mmproj", "--cache-ram", "0"])
    for _ in range(1200):
        try:
            urllib.request.urlopen("http://127.0.0.1:8080/health", timeout=5)
            break
        except Exception:  # noqa: BLE001 - loading
            if server.poll() is not None:
                sys.exit("llama-server exited while loading")
            time.sleep(1)
    else:
        sys.exit("llama-server did not become healthy")
    log("llama-server ready")

    for path, target in local:
        name = path.rsplit("/", 2)[-2]
        out = f"{work}/answers-{name}.json"
        subprocess.run([sys.executable, "-I", f"{work}/answer.py", "--endpoint", "http://127.0.0.1:8080",
                        "--api-key", key, "--requests", target, "--out", out, "--parallel", str(parallel)],
                       check=True)
        packed = base64.b64encode(gzip.compress(open(out, "rb").read(), 9)).decode("ascii")
        print(f"=====ATLAS-ANSWERS-BEGIN {name}=====", flush=True)
        for at in range(0, len(packed), 4096):
            print(packed[at:at + 4096], flush=True)
        print(f"=====ATLAS-ANSWERS-END {name} sha256={hashlib.sha256(packed.encode()).hexdigest()}=====", flush=True)

    server.terminate()
    print("=====ATLAS-MODEL " + json.dumps(info) + "=====", flush=True)
    log(f"done in {round((time.monotonic() - started) / 60, 1)} minutes")


if __name__ == "__main__":
    main()

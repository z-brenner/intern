"""Run a pinned Qwen 2B/4B ABBA comparison through Intern's actual engine.

No downloads, source rewrites, default changes, or release operations.
Needs an already provisioned llama.cpp/PDFium/OCR runtime and verified GGUFs.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from compare import compare, digest, load

ROOT = Path(__file__).resolve().parents[2]
SERVER_ARGS = "--cache-ram 0 -ub 256 --prio -1 --poll 0"


def tree_hash(files, base):
    entries = [(str(p.relative_to(base)), digest(p)) for p in sorted(files)]
    return hashlib.sha256(json.dumps(entries, separators=(",", ":")).encode()).hexdigest()


def preflight(models, corpus, gold_path):
    pins = load(Path(__file__).with_name("candidates.json"))["models"]
    for pin in pins:
        path = models / pin["file"]
        if not path.is_file() or path.stat().st_size != pin["size"] or digest(path) != pin["sha256"]:
            raise ValueError(f"missing or unverified model: {path}")
    gold = load(gold_path)
    ids = [d["id"] for d in gold["documents"]]
    if not ids or len(ids) != len(set(ids)):
        raise ValueError("empty or duplicate gold cohort")
    files = []
    for document in gold["documents"]:
        if document.get("gold", {}).get("expected_readiness") not in (None, "ready", "needs_review", "either"):
            raise ValueError(f"unrecognized expected_readiness: {document['id']}")
        path = (corpus / document["file"]).resolve()
        if not path.is_relative_to(corpus) or not path.is_file():
            raise ValueError(f"missing or unsafe corpus file: {document['file']}")
        files.append(path)
    return pins, tree_hash(files, corpus)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--models", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--gold", type=Path, required=True)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--server", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--worker", type=Path, default=ROOT / "target/release/intern-worker")
    parser.add_argument("--bench", type=Path, default=ROOT / "target/release/intern-bench")
    parser.add_argument("--threads", type=int, default=4, choices=range(1, 13))
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--preflight-only", action="store_true")
    args = parser.parse_args()
    try:
        pins, corpus_hash = preflight(args.models.resolve(), args.corpus.resolve(), args.gold)
        for path in (args.server, args.worker, args.bench):
            if not path.is_file() or not os.access(path, os.X_OK):
                raise ValueError(f"missing executable: {path}")
        if not args.runtime.is_dir():
            raise ValueError("missing parser runtime directory")
        # Hash the server and all sibling runtime libraries, not a version label alone.
        runtime_files = [p for p in args.server.resolve().parent.rglob("*") if p.is_file()]
        provenance = {
            "git_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "runtime_sha256": tree_hash(runtime_files, args.server.resolve().parent),
            "worker_sha256": digest(args.worker), "bench_sha256": digest(args.bench),
            "parser_runtime_sha256": tree_hash([p for p in args.runtime.resolve().rglob("*") if p.is_file()], args.runtime.resolve()),
            "gold_sha256": digest(args.gold), "corpus_sha256": corpus_hash,
            "profile_sha256": digest(Path(__file__).with_name("profile.json")),
            "threads": args.threads, "server_args": SERVER_ARGS,
            "server_version": subprocess.check_output([str(args.server.resolve()), "--version"], stderr=subprocess.STDOUT, text=True),
            "cgroup_cpu_max": Path("/sys/fs/cgroup/cpu.max").read_text().strip(),
            "cgroup_memory_max": Path("/sys/fs/cgroup/memory.max").read_text().strip(),
        }
        if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True):
            raise ValueError("commit tracked changes before measuring; provenance must name the measured code")
        if args.preflight_only:
            print(json.dumps({"models_verified": [p["id"] for p in pins], "provenance": provenance}, indent=2))
            return
        # An exclusive run prevents this runner from benchmarking against itself.
        import fcntl
        (ROOT / "target").mkdir(exist_ok=True)
        with (ROOT / "target/cpu-model-eval.lock").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            args.out.mkdir(parents=True, exist_ok=False)
            env = os.environ.copy()
            env.update({"CORPUS": str(args.corpus.resolve()), "GOLD": str(args.gold.resolve()),
                        "MANIFEST": str(args.manifest.resolve()) if args.manifest else "",
                        "LLAMA_SERVER": str(args.server.resolve()), "INTERN_RUNTIME_DIR": str(args.runtime.resolve()),
                        "INTERN_WORKER": str(args.worker.resolve()), "INTERN_BENCH": str(args.bench.resolve()),
                        "THREADS": str(args.threads), "LLAMA_SERVER_ARGS": SERVER_ARGS,
                        "BENCH_PROFILE": str(Path(__file__).with_name("profile.json")),
                        "SAMPLE_RSS": "1", "NICE": "0"})
            for name, pin in (("control1", pins[0]), ("candidate1", pins[1]),
                              ("candidate2", pins[1]), ("control2", pins[0])):
                directory = args.out.resolve() / name
                env["MODEL"] = str(args.models.resolve() / pin["file"])
                print(f"{name}: {pin['id']}", flush=True)
                with (args.out / f"{name}.log").open("w") as log:
                    status = subprocess.run([str(ROOT / "scripts/run-internbench-candidate.sh"),
                                             str(directory), "--", "--min-token-confidence", "0.5"],
                                            cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT).returncode
                # Keep partial artifacts even when execution fails; comparison rejects them.
                if directory.is_dir():
                    (directory / "provenance.json").write_text(json.dumps({**provenance, "model_sha256": pin["sha256"],
                                                                          "exit_code": status}, indent=2))
                if status:
                    raise ValueError(f"{name} failed ({status}); partial artifacts retained")
            result = compare(args.gold, [args.out / n for n in ("control1", "candidate1", "control2", "candidate2")])
            (args.out / "comparison.json").write_text(json.dumps(result, indent=2))
            print(json.dumps(result, indent=2))
            if not result["screening_pass"]:
                raise SystemExit(1)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(2, f"qualification stopped: {error}\n")


if __name__ == "__main__":
    main()

"""Fail-closed screening of two repeated live InternBench arms, never release approval.

python3 -I scripts/cpu-model-eval/compare.py GOLD.json CONTROL1 CANDIDATE1 CONTROL2 CANDIDATE2
Each run directory needs report.json, rss.json, startup.txt and provenance.json.
Run provenance is written by run.py. Missing evidence is an error, not a zero.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def positive(value):
    return type(value) in (int, float) and math.isfinite(value) and value > 0


def percentile(values, q):
    return sorted(values)[max(0, math.ceil(len(values) * q) - 1)]


def read_run(directory, gold, gold_hash):
    root = Path(directory)
    report = load(root / "report.json")
    provenance = load(root / "provenance.json")
    if provenance.get("exit_code") != 0:
        raise ValueError(f"{root}: benchmark exit was not successful")
    if report.get("mode") != "live" or report.get("timings_source") != "measured":
        raise ValueError(f"{root}: requires live measured results")
    if report.get("schema_version") != 1 or report.get("suite") != "internbench":
        raise ValueError(f"{root}: unsupported report schema")
    if report.get("corpus", {}).get("gold_sha256") != gold_hash:
        raise ValueError(f"{root}: gold differs")
    if provenance.get("gold_sha256") != gold_hash:
        raise ValueError(f"{root}: provenance gold differs")
    if report.get("git_commit") != provenance.get("git_commit"):
        raise ValueError(f"{root}: report code differs from provenance")
    if report.get("model", {}).get("sha256") != provenance.get("model_sha256"):
        raise ValueError(f"{root}: report model differs from provenance")
    ids = [d["id"] for d in gold["documents"]]
    records = report.get("records", [])
    if len(ids) != len(set(ids)) or len(records) != len(ids):
        raise ValueError(f"{root}: duplicate or incomplete cohort")
    if {r["id"] for r in records} != set(ids):
        raise ValueError(f"{root}: cohort differs")
    for record in records:
        if record.get("status") != "completed" or record.get("stale") or record.get("replayed"):
            raise ValueError(f"{root}: incomplete/stale/replayed document {record['id']}")
        if record.get("readiness") not in ("ready", "needs_review"):
            raise ValueError(f"{root}: missing readiness")
        if not all(positive(record.get("timings", {}).get(k)) for k in ("inference_ms", "total_ms")):
            raise ValueError(f"{root}: missing measured latency")
    rss = load(root / "rss.json").get("peak_total_mib")
    if not positive(rss):
        raise ValueError(f"{root}: missing sampled process memory")
    startup = (root / "startup.txt").read_text().split()
    if len(startup) != 2 or startup[0] != "startup_s" or not positive(float(startup[1])):
        raise ValueError(f"{root}: missing process startup latency")
    return {"directory": str(root), "provenance": provenance, "machine": report.get("machine"),
            "records": {r["id"]: r for r in records}, "peak_total_mib": rss,
            "process_startup_s": float(startup[1])}


def compare(gold_path, directories):
    gold = load(gold_path)
    for document in gold["documents"]:
        if document.get("gold", {}).get("expected_readiness") not in (None, "ready", "needs_review", "either"):
            raise ValueError(f"unrecognized expected_readiness: {document['id']}")
    if len({Path(d).resolve() for d in directories}) != 4:
        raise ValueError("four distinct run directories are required")
    runs = [read_run(d, gold, digest(gold_path)) for d in directories]
    first = runs[0]
    # The runner hashes executable files and complete model-runtime library trees.
    for run in runs:
        for key in ("git_commit", "runtime_sha256", "worker_sha256", "bench_sha256",
                    "parser_runtime_sha256", "corpus_sha256", "threads", "server_args",
                    "profile_sha256", "cgroup_cpu_max", "cgroup_memory_max"):
            if not first["provenance"].get(key) or run["provenance"].get(key) != first["provenance"][key]:
                raise ValueError(f"same-machine comparison requires identical {key}")
        if not first["machine"] or run["machine"] != first["machine"]:
            raise ValueError("machine differs or is unavailable")
    for left, right in ((0, 2), (1, 3)):
        if runs[left]["provenance"]["model_sha256"] != runs[right]["provenance"]["model_sha256"]:
            raise ValueError("repeat uses a different model")
    if runs[0]["provenance"]["model_sha256"] == runs[1]["provenance"]["model_sha256"]:
        raise ValueError("control and candidate must differ")

    reasons, pairs = [], []
    for control, candidate in ((runs[0], runs[1]), (runs[2], runs[3])):
        metrics = {}
        for key in ("date_correct", "parties_correct", "readiness_match", "filename_correct",
                    "type_correct", "description_factual", "description_specific",
                    "date_forbidden", "party_forbidden", "unsafe_ready"):
            a = {i: r["scores"][key] for i, r in control["records"].items() if key in r["scores"]}
            b = {i: r["scores"][key] for i, r in candidate["records"].items() if key in r["scores"]}
            if set(a) != set(b) or any(type(v) is not bool for v in [*a.values(), *b.values()]):
                raise ValueError(f"score denominator/shape differs for {key}")
            metrics[key] = {"control": sum(a.values()), "candidate": sum(b.values()), "total": len(a)}
            if key in ("date_correct", "parties_correct"):
                losses = [i for i in a if a[i] and not b[i]]
                if not a or losses:
                    reasons.append(f"{key}: absent score or per-document regressions {losses}")
            if key in ("date_forbidden", "party_forbidden") and any(b.values()):
                reasons.append(f"candidate {key} is not zero")
            if key == "unsafe_ready" and (not a or sum(b.values()) > sum(a.values())):
                reasons.append("wrong names ready absent or increased")
            if key in ("filename_correct", "type_correct", "description_factual", "description_specific"):
                if sum(b.values()) < sum(a.values()):
                    reasons.append(f"{key}: total correct decreased")
        gain = any(metrics[k]["candidate"] > metrics[k]["control"]
                   for k in ("date_correct", "parties_correct", "readiness_match"))
        if not gain:
            reasons.append("no strict critical-field/routing gain in this repeat")
        timing = {}
        for key in ("inference_ms", "total_ms"):
            a = [r["timings"][key] for r in control["records"].values()]
            b = [r["timings"][key] for r in candidate["records"].values()]
            timing[key] = {"control_p50": statistics.median(a), "candidate_p50": statistics.median(b),
                           "control_p95": percentile(a, .95), "candidate_p95": percentile(b, .95)}
            if statistics.median(b) > 1.5 * statistics.median(a):
                reasons.append(f"{key}: median exceeds 1.5x control")
        # Keep 4 GiB of the 16 GiB machine for the desktop, browser and OS.
        if candidate["peak_total_mib"] > 12 * 1024:
            reasons.append("sampled process memory exceeds 12 GiB")
        pairs.append({"metrics": metrics, "latency": timing,
                      "control_peak_total_mib": control["peak_total_mib"],
                      "candidate_peak_total_mib": candidate["peak_total_mib"],
                      "control_process_startup_s": control["process_startup_s"],
                      "candidate_process_startup_s": candidate["process_startup_s"]})
    return {"screening_pass": not reasons, "release_qualified": False,
            "reasons": reasons, "pairs": pairs,
            "limits": "Linux screening only. Requires source audit, all heldouts, Windows packaged-runtime qualification and release evidence. Startup is process-cold, not disk-cold. RSS sampling may miss brief peaks."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gold")
    parser.add_argument("runs", nargs=4, metavar="RUN_DIR")
    args = parser.parse_args()
    try:
        result = compare(args.gold, args.runs)
    except (ValueError, KeyError, OSError, TypeError) as error:
        parser.exit(2, f"invalid qualification evidence: {error}\n")
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["screening_pass"] else 1)


if __name__ == "__main__":
    main()

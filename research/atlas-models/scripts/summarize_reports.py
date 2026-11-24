#!/usr/bin/env python3
"""Tallies InternBench report.json files (plain or .gz) into one comparison table.

Laboratory tooling; the app does not call it. Each argument is LABEL=PATH, or a directory that
holds <label>-<set>/report.json(.gz) as the atlas-research workflow writes them. Usage:

    python3 -I research/atlas-models/scripts/summarize_reports.py \
        research/atlas-models/results/2026-11-21-screen1 [--json OUT.json]

Counts, per label and set: documents scored, names right (the filename scorer), type, date and
party right, names filed without review (ready), wrong names filed (unsafe_ready), and from
timings the p50 and p95 of the whole document and of prefill, with the server's peak memory.
"""
import gzip
import json
import os
import statistics
import sys


def load(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as file:
        return json.load(file)


def percentile(values, q):
    if not values:
        return None
    values = sorted(values)
    k = (len(values) - 1) * q
    lo, hi = int(k), min(int(k) + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def tally(report):
    rows = report.get("records", [])
    t = dict(docs=0, completed=0, right=0, type=0, date=0, parties=0, ready=0, unsafe=0,
             total_ms=[], prefill_ms=[], generation_ms=[], prompt_tokens=[], generated=[])
    for r in rows:
        t["docs"] += 1
        s = r.get("scores") or {}
        if r.get("status") == "completed":
            t["completed"] += 1
        t["right"] += bool(s.get("filename_correct"))
        t["type"] += bool(s.get("type_correct"))
        t["date"] += bool(s.get("date_correct"))
        t["parties"] += bool(s.get("parties_correct"))
        t["ready"] += bool(s.get("ready"))
        t["unsafe"] += bool(s.get("unsafe_ready"))
        tm = r.get("timings") or {}
        for key, out in (("total_ms", "total_ms"), ("prefill_ms", "prefill_ms"),
                         ("generation_ms", "generation_ms"), ("prompt_tokens", "prompt_tokens"),
                         ("generated_tokens", "generated")):
            if isinstance(tm.get(key), (int, float)) and tm.get("model_requests", 1):
                t[out].append(float(tm[key]))
    peak = ((report.get("memory") or {}).get("peak") or {}).get("server_mb")
    return t, peak


def gather(args):
    found = []
    for arg in args:
        if "=" in arg and not os.path.isdir(arg):
            label, path = arg.split("=", 1)
            found.append((label, "", path))
            continue
        for entry in sorted(os.listdir(arg)):
            for name in ("report.json.gz", "report.json"):
                path = os.path.join(arg, entry, name)
                if os.path.exists(path):
                    label, _, set_name = entry.rpartition("-on-")
                    found.append((label or entry, set_name, path))
                    break
    return found


def main(argv):
    out_json = None
    args = []
    it = iter(argv)
    for a in it:
        if a == "--json":
            out_json = next(it)
        else:
            args.append(a)
    rows = []
    for label, set_name, path in gather(args):
        t, peak = tally(load(path))
        rows.append(dict(
            label=label, set=set_name, docs=t["docs"], completed=t["completed"], right=t["right"],
            type=t["type"], date=t["date"], parties=t["parties"], ready=t["ready"],
            unsafe_ready=t["unsafe"],
            p50_s=round(percentile(t["total_ms"], 0.5) / 1000, 2) if t["total_ms"] else None,
            p95_s=round(percentile(t["total_ms"], 0.95) / 1000, 2) if t["total_ms"] else None,
            prefill_p50_s=round(percentile(t["prefill_ms"], 0.5) / 1000, 2) if t["prefill_ms"] else None,
            decode_tok_s=round(sum(t["generated"]) / (sum(t["generation_ms"]) / 1000), 2)
            if t["generation_ms"] and sum(t["generation_ms"]) else None,
            mean_prompt_tokens=round(statistics.mean(t["prompt_tokens"])) if t["prompt_tokens"] else None,
            peak_server_mb=peak,
        ))
    head = ["label", "set", "docs", "right", "type", "date", "parties", "ready", "unsafe_ready",
            "p50_s", "p95_s", "prefill_p50_s", "decode_tok_s", "peak_server_mb"]
    print("| " + " | ".join(head) + " |")
    print("|" + "---|" * len(head))
    for row in sorted(rows, key=lambda r: (r["set"], r["label"])):
        print("| " + " | ".join("" if row[h] is None else str(row[h]) for h in head) + " |")
    if out_json:
        with open(out_json, "w", encoding="utf-8") as file:
            json.dump(rows, file, indent=1)


if __name__ == "__main__":
    main(sys.argv[1:])

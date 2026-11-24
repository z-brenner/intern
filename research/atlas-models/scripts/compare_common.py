#!/usr/bin/env python3
"""Same-document comparison of InternBench runs. Laboratory tooling.

    python3 -I compare_common.py RESULTS_DIR [--sets real1,real2] [--json OUT]

RESULTS_DIR holds <label>-on-<set>/report.json.gz (atlas-research.yml / atlas-train.yml layout).
A set's documents can differ between jobs when a document fails to fetch, so every count here is
on the documents that every label in the directory scored for that set. Prints per label: right
names, files ready, unsafe ready, and, against the first label whose name contains "9b" (the
reference), how many documents each gets right that the reference misses and the reverse.
"""
import argparse
import gzip
import json
import os
import re
from collections import defaultdict


def load(results):
    runs = defaultdict(dict)
    for name in sorted(os.listdir(results)):
        match = re.fullmatch(r"(.+)-on-(.+)", name)
        path = os.path.join(results, name, "report.json.gz")
        if not match or not os.path.exists(path):
            continue
        label, dataset = match.groups()
        report = json.load(gzip.open(path, "rt", encoding="utf-8"))
        runs[dataset][label] = {r["id"]: (r.get("scores") or {}) for r in report.get("records", [])}
    return runs


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("results")
    parser.add_argument("--sets")
    parser.add_argument("--reference", default="9b")
    parser.add_argument("--json")
    args = parser.parse_args()
    runs = load(args.results)
    sets = args.sets.split(",") if args.sets else sorted(runs)
    labels = sorted({label for s in sets for label in runs.get(s, {})})
    reference = next((label for label in labels if args.reference in label), None)
    totals = {label: defaultdict(int) for label in labels}
    rows = []
    for dataset in sets:
        present = [label for label in labels if label in runs.get(dataset, {})]
        if len(present) != len(labels):
            print(f"{dataset}: missing {sorted(set(labels) - set(present))}, skipped")
            continue
        common = set.intersection(*(set(runs[dataset][label]) for label in labels))
        for label in labels:
            scores = runs[dataset][label]
            right = {d for d in common if scores[d].get("filename_correct")}
            ref_right = {d for d in common if runs[dataset][reference][d].get("filename_correct")} if reference else set()
            row = {"set": dataset, "label": label, "docs": len(common), "right": len(right),
                   "ready": sum(1 for d in common if scores[d].get("ready")),
                   "unsafe_ready": sum(1 for d in common if scores[d].get("unsafe_ready")),
                   "right_not_ref": len(right - ref_right), "ref_not_right": len(ref_right - right)}
            rows.append(row)
            for key in ("docs", "right", "ready", "unsafe_ready", "right_not_ref", "ref_not_right"):
                totals[label][key] += row[key]
    print(f"| label | docs | right | % | ready | unsafe ready | right where {reference} wrong | wrong where {reference} right |")
    print("|---|---|---|---|---|---|---|---|")
    for label in labels:
        t = totals[label]
        pct = 100 * t["right"] / t["docs"] if t["docs"] else 0
        print(f"| {label} | {t['docs']} | {t['right']} | {pct:.1f} | {t['ready']} | {t['unsafe_ready']} | {t['right_not_ref']} | {t['ref_not_right']} |")
    if args.json:
        json.dump({"reference": reference, "rows": rows, "totals": totals}, open(args.json, "w"), indent=1)


if __name__ == "__main__":
    main()

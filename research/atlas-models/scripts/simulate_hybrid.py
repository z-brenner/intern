#!/usr/bin/env python3
"""Adaptive hybrid replay: a small model reads every document, the 9B re-reads the ones it is
unsure about. Laboratory tooling; no model is run.

    python3 -I simulate_hybrid.py RESULTS_DIR --quick LABEL [--reference 9b] [--ratio 0.23]
                                  [--json OUT]

RESULTS_DIR holds <label>-on-<set>/report.json.gz from one run (atlas-research.yml layout), so
the quick model and the 9B read the same documents on the same engine. A document is escalated
when the quick model's least probable chosen token (token_confidence.min) is under a threshold;
the script sweeps the threshold. For an escalated document the 9B's reading counts, otherwise the
quick model's, with each model's own readiness (the app's 0.7 gate).

Cost, relative to the 9B reading everything: ratio + escalated share, where ratio is the quick
model's measured time per document over the 9B's on one runner (BENCHMARK_RESULTS.md). This
assumes an escalated document costs a full 9B reading on top of the quick one, which is what
the app's cascade does.

The threshold sweep is fitted on the same documents it is scored on, so the best row is an
optimistic estimate. A threshold chosen here has to be confirmed on documents it was not chosen
on before it means anything.
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
        runs[dataset][label] = {r["id"]: r for r in report.get("records", [])}
    return runs


def reading(record):
    scores = record.get("scores") or {}
    confidence = (record.get("token_confidence") or {}).get("min")
    return {
        "right": bool(scores.get("filename_correct")),
        "ready": bool(scores.get("ready")),
        "unsafe": bool(scores.get("unsafe_ready")),
        "confidence": 0.0 if confidence is None else float(confidence),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("results")
    parser.add_argument("--quick", required=True)
    parser.add_argument("--reference", default="9b")
    parser.add_argument("--ratio", type=float, required=True,
                        help="quick model's time per document over the 9B's, same runner")
    parser.add_argument("--budget", type=float, default=0.6)
    parser.add_argument("--json")
    args = parser.parse_args()

    runs = load(args.results)
    docs = []
    for dataset, labels in sorted(runs.items()):
        reference = next((label for label in labels if args.reference in label and label != args.quick), None)
        if args.quick not in labels or reference is None:
            continue
        quick, ref = labels[args.quick], labels[reference]
        for doc in sorted(set(quick) & set(ref)):
            docs.append((dataset, doc, reading(quick[doc]), reading(ref[doc])))
    n = len(docs)
    if not n:
        raise SystemExit("no documents shared by the quick model and the reference")

    def total(key, pick):
        return sum(1 for d in docs if pick(d)[key])

    ref_right = total("right", lambda d: d[3])
    ref_unsafe = total("unsafe", lambda d: d[3])
    quick_right = total("right", lambda d: d[2])
    quick_unsafe = total("unsafe", lambda d: d[2])

    rows = []
    thresholds = sorted({0.0, 1.01} | {round(d[2]["confidence"], 4) for d in docs})
    for t in thresholds:
        escalated = [d for d in docs if d[2]["confidence"] < t]
        kept = [d for d in docs if d[2]["confidence"] >= t]
        right = sum(d[3]["right"] for d in escalated) + sum(d[2]["right"] for d in kept)
        ready = sum(d[3]["ready"] for d in escalated) + sum(d[2]["ready"] for d in kept)
        unsafe = sum(d[3]["unsafe"] for d in escalated) + sum(d[2]["unsafe"] for d in kept)
        share = len(escalated) / n
        rows.append({"threshold": t, "escalated": len(escalated), "share": round(share, 4),
                     "cost": round(args.ratio + share, 4), "right": right, "ready": ready,
                     "unsafe": unsafe})

    # Oracle: escalate exactly the documents where the 9B is right and the quick model is not.
    oracle = [d for d in docs if d[3]["right"] and not d[2]["right"]]
    oracle_right = quick_right + len(oracle)

    within = [r for r in rows if r["cost"] <= args.budget]
    best = max(within, key=lambda r: (r["right"], -r["unsafe"])) if within else None

    print(f"documents: {n}; quick {args.quick}: {quick_right} right, {quick_unsafe} unsafe; "
          f"reference: {ref_right} right, {ref_unsafe} unsafe; ratio {args.ratio}")
    print(f"oracle (escalate exactly the quick model's fixable misses): {oracle_right} right, "
          f"{len(oracle)} escalated ({len(oracle) / n:.0%}), cost {args.ratio + len(oracle) / n:.2f}")
    max_share = max(0.0, args.budget - args.ratio)
    print(f"budget {args.budget}: at most {max_share:.0%} of documents can be escalated")
    print("| threshold | escalated | share | cost vs 9B | right | ready | unsafe ready |")
    print("|---|---|---|---|---|---|---|")
    shown = set()
    for r in rows:
        bucket = round(r["share"] * 20)
        if bucket in shown and r is not best:
            continue
        shown.add(bucket)
        mark = " (best within budget)" if r is best else ""
        print(f"| {r['threshold']:.3f}{mark} | {r['escalated']} | {r['share']:.0%} | {r['cost']:.2f} | "
              f"{r['right']} | {r['ready']} | {r['unsafe']} |")
    passes = best is not None and best["right"] >= ref_right and best["unsafe"] <= ref_unsafe
    print(f"passes the switch rule: {'yes' if passes else 'no'}")
    if args.json:
        json.dump({"documents": n, "quick": args.quick, "ratio": args.ratio,
                   "reference_right": ref_right, "reference_unsafe": ref_unsafe,
                   "quick_right": quick_right, "quick_unsafe": quick_unsafe,
                   "oracle_right": oracle_right, "oracle_escalated": len(oracle),
                   "best_within_budget": best, "passes": passes, "rows": rows},
                  open(args.json, "w"), indent=1)


if __name__ == "__main__":
    main()

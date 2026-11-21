#!/usr/bin/env python3
"""Offline 2B -> 9B cascade simulation from the November 2026 qualification readings.

Reads bench/reports/model-qualification-2026-11/readings.json (alpha.13 engine replays) and asks:
if the quick model reads every document first and only the documents it would hold for review
go on to the accurate model, how many names come out right, how many wrong names are filed
without review, and what share of the 9B's time is spent?

No model is run. Cost uses the measured p50 latency ratio from the same report (same Xeon,
4 threads): 2B 14.9 s, 4B 37.3 s, 9B 68.2 s per document. Usage:

    python3 -I research/atlas-models/scripts/simulate_cascade.py [READINGS.json] [--quick q2b|q4b]
"""
import json
import sys

P50 = {"q2b": 14.9, "q4b": 37.3, "q9b": 68.2}


def main(argv):
    path = "bench/reports/model-qualification-2026-11/readings.json"
    quick = "q2b"
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--quick":
            quick = args.pop(0)
        else:
            path = a
    readings = json.load(open(path))
    rows = []
    totals = None
    for name, s in readings["sets"].items():
        t = dict(docs=0, right9=0, rightq=0, rightc=0, wrong9=0, wrongq=0, wrongc=0,
                 auto9=0, autoq=0, autoc=0, escalated=0)
        for doc in s["documents"].values():
            q, b = doc.get(quick), doc.get("q9b")
            if not q or not b:
                continue
            t["docs"] += 1
            q_ready = q.get("readiness") == "ready"
            b_ready = b.get("readiness") == "ready"
            t["right9"] += bool(b.get("right"))
            t["rightq"] += bool(q.get("right"))
            t["auto9"] += b_ready
            t["autoq"] += q_ready
            t["wrong9"] += b_ready and not b.get("right")
            t["wrongq"] += q_ready and not q.get("right")
            if q_ready:
                c = q
            else:
                t["escalated"] += 1
                c = b
            c_ready = c.get("readiness") == "ready"
            t["rightc"] += bool(c.get("right"))
            t["autoc"] += c_ready
            t["wrongc"] += c_ready and not c.get("right")
        rows.append((name, t))
        if totals is None:
            totals = {k: 0 for k in t}
        for k in t:
            totals[k] += t[k]
    rows.append(("all", totals))
    ratio = P50[quick] / P50["q9b"]
    print(f"quick={quick} (p50 {P50[quick]} s) escalating held readings to q9b (p50 {P50['q9b']} s)")
    print("set          docs | right 9B quick casc | wrong-filed 9B quick casc | auto-filed 9B quick casc | escalated  time vs 9B")
    for name, t in rows:
        n = t["docs"] or 1
        cost = ratio + t["escalated"] / n
        print(f"{name:12s} {t['docs']:4d} | {t['right9']:8d} {t['rightq']:5d} {t['rightc']:4d} | "
              f"{t['wrong9']:14d} {t['wrongq']:5d} {t['wrongc']:4d} | {t['auto9']:13d} {t['autoq']:5d} {t['autoc']:4d} | "
              f"{t['escalated']:4d} ({t['escalated']/n:4.0%})  {cost:5.2f}")


if __name__ == "__main__":
    main(sys.argv[1:])

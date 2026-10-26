"""Per-document comparison of InternBench reports and 2B -> 4B routing analysis.

python3 -I analyze-routing.py OUT_DIR --b2 2B.json --b4 4B.json [--ref NAME=report.json ...]
        [--swap-s SECONDS] [--b2-time 2B_timing.json] [--b4-time 4B_timing.json]

Signals are taken only from what the app has after the 2B pass (its readiness and
review reasons, its reply, validation, the worker's route and page count, prompt and
reply lengths, token confidence) - never from the gold. Cost model: every document
pays extraction once (model-independent) and the 2B inference; an escalated document
also pays the 4B inference of the same prompt (checked: prompt SHA-256 equal) and,
per document, one model swap (unload 2B, load 4B); a batched router pays two swaps
per batch instead. --b2-time/--b4-time take inference times from other reports of the
same documents (e.g. 2-thread runs) while the scores come from --b2/--b4.

Writes OUT_DIR/per-document.csv, OUT_DIR/routing.json and OUT_DIR/routing.md.
"""
import argparse
import csv
import json
import random
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("out")
ap.add_argument("--b2", required=True)
ap.add_argument("--b4", required=True)
ap.add_argument("--b2-time")
ap.add_argument("--b4-time")
ap.add_argument("--ref", action="append", default=[])
ap.add_argument("--swap-s", type=float, default=0.0)
args = ap.parse_args()
out = Path(args.out)
out.mkdir(parents=True, exist_ok=True)


def load(p):
    return {r["id"]: r for r in json.loads(Path(p).read_text())["records"]}


B2, B4 = load(args.b2), load(args.b4)
T2 = load(args.b2_time) if args.b2_time else B2
T4 = load(args.b4_time) if args.b4_time else B4
refs = {n: load(p) for n, p in (x.split("=", 1) for x in args.ref)}
ids = [d for d in B2 if d in B4]
md = []


def s(r, k):
    return (r.get("scores") or {}).get(k)


def t(r, k):
    return ((r.get("timings") or {}).get(k) or 0.0)


def infer_s(r):
    return t(r, "inference_ms") / 1000


def extract_s(r):
    return t(r, "extraction_wall_ms") / 1000


def ready(r):
    return r.get("readiness") == "ready"


def reasons(r):
    return [x if isinstance(x, str) else json.dumps(x) for x in (r.get("review_reasons") or [])]


def summary(R, T=None):
    T = T or R
    rr = [R[d] for d in ids]
    k = lambda key: sum(1 for r in rr if s(r, key))
    return {"docs": len(rr), "filename_correct": k("filename_correct"), "ready": sum(map(ready, rr)),
            "review": sum(1 for r in rr if not ready(r)), "unsafe_ready": k("unsafe_ready"),
            "needless_review": k("needless_review"), "readiness_match": k("readiness_match"),
            "type": k("type_correct"), "date": k("date_correct"), "parties": k("parties_correct"),
            "relation": k("relation_correct"), "trap_date": k("date_forbidden"),
            "forbidden_party": k("party_forbidden"),
            "failed": sum(1 for r in rr if r.get("status") != "completed"),
            "infer_s": sum(infer_s(T[d]) for d in ids), "extract_s": sum(extract_s(T[d]) for d in ids)}


rows = []
for d in ids:
    a, b = B2[d], B4[d]
    tc = a.get("token_confidence") or {}
    mp = a.get("model_proposal") or {}
    facts = mp.get("facts") or {}
    row = {"id": d, "kind": a.get("kind"), "format": a.get("format"), "text_layer": a.get("text_layer"),
           "pages": a.get("pages"), "route_class": a.get("route_class"),
           "same_prompt": a.get("prompt_sha256") == b.get("prompt_sha256")}
    for n, r in (("b2", a), ("b4", b)):
        row.update({f"{n}_filename": r.get("filename"), f"{n}_correct": s(r, "filename_correct"),
                    f"{n}_ready": ready(r), f"{n}_unsafe": s(r, "unsafe_ready"),
                    f"{n}_needless": s(r, "needless_review"), f"{n}_type": s(r, "type_correct"),
                    f"{n}_date": s(r, "date_correct"), f"{n}_parties": s(r, "parties_correct"),
                    f"{n}_relation": s(r, "relation_correct"), f"{n}_reasons": "|".join(reasons(r)),
                    f"{n}_infer_s": round(infer_s(r), 2), f"{n}_prompt_tokens": t(r, "prompt_tokens"),
                    f"{n}_cached_tokens": t(r, "cached_tokens"), f"{n}_gen_tokens": t(r, "generated_tokens"),
                    f"{n}_status": r.get("status")})
    row["b2_conf_min"] = tc.get("min")
    row["b2_conf_mean"] = tc.get("mean")
    row["b4_conf_min"] = (b.get("token_confidence") or {}).get("min")
    row["extract_s"] = round(extract_s(a), 2)
    row["gold_expected"] = a.get("readiness_expected") or ""
    rows.append(row)
for n, R in refs.items():
    for row in rows:
        r = R.get(row["id"])
        row[f"{n}_correct"] = s(r, "filename_correct") if r else None
        row[f"{n}_ready"] = ready(r) if r else None
with open(out / "per-document.csv", "w", newline="") as f:
    w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
    w.writeheader()
    w.writerows(rows)


def signals(d):
    r = B2[d]
    mp = r.get("model_proposal") or {}
    facts = mp.get("facts") or {}
    parties = facts.get("parties") or []
    rs = reasons(r)
    tc = r.get("token_confidence") or {}
    sc = r.get("scores") or {}
    sig = {
        "review": not ready(r),
        "route_ocr": r.get("route_class") in ("ocr", "ocr_regions"),
        "pages>=5": (r.get("pages") or 0) >= 5,
        "pages>=10": (r.get("pages") or 0) >= 10,
        "not_pdf": r.get("format") != "pdf",
        "no_date": not facts.get("document_date"),
        "no_type": not facts.get("document_type"),
        "parties<2": len(parties) < 2,
        "parties>=3": len(parties) >= 3,
        "party_role_other": any(p.get("role") == "other" for p in parties),
        "date_role_other": facts.get("date_role") in (None, "other"),
        "date_role_execution": facts.get("date_role") == "execution",
        "prompt>1500": t(r, "prompt_tokens") > 1500,
        "facts_unsupported>0": (sc.get("facts_unsupported") or 0) > 0,
        "unknown_or_miscited_ids>0": ((sc.get("unknown_ids") or 0) + (sc.get("miscited_ids") or 0)) > 0,
        "cited_support<1": (sc.get("cited_support_rate") if sc.get("cited_support_rate") is not None else 1) < 1,
    }
    for reason in ("TYPE_UNSUPPORTED", "TYPE_INFERRED", "DATE_AMBIGUOUS", "DATE_UNSUPPORTED",
                   "PARTY_UNSUPPORTED", "DATE_MISSING", "TYPE_MISSING", "DATE_IS_DEADLINE",
                   "DESCRIPTION_UNSUPPORTED", "LOW_CONFIDENCE", "PARSER_WARNING"):
        sig["r:" + reason] = any(x.startswith(reason) for x in rs)
    if tc:
        for th in (0.3, 0.5, 0.7, 0.8, 0.9, 0.95):
            sig[f"conf_min<{th}"] = tc["min"] < th
        for th in (0.9, 0.95, 0.98):
            sig[f"conf_mean<{th}"] = tc["mean"] < th
    return sig


SIG = {d: signals(d) for d in ids}
names = list(SIG[ids[0]].keys())
bad2 = {d for d in ids if not s(B2[d], "filename_correct")}
gain = {d for d in ids if not s(B2[d], "filename_correct") and s(B4[d], "filename_correct")}
loss = {d for d in ids if s(B2[d], "filename_correct") and not s(B4[d], "filename_correct")}
unsafe2 = {d for d in ids if s(B2[d], "unsafe_ready")}


def evaluate(esc_set):
    k = dict(escalated=len(esc_set), correct=0, ready=0, review=0, unsafe=0, needless=0, match=0)
    infer = extract = 0.0
    for d in ids:
        final = B4[d] if d in esc_set else B2[d]
        k["correct"] += bool(s(final, "filename_correct"))
        k["ready"] += ready(final)
        k["review"] += not ready(final)
        k["unsafe"] += bool(s(final, "unsafe_ready"))
        k["needless"] += bool(s(final, "needless_review"))
        k["match"] += bool(s(final, "readiness_match"))
        infer += infer_s(T2[d]) + (infer_s(T4[d]) if d in esc_set else 0)
        extract += extract_s(T2[d])
    k["infer_s"] = infer
    k["total_s"] = infer + extract
    k["total_s_swap_per_doc"] = infer + extract + 2 * args.swap_s * len(esc_set)
    return k


def four_always():
    k = dict(escalated=len(ids), correct=0, ready=0, review=0, unsafe=0, needless=0, match=0)
    for d in ids:
        r = B4[d]
        k["correct"] += bool(s(r, "filename_correct"))
        k["ready"] += ready(r)
        k["review"] += not ready(r)
        k["unsafe"] += bool(s(r, "unsafe_ready"))
        k["needless"] += bool(s(r, "needless_review"))
        k["match"] += bool(s(r, "readiness_match"))
    k["infer_s"] = sum(infer_s(T4[d]) for d in ids)
    k["total_s"] = k["infer_s"] + sum(extract_s(T2[d]) for d in ids)
    k["total_s_swap_per_doc"] = k["total_s"]
    return k


policies = {"2B always": evaluate(set()), "4B always": four_always()}
for n in names:
    policies[f"2B, escalate if {n}"] = evaluate({d for d in ids if SIG[d][n]})
combos = [("review", x) for x in names if x != "review" and not x.startswith("r:")]
for a_, b_ in combos:
    policies[f"2B, escalate if review OR {b_}"] = evaluate({d for d in ids if SIG[d][a_] or SIG[d][b_]})
policies["oracle: escalate exactly where 4B fixes the name"] = evaluate(gain)

# Signal quality: how each signal separates "escalation helps" (gain) from the rest.
sigq = {}
for n in names:
    on = {d for d in ids if SIG[d][n]}
    sigq[n] = {"fires": len(on), "on_2b_wrong": len(on & bad2), "on_gain": len(on & gain),
               "on_loss": len(on & loss), "on_unsafe2b": len(on & unsafe2),
               "recall_gain": len(on & gain) / len(gain) if gain else None,
               "recall_unsafe2b": len(on & unsafe2) / len(unsafe2) if unsafe2 else None}


# Paired bootstrap over documents: P(policy better than 2B always / than 4B always) on correct names.
def boot(esc_a, esc_b, n=4000, seed=7):
    rnd = random.Random(seed)
    def fin(esc, d):
        if esc == "all":
            return bool(s(B4[d], "filename_correct"))
        return bool(s((B4 if d in esc else B2)[d], "filename_correct"))
    diffs = []
    for _ in range(n):
        sample = [rnd.choice(ids) for _ in ids]
        diffs.append(sum(fin(esc_a, d) - fin(esc_b, d) for d in sample))
    diffs.sort()
    return {"mean": sum(diffs) / n, "ci95": [diffs[int(0.025 * n)], diffs[int(0.975 * n)]]}


result = {"ids": len(ids), "same_prompt": sum(r["same_prompt"] for r in rows),
          "summary": {"b2": summary(B2, T2), "b4": summary(B4, T4), **{n: summary(R) for n, R in refs.items() if all(d in R for d in ids)}},
          "gain": sorted(gain), "loss": sorted(loss), "unsafe_2b": sorted(unsafe2),
          "policies": policies, "signals": sigq,
          "bootstrap_4b_vs_2b": boot("all", set()),
          "bootstrap_review_escalation_vs_2b": boot({d for d in ids if SIG[d]["review"]}, set()),
          "bootstrap_review_escalation_vs_4b": boot({d for d in ids if SIG[d]["review"]}, "all")}
(out / "routing.json").write_text(json.dumps(result, indent=1, default=str))

L = md.append
L(f"Documents: {len(ids)}; prompt SHA-256 identical 2B vs 4B: {result['same_prompt']}/{len(ids)}\n")
L("| Run | Filename | Ready | Review | Unsafe ready | Needless review | Readiness match | Type | Date | Parties | Relation | Trap date | Forbidden party | Failed | Sum inference s | Sum extraction s |")
L("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
for n, x in result["summary"].items():
    L(f"| {n} | {x['filename_correct']}/{x['docs']} | {x['ready']} | {x['review']} | {x['unsafe_ready']} | {x['needless_review']} | {x['readiness_match']} | {x['type']} | {x['date']} | {x['parties']} | {x['relation']} | {x['trap_date']} | {x['forbidden_party']} | {x['failed']} | {x['infer_s']:.0f} | {x['extract_s']:.0f} |")
L(f"\n4B fixes the 2B name on {len(gain)}: {', '.join(sorted(gain))}")
L(f"\n4B breaks a correct 2B name on {len(loss)}: {', '.join(sorted(loss))}")
L(f"\n2B unsafe ready ({len(unsafe2)}): {', '.join(sorted(unsafe2))}\n")
L("| Policy | Escalated | Filename | Ready | Review | Unsafe ready | Needless review | Readiness match | Sum inference s | Sum total s | + per-doc swaps s |")
L("|---|---|---|---|---|---|---|---|---|---|---|")
for n, k in policies.items():
    if n.startswith("2B, escalate if") and k["escalated"] == 0:
        continue
    L(f"| {n} | {k['escalated']} | {k['correct']} | {k['ready']} | {k['review']} | {k['unsafe']} | {k['needless']} | {k['match']} | {k['infer_s']:.0f} | {k['total_s']:.0f} | {k['total_s_swap_per_doc']:.0f} |")
L("\n| Signal (from the 2B pass) | Fires | on 2B-wrong | on 4B-fixes | on 4B-breaks | on 2B-unsafe |")
L("|---|---|---|---|---|---|")
for n, q in sigq.items():
    if q["fires"]:
        L(f"| {n} | {q['fires']} | {q['on_2b_wrong']} | {q['on_gain']} | {q['on_loss']} | {q['on_unsafe2b']} |")
L("\nPaired bootstrap (4000 resamples of the documents), difference in correct filenames:")
for k_ in ("bootstrap_4b_vs_2b", "bootstrap_review_escalation_vs_2b", "bootstrap_review_escalation_vs_4b"):
    L(f"- {k_}: mean {result[k_]['mean']:+.2f}, 95% interval {result[k_]['ci95']}")
L("\n| Doc | kind | layer | route | pages | 2B name ok | 2B ready | 4B name ok | 4B ready | 2B reasons | 2B conf min | 2B filename | 4B filename |")
L("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
for row in rows:
    d = row["id"]
    if d in bad2 or d in loss or d in unsafe2 or not row["b4_correct"] or row["b4_unsafe"]:
        cm = f"{row['b2_conf_min']:.2f}" if row["b2_conf_min"] is not None else ""
        L(f"| {d} | {row['kind']} | {row['text_layer']} | {row['route_class']} | {row['pages']} | {row['b2_correct']} | {row['b2_ready']} | {row['b4_correct']} | {row['b4_ready']} | {row['b2_reasons']} | {cm} | {row['b2_filename']} | {row['b4_filename']} |")
(out / "routing.md").write_text("\n".join(md) + "\n")
print("\n".join(md))

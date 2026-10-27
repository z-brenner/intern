"""Readiness-rule variants for 2B->4B routing (take 4B / sticky review / require agreement) and a
2-fold cross-validation of the rule choice. python3 -I routing-readiness-rules.py RESULTS_DIR bench/gold.json
Reads RESULTS_DIR/internbench/{2b,4b}-t4-app/report.json, writes RESULTS_DIR/routing/policies-readiness.json."""
import json, sys, random
R = sys.argv[1]; gold = sys.argv[2]
G = {d["id"]: d["gold"].get("expected_readiness") for d in json.load(open(gold))["documents"]}
def load(p): return {r["id"]: r for r in json.load(open(p))["records"]}
B2 = load(f"{R}/internbench/2b-t4-app/report.json"); B4 = load(f"{R}/internbench/4b-t4-app/report.json")
ids = list(B2)
ok = lambda r: bool(r["scores"].get("filename_correct"))
rdy = lambda r: r["readiness"] == "ready"
fn = lambda r: (r.get("filename") or "").lower()
def pc(r): return len(((r.get("model_proposal") or {}).get("facts") or {}).get("parties") or [])
def conf(r): return (r.get("token_confidence") or {}).get("min", 1.0)
def outcome(name_ok, ready, d):
    exp = G[d]
    return dict(correct=name_ok, ready=ready, unsafe=ready and not name_ok,
                needless=(not ready) and name_ok and exp == "ready",
                match=(exp == "either") or (exp == "ready" and ready) or (exp == "needs_review" and not ready) if exp else None)
def run(esc, mode):
    tot = dict(correct=0, ready=0, unsafe=0, needless=0, match=0, escalated=0, infer=0.0)
    for d in ids:
        a, b = B2[d], B4[d]
        e = esc(d)
        tot["infer"] += a["timings"]["inference_ms"] / 1000 + (b["timings"]["inference_ms"] / 1000 if e else 0)
        if not e:
            o = outcome(ok(a), rdy(a), d)
        elif mode == "take4b":
            o = outcome(ok(b), rdy(b), d)
        elif mode == "sticky":   # 4B's name, but never clears a 2B review flag
            o = outcome(ok(b), rdy(b) and rdy(a), d)
        elif mode == "agree":    # 4B's name, ready only if both models ready and agree on the name
            o = outcome(ok(b), rdy(a) and rdy(b) and fn(a) == fn(b), d)
        tot["escalated"] += e
        for k in ("correct", "ready", "unsafe", "needless", "match"):
            tot[k] += bool(o[k])
    return tot
sigs = {
  "never (2B always)": lambda d: False,
  "always (both models on every doc)": lambda d: True,
  "2B review": lambda d: not rdy(B2[d]),
  "2B parties<2": lambda d: pc(B2[d]) < 2,
  "2B review OR parties<2": lambda d: (not rdy(B2[d])) or pc(B2[d]) < 2,
  "2B conf_min<0.7": lambda d: conf(B2[d]) < 0.7,
  "2B review OR conf_min<0.7": lambda d: (not rdy(B2[d])) or conf(B2[d]) < 0.7,
  "2B review OR parties<2 OR conf_min<0.7": lambda d: (not rdy(B2[d])) or pc(B2[d]) < 2 or conf(B2[d]) < 0.7,
}
print("| Escalate when | Readiness rule | Escalated | Filename | Ready | Unsafe ready | Needless review | Readiness match | Sum inference s |")
print("|---|---|---|---|---|---|---|---|---|")
out = {}
for sn, sf in sigs.items():
    for mode in ("take4b", "sticky", "agree"):
        if sn.startswith("never") and mode != "take4b": continue
        t = run(sf, mode); out[f"{sn} / {mode}"] = t
        print(f"| {sn} | {mode} | {t['escalated']} | {t['correct']} | {t['ready']} | {t['unsafe']} | {t['needless']} | {t['match']} | {t['infer']:.0f} |")
t = dict(correct=sum(ok(B4[d]) for d in ids), ready=sum(rdy(B4[d]) for d in ids), unsafe=sum(rdy(B4[d]) and not ok(B4[d]) for d in ids))
print(f"| 4B only | 4B | 77 | {t['correct']} | {t['ready']} | {t['unsafe']} | | | {sum(B4[d]['timings']['inference_ms'] for d in ids)/1000:.0f} |")
# agreement as a correctness signal
agree = [d for d in ids if fn(B2[d]) == fn(B4[d])]
print(f"\nNames agree on {len(agree)}/77; of those correct {sum(ok(B4[d]) for d in agree)}; disagree correct(4B) {sum(ok(B4[d]) for d in ids if d not in agree)}/{77-len(agree)}")
# 2-fold cross-validation of choosing the best single-signal rule (by correct - 2*unsafe) among sigs x modes
rnd = random.Random(1); res = []
for rep in range(200):
    sh = ids[:]; rnd.shuffle(sh); A, Bh = set(sh[:38]), set(sh[38:])
    def score(sub, sf, mode):
        global ids
        keep = ids; ids = [d for d in keep if d in sub]; t = run(sf, mode); ids = keep
        return t["correct"] - 2 * t["unsafe"]
    best = max(((sn, m) for sn in sigs for m in ("take4b", "sticky", "agree")), key=lambda k: score(A, sigs[k[0]], k[1]))
    base = score(Bh, sigs["never (2B always)"], "take4b")
    res.append((best, score(Bh, sigs[best[0]], best[1]) - base))
from collections import Counter
print("\n2-fold CV (200 random splits): rule picked on one half, gain in (correct - 2*unsafe) over 2B-always on the other half")
print(Counter(r[0] for r in res).most_common(5))
g = sorted(r[1] for r in res); print("held-out gain: median", g[len(g)//2], "5%", g[10], "95%", g[189])
json.dump(out, open(f"{R}/routing/policies-readiness.json", "w"), indent=1)

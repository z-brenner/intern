"""Readiness policies simulated over replay records + token traces.
Usage: policies.py <replay report dir> <traced artifacts dir> <set-model>...
A policy maps a row (features.load) to ready True/False. Rows with no gold verdict are skipped in counts."""
import json, sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from features import load, norm
from trace import fields

LC = "LOW_CONFIDENCE"

def others(r):
    return set(r["reasons"]) - {LC}

def tc_min(r):
    return (r.get("tc") or {}).get("min")

def passes(r, g, key=tc_min):
    v = key(r)
    return v is None or v >= g

def p_today(g):
    return lambda r: not others(r) and passes(r, g)

def p_r1(g):
    def f(r):
        o = others(r)
        rel = r.get("relation")
        n = len(r.get("final_parties") or [])
        if rel == "none" and n > 0:
            return False
        ok = (not o) or (o == {"PARTY_UNSUPPORTED"} and rel == "from" and n == 1)
        return ok and passes(r, g)
    return f

def is_caption_between(r):
    t = (r.get("final_type") or "").lower()
    return r.get("relation") == "between" and len(r.get("final_parties") or []) == 2 and "notice" in t

def p_c1(g):
    return lambda r: not others(r) and (passes(r, g) or is_caption_between(r))

CLOSER = re.compile(r'^[\s.,;:)\]]*"')

def tcv2(r, order=True, closers=True, used_only=False):
    """Least probable date or party-name token, leaving out (order) a name's first token whose
    runner-up begins another party the reply names, and (closers) a token that only closes the name."""
    tr = r.get("_trace")
    if not tr:
        return tc_min(r)
    text, values, f = r["_fields"]
    names = [values.get(f"party{i}") for i in range(3)]
    worst = 1.0
    seen = False
    for k in ["date"] + [f"party{i}" for i in range(3)]:
        toks = f.get(k)
        if not toks: continue
        if used_only and k.startswith("party") and int(k[5]) not in (r.get("used") or []):
            continue
        for j, (tok, p, a, ap, over) in enumerate(toks):
            if k.startswith("party"):
                me = int(k[5])
                if closers and CLOSER.match(tok):
                    continue
                if order and j == 0:
                    # every listed alternative (top-2) that is not the chosen token
                    alts = [x for x in r["_top"][k][j] if x[0] != tok]
                    rest = [n for i, n in enumerate(names) if n and i != me]
                    if alts and any(n.lstrip().startswith(alts[0][0].strip()) and alts[0][0].strip() for n in rest):
                        p = min(1.0, p + alts[0][1])
            seen = True
            worst = min(worst, p)
    return worst if seen else None

def p_t1(g, **kw):
    return lambda r: not others(r) and passes(r, g, key=lambda r: tcv2(r, **kw))

def attach_traces(rows, art_dir):
    by = {}
    for name in {r["set"] for r in rows}:
        rec = json.load(open(f"{art_dir}/{name}/recording.json"))
        for d in rec["documents"]:
            ex = d.get("exchanges") or []
            if ex and ex[-1]["reply"].get("token_trace"):
                by[(name, d["id"])] = ex[-1]["reply"]["token_trace"]
    for r in rows:
        tr = by.get((r["set"], r["id"]))
        r["_trace"] = tr
        if tr:
            r["_fields"] = fields(tr)
            # top lists per field token
            text = ""; spans = []
            for t in tr:
                s = len(text); text += t["token"]; spans.append((s, len(text)))
            from trace import scalars, name_of
            top = {}
            for path, a, b, _ in scalars(text):
                nm = name_of(path)
                if nm is None: continue
                top[nm] = [[(x["token"], x["p"]) for x in (t.get("top") or [])] for t, (s, e) in zip(tr, spans) if (s < b and a < e) or (a == b and s <= a < e)]
            r["_top"] = top

POLICIES = {
    "today@0.70": p_today(0.70),
    "today@0.80": p_today(0.80),
    "R1@0.70": p_r1(0.70),
    "R1@0.80": p_r1(0.80),
    "C1@0.70": p_c1(0.70),
    "C1@0.80": p_c1(0.80),
    "T1(order+closers)@0.70": p_t1(0.70),
    "T1(order+closers)@0.80": p_t1(0.80),
    "T1(closers only)@0.70": p_t1(0.70, order=False),
    "T1(order only)@0.70": p_t1(0.70, closers=False),
    "T1(used parties)@0.70": p_t1(0.70, used_only=True),
    "T1(used parties)@0.80": p_t1(0.80, used_only=True),
}

def tally(rows, pol):
    right = wrong = unscored = 0
    ids_wrong = []
    for r in rows:
        if not pol(r): continue
        if not r["scored"]: unscored += 1
        elif r["right"]: right += 1
        else: wrong += 1; ids_wrong.append(r["id"])
    return right, wrong, unscored, ids_wrong

if __name__ == "__main__":
    report_dir, art_dir = sys.argv[1], sys.argv[2]
    sets = sys.argv[3:]
    allrows = {}
    for name in sets:
        rows = load(report_dir, art_dir, name)
        attach_traces(rows, art_dir)
        allrows[name] = rows
    print(f"{'policy':28s} " + " ".join(f"{n:>14s}" for n in sets) + "   (right/wrong filed)")
    for pname, pol in POLICIES.items():
        cells = []; wrongs = []
        for name in sets:
            rgt, wrg, uns, ids = tally(allrows[name], pol)
            cells.append(f"{rgt:3d}/{wrg:<2d}" + (f"+{uns}u" if uns else "   "))
            wrongs += ids
        print(f"{pname:28s} " + " ".join(f"{c:>14s}" for c in cells) + ("   wrong: " + ",".join(wrongs) if wrongs else ""))
    for name in sets:
        rows = allrows[name]
        sc = [r for r in rows if r["scored"]]
        print(f"{name}: {len(rows)} completed, {len(sc)} scored, {sum(r['right'] for r in sc)} right; validation-clean: {sum(1 for r in sc if not others(r) and r['right'])} right / {sum(1 for r in sc if not others(r) and not r['right'])} wrong")

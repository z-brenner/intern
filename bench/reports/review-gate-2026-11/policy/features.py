"""features.py <replay report dir> <traced artifacts dir> <set-model> ... -> rows (list of dict) with per-field token stats."""
import json, sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from trace import fields, stats

def norm(s):
    return re.sub(r'[^a-z0-9]+', ' ', (s or '').lower()).strip()

def load(report_dir, art_dir, name):
    rep = json.load(open(f"{report_dir}/{name}.json"))
    rec = json.load(open(f"{art_dir}/{name}/recording.json"))
    traces = {}
    for d in rec["documents"]:
        ex = d.get("exchanges") or []
        if ex and ex[-1]["reply"].get("token_trace"):
            traces[d["id"]] = ex[-1]["reply"]["token_trace"]
    rows = []
    for r in rep["records"]:
        if r["status"] != "completed": continue
        sc = r["scores"]
        row = {"set": name, "id": r["id"], "right": bool(sc.get("filename_correct")), "scored": "filename_correct" in sc and sc.get("filename_correct") is not None,
               "ready": r["readiness"] == "ready", "reasons": r.get("review_reasons") or [], "filename": r.get("filename"),
               "type_ok": sc.get("type_correct"), "date_ok": sc.get("date_correct"), "parties_ok": sc.get("parties_correct"), "rel_ok": sc.get("relation_correct"),
               "tc": r.get("token_confidence"), "kind": r.get("kind"), "relation": (r.get("proposal") or {}).get("party_relation"),
               "final_parties": (r.get("proposal") or {}).get("parties") or [], "final_type": (r.get("proposal") or {}).get("document_type"),
               "final_date": (r.get("proposal") or {}).get("document_date"),
               "model_type": (r.get("model_proposal") or {}).get("document_type"), "model_date": (r.get("model_proposal") or {}).get("document_date"),
               "model_parties": (r.get("model_proposal") or {}).get("parties") or []}
        tr = traces.get(r["id"])
        row["has_trace"] = bool(tr)
        if tr:
            text, values, f = fields(tr)
            row["reply"] = text
            st = {k: stats(v) for k, v in f.items()}
            row["f"] = st
            row["values"] = values
            # which reply parties does the final filename use?
            used = []
            fp = [norm(p) for p in row["final_parties"]]
            for i in range(3):
                v = values.get(f"party{i}")
                if v is None: continue
                nv = norm(v)
                if any(nv and (nv in p or p in nv) for p in fp): used.append(i)
            row["used"] = used
            row["n_reply_parties"] = sum(1 for i in range(3) if f"party{i}" in values)
        rows.append(row)
    return rows

if __name__ == "__main__":
    report_dir, art_dir = sys.argv[1], sys.argv[2]
    rows = []
    for name in sys.argv[3:]:
        rows += load(report_dir, art_dir, name)
    json.dump(rows, sys.stdout)

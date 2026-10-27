"""Do two InternBench reports of the same documents give identical model replies?
Compares the model proposal (raw facts), the filename and readiness per shared document, and
inference time. python3 -I compare-runs.py A.json B.json"""
import json, sys
a = {r["id"]: r for r in json.load(open(sys.argv[1]))["records"]}
b = {r["id"]: r for r in json.load(open(sys.argv[2]))["records"]}
ids = [d for d in b if d in a]
same_prop = same_name = 0
ta = tb = 0.0
for d in ids:
    pa, pb = a[d].get("model_proposal"), b[d].get("model_proposal")
    sp = json.dumps(pa, sort_keys=True) == json.dumps(pb, sort_keys=True)
    sn = a[d].get("filename") == b[d].get("filename") and a[d].get("readiness") == b[d].get("readiness")
    same_prop += sp; same_name += sn
    ta += a[d]["timings"]["inference_ms"] / 1000; tb += b[d]["timings"]["inference_ms"] / 1000
    if not sp:
        print(f"DIFF {d}: {a[d].get('filename')} [{a[d].get('readiness')}] | {b[d].get('filename')} [{b[d].get('readiness')}]")
print(f"{len(ids)} shared documents: identical model reply {same_prop}, identical filename+readiness {same_name}; "
      f"sum inference A {ta:.0f} s, B {tb:.0f} s (B/A {tb/ta:.2f})")

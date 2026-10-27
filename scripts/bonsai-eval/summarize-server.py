"""Summarize serverbench.py JSONL: one row per label, and reply identity against a reference label.
python3 -I summarize-server.py server.jsonl [REF_LABEL=OTHER_LABEL ...]"""
import json, sys
rows = {}
for line in open(sys.argv[1]):
    d = json.loads(line)
    rows[d["label"]] = d
print("| Label | Threads | Extra flags | Startup s | RSS after load MiB | Peak RSS MiB | Peak anon MiB | Requests | Sum wall s | Prefill tok/s (mean) | Decode tok/s (mean) | Server CPU s | Idle CPU s | Draft accepted/drafted |")
print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
for lab, d in rows.items():
    runs = d.get("runs", [])
    t = [r["timings"] for r in runs if r.get("timings")]
    pp = sum(x["prompt_per_second"] for x in t) / len(t) if t else 0
    tg = sum(x["predicted_per_second"] for x in t) / len(t) if t else 0
    dn = sum(x.get("draft_n") or 0 for x in t); da = sum(x.get("draft_n_accepted") or 0 for x in t)
    cmd = d["cmd"]; extra = " ".join(cmd[cmd.index("--no-mmproj") + 1:]) if "--no-mmproj" in cmd else ""
    print(f"| {lab} | {d['threads']} | {extra} | {d['startup_s']:.2f} | {d['rss_after_load_mib']:.0f} | {d['peak_rss_mib']:.0f} | "
          f"{d.get('max_sampled_rss_anon_mib') or 0:.0f} | {len(runs)} | {sum(r['wall_s'] for r in runs):.1f} | {pp:.1f} | {tg:.2f} | "
          f"{sum(r.get('server_cpu_s') or 0 for r in runs):.0f} | {d.get('idle_cpu_s', '')} | {f'{da}/{dn}' if dn else ''} |")
for pair in sys.argv[2:]:
    a, b = pair.split("=")
    if a not in rows or b not in rows:
        continue
    ra = {r["request"]: r for r in rows[a]["runs"]}; rb = {r["request"]: r for r in rows[b]["runs"]}
    same = [k for k in rb if k in ra and ra[k]["content"] == rb[k]["content"]]
    diff = [k for k in rb if k in ra and ra[k]["content"] != rb[k]["content"]]
    wa = sum(ra[k]["wall_s"] for k in rb if k in ra); wb = sum(rb[k]["wall_s"] for k in rb if k in ra)
    ga = sum(ra[k]["timings"]["predicted_ms"] for k in rb if k in ra); gb = sum(rb[k]["timings"]["predicted_ms"] for k in rb if k in ra)
    print(f"\n{b} vs {a}: identical replies {len(same)}/{len(same)+len(diff)} {('differ: ' + ', '.join(diff)) if diff else ''}; "
          f"wall {wb:.1f} s vs {wa:.1f} s ({wb/wa:.3f}x); decode time {gb/1000:.1f} s vs {ga/1000:.1f} s ({gb/ga:.3f}x)")

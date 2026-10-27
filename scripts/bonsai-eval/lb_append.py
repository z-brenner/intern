import json
import sys

label, tmp, out = sys.argv[1:4]
with open(out, "a") as f:
    for line in open(tmp):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        d["label"] = label
        f.write(json.dumps(d) + "\n")
        test = f"pp{d['n_prompt']}" if d["n_prompt"] else f"tg{d['n_gen']}"
        print(f"{label:26s} {d['model_type'][:24]:24s} t={d['n_threads']} fa={d.get('flash_attn')} "
              f"b={d['n_batch']} ub={d['n_ubatch']} ctk={d['type_k']} d={d.get('n_depth', 0)} {test:7s} "
              f"{d['avg_ts']:8.2f} +- {d['stddev_ts']:.2f} tok/s")

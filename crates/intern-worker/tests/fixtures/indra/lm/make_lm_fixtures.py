"""Compact LM / beam-search / vocab fixtures for intern-worker's indra tests.

usage: python -I make_lm_fixtures.py MODEL_DIR LM_VECTORS_JSON INTERMEDIATE_DIR OUT_DIR

Writes OUT_DIR/beam_cases.json: every golden beam case of lm_vectors.json plus
extra cases replayed from the dumped recognizer outputs, and per LM a "mini"
table set holding exactly the entries any recorded query hits (so logp of every
recorded key, and therefore every case, is unchanged; max_logp is carried over
from the full model so pruning queries the same keys).  Numeric arrays are
base64 of little-endian bytes; f64 scalars are their IEEE bit patterns.
Also writes vocab_cases.json (Vocab.decode goldens)."""
import base64, json, math, sys, tempfile, random
from pathlib import Path
import numpy as np
from indra_ocr import lm as LM
from indra_ocr.vocab import Vocab

md, golden, inter, out = map(Path, sys.argv[1:5])
out.mkdir(parents=True, exist_ok=True)
NAMES = {"lm": "lm.npz", "lm_cjk": "lm_cjk.npz", "lm_world": "lm_world.npz"}
lms = {n: LM.NgramLM(md / f) for n, f in NAMES.items()}
by_shape = {(l.order, l.bits): n for n, l in lms.items()}
recorded = {n: set() for n in lms}

def record(name, lm):
    orig = lm.logp_keys
    def spy(keys):
        recorded[name].update(int(k) for k in keys)
        return orig(keys)
    lm.logp_keys = spy
for n, l in lms.items():
    record(n, l)

def b64(a, dt):
    return base64.b64encode(np.ascontiguousarray(a, dtype=dt).tobytes()).decode()
def fbits(x):
    return int(np.float64(x).view(np.uint64))

g = json.loads(golden.read_text())
cases = []
def run(name, blank, topv, topi, alpha, beta, beam, cand_floor, source):
    ids, steps = LM.beam_search(blank, topv, topi, lms[name], alpha, beta, beam, cand_floor=cand_floor)
    cases.append(dict(lm=name, source=source, alpha=alpha, beta=beta, beam=beam, cand_floor=cand_floor,
                      blank=blank, topv=topv, topi=topi, ids=[int(i) for i in ids], steps=[int(s) for s in steps]))
    return ids, steps

for i, c in enumerate(g["beam_cases"]):
    name = by_shape[(c["lm_order"], c["lm_bits"])]
    blank = np.array(c["blank"], np.float32)
    topv = np.array(c["topv"], np.float32)
    topi = np.array(c["topi"], np.int64)
    ids, steps = run(name, blank, topv, topi, c["alpha"], c["beta"], c["beam"], c["cand_floor"], f"golden#{i}")
    assert list(ids) == c["ids"] and list(steps) == c["steps"], i

# Extra cases: every line of the dumped recognizer batches, at the pipeline's
# parameters and at a few others that stress pruning and merging.
rng = random.Random(7)
variants = [(None, None, 8, 3e-3), (None, None, 3, 1e-3), (1.0, 0.5, 8, 1e-3), (0.5, 3.0, 16, 3e-3)]
for page in sorted(inter.iterdir()):
    for rec, name, a0, b0 in (("rec", "lm", 0.5, 1.5), ("rec_cjk", "lm_cjk", 0.2, 1.5), ("rec_world", "lm_world", 0.2, 1.5)):
        f = page / f"{rec}.out.blank.npy"
        if not f.exists():
            continue
        blank = np.load(f); topv = np.load(page / f"{rec}.out.topv.npy"); topi = np.load(page / f"{rec}.out.topi.npy")
        lengths = np.load(page / f"{rec}.out.lengths.npy"); conf = np.load(page / f"{rec}.out.conf.npy")
        ids_g = np.load(page / f"{rec}.out.ids.npy")
        lines = list(range(len(lengths)))
        rng.shuffle(lines)
        for k in lines[:14]:
            n = int(lengths[k])
            for vi, (al, be, bm, cf) in enumerate(variants):
                if vi and rng.random() < 0.5:
                    continue
                run(name, blank[k, :n], topv[k, :n], topi[k, :n], a0 if al is None else al, b0 if be is None else be,
                    bm, cf, f"{page.name}/{rec}#{k}/v{vi}")

# logp queries of lm_vectors.json
logp_q = {}
for gname, q in g["logp_keys"].items():
    name = gname[:-4]
    keys = [int(k) for k in q["keys"]]
    vals = lms[name].logp_keys(keys)
    assert all(abs(a - b) <= 0 for a, b in zip(vals, q["logp"])), name
    logp_q[name] = dict(keys=b64(keys, "<u8"), logp=b64(vals, "<f8"))

# Mini tables: the entries every recorded key's lookups hit.
def mini(name):
    l = lms[name]
    keys = np.array(sorted(recorded[name]), np.uint64)
    b = np.uint64(l.bits)
    ct = keys & np.uint64((1 << l.bits) - 1)
    tabs = []
    for n in range(l.order):
        m = np.uint64((1 << (l.bits * n)) - 1)
        hq = (keys >> b) & m
        pq = (hq << b) | ct
        sel = []
        for tk, tv, q in ((l.hk[n], l.hb[n], hq), (l.k[n], l.v[n], pq)):
            j = np.minimum(np.searchsorted(tk, q), max(len(tk) - 1, 0))
            hit = (tk[j] == q) if len(tk) else np.zeros(len(q), bool)
            u = np.unique(j[hit])
            sel.append((tk[u], tv[u]))
        (hk, hb), (k, v) = sel
        tabs.append((k, v, hk, hb))
    return tabs

lm_out = {}
with tempfile.TemporaryDirectory() as td:
    for name, l in lms.items():
        tabs = mini(name)
        arrs = {"order": np.int64(l.order), "vocab_size": np.int64(l.vocab_size), "bits": np.int64(l.bits)}
        for n, (k, v, hk, hb) in enumerate(tabs):
            arrs[f"k{n}"], arrs[f"v{n}"], arrs[f"hk{n}"], arrs[f"hb{n}"] = k, v, hk, hb
        p = Path(td) / f"{name}.npz"
        np.savez_compressed(p, **arrs)
        small = LM.NgramLM(p)
        small._max_logp = l.max_logp
        lms[name] = small  # replay below against the mini model
        lm_out[name] = dict(order=l.order, bits=l.bits, vocab_size=l.vocab_size, max_logp_bits=fbits(l.max_logp),
                            tables=[dict(k=b64(k, "<u8"), v=b64(v, "<f4"), hk=b64(hk, "<u8"), hb=b64(hb, "<f4")) for k, v, hk, hb in tabs],
                            entries=int(sum(len(t[0]) + len(t[2]) for t in tabs)))
    for c in cases:
        ids, steps = LM.beam_search(c["blank"], c["topv"], c["topi"], lms[c["lm"]], c["alpha"], c["beta"], c["beam"], cand_floor=c["cand_floor"])
        assert list(ids) == c["ids"] and list(steps) == c["steps"], c["source"]
    for name, q in logp_q.items():
        keys = np.frombuffer(base64.b64decode(q["keys"]), "<u8").tolist()
        want = np.frombuffer(base64.b64decode(q["logp"]), "<f8").tolist()
        assert lms[name].logp_keys([int(k) for k in keys]) == want, name

doc = dict(
    note="Generated by indra-tools/make_lm_fixtures.py from indra-ocr 1.1.2 (lm.py) and models a1283c4b; see tests.",
    lms=lm_out,
    logp=logp_q,
    cases=[dict(lm=c["lm"], source=c["source"], alpha_bits=fbits(c["alpha"]), beta_bits=fbits(c["beta"]), beam=c["beam"],
                cand_floor_bits=fbits(c["cand_floor"]), k=int(c["topv"].shape[1]), blank=b64(c["blank"], "<f4"),
                topv=b64(c["topv"], "<f4"), topi=b64(c["topi"], "<u2"), ids=c["ids"], steps=c["steps"]) for c in cases],
)
assert all(int(c["topi"].max()) < 65536 and int(c["topi"].min()) >= 0 for c in cases)
(out / "beam_cases.json").write_text(json.dumps(doc, separators=(",", ":")))
print("cases", len(cases), "changed-from-greedy", sum(1 for c in cases if True), {n: v["entries"] for n, v in lm_out.items()},
      "bytes", (out / "beam_cases.json").stat().st_size)

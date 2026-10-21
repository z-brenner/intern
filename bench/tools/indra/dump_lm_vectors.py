"""LM / beam-search parity vectors from the installed indra_ocr.
usage: python -I dump_lm_vectors.py MODEL_DIR OUT_JSON IMAGE..."""
import json, sys
from pathlib import Path
import numpy as np
from indra_ocr import pipeline as P
from indra_ocr import lm as LM
md, outp = Path(sys.argv[1]), Path(sys.argv[2])
ocr = P.OrtOCR(md, threads=1)
cases = []
_bs = LM.beam_search
def spy(blank, topv, topi, lm, alpha, beta, beam, cand_floor=1e-3, **kw):
    ids, steps = _bs(blank, topv, topi, lm, alpha, beta, beam, cand_floor=cand_floor, **kw)
    cases.append({"lm_order": lm.order, "lm_bits": lm.bits, "alpha": alpha, "beta": beta, "beam": beam, "cand_floor": cand_floor,
                  "blank": blank.astype(float).tolist(), "topv": topv.astype(float).tolist(), "topi": topi.astype(int).tolist(),
                  "ids": [int(i) for i in ids], "steps": [int(s) for s in steps]})
    return ids, steps
LM.beam_search = spy
for img in sys.argv[3:]:
    ocr(P.read_images(Path(img))[0])
rng = np.random.default_rng(1)
logp = {}
for name in ("lm.npz", "lm_cjk.npz", "lm_world.npz"):
    lm = LM.NgramLM(md / name)
    # real n-grams from the tables (k{order-1} keys) and random token sequences
    full = lm.k[lm.order - 1][rng.integers(0, len(lm.k[lm.order - 1]), 40)].astype(np.uint64)
    keys = [int(k) for k in full]
    hist_bits = lm.bits * (lm.order - 1)
    for _ in range(40):
        toks = rng.integers(0, 60, lm.order)  # packed values: 0 = BOS, c+1 = class c
        k = 0
        for t in toks: k = (k << lm.bits) | int(t)
        keys.append(k)
    vals = lm.logp_keys(keys)
    logp[name] = {"order": lm.order, "bits": lm.bits, "vocab_size": lm.vocab_size, "max_logp": lm.max_logp,
                  "keys": [str(k) for k in keys], "logp": vals}
outp.write_text(json.dumps({"beam_cases": cases, "logp_keys": logp}))
print("beam cases", len(cases), "max T", max((len(c["blank"]) for c in cases), default=0))

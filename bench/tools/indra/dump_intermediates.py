"""Dump intermediate tensors of the installed indra_ocr pipeline for one image:
every network's inputs and outputs on its first call (<model>.in.<name>.npy,
<model>.out.<name>.npy), and for every call of the base recognizer its true
widths and outputs (calls/rec.<k>.in.widths.npy, calls/rec.<k>.out.<name>.npy;
the padded crops of later calls are left out for size).
usage: python -I dump_intermediates.py MODEL_DIR IMAGE OUT_DIR"""
import json, sys
from pathlib import Path
import numpy as np
import indra_ocr
from indra_ocr import pipeline as P
from indra_ocr import lm as LM

model_dir, image, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
out.mkdir(parents=True, exist_ok=True)
ocr = P.OrtOCR(model_dir, threads=1)
names = {id(ocr.det): "det", id(ocr.rec): "rec", id(ocr.order): "order", id(ocr.orient): "orient"}
if ocr.script is not None:
    names[id(ocr.script)] = "script"
for g, (s, _) in ocr.groups.items():
    names[id(s)] = f"rec_{g}"
log, counts = [], {}

class Spy:
    def __init__(self, sess, name):
        self._s, self._n = sess, name
    def __getattr__(self, k):
        return getattr(self._s, k)
    def run(self, outs, feeds):
        res = self._s.run(outs, feeds)
        onames = outs or [o.name for o in self._s.get_outputs()]
        k = counts.get(self._n, 0); counts[self._n] = k + 1
        log.append({"model": self._n, "call": k,
                    "inputs": {n: [list(v.shape), str(v.dtype)] for n, v in feeds.items()},
                    "outputs": {n: [list(r.shape), str(r.dtype)] for n, r in zip(onames, res)}})
        if self._n == "rec":  # every batch the base recognizer reads
            calls = out / "calls"
            calls.mkdir(exist_ok=True)
            np.save(calls / f"rec.{k}.in.widths.npy", feeds["widths"])
            for n, r in zip(onames, res):
                np.save(calls / f"rec.{k}.out.{n}.npy", r)
        if k == 0:  # save the first call of each model
            for n, v in feeds.items():
                np.save(out / f"{self._n}.in.{n}.npy", v)
            for n, r in zip(onames, res):
                np.save(out / f"{self._n}.out.{n}.npy", r)
        return res

for attr in ("det", "rec", "order", "orient", "script"):
    s = getattr(ocr, attr)
    if s is not None:
        setattr(ocr, attr, Spy(s, names[id(s)]))
ocr.groups = {g: (Spy(s, f"rec_{g}"), v) for g, (s, v) in ocr.groups.items()}
# re-key the LM table to the spies
ocr.lms = {id(ocr.rec): ocr.lms[id(ocr.rec._s)]} | {
    id(s): ocr.lms[id(s._s)] for g, (s, _) in ocr.groups.items() if id(s._s) in ocr.lms}

beam_calls = []
_bs = LM.beam_search
def spy_bs(*a, **k):
    r = _bs(*a, **k); beam_calls.append({"T": int(len(a[0])), "n_tokens": len(r[0]), "alpha": a[4], "beta": a[5], "beam": a[6]}); return r
LM.beam_search = spy_bs

gray = P.read_images(image)[0]
np.save(out / "page.gray.npy", gray)
small = P.preview(gray)
skew = P.estimate_skew(small)
probs = ocr.orientation_probs(P.rotate_page(small, skew) if abs(skew) >= P.DESKEW_MIN else small)
page = ocr(gray)
summary = {
    "image": str(image.name), "gray_shape": list(gray.shape), "preview_shape": list(small.shape),
    "estimate_skew_deg": skew, "orient_probs": [float(x) for x in probs] if probs is not None else None,
    "page_rotation": page.rotation, "page_skew": page.skew, "n_lines": len(page.lines),
    "lm_beam_calls": beam_calls, "calls": log,
    "lm_vocab": {"order": ocr.lm.order, "vocab_size": ocr.lm.vocab_size, "bits": ocr.lm.bits} if ocr.lm else None,
}
(out / "summary.json").write_text(json.dumps(summary, indent=1))
print(json.dumps({k: v for k, v in summary.items() if k != "calls"}, indent=1)[:3000])
for c in log[:40]:
    print(c["model"], c["call"], c["inputs"], c["outputs"])

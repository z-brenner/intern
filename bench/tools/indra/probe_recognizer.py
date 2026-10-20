"""Recognizer probes behind spec §1 and §6: tall-vs-band crop widths, padding
and batch invariance of rec.onnx, time steps per width, script.onnx and det.onnx
output ranges.
usage: python -I probe_recognizer.py MODEL_DIR IMAGE"""
import sys, numpy as np
from pathlib import Path
from indra_ocr import pipeline as P
from indra_ocr.line_mask import crop_lines, line_darkness
from indra_ocr.rec_preproc import pad_batch
md, img = Path(sys.argv[1]), Path(sys.argv[2])
ocr = P.OrtOCR(md, threads=1)
gray = P.read_images(img)[0]
boxes = ocr.detect(gray)
print("boxes", boxes.shape, boxes[:3])
dark = line_darkness(gray, boxes[:, :4])
c1 = crop_lines(gray, boxes[:, :4], 32, dark=dark)
c3 = crop_lines(gray, boxes[:, :4], 32, window=3, dark=dark)
w1 = np.array([c.shape[1] for c in c1]); w3 = np.array([c.shape[1] for c in c3])
print("width diff tall-band: min/max", (w3 - w1).min(), (w3 - w1).max())
i = int(np.argmax(w3 - w1)); print("worst", i, boxes[i], w1[i], w3[i])
# padding invariance of rec
j = int(np.argmin(w3))
a1, wd1 = pad_batch([c3[j]])
a2 = np.zeros((1, 1, 96, a1.shape[-1] + 64), np.float32); a2[..., : a1.shape[-1]] = a1
r1 = ocr.rec.run(None, {"x": a1, "widths": wd1}); r2 = ocr.rec.run(None, {"x": a2, "widths": wd1})
n = int(r1[2][0]); print("width", wd1, "T", r1[0].shape[1], "len", n, "padded T", r2[0].shape[1], "len", int(r2[2][0]))
print("ids equal", np.array_equal(r1[0][0, :n], r2[0][0, :n]), "conf maxdiff", np.abs(r1[1][0, :n] - r2[1][0, :n]).max())
print("conf range", r1[1][0, :n].min(), r1[1][0, :n].max(), "blank(log) range", r1[3][0, :n].min(), r1[3][0, :n].max())
print("topv[0]", r1[4][0, 0], "topi[0]", r1[5][0, 0], "exp(topv0)", np.exp(r1[4][0, 0, 0]), "conf0", r1[1][0, 0], "ids0", r1[0][0, 0])
print("lengths vs widths: ", [(int(w), int(np.ceil(w / 4)), int(w) // 4) for w in wd1], n)
for w in (8, 9, 10, 11, 12, 13, 17, 31, 33):
    x = np.zeros((1, 1, 96, 64), np.float32); r = ocr.rec.run(["lengths"], {"x": x, "widths": np.array([w])})
    print("  width", w, "-> length", int(r[0][0]))
# batch invariance
from indra_ocr.rec_preproc import width_batches
idx = list(range(min(6, len(c3))))
arr, wd = pad_batch([c3[k] for k in idx])
rb = ocr.rec.run(None, {"x": arr, "widths": wd})
for k in idx:
    a, w = pad_batch([c3[k]]); r = ocr.rec.run(None, {"x": a, "widths": w}); n = int(r[2][0])
    print("  line", k, "batch-vs-single ids equal", np.array_equal(rb[0][k, :n], r[0][0, :n]), "conf diff", float(np.abs(rb[1][k, :n] - r[1][0, :n]).max()), "len", n, int(rb[2][k]))
# script probs
a, w = pad_batch(c1[:4]); print("script p", ocr.script.run(None, {"x": a, "widths": w})[0])
# det raw output stats
x, s = P.page_to_input(gray, 1024)
o = ocr.det.run(None, {"x": x[None, None]})[0][0]
print("det out ch0 (logit) range", o[0].min(), o[0].max(), "ch1", o[1].min(), o[1].max(), "ch2", o[2].min(), o[2].max(), "scale", s)

"""The Latin-only goldens (OUT_DIR/<image>.jsonl): the reference pipeline with
script.onnx routing off, so every line goes to rec.onnx - what Intern ports.
Prints where it differs from the full multilingual pipeline.
usage: python -I dump_latin_only.py MODEL_DIR OUT_DIR IMAGE..."""
import sys, json
from dataclasses import asdict
from pathlib import Path
import indra_ocr
from indra_ocr import pipeline as P
md, out = Path(sys.argv[1]), Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
full = P.OrtOCR(md, threads=1)
lat = P.OrtOCR(md, threads=1); lat.script = None
for img in sys.argv[3:]:
    g = P.read_images(Path(img))[0]
    a, b = full(g), lat(g)
    ta, tb = [l.text for l in a.lines], [l.text for l in b.lines]
    same = sum(x == y for x, y in zip(ta, tb))
    print(f"{Path(img).name}: lines {len(ta)} vs {len(tb)}, identical {same}, markdown equal {a.markdown == b.markdown}")
    for x, y in zip(ta, tb):
        if x != y: print("   full :", x[:90]); print("   latin:", y[:90])
    rec = {"file": Path(img).name, "lines": [asdict(l) for l in b.lines], "markdown": b.markdown, "blocks": b.blocks,
           "rotation": b.rotation, "skew": b.skew, "vertical": b.vertical}
    (out / (Path(img).name + ".jsonl")).write_text(json.dumps(rec, ensure_ascii=False) + "\n")

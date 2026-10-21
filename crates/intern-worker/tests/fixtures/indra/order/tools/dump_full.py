"""Runs the reference end to end on input files and records, for every page
read by OCR, OrtOCR.assemble's inputs, the order network call(s) and what
assemble returned (boxes on the levelled page), checking the finished pages
against the JSON-lines goldens.
usage: python -I dump_full.py MODEL_DIR OUT_DIR MODE GOLDEN_DIR FILE..."""
import base64, copy, json, sys
from pathlib import Path
import numpy as np
from indra_ocr import pipeline as P
from indra_ocr.document import Converter

md, out, mode, golden = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4])
out.mkdir(parents=True, exist_ok=True)
ocr = P.OrtOCR(md, threads=1)
real_order = ocr.order
real_assemble = ocr.assemble


def b64(a):
    return base64.b64encode(np.ascontiguousarray(a, "<f4").tobytes()).decode()


class Rec:
    def __init__(self):
        self.calls = []

    def run(self, names, feeds):
        outs = real_order.run(names, feeds)
        self.calls.append({"n": int(feeds["feats"].shape[1]), "feats": b64(feeds["feats"][0]),
                           "a": b64(outs[0][0]), "cls": b64(outs[1][0]), "same": b64(outs[2][0])})
        return outs


captured = []


def assemble(wh, boxes, rec, t, **kw):
    r = Rec()
    ocr.order = r
    try:
        page = real_assemble(wh, boxes, rec, t, **kw)
    finally:
        ocr.order = real_order
    if any(len(tk) != 2 for _, _, toks in rec for tk in toks) or kw.get("vertical"):
        captured.append(None)  # a text-layer page or a vertical page: not replayed
        return page
    captured.append({
        "width": int(wh[0]), "height": int(wh[1]),
        "det_boxes": np.asarray(boxes, np.float32).astype(float).tolist(),
        "rec": [[t_, float(c), [[a if isinstance(a, str) else int(a), float(x)] for a, x in k]] for t_, c, k in rec],
        "order_calls": r.calls,
        "lines": [{"box": list(l.box), "text": l.text, "conf": l.conf} for l in page.lines],
        "blocks": copy.deepcopy(page.blocks), "markdown": page.markdown,
    })
    return page


ocr.assemble = assemble
conv = Converter(ocr, mode=mode)
for f in sys.argv[5:]:
    f = Path(f)
    gold = [json.loads(l) for l in (golden / (f.name + ".jsonl")).read_text().splitlines()]
    for (i, page), g in zip(conv.convert(f), gold):
        cap = captured.pop() if captured else None
        captured.clear()
        assert [l.text for l in page.lines] == [l["text"] for l in g["lines"]], (f, i)
        assert page.markdown == g["markdown"], (f, i)
        if cap is None or page.source != "ocr":
            print(f.name, i, "skipped", page.source)
            continue
        cap["file"] = f"{f.name}#{i}"
        name = f.stem + (f"-p{i}" if f.suffix == ".pdf" else "") + ("" if f.suffix == ".png" else f.suffix.replace(".", "-"))
        (out / f"{name}.json").write_text(json.dumps(cap, ensure_ascii=False, separators=(",", ":")))
        print(f.name, i, "->", name, len(cap["lines"]), "lines", len(cap["order_calls"]), "calls")

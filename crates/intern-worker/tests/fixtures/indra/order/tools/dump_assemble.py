"""Replays OrtOCR.assemble on recorded det boxes + recognizer output for every
stages page, recording the order network call(s) and intermediates.
usage: python -I dump_assemble.py MODEL_DIR STAGES_DIR OUT_DIR"""
import base64, json, sys
from pathlib import Path
import numpy as np
from indra_ocr import pipeline as P

md, stages, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
out.mkdir(parents=True, exist_ok=True)
ocr = P.OrtOCR(md, threads=1)
real_order = ocr.order


def b64(a, dt):
    return base64.b64encode(np.ascontiguousarray(a, dt).tobytes()).decode()


class Rec:
    def __init__(self):
        self.calls = []

    def run(self, names, feeds):
        outs = real_order.run(names, feeds)
        f = feeds["feats"][0]
        self.calls.append({"n": int(f.shape[0]), "feats": b64(f, "<f4"), "a": b64(outs[0][0], "<f4"),
                           "cls": b64(outs[1][0], "<f4"), "same": b64(outs[2][0], "<f4")})
        return outs


real_tidy = P.tidy_lines
for f in sorted(stages.glob("*.stages.json")):
    d = json.loads(f.read_text())
    H, W = d["upright_shape"]
    boxes = np.array(d["det_boxes"], np.float32).reshape(-1, 5)
    rec = [(r["text"], r["conf"], [(t[0], t[1]) for t in r["tokens"]]) for r in d["rec"]]
    r = Rec()
    ocr.order = r
    tidy = {}

    def tl(b, rc, *a):
        ob, orc = real_tidy(b, rc, *a)
        tidy["boxes"] = ob.astype(float).tolist()
        tidy["texts"] = [x[0] for x in orc]
        return ob, orc

    P.tidy_lines = tl
    page = ocr.assemble((W, H), boxes, rec, {})
    P.tidy_lines = real_tidy
    ocr.order = real_order
    res = {
        "file": d["file"], "width": W, "height": H, "skew": d["skew_applied"],
        "det_boxes": d["det_boxes"], "rec": [[t, c, [[a, x] for a, x in k]] for t, c, k in rec],
        "order_calls": r.calls,
        "tidy": tidy,
        "lines": [{"box": l.box, "text": l.text, "conf": l.conf} for l in page.lines],
        "blocks": page.blocks, "markdown": page.markdown,
    }
    # sanity vs stages (text/labels/markdown; boxes only when not deskewed)
    assert [l["text"] for l in res["lines"]] == [l["text"] for l in d["final_lines"]], f
    assert [b["label"] for b in res["blocks"]] == [b["label"] for b in d["blocks"]], f
    assert res["markdown"] == d["markdown"], f
    if not d["skew_applied"]:
        assert [l["box"] for l in res["lines"]] == [l["box"] for l in d["final_lines"]], f
    (out / (f.name.replace(".stages.json", ".json"))).write_text(json.dumps(res, ensure_ascii=False))
    print(f.name, len(boxes), "->", len(res["lines"]), "calls", len(r.calls))

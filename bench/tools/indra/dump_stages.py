"""Per-stage golden values (after turn/deskew): det boxes, recognizer output, groups, order.
usage: python -I dump_stages.py MODEL_DIR OUT_DIR IMAGE..."""
import json, sys
from pathlib import Path
import numpy as np
from indra_ocr import pipeline as P
from indra_ocr.order_post import line_features
md, out = Path(sys.argv[1]), Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
ocr = P.OrtOCR(md, threads=1)
for img in sys.argv[3:]:
    g = P.read_images(Path(img))[0]
    small = P.preview(g)
    a = P.estimate_skew(small)
    k = ocr.orientation(P.rotate_page(small, a) if abs(a) >= P.DESKEW_MIN else small)
    up, k, skew = ocr._turn(g, k, a, small)
    np.save(out / (Path(img).stem + ".upright.npy"), up)
    boxes = ocr.detect(up)
    groups, sure = ocr.box_groups(up, boxes)
    rec = ocr.recognize(up, boxes)
    page = ocr._read(g, k, a, small)
    d = {"file": Path(img).name, "estimate_skew": a, "k": k, "skew_applied": skew, "upright_shape": list(up.shape),
         "det_boxes": boxes.astype(float).tolist(),
         "groups": groups, "sure": sure,
         "rec": [{"text": t, "conf": c, "tokens": [[x if isinstance(x, str) else int(x), float(px)] for x, px in toks]} for t, c, toks in rec],
         "final_lines": [{"box": l.box, "text": l.text, "conf": l.conf} for l in page.lines],
         "blocks": page.blocks, "markdown": page.markdown}
    (out / (Path(img).stem + ".stages.json")).write_text(json.dumps(d, ensure_ascii=False))
    print(img, len(boxes), "k", k, "skew", skew)

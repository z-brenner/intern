import json, sys
import numpy as np
from indra_ocr import order_post as O, layout_post as L
rng = np.random.default_rng(int(sys.argv[2]))
cases = []
def boxes_rand(n, snap):
    x0 = rng.uniform(0, 800, n); y0 = rng.uniform(0, 1000, n)
    w = rng.uniform(5, 300, n); h = rng.uniform(6, 20, n)
    b = np.stack([x0, y0, x0 + w, y0 + h], 1)
    if snap: b = np.round(b / snap) * snap
    return b.astype(np.float32)
def grid_boxes():
    R, C = rng.integers(2, 7), rng.integers(1, 6)
    out = []
    for r in range(R):
        for c in range(C):
            if rng.random() < 0.15: continue
            x0 = 20 + c * 120 + rng.normal(0, 4); y0 = 30 + r * 25 + rng.normal(0, 1.5)
            w = rng.uniform(30, 110) if rng.random() > 0.1 else rng.uniform(150, 330)
            out.append([x0, y0, x0 + w, y0 + rng.uniform(9, 12)])
            if rng.random() < 0.1:  # wrapped continuation
                out.append([x0, y0 + 12.5, x0 + w, y0 + 23])
    if rng.random() < 0.3:
        out.insert(0, [20, 5, 20 + 120 * C * 0.9, 16])
    return np.array(out, np.float32).reshape(-1, 4)
for k in range(400):
    n = int(rng.integers(1, 45))
    base = rng.normal(0, 3, (n, n)).astype(np.float32)
    grp = rng.integers(0, max(1, n // 4), n)
    same = (base + np.where(grp[:, None] == grp[None, :], 4.0, -4.0)).astype(np.float32)
    a = rng.normal(0, 4, (n, n)).astype(np.float32)
    cls = rng.normal(0, 2, (n, 10)).astype(np.float32)
    snap = [None, None, 1.0, 5.0][k % 4]
    b = boxes_rand(n, snap)
    texts = ["".join(rng.choice(list("ab1. -•(xC)漢:"), int(rng.integers(0, 8)))) for _ in range(n)]
    g = grid_boxes()
    gt = [f"c{i}&<" for i in range(len(g))]
    cases.append({
        "n": n, "same": same.ravel().tolist(), "a": a.ravel().tolist(), "cls": cls.ravel().tolist(),
        "boxes": b.tolist(), "texts": texts,
        "cluster": O._cluster(same), "decode_order": O.decode_order(a),
        "blocks_order": O.decode_blocks_order(a, same, b, cls),
        "xycut": O.order_xycut(b), "top_down": O.order_top_down(b),
        "feats": O.line_features(b, (900, 1200), texts).ravel().tolist(),
        "grid": g.tolist(), "grid_texts": gt, "table": L.table_html(g, gt),
    })
json.dump(cases, open(sys.argv[1], "w"))

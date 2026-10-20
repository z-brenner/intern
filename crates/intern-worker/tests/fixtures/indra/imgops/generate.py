"""Golden arrays for Intern's port of Indra's image operations (src/indra/imgops.rs,
page.rs, det.rs), produced with the reference package and OpenCV 5.0.

usage: python -I generate.py REFERENCE_DIR INPUTS_DIR FIXTURE_DIR

Run with indra-ocr 1.1.2, opencv-python-headless 5.0.0.93 and numpy 2.5.4 on
an x86-64 machine with AVX2 and FMA (OpenCV's warp kernel rounds by the
instruction set it dispatches to). Copy this file outside the reference
package before running it; it imports the installed package.

REFERENCE_DIR  scratchpad/indra-reference (heavy goldens go to REFERENCE_DIR/imgops/)
INPUTS_DIR     scratchpad/indra-inputs (the reference input images)
FIXTURE_DIR    crates/intern-worker/tests/fixtures/indra/imgops (small committed goldens)
"""
import json
import sys
import types
from pathlib import Path

import cv2
import numpy as np
from indra_ocr import pipeline as P
from indra_ocr.det_post import decode, page_to_input
from indra_ocr.rec_preproc import _percentiles

ref, inputs, fix = (Path(a) for a in sys.argv[1:4])
heavy = ref / "imgops"
heavy.mkdir(parents=True, exist_ok=True)
fix.mkdir(parents=True, exist_ok=True)
rng = np.random.default_rng(20261020)


def save(d, name, a):
    np.save(d / name, np.ascontiguousarray(a))
    return name


# ---------------------------------------------------------------- committed
cases = {"resize": [], "warp": [], "rotation": [], "components": [], "percentile": [], "gray": [], "decode": []}

page = np.load(ref / "intermediate/multi_column/page.gray.npy")
srcs = {
    "doc": page[300:372, 80:180].copy(),  # 100 x 72, text
    "rand": rng.integers(0, 256, (29, 41), dtype=np.uint8),
    "col": rng.integers(0, 256, (20, 1), dtype=np.uint8),
    "row": rng.integers(0, 256, (1, 23), dtype=np.uint8),
}
for k, v in srcs.items():
    save(fix, f"src_{k}.npy", v)

resize_cases = [
    ("doc", "linear", [(173, 121), (200, 144), (101, 73), (100, 300), (37, 26), (50, 36), (1, 1), (1, 50), (300, 1), (64, 72), (100, 72)]),
    ("doc", "area", [(50, 36), (33, 24), (25, 18), (20, 24), (61, 43), (37, 26), (173, 121), (101, 73), (100, 36), (150, 36), (1, 1), (200, 144), (99, 71), (300, 216)]),
    ("rand", "linear", [(82, 58), (97, 64), (20, 14), (41, 300)]),
    ("rand", "area", [(13, 9), (82, 58), (40, 28), (97, 64), (7, 5), (41, 87)]),
    ("col", "linear", [(5, 40), (1, 7), (3, 3)]),
    ("col", "area", [(5, 40), (1, 7), (3, 3), (1, 10)]),
    ("row", "linear", [(40, 5), (7, 1)]),
    ("row", "area", [(40, 5), (7, 1), (11, 1)]),
]
flags = {"linear": cv2.INTER_LINEAR, "area": cv2.INTER_AREA}
n = 0
for src, interp, sizes in resize_cases:
    for w, h in sizes:
        out = cv2.resize(srcs[src], (w, h), interpolation=flags[interp])
        cases["resize"].append({"src": src, "interp": interp, "width": w, "height": h, "out": save(fix, f"resize_{n:02}.npy", out)})
        n += 1

n = 0
for src in ("doc", "rand"):
    g = srcs[src]
    h, w = g.shape
    for a in (-30.0, 7.3, 0.4, -44.9, 90.0, -29.999999999999993):
        M = cv2.getRotationMatrix2D((w / 2, h / 2), a, 1.0)
        for border in ("replicate", "constant"):
            if border == "replicate":
                out = cv2.warpAffine(g, M, (w, h), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_REPLICATE)
                s = src
            else:
                out = cv2.warpAffine(255 - g, M, (w, h))
                s = src + "_ink"
            cases["warp"].append({"src": s, "m": M.ravel().tolist(), "width": w, "height": h, "border": border, "out": save(fix, f"warp_{n:02}.npy", out)})
            n += 1
    for M, (w2, h2), border, value in (
        (np.array([[0.7, 0.21, 3.5], [-0.13, 1.3, -6.25]]), (80, 60), cv2.BORDER_CONSTANT, 0),
        (np.array([[1.4, -0.3, -2.0], [0.25, 0.9, 1.5]]), (57, 33), cv2.BORDER_CONSTANT, 200),
        (np.array([[0.5, 0.0, 0.0], [0.0, 0.5, 0.0]]), (17, 19), cv2.BORDER_REPLICATE, 0),
    ):
        out = cv2.warpAffine(g, M, (w2, h2), flags=cv2.INTER_LINEAR, borderMode=border, borderValue=value)
        cases["warp"].append({"src": src, "m": M.ravel().tolist(), "width": w2, "height": h2,
                              "border": "replicate" if border == cv2.BORDER_REPLICATE else "constant", "value": value,
                              "out": save(fix, f"warp_{n:02}.npy", out)})
        n += 1
save(fix, "src_doc_ink.npy", 255 - srcs["doc"])
save(fix, "src_rand_ink.npy", 255 - srcs["rand"])

for cx, cy, a in ((450.0, 600.0, -29.999999999999993), (50.0, 36.0, 7.3), (20.5, 14.5, -1.1102230246251565e-16), (0.0, 0.0, 90.0), (280.0, 150.0, 1.9000000000000001), (400.5, 3.5, -45.0)):
    m = cv2.getRotationMatrix2D((cx, cy), a, 1.0)
    cases["rotation"].append({"cx": cx, "cy": cy, "degrees": a, "m": m.ravel().tolist(), "inverse": cv2.invertAffineTransform(m).ravel().tolist()})

det_out = np.load(ref / "intermediate/multi_column/det.out.out.npy")[0]
prob = 1 / (1 + np.exp(-det_out[0]))
masks = [
    (rng.random((37, 53)) > 0.45).astype(np.uint8),
    (rng.random((37, 53)) > 0.55).astype(np.uint8),
    (rng.random((29, 17)) > 0.3).astype(np.uint8),
    (prob[100:164, 40:136] > 0.5).astype(np.uint8),
    np.zeros((5, 7), np.uint8),
    np.ones((4, 6), np.uint8),
]
for i, m in enumerate(masks):
    n_, lab, stats, _ = cv2.connectedComponentsWithStats(m, connectivity=4)
    cases["components"].append({"mask": save(fix, f"cc_mask_{i}.npy", m), "labels": save(fix, f"cc_labels_{i}.npy", lab.astype(np.int32)),
                                "stats": stats[1:].tolist()})

for size in (1, 2, 3, 7, 50, 101, 1000, 4097):
    v = rng.integers(0, 256, size, dtype=np.uint8)
    if size == 1000:
        v = np.where(rng.random(size) > 0.1, 255, v).astype(np.uint8)
    qs = [0.0, 2.0, 50.0, 98.0, 100.0, 37.5, 99.9]
    cases["percentile"].append({"values": v.tolist(), "q": qs, "numpy": [float(np.percentile(v, q)) for q in qs],
                                "reference": [float(x) for x in _percentiles(v.reshape(1, -1))]})

# IMREAD_GRAYSCALE of a colour PNG is libpng's rgb_to_gray; random colours
# separate it from the 14-bit formula on about 0.3% of pixels.
rgb = rng.integers(0, 256, (90, 120, 3), dtype=np.uint8)
rgb[:4] = rng.integers(0, 256, 3, dtype=np.uint8)  # flat colour rows
rgb[4:8] = rng.integers(0, 256, (4, 120, 1), dtype=np.uint8)  # gray pixels
ok, png = cv2.imencode(".png", cv2.cvtColor(rgb, cv2.COLOR_RGB2BGR))
gray = cv2.imdecode(png, cv2.IMREAD_GRAYSCALE)
cases["gray"].append({"rgb": save(fix, "gray_rgb.npy", rgb), "gray": save(fix, "gray_out.npy", gray)})

window = np.ascontiguousarray(det_out[:, 96:160, 0:128])
boxes = decode(window)
cases["decode"].append({"out": save(fix, "decode_out.npy", window.astype(np.float32)), "boxes": boxes.astype(float).tolist(),
                        "boxes_f32_bits": boxes.view(np.uint32).tolist()})

(fix / "cases.json").write_text(json.dumps(cases))


# ---------------------------------------------------------------- heavy
def skew_trace(g):
    """estimate_skew with every score it computes."""
    fine = P._ink_thumb(g, 800)
    coarse = P._ink_thumb(g, 400)
    out = {"fine_shape": list(fine.shape), "coarse_shape": list(coarse.shape), "blank": bool(fine.mean() < 0.003 * 255)}
    if out["blank"]:
        out["angle"] = 0.0
        return out, fine, coarse
    grid = np.arange(-P.DESKEW_MAX, P.DESKEW_MAX + 1e-6, 2.0)
    scores = [P._row_sharpness(coarse, a) for a in grid]
    starts = {0.0, *(float(grid[i]) for i in np.argsort(scores)[-2:])}
    near = [a for a0 in starts for a in np.arange(a0 - 2.0, a0 + 2.0 + 1e-6, 0.5)]
    seen = dict.fromkeys(near)
    seen.update(zip(seen, [P._row_sharpness(fine, a) for a in seen]))
    a1 = max(near, key=seen.__getitem__)
    last = np.arange(a1 - 0.5, a1 + 0.5 + 1e-6, 0.1)
    vals = [P._row_sharpness(fine, a) for a in last]
    out.update(coarse_grid=grid.tolist(), coarse_scores=scores, near=[float(a) for a in seen], near_scores=[seen[a] for a in seen],
               a1=float(a1), last=last.tolist(), last_scores=vals, angle=P.estimate_skew(g))
    assert out["angle"] == float(max(zip(last, vals), key=lambda av: av[1])[0])
    return out, fine, coarse


def gray_of(name):
    return P.read_images(inputs / name)[0]


pages = {
    "document-image": gray_of("document-image.png"),
    "multi_column": gray_of("multi_column.png"),
    "rotated-low-resolution-scan": gray_of("rotated-low-resolution-scan.png"),
    "table_pmc": gray_of("table_pmc.png"),
    "tilted_memo": gray_of("tilted_memo.png"),
    "scan": gray_of("scan.png"),
    "rotated_letter": gray_of("rotated_letter.png"),
}
mc = pages["multi_column"]
pages["multi_column_tilt7"] = P.rotate_page(mc, 7.3)
pages["multi_column_rot1"] = np.ascontiguousarray(np.rot90(mc, 1))
pages["multi_column_rot3_tilt"] = np.ascontiguousarray(np.rot90(P.rotate_page(mc, -4.2), 3))
pages["table_pmc_rot2"] = np.ascontiguousarray(np.rot90(pages["table_pmc"], 2))
pages["scan_tilt"] = P.rotate_page(pages["scan"], 2.6)
pages["big_tiled"] = np.ascontiguousarray(np.tile(pages["table_pmc"], (2, 2))[:2047, :1777])
pages["blank"] = np.full((640, 480), 255, np.uint8)
# The class each synthetic page's content is turned by (the turn _turn undoes).
turns = {"multi_column_rot1": 3, "multi_column_rot3_tilt": 1, "table_pmc_rot2": 2, "rotated-low-resolution-scan": 1}

summary = {}
dummy = types.SimpleNamespace()
for name, g in pages.items():
    d = heavy / name
    d.mkdir(exist_ok=True)
    save(d, "gray.npy", g)
    small = P.preview(g)
    save(d, "preview.npy", small)
    trace, fine, coarse = skew_trace(small)
    save(d, "fine.npy", fine)
    save(d, "coarse.npy", coarse)
    a = trace["angle"]
    levelled = P.rotate_page(small, a) if abs(a) >= P.DESKEW_MIN else small
    save(d, "orient_small.npy", cv2.resize(levelled, (max(1, round(levelled.shape[1] * 768 / max(levelled.shape))), max(1, round(levelled.shape[0] * 768 / max(levelled.shape)))), interpolation=cv2.INTER_AREA))
    if levelled is not small:
        save(d, "levelled_preview.npy", levelled)
    entry = {"shape": list(g.shape), "preview_shape": list(small.shape), "skew": trace, "turns": {}}
    for k in sorted({0, turns.get(name, 0), 4 if name == "multi_column_rot1" else 0}):
        up, k2, s = P.OrtOCR._turn(dummy, g, k, a, small)
        save(d, f"turned_{k}.npy", up)
        x, scale = page_to_input(up, 1024)
        nh, nw = max(1, round(up.shape[0] * scale)), max(1, round(up.shape[1] * scale))
        det_small = up if (nh, nw) == up.shape else cv2.resize(up, (nw, nh), interpolation=cv2.INTER_AREA if scale < 1 else cv2.INTER_LINEAR)
        assert np.array_equal(x[:nh, :nw], 1.0 - det_small.astype(np.float32) / 255.0)
        save(d, f"det_small_{k}.npy", det_small)
        entry["turns"][str(k)] = {"k": k2, "skew": s, "shape": list(up.shape), "det_shape": list(x.shape), "det_scale": scale}
    summary[name] = entry
    print(name, g.shape, "skew", a, {k: v["skew"] for k, v in entry["turns"].items()}, flush=True)

boxes_in = [[10.0, 20.0, 200.0, 41.5], [0.25, 0.75, 899.75, 1199.25], [-3.0, 5.0, 950.0, 30.0], [430.05, 600.15, 470.35, 610.45]]
unrot = []
for wh in ((900, 963), (560, 300)):
    for sk in (-29.999999999999993, 0.3, 7.3, -1.25):
        for b in boxes_in:
            unrot.append({"box": b, "skew": sk, "wh": list(wh), "out": P.unrotate_box(b, sk, wh)})
summary["unrotate_box"] = unrot
(heavy / "summary.json").write_text(json.dumps(summary))
print("done")

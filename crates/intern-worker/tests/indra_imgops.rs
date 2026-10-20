//! Indra's image operations, page preparation and detector decoding against
//! the reference package (`indra-ocr` 1.1.2 on OpenCV 5.0.0, numpy 2.5).
//!
//! Two sets of goldens, both made by `tests/fixtures/indra/imgops/
//! generate.py` with the reference package's environment:
//!
//! - `tests/fixtures/indra/imgops/` (committed, small): OpenCV's `resize`,
//!   `warpAffine`, `getRotationMatrix2D`, connected components and the
//!   grayscale decode on small and random images, numpy's percentiles, and
//!   `det_post.decode` on a window of a real detector output. These always
//!   run.
//! - The reference dump (heavy, not committed): whole pages through
//!   `preview`, the skew search (with every score), the orientation input,
//!   `_turn` for every orientation class, the detector input, and the
//!   detector's real output decoded to page boxes. These run when
//!   `INDRA_REFERENCE_DIR` names the dump (`INDRA_REFERENCE_DIR=scratchpad/
//!   indra-reference`, which holds `stages/`, `intermediate/` and the
//!   `imgops/` pages `generate.py` adds), and print a note and pass when it
//!   is unset.
//!
//! Exact means bit-for-bit. The only tolerance is on the detector's box
//! score, a mean of `1 / (1 + exp(-x))`, whose `exp` is numpy's vectorized
//! one in the reference: it may differ in the last bits (relative 1e-6).

use std::path::{Path, PathBuf};

use intern_worker::indra::det;
use intern_worker::indra::imgops::{self, Border, Interpolation};
use intern_worker::indra::npy::{self, NpyArray, NpyData};
use intern_worker::indra::page;
use intern_worker::indra::types::{DetBox, Gray};
use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/indra/imgops")
}

fn reference() -> Option<PathBuf> {
    match std::env::var_os("INDRA_REFERENCE_DIR") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!(
                "INDRA_REFERENCE_DIR is not set: skipping the whole-page goldens \
                 (set it to scratchpad/indra-reference)"
            );
            None
        }
    }
}

fn load(path: &Path) -> NpyArray {
    npy::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn gray(path: &Path) -> Gray {
    let a = load(path);
    assert_eq!(a.shape.len(), 2, "{}", path.display());
    let data = a.as_u8().expect("uint8 image").to_vec();
    Gray::new(a.shape[1], a.shape[0], data)
}

/// A JSON file with every number kept as its text (a string value), parsed
/// later by Rust's correctly rounded `str::parse`: serde_json's own float
/// parsing (without its `float_roundtrip` feature) can land one ulp off,
/// e.g. on -144.61524227066315.
fn json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut quoted = String::with_capacity(text.len() * 2);
    let mut chars = text.chars().peekable();
    let (mut in_string, mut escaped) = (false, false);
    while let Some(c) = chars.next() {
        if in_string {
            quoted.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
        } else if c == '"' {
            in_string = true;
            quoted.push(c);
        } else if c == '-' || c.is_ascii_digit() {
            quoted.push('"');
            quoted.push(c);
            while let Some(&d) = chars.peek() {
                if !(d.is_ascii_digit() || matches!(d, '.' | 'e' | 'E' | '+' | '-')) {
                    break;
                }
                quoted.push(d);
                chars.next();
            }
            quoted.push('"');
        } else {
            quoted.push(c);
        }
    }
    serde_json::from_str(&quoted).expect("json")
}

fn cases() -> Value {
    json(&fixtures().join("cases.json"))
}

fn num(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("not a number: {v}"))
}

fn f64s(v: &Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(num).collect()
}

fn usize_of(v: &Value) -> usize {
    v.as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("not a count: {v}"))
}

/// The pixels where two images differ, as `(x, y, ours, reference)`.
fn differences(ours: &Gray, expected: &Gray) -> Vec<(usize, usize, u8, u8)> {
    assert_eq!(
        (ours.width, ours.height),
        (expected.width, expected.height),
        "image size"
    );
    ours.data
        .iter()
        .zip(&expected.data)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (&a, &b))| (i % ours.width, i / ours.width, a, b))
        .collect()
}

fn assert_same_image(ours: &Gray, expected: &Gray, what: &str) {
    let diff = differences(ours, expected);
    assert!(
        diff.is_empty(),
        "{what}: {} of {} pixels differ, first {:?}",
        diff.len(),
        ours.data.len(),
        &diff[..diff.len().min(8)]
    );
}

#[test]
fn resize_matches_opencv() {
    let dir = fixtures();
    let cases = cases();
    let mut checked = 0;
    for case in cases["resize"].as_array().unwrap() {
        let src = gray(&dir.join(format!("src_{}.npy", case["src"].as_str().unwrap())));
        let interpolation = match case["interp"].as_str().unwrap() {
            "linear" => Interpolation::Linear,
            _ => Interpolation::Area,
        };
        let (w, h) = (usize_of(&case["width"]), usize_of(&case["height"]));
        let ours = imgops::resize(&src, w, h, interpolation);
        let expected = gray(&dir.join(case["out"].as_str().unwrap()));
        assert_same_image(
            &ours,
            &expected,
            &format!(
                "resize {} {}x{} -> {w}x{h} {interpolation:?}",
                case["src"], src.width, src.height
            ),
        );
        checked += 1;
    }
    assert!(checked >= 40, "{checked} resize cases");
    eprintln!("resize: {checked} cases bit-exact");
}

#[test]
fn warp_matches_opencv() {
    let dir = fixtures();
    let cases = cases();
    let mut checked = 0;
    for case in cases["warp"].as_array().unwrap() {
        let src = gray(&dir.join(format!("src_{}.npy", case["src"].as_str().unwrap())));
        let m: [f64; 6] = f64s(&case["m"]).try_into().unwrap();
        let border = match case["border"].as_str().unwrap() {
            "replicate" => Border::Replicate,
            _ => Border::Constant(case.get("value").map_or(0, |v| usize_of(v) as u8)),
        };
        let (w, h) = (usize_of(&case["width"]), usize_of(&case["height"]));
        let ours = imgops::warp_affine(&src, &m, w, h, border);
        let expected = gray(&dir.join(case["out"].as_str().unwrap()));
        assert_same_image(
            &ours,
            &expected,
            &format!("warp {} {m:?} {border:?}", case["src"]),
        );
        checked += 1;
    }
    assert!(checked >= 30, "{checked} warp cases");
    eprintln!("warp: {checked} cases bit-exact");
}

#[test]
fn rotation_matrices_match_opencv() {
    for case in cases()["rotation"].as_array().unwrap() {
        let (cx, cy, a) = (num(&case["cx"]), num(&case["cy"]), num(&case["degrees"]));
        let m = imgops::rotation_matrix(cx, cy, a);
        assert_eq!(m.to_vec(), f64s(&case["m"]), "getRotationMatrix2D {case}");
        assert_eq!(
            imgops::invert_affine(&m).to_vec(),
            f64s(&case["inverse"]),
            "invertAffineTransform {case}"
        );
    }
}

#[test]
fn components_match_opencv() {
    let dir = fixtures();
    for case in cases()["components"].as_array().unwrap() {
        let mask = load(&dir.join(case["mask"].as_str().unwrap()));
        let (h, w) = (mask.shape[0], mask.shape[1]);
        let mask: Vec<bool> = mask.as_u8().unwrap().iter().map(|&v| v != 0).collect();
        let (labels, components) = imgops::connected_components(&mask, w, h);
        let expected = load(&dir.join(case["labels"].as_str().unwrap()));
        let NpyData::I32(expected) = &expected.data else {
            panic!("int32 labels")
        };
        let ours: Vec<i32> = labels.iter().map(|&l| l as i32).collect();
        assert_eq!(&ours, expected, "labels of {}", case["mask"]);
        let stats: Vec<Vec<usize>> = components
            .iter()
            .map(|c| vec![c.left, c.top, c.width, c.height, c.area])
            .collect();
        let expected: Vec<Vec<usize>> = case["stats"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_array().unwrap().iter().map(usize_of).collect())
            .collect();
        assert_eq!(stats, expected, "stats of {}", case["mask"]);
    }
}

#[test]
fn percentiles_match_the_reference() {
    let mut numpy_differs = 0;
    let mut checked = 0;
    for case in cases()["percentile"].as_array().unwrap() {
        let values: Vec<u8> = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| usize_of(v) as u8)
            .collect();
        let qs = f64s(&case["q"]);
        let ours = imgops::percentiles(&values, &qs);
        let numpy = f64s(&case["numpy"]);
        // The reference's own histogram percentiles, at 2, 50 and 98.
        let reference = f64s(&case["reference"]);
        for (q, r) in [2.0, 50.0, 98.0].iter().zip(&reference) {
            assert_eq!(imgops::percentile(&values, *q), *r, "q {q} of {values:?}");
        }
        for ((q, o), n) in qs.iter().zip(&ours).zip(&numpy) {
            assert!(
                (o - n).abs() <= 1e-12 * n.abs().max(1.0),
                "q {q}: {o} vs numpy {n}"
            );
            numpy_differs += usize::from(o != n);
            checked += 1;
        }
    }
    eprintln!(
        "percentile: {checked} checked against numpy, {numpy_differs} differ in the last bit"
    );
}

#[test]
fn gray_decode_matches_imread() {
    let dir = fixtures();
    for case in cases()["gray"].as_array().unwrap() {
        let rgb = load(&dir.join(case["rgb"].as_str().unwrap()));
        let (h, w) = (rgb.shape[0], rgb.shape[1]);
        let ours = imgops::gray_from_rgb(rgb.as_u8().unwrap(), w, h);
        let expected = gray(&dir.join(case["gray"].as_str().unwrap()));
        assert_same_image(&ours, &expected, "IMREAD_GRAYSCALE");
    }
}

/// Boxes as their coordinates' bits (exact) and their scores (within the
/// tolerance); returns how many scores are not bit-identical.
fn assert_same_boxes(ours: &[DetBox], expected: &[[f32; 5]], what: &str) -> usize {
    assert_eq!(ours.len(), expected.len(), "{what}: box count");
    let mut inexact = 0;
    for (i, (b, e)) in ours.iter().zip(expected).enumerate() {
        assert_eq!(
            [b.x0, b.y0, b.x1, b.y1].map(f32::to_bits),
            [e[0], e[1], e[2], e[3]].map(f32::to_bits),
            "{what}: box {i} {b:?} vs {e:?}"
        );
        assert!(
            (b.score - e[4]).abs() <= 1e-6 * e[4].abs(),
            "{what}: box {i} score {} vs {}",
            b.score,
            e[4]
        );
        inexact += usize::from(b.score != e[4]);
    }
    inexact
}

fn boxes_of(v: &Value) -> Vec<[f32; 5]> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|b| {
            let b = f64s(b);
            std::array::from_fn(|i| b[i] as f32)
        })
        .collect()
}

#[test]
fn decode_matches_det_post() {
    let dir = fixtures();
    for case in cases()["decode"].as_array().unwrap() {
        let out = load(&dir.join(case["out"].as_str().unwrap()));
        let (h, w) = (out.shape[1], out.shape[2]);
        let boxes = det::decode(out.as_f32().unwrap(), h, w);
        let expected = boxes_of(&case["boxes"]);
        assert!(expected.len() >= 10, "{} boxes", expected.len());
        let inexact = assert_same_boxes(&boxes, &expected, "decode window");
        eprintln!(
            "decode window: {} boxes, coordinates bit-exact, {inexact} scores within 1e-6",
            boxes.len()
        );
    }
}

/// The pages `generate.py` dumped, with their traces.
fn heavy_pages(reference: &Path) -> (PathBuf, Value) {
    let dir = reference.join("imgops");
    let summary = json(&dir.join("summary.json"));
    (dir, summary)
}

#[test]
fn whole_pages_prepare_like_the_reference() {
    let Some(reference) = reference() else {
        return;
    };
    let (dir, summary) = heavy_pages(&reference);
    let mut pages = 0;
    let mut turns = 0;
    for (name, entry) in summary.as_object().unwrap() {
        if name == "unrotate_box" {
            continue;
        }
        let d = dir.join(name);
        let g = gray(&d.join("gray.npy"));
        let small = page::preview(&g);
        assert_same_image(
            &small,
            &gray(&d.join("preview.npy")),
            &format!("{name} preview"),
        );
        let fine = page::ink_thumb(&small, page::SKEW_SIDE);
        let coarse = page::ink_thumb(&small, page::SKEW_SIDE / 2);
        assert_same_image(
            &fine,
            &gray(&d.join("fine.npy")),
            &format!("{name} fine thumb"),
        );
        assert_same_image(
            &coarse,
            &gray(&d.join("coarse.npy")),
            &format!("{name} coarse thumb"),
        );

        let trace = &entry["skew"];
        if !trace["blank"].as_bool().unwrap() {
            let grid = f64s(&trace["coarse_grid"]);
            let scores = f64s(&trace["coarse_scores"]);
            for (a, s) in grid.iter().zip(&scores) {
                assert_eq!(
                    page::row_sharpness(&coarse, *a) as f64,
                    *s,
                    "{name} coarse {a}"
                );
            }
            for (a, s) in f64s(&trace["near"])
                .iter()
                .zip(&f64s(&trace["near_scores"]))
            {
                assert_eq!(page::row_sharpness(&fine, *a) as f64, *s, "{name} near {a}");
            }
            let last = f64s(&trace["last"]);
            let a1 = num(&trace["a1"]);
            assert_eq!(
                page::arange(a1 - 0.5, a1 + 0.5 + 1e-6, 0.1),
                last,
                "{name} last"
            );
            for (a, s) in last.iter().zip(&f64s(&trace["last_scores"])) {
                assert_eq!(page::row_sharpness(&fine, *a) as f64, *s, "{name} last {a}");
            }
        }
        let a = page::estimate_skew(&small);
        assert_eq!(a, num(&trace["angle"]), "{name} estimate_skew");

        let levelled = if a.abs() >= page::DESKEW_MIN {
            let l = page::rotate_page(&small, a);
            assert_same_image(
                &l,
                &gray(&d.join("levelled_preview.npy")),
                &format!("{name} levelled"),
            );
            l
        } else {
            small.clone()
        };
        let (ow, oh) = page::scaled_size(levelled.width, levelled.height, page::ORIENT_SIDE);
        let orient_small = imgops::resize(&levelled, ow, oh, Interpolation::Area);
        assert_same_image(
            &orient_small,
            &gray(&d.join("orient_small.npy")),
            &format!("{name} orient"),
        );

        for (k, t) in entry["turns"].as_object().unwrap() {
            let k: usize = k.parse().unwrap();
            let turned = page::turn(&g, k, a, &small);
            assert_same_image(
                &turned.gray,
                &gray(&d.join(format!("turned_{k}.npy"))),
                &format!("{name} turn {k}"),
            );
            assert_eq!(turned.skew, num(&t["skew"]), "{name} turn {k} skew");
            let x = det::page_to_input(&turned.gray, det::DET_LONG_SIDE);
            let shape: Vec<usize> = t["det_shape"]
                .as_array()
                .unwrap()
                .iter()
                .map(usize_of)
                .collect();
            assert_eq!(vec![x.height, x.width], shape, "{name} det shape");
            assert_eq!(x.scale, num(&t["det_scale"]), "{name} det scale");
            let det_small = gray(&d.join(format!("det_small_{k}.npy")));
            for y in 0..x.height {
                for xx in 0..x.width {
                    let v = if y < det_small.height && xx < det_small.width {
                        page::ink_value(det_small.get(xx, y))
                    } else {
                        0.0
                    };
                    assert_eq!(x.data[y * x.width + xx], v, "{name} det input ({xx}, {y})");
                }
            }
            turns += 1;
        }
        pages += 1;
    }
    assert!(pages >= 14, "{pages} pages");
    eprintln!(
        "whole pages: {pages} pages (preview, thumbnails, every skew score, estimate_skew, \
         orientation input) and {turns} turns with detector inputs, all bit-exact"
    );
}

#[test]
fn unrotated_boxes_match_the_reference() {
    let Some(reference) = reference() else {
        return;
    };
    let (_, summary) = heavy_pages(&reference);
    let cases = summary["unrotate_box"].as_array().unwrap();
    for case in cases {
        let b: [f64; 4] = f64s(&case["box"]).try_into().unwrap();
        let wh = f64s(&case["wh"]);
        let skew = num(&case["skew"]);
        let ours = page::unrotate_box(b, skew, wh[0] as usize, wh[1] as usize);
        assert_eq!(ours.to_vec(), f64s(&case["out"]), "unrotate_box {case}");
    }
    eprintln!("unrotate_box: {} cases exact", cases.len());
}

/// The pages of `stages/` and `intermediate/`: their inputs to the
/// orientation network and the detector, the upright page, and the
/// detector's real output decoded to the reference's page boxes.
#[test]
fn stage_pages_match_the_reference() {
    let Some(reference) = reference() else {
        return;
    };
    let names = [
        "multi_column",
        "tilted_memo",
        "table_pmc",
        "document-image",
        "rotated-low-resolution-scan",
    ];
    let mut decoded = 0;
    for name in names {
        let stages = json(&reference.join(format!("stages/{name}.stages.json")));
        let inter = reference.join(format!("intermediate/{name}"));
        let g = if inter.is_dir() {
            gray(&inter.join("page.gray.npy"))
        } else {
            gray(&reference.join(format!("imgops/{name}/gray.npy")))
        };
        let small = page::preview(&g);
        let a = page::estimate_skew(&small);
        assert_eq!(a, num(&stages["estimate_skew"]), "{name} estimate_skew");
        let k = usize_of(&stages["k"]);
        let turned = page::turn(&g, k, a, &small);
        assert_eq!(turned.skew, num(&stages["skew_applied"]), "{name} skew");
        let upright = gray(&reference.join(format!("stages/{name}.upright.npy")));
        assert_same_image(&turned.gray, &upright, &format!("{name} upright"));

        if !inter.is_dir() {
            continue;
        }
        let summary = json(&inter.join("summary.json"));
        assert_eq!(a, num(&summary["estimate_skew_deg"]), "{name} summary skew");
        let levelled = if a.abs() >= page::DESKEW_MIN {
            page::rotate_page(&small, a)
        } else {
            small.clone()
        };
        let orient = load(&inter.join("orient.in.x.npy"));
        assert!(
            page::orient_input(&levelled) == orient.as_f32().unwrap(),
            "{name} orient input"
        );
        let probabilities = load(&inter.join("orient.out.p.npy"));
        assert_eq!(
            page::orientation_class(probabilities.as_f32().unwrap()),
            k,
            "{name} k"
        );

        let x = det::page_to_input(&turned.gray, det::DET_LONG_SIDE);
        let det_in = load(&inter.join("det.in.x.npy"));
        assert_eq!(
            det_in.shape,
            vec![1, 1, x.height, x.width],
            "{name} det input shape"
        );
        assert!(x.data == det_in.as_f32().unwrap(), "{name} det input");

        let out = load(&inter.join("det.out.out.npy"));
        let (h, w) = (out.shape[2], out.shape[3]);
        let boxes = det::decode(out.as_f32().unwrap(), h, w);
        // The reference ran no second pass on these pages.
        assert_eq!(det::adaptive_long_side(&boxes), None, "{name} adaptive");
        let boxes = det::to_page(boxes, x.scale, turned.gray.width, turned.gray.height);
        let expected = boxes_of(&stages["det_boxes"]);
        let inexact = assert_same_boxes(&boxes, &expected, name);
        eprintln!(
            "{name}: {} det boxes, coordinates bit-exact, {inexact} scores within 1e-6",
            boxes.len()
        );
        decoded += 1;
    }
    assert_eq!(decoded, 4);
}

/// Not a check: how long preparing a large page takes (`--nocapture`).
#[test]
fn preparation_timing() {
    let Some(reference) = reference() else {
        return;
    };
    let g = gray(&reference.join("imgops/scan_tilt/gray.npy"));
    let t = std::time::Instant::now();
    let small = page::preview(&g);
    let preview = t.elapsed();
    let a = page::estimate_skew(&small);
    let skew = t.elapsed() - preview;
    let a4 = page::estimate_skew_threaded(&small, 4);
    let skew4 = t.elapsed() - preview - skew;
    assert_eq!(a, a4);
    let turned = page::turn(&g, 0, a, &small);
    let turn = t.elapsed() - preview - skew - skew4;
    let x = det::page_to_input(&turned.gray, det::DET_LONG_SIDE);
    let input = t.elapsed() - preview - skew - skew4 - turn;
    eprintln!(
        "{}x{} page: preview {preview:?}, estimate_skew {skew:?} ({skew4:?} on 4 threads), \
         turn {turn:?}, detector input {input:?} ({}x{})",
        g.width, g.height, x.width, x.height
    );
}

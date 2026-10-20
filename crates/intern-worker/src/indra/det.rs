//! The line detector's input and the decoding of its output into line
//! boxes (spec §5).
//!
//! `det.onnx` reads the page as an ink map scaled so its longer side is
//! 1024 pixels, and marks, at half that resolution, each text line's core
//! band and the distances from every core pixel up to the line's top and
//! down to its bottom. A line is a 4-connected core region; its box spans
//! the region's columns, widened a little at both ends, and the mean of
//! the regressed tops and bottoms (`det_post.py`). Pages whose lines come
//! out too small are detected again at a larger scale
//! ([`adaptive_long_side`]).

use super::imgops::{self, Interpolation};
use super::page::ink_value;
use super::types::{DetBox, Gray};

/// The detector's first-pass longer side, `meta.det_long_side`.
pub const DET_LONG_SIDE: usize = 1024;
/// The detector's input sides are multiples of this (`INPUT_MULTIPLE`,
/// `det_post.py:17`).
pub const INPUT_MULTIPLE: usize = 32;
/// Input pixels per output pixel (`OUT_STRIDE`, `det_post.py:16`).
pub const OUT_STRIDE: usize = 2;
/// The core band is shorter than its line by this fraction of the line's
/// height at each end, so the box is widened back by it
/// (`CORE_X_SHRINK`, `det_post.py:15`).
pub const CORE_X_SHRINK: f64 = 0.15;
/// The core probability a pixel must exceed (`decode(thresh=0.5)`).
pub const CORE_THRESHOLD: f32 = 0.5;
/// Smaller core regions are dropped (`decode(min_area=2)`).
pub const MIN_AREA: usize = 2;
/// A first pass whose median line is shorter than this many input pixels
/// is run again larger (`ADAPTIVE_MIN_H`, `pipeline.py:43`).
pub const ADAPTIVE_MIN_H: f64 = 9.0;
/// The median line height, input pixels, the second pass aims for
/// (`ADAPTIVE_TARGET_H`, `pipeline.py:46`).
pub const ADAPTIVE_TARGET_H: f64 = 12.0;
/// The second pass's longest side (`ADAPTIVE_MAX_SIDE`, `pipeline.py:54`).
pub const ADAPTIVE_MAX_SIDE: usize = 2048;
/// The fewest first-pass lines that can trigger a second pass
/// (`pipeline.py:383`).
pub const ADAPTIVE_MIN_LINES: usize = 8;

/// The detector's input for one pass.
#[derive(Clone, Debug, PartialEq)]
pub struct DetInput {
    /// `[1, 1, height, width]` ink values, padded with 0.
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
    /// Detector pixels per page pixel.
    pub scale: f64,
}

/// `page_to_input(gray, long_side)` (spec §5.1).
///
/// The page scaled so its longer side is `long_side` (area averaging to
/// shrink, bilinear to enlarge), as ink `1 - v / 255`, padded with 0 (white)
/// on the right and at the bottom to multiples of 32.
pub fn page_to_input(gray: &Gray, long_side: usize) -> DetInput {
    let (w, h) = (gray.width, gray.height);
    let scale = long_side as f64 / w.max(h) as f64;
    let nh = ((h as f64 * scale).round_ties_even() as usize).max(1);
    let nw = ((w as f64 * scale).round_ties_even() as usize).max(1);
    let resized;
    let page = if (nw, nh) == (w, h) {
        gray
    } else {
        let interpolation = if scale < 1.0 {
            Interpolation::Area
        } else {
            Interpolation::Linear
        };
        resized = imgops::resize(gray, nw, nh, interpolation);
        &resized
    };
    let height = nh.div_ceil(INPUT_MULTIPLE) * INPUT_MULTIPLE;
    let width = nw.div_ceil(INPUT_MULTIPLE) * INPUT_MULTIPLE;
    let mut data = vec![0f32; width * height];
    for (out_row, row) in data.chunks_exact_mut(width).zip(page.data.chunks_exact(nw)) {
        for (o, &v) in out_row.iter_mut().zip(row) {
            *o = ink_value(v);
        }
    }
    DetInput {
        data,
        width,
        height,
        scale,
    }
}

/// Decodes the detector's `[3, h, w]` output into boxes in detector-input
/// pixels, in component order (spec §5.2).
///
/// `out` may hold more channels after the three read (a picture channel);
/// they are ignored. The arithmetic follows `det_post.decode`: the core
/// probability `1 / (1 + exp(-logit))` in single precision; each output
/// row's centre `(r + 0.5) * 2` in single precision, the regressed top and
/// bottom subtracted from and added to it in single precision; the sums
/// per component in double precision in raster order (`np.bincount`),
/// divided by the pixel count; the box in double precision, rounded to
/// single at the end. Coordinates match the reference bit for bit; the
/// score can differ in its last bits because numpy's vectorized `exp` is
/// not the C library's.
///
/// # Panics
/// When `out` holds fewer than `3 * height * width` values.
pub fn decode(out: &[f32], height: usize, width: usize) -> Vec<DetBox> {
    let plane = height * width;
    assert!(out.len() >= 3 * plane, "detector output size");
    let (logit, rest) = out.split_at(plane);
    let (up, rest) = rest.split_at(plane);
    let down = &rest[..plane];
    let prob: Vec<f32> = logit.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
    let mask: Vec<bool> = prob.iter().map(|&p| p > CORE_THRESHOLD).collect();
    let (labels, components) = imgops::connected_components(&mask, width, height);
    if components.is_empty() {
        return Vec::new();
    }
    // (sum of tops, sum of bottoms, sum of probabilities) per component.
    let mut sums = vec![[0f64; 3]; components.len()];
    for (r, row) in labels.chunks_exact(width).enumerate() {
        let centre = (r as f32 + 0.5) * OUT_STRIDE as f32;
        for (c, &label) in row.iter().enumerate() {
            if label == 0 {
                continue;
            }
            let i = r * width + c;
            let s = &mut sums[label as usize - 1];
            s[0] += f64::from(centre - up[i]);
            s[1] += f64::from(centre + down[i]);
            s[2] += f64::from(prob[i]);
        }
    }
    let stride = OUT_STRIDE as f64;
    components
        .iter()
        .zip(&sums)
        .filter(|(c, _)| c.area >= MIN_AREA)
        .map(|(c, s)| {
            let count = f64::from(c.area as f32);
            let (y0, y1, score) = (s[0] / count, s[1] / count, s[2] / count);
            let lh = (y1 - y0).max(1.0);
            let ex = (CORE_X_SHRINK * lh).min(0.5 * lh);
            DetBox {
                x0: (c.left as f64 * stride - ex) as f32,
                y0: y0 as f32,
                x1: ((c.left + c.width) as f64 * stride + ex) as f32,
                y1: y1 as f32,
                score: score as f32,
            }
        })
        .collect()
}

/// The longer side of the adaptive second pass, when the first pass's lines
/// are too small to read well (spec §5.3).
///
/// With at least eight boxes (detector-input pixels) whose median height
/// is under 9 pixels: `1024 * 12 / median`, at most 2048, truncated to a
/// multiple of 32, when that is larger than 1024. The median is numpy's:
/// the mean of the two middle heights for an even count, in single
/// precision as the box array is.
pub fn adaptive_long_side(boxes: &[DetBox]) -> Option<usize> {
    if boxes.len() < ADAPTIVE_MIN_LINES {
        return None;
    }
    let mut heights: Vec<f32> = boxes.iter().map(|b| b.y1 - b.y0).collect();
    heights.sort_by(f32::total_cmp);
    let n = heights.len();
    let median = if n % 2 == 1 {
        heights[n / 2]
    } else {
        (heights[n / 2 - 1] + heights[n / 2]) / 2.0
    };
    let median = f64::from(median);
    if median.is_nan() || median >= ADAPTIVE_MIN_H {
        return None;
    }
    let target =
        (DET_LONG_SIDE as f64 * ADAPTIVE_TARGET_H / median.max(1.0)).min(ADAPTIVE_MAX_SIDE as f64);
    let side = target as usize / 32 * 32;
    (side > DET_LONG_SIDE).then_some(side)
}

/// Boxes from detector-input pixels to page pixels, clipped to the page
/// (spec §5.3).
///
/// numpy divides the single-precision boxes in place by the scale, which
/// numpy 2 rounds to single precision first, and clips x to
/// `[0, page_width]` and y to `[0, page_height]`. The score is kept.
pub fn to_page(
    boxes: Vec<DetBox>,
    scale: f64,
    page_width: usize,
    page_height: usize,
) -> Vec<DetBox> {
    let s = scale as f32;
    let (w, h) = (page_width as f32, page_height as f32);
    boxes
        .into_iter()
        .map(|b| DetBox {
            x0: (b.x0 / s).max(0.0).min(w),
            y0: (b.y0 / s).max(0.0).min(h),
            x1: (b.x1 / s).max(0.0).min(w),
            y1: (b.y1 / s).max(0.0).min(h),
            score: b.score,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A detector output with the given core logits and constant distances.
    fn output(logits: &[f32], up: f32, down: f32) -> Vec<f32> {
        let mut out = logits.to_vec();
        out.extend(std::iter::repeat_n(up, logits.len()));
        out.extend(std::iter::repeat_n(down, logits.len()));
        out
    }

    #[test]
    fn input_is_scaled_padded_ink() {
        // 560 x 300 -> 1024 x 549 (300 * 1.828... = 548.57), padded to 576.
        let x = page_to_input(&Gray::filled(560, 300, 0), DET_LONG_SIDE);
        assert_eq!((x.width, x.height), (1024, 576));
        assert_eq!(x.scale, 1024.0 / 560.0);
        assert_eq!(x.data[548 * 1024 + 1023], 1.0);
        assert_eq!(x.data[549 * 1024], 0.0);
        // An exact fit is not resized.
        let g = Gray::new(64, 32, (0..64 * 32).map(|i| (i % 256) as u8).collect());
        let x = page_to_input(&g, 64);
        assert_eq!((x.width, x.height, x.scale), (64, 32, 1.0));
        assert_eq!(x.data[255], 0.0);
        assert_eq!(x.data[0], 1.0);
    }

    #[test]
    fn decode_measures_each_core() {
        // Two cores on a 3 x 6 map: a 2 x 3 block and a single pixel (too
        // small), plus a pixel at exactly 0.5 probability (not above it).
        #[rustfmt::skip]
        let logits = [
            -9.0,  3.0, 3.0, 3.0, -9.0, 0.0,
            -9.0,  3.0, 3.0, 3.0, -9.0, -9.0,
            -9.0, -9.0, -9.0, -9.0, -9.0, 4.0,
        ];
        let boxes = decode(&output(&logits, 4.0, 6.0), 3, 6);
        assert_eq!(boxes.len(), 1);
        let b = boxes[0];
        // Row centres 1 and 3: tops -3 and -1, bottoms 7 and 9.
        assert_eq!((b.y0, b.y1), (-2.0, 8.0));
        // Height 10: widened by 1.5 at each end of columns 1-3 (x 2-8).
        assert_eq!((b.x0, b.x1), (0.5, 9.5));
        let p = 1.0 / (1.0 + (-3.0f32).exp());
        assert_eq!(b.score, p);
        assert!(decode(&output(&[-1.0; 4], 0.0, 0.0), 2, 2).is_empty());
    }

    #[test]
    fn short_lines_widen_by_at_least_a_pixel_height() {
        // Tops and bottoms that cross: the height counts as 1.
        let boxes = decode(&output(&[5.0, 5.0], 0.0, -3.0), 1, 2);
        assert_eq!(boxes[0].x0, -0.15f64 as f32);
        assert_eq!(boxes[0].x1, 4.15f64 as f32);
    }

    fn line(y0: f32, h: f32) -> DetBox {
        DetBox {
            x0: 0.0,
            y0,
            x1: 100.0,
            y1: y0 + h,
            score: 0.9,
        }
    }

    #[test]
    fn small_lines_trigger_a_larger_pass() {
        let boxes: Vec<DetBox> = (0..8).map(|i| line(i as f32 * 20.0, 6.0)).collect();
        // 1024 * 12 / 6 = 2048.
        assert_eq!(adaptive_long_side(&boxes), Some(2048));
        let boxes: Vec<DetBox> = (0..9).map(|i| line(i as f32 * 20.0, 8.5)).collect();
        // 12288 / 8.5 = 1445.6 -> 1440.
        assert_eq!(adaptive_long_side(&boxes), Some(1440));
        assert_eq!(adaptive_long_side(&boxes[..7]), None);
        let tall: Vec<DetBox> = (0..8).map(|i| line(i as f32 * 20.0, 9.0)).collect();
        assert_eq!(adaptive_long_side(&tall), None);
        // Even count: the median is the mean of 6 and 12 = 9, not small.
        let mixed: Vec<DetBox> = (0..8)
            .map(|i| line(i as f32 * 20.0, if i < 4 { 6.0 } else { 12.0 }))
            .collect();
        assert_eq!(adaptive_long_side(&mixed), None);
        // Tiny lines are capped at the largest side.
        let tiny: Vec<DetBox> = (0..8).map(|i| line(i as f32 * 20.0, 0.5)).collect();
        assert_eq!(adaptive_long_side(&tiny), Some(2048));
        // 12288 / 8.99 = 1366.9 -> 1344 (> 1024).
        let near: Vec<DetBox> = (0..8).map(|i| line(i as f32 * 20.0, 8.99)).collect();
        assert_eq!(adaptive_long_side(&near), Some(1344));
    }

    #[test]
    fn page_boxes_are_scaled_and_clipped() {
        let boxes = vec![DetBox {
            x0: -2.0,
            y0: 10.0,
            x1: 1100.0,
            y1: 30.0,
            score: 0.7,
        }];
        let page = to_page(boxes, 1024.0 / 900.0, 900, 1200);
        let s = (1024.0f64 / 900.0) as f32;
        assert_eq!(page[0].x0, 0.0);
        assert_eq!(page[0].y0, 10.0 / s);
        assert_eq!(page[0].x1, 900.0);
        assert_eq!(page[0].y1, 30.0 / s);
        assert_eq!(page[0].score, 0.7);
    }
}

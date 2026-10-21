//! Cutting lines out for the recognizer, batching them, and decoding what
//! it reports (spec §6).
//!
//! The arithmetic follows the reference's numpy expressions operation for
//! operation, so crop sizes, batch membership, confidences and token
//! positions come out bit-identical given the same boxes and network
//! outputs.

use super::beam::{BeamParams, beam_search};
use super::imgops::{self, Interpolation};
use super::lm::NgramLm;
use super::types::{DetBox, Gray, RecLine, Token, TokenKind};
use super::vocab::Vocab;

/// Lines read with a lower confidence keep their box and lose their text.
pub const MIN_CONF: f32 = 0.5;

/// The base recognizer's lines are searched again under the language model
/// when their greedy confidence is at least [`MIN_CONF`] and below this.
pub use super::beam::LM_GATE;

/// The band crop's height (`meta.rec_height`, `LINE_HEIGHT`).
pub const LINE_HEIGHT: usize = 32;
/// Room above and below a line, in line heights, in the band crop.
pub const PAD_Y: f64 = 0.2;
/// Room left and right of a line, in line heights.
pub const PAD_X: f64 = 0.3;
/// Crop widths are padded to a multiple of this, the recognizer's stride
/// over two time steps.
pub const WIDTH_MULTIPLE: usize = 8;
/// The widest crop.
pub const MAX_WIDTH: usize = 4096;

/// Most lines in one recognizer batch.
pub const MAX_BATCH: usize = 64;
/// A batch takes lines up to this many times as wide as its first.
pub const MAX_RATIO: f64 = 1.3;
/// Narrower lines count as this wide when batched.
pub const MIN_BATCH_WIDTH: usize = 64;
/// Lines times width a batch may hold, in pixel columns.
pub const MAX_AREA: usize = 64 * 640;

/// `is_dark_line`: whether the line is light text on a dark ground - the
/// median of its box lies nearer the dark end than the text does (spec
/// §6.1).
pub fn is_dark_line(gray: &Gray, bbox: &DetBox) -> bool {
    let (x0, y0, x1, y1) = (
        f64::from(bbox.x0),
        f64::from(bbox.y0),
        f64::from(bbox.x1),
        f64::from(bbox.y1),
    );
    // Python's `int()` truncates toward zero; slices clamp at both ends.
    let clamp = |start: f64, end: f64, size: usize| {
        let start = (start as i64).max(0) as usize;
        let end = ((end as i64) + 1).clamp(0, size as i64) as usize;
        (start.min(size), end)
    };
    let (top, bottom) = clamp(y0, y1, gray.height);
    let (left, right) = clamp(x0, x1, gray.width);
    if top >= bottom || left >= right {
        return false;
    }
    let mut region = Vec::with_capacity((bottom - top) * (right - left));
    for row in top..bottom {
        region.extend_from_slice(&gray.data[row * gray.width + left..row * gray.width + right]);
    }
    let lo = imgops::percentile(&region, 2.0);
    let med = imgops::percentile(&region, 50.0);
    let hi = imgops::percentile(&region, 98.0);
    hi - med > 2.0 * (med - lo) + 8.0
}

/// Where a line's crop sits on the page and the size it is resized to
/// (spec §6.2), without the pixels: what batching and token positions need.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CropGeometry {
    /// The crop's left and top edges in page pixels; may lie off the page.
    pub ix0: i64,
    pub iy0: i64,
    /// The crop's size in page pixels.
    pub cw: usize,
    pub ch: usize,
    /// The resized crop's size.
    pub width: usize,
    pub height: usize,
}

/// The geometry of `crop_line(gray, box, window=window)`.
pub fn crop_geometry(bbox: &DetBox, window: usize) -> CropGeometry {
    let pad_y = (window as f64 * (1.0 + 2.0 * PAD_Y) - 1.0) / 2.0;
    let height = LINE_HEIGHT * window;
    let (x0, y0, x1, y1) = (
        f64::from(bbox.x0),
        f64::from(bbox.y0),
        f64::from(bbox.x1),
        f64::from(bbox.y1),
    );
    let h = (y1 - y0).max(1.0);
    let (left, right) = (x0 - PAD_X * h, x1 + PAD_X * h);
    let (top, bottom) = (y0 - pad_y * h, y1 + pad_y * h);
    let (ix0, iy0) = (left.floor() as i64, top.floor() as i64);
    let (ix1, iy1) = (right.ceil() as i64, bottom.ceil() as i64);
    let cw = (ix1 - ix0).max(1) as usize;
    let ch = (iy1 - iy0).max(1) as usize;
    // Python's `round` of a true division: half to even.
    let scaled = ((cw * height) as f64 / ch as f64).round_ties_even() as usize;
    CropGeometry {
        ix0,
        iy0,
        cw,
        ch,
        width: scaled.clamp(WIDTH_MULTIPLE, MAX_WIDTH),
        height,
    }
}

/// A line cut out for the recognizer: `height` rows of ink values.
#[derive(Clone, Debug, PartialEq)]
pub struct LineCrop {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
    /// The crop's left edge and width in page pixels, before resizing.
    pub ix0: i64,
    pub cw: usize,
}

/// `crop_line(gray, box, window=window, dark=dark)` (spec §6.2): window 1
/// gives the 32-pixel band, window 3 the 96-pixel tall crop.
pub fn crop_line(gray: &Gray, bbox: &DetBox, window: usize, dark: bool) -> LineCrop {
    let geometry = crop_geometry(bbox, window);
    let CropGeometry {
        ix0,
        iy0,
        cw,
        ch,
        width,
        height,
    } = geometry;
    let mut crop = Gray::filled(cw, ch, if dark { 0 } else { 255 });
    // The part of the crop on the page.
    let sx0 = ix0.max(0);
    let sy0 = iy0.max(0);
    let sx1 = (ix0 + cw as i64).min(gray.width as i64);
    let sy1 = (iy0 + ch as i64).min(gray.height as i64);
    if sx1 > sx0 && sy1 > sy0 {
        let run = (sx1 - sx0) as usize;
        for y in sy0..sy1 {
            let source = y as usize * gray.width + sx0 as usize;
            let target = (y - iy0) as usize * cw + (sx0 - ix0) as usize;
            crop.data[target..target + run].copy_from_slice(&gray.data[source..source + run]);
        }
    }
    let interpolation = if ch > height {
        Interpolation::Area
    } else {
        Interpolation::Linear
    };
    let resized = imgops::resize(&crop, width, height, interpolation);
    // `resized.astype(float32) / 255.0`, inverted unless the line is dark.
    let data = resized
        .data
        .iter()
        .map(|&value| {
            let ink = f32::from(value) / 255.0;
            if dark { ink } else { 1.0 - ink }
        })
        .collect();
    LineCrop {
        data,
        width,
        height,
        ix0,
        cw,
    }
}

/// `width_batches(widths)`: indices grouped into batches (spec §6.3).
///
/// Lines are taken narrowest first (ties in index order). A batch closes at
/// [`MAX_BATCH`] lines, when the next line is wider than [`MAX_RATIO`]
/// times its first (widths under [`MIN_BATCH_WIDTH`] counting as that), or
/// when lines times width would pass [`MAX_AREA`].
pub fn width_batches(widths: &[usize]) -> Vec<Vec<usize>> {
    let mut order: Vec<usize> = (0..widths.len()).collect();
    order.sort_by_key(|&index| widths[index]);
    let mut batches: Vec<Vec<usize>> = Vec::new();
    for index in order {
        let width = widths[index].max(MIN_BATCH_WIDTH);
        let joins = batches.last().is_some_and(|batch| {
            let first = widths[batch[0]].max(MIN_BATCH_WIDTH);
            batch.len() < MAX_BATCH
                && width as f64 <= MAX_RATIO * first as f64
                && (batch.len() + 1) * width <= MAX_AREA
        });
        match batches.last_mut() {
            Some(batch) if joins => batch.push(index),
            _ => batches.push(vec![index]),
        }
    }
    batches
}

/// `pad_batch`: the crops right-padded with 0 to a common width that is a
/// multiple of 8, as `[B, 1, H, W]`, with the true widths. Returns the
/// data, `H`, `W` and the widths.
pub fn pad_batch(crops: &[&LineCrop]) -> (Vec<f32>, usize, usize, Vec<i64>) {
    let height = crops.first().map_or(0, |crop| crop.height);
    let widest = crops.iter().map(|crop| crop.width).max().unwrap_or(0);
    let width = widest.div_ceil(WIDTH_MULTIPLE) * WIDTH_MULTIPLE;
    let mut data = vec![0.0_f32; crops.len() * height * width];
    for (item, crop) in crops.iter().enumerate() {
        assert_eq!(crop.height, height, "crops of one batch share a height");
        for row in 0..height {
            let target = (item * height + row) * width;
            data[target..target + crop.width]
                .copy_from_slice(&crop.data[row * crop.width..(row + 1) * crop.width]);
        }
    }
    let widths = crops.iter().map(|crop| crop.width as i64).collect();
    (data, height, width, widths)
}

/// numpy's sum of a contiguous `float32` array: pairwise, in `float32`,
/// eight accumulators to a block of up to 128 (numpy's `pairwise_sum`).
fn pairwise_sum(values: &[f32]) -> f32 {
    let n = values.len();
    if n < 8 {
        values.iter().fold(0.0_f32, |sum, &value| sum + value)
    } else if n <= 128 {
        let mut r = [0.0_f32; 8];
        r.copy_from_slice(&values[..8]);
        let mut i = 8;
        while i < n - n % 8 {
            for (j, accumulator) in r.iter_mut().enumerate() {
                *accumulator += values[i + j];
            }
            i += 8;
        }
        let mut sum = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        for &value in &values[i..] {
            sum += value;
        }
        sum
    } else {
        let mut half = n / 2;
        half -= half % 8;
        pairwise_sum(&values[..half]) + pairwise_sum(&values[half..])
    }
}

/// numpy's `mean` of a `float32` array, to the bit.
fn numpy_mean(values: &[f32]) -> f32 {
    pairwise_sum(values) / values.len() as f32
}

/// The greedy reading of one line from its argmax ids and max
/// probabilities over its `n` valid steps: the emitted class ids, the steps
/// that emitted them, and the line confidence (spec §6.5) - the mean
/// probability over the emitted steps, 0 when none is.
pub fn greedy(ids: &[i64], conf: &[f32]) -> (Vec<u32>, Vec<usize>, f32) {
    let mut emitted = Vec::new();
    let mut steps = Vec::new();
    let mut probabilities = Vec::new();
    for (step, &id) in ids.iter().enumerate() {
        if id != 0 && (step == 0 || id != ids[step - 1]) {
            emitted.push(id as u32);
            steps.push(step);
            probabilities.push(conf[step]);
        }
    }
    let confidence = if probabilities.is_empty() {
        0.0
    } else {
        numpy_mean(&probabilities)
    };
    (emitted, steps, confidence)
}

/// The page x position of every emitted step (spec §6.5): `stride` is the
/// padded batch width over the time steps, `band_width` the line's band
/// crop width.
pub fn token_positions(bbox: &DetBox, steps: &[usize], stride: f64, band_width: usize) -> Vec<f64> {
    let (x0, y0, x1, y1) = (
        f64::from(bbox.x0),
        f64::from(bbox.y0),
        f64::from(bbox.x1),
        f64::from(bbox.y1),
    );
    let h = (y1 - y0).max(1.0);
    let ix0 = (x0 - PAD_X * h).floor();
    let cw = (x1 + PAD_X * h).ceil() - ix0;
    let f = cw / band_width as f64;
    // `ix0 + (t + 0.5) * stride * f`, evaluated left to right as numpy does.
    steps
        .iter()
        .map(|&step| ix0 + (step as f64 + 0.5) * stride * f)
        .collect()
}

/// Tokens from class ids, their positions and their probabilities.
pub fn id_tokens(ids: &[u32], xs: &[f64], ps: &[f32]) -> Vec<Token> {
    ids.iter()
        .zip(xs)
        .zip(ps)
        .map(|((&id, &x), &p)| Token {
            kind: TokenKind::Id(id),
            x,
            p,
        })
        .collect()
}

/// One line's recognizer outputs over its `n` valid steps: argmax class
/// and its probability, log P(blank), and the top `k` log-probabilities
/// and classes a step (`n * k`, row-major).
#[derive(Clone, Copy, Debug)]
pub struct LineOutputs<'a> {
    pub ids: &'a [i64],
    pub conf: &'a [f32],
    pub blank: &'a [f32],
    pub topv: &'a [f32],
    pub topi: &'a [i64],
    pub k: usize,
}

/// The recognizer's language model and how it is searched.
#[derive(Clone, Copy, Debug)]
pub struct LineLm<'a> {
    pub lm: &'a NgramLm,
    pub params: BeamParams,
    /// Lines whose greedy confidence is below this are searched again.
    /// Compared in `f64`, as the reference compares a Python float.
    pub gate: f64,
}

/// The probability of class `class` at `step`: the recognizer's own top-k
/// probability of it there, or, when the class is not among the step's top
/// k (only a beam can choose that), the step's greedy probability.
fn step_probability(out: &LineOutputs<'_>, step: usize, class: u32) -> f32 {
    let row = step * out.k..(step + 1) * out.k;
    out.topi[row.clone()]
        .iter()
        .zip(&out.topv[row])
        .find(|&(&candidate, _)| candidate == i64::from(class))
        .map_or(out.conf[step], |(_, &logp)| logp.exp())
}

/// One line of the base recognizer read from its outputs (spec §6.5): the
/// greedy reading, searched again under the language model when its
/// confidence is in `[MIN_CONF, gate)`; tokens placed on the page; empty
/// text under [`MIN_CONF`]. The confidence is always the greedy one.
///
/// Every token carries its probability: the step's max probability for a
/// greedy reading, which is the class's; for a step a beam chose, the
/// recognizer's probability of the class the beam chose there, from the
/// step's top k (see [`step_probability`]).
pub fn read_line(
    out: &LineOutputs<'_>,
    bbox: &DetBox,
    stride: f64,
    band_width: usize,
    vocab: &Vocab,
    lm: Option<LineLm<'_>>,
) -> RecLine {
    read_line_steps(out, bbox, stride, band_width, vocab, lm).0
}

/// [`read_line`], and the time step each of its tokens was emitted at.
pub fn read_line_steps(
    out: &LineOutputs<'_>,
    bbox: &DetBox,
    stride: f64,
    band_width: usize,
    vocab: &Vocab,
    lm: Option<LineLm<'_>>,
) -> (RecLine, Vec<usize>) {
    let (mut ids, mut steps, conf) = greedy(out.ids, out.conf);
    let mut probabilities: Vec<f32> = steps.iter().map(|&step| out.conf[step]).collect();
    if let Some(lm) = lm
        && (f64::from(MIN_CONF)..lm.gate).contains(&f64::from(conf))
    {
        (ids, steps) = beam_search(out.blank, out.topv, out.topi, out.k, lm.lm, lm.params);
        probabilities = ids
            .iter()
            .zip(&steps)
            .map(|(&class, &step)| step_probability(out, step, class))
            .collect();
    }
    if conf < MIN_CONF {
        let line = RecLine {
            text: String::new(),
            conf,
            tokens: Vec::new(),
        };
        return (line, Vec::new());
    }
    let xs = token_positions(bbox, &steps, stride, band_width);
    let line = RecLine {
        text: vocab.decode(&ids),
        conf,
        tokens: id_tokens(&ids, &xs, &probabilities),
    };
    (line, steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbox(x0: f32, y0: f32, x1: f32, y1: f32) -> DetBox {
        DetBox {
            x0,
            y0,
            x1,
            y1,
            score: 1.0,
        }
    }

    #[test]
    fn crop_geometry_follows_the_reference_arithmetic() {
        // h = 20: X0 = 4, X1 = 116; band Y0 = 6, Y1 = 34 -> 112 x 28,
        // resized to round(112 * 32 / 28) = 128.
        let band = crop_geometry(&bbox(10.0, 10.0, 110.0, 30.0), 1);
        assert_eq!(
            band,
            CropGeometry {
                ix0: 4,
                iy0: 6,
                cw: 112,
                ch: 28,
                width: 128,
                height: 32
            }
        );
        // The tall window: pad_y = (3 * 1.4 - 1) / 2, a hair under 1.6.
        let tall = crop_geometry(&bbox(10.0, 10.0, 110.0, 30.0), 3);
        assert_eq!((tall.iy0, tall.ch, tall.height), (-22, 84, 96));
        assert_eq!(tall.width, 128);
        // A long, thin line is never wider than 4096.
        assert_eq!(crop_geometry(&bbox(0.0, 0.0, 9000.0, 2.0), 1).width, 4096);
    }

    #[test]
    fn batches_group_similar_widths_narrowest_first() {
        // Narrow lines count as 64 wide, so they share a batch.
        assert_eq!(width_batches(&[10, 30, 60]), vec![vec![0, 1, 2]]);
        // 84 <= 1.3 * 64 = 83.2 fails: a new batch.
        assert_eq!(width_batches(&[64, 84]), vec![vec![0], vec![1]]);
        assert_eq!(width_batches(&[100, 80, 104]), vec![vec![1, 0, 2]]);
        // Ties keep index order.
        assert_eq!(width_batches(&[70, 70, 70]), vec![vec![0, 1, 2]]);
        // At most 64 lines.
        let many = vec![8; 65];
        let batches = width_batches(&many);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 64);
        // And at most 64 * 640 columns: 41 lines of 1000 pass it.
        let wide = vec![1000; 41];
        let batches = width_batches(&wide);
        assert_eq!(batches[0].len(), 40);
        assert_eq!(batches[1].len(), 1);
        assert!(width_batches(&[]).is_empty());
    }

    #[test]
    fn a_batch_is_padded_to_a_multiple_of_eight() {
        let crop = |width: usize, value: f32| LineCrop {
            data: vec![value; width * 2],
            width,
            height: 2,
            ix0: 0,
            cw: width,
        };
        let (a, b) = (crop(3, 1.0), crop(9, 0.5));
        let (data, height, width, widths) = pad_batch(&[&a, &b]);
        assert_eq!((height, width, widths), (2, 16, vec![3, 9]));
        assert_eq!(
            &data[..16],
            &[
                1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            ]
        );
        assert_eq!(data[32..41], [0.5; 9]);
        assert_eq!(data[41..48], [0.0; 7]);
    }

    #[test]
    fn greedy_collapses_repeats_and_drops_blanks() {
        let ids = [0, 5, 5, 0, 5, 7, 7, 0];
        let conf = [0.9, 0.8, 0.7, 0.9, 0.6, 1.0, 0.5, 0.9];
        let (emitted, steps, mean) = greedy(&ids, &conf);
        assert_eq!(emitted, vec![5, 5, 7]);
        assert_eq!(steps, vec![1, 4, 5]);
        assert_eq!(mean, (0.8_f32 + 0.6 + 1.0) / 3.0);
        assert_eq!(greedy(&[0, 0], &[0.9, 0.9]), (vec![], vec![], 0.0));
    }

    #[test]
    fn the_mean_is_numpys_pairwise_float32_sum() {
        // Values whose float32 sum depends on the order of addition.
        let values: Vec<f32> = (0..300).map(|i| 1.0 / (i as f32 + 1.0) + 0.3).collect();
        let naive = values.iter().fold(0.0_f32, |sum, &value| sum + value);
        let pairwise = pairwise_sum(&values);
        // Pairwise is closer to the exact sum.
        let exact: f64 = values.iter().map(|&value| f64::from(value)).sum();
        assert!((f64::from(pairwise) - exact).abs() <= (f64::from(naive) - exact).abs());
        assert_eq!(pairwise_sum(&[1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn token_positions_map_steps_back_through_the_band_crop() {
        // h = 20: ix0 = floor(10 - 6) = 4, cw = ceil(116) - 4 = 112;
        // band width 128: f = 0.875; stride 4.
        let xs = token_positions(&bbox(10.0, 10.0, 110.0, 30.0), &[0, 10], 4.0, 128);
        assert_eq!(xs, vec![4.0 + 0.5 * 4.0 * 0.875, 4.0 + 10.5 * 4.0 * 0.875]);
    }

    #[test]
    fn a_beam_chosen_step_takes_its_class_probability_from_the_top_k() {
        let out = LineOutputs {
            ids: &[3, 0],
            conf: &[0.6, 0.9],
            blank: &[-2.0, -0.1],
            topv: &[0.6_f32.ln(), 0.3_f32.ln(), (0.9_f32).ln(), (0.05_f32).ln()],
            topi: &[3, 4, 0, 4],
            k: 2,
        };
        assert!((step_probability(&out, 0, 4) - 0.3).abs() < 1e-6);
        assert!((step_probability(&out, 0, 3) - 0.6).abs() < 1e-6);
        // Not in the top k: the greedy probability.
        assert_eq!(step_probability(&out, 0, 9), 0.6);
    }

    /// The reference's own crops, batches and readings, dumped by
    /// `bench/tools/indra/dump_intermediates.py` and `dump_stages.py` into
    /// `INDRA_REFERENCE_DIR`. Without it these say so and pass.
    mod reference {
        use super::super::*;
        use crate::indra::meta::Meta;
        use crate::indra::npy;
        use std::path::{Path, PathBuf};

        const PAGES: [&str; 4] = [
            "multi_column",
            "tilted_memo",
            "document-image",
            "rotated-low-resolution-scan",
        ];

        fn reference() -> Option<PathBuf> {
            let directory = std::env::var_os("INDRA_REFERENCE_DIR").map(PathBuf::from);
            if directory.is_none() {
                eprintln!("INDRA_REFERENCE_DIR is not set: the reference comparisons are skipped");
            }
            directory
        }

        /// One run of the base recognizer: the batch's true widths and the
        /// six outputs.
        struct Call {
            widths: Vec<i64>,
            steps: usize,
            k: usize,
            ids: Vec<i64>,
            conf: Vec<f32>,
            lengths: Vec<i64>,
            blank: Vec<f32>,
            topv: Vec<f32>,
            topi: Vec<i64>,
        }

        impl Call {
            fn read(directory: &Path, prefix: &str) -> Self {
                let read =
                    |file: &str| npy::read(&directory.join(format!("{prefix}{file}.npy"))).unwrap();
                let ids = read("out.ids");
                let topv = read("out.topv");
                Call {
                    widths: read("in.widths").as_i64().unwrap().to_vec(),
                    steps: ids.shape[1],
                    k: topv.shape[2],
                    ids: ids.as_i64().unwrap().to_vec(),
                    conf: read("out.conf").as_f32().unwrap().to_vec(),
                    lengths: read("out.lengths").as_i64().unwrap().to_vec(),
                    blank: read("out.blank").as_f32().unwrap().to_vec(),
                    topv: topv.as_f32().unwrap().to_vec(),
                    topi: read("out.topi").as_i64().unwrap().to_vec(),
                }
            }

            /// The padded width the batch ran at.
            fn padded_width(&self) -> usize {
                (*self.widths.iter().max().unwrap() as usize).div_ceil(WIDTH_MULTIPLE)
                    * WIDTH_MULTIPLE
            }

            fn outputs(&self, item: usize) -> LineOutputs<'_> {
                let n = self.lengths[item] as usize;
                let row = item * self.steps..item * self.steps + n;
                let top = item * self.steps * self.k..(item * self.steps + n) * self.k;
                LineOutputs {
                    ids: &self.ids[row.clone()],
                    conf: &self.conf[row.clone()],
                    blank: &self.blank[row],
                    topv: &self.topv[top.clone()],
                    topi: &self.topi[top],
                    k: self.k,
                }
            }
        }

        struct Page {
            name: &'static str,
            boxes: Vec<DetBox>,
            /// The base recognizer's batches, as line indices.
            batches: Vec<Vec<usize>>,
            /// Whether the base reading is the line's (spec §6.5: a group
            /// recognizer reads its lines too, and the surer reading wins).
            base_won: Vec<bool>,
            rec: Vec<serde_json::Value>,
            upright: Gray,
            /// The first batch's padded crops.
            x: npy::NpyArray,
            /// Every batch's run where `calls/` was dumped, else the first.
            calls: Vec<Call>,
        }

        fn page(directory: &Path, name: &'static str) -> Page {
            let stages: serde_json::Value = serde_json::from_slice(
                &std::fs::read(directory.join(format!("stages/{name}.stages.json"))).unwrap(),
            )
            .unwrap();
            let boxes = stages["det_boxes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    let value = |index: usize| row[index].as_f64().unwrap() as f32;
                    DetBox {
                        x0: value(0),
                        y0: value(1),
                        x1: value(2),
                        y1: value(3),
                        score: value(4),
                    }
                })
                .collect::<Vec<_>>();
            let rec = stages["rec"].as_array().unwrap().clone();
            // The base recognizer reads the lines the script classifier
            // calls base, and those it is unsure of.
            let base: Vec<usize> = (0..boxes.len())
                .filter(|&index| {
                    stages["groups"][index] == "base" || stages["sure"][index] == false
                })
                .collect();
            let widths: Vec<usize> = base
                .iter()
                .map(|&index| crop_geometry(&boxes[index], 1).width)
                .collect();
            let batches = width_batches(&widths)
                .into_iter()
                .map(|batch| batch.into_iter().map(|position| base[position]).collect())
                .collect();
            // The base reading is the line's when no group recognizer read
            // it, or when the reading kept has class-id tokens (a group
            // recognizer's are characters).
            let base_won = (0..boxes.len())
                .map(|index| {
                    let tokens = rec[index]["tokens"].as_array().unwrap();
                    stages["groups"][index] == "base"
                        || (!tokens.is_empty() && tokens.iter().all(|token| token[0].is_u64()))
                })
                .collect();
            let upright = npy::read(&directory.join(format!("stages/{name}.upright.npy"))).unwrap();
            let upright = Gray::new(
                upright.shape[1],
                upright.shape[0],
                upright.as_u8().unwrap().to_vec(),
            );
            let intermediate = directory.join(format!("intermediate/{name}"));
            let mut calls = Vec::new();
            while intermediate
                .join(format!("calls/rec.{}.out.ids.npy", calls.len()))
                .is_file()
            {
                calls.push(Call::read(
                    &intermediate,
                    &format!("calls/rec.{}.", calls.len()),
                ));
            }
            if calls.is_empty() {
                calls.push(Call::read(&intermediate, "rec."));
            }
            Page {
                name,
                boxes,
                batches,
                base_won,
                rec,
                upright,
                x: npy::read(&intermediate.join("rec.in.x.npy")).unwrap(),
                calls,
            }
        }

        /// Each batch's line and position in it, its run, and the line's
        /// reference reading, for the lines whose base reading is theirs.
        fn base_lines(page: &Page) -> Vec<(usize, usize, &Call, &serde_json::Value)> {
            let mut lines = Vec::new();
            for (batch, call) in page.batches.iter().zip(&page.calls) {
                for (item, &index) in batch.iter().enumerate() {
                    if page.base_won[index] {
                        lines.push((index, item, call, &page.rec[index]));
                    }
                }
            }
            lines
        }

        fn assert_reading(page: &Page, index: usize, line: &RecLine, expected: &serde_json::Value) {
            let what = format!("{} line {index}", page.name);
            // The reference wrote float32 confidences as float64; back to
            // float32 they are exact whatever the JSON parser's last bit.
            assert_eq!(
                line.conf,
                expected["conf"].as_f64().unwrap() as f32,
                "{what}"
            );
            assert_eq!(line.text, expected["text"].as_str().unwrap(), "{what}");
            let tokens = expected["tokens"].as_array().unwrap();
            assert_eq!(line.tokens.len(), tokens.len(), "{what}: {:?}", line.text);
            for (token, expected) in line.tokens.iter().zip(tokens) {
                assert_eq!(
                    token.kind,
                    TokenKind::Id(expected[0].as_u64().unwrap() as u32),
                    "{what}"
                );
                // serde_json reads floats to within a bit of the nearest; a
                // position computed any other way is off by far more.
                let x = expected[1].as_f64().unwrap();
                assert!(
                    (token.x - x).abs() <= x.abs() * 1e-15,
                    "{what}: x {} against {x}",
                    token.x
                );
            }
        }

        /// Batches are formed from the band crops' widths and padded from
        /// the tall crops' widths, as the reference forms and pads them.
        #[test]
        fn batches_and_their_widths_are_the_references() {
            let Some(directory) = reference() else { return };
            for name in PAGES {
                let page = page(&directory, name);
                for (batch, call) in page.batches.iter().zip(&page.calls) {
                    let tall: Vec<i64> = batch
                        .iter()
                        .map(|&index| crop_geometry(&page.boxes[index], 3).width as i64)
                        .collect();
                    assert_eq!(tall, call.widths, "{name}");
                }
                if page.calls.len() > 1 {
                    assert_eq!(page.calls.len(), page.batches.len(), "{name}");
                }
                assert_eq!(
                    page.x.shape,
                    vec![page.batches[0].len(), 1, 96, page.calls[0].padded_width()],
                    "{name}"
                );
            }
        }

        /// The tall crops are the reference's to within OpenCV's rounding.
        #[test]
        fn the_first_batch_is_cut_as_the_reference_cuts_it() {
            let Some(directory) = reference() else { return };
            for name in PAGES {
                let page = page(&directory, name);
                let crops: Vec<LineCrop> = page.batches[0]
                    .iter()
                    .map(|&index| {
                        let dark = is_dark_line(&page.upright, &page.boxes[index]);
                        crop_line(&page.upright, &page.boxes[index], 3, dark)
                    })
                    .collect();
                let refs: Vec<&LineCrop> = crops.iter().collect();
                let (data, height, width, widths) = pad_batch(&refs);
                assert_eq!(widths, page.calls[0].widths, "{name}");
                assert_eq!(vec![crops.len(), 1, height, width], page.x.shape, "{name}");
                let expected = page.x.as_f32().unwrap();
                let worst = data
                    .iter()
                    .zip(expected)
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0_f32, f32::max);
                let off = data
                    .iter()
                    .zip(expected)
                    .filter(|(a, b)| (*a - *b).abs() > 1e-6)
                    .count();
                // One grey level is 1/255; OpenCV's area resize is matched
                // to within a level on a few pixels (spec §13).
                assert!(worst <= 1.5 / 255.0, "{name}: worst {worst}");
                assert!(
                    off * 1000 <= data.len(),
                    "{name}: {off} of {} values differ",
                    data.len()
                );
            }
        }

        /// Given the reference's recognizer outputs, the greedy readings,
        /// confidences and token positions are the reference's to the bit,
        /// for every line the language model was not asked about.
        #[test]
        fn greedy_readings_and_positions_are_the_references() {
            let Some(directory) = reference() else { return };
            let mut compared = 0;
            for name in PAGES {
                let page = page(&directory, name);
                for (index, item, call, expected) in base_lines(&page) {
                    let out = call.outputs(item);
                    let (emitted, steps, conf) = greedy(out.ids, out.conf);
                    let what = format!("{name} line {index}");
                    assert_eq!(conf, expected["conf"].as_f64().unwrap() as f32, "{what}");
                    if (0.5..LM_GATE).contains(&f64::from(conf)) {
                        continue;
                    }
                    let tokens = expected["tokens"].as_array().unwrap();
                    if conf < MIN_CONF {
                        assert!(tokens.is_empty(), "{what}");
                        continue;
                    }
                    let stride = call.padded_width() as f64 / call.steps as f64;
                    let band = crop_geometry(&page.boxes[index], 1).width;
                    let xs = token_positions(&page.boxes[index], &steps, stride, band);
                    let probabilities: Vec<f32> =
                        steps.iter().map(|&step| out.conf[step]).collect();
                    let line = RecLine {
                        text: expected["text"].as_str().unwrap().to_owned(),
                        conf,
                        tokens: id_tokens(&emitted, &xs, &probabilities),
                    };
                    assert_reading(&page, index, &line, expected);
                    compared += 1;
                }
            }
            assert!(compared >= 8, "only {compared} lines compared");
        }

        /// Every base reading - the language model's searches included -
        /// is the reference's: text, confidence, tokens and positions. Needs
        /// the vocabulary and language model, from `INTERN_RUNTIME_DIR`'s
        /// `indra-models`.
        #[test]
        fn every_base_reading_is_the_references() {
            let Some(directory) = reference() else { return };
            let Some(models) = std::env::var_os("INTERN_RUNTIME_DIR")
                .map(|runtime| PathBuf::from(runtime).join(crate::indra::MODEL_DIRECTORY))
                .filter(|models| models.join(crate::indra::META).is_file())
            else {
                eprintln!(
                    "INTERN_RUNTIME_DIR holds no Indra models: the readings are not compared"
                );
                return;
            };
            let meta = Meta::load(&models.join(crate::indra::META)).unwrap();
            let lm = NgramLm::load(&models.join(crate::indra::LANGUAGE_MODEL)).unwrap();
            let line_lm = LineLm {
                lm: &lm,
                params: BeamParams::default(),
                gate: LM_GATE,
            };
            let (mut compared, mut searched) = (0, 0);
            for name in PAGES {
                let page = page(&directory, name);
                for (index, item, call, expected) in base_lines(&page) {
                    let out = call.outputs(item);
                    let stride = call.padded_width() as f64 / call.steps as f64;
                    let band = crop_geometry(&page.boxes[index], 1).width;
                    let line = read_line(
                        &out,
                        &page.boxes[index],
                        stride,
                        band,
                        &meta.vocab,
                        Some(line_lm),
                    );
                    assert_reading(&page, index, &line, expected);
                    compared += 1;
                    searched += usize::from((0.5..LM_GATE).contains(&f64::from(line.conf)));
                }
            }
            eprintln!("{compared} readings compared, {searched} of them searched");
            assert!(searched > 0, "no line went to the language model");
        }
    }
}

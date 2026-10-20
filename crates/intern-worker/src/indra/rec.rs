//! Cutting lines out for the recognizer, batching them, and decoding what
//! it reports (spec §6).

use super::types::{DetBox, Gray, Token};

/// Lines read with a lower confidence keep their box and lose their text.
pub const MIN_CONF: f32 = 0.5;

/// `is_dark_line`: whether the line is light text on a dark ground.
pub fn is_dark_line(gray: &Gray, bbox: &DetBox) -> bool {
    let _ = (gray, bbox);
    todo!("spec §6.1")
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
    let _ = (gray, bbox, window, dark);
    todo!("spec §6.2")
}

/// `width_batches(widths)`: indices grouped into batches (spec §6.3).
pub fn width_batches(widths: &[usize]) -> Vec<Vec<usize>> {
    let _ = widths;
    todo!("spec §6.3")
}

/// `pad_batch`: the crops right-padded with 0 to a common width that is a
/// multiple of 8, as `[B, 1, H, W]`, with the true widths.
pub fn pad_batch(crops: &[&LineCrop]) -> (Vec<f32>, usize, usize, Vec<i64>) {
    let _ = crops;
    todo!("spec §6.3")
}

/// The greedy reading of one line from its argmax ids and max
/// probabilities over its `n` valid steps: the emitted class ids, the steps
/// that emitted them, and the line confidence (spec §6.5).
pub fn greedy(ids: &[i64], conf: &[f32]) -> (Vec<u32>, Vec<usize>, f32) {
    let _ = (ids, conf);
    todo!("spec §6.5")
}

/// The page x position of every emitted step (spec §6.5): `stride` is the
/// padded batch width over the time steps, `band_width` the line's band
/// crop width.
pub fn token_positions(
    bbox: &DetBox,
    steps: &[usize],
    stride: f64,
    band_width: usize,
) -> Vec<f64> {
    let _ = (bbox, steps, stride, band_width);
    todo!("spec §6.5")
}

/// Tokens from class ids and their positions.
pub fn id_tokens(ids: &[u32], xs: &[f64]) -> Vec<Token> {
    let _ = (ids, xs);
    todo!("spec §6.5")
}

//! The line detector's input and the decoding of its output into line
//! boxes (spec §5).

use super::types::{DetBox, Gray};

/// The detector's first-pass longer side, `meta.det_long_side`.
pub const DET_LONG_SIDE: usize = 1024;

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
pub fn page_to_input(gray: &Gray, long_side: usize) -> DetInput {
    let _ = (gray, long_side);
    todo!("spec §5.1")
}

/// Decodes the detector's `[3, h, w]` output into boxes in detector-input
/// pixels, in component order (spec §5.2).
pub fn decode(out: &[f32], height: usize, width: usize) -> Vec<DetBox> {
    let _ = (out, height, width);
    todo!("spec §5.2")
}

/// The longer side of the adaptive second pass, when the first pass's lines
/// are too small to read well (spec §5.3).
pub fn adaptive_long_side(boxes: &[DetBox]) -> Option<usize> {
    let _ = boxes;
    todo!("spec §5.3")
}

/// Boxes from detector-input pixels to page pixels, clipped to the page
/// (spec §5.3).
pub fn to_page(boxes: Vec<DetBox>, scale: f64, page_width: usize, page_height: usize) -> Vec<DetBox> {
    let _ = (boxes, scale, page_width, page_height);
    todo!("spec §5.3")
}

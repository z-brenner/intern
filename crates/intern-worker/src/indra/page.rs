//! A page's skew and quarter turn, found before detection, and the mapping
//! of boxes back from the levelled page (spec §4).

use super::types::Gray;

/// Tilts smaller than this many degrees are left alone.
pub const DESKEW_MIN: f64 = 0.3;
/// The orientation classifier's probability above which its turn is taken.
pub const ORIENT_MIN: f32 = 0.95;
/// The orientation class of a page set in vertical columns.
pub const VERTICAL: usize = 4;

/// The page shrunk so its longer side is at most 1600 pixels (`preview`).
pub fn preview(gray: &Gray) -> Gray {
    let _ = gray;
    todo!("spec §4")
}

/// Degrees counter-clockwise that straighten the page's rows
/// (`estimate_skew`, spec §4.1). 0 for a blank page.
pub fn estimate_skew(small: &Gray) -> f64 {
    let _ = small;
    todo!("spec §4.1")
}

/// The orientation classifier's input, `[768, 768]` ink values (spec §4.2).
pub fn orient_input(gray: &Gray) -> Vec<f32> {
    let _ = gray;
    todo!("spec §4.2")
}

/// The turn to take from the classifier's five probabilities: the argmax
/// class when it is above [`ORIENT_MIN`], else 0.
pub fn orientation_class(probabilities: &[f32]) -> usize {
    let _ = probabilities;
    todo!("spec §4.2")
}

/// `rotate_page`: the page turned `degrees` about its centre, edges
/// replicated.
pub fn rotate_page(gray: &Gray, degrees: f64) -> Gray {
    let _ = (gray, degrees);
    todo!("spec §4.3")
}

/// The page after `_turn` (spec §4.3).
#[derive(Clone, Debug, PartialEq)]
pub struct Turned {
    pub gray: Gray,
    /// The orientation class taken (0-4).
    pub k: usize,
    /// The skew straightened, degrees; 0 when under [`DESKEW_MIN`].
    pub skew: f64,
}

/// `_turn(gray, k, a, small)`: the page turned by class `k` and levelled by
/// `a`, re-estimating the skew after a quarter turn.
pub fn turn(gray: &Gray, k: usize, skew: f64, small: &Gray) -> Turned {
    let _ = (gray, k, skew, small);
    todo!("spec §4.3")
}

/// `unrotate_box`: a box on the levelled page mapped back to the page
/// before levelling, clipped to it and rounded to 0.1.
pub fn unrotate_box(bbox: [f64; 4], skew: f64, width: usize, height: usize) -> [f64; 4] {
    let _ = (bbox, skew, width, height);
    todo!("spec §4.3")
}

//! The OpenCV image operations the reference pipeline uses, reimplemented to
//! match OpenCV 5.0's results (spec §13): `resize` with `INTER_LINEAR` and
//! `INTER_AREA`, `warpAffine` with `INTER_LINEAR`, `getRotationMatrix2D`,
//! numpy's `rot90`, the grayscale decode, percentiles, and 4-connected
//! components numbered in raster order.

use super::types::Gray;

/// How `resize` samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Area,
}

/// What `warp_affine` reads outside the source image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Border {
    /// A constant value (`BORDER_CONSTANT`).
    Constant(u8),
    /// The nearest edge pixel (`BORDER_REPLICATE`).
    Replicate,
}

/// `cv2.resize(src, (width, height), interpolation)`.
pub fn resize(src: &Gray, width: usize, height: usize, interpolation: Interpolation) -> Gray {
    let _ = (src, width, height, interpolation);
    todo!("spec §13")
}

/// `cv2.getRotationMatrix2D((cx, cy), degrees, 1.0)` as
/// `[a, b, c, d, e, f]` for `x' = a x + b y + c`, `y' = d x + e y + f`.
pub fn rotation_matrix(cx: f64, cy: f64, degrees: f64) -> [f64; 6] {
    let _ = (cx, cy, degrees);
    todo!("spec §4.1")
}

/// `cv2.invertAffineTransform`.
pub fn invert_affine(m: &[f64; 6]) -> [f64; 6] {
    let _ = m;
    todo!()
}

/// `cv2.warpAffine(src, m, (width, height), INTER_LINEAR, border)`.
pub fn warp_affine(src: &Gray, m: &[f64; 6], width: usize, height: usize, border: Border) -> Gray {
    let _ = (src, m, width, height, border);
    todo!("spec §13")
}

/// `numpy.rot90(src, k)`: k quarter turns counter-clockwise (negative k
/// turns clockwise).
pub fn rot90(src: &Gray, k: i32) -> Gray {
    let _ = (src, k);
    todo!()
}

/// OpenCV's grayscale decode of 8-bit RGB: `(R*4899 + G*9617 + B*1868) >> 14`
/// (spec §3.1). `rgb` is packed RGB, three bytes a pixel.
pub fn gray_from_rgb(rgb: &[u8], width: usize, height: usize) -> Gray {
    let _ = (rgb, width, height);
    todo!("spec §3.1")
}

/// numpy's default ("linear") percentile of the values, `q` in 0-100.
pub fn percentile(values: &[u8], q: f64) -> f64 {
    let _ = (values, q);
    todo!("spec §6.1")
}

/// One connected component's bounding box and area, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Component {
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize,
    pub area: usize,
}

/// `cv2.connectedComponentsWithStats(mask, connectivity=4)` without the
/// background: the label of every pixel (0 background, `i + 1` for the
/// i-th component) and the components, numbered in raster order of each
/// component's first pixel.
pub fn connected_components(mask: &[bool], width: usize, height: usize) -> (Vec<u32>, Vec<Component>) {
    let _ = (mask, width, height);
    todo!("spec §5.2")
}

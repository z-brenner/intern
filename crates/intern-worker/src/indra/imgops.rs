//! The OpenCV image operations the reference pipeline uses, reimplemented to
//! match OpenCV 5.0's results (spec §13): `resize` with `INTER_LINEAR` and
//! `INTER_AREA`, `warpAffine` with `INTER_LINEAR`, `getRotationMatrix2D`,
//! numpy's `rot90`, the grayscale decode, percentiles, and 4-connected
//! components numbered in raster order.
//!
//! Each operation follows the code path OpenCV 5.0.0 (the
//! `opencv-python-headless` 5.0.0.93 wheel the reference ran with) takes for
//! a one-channel 8-bit image, step for step and in the same arithmetic:
//! resizing in 11-bit fixed point or single-precision area weights, warping
//! in single precision with fused multiply-adds where OpenCV's AVX2 kernel
//! fuses them. Every operation is bit-exact against that wheel on the
//! goldens in `tests/indra_imgops.rs`; the notes on each function say which
//! OpenCV source each step comes from.

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

/// OpenCV's `INTER_RESIZE_COEF_BITS`: bilinear resize weights are integers
/// out of 2^11 (`imgproc/src/resize.cpp`).
const RESIZE_COEF_SCALE: f32 = 2048.0;

/// `cv2.resize(src, (width, height), interpolation)`.
///
/// The paths, as `cv::resize` chooses them:
/// - the same size is a copy;
/// - `INTER_LINEAR` halving both sides exactly is done as `INTER_AREA`;
/// - `INTER_AREA` shrinking both sides by whole factors averages blocks
///   (`resizeAreaFast_`), by other factors weighs the overlap of every source
///   pixel with the destination cell in single precision (`resizeArea_`);
/// - `INTER_AREA` enlarging either side is bilinear with the area-mode
///   coefficients, and `INTER_LINEAR` is bilinear with the half-pixel ones,
///   both in 11-bit fixed point (`resizeGeneric_` with `HResizeLinear` and
///   `VResizeLinear<uchar, int, short, FixedPtCast<..>>`).
///
/// # Panics
/// When the source or the destination is empty.
pub fn resize(src: &Gray, width: usize, height: usize, interpolation: Interpolation) -> Gray {
    assert!(
        src.width > 0 && src.height > 0 && width > 0 && height > 0,
        "resize of an empty image"
    );
    if width == src.width && height == src.height {
        return src.clone();
    }
    // cv::resize: inv_scale = dsize / ssize, scale = 1 / inv_scale, both f64;
    // the area tables and the "fast" test use scale, not ssize / dsize.
    let inv_x = width as f64 / src.width as f64;
    let inv_y = height as f64 / src.height as f64;
    let scale_x = 1.0 / inv_x;
    let scale_y = 1.0 / inv_y;
    let int_x = scale_x.round_ties_even();
    let int_y = scale_y.round_ties_even();
    let area_fast =
        (scale_x - int_x).abs() < f64::EPSILON && (scale_y - int_y).abs() < f64::EPSILON;
    let mut interpolation = interpolation;
    if interpolation == Interpolation::Linear && area_fast && int_x == 2.0 && int_y == 2.0 {
        interpolation = Interpolation::Area;
    }
    if interpolation == Interpolation::Area && scale_x >= 1.0 && scale_y >= 1.0 {
        return if area_fast {
            resize_area_fast(src, width, height, int_x as usize, int_y as usize)
        } else {
            resize_area(src, width, height, scale_x, scale_y)
        };
    }
    let area_mode = interpolation == Interpolation::Area;
    resize_linear(
        src,
        width,
        height,
        [scale_x, scale_y],
        [inv_x, inv_y],
        area_mode,
    )
}

/// One axis of a bilinear resize, as `cv::resize` computes it: for every
/// destination index, the first source index and the fixed-point weights
/// of it and the next. Half-pixel centres for `INTER_LINEAR`, cell edges
/// for the enlarging `INTER_AREA`, each fraction rounded to single
/// precision. The horizontal axis clamps its taps into the image and zeroes
/// the fraction there; the vertical axis keeps the fraction and lets the
/// row fetch clamp (`clamp` false), as OpenCV does.
struct LinearAxis {
    offsets: Vec<isize>,
    weights: Vec<[i32; 2]>,
    /// Destination indices from here on read only `offsets[i]` (the source's
    /// last pixel), at full weight (`xmax` in `cv::resize`).
    edge_from: usize,
}

impl LinearAxis {
    fn new(dst: usize, src: usize, scale: f64, inv: f64, area_mode: bool, clamp: bool) -> Self {
        let mut offsets = Vec::with_capacity(dst);
        let mut weights = Vec::with_capacity(dst);
        let mut edge_from = dst;
        for d in 0..dst {
            let (mut s, mut f) = if area_mode {
                let s = (d as f64 * scale).floor() as isize;
                let f = ((d + 1) as f64 - (s + 1) as f64 * inv) as f32;
                (s, if f <= 0.0 { 0.0 } else { f - f.floor() })
            } else {
                let f = ((d as f64 + 0.5) * scale - 0.5) as f32;
                let s = f.floor();
                (s as isize, f - s)
            };
            if clamp {
                if s < 0 {
                    s = 0;
                    f = 0.0;
                }
                if s + 1 >= src as isize {
                    edge_from = edge_from.min(d);
                    if s >= src as isize - 1 {
                        s = src as isize - 1;
                        f = 0.0;
                    }
                }
            }
            offsets.push(s);
            // saturate_cast<short>(cbuf[k] * INTER_RESIZE_COEF_SCALE), with
            // cbuf[0] = 1.f - fx: each weight rounded on its own.
            weights.push([
                ((1.0 - f) * RESIZE_COEF_SCALE).round_ties_even() as i32,
                (f * RESIZE_COEF_SCALE).round_ties_even() as i32,
            ]);
        }
        Self {
            offsets,
            weights,
            edge_from,
        }
    }
}

fn resize_linear(
    src: &Gray,
    width: usize,
    height: usize,
    scale: [f64; 2],
    inv: [f64; 2],
    area_mode: bool,
) -> Gray {
    let x = LinearAxis::new(width, src.width, scale[0], inv[0], area_mode, true);
    let y = LinearAxis::new(height, src.height, scale[1], inv[1], area_mode, false);
    let last_row = src.height as isize - 1;

    // Horizontal passes are kept for the two source rows the current
    // destination row reads; rows only move down, so each is computed once.
    let mut upper = vec![0i32; width];
    let mut lower = vec![0i32; width];
    let mut upper_row = usize::MAX;
    let mut lower_row = usize::MAX;
    let mut out = Vec::with_capacity(width * height);
    for (&sy, &[b0, b1]) in y.offsets.iter().zip(&y.weights) {
        let r0 = sy.clamp(0, last_row) as usize;
        let r1 = (sy + 1).clamp(0, last_row) as usize;
        if upper_row != r0 {
            if lower_row == r0 {
                std::mem::swap(&mut upper, &mut lower);
                std::mem::swap(&mut upper_row, &mut lower_row);
            } else {
                horizontal_linear(row(src, r0), &x, &mut upper);
                upper_row = r0;
            }
        }
        if r1 != r0 && lower_row != r1 {
            horizontal_linear(row(src, r1), &x, &mut lower);
            lower_row = r1;
        }
        let second = if r1 == r0 { &upper } else { &lower };
        // VResizeLinear<uchar, int, short, FixedPtCast<int, uchar, 22>>.
        // The sum is at most 1021 + 2 before the last shift, so it fits.
        out.extend(upper.iter().zip(second).map(|(&s0, &s1)| {
            let v = (((b0 * (s0 >> 4)) >> 16) + ((b1 * (s1 >> 4)) >> 16) + 2) >> 2;
            v as u8
        }));
    }
    Gray::new(width, height, out)
}

/// `HResizeLinear<uchar, int, short, INTER_RESIZE_COEF_SCALE>`.
fn horizontal_linear(src: &[u8], axis: &LinearAxis, out: &mut [i32]) {
    let edge = axis.edge_from;
    for ((o, &s), &[w0, w1]) in out[..edge].iter_mut().zip(&axis.offsets).zip(&axis.weights) {
        let s = s as usize;
        *o = i32::from(src[s]) * w0 + i32::from(src[s + 1]) * w1;
    }
    for (o, &s) in out[edge..].iter_mut().zip(&axis.offsets[edge..]) {
        *o = i32::from(src[s as usize]) * RESIZE_COEF_SCALE as i32;
    }
}

/// `resizeAreaFast_`: the mean of whole `kx` by `ky` blocks.
fn resize_area_fast(src: &Gray, width: usize, height: usize, kx: usize, ky: usize) -> Gray {
    let area = kx * ky;
    let inv_area = 1.0f32 / area as f32;
    let mut out = vec![0u8; width * height];
    for (dy, out_row) in out.chunks_exact_mut(width).enumerate() {
        let sy0 = dy * ky;
        if sy0 >= src.height {
            continue;
        }
        // Destination pixels whose block lies wholly inside the source.
        let whole = if sy0 + ky <= src.height {
            src.width / kx
        } else {
            0
        };
        for (dx, o) in out_row.iter_mut().enumerate().take(whole) {
            let mut sum = 0u32;
            for sy in sy0..sy0 + ky {
                let r = &row(src, sy)[dx * kx..dx * kx + kx];
                sum += r.iter().map(|&v| u32::from(v)).sum::<u32>();
            }
            *o = if kx == 2 && ky == 2 {
                // ResizeAreaFastVec's 2x2 kernel: (a + b + c + d + 2) >> 2.
                ((sum + 2) >> 2) as u8
            } else {
                saturate_u8(sum as f32 * inv_area)
            };
        }
        // Blocks cut by the right or bottom edge average what they hold.
        for (dx, o) in out_row.iter_mut().enumerate().skip(whole) {
            let (mut sum, mut count) = (0u32, 0u32);
            for sy in (sy0..sy0 + ky).take_while(|&sy| sy < src.height) {
                for &v in row(src, sy).iter().skip(dx * kx).take(kx) {
                    sum += u32::from(v);
                    count += 1;
                }
            }
            *o = if count == 0 {
                0
            } else {
                saturate_u8(sum as f32 / count as f32)
            };
        }
    }
    Gray::new(width, height, out)
}

/// One `DecimateAlpha` of `computeResizeAreaTab`: source index `s` adds
/// `alpha` of itself to destination index `d`.
#[derive(Clone, Copy)]
struct AreaTap {
    d: usize,
    s: usize,
    alpha: f32,
}

/// `computeResizeAreaTab`: the overlap of every source pixel with each
/// destination cell, over the cell's width, in double precision rounded
/// to single.
fn area_taps(src: usize, dst: usize, scale: f64) -> Vec<AreaTap> {
    let mut taps = Vec::with_capacity(src * 2);
    for d in 0..dst {
        let fs1 = d as f64 * scale;
        let fs2 = fs1 + scale;
        let cell = scale.min(src as f64 - fs1);
        let mut s2 = fs2.floor() as isize;
        s2 = s2.min(src as isize - 1);
        let s1 = (fs1.ceil() as isize).min(s2);
        if s1 as f64 - fs1 > 1e-3 {
            taps.push(AreaTap {
                d,
                s: (s1 - 1) as usize,
                alpha: ((s1 as f64 - fs1) / cell) as f32,
            });
        }
        for s in s1..s2 {
            taps.push(AreaTap {
                d,
                s: s as usize,
                alpha: (1.0 / cell) as f32,
            });
        }
        if fs2 - s2 as f64 > 1e-3 {
            taps.push(AreaTap {
                d,
                s: s2 as usize,
                alpha: ((fs2 - s2 as f64).min(1.0).min(cell) / cell) as f32,
            });
        }
    }
    taps
}

/// `ResizeArea_Invoker<uchar, float>`: every source row is weighed across
/// into a single-precision buffer, and the buffers are weighed down into
/// the destination row, which is rounded half to even.
fn resize_area(src: &Gray, width: usize, height: usize, scale_x: f64, scale_y: f64) -> Gray {
    let x_taps = area_taps(src.width, width, scale_x);
    let y_taps = area_taps(src.height, height, scale_y);
    let mut out = vec![0u8; width * height];
    let mut across = vec![0f32; width];
    let mut sum = vec![0f32; width];
    let mut current = y_taps.first().map_or(0, |t| t.d);
    for tap in &y_taps {
        across.fill(0.0);
        let s = row(src, tap.s);
        for x in &x_taps {
            across[x.d] += f32::from(s[x.s]) * x.alpha;
        }
        if tap.d != current {
            let o = &mut out[current * width..(current + 1) * width];
            for ((o, acc), &v) in o.iter_mut().zip(sum.iter_mut()).zip(&across) {
                *o = saturate_u8(*acc);
                *acc = tap.alpha * v;
            }
            current = tap.d;
        } else {
            for (acc, &v) in sum.iter_mut().zip(&across) {
                *acc += tap.alpha * v;
            }
        }
    }
    let o = &mut out[current * width..(current + 1) * width];
    for (o, &acc) in o.iter_mut().zip(&sum) {
        *o = saturate_u8(acc);
    }
    Gray::new(width, height, out)
}

/// `saturate_cast<uchar>(float)`: rounded half to even (`cvRound`), then
/// clamped (clamping first gives the same byte).
fn saturate_u8(v: f32) -> u8 {
    round_half_even(v.clamp(0.0, 255.0)) as u8
}

/// Half-to-even rounding of `0 <= v < 2^22` by the 2^23 trick: adding 2^23
/// leaves no fraction bits, so the addition itself rounds (to nearest,
/// ties to even). `f32::round_ties_even` is a library call on targets
/// without SSE4.1, and this runs for every pixel.
#[inline(always)]
fn round_half_even(v: f32) -> f32 {
    const TWO_23: f32 = 8_388_608.0;
    (v + TWO_23) - TWO_23
}

/// `cvFloor` of a float, without the library call `f32::floor` is on
/// targets without SSE4.1. Saturates far outside `i32`, where the warp
/// reads only the border anyway.
#[inline(always)]
fn floor_i64(v: f32) -> i64 {
    let t = v as i64;
    t - i64::from((t as f32) > v)
}

fn row(src: &Gray, y: usize) -> &[u8] {
    &src.data[y * src.width..(y + 1) * src.width]
}

/// `cv2.getRotationMatrix2D((cx, cy), degrees, 1.0)` as
/// `[a, b, c, d, e, f]` for `x' = a x + b y + c`, `y' = d x + e y + f`.
///
/// OpenCV takes the centre as a `Point2f`, so it is rounded to single
/// precision first (exact for the half pixels the pipeline passes).
pub fn rotation_matrix(cx: f64, cy: f64, degrees: f64) -> [f64; 6] {
    let cx = f64::from(cx as f32);
    let cy = f64::from(cy as f32);
    let angle = degrees * (std::f64::consts::PI / 180.0);
    // Not `f64::cos`/`sin`, which call the platform's C runtime: Windows'
    // gives sin(-45 degrees) one place off glibc's, which OpenCV was
    // measured with. libm's are the same everywhere.
    let alpha = libm::cos(angle);
    let beta = libm::sin(angle);
    [
        alpha,
        beta,
        (1.0 - alpha) * cx - beta * cy,
        -beta,
        alpha,
        beta * cx + (1.0 - alpha) * cy,
    ]
}

/// `cv2.invertAffineTransform` (and the inversion `warpAffine` does
/// itself): a singular matrix inverts to zeros, as in OpenCV.
pub fn invert_affine(m: &[f64; 6]) -> [f64; 6] {
    let det = m[0] * m[4] - m[1] * m[3];
    let d = if det != 0.0 { 1.0 / det } else { 0.0 };
    let a11 = m[4] * d;
    let a22 = m[0] * d;
    let a12 = -m[1] * d;
    let a21 = -m[3] * d;
    let b1 = -a11 * m[2] - a12 * m[5];
    let b2 = -a21 * m[2] - a22 * m[5];
    [a11, a12, b1, a21, a22, b2]
}

/// Destination pixels of a row that OpenCV's AVX2 `warpAffine` kernel maps
/// sixteen at a time (two vectors of eight floats); the rest of the row
/// goes through its scalar tail, which rounds the source coordinate
/// differently (see [`warp_affine_rows`]).
const WARP_VECTOR_BLOCK: usize = 16;

/// `cv2.warpAffine(src, m, (width, height), INTER_LINEAR, border)`.
pub fn warp_affine(src: &Gray, m: &[f64; 6], width: usize, height: usize, border: Border) -> Gray {
    let mut out = Vec::with_capacity(width * height);
    warp_affine_rows(src, m, width, height, border, |_, row| {
        out.extend_from_slice(row)
    });
    Gray::new(width, height, out)
}

/// [`warp_affine`] a row at a time: `each(y, row)` is called for every
/// destination row in order, so a caller that only needs, say, row sums
/// does not hold the warped image.
///
/// OpenCV 5's `warpAffineLinearInvoker_8UC1` (`imgproc/src/
/// warp_kernels.simd.hpp`), as dispatched on an AVX2 machine with FMA:
/// the inverse matrix is rounded to single precision; along a row, the
/// first `16 * floor(width / 16)` pixels map as
/// `sx = fma(M0, x, (y * M1) + M2)`, the rest as `sx = fma(x, M0, y * M1) +
/// M2` (the compiler fused the scalar tail's first product only); the four
/// neighbours are interpolated as `fma(fx, p01 - p00, p00)` across and
/// `fma(fy, bottom - top, top)` down, and rounded half to even. Bit-exact
/// against the reference wheel on every golden; another OpenCV build (no
/// AVX2, or AVX-512 with sixteen-float vectors) rounds a few pixels
/// differently.
pub fn warp_affine_rows(
    src: &Gray,
    m: &[f64; 6],
    width: usize,
    height: usize,
    border: Border,
    each: impl FnMut(usize, &[u8]),
) {
    #[cfg(target_arch = "x86_64")]
    let fused = fused::available();
    #[cfg(not(target_arch = "x86_64"))]
    let fused = false;
    warp_rows(src, m, width, height, border, fused, each);
}

/// [`warp_affine_rows`], sampling with the processor's fused multiply-add
/// when `fused` (which the caller has checked it has) and emulating it
/// otherwise.
fn warp_rows(
    src: &Gray,
    m: &[f64; 6],
    width: usize,
    height: usize,
    border: Border,
    fused: bool,
    mut each: impl FnMut(usize, &[u8]),
) {
    let inv = invert_affine(m);
    let mi: [f32; 6] = std::array::from_fn(|i| inv[i] as f32);
    let vector_end = width / WARP_VECTOR_BLOCK * WARP_VECTOR_BLOCK;
    let sampler = Sampler {
        data: &src.data,
        width: src.width as i64,
        height: src.height as i64,
        border,
    };
    let mut out = vec![0u8; width];
    let (mut xs, mut ys) = (vec![0f32; width], vec![0f32; width]);
    let mut redo = Vec::new();
    #[cfg(not(target_arch = "x86_64"))]
    let _ = fused;
    for y in 0..height {
        let yf = y as f32;
        let (y1, y4) = (yf * mi[1], yf * mi[4]);
        let (row_x, row_y) = (y1 + mi[2], y4 + mi[5]);
        fma_row(&mut xs[..vector_end], 0, mi[0], row_x);
        fma_row(&mut ys[..vector_end], 0, mi[3], row_y);
        fma_row(&mut xs[vector_end..], vector_end, mi[0], y1);
        fma_row(&mut ys[vector_end..], vector_end, mi[3], y4);
        for (sx, sy) in xs[vector_end..].iter_mut().zip(&mut ys[vector_end..]) {
            *sx += mi[2];
            *sy += mi[5];
        }
        #[cfg(target_arch = "x86_64")]
        if fused {
            // SAFETY: the caller checked that the processor has AVX2 and
            // FMA.
            unsafe { fused::sample_row(&sampler, &xs, &ys, &mut out) };
            each(y, &out);
            continue;
        }
        redo.clear();
        for (x, ((o, &sx), &sy)) in out.iter_mut().zip(&xs).zip(&ys).enumerate() {
            let (v, exact) = sampler.sample(sx, sy);
            *o = v;
            if !exact {
                redo.push(x);
            }
        }
        for &x in &redo {
            out[x] = sampler.sample_exactly(xs[x], ys[x]);
        }
        each(y, &out);
    }
}

/// The warp's sampling with the processor's own fused multiply-add, on an
/// x86-64 processor with AVX2 and FMA (nearly every one since 2013): the
/// same three single-precision `fma`s OpenCV's kernel computes
/// ([`warp_affine_rows`]), so the same bytes as the emulation
/// ([`Sampler::sample`], [`fma32_exact`]), eight pixels at a time. A
/// processor without them, or another architecture, emulates.
///
/// Measured on InternBench's pages, the skew search ([`super::page::estimate_skew`])
/// costs a third of what it does emulated (`docs/ocr.md`).
#[cfg(target_arch = "x86_64")]
mod fused {
    use std::arch::x86_64::*;

    use super::{Sampler, saturate_u8};

    /// Whether this processor has what [`sample_row`] needs.
    pub(super) fn available() -> bool {
        std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
    }

    /// One pixel, through [`Sampler::neighbours`] (the image's edges and
    /// border) and the processor's `fma`.
    #[target_feature(enable = "fma")]
    fn sample(sampler: &Sampler<'_>, sx: f32, sy: f32) -> u8 {
        let ([p00, p01, p10, p11], fx, fy) = sampler.neighbours(sx, sy);
        let top = fx.mul_add(p01 - p00, p00);
        let bottom = fx.mul_add(p11 - p10, p10);
        saturate_u8(fy.mul_add(bottom - top, top))
    }

    /// `out[i]` sampled at `(xs[i], ys[i])`.
    ///
    /// Eight pixels at a time when all eight read four neighbours inside
    /// the image: their left neighbours at least four pixels from the
    /// right edge and their top neighbours above the last row, so the two
    /// four-byte gathers at `iy * width + ix` and one row below stay inside
    /// the image's bytes. Any other pixel, and the row's last few, one at a
    /// time.
    ///
    /// # Safety
    /// The processor has AVX2 and FMA ([`available`]).
    #[target_feature(enable = "avx2,fma")]
    pub(super) unsafe fn sample_row(sampler: &Sampler<'_>, xs: &[f32], ys: &[f32], out: &mut [u8]) {
        let n = out.len().min(xs.len()).min(ys.len());
        let (width, height) = (sampler.width, sampler.height);
        let mut x = 0;
        // Byte offsets into the image are gathered as 32-bit integers.
        let vectors = width >= 4
            && height >= 2
            && sampler.data.len() as i64 == width * height
            && width * height < i64::from(i32::MAX);
        if vectors {
            let row = _mm256_set1_epi32(width as i32);
            let x_end = _mm256_set1_epi32((width - 3) as i32);
            let y_end = _mm256_set1_epi32((height - 1) as i32);
            let before = _mm256_set1_epi32(-1);
            let byte = _mm256_set1_epi32(0xff);
            let (black, white) = (_mm256_set1_ps(0.0), _mm256_set1_ps(255.0));
            let base = sampler.data.as_ptr().cast::<i32>();
            while x + 8 <= n {
                // SAFETY: x + 8 <= n <= xs.len(), ys.len().
                let (sx, sy) = unsafe {
                    (
                        _mm256_loadu_ps(xs.as_ptr().add(x)),
                        _mm256_loadu_ps(ys.as_ptr().add(x)),
                    )
                };
                let (fx0, fy0) = (_mm256_floor_ps(sx), _mm256_floor_ps(sy));
                // A coordinate past i32 (or NaN) converts to i32::MIN and
                // fails the test below.
                let (ix, iy) = (_mm256_cvttps_epi32(fx0), _mm256_cvttps_epi32(fy0));
                let inside = _mm256_and_si256(
                    _mm256_and_si256(
                        _mm256_cmpgt_epi32(ix, before),
                        _mm256_cmpgt_epi32(x_end, ix),
                    ),
                    _mm256_and_si256(
                        _mm256_cmpgt_epi32(iy, before),
                        _mm256_cmpgt_epi32(y_end, iy),
                    ),
                );
                if _mm256_movemask_epi8(inside) != -1 {
                    for i in x..x + 8 {
                        out[i] = sample(sampler, xs[i], ys[i]);
                    }
                    x += 8;
                    continue;
                }
                // 0 <= ix <= width - 4 and 0 <= iy <= height - 2: both
                // gathers read four bytes inside the image's
                // width * height, which fits an i32.
                let at = _mm256_add_epi32(_mm256_mullo_epi32(iy, row), ix);
                // SAFETY: every lane's four bytes at `at` and `at + width`
                // are inside `sampler.data` (above).
                let (top_bytes, bottom_bytes) = unsafe {
                    (
                        _mm256_i32gather_epi32::<1>(base, at),
                        _mm256_i32gather_epi32::<1>(base, _mm256_add_epi32(at, row)),
                    )
                };
                let first = |bytes| _mm256_cvtepi32_ps(_mm256_and_si256(bytes, byte));
                let second = |bytes| {
                    _mm256_cvtepi32_ps(_mm256_and_si256(_mm256_srli_epi32::<8>(bytes), byte))
                };
                let (p00, p01) = (first(top_bytes), second(top_bytes));
                let (p10, p11) = (first(bottom_bytes), second(bottom_bytes));
                let (fx, fy) = (_mm256_sub_ps(sx, fx0), _mm256_sub_ps(sy, fy0));
                let top = _mm256_fmadd_ps(fx, _mm256_sub_ps(p01, p00), p00);
                let bottom = _mm256_fmadd_ps(fx, _mm256_sub_ps(p11, p10), p10);
                let value = _mm256_fmadd_ps(fy, _mm256_sub_ps(bottom, top), top);
                // Clamped, then rounded half to even (the default rounding
                // mode), as saturate_u8 rounds.
                let value = _mm256_min_ps(_mm256_max_ps(value, black), white);
                let mut bytes = [0i32; 8];
                // SAFETY: `bytes` holds eight i32s.
                unsafe {
                    _mm256_storeu_si256(
                        bytes.as_mut_ptr().cast::<__m256i>(),
                        _mm256_cvtps_epi32(value),
                    )
                };
                for (o, v) in out[x..x + 8].iter_mut().zip(bytes) {
                    *o = v as u8;
                }
                x += 8;
            }
        }
        for i in x..n {
            out[i] = sample(sampler, xs[i], ys[i]);
        }
    }
}

/// The double sums whose rounding to single precision may not be the
/// fused result (see [`fma32`]): exactly halfway between two singles, or
/// below single precision's normal range.
#[inline(always)]
fn needs_exact_fma(s: f64) -> bool {
    // The 29 low mantissa bits of a double are below a single's last.
    const BELOW_SINGLE: u64 = (1 << 29) - 1;
    const MIDPOINT: u64 = 1 << 28;
    // An exact zero is exact; a nonzero sum this small cannot be from
    // anything but products of tiny singles.
    // `|` and `&`, not `||` and `&&`: no branches in the pixel loop.
    (s.to_bits() & BELOW_SINGLE == MIDPOINT)
        | ((s != 0.0) & (s.abs() < 2.0 * f64::from(f32::MIN_POSITIVE)))
}

/// `out[i] = fma(a, x0 + i, c)` for a row of coordinates. When every
/// `a x + c` along the row is exact in double precision (nearly always:
/// see [`line_is_exact`]), it is also reached exactly by adding `a` from
/// `a x0 + c`, and its one rounding to single precision is the fused
/// result; otherwise each is emulated.
fn fma_row(out: &mut [f32], x0: usize, a: f32, c: f32) {
    if line_is_exact(a, c, x0 + out.len()) {
        let a64 = f64::from(a);
        let mut s = a64 * x0 as f64 + f64::from(c);
        for o in out.iter_mut() {
            *o = s as f32;
            s += a64;
        }
    } else {
        for (x, o) in (x0..).zip(out.iter_mut()) {
            *o = fma32(a, x as f32, c);
        }
    }
}

/// Whether `a x + c` is exactly representable in double precision for
/// every integer `0 <= x < n`: the bits from the lowest set bit of `a` or
/// `c` up to the largest magnitude reached span at most 53.
fn line_is_exact(a: f32, c: f32, n: usize) -> bool {
    /// The exponent of the lowest set bit of a nonzero single.
    fn lowest_bit(v: f32) -> Option<i32> {
        let bits = v.to_bits() & 0x7fff_ffff;
        if bits == 0 {
            return None;
        }
        let exponent = (bits >> 23) as i32;
        let (mantissa, e) = if exponent == 0 {
            (bits, -149)
        } else {
            (bits & 0x7f_ffff | 0x80_0000, exponent - 150)
        };
        Some(e + mantissa.trailing_zeros() as i32)
    }
    let low = match (lowest_bit(a), lowest_bit(c)) {
        (None, None) => return true,
        (a, c) => a.unwrap_or(i32::MAX).min(c.unwrap_or(i32::MAX)),
    };
    let bound = f64::from(a).abs() * n as f64 + f64::from(c).abs();
    // bound < 2^(high + 1); every value is a multiple of 2^low below it.
    let high = ((bound.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    bound.is_finite() && high + 1 - low <= 53
}

/// Bilinear sampling of an image at single-precision source coordinates,
/// as OpenCV's warp kernel samples (see [`warp_affine_rows`]).
///
/// Inside the image the four neighbours are read directly; at and beyond
/// its edges, through clamped indices and, for a constant border, replaced
/// by the value where they fall outside. A flat neighbourhood is its
/// value; otherwise the three fused multiply-adds are formed in double
/// precision and flagged when rounding to single might not be the fused
/// result ([`needs_exact_fma`]), and only the flagged pixels are sampled again
/// with the exact emulation.
struct Sampler<'a> {
    data: &'a [u8],
    width: i64,
    height: i64,
    border: Border,
}

impl Sampler<'_> {
    /// The four neighbours of `(ix, iy)` and the coordinate fractions.
    #[inline(always)]
    fn neighbours(&self, sx: f32, sy: f32) -> ([f32; 4], f32, f32) {
        let (ix, iy) = (floor_i64(sx), floor_i64(sy));
        let (sw, sh) = (self.width, self.height);
        let (fx, fy) = (sx - ix as f32, sy - iy as f32);
        // Inside the image (a contiguous stretch of each row, so this
        // branch is predictable): the four pixels directly.
        if ix >= 0 && iy >= 0 && ix < sw - 1 && iy < sh - 1 {
            let at = (iy * sw + ix) as usize;
            let below = at + sw as usize;
            let p = [
                self.data[at],
                self.data[at + 1],
                self.data[below],
                self.data[below + 1],
            ];
            return (p.map(f32::from), fx, fy);
        }
        let (x0, x1) = (ix.clamp(0, sw - 1), ix.saturating_add(1).clamp(0, sw - 1));
        let (y0, y1) = (
            iy.clamp(0, sh - 1) * sw,
            iy.saturating_add(1).clamp(0, sh - 1) * sw,
        );
        let mut p = [y0 + x0, y0 + x1, y1 + x0, y1 + x1].map(|i| self.data[i as usize]);
        if let Border::Constant(value) = self.border {
            let in_x = [ix >= 0 && ix < sw, ix >= -1 && ix < sw - 1];
            let in_y = [iy >= 0 && iy < sh, iy >= -1 && iy < sh - 1];
            let inside = [
                in_y[0] & in_x[0],
                in_y[0] & in_x[1],
                in_y[1] & in_x[0],
                in_y[1] & in_x[1],
            ];
            for (v, inside) in p.iter_mut().zip(inside) {
                *v = if inside { *v } else { value };
            }
        }
        (p.map(f32::from), fx, fy)
    }

    /// The sampled value, and false when it must be redone exactly.
    #[inline(always)]
    fn sample(&self, sx: f32, sy: f32) -> (u8, bool) {
        let ([p00, p01, p10, p11], fx, fy) = self.neighbours(sx, sy);
        if p00 == p01 && p00 == p10 && p00 == p11 {
            // fma(f, 0, p) is p. Most of a page is flat paper (or, as ink,
            // flat zero); this branch costs less than the arithmetic.
            return (p00 as u8, true);
        }
        let fx64 = f64::from(fx);
        let top = fx64 * f64::from(p01 - p00) + f64::from(p00);
        let bottom = fx64 * f64::from(p11 - p10) + f64::from(p10);
        let (top32, bottom32) = (top as f32, bottom as f32);
        let v = f64::from(fy) * f64::from(bottom32 - top32) + f64::from(top32);
        let exact = !(needs_exact_fma(top) | needs_exact_fma(bottom) | needs_exact_fma(v));
        (saturate_u8(v as f32), exact)
    }

    /// [`Sampler::sample`] with every fused multiply-add emulated exactly.
    fn sample_exactly(&self, sx: f32, sy: f32) -> u8 {
        let ([p00, p01, p10, p11], fx, fy) = self.neighbours(sx, sy);
        let top = fma32_exact(fx, p01 - p00, p00);
        let bottom = fma32_exact(fx, p11 - p10, p10);
        saturate_u8(fma32_exact(fy, bottom - top, top))
    }
}

/// `a * b + c` rounded once to single precision, as the FMA instruction
/// OpenCV's kernel uses. A build for a target with FMA uses the
/// instruction. Otherwise the product is formed exactly in double
/// precision (24 + 24 bits) and the sum rounded to double: rounding that
/// to single gives the fused result unless the double sum landed exactly
/// on a midpoint between two singles (rounding is monotonic, and the
/// midpoints are doubles), or is too small for single precision's full 24
/// bits. Those rare sums are redone exactly ([`fma32_exact`]).
#[inline(always)]
pub fn fma32(a: f32, b: f32, c: f32) -> f32 {
    #[cfg(target_feature = "fma")]
    {
        a.mul_add(b, c)
    }
    #[cfg(not(target_feature = "fma"))]
    {
        let s = f64::from(a) * f64::from(b) + f64::from(c);
        if needs_exact_fma(s) {
            fma32_exact(a, b, c)
        } else {
            s as f32
        }
    }
}

/// [`fma32`] without the shortcut: the double sum rounded to odd (TwoSum
/// finds whether it is inexact, and an inexact even result moves one step
/// towards the exact value), which rounds correctly to single precision
/// (53 >= 24 + 2 bits; Boldo and Melquiond, "Emulation of FMA and
/// correctly rounded sums", 2008).
pub fn fma32_exact(a: f32, b: f32, c: f32) -> f32 {
    let p = f64::from(a) * f64::from(b);
    let c = f64::from(c);
    let s = p + c;
    let bp = s - c;
    let err = (p - bp) + (c - (s - bp));
    let bits = s.to_bits();
    if err != 0.0 && bits & 1 == 0 {
        let away = (err > 0.0) == (s > 0.0);
        let bits = if away { bits + 1 } else { bits - 1 };
        return f64::from_bits(bits) as f32;
    }
    s as f32
}

/// `numpy.rot90(src, k)`: k quarter turns counter-clockwise (negative k
/// turns clockwise).
pub fn rot90(src: &Gray, k: i32) -> Gray {
    let (w, h) = (src.width, src.height);
    match k.rem_euclid(4) {
        0 => src.clone(),
        // out[i][j] = src[j][w - 1 - i]
        1 => {
            let mut out = Vec::with_capacity(w * h);
            for i in 0..w {
                out.extend((0..h).map(|j| src.data[j * w + (w - 1 - i)]));
            }
            Gray::new(h, w, out)
        }
        2 => {
            let mut out = src.data.clone();
            out.reverse();
            Gray::new(w, h, out)
        }
        // out[i][j] = src[h - 1 - j][i]
        _ => {
            let mut out = Vec::with_capacity(w * h);
            for i in 0..w {
                out.extend((0..h).map(|j| src.data[(h - 1 - j) * w + i]));
            }
            Gray::new(h, w, out)
        }
    }
}

/// OpenCV's grayscale decode of 8-bit RGB (spec §3.1): what
/// `cv2.imread(path, IMREAD_GRAYSCALE)` gives for a colour PNG. `rgb` is
/// packed RGB, three bytes a pixel.
///
/// OpenCV's PNG decoder asks libpng for the conversion
/// (`png_set_rgb_to_gray(png, 1, 0.299, 0.587)`), and libpng truncates the
/// coefficients to 15 bits (9797 and 19234, blue the rest of 32768, 3737)
/// and the result too: `(9797 R + 19234 G + 3737 B) >> 15`
/// (`png_set_rgb_to_gray_fixed`, `png_do_rgb_to_gray`). Spec §3.1's
/// `(4899 R + 9617 G + 1868 B) >> 14` is one off on a few pixels (2 of
/// `table_pmc.png`'s 1.05 million, 0.3% of random colours); this is exact on
/// every reference page and the random goldens. JPEG is different:
/// `IMREAD_GRAYSCALE` has libjpeg decode the luma plane directly, which
/// no formula over the decoded RGB reproduces.
///
/// # Panics
/// When `rgb` does not hold `width * height` pixels.
pub fn gray_from_rgb(rgb: &[u8], width: usize, height: usize) -> Gray {
    assert_eq!(rgb.len(), width * height * 3, "rgb image size");
    let data = rgb
        .chunks_exact(3)
        .map(|p| {
            ((u32::from(p[0]) * 9797 + u32::from(p[1]) * 19234 + u32::from(p[2]) * 3737) >> 15)
                as u8
        })
        .collect();
    Gray::new(width, height, data)
}

/// numpy's default ("linear") percentile of the values, `q` in 0-100.
///
/// Computed as the reference's `rec_preproc._percentiles` computes it for
/// 8-bit pixels: from the histogram, at virtual index `q / 100 * (n - 1)`,
/// interpolated with numpy's `_lerp` (from the nearer end). NaN for no
/// values (the reference never asks: `is_dark_line` returns early).
pub fn percentile(values: &[u8], q: f64) -> f64 {
    percentiles(values, &[q])[0]
}

/// [`percentile`] at several `q`, from one histogram.
pub fn percentiles(values: &[u8], qs: &[f64]) -> Vec<f64> {
    let mut histogram = [0usize; 256];
    for &v in values {
        histogram[usize::from(v)] += 1;
    }
    percentiles_of_histogram(&histogram, qs)
}

/// [`percentiles`] of values already counted into a histogram.
pub fn percentiles_of_histogram(histogram: &[usize; 256], qs: &[f64]) -> Vec<f64> {
    let n: usize = histogram.iter().sum();
    if n == 0 {
        return vec![f64::NAN; qs.len()];
    }
    // The value of the i-th smallest element: searchsorted(cumsum, i, "right").
    let order_statistic = |i: usize| -> f64 {
        let mut seen = 0;
        for (value, &count) in histogram.iter().enumerate() {
            seen += count;
            if seen > i {
                return value as f64;
            }
        }
        255.0
    };
    qs.iter()
        .map(|&q| {
            let x = q / 100.0 * (n - 1) as f64;
            let i = x.floor();
            let g = x - i;
            let lo = order_statistic(i as usize);
            let hi = order_statistic((i as usize + 1).min(n - 1));
            let d = hi - lo;
            if g >= 0.5 {
                hi - d * (1.0 - g)
            } else {
                lo + d * g
            }
        })
        .collect()
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
///
/// OpenCV's labelling (Wu's union-find for 4-connectivity) numbers the
/// components in raster order of their first pixels (spec §13,
/// measured); so does this two-pass union-find, by relabelling the roots
/// in the order the second pass meets them.
///
/// # Panics
/// When `mask` does not hold `width * height` pixels.
pub fn connected_components(
    mask: &[bool],
    width: usize,
    height: usize,
) -> (Vec<u32>, Vec<Component>) {
    assert_eq!(mask.len(), width * height, "mask size");
    let mut labels = vec![0u32; mask.len()];
    // parent[0] is the background; provisional labels start at 1.
    let mut parent: Vec<u32> = vec![0];
    fn find(parent: &mut [u32], mut a: u32) -> u32 {
        while parent[a as usize] != a {
            let up = parent[parent[a as usize] as usize];
            parent[a as usize] = up;
            a = up;
        }
        a
    }
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if !mask[i] {
                continue;
            }
            let up = if y > 0 { labels[i - width] } else { 0 };
            let left = if x > 0 { labels[i - 1] } else { 0 };
            labels[i] = match (up, left) {
                (0, 0) => {
                    let l = parent.len() as u32;
                    parent.push(l);
                    l
                }
                (l, 0) | (0, l) => l,
                (a, b) => {
                    let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
                    let (lo, hi) = (ra.min(rb), ra.max(rb));
                    parent[hi as usize] = lo;
                    lo
                }
            };
        }
    }
    let mut final_label = vec![0u32; parent.len()];
    let mut components: Vec<Component> = Vec::new();
    let mut right = Vec::new();
    let mut bottom = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            if labels[i] == 0 {
                continue;
            }
            let root = find(&mut parent, labels[i]) as usize;
            if final_label[root] == 0 {
                components.push(Component {
                    left: x,
                    top: y,
                    width: 0,
                    height: 0,
                    area: 0,
                });
                right.push(x);
                bottom.push(y);
                final_label[root] = components.len() as u32;
            }
            let l = final_label[root];
            labels[i] = l;
            let k = l as usize - 1;
            let c = &mut components[k];
            c.left = c.left.min(x);
            c.area += 1;
            right[k] = right[k].max(x);
            bottom[k] = bottom[k].max(y);
        }
    }
    for ((c, r), b) in components.iter_mut().zip(right).zip(bottom) {
        c.width = r + 1 - c.left;
        c.height = b + 1 - c.top;
    }
    (labels, components)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray(width: usize, height: usize, data: &[u8]) -> Gray {
        Gray::new(width, height, data.to_vec())
    }

    #[test]
    fn rot90_turns_like_numpy() {
        // [[1, 2, 3], [4, 5, 6]]
        let g = gray(3, 2, &[1, 2, 3, 4, 5, 6]);
        // np.rot90(a, 1) == [[3, 6], [2, 5], [1, 4]]
        assert_eq!(rot90(&g, 1), gray(2, 3, &[3, 6, 2, 5, 1, 4]));
        // np.rot90(a, 2) == [[6, 5, 4], [3, 2, 1]]
        assert_eq!(rot90(&g, 2), gray(3, 2, &[6, 5, 4, 3, 2, 1]));
        // np.rot90(a, -1) == np.rot90(a, 3) == [[4, 1], [5, 2], [6, 3]]
        assert_eq!(rot90(&g, -1), gray(2, 3, &[4, 1, 5, 2, 6, 3]));
        assert_eq!(rot90(&g, 3), rot90(&g, -1));
        assert_eq!(rot90(&g, -2), rot90(&g, 2));
        assert_eq!(rot90(&g, 4), g);
        assert_eq!(rot90(&rot90(&g, 1), -1), g);
    }

    #[test]
    fn gray_decode_truncates() {
        // The coefficients sum to 32768: white stays white.
        let g = gray_from_rgb(&[255, 255, 255, 0, 0, 0, 10, 20, 30, 0, 21, 155], 2, 2);
        // (10*9797 + 20*19234 + 30*3737) >> 15 = 594760 >> 15 = 18;
        // (21*19234 + 155*3737) >> 15 = 983149 >> 15 = 30 (29 by the 14-bit
        // formula).
        assert_eq!(g.data, vec![255, 0, 18, 30]);
    }

    #[test]
    fn percentile_interpolates_like_numpy() {
        let v = [10u8, 0, 30, 20];
        // np.percentile([0, 10, 20, 30], q)
        assert_eq!(percentile(&v, 0.0), 0.0);
        assert_eq!(percentile(&v, 100.0), 30.0);
        assert_eq!(percentile(&v, 50.0), 15.0);
        assert_eq!(percentile(&[7], 37.0), 7.0);
        assert!(percentile(&[], 50.0).is_nan());
    }

    #[test]
    fn components_are_numbered_in_raster_order() {
        // A U shape whose arms meet below: one component whose first pixel
        // is the left arm's top, then a dot that starts later.
        #[rustfmt::skip]
        let mask = [
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 1,
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 0,
        ].map(|v| v == 1);
        let (labels, components) = connected_components(&mask, 5, 4);
        #[rustfmt::skip]
        assert_eq!(labels, vec![
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 1,
            0, 1, 1, 1, 0,
            2, 0, 0, 0, 0,
        ]);
        assert_eq!(
            components,
            vec![
                Component {
                    left: 1,
                    top: 0,
                    width: 4,
                    height: 3,
                    area: 8
                },
                Component {
                    left: 0,
                    top: 3,
                    width: 1,
                    height: 1,
                    area: 1
                },
            ]
        );
        // Diagonal neighbours are not connected.
        let (_, components) = connected_components(&[true, false, false, true], 2, 2);
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn components_merge_later_labels_into_earlier() {
        // Three arms joined only on the last row: the merged component keeps
        // the first arm's number even though the middle arm started it.
        #[rustfmt::skip]
        let mask = [
            1, 0, 0, 0, 1,
            1, 0, 1, 0, 1,
            1, 1, 1, 1, 1,
        ].map(|v| v == 1);
        let (labels, components) = connected_components(&mask, 5, 3);
        assert!(labels.iter().zip(&mask).all(|(&l, &m)| (l == 1) == m));
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].area, 10);
    }

    #[test]
    fn rotation_matrix_matches_the_opencv_formula() {
        let m = rotation_matrix(50.0, 36.0, 90.0);
        assert!((m[0]).abs() < 1e-15 && (m[1] - 1.0).abs() < 1e-15);
        // getRotationMatrix2D((50, 36), 90, 1) maps the centre to itself.
        let (x, y) = (
            m[0] * 50.0 + m[1] * 36.0 + m[2],
            m[3] * 50.0 + m[4] * 36.0 + m[5],
        );
        assert!((x - 50.0).abs() < 1e-12 && (y - 36.0).abs() < 1e-12);
        let i = invert_affine(&m);
        let back = invert_affine(&i);
        for (a, b) in back.iter().zip(&m) {
            assert!((a - b).abs() < 1e-12);
        }
        assert!(invert_affine(&[0.0; 6]).iter().all(|&v| v == 0.0));
    }

    /// The processor's fused multiply-add samples every pixel as the
    /// emulation does: at every angle and scale, inside the image, at its
    /// edges and past them, with either border.
    #[test]
    fn fused_sampling_matches_the_emulation() {
        #[cfg(target_arch = "x86_64")]
        if fused::available() {
            // A deterministic, noisy page with flat stretches.
            let mut state = 0x2545_f491_u32;
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state
            };
            let (w, h) = (157, 93);
            let data = (0..w * h)
                .map(|_| match next() % 4 {
                    0 => 255,
                    1 => 0,
                    _ => (next() % 256) as u8,
                })
                .collect();
            let src = Gray::new(w, h, data);
            let mut checked = 0;
            for angle in [
                -45.0, -12.3, -2.0, -0.7, 0.0, 0.1, 0.35, 3.0, 29.9, 90.0, 133.0,
            ] {
                for (cx, cy) in [(78.5, 46.5), (0.0, 0.0), (300.25, -20.0)] {
                    for border in [
                        Border::Constant(0),
                        Border::Constant(255),
                        Border::Replicate,
                    ] {
                        for (dw, dh) in [(w, h), (200, 61), (7, 3)] {
                            let mut m = rotation_matrix(cx, cy, angle);
                            m[0] *= 1.13;
                            m[4] *= 0.91;
                            let mut emulated = Vec::new();
                            warp_rows(&src, &m, dw, dh, border, false, |_, row| {
                                emulated.extend_from_slice(row)
                            });
                            let mut fused = Vec::new();
                            warp_rows(&src, &m, dw, dh, border, true, |_, row| {
                                fused.extend_from_slice(row)
                            });
                            assert_eq!(fused, emulated, "{angle} ({cx}, {cy}) {border:?}");
                            checked += 1;
                        }
                    }
                }
            }
            assert_eq!(checked, 297);
        }
    }

    #[test]
    fn fma32_rounds_once() {
        let x = 1.0f32 + f32::EPSILON; // 1 + 2^-23
        // (1 + 2^-23)^2 = 1 + 2^-22 + 2^-46: only the tail survives.
        assert_eq!(fma32(x, x, -(1.0 + 2.0 * f32::EPSILON)), 2f32.powi(-46));
        // p * p = 1 + 2^-11 + 2^-24 is a tie between two floats. A tiny c
        // breaks it, but is lost if the sum is first rounded to double
        // precision, which would then round the tie to even (down).
        let p = 1.0f32 + 2f32.powi(-12);
        let c = 2f32.powi(-60);
        assert_eq!(fma32(p, p, c), 1.0 + 2f32.powi(-11) + 2f32.powi(-23));
        assert_eq!(fma32(p, p, -c), 1.0 + 2f32.powi(-11));
        assert_eq!(fma32(p, p, 0.0), 1.0 + 2f32.powi(-11));
        let q = -p;
        assert_eq!(fma32(q, p, -c), -(1.0 + 2f32.powi(-11) + 2f32.powi(-23)));
        // The shortcut and the exact path agree everywhere they both apply.
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            f32::from_bits((state >> 32) as u32 & 0x3fff_ffff ^ 0x0800_0000)
        };
        for _ in 0..100_000 {
            let (a, b, c) = (next(), next(), next());
            assert_eq!(fma32(a, b, c), fma32_exact(a, b, c), "{a} {b} {c}");
            assert_eq!(fma32(a, b, -c), fma32_exact(a, b, -c), "{a} {b} -{c}");
        }
    }

    #[test]
    fn resize_to_the_same_size_copies() {
        let g = gray(3, 2, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(resize(&g, 3, 2, Interpolation::Linear), g);
        assert_eq!(resize(&g, 3, 2, Interpolation::Area), g);
    }

    #[test]
    fn halving_is_block_averaging() {
        // cv2.resize(a, (1, 1), INTER_LINEAR) of a 2x2 block is the 2x2 area
        // mean (a + b + c + d + 2) >> 2: 1 + 2 + 3 + 5 = 11 -> 3.
        let g = gray(2, 2, &[1, 2, 3, 5]);
        assert_eq!(resize(&g, 1, 1, Interpolation::Linear).data, vec![3]);
        assert_eq!(resize(&g, 1, 1, Interpolation::Area).data, vec![3]);
    }
}

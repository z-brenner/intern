//! A page's skew and quarter turn, found before detection, and the mapping
//! of boxes back from the levelled page (spec §4).
//!
//! The order of a page's preparation, as `OrtOCR.__call__` and `_turn` run
//! it (`pipeline.py:691-731`): the page is shrunk once ([`preview`]); its
//! rows' tilt is measured on that ([`estimate_skew`]); the orientation
//! network reads the levelled preview ([`orient_input`],
//! [`orientation_class`]); and [`turn`] gives the full page its quarter
//! turn and levels it, measuring the tilt again when rows and columns
//! swapped.

use super::imgops::{self, Border, Interpolation};
use super::types::Gray;

/// Tilts smaller than this many degrees are left alone.
pub const DESKEW_MIN: f64 = 0.3;
/// The orientation classifier's probability above which its turn is taken.
pub const ORIENT_MIN: f32 = 0.95;
/// The orientation class of a page set in vertical columns.
pub const VERTICAL: usize = 4;
/// Degrees of tilt the skew search covers either way (`pipeline.py:98`);
/// quarter turns cover the rest.
pub const DESKEW_MAX: f64 = 45.0;
/// The preview's longest side (`PREVIEW_SIDE`, `pipeline.py:149`).
pub const PREVIEW_SIDE: usize = 1600;
/// The orientation classifier's square input side (`ORIENT_SIDE`,
/// `pipeline.py:55`).
pub const ORIENT_SIDE: usize = 768;
/// The skew search's thumbnail side (`estimate_skew(side=800)`); the
/// coarse pass reads one of half that.
pub const SKEW_SIDE: usize = 800;

/// The size `(width, height)` a page scaled to `side` on its longer side
/// gets: `(max(1, round(w * s)), max(1, round(h * s)))` with Python's
/// half-to-even `round`.
pub fn scaled_size(width: usize, height: usize, side: usize) -> (usize, usize) {
    let s = side as f64 / width.max(height) as f64;
    let scaled = |n: usize| ((n as f64 * s).round_ties_even() as usize).max(1);
    (scaled(width), scaled(height))
}

/// The page shrunk so its longer side is at most 1600 pixels (`preview`).
pub fn preview(gray: &Gray) -> Gray {
    let s = PREVIEW_SIDE as f64 / gray.width.max(gray.height) as f64;
    if s >= 1.0 {
        return gray.clone();
    }
    let (w, h) = scaled_size(gray.width, gray.height, PREVIEW_SIDE);
    imgops::resize(gray, w, h, Interpolation::Area)
}

/// `_ink_thumb`: the page area-resized to `side` on its longer side (also
/// when that enlarges it), as ink (255 - gray).
pub fn ink_thumb(gray: &Gray, side: usize) -> Gray {
    let (w, h) = scaled_size(gray.width, gray.height, side);
    let mut thumb = imgops::resize(gray, w, h, Interpolation::Area);
    for v in &mut thumb.data {
        *v = 255 - *v;
    }
    thumb
}

/// `_row_sharpness(ink, a)`: the sum of squared differences between the
/// ink totals of neighbouring rows once the thumbnail is turned `a`
/// degrees about its centre (black outside). The reference sums in
/// double precision; every term is an integer and the total stays below
/// 2^53 for an 800-pixel thumbnail (at most 799 * (800 * 255)^2), so the
/// integer sum here is the same number.
pub fn row_sharpness(ink: &Gray, degrees: f64) -> u64 {
    let m = imgops::rotation_matrix(ink.width as f64 / 2.0, ink.height as f64 / 2.0, degrees);
    let mut previous: Option<i64> = None;
    let mut score = 0u64;
    imgops::warp_affine_rows(
        ink,
        &m,
        ink.width,
        ink.height,
        Border::Constant(0),
        |_, row| {
            let total: i64 = row.iter().map(|&v| i64::from(v)).sum();
            if let Some(p) = previous {
                score += (total - p).unsigned_abs().pow(2);
            }
            previous = Some(total);
        },
    );
    score
}

/// `numpy.arange(start, stop, step)` for floats: `ceil((stop - start) /
/// step)` values, the first two `start` and `start + step`, the rest
/// `start + i * delta` where `delta` is the difference of the first two
/// (`DOUBLE_fill`), which is where values like -29.999999999999993 come
/// from.
pub fn arange(start: f64, stop: f64, step: f64) -> Vec<f64> {
    let len = ((stop - start) / step).ceil();
    if len.is_nan() || len <= 0.0 {
        return Vec::new();
    }
    let len = len as usize;
    let next = start + step;
    let delta = next - start;
    (0..len)
        .map(|i| match i {
            0 => start,
            1 => next,
            _ => start + i as f64 * delta,
        })
        .collect()
}

/// The order CPython iterates the set `{0.0, a, b}` in, for the
/// integer-valued angles the skew search starts from: each goes to slot
/// `hash & 7` of an eight-slot table, probing `i = 5 i + 1 + perturb`
/// (`perturb` the hash shifted right 5 more bits each time) on a collision
/// (`Objects/setobject.c`, `set_add_entry`). The order only matters when
/// two angles score exactly alike.
fn python_set_order(values: &[f64]) -> Vec<f64> {
    let mut table: [Option<f64>; 8] = [None; 8];
    for &v in values {
        if table.contains(&Some(v)) {
            continue;
        }
        // hash(float) of an integral value is hash(int): the value itself,
        // except that -1 hashes to -2.
        let hash = match v as i64 {
            -1 => -2,
            h => h,
        };
        let mut i = (hash as u64) & 7;
        let mut perturb = hash as u64;
        while table[i as usize].is_some() {
            perturb >>= 5;
            i = i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb) & 7;
        }
        table[i as usize] = Some(v);
    }
    table.iter().flatten().copied().collect()
}

/// The first of `items` with the highest score (Python's `max` with a key
/// keeps the first maximum).
fn first_max<T: Copy>(items: impl IntoIterator<Item = (T, u64)>) -> Option<T> {
    let mut best: Option<(T, u64)> = None;
    for (item, score) in items {
        if best.is_none_or(|(_, b)| score > b) {
            best = Some((item, score));
        }
    }
    best.map(|(item, _)| item)
}

/// Degrees counter-clockwise that straighten the page's rows
/// (`estimate_skew`, spec §4.1). 0 for a blank page.
///
/// The angle whose row profile changes most sharply ([`row_sharpness`]):
/// every 2 degrees over ±45 on a 400-pixel ink thumbnail; then on the
/// 800-pixel one, ±2 degrees in half degrees around 0 and the two best
/// coarse angles, and ±0.5 degrees in tenths around the best of those.
/// The angles are generated as numpy generates them ([`arange`]), so the
/// result is the reference's float, e.g. -29.999999999999993 rather than
/// -30.
///
/// One tie rule is not reproduced exactly: numpy's `argsort` (an
/// introsort) may order equal coarse scores either way; here the later
/// angle ranks higher, as a stable sort would.
pub fn estimate_skew(small: &Gray) -> f64 {
    estimate_skew_threaded(small, 1)
}

/// [`estimate_skew`] scoring the angles of each stage on up to `threads`
/// threads, as the reference does with its thread pool (`estimate_skew(...,
/// pool)`). The result is the same for any number of threads.
pub fn estimate_skew_threaded(small: &Gray, threads: usize) -> f64 {
    let fine = ink_thumb(small, SKEW_SIDE);
    let ink: u64 = fine.data.iter().map(|&v| u64::from(v)).sum();
    if (ink as f64 / fine.data.len() as f64) < 0.003 * 255.0 {
        // A blank page has no rows to straighten.
        return 0.0;
    }
    let coarse = ink_thumb(small, SKEW_SIDE / 2);
    let grid = arange(-DESKEW_MAX, DESKEW_MAX + 1e-6, 2.0);
    let scores = sharpness_at(&coarse, &grid, threads);
    let mut ranked: Vec<usize> = (0..grid.len()).collect();
    ranked.sort_by_key(|&i| scores[i]);
    let best_two = &ranked[ranked.len().saturating_sub(2)..];
    let mut starts = vec![0.0];
    starts.extend(best_two.iter().map(|&i| grid[i]));
    let starts = python_set_order(&starts);

    // Each angle of the overlapping half-degree ranges is scored once, in
    // the order it first appears.
    let mut near: Vec<f64> = Vec::new();
    for a0 in starts {
        for a in arange(a0 - 2.0, a0 + 2.0 + 1e-6, 0.5) {
            if !near.contains(&a) {
                near.push(a);
            }
        }
    }
    let near_scores = sharpness_at(&fine, &near, threads);
    let Some(a1) = first_max(near.into_iter().zip(near_scores)) else {
        return 0.0;
    };
    let last = arange(a1 - 0.5, a1 + 0.5 + 1e-6, 0.1);
    let last_scores = sharpness_at(&fine, &last, threads);
    first_max(last.into_iter().zip(last_scores)).unwrap_or(0.0)
}

/// [`row_sharpness`] at every angle, in order, on up to `threads` threads.
fn sharpness_at(ink: &Gray, angles: &[f64], threads: usize) -> Vec<u64> {
    let threads = threads.clamp(1, angles.len().max(1));
    if threads == 1 {
        return angles.iter().map(|&a| row_sharpness(ink, a)).collect();
    }
    let chunk = angles.len().div_ceil(threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> = angles
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|&a| row_sharpness(ink, a))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("skew scoring thread"))
            .collect()
    })
}

/// The orientation classifier's input, `[768, 768]` ink values (spec §4.2).
///
/// The page area-resized to 768 on its longer side (also when that
/// enlarges it), as `1 - v / 255` in single precision at the top left,
/// zero elsewhere.
pub fn orient_input(gray: &Gray) -> Vec<f32> {
    let (w, h) = scaled_size(gray.width, gray.height, ORIENT_SIDE);
    let small = imgops::resize(gray, w, h, Interpolation::Area);
    let mut out = vec![0f32; ORIENT_SIDE * ORIENT_SIDE];
    for (out_row, row) in out
        .chunks_exact_mut(ORIENT_SIDE)
        .zip(small.data.chunks_exact(small.width))
    {
        for (o, &v) in out_row.iter_mut().zip(row) {
            *o = ink_value(v);
        }
    }
    out
}

/// `1.0 - v.astype(np.float32) / 255.0`: numpy 2 keeps the Python floats
/// weak, so both steps are single precision.
pub fn ink_value(v: u8) -> f32 {
    1.0 - f32::from(v) / 255.0
}

/// The turn to take from the classifier's five probabilities: the argmax
/// class when it is above [`ORIENT_MIN`], else 0.
///
/// numpy's `argmax` keeps the first of equal maxima, and compares the
/// float32 probability with 0.95 in single precision (numpy 2's weak
/// Python scalars).
pub fn orientation_class(probabilities: &[f32]) -> usize {
    let mut k = 0;
    for (i, &p) in probabilities.iter().enumerate() {
        if p > probabilities[k] {
            k = i;
        }
    }
    match probabilities.get(k) {
        Some(&p) if p > ORIENT_MIN => k,
        _ => 0,
    }
}

/// `rotate_page`: the page turned `degrees` about its centre, edges
/// replicated.
pub fn rotate_page(gray: &Gray, degrees: f64) -> Gray {
    let m = imgops::rotation_matrix(gray.width as f64 / 2.0, gray.height as f64 / 2.0, degrees);
    imgops::warp_affine(gray, &m, gray.width, gray.height, Border::Replicate)
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
///
/// Class 1-3 means the content is turned that many quarter turns
/// counter-clockwise, so the page is turned back clockwise; class
/// [`VERTICAL`] turns it a quarter counter-clockwise, so its columns read
/// as lines, the first (right) column on top. After a quarter turn the
/// rows were columns, so the tilt is measured again on the preview turned
/// alike. `skew` in the result is unrounded; the reference reports it
/// rounded to 0.01 ([`round_to`]) but levels and maps boxes back with the
/// unrounded angle.
///
/// # Panics
/// When `k` is not 0-4.
pub fn turn(gray: &Gray, k: usize, skew: f64, small: &Gray) -> Turned {
    assert!(k <= VERTICAL, "orientation class {k}");
    let q = if k == VERTICAL { 1 } else { -(k as i32) };
    let mut page = if k == 0 {
        gray.clone()
    } else {
        imgops::rot90(gray, q)
    };
    let mut a = skew;
    if matches!(k, 1 | 3 | VERTICAL) {
        a = estimate_skew(&imgops::rot90(small, q));
    }
    if a.abs() >= DESKEW_MIN {
        page = rotate_page(&page, a);
    } else {
        a = 0.0;
    }
    Turned {
        gray: page,
        k,
        skew: a,
    }
}

/// `unrotate_box`: a box on the levelled page mapped back to the page
/// before levelling, clipped to it and rounded to 0.1.
///
/// The four corners go through `getRotationMatrix2D((w/2, h/2), -skew, 1)`;
/// the result is their bounding box. `width` and `height` are the levelled
/// page's (the same as the unlevelled one's).
pub fn unrotate_box(bbox: [f64; 4], skew: f64, width: usize, height: usize) -> [f64; 4] {
    let (w, h) = (width as f64, height as f64);
    let m = imgops::rotation_matrix(w / 2.0, h / 2.0, -skew);
    let [x0, y0, x1, y1] = bbox;
    let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
    let mapped = corners.map(|(x, y)| (m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5]));
    let fold = |pick: fn(&(f64, f64)) -> f64, better: fn(f64, f64) -> f64| {
        mapped.iter().map(pick).reduce(better).unwrap_or(0.0)
    };
    let clip = |v: f64, hi: f64| round_to(v.max(0.0).min(hi), 1);
    [
        clip(fold(|c| c.0, f64::min), w),
        clip(fold(|c| c.1, f64::min), h),
        clip(fold(|c| c.0, f64::max), w),
        clip(fold(|c| c.1, f64::max), h),
    ]
}

/// Python's `round(x, digits)`: the nearest number with that many decimals
/// to the exact binary value of `x`, ties to even (`round(0.25, 1)` is
/// 0.2; `round(2.675, 2)` is 2.67 because 2.675 is stored below the tie).
/// Rust's fixed-precision formatting rounds the exact value the same way.
pub fn round_to(x: f64, digits: usize) -> f64 {
    if !x.is_finite() {
        return x;
    }
    format!("{x:.digits$}").parse().unwrap_or(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page of dark horizontal rules, tilted `degrees`.
    fn ruled_page(width: usize, height: usize, degrees: f64) -> Gray {
        let mut g = Gray::filled(width, height, 255);
        for y in (40..height - 40).step_by(24) {
            for x in 30..width - 30 {
                for t in 0..3 {
                    g.data[(y + t) * width + x] = 20;
                }
            }
        }
        rotate_page(&g, degrees)
    }

    #[test]
    fn arange_reproduces_numpys_floats() {
        // np.arange(-30.5, -29.5 + 1e-6, 0.1)
        let last = arange(-30.5, -30.0 + 0.5 + 1e-6, 0.1);
        assert_eq!(last.len(), 11);
        assert_eq!(last[0], -30.5);
        assert_eq!(last[1], -30.4);
        assert_eq!(last[5], -29.999999999999993);
        let grid = arange(-DESKEW_MAX, DESKEW_MAX + 1e-6, 2.0);
        assert_eq!(grid.len(), 46);
        assert_eq!((grid[0], grid[45]), (-45.0, 45.0));
        assert_eq!(arange(-2.0, 2.0 + 1e-6, 0.5).len(), 9);
        assert!(arange(1.0, 0.0, 0.5).is_empty());
    }

    #[test]
    fn set_order_follows_cpython() {
        // Checked with CPython 3.12: list({0.0, *(float(a) for a in (...))}).
        assert_eq!(
            python_set_order(&[0.0, -45.0, 43.0]),
            vec![0.0, 43.0, -45.0]
        );
        assert_eq!(python_set_order(&[0.0, -1.0, 1.0]), vec![0.0, 1.0, -1.0]);
        assert_eq!(python_set_order(&[0.0, 3.0, 11.0]), vec![0.0, 11.0, 3.0]);
        assert_eq!(python_set_order(&[0.0, 1.0, 9.0]), vec![0.0, 1.0, 9.0]);
        assert_eq!(
            python_set_order(&[0.0, -45.0, -37.0]),
            vec![0.0, -45.0, -37.0]
        );
        assert_eq!(python_set_order(&[0.0, 5.0, -3.0]), vec![0.0, -3.0, 5.0]);
        assert_eq!(python_set_order(&[0.0, -1.0, 7.0]), vec![0.0, -1.0, 7.0]);
    }

    #[test]
    fn skew_of_a_tilted_page_is_found() {
        let page = ruled_page(500, 400, -7.3);
        let a = estimate_skew(&page);
        assert!((a - 7.3).abs() < 0.15, "{a}");
        let level = ruled_page(500, 400, 0.0);
        assert!(estimate_skew(&level).abs() < 0.15);
    }

    #[test]
    fn threads_do_not_change_the_skew() {
        let page = ruled_page(400, 300, 3.1);
        let one = estimate_skew(&page);
        assert_eq!(estimate_skew_threaded(&page, 4), one);
        assert_eq!(estimate_skew_threaded(&page, 100), one);
    }

    #[test]
    fn blank_page_has_no_skew() {
        assert_eq!(estimate_skew(&Gray::filled(300, 200, 255)), 0.0);
        assert_eq!(estimate_skew(&Gray::filled(30, 2000, 255)), 0.0);
    }

    #[test]
    fn preview_shrinks_only_large_pages() {
        let g = Gray::filled(1600, 900, 200);
        assert_eq!(preview(&g), g);
        let p = preview(&Gray::filled(3300, 2550, 200));
        // 1600 / 3300 * 2550 = 1236.36...
        assert_eq!((p.width, p.height), (1600, 1236));
        assert!(p.data.iter().all(|&v| v == 200));
    }

    #[test]
    fn scaled_sizes_round_half_to_even() {
        // 768 / 1536 * 1 = 0.5 -> 0 -> at least 1; 768 / 512 * 3 = 4.5 -> 4.
        assert_eq!(scaled_size(1536, 1, 768), (768, 1));
        assert_eq!(scaled_size(3, 512, 768), (4, 768));
        assert_eq!(scaled_size(5, 512, 768), (8, 768)); // 7.5 -> 8
    }

    #[test]
    fn orient_input_is_padded_ink() {
        let g = Gray::filled(200, 100, 0);
        let x = orient_input(&g);
        assert_eq!(x.len(), 768 * 768);
        assert_eq!(x[0], 1.0);
        assert_eq!(x[383 * 768 + 767], 1.0);
        assert_eq!(x[384 * 768], 0.0);
        assert_eq!(ink_value(255), 0.0);
        assert_eq!(ink_value(51), 0.8);
    }

    #[test]
    fn orientation_needs_confidence() {
        assert_eq!(orientation_class(&[0.009, 0.059, 0.012, 0.918, 0.003]), 0);
        assert_eq!(orientation_class(&[0.01, 0.0, 0.96, 0.03, 0.0]), 2);
        assert_eq!(orientation_class(&[0.0, 0.0, 0.0, 0.0, 0.99]), VERTICAL);
        // 0.95 itself, as float32, is not above float32(0.95).
        assert_eq!(orientation_class(&[0.05, 0.95, 0.0, 0.0, 0.0]), 0);
        assert_eq!(orientation_class(&[0.5, 0.5]), 0);
        assert_eq!(orientation_class(&[]), 0);
    }

    #[test]
    fn turn_quarter_turns_back_and_relevels() {
        let page = ruled_page(300, 200, 0.0);
        // Content turned a quarter counter-clockwise (class 1) is turned
        // back clockwise.
        let sideways = imgops::rot90(&page, 1);
        let t = turn(&sideways, 1, 12.0, &sideways);
        assert_eq!(t.gray, page);
        assert_eq!((t.k, t.skew), (1, 0.0));
        let t = turn(&page, 2, 0.1, &page);
        assert_eq!(t.gray, imgops::rot90(&page, 2));
        assert_eq!(t.skew, 0.0);
        let t = turn(&page, VERTICAL, 0.0, &page);
        assert_eq!(t.gray, imgops::rot90(&page, 1));
        let t = turn(&page, 0, -2.5, &page);
        assert_eq!(t.gray, rotate_page(&page, -2.5));
        assert_eq!(t.skew, -2.5);
    }

    #[test]
    fn unrotate_box_maps_corners_back() {
        // No skew: the box is only clipped and rounded.
        assert_eq!(
            unrotate_box([-3.0, 0.25, 101.06, 40.75], 0.0, 100, 50),
            [0.0, 0.2, 100.0, 40.8]
        );
        // A box about the centre turned by 90 degrees swaps its extents.
        let b = unrotate_box([40.0, 45.0, 60.0, 55.0], 90.0, 100, 100);
        assert_eq!(b, [45.0, 40.0, 55.0, 60.0]);
    }

    #[test]
    fn round_to_matches_python() {
        assert_eq!(round_to(0.25, 1), 0.2);
        assert_eq!(round_to(0.75, 1), 0.8);
        assert_eq!(round_to(2.675, 2), 2.67);
        assert_eq!(round_to(-29.999999999999993, 2), -30.0);
        assert_eq!(round_to(1.0e-16, 2), 0.0);
        assert_eq!(round_to(123.45, 1), 123.5); // stored above the tie
        assert_eq!(round_to(0.5, 0), 0.0);
        assert_eq!(round_to(1.5, 0), 2.0);
    }
}

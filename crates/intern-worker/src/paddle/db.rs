//! Text lines from a detection probability map.
//!
//! The detector scores every pixel of the page it was given for being the
//! core of a text line. This turns that map into line outlines the way the
//! reference DB post-processing does: threshold the map, take each
//! connected blob, fit the smallest rotated rectangle around it, keep it
//! if the map is confident across that rectangle, and grow it back out to
//! the ink the detector was trained to shrink it from.

use super::geometry::{Point, Quad, min_area_rect};

/// The post-processing settings that come with a detection model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DbParams {
    /// A pixel at or below this probability is background.
    pub thresh: f32,
    /// A blob whose mean probability over its rectangle is below this is
    /// dropped.
    pub box_thresh: f32,
    /// How far each rectangle is grown: its area times this over its
    /// perimeter, on every side.
    pub unclip_ratio: f32,
    /// At most this many blobs are considered, in scan order.
    pub max_candidates: usize,
    /// A rectangle whose shorter side is under this many map pixels is
    /// noise, before growing; after growing the bar is two pixels higher.
    pub min_size: f32,
}

impl DbParams {
    /// The settings published with the mobile detection models.
    pub const MOBILE: Self = Self {
        thresh: 0.3,
        box_thresh: 0.6,
        unclip_ratio: 1.5,
        max_candidates: 1000,
        min_size: 3.0,
    };
}

/// One detected line: its outline in the destination image's pixels, and
/// how confident the map was across it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectedLine {
    pub quad: Quad,
    pub score: f32,
}

/// Lines found on a `width` x `height` probability map, scaled to a
/// `dest_width` x `dest_height` image.
pub fn lines_from_map(
    probabilities: &[f32],
    width: usize,
    height: usize,
    dest_width: f32,
    dest_height: f32,
    params: &DbParams,
) -> Vec<DetectedLine> {
    assert_eq!(probabilities.len(), width * height, "map size");
    let mut labelled = vec![false; width * height];
    let mut lines = Vec::new();
    let mut candidates = 0;
    let mut stack: Vec<usize> = Vec::new();
    // Each blob's leftmost and rightmost pixel per row: its hull is the
    // hull of these, which is a fraction of its pixels. The per-row slots
    // are shared by every blob and reset through `touched`.
    let mut extremes: Vec<Point> = Vec::new();
    let mut row_low = vec![usize::MAX; height];
    let mut row_high = vec![0_usize; height];
    let mut touched: Vec<usize> = Vec::new();
    let scale_x = dest_width / width as f32;
    let scale_y = dest_height / height as f32;
    for start in 0..probabilities.len() {
        if labelled[start] || probabilities[start] <= params.thresh {
            continue;
        }
        if candidates >= params.max_candidates {
            break;
        }
        candidates += 1;
        extremes.clear();
        labelled[start] = true;
        stack.push(start);
        while let Some(index) = stack.pop() {
            let (x, y) = (index % width, index / width);
            if row_low[y] == usize::MAX {
                touched.push(y);
            }
            row_low[y] = row_low[y].min(x);
            row_high[y] = row_high[y].max(x);
            let (x0, x1) = (x.saturating_sub(1), (x + 1).min(width - 1));
            let (y0, y1) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for ny in y0..=y1 {
                for nx in x0..=x1 {
                    let neighbour = ny * width + nx;
                    if !labelled[neighbour] && probabilities[neighbour] > params.thresh {
                        labelled[neighbour] = true;
                        stack.push(neighbour);
                    }
                }
            }
        }
        for row in touched.drain(..) {
            extremes.push(Point::new(row_low[row] as f32, row as f32));
            extremes.push(Point::new(row_high[row] as f32, row as f32));
            row_low[row] = usize::MAX;
            row_high[row] = 0;
        }
        let Some(rect) = min_area_rect(&extremes) else {
            continue;
        };
        if rect.shorter_side() < params.min_size {
            continue;
        }
        let core = Quad::from_rect(&rect);
        let score = mean_inside(probabilities, width, height, &core);
        if score < params.box_thresh {
            continue;
        }
        let area = rect.width * rect.height;
        let perimeter = 2.0 * (rect.width + rect.height);
        let grown = rect.grown(area * params.unclip_ratio / perimeter.max(f32::EPSILON));
        if grown.shorter_side() < params.min_size + 2.0 {
            continue;
        }
        let quad = Quad::from_rect(&grown)
            .scaled(scale_x, scale_y)
            .clamped(dest_width, dest_height);
        lines.push(DetectedLine { quad, score });
    }
    lines
}

/// The mean probability over the map pixels inside `quad`.
///
/// The reference "fast" score: the pixels of the quad's upright bounding
/// box that the quad covers, by pixel position.
fn mean_inside(probabilities: &[f32], width: usize, height: usize, quad: &Quad) -> f32 {
    let [x0, y0, x1, y1] = quad.bounds();
    let clamp = |value: f32, limit: usize| (value.max(0.0) as usize).min(limit - 1);
    let (left, right) = (clamp(x0.floor(), width), clamp(x1.ceil(), width));
    let (top, bottom) = (clamp(y0.floor(), height), clamp(y1.ceil(), height));
    let mut sum = 0.0_f64;
    let mut count = 0_u32;
    for y in top..=bottom {
        for x in left..=right {
            if quad.contains(Point::new(x as f32, y as f32)) {
                sum += f64::from(probabilities[y * width + x]);
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum / f64::from(count)) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_with(
        width: usize,
        height: usize,
        boxes: &[(usize, usize, usize, usize, f32)],
    ) -> Vec<f32> {
        let mut map = vec![0.0; width * height];
        for &(x0, y0, x1, y1, value) in boxes {
            for y in y0..y1 {
                for x in x0..x1 {
                    map[y * width + x] = value;
                }
            }
        }
        map
    }

    #[test]
    fn a_confident_blob_becomes_one_grown_line() {
        // An 80 x 10 core at (20, 30) on a 200 x 100 map.
        let map = map_with(200, 100, &[(20, 30, 100, 40, 0.9)]);
        let lines = lines_from_map(&map, 200, 100, 200.0, 100.0, &DbParams::MOBILE);
        assert_eq!(lines.len(), 1, "{lines:?}");
        let line = lines[0];
        assert!((line.score - 0.9).abs() < 1e-4, "{line:?}");
        // The core spans pixel centres 20..=99 by 30..=39: 79 x 9. Grown by
        // 79 * 9 * 1.5 / (2 * 88) = 6.06 on every side.
        let [x0, y0, x1, y1] = line.quad.bounds();
        let grow = 79.0 * 9.0 * 1.5 / 176.0;
        for (found, expected) in [
            (x0, 20.0 - grow),
            (y0, 30.0 - grow),
            (x1, 99.0 + grow),
            (y1, 39.0 + grow),
        ] {
            assert!((found - expected).abs() < 1e-3, "{found} vs {expected}");
        }
    }

    #[test]
    fn faint_blobs_and_specks_are_dropped() {
        let map = map_with(
            200,
            100,
            &[
                // Above the pixel threshold, below the box threshold.
                (10, 10, 90, 20, 0.45),
                // Confident, but two pixels tall: noise.
                (10, 60, 90, 62, 0.95),
                // A real line.
                (10, 80, 150, 92, 0.8),
            ],
        );
        let lines = lines_from_map(&map, 200, 100, 200.0, 100.0, &DbParams::MOBILE);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].quad.bounds()[1] > 70.0);
    }

    #[test]
    fn lines_are_scaled_back_to_the_page_and_held_inside_it() {
        // The map is half the page each way, and the line touches its edge.
        let map = map_with(100, 50, &[(0, 10, 100, 20, 0.9)]);
        let lines = lines_from_map(&map, 100, 50, 200.0, 100.0, &DbParams::MOBILE);
        assert_eq!(lines.len(), 1);
        let [x0, y0, x1, y1] = lines[0].quad.bounds();
        assert_eq!(x0, 0.0);
        assert_eq!(x1, 200.0);
        assert!(y0 > 5.0 && y0 < 20.0, "{y0}");
        assert!(y1 > 38.0 && y1 < 60.0, "{y1}");
    }

    #[test]
    fn separate_blobs_are_separate_lines_and_diagonal_neighbours_join() {
        let mut map = map_with(
            300,
            60,
            &[
                (10, 10, 110, 22, 0.9),
                (150, 10, 290, 22, 0.9),
                // Two halves of one line with an empty column between them...
                (10, 40, 60, 52, 0.9),
                (61, 40, 110, 52, 0.9),
            ],
        );
        // ...joined only through one pixel above the gap, which touches
        // each half at a corner. Eight-connectivity follows it.
        map[39 * 300 + 60] = 0.9;
        let lines = lines_from_map(&map, 300, 60, 300.0, 60.0, &DbParams::MOBILE);
        assert_eq!(lines.len(), 3, "{lines:?}");
    }

    #[test]
    fn a_tilted_line_keeps_its_slope() {
        let (width, height) = (400, 200);
        let mut map = vec![0.0_f32; width * height];
        // A 300-pixel line, 12 pixels thick, climbing at four degrees.
        let slope = 4_f32.to_radians().tan();
        for x in 50..350 {
            let centre = 120.0 - (x as f32 - 50.0) * slope;
            for y in (centre - 6.0) as usize..(centre + 6.0) as usize {
                map[y * width + x] = 0.9;
            }
        }
        let lines = lines_from_map(
            &map,
            width,
            height,
            width as f32,
            height as f32,
            &DbParams::MOBILE,
        );
        assert_eq!(lines.len(), 1);
        assert!(
            (lines[0].quad.slope_degrees() - 4.0).abs() < 0.5,
            "{:?}",
            lines[0]
        );
    }
}

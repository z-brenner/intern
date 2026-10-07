//! How far a scanned page is turned off level, read from the lines the
//! detector found on it.
//!
//! Every detected line is a rectangle at its own angle, so a page fed in
//! three degrees crooked reports three degrees on nearly every long line.
//! Short lines - a page number, a checkbox label - are a few characters
//! wide and their angle is mostly noise, so only lines several times longer
//! than they are tall vote, each in proportion to its length.

use super::geometry::Quad;

/// A line has to be this many times longer than it is tall to vote.
const ELONGATION: f32 = 4.0;

/// Lines that have to vote before the estimate is believed.
const MIN_VOTERS: usize = 3;

/// The page's skew in degrees, positive when its lines climb to the right,
/// as the length-weighted median of its long lines' slopes. `None` when too
/// few lines are long enough to say.
pub fn estimate_skew(quads: &[Quad]) -> Option<f32> {
    let mut votes: Vec<(f32, f32)> = quads
        .iter()
        .filter(|quad| quad.width() >= ELONGATION * quad.height().max(1.0))
        .map(|quad| (quad.slope_degrees(), quad.width()))
        // A "line" at more than 45 degrees is a vertical one, read sideways;
        // its slope says nothing about the page's.
        .filter(|(slope, _)| slope.abs() < 45.0)
        .collect();
    if votes.len() < MIN_VOTERS {
        return None;
    }
    votes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total: f32 = votes.iter().map(|(_, weight)| weight).sum();
    let mut running = 0.0;
    for (slope, weight) in &votes {
        running += weight;
        if running >= total / 2.0 {
            return Some(*slope);
        }
    }
    votes.last().map(|(slope, _)| *slope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paddle::geometry::{Point, RotatedRect};

    fn line(x: f32, y: f32, length: f32, thickness: f32, degrees: f32) -> Quad {
        let radians = degrees.to_radians();
        Quad::from_rect(&RotatedRect {
            centre: Point::new(x, y),
            width: length,
            height: thickness,
            axis: Point::new(radians.cos(), -radians.sin()),
        })
    }

    #[test]
    fn a_crooked_page_reports_its_angle() {
        let quads: Vec<Quad> = (0..8)
            .map(|row| line(1000.0, 200.0 + row as f32 * 60.0, 1500.0, 40.0, 3.0))
            .collect();
        let skew = estimate_skew(&quads).unwrap();
        assert!((skew - 3.0).abs() < 0.05, "{skew}");
    }

    #[test]
    fn short_and_vertical_lines_do_not_vote() {
        let mut quads: Vec<Quad> = (0..4)
            .map(|row| line(1000.0, 200.0 + row as f32 * 60.0, 1500.0, 40.0, -1.5))
            .collect();
        // Page numbers at a wild angle, and a vertical margin note.
        quads.extend((0..10).map(|row| line(100.0, 100.0 * row as f32, 60.0, 40.0, 20.0)));
        quads.push(line(50.0, 1500.0, 1200.0, 40.0, 90.0));
        let skew = estimate_skew(&quads).unwrap();
        assert!((skew + 1.5).abs() < 0.05, "{skew}");
    }

    #[test]
    fn long_lines_outvote_many_short_ones() {
        let mut quads: Vec<Quad> = (0..3)
            .map(|row| line(1000.0, 200.0 + row as f32 * 60.0, 1800.0, 40.0, 2.0))
            .collect();
        quads.extend((0..4).map(|row| line(300.0, 600.0 + row as f32 * 60.0, 200.0, 40.0, 0.0)));
        let skew = estimate_skew(&quads).unwrap();
        assert!((skew - 2.0).abs() < 0.05, "{skew}");
    }

    #[test]
    fn too_few_lines_say_nothing() {
        let quads = [line(500.0, 500.0, 900.0, 30.0, 5.0)];
        assert_eq!(estimate_skew(&quads), None);
        assert_eq!(estimate_skew(&[]), None);
    }
}

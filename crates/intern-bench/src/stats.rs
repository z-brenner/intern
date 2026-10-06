//! Distributions for latency and size metrics.
//!
//! Percentiles are nearest-rank: the value at rank ⌈p·n⌉ of the sorted
//! sample, so every percentile reported is a value some document actually
//! took. With the fifty-odd documents of the corpus an interpolated p95
//! would describe a document that does not exist.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Distribution {
    pub count: usize,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub max: f64,
    pub mean: f64,
}

impl Distribution {
    /// `None` for an empty sample. Values that are not finite are dropped.
    pub fn of(values: &[f64]) -> Option<Self> {
        let mut sorted = values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .collect::<Vec<_>>();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f64::total_cmp);
        let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
        Some(Self {
            count: sorted.len(),
            p50: nearest_rank(&sorted, 50.0),
            p90: nearest_rank(&sorted, 90.0),
            p95: nearest_rank(&sorted, 95.0),
            max: sorted[sorted.len() - 1],
            mean: round(mean, 3),
        })
    }
}

/// The nearest-rank percentile of an already sorted, non-empty sample.
pub fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    // Multiplied before dividing, so 95 × 20 / 100 is exactly 19 rather than
    // a hair above it and a rank too far.
    let rank = (percentile * sorted.len() as f64 / 100.0).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// Rounds to `places` decimal places, so the report does not carry
/// floating-point noise that makes two equal runs look different.
pub fn round(value: f64, places: i32) -> f64 {
    let scale = 10_f64.powi(places);
    (value * scale).round() / scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_picks_a_real_sample() {
        let sorted = (1..=20).map(f64::from).collect::<Vec<_>>();
        assert_eq!(nearest_rank(&sorted, 50.0), 10.0);
        assert_eq!(nearest_rank(&sorted, 90.0), 18.0);
        assert_eq!(nearest_rank(&sorted, 95.0), 19.0);
        assert_eq!(nearest_rank(&sorted, 100.0), 20.0);
        assert_eq!(
            nearest_rank(&sorted, 0.0),
            1.0,
            "rank never falls below one"
        );
        // The textbook example: 15, 20, 35, 40, 50.
        let sample = [15.0, 20.0, 35.0, 40.0, 50.0];
        assert_eq!(nearest_rank(&sample, 30.0), 20.0);
        assert_eq!(nearest_rank(&sample, 40.0), 20.0);
        assert_eq!(nearest_rank(&sample, 50.0), 35.0);
    }

    #[test]
    fn a_distribution_sorts_its_sample_and_ignores_what_is_not_a_number() {
        let distribution = Distribution::of(&[5.0, 1.0, f64::NAN, 3.0, 2.0, 4.0]).unwrap();
        assert_eq!(distribution.count, 5);
        assert_eq!(distribution.p50, 3.0);
        assert_eq!(distribution.p90, 5.0);
        assert_eq!(distribution.p95, 5.0);
        assert_eq!(distribution.max, 5.0);
        assert_eq!(distribution.mean, 3.0);

        let single = Distribution::of(&[7.25]).unwrap();
        assert_eq!((single.p50, single.p95, single.max), (7.25, 7.25, 7.25));
        assert_eq!(Distribution::of(&[]), None);
    }

    #[test]
    fn rounding_keeps_the_report_free_of_float_noise() {
        assert_eq!(round(0.1 + 0.2, 6), 0.3);
        assert_eq!(round(2.0 / 3.0, 3), 0.667);
    }
}

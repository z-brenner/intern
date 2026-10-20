//! CTC prefix beam search under the language model, for the lines the
//! recognizer is unsure of (spec §7.4).

use super::lm::NgramLm;

/// How a line is searched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeamParams {
    pub alpha: f64,
    pub beta: f64,
    pub beam: usize,
    pub blank_skip: f64,
    pub cand_floor: f64,
}

impl Default for BeamParams {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 1.5,
            beam: 8,
            blank_skip: 0.999,
            cand_floor: 3e-3,
        }
    }
}

/// The best prefix and the steps that emitted it, from one line's `n` steps
/// of log P(blank), and its top-`k` log-probabilities and classes a step
/// (`topv`/`topi` are `n * k`, row-major).
pub fn beam_search(
    blank: &[f32],
    topv: &[f32],
    topi: &[i64],
    k: usize,
    lm: &NgramLm,
    params: BeamParams,
) -> (Vec<u32>, Vec<usize>) {
    let _ = (blank, topv, topi, k, lm, params);
    todo!("spec §7.4")
}

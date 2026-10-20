//! CTC prefix beam search under the language model, for the lines the
//! recognizer is unsure of (spec §7.4, `beam_search`, `lm.py:163-254`).
//!
//! Each prefix (a sequence of class ids) carries the log-probability of the
//! steps so far ending in a blank and ending in its last class, the summed
//! language-model bonus, the step that emitted each of its classes, and its
//! last `order - 1` tokens packed as a history key. A prefix is ranked by
//! `lse(blank, non-blank) + bonus`. Ties are broken exactly as the reference
//! breaks them: prefixes are kept in insertion order and sorted stably.

use super::lm::NgramLm;

/// A recognizer's lines are searched when their greedy confidence is at
/// least the reading threshold and under this gate (`LM_GATE`,
/// `pipeline.py:73`); `meta.json` `lm_params` may set it per group.
pub const LM_GATE: f64 = 0.9;
/// The language model's weight (`LM_ALPHA`, `pipeline.py:74`).
pub const LM_ALPHA: f64 = 0.5;
/// The reward for every emitted token (`LM_BETA`, `pipeline.py:75`).
pub const LM_BETA: f64 = 1.5;
/// Prefixes kept a step (`LM_BEAM`, `pipeline.py:76`).
pub const LM_BEAM: usize = 8;
/// A class under this probability at a step is not tried there
/// (`LM_CAND_FLOOR`, `pipeline.py:79`; the function's own default,
/// 1e-3, is never used by the pipeline).
pub const LM_CAND_FLOOR: f64 = 3e-3;
/// A step whose blank probability is over this is a blank for every prefix
/// (`blank_skip`, `lm.py:171`).
pub const BLANK_SKIP: f64 = 0.999;

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
    /// The pipeline's parameters for the base recognizer
    /// (`pipeline.py:487-500`).
    fn default() -> Self {
        Self {
            alpha: LM_ALPHA,
            beta: LM_BETA,
            beam: LM_BEAM,
            blank_skip: BLANK_SKIP,
            cand_floor: LM_CAND_FLOOR,
        }
    }
}

/// `log(exp(a) + exp(b))` as the reference computes it (`_lse`,
/// `lm.py:154-160`), in f64.
fn lse(a: f64, b: f64) -> f64 {
    if a == f64::NEG_INFINITY {
        return b;
    }
    if b == f64::NEG_INFINITY {
        return a;
    }
    let m = if b > a { b } else { a };
    m + ((a - m).exp() + (b - m).exp()).ln()
}

/// One prefix of the search and its scores.
#[derive(Clone, Debug)]
struct Beam {
    prefix: Vec<u32>,
    /// log P of the steps so far, ending in a blank.
    blank: f64,
    /// log P of the steps so far, ending in the prefix's last class.
    non_blank: f64,
    /// The summed language-model bonus `alpha * logp + beta` a token.
    bonus: f64,
    /// The step that first emitted each class of the prefix.
    steps: Vec<usize>,
    /// The prefix's last `order - 1` tokens, packed.
    history: u64,
}

impl Beam {
    fn rank(&self) -> f64 {
        lse(self.blank, self.non_blank) + self.bonus
    }

    /// Whether this prefix is `parent` and then `class`.
    fn extends(&self, parent: &[u32], class: u32) -> bool {
        self.prefix.len() == parent.len() + 1
            && self.prefix[parent.len()] == class
            && self.prefix[..parent.len()] == *parent
    }
}

/// The reference's `add` (`lm.py:196-201`) for a prefix already in `next`:
/// it gathers the new probability mass and keeps its bonus, steps and
/// history. A new prefix is appended instead, so `next` stays in insertion
/// order.
fn add_existing(beam: &mut Beam, blank: f64, non_blank: f64) {
    beam.blank = lse(beam.blank, blank);
    beam.non_blank = lse(beam.non_blank, non_blank);
}

/// A new prefix found at a step: the beam it extends, the class, and its
/// log-probability ending in that class.
struct Extension {
    from: usize,
    class: u32,
    non_blank: f64,
}

/// The best prefix and the steps that emitted it, from one line's `n` steps
/// of log P(blank), and its top-`k` log-probabilities and classes a step
/// (`topv`/`topi` are `n * k`, row-major).
///
/// A class id under 1 (the blank, or anything negative, which the
/// recognizer never emits) is not a candidate.
pub fn beam_search(
    blank: &[f32],
    topv: &[f32],
    topi: &[i64],
    k: usize,
    lm: &NgramLm,
    params: BeamParams,
) -> (Vec<u32>, Vec<usize>) {
    let steps = blank.len();
    assert_eq!(topv.len(), steps * k, "topv is n x k");
    assert_eq!(topi.len(), steps * k, "topi is n x k");
    let bits = lm.bits;
    let history_mask = lm.history_mask();
    let log_skip = params.blank_skip.ln();
    let log_floor = params.cand_floor.ln();
    // An upper bound of what a token's bonus can add (`reach`, `lm.py:223`).
    let reach = params.beta + params.alpha * lm.max_logp;
    let mut beams = vec![Beam {
        prefix: Vec::new(),
        blank: 0.0,
        non_blank: f64::NEG_INFINITY,
        bonus: 0.0,
        steps: Vec::new(),
        history: 0,
    }];
    let mut candidates: Vec<(u32, f64)> = Vec::with_capacity(k);
    let mut extensions: Vec<Extension> = Vec::new();
    for (t, &b0) in blank.iter().enumerate() {
        let b0 = f64::from(b0);
        if b0 > log_skip {
            // A sure blank: every prefix ends in a blank, nothing is tried.
            for beam in &mut beams {
                beam.blank = lse(beam.blank, beam.non_blank) + b0;
                beam.non_blank = f64::NEG_INFINITY;
            }
            continue;
        }
        candidates.clear();
        let row = t * k..(t + 1) * k;
        for (&class, &lp) in topi[row.clone()].iter().zip(&topv[row]) {
            let lp = f64::from(lp);
            if class != 0 && lp >= log_floor {
                // The reference keeps any non-blank class; ids are never
                // negative, and a u32 holds every class of every vocabulary.
                if let Ok(class) = u32::try_from(class) {
                    candidates.push((class, lp));
                }
            }
        }
        // Every beam's own entries first: its blank, and a repeat of its last
        // class that collapses into it (`lm.py:208-217`). Beams hold
        // distinct prefixes, so each beam appends one entry here.
        let mut next: Vec<Beam> = Vec::with_capacity(beams.len() + beams.len() * k);
        for beam in &beams {
            let mut own = Beam {
                blank: lse(beam.blank, beam.non_blank) + b0,
                non_blank: f64::NEG_INFINITY,
                ..beam.clone()
            };
            let last = beam.prefix.last().copied();
            for &(class, lp) in &candidates {
                if Some(class) == last {
                    add_existing(&mut own, f64::NEG_INFINITY, beam.non_blank + lp);
                }
            }
            next.push(own);
        }
        // With a full beam the lowest of these entries is a floor no new
        // prefix under it can pass, as entries only gain mass below
        // (`lm.py:218-222`). Result-neutral; it spares the lookups.
        let floor = if beams.len() >= params.beam {
            next.iter()
                .map(Beam::rank)
                .fold(f64::INFINITY, |a, b| if b < a { b } else { a })
        } else {
            f64::NEG_INFINITY
        };
        extensions.clear();
        for (from, beam) in beams.iter().enumerate() {
            let total = lse(beam.blank, beam.non_blank);
            let last = beam.prefix.last().copied();
            for &(class, lp) in &candidates {
                // After a blank the same class is a new token.
                let new_from = if Some(class) == last {
                    beam.blank + lp
                } else {
                    total + lp
                };
                if new_from == f64::NEG_INFINITY {
                    continue;
                }
                // An existing beam merges and keeps its bonus (`lm.py:234-235`).
                // Only the step's own entries are in `next` here, one a beam.
                if let Some(existing) = next[..beams.len()]
                    .iter_mut()
                    .find(|other| other.extends(&beam.prefix, class))
                {
                    add_existing(existing, f64::NEG_INFINITY, new_from);
                } else if new_from + beam.bonus + reach >= floor - 1e-9 {
                    extensions.push(Extension {
                        from,
                        class,
                        non_blank: new_from,
                    });
                }
            }
        }
        // New prefixes in creation order, each scored by the model
        // (`lm.py:238-247`). Each is a distinct prefix not yet in `next`.
        for extension in &extensions {
            let beam = &beams[extension.from];
            let key = (beam.history << bits) | (u64::from(extension.class) + 1);
            let logp = lm.logp(key);
            let mut prefix = Vec::with_capacity(beam.prefix.len() + 1);
            prefix.extend_from_slice(&beam.prefix);
            prefix.push(extension.class);
            let mut emitted = Vec::with_capacity(beam.steps.len() + 1);
            emitted.extend_from_slice(&beam.steps);
            emitted.push(t);
            next.push(Beam {
                prefix,
                blank: f64::NEG_INFINITY,
                non_blank: extension.non_blank,
                bonus: beam.bonus + params.alpha * logp + params.beta,
                steps: emitted,
                history: key & history_mask,
            });
        }
        // The best `beam` by rank, stably: ties keep insertion order, as
        // Python's `sorted` keeps them (`lm.py:248-252`).
        let mut ranked: Vec<(f64, Beam)> = next.into_iter().map(|b| (b.rank(), b)).collect();
        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(params.beam);
        beams = ranked.into_iter().map(|(_, b)| b).collect();
    }
    // The first prefix of the highest rank (`max`, `lm.py:253`).
    let mut best = 0;
    let mut best_rank = f64::NEG_INFINITY;
    for (i, beam) in beams.iter().enumerate() {
        let rank = beam.rank();
        if i == 0 || rank > best_rank {
            best = i;
            best_rank = rank;
        }
    }
    let beam = beams.swap_remove(best);
    (beam.prefix, beam.steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indra::lm::LmTable;

    /// Order 2, 4 bits a token, classes 1..=3 (tokens 2..=4): every class
    /// has unigram -1; after class 1, class 2 has -0.1 and the rest back off
    /// with -0.5.
    fn tiny() -> NgramLm {
        let unigrams = LmTable {
            keys: vec![2, 3, 4],
            logp: vec![-1.0, -1.0, -1.0],
            histories: vec![0],
            backoff: vec![0.0],
        };
        let bigrams = LmTable {
            keys: vec![(2 << 4) | 3],
            logp: vec![-0.1],
            histories: vec![2],
            backoff: vec![-0.5],
        };
        NgramLm::from_tables(2, 4, 4, vec![unigrams, bigrams]).unwrap()
    }

    /// One step of a line: log P(blank) and the top classes and their
    /// log-probabilities.
    struct Line {
        blank: Vec<f32>,
        topv: Vec<f32>,
        topi: Vec<i64>,
        k: usize,
    }

    impl Line {
        fn new(k: usize) -> Self {
            Self {
                blank: Vec::new(),
                topv: Vec::new(),
                topi: Vec::new(),
                k,
            }
        }

        fn step(mut self, blank: f32, top: &[(i64, f32)]) -> Self {
            assert_eq!(top.len(), self.k);
            self.blank.push(blank);
            for &(class, lp) in top {
                self.topi.push(class);
                self.topv.push(lp);
            }
            self
        }

        fn search(&self, lm: &NgramLm, params: BeamParams) -> (Vec<u32>, Vec<usize>) {
            beam_search(&self.blank, &self.topv, &self.topi, self.k, lm, params)
        }
    }

    const SURE: f32 = -0.0001;

    #[test]
    fn collapses_repeats_and_keeps_a_class_repeated_after_a_blank() {
        let lm = tiny();
        let line = Line::new(2)
            .step(-5.0, &[(1, -0.01), (0, -5.0)])
            .step(-5.0, &[(1, -0.01), (0, -5.0)])
            .step(SURE, &[(0, SURE), (1, -9.0)])
            .step(-5.0, &[(1, -0.01), (0, -5.0)]);
        assert_eq!(
            line.search(&lm, BeamParams::default()),
            (vec![1, 1], vec![0, 3])
        );
        assert_eq!(
            Line::new(2).search(&lm, BeamParams::default()),
            (vec![], vec![])
        );
    }

    #[test]
    fn the_model_breaks_a_tie_and_insertion_order_breaks_an_exact_one() {
        let lm = tiny();
        let even = (0.45f32).ln();
        let line = Line::new(3)
            .step(-5.0, &[(1, -0.01), (0, -5.0), (2, -8.0)])
            .step(SURE, &[(0, SURE), (1, -9.0), (2, -9.0)])
            .step((0.1f32).ln(), &[(3, even), (2, even), (0, (0.1f32).ln())]);
        // Class 2 follows class 1 in the model.
        assert_eq!(
            line.search(&lm, BeamParams::default()),
            (vec![1, 2], vec![0, 2])
        );
        // Without the model the two readings tie and the first tried stays.
        let blind = BeamParams {
            alpha: 0.0,
            ..BeamParams::default()
        };
        assert_eq!(line.search(&lm, blind), (vec![1, 3], vec![0, 2]));
    }

    #[test]
    fn a_class_under_the_floor_is_not_tried() {
        let lm = tiny();
        // After class 1 the model favours class 2 (-0.1) over class 3
        // (-1.5) by more than the recognizer favours 3 (0.01) over 2 (2e-3),
        // but 2e-3 is under the 3e-3 floor.
        let line = Line::new(3)
            .step(-5.0, &[(1, -0.01), (0, -5.0), (2, -8.0)])
            .step(
                (0.988f32).ln(),
                &[
                    (0, (0.988f32).ln()),
                    (3, (0.01f32).ln()),
                    (2, (0.002f32).ln()),
                ],
            );
        let params = BeamParams {
            alpha: 2.0,
            beta: 10.0,
            ..BeamParams::default()
        };
        assert_eq!(line.search(&lm, params), (vec![1, 3], vec![0, 1]));
        let lower = BeamParams {
            cand_floor: 1e-3,
            ..params
        };
        assert_eq!(line.search(&lm, lower), (vec![1, 2], vec![0, 1]));
    }

    #[test]
    fn the_bound_holds_and_pruning_changes_nothing() {
        let lm = tiny();
        for history in 0..16u64 {
            for token in 0..16u64 {
                assert!(lm.logp((history << 4) | token) <= lm.max_logp);
            }
        }
        let mut unpruned = tiny();
        unpruned.max_logp = f64::INFINITY;
        // A small linear congruential generator: the same lines every run.
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) as f64 / (1u64 << 31) as f64
        };
        for _ in 0..300 {
            let k = 4;
            let mut line = Line::new(k);
            for _ in 0..(3 + (next() * 25.0) as usize) {
                let blank = if next() < 0.2 {
                    SURE
                } else {
                    -(next() * 6.0) as f32
                };
                let mut top: Vec<(i64, f32)> =
                    (0..k).map(|j| (j as i64, -(next() * 7.0) as f32)).collect();
                top.sort_by(|a, b| b.1.total_cmp(&a.1));
                line = line.step(blank, &top);
            }
            for beam in [1, 2, 3, 8] {
                let params = BeamParams {
                    beam,
                    cand_floor: 1e-3,
                    ..BeamParams::default()
                };
                assert_eq!(line.search(&lm, params), line.search(&unpruned, params));
            }
        }
    }
}

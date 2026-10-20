//! Intern's own step after the recognizer: a damaged date or amount
//! repaired from the recognizer's own second choices.
//!
//! The recognizer reports, for every character it emits, its eight most
//! likely classes. Where a line holds a date or an amount that is damaged -
//! a dollar sign read as an `S` (`= S8,220.00`), a letter among a date's
//! digits, a day the month does not have, as PP-OCR's second reading judges
//! them ([`crate::paddle::reocr::field_tally`]) - a character is replaced by
//! one of its step's other candidates when that repairs a damaged field and
//! loses none: the most likely such candidate, one character at a time, at
//! most [`MAX_REPAIRS`] in a line. A line with nothing damaged is left as
//! the recognizer read it.
//!
//! Measured on InternBench's scans, the recognizer read three of a
//! purchase order's line totals with an `S` for the dollar sign; at those
//! steps `$` was its second choice (at 0.45, 0.04 and 0.45), and all three
//! are repaired (`docs/ocr.md`).
//!
//! [`super::IndraOcr::read_reference`], which parity with the reference is
//! measured on, does not run this step.

use super::rec::LineOutputs;
use super::types::{RecLine, TokenKind};
use super::vocab::Vocab;
use crate::paddle::reocr::{FieldTally, field_tally};

/// A candidate less likely than this is not considered: it is noise in the
/// recognizer's list, not a reading it entertained.
pub const MIN_ALTERNATIVE: f32 = 0.01;

/// The most characters repaired in one line.
pub const MAX_REPAIRS: usize = 4;

fn total(tally: FieldTally) -> usize {
    tally.sound + tally.damaged
}

/// Repairs `line`'s damaged dates and amounts from the recognizer's
/// candidates (see the module documentation). `steps` are the time steps
/// its tokens were emitted at, `out` the recognizer's outputs for the line.
/// Returns how many characters were replaced.
pub fn repair_fields(
    line: &mut RecLine,
    steps: &[usize],
    out: &LineOutputs<'_>,
    vocab: &Vocab,
) -> usize {
    if line.tokens.len() != steps.len() || line.text.is_empty() {
        return 0;
    }
    let mut ids: Vec<u32> = Vec::with_capacity(line.tokens.len());
    for token in &line.tokens {
        match token.kind {
            TokenKind::Id(id) => ids.push(id),
            TokenKind::Char(_) => return 0,
        }
    }
    let space = vocab
        .chars
        .iter()
        .position(|character| character == " ")
        .map(|index| index as u32 + 1);
    let mut tally = field_tally(&line.text);
    let mut repaired = 0;
    while tally.damaged > 0 && repaired < MAX_REPAIRS {
        // The most likely single replacement that repairs a field and
        // loses none.
        let mut best: Option<(usize, u32, f32, FieldTally)> = None;
        for (position, &step) in steps.iter().enumerate() {
            if Some(ids[position]) == space {
                continue;
            }
            let row = step * out.k..(step + 1) * out.k;
            let (Some(classes), Some(log_probabilities)) =
                (out.topi.get(row.clone()), out.topv.get(row))
            else {
                continue;
            };
            for (&class, &log_probability) in classes.iter().zip(log_probabilities) {
                let probability = log_probability.exp();
                let Ok(class) = u32::try_from(class) else {
                    continue;
                };
                if class == 0
                    || class == ids[position]
                    || Some(class) == space
                    || probability < MIN_ALTERNATIVE
                    || best.is_some_and(|(_, _, p, _)| p >= probability)
                {
                    continue;
                }
                let previous = std::mem::replace(&mut ids[position], class);
                let offered = field_tally(&vocab.decode(&ids));
                ids[position] = previous;
                if offered.damaged < tally.damaged && total(offered) >= total(tally) {
                    best = Some((position, class, probability, offered));
                }
            }
        }
        let Some((position, class, probability, offered)) = best else {
            break;
        };
        ids[position] = class;
        line.tokens[position].kind = TokenKind::Id(class);
        line.tokens[position].p = probability;
        tally = offered;
        repaired += 1;
    }
    if repaired > 0 {
        line.text = vocab.decode(&ids);
    }
    repaired
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indra::types::Token;

    const K: usize = 3;

    fn vocab() -> Vocab {
        Vocab::new(
            [
                " ", "$", "S", "0", "1", "2", "5", "6", "8", ",", ".", "=", "/", "O",
            ]
            .iter()
            .map(|c| c.to_string())
            .collect(),
        )
    }

    fn id(vocab: &Vocab, c: char) -> u32 {
        vocab
            .chars
            .iter()
            .position(|known| *known == c.to_string())
            .unwrap() as u32
            + 1
    }

    /// A step the recognizer is sure of.
    fn sure(text: &str) -> Vec<Vec<(char, f32)>> {
        text.chars().map(|c| vec![(c, 0.99)]).collect()
    }

    /// A line read one step a character, each step with its candidates as
    /// `(char, p)`, the first the one emitted; the unused candidate slots
    /// are the blank. The line, its steps, and the top-k outputs.
    fn read(
        vocab: &Vocab,
        steps: &[Vec<(char, f32)>],
    ) -> (RecLine, Vec<usize>, Vec<f32>, Vec<i64>) {
        let (mut topv, mut topi, mut tokens) = (Vec::new(), Vec::new(), Vec::new());
        for candidates in steps {
            for slot in 0..K {
                match candidates.get(slot) {
                    Some(&(c, p)) => {
                        topi.push(i64::from(id(vocab, c)));
                        topv.push(p.ln());
                    }
                    None => {
                        topi.push(0);
                        topv.push(1e-6_f32.ln());
                    }
                }
            }
            tokens.push(Token {
                kind: TokenKind::Id(id(vocab, candidates[0].0)),
                x: 0.0,
                p: candidates[0].1,
            });
        }
        let ids: Vec<u32> = steps.iter().map(|c| id(vocab, c[0].0)).collect();
        let line = RecLine {
            text: vocab.decode(&ids),
            conf: 0.95,
            tokens,
        };
        (line, (0..steps.len()).collect(), topv, topi)
    }

    fn repair(vocab: &Vocab, steps: &[Vec<(char, f32)>]) -> (RecLine, usize) {
        let (mut line, steps, topv, topi) = read(vocab, steps);
        let out = LineOutputs {
            ids: &[],
            conf: &[],
            blank: &[],
            topv: &topv,
            topi: &topi,
            k: K,
        };
        let repaired = repair_fields(&mut line, &steps, &out, vocab);
        (line, repaired)
    }

    /// "= S8.50": the S is the recognizer's first choice and `$` its
    /// second; the amount is repaired.
    #[test]
    fn a_dollar_sign_read_as_an_s_is_repaired_from_the_second_choice() {
        let vocab = vocab();
        let mut steps = sure("= ");
        steps.push(vec![('S', 0.54), ('$', 0.45), ('5', 0.01)]);
        steps.extend(sure("8.50"));
        let (line, repaired) = repair(&vocab, &steps);
        assert_eq!((line.text.as_str(), repaired), ("= $8.50", 1));
        assert_eq!(line.tokens[2].kind, TokenKind::Id(id(&vocab, '$')));
        assert!((line.tokens[2].p - 0.45).abs() < 1e-5);
    }

    /// A line with nothing damaged is not touched, even where a candidate
    /// is close; and a candidate under the floor is never taken.
    #[test]
    fn sound_lines_and_unlikely_candidates_are_left_alone() {
        let vocab = vocab();
        let mut steps = vec![vec![('$', 0.51), ('S', 0.49)]];
        steps.extend(sure("8.50"));
        let (line, repaired) = repair(&vocab, &steps);
        assert_eq!((line.text.as_str(), repaired), ("$8.50", 0));

        let mut steps = vec![vec![('S', 0.995), ('$', 0.005)]];
        steps.extend(sure("8.50"));
        let (line, repaired) = repair(&vocab, &steps);
        assert_eq!((line.text.as_str(), repaired), ("S8.50", 0));
    }

    /// A letter among a date's digits: the digit the recognizer also
    /// considered replaces it.
    #[test]
    fn a_letter_in_a_date_is_repaired() {
        let vocab = vocab();
        let mut steps = sure("06/12/2");
        steps.push(vec![('O', 0.7), ('0', 0.25)]);
        steps.extend(sure("26"));
        let (line, repaired) = repair(&vocab, &steps);
        assert_eq!((line.text.as_str(), repaired), ("06/12/2026", 1));
    }
}

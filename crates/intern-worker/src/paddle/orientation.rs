//! Which way up a page is read, decided from what each pass found.
//!
//! The orientation classifier names a turn before anything is read, and
//! nearly always rightly: a page goes through one recognition pass. What it
//! cannot be trusted with alone is a page it is wrong about, so every
//! reading still has to convince on its own terms - confident, and the
//! right way round - before the search stops.
//!
//! PP-OCR adds a failure Tesseract does not have. A line taller than it is
//! wide is cut out turned a quarter, which is how vertical text is read, so
//! a page lying on its side comes back as confident text: every line read,
//! in columns, in the wrong order, with boxes that run down the page. Its
//! confidence cannot catch it; the shape of what the detector found can.

use crate::extract::{OcrResult, better_reading, upright_reading_is_accepted};

use super::geometry::Quad;

/// One recognition pass, whether the detector found the page lying on its
/// side at that turn, and how many lines it found there - read or not.
#[derive(Clone, Debug, PartialEq)]
pub struct PassReading {
    pub result: OcrResult,
    pub sideways: bool,
    pub detected: usize,
}

/// Whether most of the text the detector found runs down the page: more
/// than half the total length of the lines found is in lines at least half
/// again as tall as they are wide, and there are at least two of them.
///
/// A form with a rotated column header or a spine label has one or two
/// vertical lines among many horizontal ones; a page on its side has
/// nothing else.
pub fn mostly_vertical(quads: &[Quad]) -> bool {
    let (mut vertical, mut total, mut count) = (0.0_f32, 0.0_f32, 0_usize);
    for quad in quads {
        let (width, height) = (quad.width(), quad.height());
        let length = width.max(height);
        total += length;
        if height >= 1.5 * width {
            vertical += length;
            count += 1;
        }
    }
    count >= 2 && vertical > total / 2.0
}

/// Whether a pass ends the search: read the right way round and
/// confidently, or nothing to read.
///
/// A pass that read nothing is proof of a blank page only where nothing
/// would be read at any turn: the detector found no lines, or the pass was
/// upright, as the page is nearly always scanned. Lines read at the wrong
/// turn come back below the score a line is kept at, so a turned pass that
/// found lines but read none of them says only that the turn was wrong.
pub fn pass_is_final(reading: &PassReading) -> bool {
    let blank = reading.result.text.trim().is_empty()
        && (reading.detected == 0 || reading.result.rotation_degrees % 360 == 0);
    blank || (!reading.sideways && upright_reading_is_accepted(&reading.result))
}

/// Of two passes over the same page, the one to keep.
///
/// A pass that read the page the right way round beats one that read it on
/// its side, unless it is worse by confidence and not confident either: a
/// quarter turn that reads as gibberish is not an improvement on a
/// sideways reading of the right text. Otherwise the confidence and density
/// rule the Tesseract path uses decides.
pub fn better_pass(incumbent: PassReading, challenger: PassReading) -> PassReading {
    match (incumbent.sideways, challenger.sideways) {
        (true, false) => {
            let words = |reading: &OcrResult| reading.text.split_whitespace().count();
            let dense = words(&challenger.result) * 2 >= words(&incumbent.result);
            let convincing = upright_reading_is_accepted(&challenger.result)
                || challenger.result.mean_confidence > incumbent.result.mean_confidence;
            if dense && convincing {
                challenger
            } else {
                incumbent
            }
        }
        (false, true) => incumbent,
        _ => {
            let sideways = incumbent.sideways;
            let kept = better_reading(incumbent.result.clone(), challenger.result.clone());
            if kept == incumbent.result {
                incumbent
            } else {
                PassReading {
                    result: kept,
                    sideways,
                    detected: challenger.detected,
                }
            }
        }
    }
}

/// The turns tried after `first`, in order, without repeating it.
///
/// A pass that found the page on its side is a quarter turn out: both
/// quarter turns from it come next, the one a top-to-bottom line needs
/// first, since that is the way a sideways reading came out confident. A
/// `suggested` turn (the classifier's, when the page was read upright
/// first) goes ahead of the rest. Otherwise the order is Tesseract's search:
/// upright, a quarter turn back, a quarter forward, upside down.
pub fn search_order(first: u16, first_sideways: bool, suggested: Option<u16>) -> Vec<u16> {
    let mut order = Vec::with_capacity(4);
    if let Some(turn) = suggested {
        order.push(turn % 360);
    }
    if first_sideways {
        order.extend([(first + 270) % 360, (first + 90) % 360, (first + 180) % 360]);
    }
    order.extend([0, 270, 90, 180]);
    let mut seen = vec![first % 360];
    order.retain(|turn| {
        if seen.contains(turn) {
            false
        } else {
            seen.push(*turn);
            true
        }
    });
    order
}

/// Reads the page at `first`, and then, while no pass has convinced, at
/// each turn [`search_order`] names, keeping the [`better_pass`]. `read`
/// performs one pass at a clockwise turn.
pub fn search_orientation<E>(
    first: u16,
    suggested: Option<u16>,
    mut read: impl FnMut(u16) -> Result<PassReading, E>,
) -> Result<OcrResult, E> {
    let mut best = read(first)?;
    if pass_is_final(&best) {
        return Ok(best.result);
    }
    for turn in search_order(first, best.sideways, suggested) {
        let attempt = read(turn)?;
        best = better_pass(best, attempt);
        if pass_is_final(&best) {
            break;
        }
    }
    Ok(best.result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paddle::geometry::Point;

    fn quad(x: f32, y: f32, width: f32, height: f32) -> Quad {
        Quad([
            Point::new(x, y),
            Point::new(x + width, y),
            Point::new(x + width, y + height),
            Point::new(x, y + height),
        ])
    }

    fn pass(text: &str, confidence: f32, sideways: bool, rotation: u16) -> PassReading {
        PassReading {
            result: OcrResult::new(text, confidence).with_rotation(rotation),
            sideways,
            detected: text.lines().count(),
        }
    }

    /// The classifier names a wrong turn, and the lines read there come back
    /// below the score a line is kept at: nothing is read, but lines were
    /// found, so the search goes on and reads the page upright.
    #[test]
    fn a_turned_pass_that_reads_none_of_the_lines_it_found_is_not_a_blank_page() {
        let mut turns = Vec::new();
        let result = search_orientation::<()>(180, None, |turn| {
            turns.push(turn);
            Ok(if turn == 0 {
                pass(PROSE, 97.0, false, turn)
            } else {
                PassReading {
                    detected: 12,
                    ..pass("", 0.0, false, turn)
                }
            })
        })
        .unwrap();
        assert_eq!(turns, vec![180, 0]);
        assert_eq!(result.text, PROSE);

        // Where the detector found nothing at all, the page is blank at any
        // turn: one pass.
        let mut turns = Vec::new();
        let result = search_orientation::<()>(180, None, |turn| {
            turns.push(turn);
            Ok(pass("", 0.0, false, turn))
        })
        .unwrap();
        assert_eq!(turns, vec![180]);
        assert!(result.text.is_empty());

        // And an upright pass that read nothing still ends the search.
        let upright = PassReading {
            detected: 3,
            ..pass("", 0.0, false, 0)
        };
        assert!(pass_is_final(&upright));
    }

    const PROSE: &str = "Invoice Date: April 14, 2026 Bill To: Larkspur Bistro LLC";

    #[test]
    fn a_page_on_its_side_is_mostly_vertical_and_a_stray_label_is_not() {
        let columns: Vec<Quad> = (0..4)
            .map(|column| quad(100.0 + 40.0 * column as f32, 50.0, 25.0, 600.0))
            .collect();
        assert!(mostly_vertical(&columns));
        let mut page: Vec<Quad> = (0..20)
            .map(|row| quad(100.0, 100.0 + 50.0 * row as f32, 900.0, 30.0))
            .collect();
        page.push(quad(20.0, 100.0, 30.0, 400.0));
        page.push(quad(1100.0, 100.0, 30.0, 300.0));
        assert!(!mostly_vertical(&page));
        // One tall line alone is a label, however long.
        assert!(!mostly_vertical(&[quad(0.0, 0.0, 20.0, 900.0)]));
        assert!(!mostly_vertical(&[]));
    }

    #[test]
    fn the_classifier_turn_that_reads_well_is_the_only_pass() {
        let mut turns = Vec::new();
        let result = search_orientation::<()>(270, None, |turn| {
            turns.push(turn);
            Ok(pass(PROSE, 99.0, false, turn))
        })
        .unwrap();
        assert_eq!(turns, vec![270]);
        assert_eq!(result.rotation_degrees, 270);
    }

    #[test]
    fn a_confident_sideways_reading_is_turned_a_quarter() {
        // At no turn the page reads on its side, confidently; a quarter
        // turn back reads it upright.
        let mut turns = Vec::new();
        let result = search_orientation::<()>(0, None, |turn| {
            turns.push(turn);
            Ok(match turn {
                0 => pass(PROSE, 97.0, true, 0),
                270 => pass(PROSE, 98.0, false, 270),
                _ => pass("x7 ;r q", 30.0, false, turn),
            })
        })
        .unwrap();
        assert_eq!(turns, vec![0, 270]);
        assert_eq!(result.rotation_degrees, 270);
    }

    #[test]
    fn gibberish_the_right_way_round_does_not_beat_a_sideways_reading() {
        let result = search_orientation::<()>(0, None, |turn| {
            Ok(match turn {
                0 => pass(PROSE, 90.0, true, 0),
                _ => pass("x7 ;r q zz 0O lI", 35.0, false, turn),
            })
        })
        .unwrap();
        assert_eq!(result.rotation_degrees, 0);
    }

    #[test]
    fn a_wrong_classifier_turn_is_caught_by_confidence() {
        // The classifier said 180; the page is upright.
        let mut turns = Vec::new();
        let result = search_orientation::<()>(180, None, |turn| {
            turns.push(turn);
            Ok(match turn {
                0 => pass(PROSE, 99.0, false, 0),
                _ => pass("ɐ ɟ ʇ 9 ∀ 'z", 41.0, false, turn),
            })
        })
        .unwrap();
        assert_eq!(turns, vec![180, 0]);
        assert_eq!(result.rotation_degrees, 0);
    }

    #[test]
    fn a_blank_page_is_read_once() {
        let mut passes = 0;
        search_orientation::<()>(0, None, |turn| {
            passes += 1;
            Ok(pass("", 0.0, false, turn))
        })
        .unwrap();
        assert_eq!(passes, 1);
    }

    #[test]
    fn the_search_order_never_repeats_and_starts_where_it_should() {
        assert_eq!(search_order(0, false, None), vec![270, 90, 180]);
        assert_eq!(search_order(90, false, None), vec![0, 270, 180]);
        assert_eq!(search_order(0, true, None), vec![270, 90, 180]);
        assert_eq!(search_order(90, true, None), vec![0, 180, 270]);
        assert_eq!(search_order(0, false, Some(180)), vec![180, 270, 90]);
        assert_eq!(search_order(0, false, Some(0)), vec![270, 90, 180]);
    }
}

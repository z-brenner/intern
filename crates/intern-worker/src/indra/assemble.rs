//! From detected boxes and their readings to the finished page: gutter
//! cuts, line splits, list markers, reading order, blocks, markdown and
//! tables (spec §8).

use super::types::{DetBox, OrderOutputs, PageReading, RecLine};
use super::vocab::Vocab;

/// `OrtOCR.assemble`: the page `width` x `height` (turned and levelled)
/// from its detected boxes and their readings, asking `order` for the
/// order network's outputs given `[n, 16]` features. `order` is not called
/// for fewer than two lines. Boxes are left on the levelled page; the
/// engine maps them back.
pub fn assemble(
    width: usize,
    height: usize,
    boxes: &[DetBox],
    lines: &[RecLine],
    vocab: &Vocab,
    order: &mut dyn FnMut(&[f32], usize) -> OrderOutputs,
) -> PageReading {
    let _ = (width, height, boxes, lines, vocab, order);
    todo!("spec §8")
}

/// `join_lines(texts)` (spec §8.6).
pub fn join_lines(texts: &[&str]) -> String {
    let _ = texts;
    todo!("spec §8.6")
}

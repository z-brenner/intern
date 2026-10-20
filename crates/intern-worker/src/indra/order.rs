//! Reading order and blocks from the order network's outputs (spec §8.5).

use super::types::{BlockClass, OrderOutputs};

/// `line_features(boxes, wh, texts)`: sixteen features a line, `[n, 16]`.
pub fn line_features(boxes: &[[f64; 4]], width: f64, height: f64, texts: &[String]) -> Vec<f32> {
    let _ = (boxes, width, height, texts);
    todo!("spec §8.5")
}

/// `decode_blocks_order(a, same, boxes, cls)`: the lines' reading order.
pub fn decode_blocks_order(outputs: &OrderOutputs, boxes: &[[f64; 4]]) -> Vec<usize> {
    let _ = (outputs, boxes);
    todo!("spec §8.5")
}

/// `order_xycut(boxes)`: the order of the given lines by recursive cuts.
pub fn order_xycut(boxes: &[[f64; 4]], indices: &[usize]) -> Vec<usize> {
    let _ = (boxes, indices);
    todo!("spec §8.5")
}

/// `split_blocks(order, same)` with each block's class (spec §8.6): runs of
/// consecutive ordered lines, as indices into the original lines.
pub fn split_blocks(order: &[usize], outputs: &OrderOutputs) -> Vec<(BlockClass, Vec<usize>)> {
    let _ = (order, outputs);
    todo!("spec §8.6")
}

//! The recognizer's character n-gram language model: Katz back-off over
//! packed keys, read from the reference package's `.npz` (spec §7.2, §7.3).

use std::path::Path;

use super::npy::NpyError;

/// A back-off n-gram model over recognizer class ids.
#[derive(Clone, Debug)]
pub struct NgramLm {
    pub order: usize,
    pub vocab_size: usize,
    pub bits: u32,
    /// The largest log-probability a token can be given; for pruning.
    pub max_logp: f64,
}

impl NgramLm {
    pub fn load(path: &Path) -> Result<Self, NpyError> {
        let _ = path;
        todo!("spec §7.2")
    }

    /// `logp_keys` for one key `(history << bits) | (class + 1)` (spec §7.3).
    pub fn logp(&self, key: u64) -> f64 {
        let _ = key;
        todo!("spec §7.3")
    }

    /// The mask of a rolling history of `order - 1` tokens.
    pub fn history_mask(&self) -> u64 {
        todo!("spec §7.4")
    }
}

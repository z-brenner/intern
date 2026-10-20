//! `meta.json`: the vocabularies and sizes the networks were exported with.

use std::path::Path;

use super::vocab::Vocab;

/// The parts of `meta.json` the Latin pipeline reads.
#[derive(Clone, Debug)]
pub struct Meta {
    pub det_long_side: usize,
    pub rec_height: usize,
    pub rec_window: usize,
    pub vocab: Vocab,
}

impl Meta {
    pub fn load(path: &Path) -> Result<Self, String> {
        let _ = path;
        todo!()
    }
}

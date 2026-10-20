//! Indra OCR: Datalab's compact document OCR networks run through ONNX
//! Runtime, as the primary engine for scans when its runtime and models are
//! installed, ahead of PP-OCR and Tesseract.
//!
//! A page is read in five stages, each a faithful port of the reference
//! `indra-ocr` 1.1.2 package (github.com/datalab-to/indra at
//! `0e96b93c6810f17cf8352acbb4487cc9e45b66d5`, models at Hugging Face
//! `datalab-to/indra-ocr` commit `a1283c4b42a273c09480ad50e71461daff50fcc7`):
//!
//! 1. [`page`]: the page's skew is found by a projection-profile search and
//!    its quarter turn by the orientation network; the page is turned and
//!    levelled.
//! 2. [`det`]: the line detector marks every text line's core and the
//!    distances to its top and bottom.
//! 3. [`rec`]: each line is cut out and read by the recognizer, which
//!    reports a class and a probability for every time step; the lines it is
//!    unsure of are searched again under [`lm`]'s character language model
//!    by [`beam`].
//! 4. [`order`]: the reading-order network scores every pair of lines for
//!    which comes first and whether they share a block, and labels blocks.
//! 5. [`assemble`]: lines are split at gutters, list markers joined to their
//!    items, blocks formed and put in order, and the page's text written.
//!
//! Everything here except [`engine`] is plain Rust over pixels and
//! probabilities and is tested on every platform. The engine, which loads
//! ONNX Runtime from the runtime directory, is built only with the
//! `onnx-ocr` feature.

use std::path::{Path, PathBuf};

pub mod assemble;
pub mod beam;
pub mod det;
#[cfg(feature = "onnx-ocr")]
mod engine;
pub mod imgops;
pub mod lm;
pub mod meta;
pub mod npy;
pub mod order;
pub mod page;
pub mod rec;
pub mod types;
pub mod vocab;

#[cfg(feature = "onnx-ocr")]
pub use self::engine::IndraOcr;

/// The directory under the runtime directory that holds Indra's files,
/// beside PP-OCR's `ocr-models`.
pub const MODEL_DIRECTORY: &str = "indra-models";
/// The line detector.
pub const DETECTION_MODEL: &str = "det.onnx";
/// The page orientation classifier.
pub const ORIENTATION_MODEL: &str = "orient.onnx";
/// The Latin, Cyrillic and Greek recognizer.
pub const RECOGNITION_MODEL: &str = "rec.onnx";
/// The reading-order and layout network.
pub const ORDER_MODEL: &str = "order.onnx";
/// The recognizer's character language model.
pub const LANGUAGE_MODEL: &str = "lm.npz";
/// Vocabularies and sizes.
pub const META: &str = "meta.json";

/// Where Indra's files are expected in a runtime directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndraAssets {
    pub runtime_library: PathBuf,
    pub detection_model: PathBuf,
    pub orientation_model: PathBuf,
    pub recognition_model: PathBuf,
    pub order_model: PathBuf,
    pub language_model: PathBuf,
    pub meta: PathBuf,
}

impl IndraAssets {
    pub fn in_directory(runtime: &Path) -> Self {
        let models = runtime.join(MODEL_DIRECTORY);
        Self {
            runtime_library: runtime.join(crate::paddle::RUNTIME_LIBRARY),
            detection_model: models.join(DETECTION_MODEL),
            orientation_model: models.join(ORIENTATION_MODEL),
            recognition_model: models.join(RECOGNITION_MODEL),
            order_model: models.join(ORDER_MODEL),
            language_model: models.join(LANGUAGE_MODEL),
            meta: models.join(META),
        }
    }

    fn files(&self) -> [&PathBuf; 7] {
        [
            &self.runtime_library,
            &self.detection_model,
            &self.orientation_model,
            &self.recognition_model,
            &self.order_model,
            &self.language_model,
            &self.meta,
        ]
    }

    /// Whether every file Indra reads a page with is there.
    pub fn present(&self) -> bool {
        self.files().iter().all(|path| path.is_file())
    }

    /// The first of Indra's own files that is missing when some of them are
    /// installed, or nothing when all of them are or none is. The runtime
    /// library is shared with PP-OCR and does not count as one of Indra's.
    pub fn partly_missing(&self) -> Option<&Path> {
        let files = &self.files()[1..];
        let missing = files.iter().find(|path| !path.is_file())?;
        files
            .iter()
            .any(|path| path.is_file())
            .then_some(missing.as_path())
    }
}

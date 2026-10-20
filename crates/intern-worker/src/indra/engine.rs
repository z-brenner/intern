//! The Indra engine: ONNX Runtime sessions for the networks, the page
//! pipeline that drives the stages, and the [`OcrBackend`] the worker
//! reads scans with.

use crate::extract::{CancellationToken, ExtractionError, OcrBackend, OcrResult, RenderedPage};

use super::IndraAssets;

/// Reads scans with Indra's networks.
pub struct IndraOcr {
    _assets: IndraAssets,
}

impl IndraOcr {
    pub fn new(assets: IndraAssets) -> Result<Self, ExtractionError> {
        let _ = assets;
        todo!()
    }
}

impl OcrBackend for IndraOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let _ = (page, cancel);
        todo!()
    }
}

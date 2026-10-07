//! Where one document's extraction time went.
//!
//! A scan that takes a minute to read is a scan whose minute has to be
//! accounted for before it can be made shorter: rendering, encoding pages for
//! Tesseract, and waiting on Tesseract are different costs with different
//! remedies. Every figure here is wall time measured around the work it
//! names, cheap enough to take on every document, and none of it changes
//! what is read.

use std::time::Instant;

use serde::{Deserialize, Serialize};

/// How long each stage of one extraction took, in microseconds, and how much
/// OCR it needed.
///
/// The stages do not overlap except where a field says it is part of
/// another, so their sum is close to `total_micros`; what is left over is the
/// page loop's own bookkeeping.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractionTimings {
    /// The whole extraction in the worker, the snapshot included.
    pub total_micros: u64,
    /// Copying the file into the private workspace it is read from.
    pub snapshot_micros: u64,
    /// Reading the format: binding PDFium (on the first PDF a worker
    /// process reads), loading a PDF and its pages' text, or the whole of
    /// every other reader less the stages below that it reported.
    pub parse_micros: u64,
    /// Measuring a PDF page's image coverage and deciding whether it is a
    /// scan.
    pub analysis_micros: u64,
    /// PDFium rasterising pages: the ones that go to OCR, and the rare page
    /// rendered only to be the page image.
    pub render_micros: u64,
    /// Decoding a standalone image file, scaling it to the page cap, and
    /// turning it upright.
    pub image_decode_micros: u64,
    /// All OCR for the document: every recognition and orientation pass.
    pub ocr_micros: u64,
    /// Of `ocr_micros`: turning pages grey and encoding the PNGs Tesseract
    /// is handed.
    pub ocr_encode_micros: u64,
    /// Of `ocr_micros`: waiting on Tesseract processes.
    pub ocr_engine_micros: u64,
    /// Building the optional page image.
    pub vision_micros: u64,
    /// Pages read by OCR.
    pub ocr_pages: u32,
    /// Recognition passes, in every orientation tried.
    pub ocr_passes: u32,
    /// Orientation-detection passes.
    pub orientation_passes: u32,
    /// Pixels PDFium rendered, summed over the pages counted in
    /// `render_micros`.
    pub rendered_pixels: u64,
}

impl ExtractionTimings {
    /// Records a reader that does not measure its own parse: everything it
    /// spent, `reader_micros`, less the stages it did report, was reading
    /// the format.
    ///
    /// Only the PDF reader separates reading a page from measuring it, so
    /// it alone reports parse time as it goes; every other reader reads in
    /// one call, and the remainder is the honest figure for it.
    pub fn attribute_remainder_to_parse(&mut self, reader_micros: u64) {
        let reported = self
            .analysis_micros
            .saturating_add(self.render_micros)
            .saturating_add(self.image_decode_micros)
            .saturating_add(self.ocr_micros)
            .saturating_add(self.vision_micros);
        self.parse_micros = reader_micros.saturating_sub(reported);
    }
}

/// Microseconds since `started`, saturating rather than wrapping on a clock
/// that has run for half a million years.
pub fn micros_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reader_without_its_own_parse_figure_is_charged_the_remainder() {
        let mut timings = ExtractionTimings {
            image_decode_micros: 300,
            ocr_micros: 5_000,
            vision_micros: 200,
            ..ExtractionTimings::default()
        };

        timings.attribute_remainder_to_parse(5_650);

        assert_eq!(timings.parse_micros, 150);
        // A clock that disagrees with itself is never a negative parse.
        timings.attribute_remainder_to_parse(10);
        assert_eq!(timings.parse_micros, 0);
    }

    #[test]
    fn timings_serialize_with_the_field_names_the_host_reads() {
        let value = serde_json::to_value(ExtractionTimings {
            total_micros: 1,
            ocr_pages: 2,
            ..ExtractionTimings::default()
        })
        .unwrap();

        assert_eq!(value["total_micros"], 1);
        assert_eq!(value["ocr_pages"], 2);
        assert_eq!(value["ocr_engine_micros"], 0);
        assert_eq!(value.as_object().unwrap().len(), 14);
    }
}

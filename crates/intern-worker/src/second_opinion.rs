//! A second engine's reading of a page the first engine was not sure of.
//!
//! Indra OCR reads scans first. A page it reads with a confidence under
//! [`CONFIDENT_READING`] - the bar under which a page earns a
//! low-confidence warning and may go to vision - is read again by PP-OCR
//! when PP-OCR is installed, and the better of the two readings is kept by
//! the rule every other choice between readings uses ([`better_reading`]):
//! the more confident, if it found about as many words. Measured on
//! InternBench's scans made at a third of their resolution, Indra read two
//! pages under the bar where PP-OCR read them better, and the pair found
//! more dates, names and identifiers than either engine alone; at full
//! resolution no page is under the bar and nothing changes
//! (`docs/ocr.md`).
//!
//! The second engine is loaded the first time a page needs it, not before:
//! most documents never do, and its sessions cost as much memory as the
//! first engine's. A second engine that cannot be loaded is reported once
//! and not asked again; the first reading stands.

use std::sync::OnceLock;

use crate::extract::{
    CONFIDENT_READING, CancellationToken, ExtractionError, OcrBackend, OcrResult, RenderedPage,
    better_reading,
};

/// An OCR engine any number of pages can be read with at once.
pub type SharedOcr = Box<dyn OcrBackend + Send + Sync>;

/// Builds the second engine.
pub type Loader = Box<dyn Fn() -> Result<SharedOcr, ExtractionError> + Send + Sync>;

/// Told once when the second engine could not be loaded.
pub type FailureReport = Box<dyn Fn(&ExtractionError) + Send + Sync>;

/// Reads with `first`, and with a second engine the pages `first` is not
/// sure of (see the module documentation).
pub struct SecondOpinion {
    first: SharedOcr,
    load: Loader,
    report: FailureReport,
    second: OnceLock<Option<SharedOcr>>,
}

impl SecondOpinion {
    pub fn new(first: SharedOcr, load: Loader, report: FailureReport) -> Self {
        Self {
            first,
            load,
            report,
            second: OnceLock::new(),
        }
    }

    fn second(&self) -> Option<&SharedOcr> {
        self.second
            .get_or_init(|| match (self.load)() {
                Ok(engine) => Some(engine),
                Err(error) => {
                    (self.report)(&error);
                    None
                }
            })
            .as_ref()
    }
}

/// Whether a first reading is worth a second engine's: not confident, and
/// not blank - a blank page is blank to every engine, and the back of every
/// scanned sheet would otherwise load and run the second.
pub fn wants_second_opinion(reading: &OcrResult) -> bool {
    reading.mean_confidence < CONFIDENT_READING && !reading.text.trim().is_empty()
}

impl OcrBackend for SecondOpinion {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let first = self.first.recognize(page, cancel)?;
        if !wants_second_opinion(&first) {
            return Ok(first);
        }
        let Some(second) = self.second() else {
            return Ok(first);
        };
        match second.recognize(page, cancel) {
            Ok(reading) => Ok(better_reading(first, reading)),
            // A canceled or timed-out document stops; a second engine that
            // failed on this page leaves the first reading.
            Err(_) => cancel.check().map(|()| first),
        }
    }

    fn concurrency(&self) -> usize {
        self.first.concurrency()
    }

    fn release(&self) {
        self.first.release();
        if let Some(Some(second)) = self.second.get() {
            second.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// Reads every page as `text` at `confidence`, and counts its reads and
    /// releases.
    struct Fixed {
        text: &'static str,
        confidence: f32,
        reads: Arc<AtomicUsize>,
        releases: Arc<AtomicUsize>,
    }

    impl OcrBackend for Fixed {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, ExtractionError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok(OcrResult::new(self.text, self.confidence))
        }

        fn concurrency(&self) -> usize {
            2
        }

        fn release(&self) {
            self.releases.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct Counts {
        reads: Arc<AtomicUsize>,
        releases: Arc<AtomicUsize>,
    }

    fn fixed(text: &'static str, confidence: f32) -> (SharedOcr, Counts) {
        let (reads, releases) = (Arc::default(), Arc::default());
        let engine = Fixed {
            text,
            confidence,
            reads: Arc::clone(&reads),
            releases: Arc::clone(&releases),
        };
        (Box::new(engine), Counts { reads, releases })
    }

    fn page() -> RenderedPage {
        RenderedPage::new(0, image::DynamicImage::new_luma8(4, 4))
    }

    /// The first engine, and a second built at most once, counting how many
    /// times it was built.
    fn pair(
        first: (&'static str, f32),
        second: (&'static str, f32),
    ) -> (SecondOpinion, Counts, Counts, Arc<AtomicUsize>) {
        let (first, first_counts) = fixed(first.0, first.1);
        let (second, second_counts) = fixed(second.0, second.1);
        let second = std::sync::Mutex::new(Some(second));
        let loads = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&loads);
        let engine = SecondOpinion::new(
            first,
            Box::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
                Ok(second.lock().unwrap().take().expect("built once"))
            }),
            Box::new(|_| panic!("the second engine loads")),
        );
        (engine, first_counts, second_counts, loads)
    }

    #[test]
    fn a_confident_reading_is_not_read_again() {
        let (engine, first, second, loads) =
            pair(("Invoice 2026-04-01 Harbourline", 92.0), ("other", 99.0));
        let cancel = CancellationToken::new();
        let reading = engine.recognize(&page(), &cancel).unwrap();
        assert_eq!(reading.text, "Invoice 2026-04-01 Harbourline");
        assert_eq!(first.reads.load(Ordering::SeqCst), 1);
        assert_eq!(second.reads.load(Ordering::SeqCst), 0);
        // Never needed, never loaded.
        assert_eq!(loads.load(Ordering::SeqCst), 0);
        assert_eq!(engine.concurrency(), 2);
        engine.release();
        assert_eq!(first.releases.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn an_unsure_reading_keeps_the_better_of_the_two() {
        let (engine, _, second, loads) = pair(
            ("Remittanoe advioe Sablewood", 40.0),
            ("Remittance advice Sablewood", 80.0),
        );
        let cancel = CancellationToken::new();
        for _ in 0..3 {
            let reading = engine.recognize(&page(), &cancel).unwrap();
            assert_eq!(reading.text, "Remittance advice Sablewood");
        }
        assert_eq!(second.reads.load(Ordering::SeqCst), 3);
        assert_eq!(loads.load(Ordering::SeqCst), 1);
        engine.release();
        assert_eq!(second.releases.load(Ordering::SeqCst), 1);

        // A second reading less sure than the first, or far sparser, loses.
        let (engine, ..) = pair(("Remittance advice Sablewood", 60.0), ("Remittance", 50.0));
        let reading = engine.recognize(&page(), &cancel).unwrap();
        assert_eq!(reading.text, "Remittance advice Sablewood");
        let (engine, ..) = pair(("one two three four five six", 60.0), ("one", 99.0));
        let reading = engine.recognize(&page(), &cancel).unwrap();
        assert_eq!(reading.text, "one two three four five six");
    }

    #[test]
    fn a_blank_page_is_not_read_again() {
        let (engine, _, second, loads) = pair(("  ", 0.0), ("noise", 80.0));
        let reading = engine
            .recognize(&page(), &CancellationToken::new())
            .unwrap();
        assert_eq!(reading.text.trim(), "");
        assert_eq!(second.reads.load(Ordering::SeqCst), 0);
        assert_eq!(loads.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_second_engine_that_does_not_load_is_reported_once_and_left() {
        let (first, _) = fixed("Remittanoe advioe Sablewood", 40.0);
        let reports = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&reports);
        let engine = SecondOpinion::new(
            first,
            Box::new(|| Err(ExtractionError::native_assets_missing("no PP-OCR"))),
            Box::new(move |_| {
                counted.fetch_add(1, Ordering::SeqCst);
            }),
        );
        let cancel = CancellationToken::new();
        for _ in 0..2 {
            let reading = engine.recognize(&page(), &cancel).unwrap();
            assert_eq!(reading.text, "Remittanoe advioe Sablewood");
        }
        assert_eq!(reports.load(Ordering::SeqCst), 1);
        engine.release();
    }
}

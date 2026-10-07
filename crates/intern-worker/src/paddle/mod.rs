//! PP-OCR: text detection and recognition networks run through ONNX
//! Runtime, as the primary OCR engine when its runtime and models are
//! installed.
//!
//! A page is read in two stages. The detector marks where text lines are,
//! and each line is cut out along its own slope and handed to the
//! recognizer, which reads it in one pass with a probability for every
//! character. Those probabilities are what the selective second pass
//! works from: the few lines that hold a date, an amount, an identifier or
//! a name and were read with an uncertain character are cut out again,
//! preprocessed differently, and re-read, and the reading that both parses
//! and is most confident is kept.
//!
//! Everything here except [`engine`] is plain Rust over pixels and
//! probabilities and is tested on every platform. The engine itself, which
//! loads ONNX Runtime from the runtime directory, is built only with the
//! `onnx-ocr` feature; without it, or without the runtime and models in
//! the runtime directory, the worker reads scans with Tesseract exactly as
//! it always has.

use std::path::{Path, PathBuf};

pub mod ctc;
pub mod db;
#[cfg(feature = "onnx-ocr")]
mod engine;
pub mod geometry;
pub mod orientation;
pub mod reocr;
pub mod skew;
pub mod tensor;

use crate::extract::OcrLine;

use self::db::DbParams;
use self::tensor::DetectionSize;

#[cfg(feature = "onnx-ocr")]
pub use self::engine::PaddleOcr;

/// The directory under the runtime directory that holds the models.
pub const MODEL_DIRECTORY: &str = "ocr-models";
/// The text detection network.
pub const DETECTION_MODEL: &str = "text-detection.onnx";
/// The English text recognition network. Its character list is
/// [`RECOGNITION_DICTIONARY`], built in.
pub const RECOGNITION_MODEL: &str = "text-recognition.onnx";
/// The page orientation classifier. Optional: without it a page is read
/// upright first and turned only when that reading is unconvincing.
pub const ORIENTATION_MODEL: &str = "page-orientation.onnx";

/// The recognizer's characters, in class order, from the configuration
/// published with the pinned recognition model. The model's output has a
/// class for each, plus the CTC blank before them and a space after; the
/// engine refuses a model whose output does not.
pub const RECOGNITION_DICTIONARY: &str = include_str!("recognition-dictionary.txt");

/// The ONNX Runtime library's file name on this platform.
#[cfg(windows)]
pub const RUNTIME_LIBRARY: &str = "onnxruntime.dll";
#[cfg(target_os = "macos")]
pub const RUNTIME_LIBRARY: &str = "libonnxruntime.dylib";
#[cfg(not(any(windows, target_os = "macos")))]
pub const RUNTIME_LIBRARY: &str = "libonnxruntime.so";

/// Where PP-OCR's files are expected in a runtime directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaddleAssets {
    pub runtime_library: PathBuf,
    pub detection_model: PathBuf,
    pub recognition_model: PathBuf,
    pub orientation_model: PathBuf,
}

impl PaddleAssets {
    pub fn in_directory(runtime: &Path) -> Self {
        let models = runtime.join(MODEL_DIRECTORY);
        Self {
            runtime_library: runtime.join(RUNTIME_LIBRARY),
            detection_model: models.join(DETECTION_MODEL),
            recognition_model: models.join(RECOGNITION_MODEL),
            orientation_model: models.join(ORIENTATION_MODEL),
        }
    }

    /// Whether the files PP-OCR cannot read a page without are all there:
    /// the runtime, the detector and the recognizer.
    pub fn present(&self) -> bool {
        [
            &self.runtime_library,
            &self.detection_model,
            &self.recognition_model,
        ]
        .iter()
        .all(|path| path.is_file())
    }
}

/// How a page's orientation is found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrientationStrategy {
    /// Ask the orientation classifier first and read the page the way it
    /// says; turn it further only if that reading is unconvincing.
    ClassifierFirst,
    /// Read the page as it came; ask the classifier only when that reading
    /// is unconvincing, as the Tesseract path asks its detector.
    UprightFirst,
}

/// How the lines worth a second look are re-read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RereadPreprocessing {
    /// Cut out with more margin and the contrast stretched.
    PaddedContrast,
    /// Cut out with more margin and made black and white.
    PaddedBinarized,
}

/// The selective second pass.
#[derive(Clone, Debug, PartialEq)]
pub struct RereadSettings {
    /// A line whose least certain character is below this, and that holds
    /// a critical field, is re-read.
    pub min_char_probability: f32,
    /// At most this many lines per page.
    pub max_lines: usize,
    /// The preprocessings each chosen line is re-read with.
    pub preprocessings: Vec<RereadPreprocessing>,
}

impl Default for RereadSettings {
    fn default() -> Self {
        Self {
            min_char_probability: reocr::DEFAULT_MIN_CHAR_PROBABILITY,
            max_lines: reocr::DEFAULT_MAX_LINES,
            preprocessings: vec![
                RereadPreprocessing::PaddedContrast,
                RereadPreprocessing::PaddedBinarized,
            ],
        }
    }
}

/// Everything the engine can be tuned by. [`PaddleConfig::default`] is what
/// ships; the rest exists so the benchmark can measure the alternatives.
#[derive(Clone, Debug, PartialEq)]
pub struct PaddleConfig {
    pub detection_size: DetectionSize,
    pub db: DbParams,
    /// Threads each network may use for one page.
    pub intra_threads: usize,
    /// Pages read at once; each has its own sessions.
    pub workers: usize,
    pub orientation: OrientationStrategy,
    /// The classifier's probability a turn needs before the page is turned
    /// on its word alone.
    pub orientation_min_probability: f32,
    /// Lines read with a mean probability below this are dropped as noise.
    pub drop_score: f32,
    /// Skew, in degrees, past which the page is levelled and detected
    /// again. Below it the lines are cut out along their own slopes and
    /// only their reported boxes are levelled.
    pub redetect_skew_degrees: f32,
    pub reread: Option<RereadSettings>,
    /// Lines recognized in one batch.
    pub recognition_batch: usize,
}

impl Default for PaddleConfig {
    fn default() -> Self {
        let threads = default_thread_budget();
        Self {
            detection_size: DetectionSize::LongSide { max: 1600, min: 0 },
            db: DbParams::MOBILE,
            intra_threads: threads.intra_threads,
            workers: threads.workers,
            orientation: OrientationStrategy::ClassifierFirst,
            orientation_min_probability: 0.5,
            drop_score: 0.5,
            redetect_skew_degrees: 5.0,
            reread: Some(RereadSettings::default()),
            recognition_batch: 6,
        }
    }
}

/// How the OCR threads are shared out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadBudget {
    /// Pages read at once.
    pub workers: usize,
    /// Threads each of them uses.
    pub intra_threads: usize,
}

/// Threads one page's networks may use. Two keeps a page's latency close
/// to what four give (the networks are small and their layers short) and
/// leaves the rest for reading another page at the same time.
const INTRA_THREADS: usize = 2;

/// The OCR thread budget for a machine with `logical_cores`: at most half
/// of them, so Windows and the model server stay responsive, in workers of
/// [`INTRA_THREADS`] threads each, and never fewer than one worker.
pub fn thread_budget(logical_cores: usize) -> ThreadBudget {
    let budget = (logical_cores / 2).max(1);
    let intra_threads = INTRA_THREADS.min(budget);
    ThreadBudget {
        workers: (budget / intra_threads).max(1),
        intra_threads,
    }
}

/// [`thread_budget`] for this machine.
pub fn default_thread_budget() -> ThreadBudget {
    thread_budget(
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(2),
    )
}

/// A line as read, before it is reported: its text, where it sits in the
/// reported frame, and the probability of each character.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedLine {
    pub text: String,
    /// `[x0, y0, x1, y1]` in the upright page's pixels.
    pub bounds: [f32; 4],
    pub char_probabilities: Vec<f32>,
}

impl PlacedLine {
    fn height(&self) -> f32 {
        (self.bounds[3] - self.bounds[1]).max(1.0)
    }

    /// The least certain character of each word, in order. A word is what
    /// lies between spaces; the spaces themselves are not scored.
    fn word_minimums(&self) -> impl Iterator<Item = f32> + '_ {
        let mut minimums = Vec::new();
        let mut current: Option<f32> = None;
        for (character, probability) in self.text.chars().zip(&self.char_probabilities) {
            if character.is_whitespace() {
                minimums.extend(current.take());
            } else {
                current = Some(current.map_or(*probability, |least| least.min(*probability)));
            }
        }
        minimums.extend(current);
        minimums.into_iter()
    }

    /// The line as the reading reports it, with [`confidence_of`] its words.
    pub fn to_ocr_line(&self) -> OcrLine {
        let [x0, y0, x1, y1] = self.bounds.map(|value| value.max(0.0).round() as u32);
        OcrLine {
            text: self.text.clone(),
            bbox: [x0, y0, x1, y1],
            confidence: confidence_of(self.word_minimums())
                .round()
                .clamp(0.0, 100.0) as u8,
        }
    }
}

/// Confidence, 0-100, from the least certain character of each word: their
/// mean, times a hundred. Nothing read is no confidence.
///
/// A word is as trustworthy as its weakest character - "INV-2O417" is wrong
/// however sure the recognizer was of the other eight - which is also how
/// Tesseract's word confidences behave, and why `CONFIDENT_READING` keeps
/// its meaning across the two engines. Measured on InternBench, the
/// research synthetic pages and the same pages turned upside down: real
/// readings score 92-100 (the lowest on faxes the recognizer still read 90%
/// of the words of), and upside-down ones 23-46. The mean over every
/// character, the recognizer's own line score, crowds every real reading
/// into 98-100.
fn confidence_of(word_minimums: impl Iterator<Item = f32>) -> f32 {
    let (sum, count) = word_minimums.fold((0.0_f64, 0_u32), |(sum, count), least| {
        (sum + f64::from(least), count + 1)
    });
    if count == 0 {
        0.0
    } else {
        (sum / f64::from(count) * 100.0) as f32
    }
}

/// The page's text from its lines, in the order given: lines that share a
/// row are joined with a space, rows with a newline, and a gap between rows
/// taller than a line is a blank line - the shape Tesseract's own text
/// output has, which is what distillation finds paragraphs and headings in.
pub fn page_text(lines: &[PlacedLine]) -> String {
    let mut heights: Vec<f32> = lines.iter().map(PlacedLine::height).collect();
    heights.sort_by(f32::total_cmp);
    let typical = heights.get(heights.len() / 2).copied().unwrap_or(1.0);
    let mut text = String::new();
    let mut row: Option<[f32; 4]> = None;
    for line in lines {
        let [_, top, _, bottom] = line.bounds;
        let centre = (top + bottom) / 2.0;
        match row {
            None => {}
            Some([_, row_top, _, row_bottom]) => {
                let row_centre = (row_top + row_bottom) / 2.0;
                let same_row = (centre - row_centre).abs() < typical * 0.5 && top < row_bottom;
                if same_row {
                    text.push(' ');
                } else if top - row_bottom > typical {
                    text.push_str("\n\n");
                } else {
                    text.push('\n');
                }
                if !same_row {
                    row = None;
                }
            }
        }
        text.push_str(&line.text);
        row = Some(match row {
            None => line.bounds,
            Some(bounds) => [
                bounds[0].min(line.bounds[0]),
                bounds[1].min(top),
                bounds[2].max(line.bounds[2]),
                bounds[3].max(bottom),
            ],
        });
    }
    text
}

/// The page's confidence, 0-100: [`confidence_of`] every word on it.
pub fn page_confidence(lines: &[PlacedLine]) -> f32 {
    confidence_of(lines.iter().flat_map(PlacedLine::word_minimums))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(text: &str, bounds: [f32; 4], probability: f32) -> PlacedLine {
        PlacedLine {
            text: text.to_owned(),
            bounds,
            char_probabilities: vec![probability; text.chars().count()],
        }
    }

    #[test]
    fn assets_are_looked_for_in_the_runtime_directory() {
        let directory = tempfile::tempdir().unwrap();
        let assets = PaddleAssets::in_directory(directory.path());
        assert_eq!(
            assets.detection_model,
            directory.path().join("ocr-models/text-detection.onnx")
        );
        assert!(!assets.present());
        std::fs::create_dir(directory.path().join(MODEL_DIRECTORY)).unwrap();
        for path in [
            &assets.runtime_library,
            &assets.detection_model,
            &assets.recognition_model,
        ] {
            std::fs::write(path, b"stand-in").unwrap();
        }
        // The orientation classifier is optional.
        assert!(assets.present());
        std::fs::remove_file(&assets.recognition_model).unwrap();
        assert!(!assets.present());
    }

    #[test]
    fn the_built_in_dictionary_is_the_recognizers() {
        let dictionary = ctc::Dictionary::from_lines(RECOGNITION_DICTIONARY);
        // The pinned recognizer outputs 438 classes.
        assert_eq!(dictionary.classes(), 438);
    }

    #[test]
    fn ocr_threads_stay_within_half_the_machine() {
        assert_eq!(
            thread_budget(4),
            ThreadBudget {
                workers: 1,
                intra_threads: 2
            }
        );
        assert_eq!(
            thread_budget(8),
            ThreadBudget {
                workers: 2,
                intra_threads: 2
            }
        );
        assert_eq!(
            thread_budget(2),
            ThreadBudget {
                workers: 1,
                intra_threads: 1
            }
        );
        assert_eq!(
            thread_budget(1),
            ThreadBudget {
                workers: 1,
                intra_threads: 1
            }
        );
        for cores in 1..64 {
            let budget = thread_budget(cores);
            assert!(
                budget.workers * budget.intra_threads <= (cores / 2).max(1),
                "{cores}"
            );
        }
    }

    #[test]
    fn page_text_joins_rows_and_breaks_paragraphs() {
        let lines = [
            placed("INVOICE", [100.0, 100.0, 300.0, 140.0], 0.99),
            placed("Invoice Date:", [100.0, 200.0, 300.0, 240.0], 0.99),
            // A cell on the same row, a few pixels higher.
            placed("March 4, 2026", [600.0, 197.0, 900.0, 238.0], 0.99),
            placed(
                "Bill To: Larkspur Bistro LLC",
                [100.0, 250.0, 700.0, 290.0],
                0.99,
            ),
            // Far below: a new paragraph.
            placed("Terms: Net 30", [100.0, 400.0, 400.0, 440.0], 0.99),
        ];
        assert_eq!(
            page_text(&lines),
            "INVOICE\n\nInvoice Date: March 4, 2026\nBill To: Larkspur Bistro LLC\n\nTerms: Net 30"
        );
        assert_eq!(page_text(&[]), "");
    }

    #[test]
    fn confidence_is_each_words_weakest_character() {
        let lines = [
            placed("ab", [0.0, 0.0, 10.0, 10.0], 1.0),
            PlacedLine {
                text: "c de".into(),
                bounds: [0.0, 20.0, 10.0, 30.0],
                char_probabilities: vec![0.4, 0.0, 0.9, 0.6],
            },
        ];
        // Words "ab" (1.0), "c" (0.4) and "de" (0.6); the space's 0.0 is
        // no word's.
        assert!((page_confidence(&lines) - 200.0 / 3.0).abs() < 1e-3);
        assert_eq!(page_confidence(&[]), 0.0);
        let line = lines[1].to_ocr_line();
        assert_eq!(line.bbox, [0, 20, 10, 30]);
        assert_eq!(line.confidence, 50);
        assert_eq!(lines[0].to_ocr_line().confidence, 100);
    }
}

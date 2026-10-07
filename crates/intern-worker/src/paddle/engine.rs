//! The PP-OCR engine: ONNX Runtime loaded from the runtime directory,
//! sessions made once and reused, pages read under a fixed thread budget.

use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;
use std::sync::{Condvar, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use image::RgbImage;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::TensorRef;

use crate::extract::{CancellationToken, ExtractionError, OcrBackend, OcrResult, RenderedPage};

use super::ctc::{DecodedLine, Dictionary, greedy_decode};
use super::db::lines_from_map;
use super::geometry::{Point, Quad, reading_order};
use super::orientation::{PassReading, mostly_vertical, pass_is_final, search_orientation};
use super::reocr::{Reading, best_reading, lines_to_reread};
use super::skew::estimate_skew;
use super::tensor::{
    LineCrop, RECOGNITION_HEIGHT, binarize, crop_line, detection_input, detection_input_size,
    line_width, orientation_input, recognition_batch, rotate_page, stretch_contrast,
};
use super::{
    OrientationStrategy, PaddleAssets, PaddleConfig, PlacedLine, RECOGNITION_DICTIONARY,
    RereadPreprocessing, page_confidence, page_text,
};

/// ONNX Runtime binds once per process, to the library it was first given.
///
/// It is loaded from an absolute path in the runtime directory and never
/// by bare name: a bare name would be resolved through the DLL search
/// order, which looks in places anyone can write to.
static RUNTIME: OnceLock<Result<(), String>> = OnceLock::new();

fn load_runtime(library: &Path) -> Result<(), ExtractionError> {
    RUNTIME
        .get_or_init(|| {
            let absolute = std::path::absolute(library).map_err(|error| error.to_string())?;
            let builder = ort::init_from(&absolute).map_err(|error| {
                format!(
                    "ONNX Runtime did not load from {}: {error}",
                    absolute.display()
                )
            })?;
            // Telemetry is on by default in the Windows build. Intern is
            // local-first, and a scan's OCR is no one else's business.
            builder
                .with_name("intern-ocr")
                .with_telemetry(false)
                .commit();
            Ok(())
        })
        .clone()
        .map_err(ExtractionError::native_assets_missing)
}

/// The widest line batch is padded to at least this, as the recognizer was
/// trained: a 48-pixel-high input is never narrower than 320.
const MIN_RECOGNITION_WIDTH: u32 = 320;

/// How far a line is grown on each side, as a fraction of its height, when
/// it is cut out again for a second reading.
const REREAD_PADDING: f32 = 0.15;

/// Skew below this is not worth levelling even the reported boxes for.
const MIN_REPORTED_SKEW_DEGREES: f32 = 0.2;

/// One worker's networks.
struct Sessions {
    detection: Session,
    recognition: Session,
    orientation: Option<Session>,
}

struct PoolState<T> {
    idle: Vec<T>,
    created: usize,
}

/// At most `limit` sets of sessions, made as pages need them and kept.
///
/// A page that arrives while every set is busy waits for one, so however
/// many pages a caller reads at once, the threads OCR uses stay within
/// the budget.
struct SessionPool<T = Sessions> {
    state: Mutex<PoolState<T>>,
    returned: Condvar,
    limit: usize,
}

impl<T> SessionPool<T> {
    fn new(limit: usize) -> Self {
        Self {
            state: Mutex::new(PoolState {
                idle: Vec::new(),
                created: 0,
            }),
            returned: Condvar::new(),
            limit: limit.max(1),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PoolState<T>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn acquire(
        &self,
        build: impl Fn() -> Result<T, ExtractionError>,
        cancel: &CancellationToken,
    ) -> Result<Lease<'_, T>, ExtractionError> {
        let mut may_build = true;
        let mut state = self.lock();
        loop {
            if let Some(sessions) = state.idle.pop() {
                return Ok(Lease {
                    pool: self,
                    sessions: Some(sessions),
                });
            }
            if may_build && state.created < self.limit {
                state.created += 1;
                drop(state);
                match build() {
                    Ok(sessions) => {
                        return Ok(Lease {
                            pool: self,
                            sessions: Some(sessions),
                        });
                    }
                    Err(error) => {
                        state = self.lock();
                        state.created -= 1;
                        self.returned.notify_one();
                        // A second set that will not build - memory is
                        // short - is no reason to fail the page while
                        // another set will come back: wait for it.
                        if state.created == 0 {
                            return Err(error);
                        }
                        may_build = false;
                        continue;
                    }
                }
            }
            state = self
                .returned
                .wait_timeout(state, Duration::from_millis(100))
                .unwrap_or_else(PoisonError::into_inner)
                .0;
            cancel.check()?;
        }
    }

    fn give_back(&self, sessions: T) {
        self.lock().idle.push(sessions);
        self.returned.notify_one();
    }
}

/// A set of sessions out of the pool, returned when dropped.
struct Lease<'a, T = Sessions> {
    pool: &'a SessionPool<T>,
    sessions: Option<T>,
}

impl<T> Lease<'_, T> {
    fn sessions(&mut self) -> &mut T {
        self.sessions
            .as_mut()
            .expect("a lease holds its sessions until dropped")
    }
}

impl<T> Drop for Lease<'_, T> {
    fn drop(&mut self) {
        if let Some(sessions) = self.sessions.take() {
            self.pool.give_back(sessions);
        }
    }
}

/// PP-OCR as an [`OcrBackend`]. Cheap to share between threads: every page
/// read at the same time borrows its own sessions from the pool.
pub struct PaddleOcr {
    assets: PaddleAssets,
    config: PaddleConfig,
    dictionary: Dictionary,
    has_orientation: bool,
    pool: SessionPool,
}

impl std::fmt::Debug for PaddleOcr {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PaddleOcr")
            .field("assets", &self.assets)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

fn model_error(path: &Path, error: impl Display) -> ExtractionError {
    ExtractionError::native_assets_missing(format!(
        "OCR model {} did not load: {error}",
        path.display()
    ))
}

/// A run that fails on networks that loaded - out of memory, almost always -
/// says nothing about the document, so the document may be tried again.
fn run_error(error: impl Display) -> ExtractionError {
    ExtractionError::io(std::io::Error::other(format!(
        "OCR inference failed: {error}"
    )))
}

fn session(path: &Path, threads: usize) -> Result<Session, ExtractionError> {
    let error = |error: &dyn Display| model_error(path, error);
    Session::builder()
        .map_err(|e| error(&e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| error(&e))?
        .with_intra_threads(threads)
        .map_err(|e| error(&e))?
        .with_inter_threads(1)
        .map_err(|e| error(&e))?
        // A spinning thread waits for work by burning a core. Between the
        // short layers of these networks that is most of the time, on a
        // machine whose user is still working.
        .with_intra_op_spinning(false)
        .map_err(|e| error(&e))?
        .with_inter_op_spinning(false)
        .map_err(|e| error(&e))?
        // The default arena keeps the largest page's buffers for the life
        // of the process, and the worker outlives every document: half a
        // gigabyte held, idle, beside the model server. Without it each run
        // hands its memory back.
        .with_execution_providers([ort::ep::CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| error(&e))?
        .commit_from_file(path)
        .map_err(|e| error(&e))
}

impl PaddleOcr {
    /// The engine over the runtime and models in `runtime`, as it ships.
    pub fn new(runtime: &Path) -> Result<Self, ExtractionError> {
        Self::with_config(PaddleAssets::in_directory(runtime), PaddleConfig::default())
    }

    /// The engine with `config`. One set of sessions is made here, so a
    /// runtime or model that does not load fails now, while the caller can
    /// still fall back to Tesseract, rather than on the first page.
    pub fn with_config(
        assets: PaddleAssets,
        config: PaddleConfig,
    ) -> Result<Self, ExtractionError> {
        if !assets.present() {
            return Err(ExtractionError::native_assets_missing(
                "ONNX Runtime or the OCR models are absent",
            ));
        }
        load_runtime(&assets.runtime_library)?;
        let has_orientation = assets.orientation_model.is_file();
        let engine = Self {
            pool: SessionPool::new(config.workers),
            dictionary: Dictionary::from_lines(RECOGNITION_DICTIONARY),
            assets,
            config,
            has_orientation,
        };
        let mut first = engine
            .pool
            .acquire(|| engine.build_sessions(), &CancellationToken::new())?;
        engine.check_recognizer(first.sessions())?;
        drop(first);
        Ok(engine)
    }

    pub fn config(&self) -> &PaddleConfig {
        &self.config
    }

    /// The clockwise turn the orientation classifier says `page` needs and
    /// its probability, or `None` without a classifier.
    pub fn page_orientation(&self, page: &RgbImage) -> Result<Option<(u16, f32)>, ExtractionError> {
        let cancel = CancellationToken::new();
        let mut lease = self.pool.acquire(|| self.build_sessions(), &cancel)?;
        self.classify(lease.sessions(), page, &cancel)
    }

    fn build_sessions(&self) -> Result<Sessions, ExtractionError> {
        let threads = self.config.intra_threads.max(1);
        Ok(Sessions {
            detection: session(&self.assets.detection_model, threads)?,
            recognition: session(&self.assets.recognition_model, threads)?,
            orientation: if self.has_orientation {
                Some(session(&self.assets.orientation_model, threads)?)
            } else {
                None
            },
        })
    }

    /// Refuses a recognizer whose classes are not the built-in dictionary's:
    /// decoding it would put the wrong letter in every column.
    fn check_recognizer(&self, sessions: &mut Sessions) -> Result<(), ExtractionError> {
        let classes = sessions.recognition.outputs().first().and_then(|output| {
            output
                .dtype()
                .tensor_shape()
                .and_then(|shape| shape.last().copied())
        });
        match classes {
            Some(classes) if classes == self.dictionary.classes() as i64 => Ok(()),
            // A dynamic class axis says nothing until a line is read.
            Some(-1) | None => Ok(()),
            Some(classes) => Err(model_error(
                &self.assets.recognition_model,
                format!(
                    "it reads {classes} classes, but its dictionary has {}",
                    self.dictionary.classes()
                ),
            )),
        }
    }

    /// The probability map the detector gives for `page`, scaled into line
    /// outlines on `page`.
    fn detect(
        &self,
        sessions: &mut Sessions,
        page: &RgbImage,
        cancel: &CancellationToken,
    ) -> Result<Vec<Quad>, ExtractionError> {
        cancel.check()?;
        let (width, height) =
            detection_input_size(page.width(), page.height(), self.config.detection_size);
        let input = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || detection_input(page, width, height),
        );
        let lines = cancel.timed(
            |timings| &mut timings.ocr_engine_micros,
            || -> Result<_, ExtractionError> {
                let tensor = TensorRef::from_array_view((
                    [1_usize, 3, height as usize, width as usize],
                    input.as_slice(),
                ))
                .map_err(run_error)?;
                let outputs = sessions
                    .detection
                    .run(ort::inputs![tensor])
                    .map_err(run_error)?;
                let (shape, map) = outputs[0].try_extract_tensor::<f32>().map_err(run_error)?;
                let (map_height, map_width) = match shape.as_ref() {
                    [_, _, h, w] => (*h as usize, *w as usize),
                    other => {
                        return Err(run_error(format!("detection output shape {other:?}")));
                    }
                };
                Ok(lines_from_map(
                    map,
                    map_width,
                    map_height,
                    page.width() as f32,
                    page.height() as f32,
                    &self.config.db,
                ))
            },
        )?;
        Ok(lines.into_iter().map(|line| line.quad).collect())
    }

    /// Every crop read, in the order given.
    fn recognize_crops(
        &self,
        sessions: &mut Sessions,
        crops: &[LineCrop],
        cancel: &CancellationToken,
    ) -> Result<Vec<DecodedLine>, ExtractionError> {
        let widths: Vec<u32> = crops.iter().map(|crop| crop.width).collect();
        self.recognize_lines(
            sessions,
            &widths,
            |index| Cow::Borrowed(&crops[index]),
            cancel,
        )
    }

    /// Reads lines `0..widths.len()`, cutting each one's crop only when its
    /// batch comes up: a page's lines are never all held at once, and a
    /// page of a thousand long lines costs one batch of crops, not a
    /// thousand.
    fn recognize_lines<'c>(
        &self,
        sessions: &mut Sessions,
        widths: &[u32],
        crop: impl Fn(usize) -> Cow<'c, LineCrop>,
        cancel: &CancellationToken,
    ) -> Result<Vec<DecodedLine>, ExtractionError> {
        let mut decoded = vec![DecodedLine::default(); widths.len()];
        // Lines of similar length share a batch, so little of it is padding.
        let mut order: Vec<usize> = (0..widths.len()).collect();
        order.sort_by_key(|&index| (widths[index], index));
        for chunk in order.chunks(self.config.recognition_batch.max(1)) {
            cancel.check()?;
            let (input, width) = cancel.timed(
                |timings| &mut timings.ocr_encode_micros,
                || {
                    let crops: Vec<Cow<'c, LineCrop>> =
                        chunk.iter().map(|&index| crop(index)).collect();
                    let batch: Vec<&LineCrop> = crops.iter().map(AsRef::as_ref).collect();
                    recognition_batch(&batch, MIN_RECOGNITION_WIDTH)
                },
            );
            let lines = cancel.timed(
                |timings| &mut timings.ocr_engine_micros,
                || -> Result<Vec<DecodedLine>, ExtractionError> {
                    let tensor = TensorRef::from_array_view((
                        [chunk.len(), 3, RECOGNITION_HEIGHT as usize, width as usize],
                        input.as_slice(),
                    ))
                    .map_err(run_error)?;
                    let outputs = sessions
                        .recognition
                        .run(ort::inputs![tensor])
                        .map_err(run_error)?;
                    let (shape, probabilities) =
                        outputs[0].try_extract_tensor::<f32>().map_err(run_error)?;
                    let (steps, classes) = match shape.as_ref() {
                        [_, steps, classes] => (*steps as usize, *classes as usize),
                        other => {
                            return Err(run_error(format!("recognition output shape {other:?}")));
                        }
                    };
                    if classes != self.dictionary.classes() {
                        return Err(run_error(format!(
                            "recognizer read {classes} classes for a {}-class dictionary",
                            self.dictionary.classes()
                        )));
                    }
                    Ok((0..chunk.len())
                        .map(|item| {
                            let rows = &probabilities
                                [item * steps * classes..(item + 1) * steps * classes];
                            greedy_decode(rows, steps, classes, &self.dictionary).trimmed()
                        })
                        .collect())
                },
            )?;
            for (&index, line) in chunk.iter().zip(lines) {
                decoded[index] = line;
            }
        }
        Ok(decoded)
    }

    /// The clockwise turn the classifier says the page needs, and how sure
    /// it is.
    fn classify(
        &self,
        sessions: &mut Sessions,
        page: &RgbImage,
        cancel: &CancellationToken,
    ) -> Result<Option<(u16, f32)>, ExtractionError> {
        let Some(classifier) = sessions.orientation.as_mut() else {
            return Ok(None);
        };
        cancel.record(|timings| timings.orientation_passes += 1);
        let input = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || orientation_input(page),
        );
        let scores = cancel.timed(
            |timings| &mut timings.ocr_engine_micros,
            || -> Result<Vec<f32>, ExtractionError> {
                let tensor = TensorRef::from_array_view(([1_usize, 3, 224, 224], input.as_slice()))
                    .map_err(run_error)?;
                let outputs = classifier.run(ort::inputs![tensor]).map_err(run_error)?;
                let (_, scores) = outputs[0].try_extract_tensor::<f32>().map_err(run_error)?;
                Ok(scores.to_vec())
            },
        )?;
        if scores.len() != 4 {
            return Err(run_error(format!(
                "orientation classifier gave {} scores",
                scores.len()
            )));
        }
        let probabilities = softmax_if_needed(&scores);
        let (class, probability) = probabilities.iter().copied().enumerate().fold(
            (0, f32::MIN),
            |(best, top), (index, value)| {
                if value > top {
                    (index, value)
                } else {
                    (best, top)
                }
            },
        );
        Ok(Some((CLASS_TURNS[class], probability)))
    }

    /// One recognition pass over the page turned clockwise by `rotation`.
    fn read_at(
        &self,
        sessions: &mut Sessions,
        page: &RgbImage,
        rotation: u16,
        cancel: &CancellationToken,
    ) -> Result<PassReading, ExtractionError> {
        let (placed, sideways) = self.read_lines_at(sessions, page, rotation, cancel)?;
        Ok(PassReading {
            result: OcrResult::new(page_text(&placed), page_confidence(&placed))
                .with_rotation(rotation % 360)
                .with_lines(placed.iter().map(PlacedLine::to_ocr_line).collect()),
            sideways,
        })
    }

    /// The lines one pass reads, in reading order, with every character's
    /// probability, and whether the page lay on its side.
    fn read_lines_at(
        &self,
        sessions: &mut Sessions,
        page: &RgbImage,
        rotation: u16,
        cancel: &CancellationToken,
    ) -> Result<(Vec<PlacedLine>, bool), ExtractionError> {
        cancel.check()?;
        cancel.record(|timings| timings.ocr_passes += 1);
        let turned: Cow<'_, RgbImage> = match rotation % 360 {
            0 => Cow::Borrowed(page),
            degrees => Cow::Owned(cancel.timed(
                |timings| &mut timings.ocr_encode_micros,
                || turn_clockwise(page, degrees),
            )),
        };
        let mut frame = turned;
        let mut quads = self.detect(sessions, &frame, cancel)?;
        let sideways = mostly_vertical(&quads);
        // Levelling. A page turned well off level is turned back and
        // detected again; a page a little off is read as it is, along each
        // line's own slope, and only the boxes it reports are levelled.
        let mut report_turn = 0.0_f32;
        if let Some(skew) = estimate_skew(&quads) {
            if skew.abs() >= self.config.redetect_skew_degrees {
                let levelled = cancel.timed(
                    |timings| &mut timings.ocr_encode_micros,
                    || rotate_page(&frame, -skew),
                );
                frame = Cow::Owned(levelled);
                quads = self.detect(sessions, &frame, cancel)?;
            } else if skew.abs() >= MIN_REPORTED_SKEW_DEGREES {
                report_turn = -skew;
            }
        }
        let centre = Point::new(frame.width() as f32 / 2.0, frame.height() as f32 / 2.0);
        let reported: Vec<Quad> = quads
            .iter()
            .map(|quad| {
                if report_turn == 0.0 {
                    *quad
                } else {
                    quad.rotated_about(centre, report_turn.to_radians())
                }
            })
            .collect();
        let widths: Vec<u32> = quads
            .iter()
            .map(|quad| line_width(quad, RECOGNITION_HEIGHT))
            .collect();
        let mut decoded = self.recognize_lines(
            sessions,
            &widths,
            |index| Cow::Owned(crop_line(&frame, &quads[index], RECOGNITION_HEIGHT)),
            cancel,
        )?;
        if let Some(settings) = &self.config.reread {
            self.reread(sessions, &frame, &quads, &mut decoded, settings, cancel)?;
        }
        let typical_height = {
            let mut heights: Vec<f32> = reported.iter().map(Quad::height).collect();
            heights.sort_by(f32::total_cmp);
            heights.get(heights.len() / 2).copied().unwrap_or(10.0)
        };
        let order = reading_order(&reported, typical_height * 0.5);
        let placed: Vec<PlacedLine> = order
            .into_iter()
            .filter(|&index| {
                !decoded[index].text.is_empty()
                    && decoded[index].mean_probability() >= self.config.drop_score
            })
            .map(|index| PlacedLine {
                text: decoded[index].text.clone(),
                bounds: reported[index].bounds(),
                char_probabilities: decoded[index].char_probabilities.clone(),
            })
            .collect();
        Ok((placed, sideways))
    }

    /// The selective second pass: the lines holding a critical field that
    /// were read with an uncertain character are cut out again with more
    /// margin, preprocessed each way the settings name, read again, and
    /// replaced by whichever reading parses and is most confident.
    fn reread(
        &self,
        sessions: &mut Sessions,
        frame: &RgbImage,
        quads: &[Quad],
        decoded: &mut [DecodedLine],
        settings: &super::RereadSettings,
        cancel: &CancellationToken,
    ) -> Result<(), ExtractionError> {
        let candidates: Vec<(&str, f32)> = decoded
            .iter()
            .map(|line| (line.text.as_str(), line.min_probability()))
            .collect();
        let chosen = lines_to_reread(
            &candidates,
            settings.min_char_probability,
            settings.max_lines,
        );
        if chosen.is_empty() || settings.preprocessings.is_empty() {
            return Ok(());
        }
        let crops: Vec<LineCrop> = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || {
                chosen
                    .iter()
                    .flat_map(|&index| {
                        let crop = crop_line(
                            frame,
                            &quads[index].padded(REREAD_PADDING),
                            RECOGNITION_HEIGHT,
                        );
                        settings
                            .preprocessings
                            .iter()
                            .map(move |preprocessing| match preprocessing {
                                RereadPreprocessing::PaddedContrast => stretch_contrast(&crop),
                                RereadPreprocessing::PaddedBinarized => binarize(&crop),
                            })
                    })
                    .collect()
            },
        );
        let rereads = self.recognize_crops(sessions, &crops, cancel)?;
        let per_line = settings.preprocessings.len();
        for (position, &index) in chosen.iter().enumerate() {
            let alternatives = &rereads[position * per_line..(position + 1) * per_line];
            let readings: Vec<Reading> = std::iter::once(&decoded[index])
                .chain(alternatives)
                .map(|line| Reading {
                    text: line.text.clone(),
                    mean_probability: line.mean_probability(),
                })
                .collect();
            let best = best_reading(&readings);
            if best > 0 {
                decoded[index] = alternatives[best - 1].clone();
            }
        }
        Ok(())
    }

    fn read_page(
        &self,
        sessions: &mut Sessions,
        page: &RgbImage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let minimum = self.config.orientation_min_probability;
        let trusted = |verdict: Option<(u16, f32)>| match verdict {
            Some((turn, probability)) if probability >= minimum => Some(turn),
            _ => None,
        };
        match self.config.orientation {
            OrientationStrategy::ClassifierFirst => {
                let first = trusted(self.classify(sessions, page, cancel)?).unwrap_or(0);
                search_orientation(first, None, |turn| {
                    self.read_at(sessions, page, turn, cancel)
                })
            }
            OrientationStrategy::UprightFirst => {
                let upright = self.read_at(sessions, page, 0, cancel)?;
                if pass_is_final(&upright) {
                    return Ok(upright.result);
                }
                let suggested = trusted(self.classify(sessions, page, cancel)?);
                search_orientation(0, suggested, |turn| {
                    if turn == 0 {
                        Ok(upright.clone())
                    } else {
                        self.read_at(sessions, page, turn, cancel)
                    }
                })
            }
        }
    }

    /// One pass at a fixed turn, with no orientation search, line by line
    /// with every character's probability. For measuring the engine.
    #[doc(hidden)]
    pub fn read_turned_lines(
        &self,
        page: &RgbImage,
        rotation: u16,
    ) -> Result<Vec<PlacedLine>, ExtractionError> {
        let cancel = CancellationToken::new();
        let mut lease = self.pool.acquire(|| self.build_sessions(), &cancel)?;
        Ok(self
            .read_lines_at(lease.sessions(), page, rotation, &cancel)?
            .0)
    }

    /// One pass at a fixed turn, with no orientation search: what the page
    /// reads as when turned clockwise by `rotation`. For measuring.
    pub fn read_turned(
        &self,
        page: &RgbImage,
        rotation: u16,
    ) -> Result<OcrResult, ExtractionError> {
        let cancel = CancellationToken::new();
        let mut lease = self.pool.acquire(|| self.build_sessions(), &cancel)?;
        Ok(self
            .read_at(lease.sessions(), page, rotation, &cancel)?
            .result)
    }
}

/// The clockwise turn that rights a page of each classifier class. The
/// classifier names the angle the page was turned counter-clockwise by;
/// a page it calls 90 is righted by turning it 270 clockwise.
const CLASS_TURNS: [u16; 4] = [0, 270, 180, 90];

fn softmax_if_needed(scores: &[f32]) -> Vec<f32> {
    let sum: f32 = scores.iter().sum();
    if scores.iter().all(|score| (0.0..=1.0).contains(score)) && (sum - 1.0).abs() < 1e-3 {
        return scores.to_vec();
    }
    let max = scores.iter().copied().fold(f32::MIN, f32::max);
    let exponentials: Vec<f32> = scores.iter().map(|score| (score - max).exp()).collect();
    let total: f32 = exponentials.iter().sum();
    exponentials.iter().map(|value| value / total).collect()
}

fn turn_clockwise(page: &RgbImage, degrees: u16) -> RgbImage {
    match degrees {
        90 => image::imageops::rotate90(page),
        180 => image::imageops::rotate180(page),
        270 => image::imageops::rotate270(page),
        _ => page.clone(),
    }
}

impl OcrBackend for PaddleOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        cancel.check()?;
        let rgb: Cow<'_, RgbImage> = match page.image.as_rgb8() {
            Some(rgb) => Cow::Borrowed(rgb),
            None => Cow::Owned(cancel.timed(
                |timings| &mut timings.ocr_encode_micros,
                || page.image.to_rgb8(),
            )),
        };
        if rgb.width() == 0 || rgb.height() == 0 {
            return Ok(OcrResult::new("", 0.0));
        }
        let mut lease = self.pool.acquire(|| self.build_sessions(), cancel)?;
        self.read_page(lease.sessions(), &rgb, cancel)
    }

    /// As many pages as the pool has sessions for; a page past that would
    /// only wait for one.
    fn concurrency(&self) -> usize {
        self.config.workers.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A second set of sessions that will not build - memory is short -
    /// makes a page wait for the set another page holds, not fail; only a
    /// pool with no set at all reports the error.
    #[test]
    fn a_set_that_will_not_build_waits_for_one_that_did() {
        let pool = SessionPool::<u32>::new(2);
        let cancel = CancellationToken::new();
        let first = pool.acquire(|| Ok(1), &cancel).unwrap();
        std::thread::scope(|scope| {
            let waiting = scope.spawn(|| {
                let mut lease = pool
                    .acquire(
                        || Err(ExtractionError::resource_limit("no memory")),
                        &cancel,
                    )
                    .expect("waits for the first set instead of failing");
                *lease.sessions()
            });
            std::thread::sleep(Duration::from_millis(250));
            drop(first);
            assert_eq!(waiting.join().unwrap(), 1);
        });

        let empty = SessionPool::<u32>::new(2);
        assert!(
            empty
                .acquire(
                    || Err(ExtractionError::resource_limit("no memory")),
                    &cancel
                )
                .is_err()
        );
    }
}

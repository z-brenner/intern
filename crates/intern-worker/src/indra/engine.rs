//! The Indra engine: ONNX Runtime sessions for the networks, the page
//! pipeline that drives the stages, and the [`OcrBackend`] the worker
//! reads scans with.
//!
//! A page is read as the reference `OrtOCR.__call__` reads it (spec §4-§9):
//! its skew is measured on a preview, the orientation network names its
//! quarter turn, the page is turned and levelled, lines are detected (again
//! at a larger size when they come out small), cut out and read in batches
//! of similar width, the unsure ones searched again under the language
//! model, and the lines assembled into blocks in reading order by the order
//! network. Boxes are mapped back off the levelled page.
//!
//! Where Intern goes further than the reference: a page the orientation
//! network is not sure of (its best turn at or under 0.95) and that does not
//! read convincingly upright is read at the other quarter turns, and the
//! better reading kept, as PP-OCR's pages are (`paddle::orientation`). The
//! reference reads such a page as it lies, often into gibberish.

use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use image::DynamicImage;
use ort::session::Session;
use ort::value::TensorRef;

use crate::extract::{
    CancellationToken, ExtractionError, OcrBackend, OcrBlock, OcrBlockLabel, OcrLine, OcrResult,
    RenderedPage,
};
use crate::onnx::{SessionPool, SessionSettings, load_runtime, model_error, run_error, session};
use crate::paddle::geometry::{Point, Quad};
use crate::paddle::orientation::{PassReading, mostly_vertical, search_orientation};
use crate::paddle::{ThreadBudget, default_thread_budget};

use super::IndraAssets;
use super::assemble::{AssembleOptions, TableCell, assemble_with};
use super::beam::BeamParams;
use super::det::{adaptive_long_side, decode, page_to_input, to_page};
use super::fields::join_labelled_values;
use super::imgops;
use super::lm::NgramLm;
use super::meta::Meta;
use super::page::{
    DESKEW_MIN, ORIENT_MIN, VERTICAL, estimate_skew_threaded, orient_input, orientation_class,
    preview, rotate_page, round_to, turn, unrotate_box,
};
use super::rec::{
    LineLm, LineOutputs, crop_geometry, crop_line, is_dark_line, pad_batch, read_line,
    width_batches,
};
use super::types::{BlockClass, DetBox, Gray, OrderOutputs, PageReading, RecLine, TokenKind};

/// The orientation network's input side.
const ORIENT_SIDE: usize = 768;
/// The orientation network's classes: four quarter turns and vertical.
const ORIENT_CLASSES: usize = 5;
/// Features the order network reads a line by.
const ORDER_FEATURES: usize = 16;
/// The order network's block classes.
const ORDER_CLASSES: usize = 10;

/// What the recognizer reports, in the order the reference asks for it
/// (`OUT_GREEDY + OUT_LM`).
const REC_OUTPUTS: [&str; 6] = ["ids", "conf", "lengths", "blank", "topv", "topi"];

/// One worker's networks.
struct Sessions {
    detection: Session,
    orientation: Session,
    recognition: Session,
    order: Session,
}

/// What every worker reads with, loaded once: the vocabulary and sizes, and
/// the language model (15 MB of tables, so it is not loaded per document).
struct Shared {
    meta: Meta,
    lm: NgramLm,
    /// The class id of the space, which separates words.
    space: Option<u32>,
}

/// Reads scans with Indra's networks. Cheap to share between threads:
/// every page read at the same time borrows its own sessions from the
/// pool, and all of them share one vocabulary and language model.
pub struct IndraOcr {
    assets: IndraAssets,
    budget: ThreadBudget,
    shared: Arc<Shared>,
    pool: SessionPool<Sessions>,
}

impl std::fmt::Debug for IndraOcr {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IndraOcr")
            .field("assets", &self.assets)
            .field("budget", &self.budget)
            .finish_non_exhaustive()
    }
}

/// One read of a page at one turn, before it is reported.
struct Pass {
    reading: PageReading,
    /// Block by block, the cells of the blocks assembly writes as tables.
    cells: Vec<Option<Vec<TableCell>>>,
    /// The recognizer's reading of every detected line, in detection order,
    /// with each character's probability.
    rec: Vec<RecLine>,
    /// The detected lines, on the levelled page.
    boxes: Vec<DetBox>,
    /// The orientation network's five probabilities, when it was asked.
    orientation: Option<Vec<f32>>,
    /// The size of the page the boxes are on (turned, unlevelled).
    width: usize,
    height: usize,
}

impl IndraOcr {
    /// The engine over the runtime and models `assets` names, with the
    /// thread budget PP-OCR uses on this machine.
    ///
    /// One set of sessions is made here, so a runtime or model that does not
    /// load fails now, while the caller can still fall back to another
    /// engine, rather than on the first page.
    pub fn new(assets: IndraAssets) -> Result<Self, ExtractionError> {
        Self::with_budget(assets, default_thread_budget())
    }

    /// The engine reading `budget.workers` pages at once on
    /// `budget.intra_threads` threads each.
    pub fn with_budget(assets: IndraAssets, budget: ThreadBudget) -> Result<Self, ExtractionError> {
        if !assets.present() {
            return Err(ExtractionError::native_assets_missing(
                "ONNX Runtime or Indra's OCR models are absent",
            ));
        }
        load_runtime(&assets.runtime_library)?;
        let meta = Meta::load(&assets.meta).map_err(|error| model_error(&assets.meta, error))?;
        let lm = NgramLm::load(&assets.language_model)
            .map_err(|error| model_error(&assets.language_model, error))?;
        let space = meta
            .vocab
            .chars
            .iter()
            .position(|character| character == " ")
            .map(|index| index as u32 + 1);
        let engine = Self {
            pool: SessionPool::new(budget.workers),
            shared: Arc::new(Shared { meta, lm, space }),
            assets,
            budget,
        };
        engine.check_classes()?;
        let mut first = engine
            .pool
            .acquire(|| engine.build_sessions(), &CancellationToken::new())?;
        engine.check_sessions(first.sessions())?;
        drop(first);
        Ok(engine)
    }

    /// The thread budget the engine reads under.
    pub fn budget(&self) -> ThreadBudget {
        self.budget
    }

    fn build_sessions(&self) -> Result<Sessions, ExtractionError> {
        // No CPU arena (spec §2): with line batches of changing shape it
        // kept growing in the reference (1.4 GB over ten runs of one page)
        // and bought no measurable speed.
        let settings = SessionSettings {
            threads: self.budget.intra_threads.max(1),
            arena: false,
        };
        Ok(Sessions {
            detection: session(&self.assets.detection_model, settings)?,
            orientation: session(&self.assets.orientation_model, settings)?,
            recognition: session(&self.assets.recognition_model, settings)?,
            order: session(&self.assets.order_model, settings)?,
        })
    }

    /// Refuses a recognizer whose classes are not the vocabulary's plus the
    /// blank: decoding it would put the wrong letter in every place. The
    /// recognizer's outputs are argmax ids and top-k lists, which carry no
    /// class axis, so the count is read from the classifier head in the
    /// model file; a graph whose head cannot be found is checked line by
    /// line instead (see [`IndraOcr::recognize_lines`]).
    fn check_classes(&self) -> Result<(), ExtractionError> {
        let path = &self.assets.recognition_model;
        let bytes = std::fs::read(path).map_err(|error| model_error(path, error))?;
        let expected = self.shared.meta.vocab.chars.len() + 1;
        match onnx_graph::recognizer_classes(&bytes) {
            Some(classes) if classes != expected => Err(model_error(
                path,
                format!("it reads {classes} classes, but meta.json's vocabulary has {expected}"),
            )),
            _ => Ok(()),
        }
    }

    /// Refuses networks that do not take and give what the pipeline feeds
    /// them and reads from them.
    fn check_sessions(&self, sessions: &mut Sessions) -> Result<(), ExtractionError> {
        fn check(
            path: &Path,
            session: &Session,
            inputs: &[&str],
            outputs: &[&str],
        ) -> Result<(), ExtractionError> {
            let has_input =
                |name: &&str| session.inputs().iter().any(|input| input.name() == *name);
            let has_output = |name: &&str| {
                session
                    .outputs()
                    .iter()
                    .any(|output| output.name() == *name)
            };
            if let Some(missing) = inputs.iter().find(|name| !has_input(name)) {
                return Err(model_error(path, format!("it has no input {missing:?}")));
            }
            if let Some(missing) = outputs.iter().find(|name| !has_output(name)) {
                return Err(model_error(path, format!("it has no output {missing:?}")));
            }
            Ok(())
        }
        let assets = &self.assets;
        check(
            &assets.detection_model,
            &sessions.detection,
            &["x"],
            &["out"],
        )?;
        check(
            &assets.orientation_model,
            &sessions.orientation,
            &["x"],
            &["p"],
        )?;
        check(
            &assets.recognition_model,
            &sessions.recognition,
            &["x", "widths"],
            &REC_OUTPUTS,
        )?;
        check(
            &assets.order_model,
            &sessions.order,
            &["feats", "mask"],
            &["a", "cls", "same"],
        )?;
        // The recognizer's input height is the tall window's.
        let meta = &self.shared.meta;
        let expected = (meta.rec_height * meta.rec_window) as i64;
        let height = sessions
            .recognition
            .inputs()
            .iter()
            .find(|input| input.name() == "x")
            .and_then(|input| {
                input
                    .dtype()
                    .tensor_shape()
                    .and_then(|shape| shape.get(2).copied())
            });
        match height {
            Some(height) if height != -1 && height != expected => Err(model_error(
                &assets.recognition_model,
                format!("it reads lines {height} high, but meta.json says {expected}"),
            )),
            _ => Ok(()),
        }
    }

    /// The orientation network's five probabilities for `gray`.
    fn orient(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        cancel: &CancellationToken,
    ) -> Result<Vec<f32>, ExtractionError> {
        cancel.check()?;
        cancel.record(|timings| timings.orientation_passes += 1);
        let input = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || orient_input(gray),
        );
        let probabilities = cancel.timed(
            |timings| &mut timings.ocr_engine_micros,
            || -> Result<Vec<f32>, ExtractionError> {
                let tensor =
                    TensorRef::from_array_view(([1_usize, 1, ORIENT_SIDE, ORIENT_SIDE], &*input))
                        .map_err(run_error)?;
                let outputs = sessions
                    .orientation
                    .run(ort::inputs!["x" => tensor])
                    .map_err(run_error)?;
                let (_, probabilities) = output(&outputs, "p")?
                    .try_extract_tensor::<f32>()
                    .map_err(run_error)?;
                Ok(probabilities.to_vec())
            },
        )?;
        if probabilities.len() != ORIENT_CLASSES {
            return Err(run_error(format!(
                "the orientation network gave {} probabilities",
                probabilities.len()
            )));
        }
        Ok(probabilities)
    }

    /// One detector pass at `long_side`: boxes in detector pixels and the
    /// detector's scale.
    fn detect_at(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        long_side: usize,
        cancel: &CancellationToken,
    ) -> Result<(Vec<DetBox>, f64), ExtractionError> {
        cancel.check()?;
        let input = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || page_to_input(gray, long_side),
        );
        let boxes = cancel.timed(
            |timings| &mut timings.ocr_engine_micros,
            || -> Result<Vec<DetBox>, ExtractionError> {
                let tensor = TensorRef::from_array_view((
                    [1_usize, 1, input.height, input.width],
                    input.data.as_slice(),
                ))
                .map_err(run_error)?;
                let outputs = sessions
                    .detection
                    .run(ort::inputs!["x" => tensor])
                    .map_err(run_error)?;
                let (shape, map) = output(&outputs, "out")?
                    .try_extract_tensor::<f32>()
                    .map_err(run_error)?;
                let (channels, height, width) = match shape.as_ref() {
                    [1, channels, height, width] if *channels >= 3 => {
                        (*channels as usize, *height as usize, *width as usize)
                    }
                    other => return Err(run_error(format!("detector output shape {other:?}"))),
                };
                debug_assert_eq!(map.len(), channels * height * width);
                // The first three channels: core logit, distances to the top
                // and bottom. A fourth (pictures) is not in the v1.1.0 model.
                Ok(decode(&map[..3 * height * width], height, width))
            },
        )?;
        Ok((boxes, input.scale))
    }

    /// `detect` (spec §5.3): the lines on `gray`, in page pixels, with the
    /// adaptive second pass for a page of small lines.
    fn detect(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        cancel: &CancellationToken,
    ) -> Result<Vec<DetBox>, ExtractionError> {
        let (mut boxes, mut scale) =
            self.detect_at(sessions, gray, self.shared.meta.det_long_side, cancel)?;
        if let Some(long_side) = adaptive_long_side(&boxes) {
            (boxes, scale) = self.detect_at(sessions, gray, long_side, cancel)?;
        }
        Ok(to_page(boxes, scale, gray.width, gray.height))
    }

    /// `recognize` (spec §6) for the base recognizer: every line read, in
    /// the order of `boxes`.
    ///
    /// Batches are formed from the band crops' widths, as the reference
    /// forms them, but only their geometry is computed up front: a batch's
    /// tall crops are cut when its turn comes, so a page of a thousand long
    /// lines holds one batch of crops at a time, not a thousand.
    fn recognize_lines(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        boxes: &[DetBox],
        cancel: &CancellationToken,
    ) -> Result<Vec<RecLine>, ExtractionError> {
        let meta = &self.shared.meta;
        let band_widths: Vec<usize> = boxes
            .iter()
            .map(|bbox| crop_geometry(bbox, 1).width)
            .collect();
        // meta.json may tune the base recognizer's search; it does not for
        // the pinned models, which leaves the pipeline's defaults.
        let (alpha, beta, gate) = meta.lm_settings("base");
        let lm = LineLm {
            lm: &self.shared.lm,
            params: BeamParams {
                alpha,
                beta,
                ..BeamParams::default()
            },
            gate,
        };
        let classes = meta.vocab.chars.len() + 1;
        let mut lines: Vec<Option<RecLine>> = vec![None; boxes.len()];
        for batch in width_batches(&band_widths) {
            cancel.check()?;
            let crops = cancel.timed(
                |timings| &mut timings.ocr_encode_micros,
                || {
                    batch
                        .iter()
                        .map(|&index| {
                            // Light-on-dark is decided once per line.
                            let dark = is_dark_line(gray, &boxes[index]);
                            crop_line(gray, &boxes[index], meta.rec_window, dark)
                        })
                        .collect::<Vec<_>>()
                },
            );
            let (data, height, width, widths) = {
                let refs: Vec<_> = crops.iter().collect();
                pad_batch(&refs)
            };
            drop(crops);
            let read = cancel.timed(
                |timings| &mut timings.ocr_engine_micros,
                || -> Result<Vec<RecLine>, ExtractionError> {
                    let x = TensorRef::from_array_view((
                        [batch.len(), 1, height, width],
                        data.as_slice(),
                    ))
                    .map_err(run_error)?;
                    let w = TensorRef::from_array_view(([batch.len()], widths.as_slice()))
                        .map_err(run_error)?;
                    let outputs = sessions
                        .recognition
                        .run(ort::inputs!["x" => x, "widths" => w])
                        .map_err(run_error)?;
                    let (shape, ids) = output(&outputs, "ids")?
                        .try_extract_tensor::<i64>()
                        .map_err(run_error)?;
                    let steps = match shape.as_ref() {
                        [b, t] if *b as usize == batch.len() => *t as usize,
                        other => return Err(run_error(format!("recognizer ids shape {other:?}"))),
                    };
                    let (_, conf) = output(&outputs, "conf")?
                        .try_extract_tensor::<f32>()
                        .map_err(run_error)?;
                    let (_, lengths) = output(&outputs, "lengths")?
                        .try_extract_tensor::<i64>()
                        .map_err(run_error)?;
                    let (_, blank) = output(&outputs, "blank")?
                        .try_extract_tensor::<f32>()
                        .map_err(run_error)?;
                    let (top_shape, topv) = output(&outputs, "topv")?
                        .try_extract_tensor::<f32>()
                        .map_err(run_error)?;
                    let (_, topi) = output(&outputs, "topi")?
                        .try_extract_tensor::<i64>()
                        .map_err(run_error)?;
                    let k = match top_shape.as_ref() {
                        [_, _, k] => *k as usize,
                        other => return Err(run_error(format!("recognizer topv shape {other:?}"))),
                    };
                    let sizes_agree = conf.len() == ids.len()
                        && blank.len() == ids.len()
                        && lengths.len() == batch.len()
                        && topv.len() == ids.len() * k
                        && topi.len() == ids.len() * k;
                    if !sizes_agree {
                        return Err(run_error("recognizer outputs of different sizes"));
                    }
                    // A class past the vocabulary means the model and
                    // meta.json are not a pair.
                    if let Some(class) = topi.iter().find(|&&class| class < 0 || class as usize >= classes) {
                        return Err(run_error(format!(
                            "the recognizer named class {class}, past its {classes}-class vocabulary"
                        )));
                    }
                    let stride = width as f64 / steps.max(1) as f64;
                    Ok(batch
                        .iter()
                        .enumerate()
                        .map(|(item, &index)| {
                            let n = (lengths[item].max(0) as usize).min(steps);
                            let row = item * steps..item * steps + n;
                            let top = item * steps * k..(item * steps + n) * k;
                            let out = LineOutputs {
                                ids: &ids[row.clone()],
                                conf: &conf[row.clone()],
                                blank: &blank[row],
                                topv: &topv[top.clone()],
                                topi: &topi[top],
                                k,
                            };
                            read_line(
                                &out,
                                &boxes[index],
                                stride,
                                band_widths[index],
                                &meta.vocab,
                                Some(lm),
                            )
                        })
                        .collect())
                },
            )?;
            for (&index, line) in batch.iter().zip(read) {
                lines[index] = Some(line);
            }
        }
        Ok(lines
            .into_iter()
            .map(|line| line.expect("every line is in one batch"))
            .collect())
    }

    /// The order network's outputs for `n` lines' features.
    fn run_order(
        sessions: &mut Sessions,
        features: &[f32],
        n: usize,
        cancel: &CancellationToken,
    ) -> Result<OrderOutputs, ExtractionError> {
        cancel.check()?;
        cancel.timed(
            |timings| &mut timings.ocr_engine_micros,
            || {
                let feats = TensorRef::from_array_view(([1_usize, n, ORDER_FEATURES], features))
                    .map_err(run_error)?;
                let mask = vec![true; n];
                let mask = TensorRef::from_array_view(([1_usize, n], mask.as_slice()))
                    .map_err(run_error)?;
                let outputs = sessions
                    .order
                    .run(ort::inputs!["feats" => feats, "mask" => mask])
                    .map_err(run_error)?;
                let (_, a) = output(&outputs, "a")?
                    .try_extract_tensor::<f32>()
                    .map_err(run_error)?;
                let (_, cls) = output(&outputs, "cls")?
                    .try_extract_tensor::<f32>()
                    .map_err(run_error)?;
                let (_, same) = output(&outputs, "same")?
                    .try_extract_tensor::<f32>()
                    .map_err(run_error)?;
                if a.len() != n * n || same.len() != n * n || cls.len() != n * ORDER_CLASSES {
                    return Err(run_error("order network outputs of the wrong size"));
                }
                Ok(OrderOutputs {
                    n,
                    a: a.to_vec(),
                    cls: cls.to_vec(),
                    same: same.to_vec(),
                })
            },
        )
    }

    /// One read of `gray` (spec §4-§9, `OrtOCR.__call__`): levelled, turned
    /// as the orientation network says when `classify`, else read as it
    /// lies; detected, recognized and assembled; boxes mapped back off the
    /// levelled page.
    ///
    /// A page the network calls vertical (CJK columns) is read as it lies:
    /// only the CJK recognizer reads such columns, and this engine carries
    /// the Latin one.
    fn read_gray(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        classify: bool,
        cancel: &CancellationToken,
    ) -> Result<Pass, ExtractionError> {
        cancel.check()?;
        cancel.record(|timings| timings.ocr_passes += 1);
        let (small, angle) = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || {
                let small = preview(gray);
                // Scored on the page's threads, as the reference scores it on
                // its pool; the angle is the same for any number.
                let angle = estimate_skew_threaded(&small, self.budget.intra_threads.max(1));
                (small, angle)
            },
        );
        let orientation = if classify {
            let straightened: Cow<'_, Gray> = if angle.abs() >= DESKEW_MIN {
                Cow::Owned(cancel.timed(
                    |timings| &mut timings.ocr_encode_micros,
                    || rotate_page(&small, angle),
                ))
            } else {
                Cow::Borrowed(&small)
            };
            Some(self.orient(sessions, &straightened, cancel)?)
        } else {
            None
        };
        let k = match orientation.as_deref().map(orientation_class) {
            Some(VERTICAL) | None => 0,
            Some(k) => k,
        };
        cancel.check()?;
        let turned = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || turn(gray, k, angle, &small),
        );
        drop(small);
        let page = &turned.gray;
        let boxes = self.detect(sessions, page, cancel)?;
        let rec = self.recognize_lines(sessions, page, &boxes, cancel)?;
        cancel.check()?;
        let mut failure: Option<ExtractionError> = None;
        let mut order = |features: &[f32], n: usize| -> OrderOutputs {
            if failure.is_none() {
                match Self::run_order(sessions, features, n, cancel) {
                    Ok(outputs) => return outputs,
                    Err(error) => failure = Some(error),
                }
            }
            // The page fails below; assembly only needs outputs of the
            // right shape to finish.
            OrderOutputs {
                n,
                a: vec![0.0; n * n],
                cls: vec![0.0; n * ORDER_CLASSES],
                same: vec![0.0; n * n],
            }
        };
        let assembly = assemble_with(
            page.width,
            page.height,
            &boxes,
            &rec,
            &self.shared.meta.vocab,
            AssembleOptions::default(),
            &mut order,
        );
        let (mut reading, mut cells) = (assembly.page, assembly.cells);
        if let Some(error) = failure {
            return Err(error);
        }
        reading.rotation = 90 * k as u32;
        reading.skew = round_to(turned.skew, 2);
        if turned.skew != 0.0 {
            // Boxes back on the page as turned, before levelling.
            for line in &mut reading.lines {
                line.bbox = unrotate_box(line.bbox, turned.skew, page.width, page.height);
            }
            for block in &mut reading.blocks {
                block.bbox = unrotate_box(block.bbox, turned.skew, page.width, page.height);
            }
            for cell in cells.iter_mut().flatten().flatten() {
                cell.bbox = unrotate_box(cell.bbox, turned.skew, page.width, page.height);
            }
        }
        Ok(Pass {
            reading,
            cells,
            rec,
            boxes,
            orientation,
            width: page.width,
            height: page.height,
        })
    }

    /// One read of `gray` turned clockwise by `rotation`, without asking
    /// the orientation network.
    fn read_turned_pass(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        rotation: u16,
        cancel: &CancellationToken,
    ) -> Result<PassReading, ExtractionError> {
        let rotation = rotation % 360;
        let turned: Cow<'_, Gray> = if rotation == 0 {
            Cow::Borrowed(gray)
        } else {
            Cow::Owned(cancel.timed(
                |timings| &mut timings.ocr_encode_micros,
                || imgops::rot90(gray, -i32::from(rotation / 90)),
            ))
        };
        let pass = self.read_gray(sessions, &turned, false, cancel)?;
        Ok(self.pass_reading(&pass, rotation))
    }

    /// A pass as the orientation search weighs it.
    fn pass_reading(&self, pass: &Pass, rotation: u16) -> PassReading {
        let quads: Vec<Quad> = pass.boxes.iter().map(quad).collect();
        PassReading {
            result: self.to_result(pass, rotation),
            sideways: mostly_vertical(&quads),
            detected: pass.boxes.len(),
        }
    }

    /// The page read the right way up (see the module documentation).
    fn read_page(
        &self,
        sessions: &mut Sessions,
        gray: &Gray,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let first = self.read_gray(sessions, gray, true, cancel)?;
        let probabilities = first.orientation.clone().unwrap_or_default();
        let (best, top) = argmax(&probabilities);
        let first = self.pass_reading(&first, first.reading.rotation as u16);
        // The network is sure of its turn: the reference's reading stands.
        if top > ORIENT_MIN && best != VERTICAL {
            return Ok(first.result);
        }
        // Not sure: its best quarter turn goes first in the search.
        let suggested = (1..VERTICAL).contains(&best).then(|| 90 * best as u16);
        search_orientation(0, suggested, |rotation| {
            if rotation == 0 {
                Ok(first.clone())
            } else {
                self.read_turned_pass(sessions, gray, rotation, cancel)
            }
        })
    }

    /// The pass as an [`OcrResult`] read at clockwise `rotation`, after
    /// Intern's own step: a form's labels joined to their values
    /// ([`super::fields`]).
    fn to_result(&self, pass: &Pass, rotation: u16) -> OcrResult {
        let mut reading = pass.reading.clone();
        join_labelled_values(&mut reading);
        to_result(
            &reading,
            &pass.cells,
            &pass.rec,
            self.shared.space,
            pass.width,
            pass.height,
            rotation,
        )
    }

    /// The reference pipeline exactly, for measuring parity with it: the
    /// orientation network's turn alone, no search over the others, and the
    /// page as the reference reports it (rotation counter-clockwise).
    #[doc(hidden)]
    pub fn read_reference(&self, gray: &Gray) -> Result<PageReading, ExtractionError> {
        let cancel = CancellationToken::new();
        let mut lease = self.pool.acquire(|| self.build_sessions(), &cancel)?;
        Ok(self
            .read_gray(lease.sessions(), gray, true, &cancel)?
            .reading)
    }

    /// The page in grey as the engine reads it, as OpenCV's
    /// `IMREAD_GRAYSCALE` reads an image file (spec §3.1).
    #[doc(hidden)]
    pub fn gray_page(image: &DynamicImage) -> Gray {
        to_gray(image)
    }

    /// One read at a fixed clockwise turn, with no orientation network and
    /// no search: what the page reads as when turned by `rotation`. For
    /// measuring.
    #[doc(hidden)]
    pub fn read_turned(
        &self,
        image: &DynamicImage,
        rotation: u16,
    ) -> Result<OcrResult, ExtractionError> {
        let cancel = CancellationToken::new();
        let gray = to_gray(image);
        let mut lease = self.pool.acquire(|| self.build_sessions(), &cancel)?;
        Ok(self
            .read_turned_pass(lease.sessions(), &gray, rotation, &cancel)?
            .result)
    }
}

impl OcrBackend for IndraOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        cancel.check()?;
        if page.image.width() == 0 || page.image.height() == 0 {
            return Ok(OcrResult::new("", 0.0));
        }
        let gray = cancel.timed(
            |timings| &mut timings.ocr_encode_micros,
            || to_gray(&page.image),
        );
        let mut lease = self.pool.acquire(|| self.build_sessions(), cancel)?;
        self.read_page(lease.sessions(), &gray, cancel)
    }

    /// As many pages as the pool has sessions for; a page past that would
    /// only wait for one.
    fn concurrency(&self) -> usize {
        self.budget.workers.max(1)
    }

    /// Drops the sessions no page is using. The vocabulary and the language
    /// model stay: they are the same for every document.
    fn release(&self) {
        self.pool.release_idle();
    }
}

fn output<'o>(
    outputs: &'o ort::session::SessionOutputs<'_>,
    name: &str,
) -> Result<&'o ort::value::DynValue, ExtractionError> {
    outputs
        .get(name)
        .ok_or_else(|| run_error(format!("the network gave no output {name:?}")))
}

/// The index and value of the largest probability, the first on ties
/// (numpy's `argmax`); `(0, 0.0)` for none.
fn argmax(values: &[f32]) -> (usize, f32) {
    let mut best = (0, 0.0);
    for (index, &value) in values.iter().enumerate() {
        if index == 0 || value > best.1 {
            best = (index, value);
        }
    }
    best
}

fn quad(bbox: &DetBox) -> Quad {
    Quad([
        Point::new(bbox.x0, bbox.y0),
        Point::new(bbox.x1, bbox.y0),
        Point::new(bbox.x1, bbox.y1),
        Point::new(bbox.x0, bbox.y1),
    ])
}

/// The page in 8-bit gray as OpenCV's `IMREAD_GRAYSCALE` reads it (spec
/// §3.1): colour by OpenCV's integer weights, alpha dropped, 16-bit samples
/// to their high byte.
fn to_gray(image: &DynamicImage) -> Gray {
    let (width, height) = (image.width() as usize, image.height() as usize);
    match image {
        DynamicImage::ImageLuma8(luma) => Gray::new(width, height, luma.as_raw().clone()),
        DynamicImage::ImageLumaA8(luma) => Gray::new(
            width,
            height,
            luma.as_raw()
                .chunks_exact(2)
                .map(|pixel| pixel[0])
                .collect(),
        ),
        DynamicImage::ImageLuma16(luma) => Gray::new(
            width,
            height,
            luma.as_raw()
                .iter()
                .map(|&value| (value >> 8) as u8)
                .collect(),
        ),
        DynamicImage::ImageLumaA16(luma) => Gray::new(
            width,
            height,
            luma.as_raw()
                .chunks_exact(2)
                .map(|pixel| (pixel[0] >> 8) as u8)
                .collect(),
        ),
        DynamicImage::ImageRgb8(rgb) => imgops::gray_from_rgb(rgb.as_raw(), width, height),
        DynamicImage::ImageRgba8(rgba) => {
            let rgb: Vec<u8> = rgba
                .as_raw()
                .chunks_exact(4)
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2]])
                .collect();
            imgops::gray_from_rgb(&rgb, width, height)
        }
        DynamicImage::ImageRgb16(_) | DynamicImage::ImageRgba16(_) => {
            let rgb: Vec<u8> = match image {
                DynamicImage::ImageRgb16(rgb) => rgb
                    .as_raw()
                    .iter()
                    .map(|&value| (value >> 8) as u8)
                    .collect(),
                DynamicImage::ImageRgba16(rgba) => rgba
                    .as_raw()
                    .chunks_exact(4)
                    .flat_map(|pixel| {
                        [pixel[0], pixel[1], pixel[2]].map(|value| (value >> 8) as u8)
                    })
                    .collect(),
                _ => unreachable!(),
            };
            imgops::gray_from_rgb(&rgb, width, height)
        }
        other => imgops::gray_from_rgb(other.to_rgb8().as_raw(), width, height),
    }
}

/// The least certain character of every word the recognizer read, in
/// line order: a word is a run of tokens between spaces.
fn word_minimums(lines: &[RecLine], space: Option<u32>) -> Vec<f32> {
    let mut minimums = Vec::new();
    for line in lines {
        let mut current: Option<f32> = None;
        for token in &line.tokens {
            let is_space = match &token.kind {
                TokenKind::Id(id) => Some(*id) == space,
                TokenKind::Char(text) => text.trim().is_empty(),
            };
            if is_space {
                minimums.extend(current.take());
            } else {
                current = Some(current.map_or(token.p, |least| least.min(token.p)));
            }
        }
        minimums.extend(current);
    }
    minimums
}

/// The page's confidence, 0-100: the mean over its words of each word's
/// least certain character, as PP-OCR's (`paddle::page_confidence`) and
/// Tesseract's are, so [`crate::extract::CONFIDENT_READING`] means the same
/// for every engine. Nothing read is no confidence.
fn page_confidence(lines: &[RecLine], space: Option<u32>) -> f32 {
    let minimums = word_minimums(lines, space);
    if minimums.is_empty() {
        return 0.0;
    }
    let sum: f64 = minimums.iter().map(|&value| f64::from(value)).sum();
    (sum / minimums.len() as f64 * 100.0) as f32
}

fn block_label(class: BlockClass) -> OcrBlockLabel {
    match class {
        BlockClass::Text => OcrBlockLabel::Text,
        BlockClass::SectionHeader => OcrBlockLabel::SectionHeader,
        BlockClass::PageHeader => OcrBlockLabel::PageHeader,
        BlockClass::PageFooter => OcrBlockLabel::PageFooter,
        BlockClass::ListGroup => OcrBlockLabel::ListGroup,
        BlockClass::Table => OcrBlockLabel::Table,
        BlockClass::Caption => OcrBlockLabel::Caption,
        BlockClass::Footnote => OcrBlockLabel::Footnote,
        BlockClass::TableOfContents => OcrBlockLabel::TableOfContents,
        BlockClass::Form => OcrBlockLabel::Form,
    }
}

/// A box in page pixels as whole pixels that still hold it: edges out to
/// the pixel, inside the page.
fn pixel_box(bbox: [f64; 4], width: usize, height: usize) -> [u32; 4] {
    let clamp = |value: f64, limit: usize| value.clamp(0.0, limit as f64) as u32;
    [
        clamp(bbox[0].floor(), width),
        clamp(bbox[1].floor(), height),
        clamp(bbox[2].ceil(), width),
        clamp(bbox[3].ceil(), height),
    ]
}

/// A reading as an [`OcrResult`] at clockwise `rotation`.
///
/// Lines read as nothing (under the confidence floor, or only spaces) are
/// left out, as every other engine leaves them out; blocks keep the lines
/// that are left. A line of a block assembly writes as a table is reported
/// as its cells - the line cut where its characters are more than
/// [`super::assemble::CELL_GAP`] line heights apart, the cells the table is
/// laid out from - each a line of its own with its own box and the line's
/// confidence, so the layout analysis need not cut it again without the
/// character positions. The text is the blocks in reading order, a block's
/// lines (whole, not cut) one to a line and a blank line between blocks, so
/// distillation finds the network's paragraphs; page headers and footers
/// stay in it - leaving them out is the layout analysis's call.
fn to_result(
    reading: &PageReading,
    cells: &[Option<Vec<TableCell>>],
    rec: &[RecLine],
    space: Option<u32>,
    width: usize,
    height: usize,
    rotation: u16,
) -> OcrResult {
    let confidence = |conf: f64| (conf * 100.0).round().clamp(0.0, 100.0) as u8;
    // The cells each table line is reported as.
    let mut cut: Vec<Option<Vec<&TableCell>>> = vec![None; reading.lines.len()];
    for cell in cells.iter().flatten().flatten() {
        if let Some(slot) = cut.get_mut(cell.line) {
            slot.get_or_insert_with(Vec::new).push(cell);
        }
    }
    // Each page line's reported lines.
    let mut kept: Vec<Vec<usize>> = vec![Vec::new(); reading.lines.len()];
    let mut lines = Vec::new();
    for (index, line) in reading.lines.iter().enumerate() {
        let pieces: Vec<(&str, [f64; 4])> = match &cut[index] {
            Some(cells) => cells
                .iter()
                .map(|cell| (cell.text.trim(), cell.bbox))
                .collect(),
            None => vec![(line.text.trim(), line.bbox)],
        };
        for (text, bbox) in pieces {
            if text.is_empty() {
                continue;
            }
            kept[index].push(lines.len());
            lines.push(OcrLine {
                text: text.to_owned(),
                bbox: pixel_box(bbox, width, height),
                confidence: confidence(line.conf),
            });
        }
    }
    let mut placed = vec![false; reading.lines.len()];
    let mut paragraphs: Vec<String> = Vec::new();
    let mut blocks = Vec::new();
    for block in &reading.blocks {
        let members: Vec<usize> = block
            .lines
            .iter()
            .filter_map(|&index| kept.get(index))
            .flatten()
            .copied()
            .collect();
        if members.is_empty() {
            continue;
        }
        blocks.push(OcrBlock {
            label: block_label(block.class),
            bbox: pixel_box(block.bbox, width, height),
            lines: members,
        });
        let texts: Vec<&str> = block
            .lines
            .iter()
            .filter(|&&index| {
                index < placed.len()
                    && !kept[index].is_empty()
                    && !std::mem::replace(&mut placed[index], true)
            })
            .map(|&index| reading.lines[index].text.trim())
            .collect();
        if !texts.is_empty() {
            paragraphs.push(texts.join("\n"));
        }
    }
    // Lines no block holds (a page of one line has no layout) follow, one
    // to a line.
    let loose: Vec<&str> = reading
        .lines
        .iter()
        .enumerate()
        .filter(|&(index, _)| !placed[index] && !kept[index].is_empty())
        .map(|(_, line)| line.text.trim())
        .collect();
    if !loose.is_empty() {
        paragraphs.push(loose.join("\n"));
    }
    OcrResult::new(paragraphs.join("\n\n"), page_confidence(rec, space))
        .with_rotation(rotation % 360)
        .with_lines(lines)
        .with_blocks(blocks)
}

/// Just enough of the ONNX protobuf to find how many classes a recognizer
/// reads: the head that feeds its argmax.
mod onnx_graph {
    /// One protobuf field: its number and its value.
    enum Field<'a> {
        Varint(u64),
        Bytes(&'a [u8]),
        Fixed,
    }

    fn varint(bytes: &[u8], at: &mut usize) -> Option<u64> {
        let mut value = 0_u64;
        for shift in (0..64).step_by(7) {
            let byte = *bytes.get(*at)?;
            *at += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    /// The fields of one message, in order; `None` if it does not parse.
    fn fields(bytes: &[u8]) -> Option<Vec<(u64, Field<'_>)>> {
        let mut at = 0;
        let mut out = Vec::new();
        while at < bytes.len() {
            let key = varint(bytes, &mut at)?;
            let field = match key & 7 {
                0 => Field::Varint(varint(bytes, &mut at)?),
                1 => {
                    at = at.checked_add(8)?;
                    Field::Fixed
                }
                2 => {
                    let length = usize::try_from(varint(bytes, &mut at)?).ok()?;
                    let end = at.checked_add(length)?;
                    let value = bytes.get(at..end)?;
                    at = end;
                    Field::Bytes(value)
                }
                5 => {
                    at = at.checked_add(4)?;
                    Field::Fixed
                }
                _ => return None,
            };
            out.push((key >> 3, field));
        }
        (at == bytes.len()).then_some(out)
    }

    fn text(bytes: &[u8]) -> Option<&str> {
        std::str::from_utf8(bytes).ok()
    }

    struct Node<'a> {
        inputs: Vec<&'a str>,
        outputs: Vec<&'a str>,
        op: &'a str,
    }

    /// The number of classes the recognizer's `ids` are an argmax over:
    /// the output channels of the convolution, matrix product or dense
    /// layer behind it (through transposes and reshapes), from that
    /// layer's bias, or its weight's first axis. `None` if the graph is not
    /// shaped so.
    pub(super) fn recognizer_classes(model: &[u8]) -> Option<usize> {
        // ModelProto.graph = 7; GraphProto.node = 1, .initializer = 5;
        // NodeProto.input = 1, .output = 2, .op_type = 4;
        // TensorProto.dims = 1, .name = 8.
        let graph = fields(model)?
            .into_iter()
            .find_map(|(number, field)| match field {
                Field::Bytes(bytes) if number == 7 => Some(bytes),
                _ => None,
            })?;
        let mut nodes = Vec::new();
        let mut dims: Vec<(&str, Vec<u64>)> = Vec::new();
        for (number, field) in fields(graph)? {
            let Field::Bytes(bytes) = field else { continue };
            match number {
                1 => {
                    let mut node = Node {
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                        op: "",
                    };
                    for (number, field) in fields(bytes)? {
                        if let Field::Bytes(value) = field {
                            match number {
                                1 => node.inputs.push(text(value)?),
                                2 => node.outputs.push(text(value)?),
                                4 => node.op = text(value)?,
                                _ => {}
                            }
                        }
                    }
                    nodes.push(node);
                }
                5 => {
                    let mut name = "";
                    let mut shape = Vec::new();
                    for (number, field) in fields(bytes)? {
                        match (number, field) {
                            (1, Field::Varint(value)) => shape.push(value),
                            (1, Field::Bytes(packed)) => {
                                let mut at = 0;
                                while at < packed.len() {
                                    shape.push(varint(packed, &mut at)?);
                                }
                            }
                            (8, Field::Bytes(value)) => name = text(value)?,
                            _ => {}
                        }
                    }
                    dims.push((name, shape));
                }
                _ => {}
            }
        }
        let producer = |name: &str| nodes.iter().find(|node| node.outputs.contains(&name));
        let shape = |name: &str| {
            dims.iter()
                .find(|(initializer, _)| *initializer == name)
                .map(|(_, shape)| shape)
        };
        let argmax = producer("ids").filter(|node| node.op == "ArgMax")?;
        let mut node = producer(argmax.inputs.first()?)?;
        for _ in 0..8 {
            match node.op {
                "Transpose" | "Reshape" | "Squeeze" | "Unsqueeze" | "Identity" | "Flatten" => {
                    node = producer(node.inputs.first()?)?;
                }
                "Conv" | "Gemm" => {
                    let bias = node.inputs.get(2).and_then(|name| shape(name));
                    let weight = node.inputs.get(1).and_then(|name| shape(name));
                    return match (bias, weight) {
                        (Some(bias), _) if bias.len() == 1 => usize::try_from(bias[0]).ok(),
                        (_, Some(weight)) if node.op == "Conv" => {
                            usize::try_from(*weight.first()?).ok()
                        }
                        _ => None,
                    };
                }
                "Add" => {
                    // A matrix product plus a bias: the bias's length.
                    let bias = node.inputs.iter().find_map(|name| shape(name))?;
                    return (bias.len() == 1)
                        .then(|| usize::try_from(bias[0]).ok())
                        .flatten();
                }
                _ => return None,
            }
        }
        None
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn key(number: u64, wire: u64) -> Vec<u8> {
            encode(number << 3 | wire)
        }

        fn encode(mut value: u64) -> Vec<u8> {
            let mut out = Vec::new();
            loop {
                let byte = (value & 0x7f) as u8;
                value >>= 7;
                if value == 0 {
                    out.push(byte);
                    return out;
                }
                out.push(byte | 0x80);
            }
        }

        fn bytes(number: u64, value: &[u8]) -> Vec<u8> {
            let mut out = key(number, 2);
            out.extend(encode(value.len() as u64));
            out.extend_from_slice(value);
            out
        }

        fn node(op: &str, inputs: &[&str], outputs: &[&str]) -> Vec<u8> {
            let mut out = Vec::new();
            for input in inputs {
                out.extend(bytes(1, input.as_bytes()));
            }
            for output in outputs {
                out.extend(bytes(2, output.as_bytes()));
            }
            out.extend(bytes(4, op.as_bytes()));
            bytes(1, &out)
        }

        fn initializer(name: &str, dims: &[u64]) -> Vec<u8> {
            let mut out = Vec::new();
            for &dim in dims {
                out.extend(key(1, 0));
                out.extend(encode(dim));
            }
            out.extend(bytes(8, name.as_bytes()));
            // Raw data, skipped.
            out.extend(bytes(9, &[0; 16]));
            bytes(5, &out)
        }

        #[test]
        fn the_class_count_is_read_from_the_head_behind_the_argmax() {
            let mut graph = Vec::new();
            graph.extend(node(
                "Conv",
                &["h", "head.weight", "head.bias"],
                &["logits"],
            ));
            graph.extend(node("Transpose", &["logits"], &["t"]));
            graph.extend(node("ArgMax", &["t"], &["ids"]));
            graph.extend(initializer("head.weight", &[1698, 128, 1]));
            graph.extend(initializer("head.bias", &[1698]));
            let mut model = key(1, 0);
            model.extend(encode(8));
            model.extend(bytes(7, &graph));
            assert_eq!(recognizer_classes(&model), Some(1698));

            // No argmax named ids: nothing to say.
            assert_eq!(recognizer_classes(&key(1, 0)), None);
            assert_eq!(recognizer_classes(b"\xff\xff"), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indra::types::{PageBlock, PageLine, Token};

    fn token(id: u32, p: f32) -> Token {
        Token {
            kind: TokenKind::Id(id),
            x: 0.0,
            p,
        }
    }

    #[test]
    fn a_word_is_as_sure_as_its_least_sure_character() {
        // "ab cd": space is class 1.
        let lines = vec![
            RecLine {
                text: "ab cd".into(),
                conf: 0.9,
                tokens: vec![
                    token(66, 0.9),
                    token(67, 0.5),
                    token(1, 0.1),
                    token(68, 1.0),
                    token(69, 0.7),
                ],
            },
            RecLine {
                text: String::new(),
                conf: 0.3,
                tokens: Vec::new(),
            },
        ];
        assert_eq!(word_minimums(&lines, Some(1)), vec![0.5, 0.7]);
        assert!((page_confidence(&lines, Some(1)) - 60.0).abs() < 1e-4);
        assert_eq!(page_confidence(&[], Some(1)), 0.0);
    }

    fn line(text: &str, bbox: [f64; 4], conf: f64) -> PageLine {
        PageLine {
            bbox,
            text: text.into(),
            conf,
        }
    }

    #[test]
    fn blocks_become_paragraphs_and_empty_lines_are_left_out() {
        let reading = PageReading {
            rotation: 0,
            skew: 0.0,
            lines: vec![
                line("Heading", [10.2, 10.0, 100.7, 20.0], 0.99),
                line("", [10.0, 30.0, 100.0, 40.0], 0.3),
                line("First line", [10.0, 50.0, 100.0, 60.0], 0.951),
                line("second line", [10.0, 62.0, 100.0, 72.0], 0.9),
            ],
            blocks: vec![
                PageBlock {
                    class: BlockClass::SectionHeader,
                    bbox: [10.0, 10.0, 100.0, 40.0],
                    text: "Heading\n".into(),
                    lines: vec![0, 1],
                },
                PageBlock {
                    class: BlockClass::Text,
                    bbox: [10.0, 50.0, 100.0, 72.0],
                    text: "First line\nsecond line".into(),
                    lines: vec![2, 3],
                },
            ],
            markdown: String::new(),
            text: String::new(),
        };
        let result = to_result(&reading, &[], &[], Some(1), 90, 70, 270);
        assert_eq!(result.text, "Heading\n\nFirst line\nsecond line");
        assert_eq!(result.rotation_degrees, 270);
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.lines[0].bbox, [10, 10, 90, 20]);
        assert_eq!(result.lines[1].confidence, 95);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(result.blocks[0].label, OcrBlockLabel::SectionHeader);
        assert_eq!(result.blocks[0].lines, vec![0]);
        assert_eq!(result.blocks[1].lines, vec![1, 2]);
        assert_eq!(result.blocks[1].bbox, [10, 50, 90, 70]);

        // A page without layout: lines one to a line.
        let loose = PageReading {
            blocks: Vec::new(),
            ..reading
        };
        assert_eq!(
            to_result(&loose, &[], &[], Some(1), 200, 200, 0).text,
            "Heading\nFirst line\nsecond line"
        );
    }

    /// A table's lines are reported as their cells, each its own line in
    /// the block; the page's text keeps each line whole.
    #[test]
    fn table_lines_are_reported_as_their_cells() {
        let reading = PageReading {
            rotation: 0,
            skew: 0.0,
            lines: vec![
                line("Prices", [10.0, 5.0, 60.0, 15.0], 0.97),
                line("Item  Price", [10.0, 20.0, 120.0, 30.0], 0.96),
                line("Tea  2.50", [10.0, 32.0, 120.0, 42.0], 0.913),
            ],
            blocks: vec![
                PageBlock {
                    class: BlockClass::SectionHeader,
                    bbox: [10.0, 5.0, 60.0, 15.0],
                    text: "Prices".into(),
                    lines: vec![0],
                },
                PageBlock {
                    class: BlockClass::Table,
                    bbox: [10.0, 20.0, 120.0, 42.0],
                    text: "Item  Price\nTea  2.50".into(),
                    lines: vec![1, 2],
                },
            ],
            markdown: String::new(),
            text: String::new(),
        };
        let cell = |line: usize, text: &str, bbox: [f64; 4]| TableCell {
            line,
            bbox,
            text: text.into(),
        };
        let cells = vec![
            None,
            Some(vec![
                cell(1, "Item", [10.0, 20.0, 40.2, 30.0]),
                cell(1, "Price", [80.0, 20.0, 120.0, 30.0]),
                cell(2, "Tea", [10.0, 32.0, 35.0, 42.0]),
                cell(2, " ", [50.0, 32.0, 55.0, 42.0]),
                cell(2, "2.50", [85.5, 32.0, 120.0, 42.0]),
            ]),
        ];
        let result = to_result(&reading, &cells, &[], Some(1), 200, 100, 0);
        let texts: Vec<&str> = result.lines.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(texts, ["Prices", "Item", "Price", "Tea", "2.50"]);
        assert_eq!(result.lines[1].bbox, [10, 20, 41, 30]);
        assert_eq!(result.lines[4].bbox, [85, 32, 120, 42]);
        assert_eq!(result.lines[4].confidence, 91);
        assert_eq!(result.blocks[0].lines, vec![0]);
        assert_eq!(result.blocks[1].label, OcrBlockLabel::Table);
        assert_eq!(result.blocks[1].lines, vec![1, 2, 3, 4]);
        assert_eq!(result.text, "Prices\n\nItem  Price\nTea  2.50");
    }

    #[test]
    fn colour_is_grayed_with_opencvs_weights() {
        let rgb = image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 10, 200, 30]).unwrap();
        let rgba = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 0, 10, 200, 30, 9]).unwrap();
        let expected = imgops::gray_from_rgb(rgb.as_raw(), 2, 1);
        assert_eq!(to_gray(&DynamicImage::ImageRgb8(rgb)), expected);
        assert_eq!(to_gray(&DynamicImage::ImageRgba8(rgba)), expected);
        let luma16 =
            image::ImageBuffer::<image::Luma<u16>, _>::from_raw(2, 1, vec![0xff00, 0x01ff])
                .unwrap();
        assert_eq!(
            to_gray(&DynamicImage::ImageLuma16(luma16)).data,
            vec![255, 1]
        );
    }

    #[test]
    fn argmax_takes_the_first_largest() {
        assert_eq!(argmax(&[0.1, 0.4, 0.4, 0.1]), (1, 0.4));
        assert_eq!(argmax(&[]), (0, 0.0));
    }
}

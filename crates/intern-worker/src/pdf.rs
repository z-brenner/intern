use std::path::Path;
#[cfg(feature = "native-pdfium")]
use std::path::PathBuf;
#[cfg(feature = "native-pdfium")]
use std::time::Instant;

use crate::extract::{
    CancellationToken, ExtractionError, PdfBackend, PdfPageInspection, RenderedPage,
};
#[cfg(feature = "native-pdfium")]
use crate::layout::router::bounds::{
    MAX_DOCUMENT_RUNS, MAX_FORM_DEPTH, MAX_IMAGES, MAX_RULINGS, MAX_RUNS, MAX_SEGMENTS,
    MAX_SURVEY_OBJECTS,
};
#[cfg(feature = "native-pdfium")]
use crate::layout::{NativePage, TextRun, measure_signals, router::needs_runs};
#[cfg(feature = "native-pdfium")]
use crate::limits::{MAX_DOCUMENT_CHARS, MAX_PAGE_COUNT, render_size_within};
#[cfg(feature = "native-pdfium")]
use crate::timing::micros_since;

#[cfg(feature = "native-pdfium")]
use pdfium_render::prelude::*;

/// PDFium initialises itself once per process. Binding it again - which is what
/// building a backend per document used to do - fails on the second attempt and
/// can abort the process outright, so the binding is created exactly once and
/// every backend shares it.
#[cfg(feature = "native-pdfium")]
static PDFIUM: std::sync::OnceLock<Result<Pdfium, String>> = std::sync::OnceLock::new();

/// Only [`PdfiumBackend::new`] makes one, so holding one means PDFium bound.
///
/// A backend keeps the last document it opened, so inspecting a document and
/// rendering its scanned pages share one open: the PDF is parsed once per
/// extraction, not once more for every page that goes to OCR. PDFium is not
/// thread-safe, and the cell makes that a compile-time fact: a backend
/// cannot be shared between threads, so every render happens on the thread
/// that inspected the document.
#[cfg(feature = "native-pdfium")]
pub struct PdfiumBackend {
    open: std::cell::RefCell<Option<OpenDocument>>,
}

#[cfg(feature = "native-pdfium")]
struct OpenDocument {
    path: PathBuf,
    document: PdfDocument<'static>,
}

#[cfg(feature = "native-pdfium")]
impl PdfiumBackend {
    pub fn new(library_directory: impl AsRef<Path>) -> Result<Self, ExtractionError> {
        let library_path = Pdfium::pdfium_platform_library_name_at_path(&library_directory);
        if !library_path.exists() {
            return Err(ExtractionError::native_assets_missing(format!(
                "PDFium library is absent at {}",
                library_path.display()
            )));
        }
        match PDFIUM.get_or_init(|| {
            Pdfium::bind_to_library(&library_path)
                .map(Pdfium::new)
                .map_err(|error| format!("PDFium did not load: {error:?}"))
        }) {
            Ok(_) => Ok(Self {
                open: std::cell::RefCell::new(None),
            }),
            Err(message) => Err(ExtractionError::native_assets_missing(message.clone())),
        }
    }

    fn pdfium(&self) -> Result<&'static Pdfium, ExtractionError> {
        match PDFIUM.get() {
            Some(Ok(pdfium)) => Ok(pdfium),
            Some(Err(message)) => Err(ExtractionError::native_assets_missing(message.clone())),
            None => Err(ExtractionError::native_assets_missing(
                "PDFium was not initialised",
            )),
        }
    }

    /// Runs `work` on the document at `path`, opening it only if it is not
    /// the one already open.
    fn with_document<T>(
        &self,
        path: &Path,
        work: impl FnOnce(&PdfDocument<'static>) -> Result<T, ExtractionError>,
    ) -> Result<T, ExtractionError> {
        let mut open = self.open.borrow_mut();
        if open.as_ref().is_none_or(|open| open.path != path) {
            // The previous document is closed before the next is opened.
            *open = None;
            let document = self
                .pdfium()?
                .load_pdf_from_file(path, None)
                .map_err(load_error)?;
            *open = Some(OpenDocument {
                path: path.to_path_buf(),
                document,
            });
        }
        work(&open.as_ref().expect("opened above").document)
    }
}

/// Why PDFium would not open a document, as something a person can act on.
///
/// A PDF that needs a password to open is the one failure worth naming. The
/// rest are reported in one line: `PdfiumError`'s `Display` is its
/// pretty-printed `Debug`, which spreads one enum variant over several lines.
#[cfg(feature = "native-pdfium")]
fn load_error(error: PdfiumError) -> ExtractionError {
    match error {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => {
            ExtractionError::encrypted()
        }
        other => one_line(&other),
    }
}

#[cfg(feature = "native-pdfium")]
fn one_line(error: &PdfiumError) -> ExtractionError {
    ExtractionError::parse_failed(format!("PDFium could not read the document: {error:?}"))
}

/// A page's own frame: the box its content is drawn in, before `/Rotate`.
/// Converts PDF user space (points, origin bottom left) into tenths of a
/// point with the origin at the frame's top left.
#[cfg(feature = "native-pdfium")]
#[derive(Clone, Copy, Debug)]
struct Frame {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

#[cfg(feature = "native-pdfium")]
impl Frame {
    fn of(page: &PdfPage<'_>, rotation: u16) -> Self {
        let bounds = page
            .boundaries()
            .bounding()
            .map(|boundary| boundary.bounds)
            .ok()
            .filter(|bounds| bounds.width().value > 0.0 && bounds.height().value > 0.0);
        match bounds {
            Some(bounds) => Self {
                left: bounds.left().value,
                top: bounds.top().value,
                width: bounds.width().value,
                height: bounds.height().value,
            },
            None => {
                // The displayed size, turned back into the frame's.
                let (width, height) = if rotation % 180 == 90 {
                    (page.height().value, page.width().value)
                } else {
                    (page.width().value, page.height().value)
                };
                Self {
                    left: 0.0,
                    top: height,
                    width,
                    height,
                }
            }
        }
    }

    fn units(value: f32) -> u32 {
        (f64::from(value) * crate::layout::UNITS_PER_POINT)
            .round()
            .max(0.0) as u32
    }

    fn rect(&self, rect: &PdfRect) -> [u32; 4] {
        self.points(
            rect.left().value,
            rect.bottom().value,
            rect.right().value,
            rect.top().value,
        )
    }

    /// `left, bottom, right, top` in user space as a frame box.
    fn points(&self, left: f32, bottom: f32, right: f32, top: f32) -> [u32; 4] {
        let clamp_x = |value: f32| (value - self.left).clamp(0.0, self.width);
        let clamp_y = |value: f32| (self.top - value).clamp(0.0, self.height);
        let (x0, x1) = (clamp_x(left.min(right)), clamp_x(left.max(right)));
        let (y0, y1) = (clamp_y(top.max(bottom)), clamp_y(top.min(bottom)));
        [
            Self::units(x0),
            Self::units(y0),
            Self::units(x1),
            Self::units(y1),
        ]
    }
}

/// What walking a page's objects finds: the area its images cover, their
/// boxes, how many text objects it has and how many are invisible, and its
/// rules.
#[cfg(feature = "native-pdfium")]
#[derive(Default)]
struct ObjectSurvey {
    image_area: f32,
    /// The images the router reads, set by [`ObjectSurvey::bounded`].
    images: Vec<[u32; 4]>,
    /// The largest images drawn so far, at most [`MAX_IMAGES`], each with
    /// its area and its place in drawing order.
    largest_images: Vec<(u64, usize, [u32; 4])>,
    images_drawn: usize,
    /// Objects visited, forms' children included.
    visited: usize,
    /// The page had more objects than [`MAX_SURVEY_OBJECTS`], or forms
    /// nested deeper than [`MAX_FORM_DEPTH`], and the walk stopped: what it
    /// found describes only part of the page.
    incomplete: bool,
    text_objects: u32,
    invisible_text_objects: u32,
    /// The box of each text object, in content-stream order: the runs of
    /// text the producer placed, one style each.
    text_boxes: Vec<[u32; 4]>,
    rulings: Vec<[u32; 4]>,
}

/// A path this thin is a rule, not a shape.
#[cfg(feature = "native-pdfium")]
const RULE_THICKNESS: f32 = 2.0;
/// ...and this long.
#[cfg(feature = "native-pdfium")]
const RULE_LENGTH: f32 = 10.0;

/// An affine map from an object's own space to the page's user space,
/// `[a, b, c, d, e, f]` as PDF writes them: `x' = a x + c y + e`,
/// `y' = b x + d y + f`.
#[cfg(feature = "native-pdfium")]
type Placement = [f32; 6];

#[cfg(feature = "native-pdfium")]
const ON_THE_PAGE: Placement = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// `inner` followed by `outer`: where a form's child lands on the page.
#[cfg(feature = "native-pdfium")]
fn placed_within(inner: &Placement, outer: &Placement) -> Placement {
    let [a, b, c, d, e, f] = *inner;
    let [oa, ob, oc, od, oe, of] = *outer;
    [
        oa * a + oc * b,
        ob * a + od * b,
        oa * c + oc * d,
        ob * c + od * d,
        oa * e + oc * f + oe,
        ob * e + od * f + of,
    ]
}

/// A box in an object's own space, `left, bottom, right, top`, placed on the
/// page: the box around its four placed corners.
#[cfg(feature = "native-pdfium")]
fn place(placement: &Placement, rect: &PdfRect) -> [f32; 4] {
    let [a, b, c, d, e, f] = *placement;
    let (left, bottom, right, top) = (
        rect.left().value,
        rect.bottom().value,
        rect.right().value,
        rect.top().value,
    );
    let corners = [(left, bottom), (right, bottom), (left, top), (right, top)]
        .map(|(x, y)| (a * x + c * y + e, b * x + d * y + f));
    let xs = corners.map(|corner| corner.0);
    let ys = corners.map(|corner| corner.1);
    [
        xs.iter().copied().fold(f32::INFINITY, f32::min),
        ys.iter().copied().fold(f32::INFINITY, f32::min),
        xs.iter().copied().fold(f32::NEG_INFINITY, f32::max),
        ys.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    ]
}

/// The map a form XObject draws its children with, from its own space to
/// its parent's.
#[cfg(feature = "native-pdfium")]
fn form_placement(object: &PdfPageObject<'_>) -> Placement {
    object.matrix().map_or(ON_THE_PAGE, |matrix| {
        [
            matrix.a(),
            matrix.b(),
            matrix.c(),
            matrix.d(),
            matrix.e(),
            matrix.f(),
        ]
    })
}

/// Surveys one object drawn with `placement`. PDFium gives an object's
/// bounds in the space of whatever holds it: the page, or the form XObject
/// it is drawn in. A form's children are placed on the page through the
/// form's own map, so a page a tool wrapped whole in one form - an imported
/// page from iText, pdfpages, or macOS - is surveyed as the page it is: its
/// text objects are text boxes, its logo an image the size of the logo.
///
/// The walk is bounded: past [`MAX_SURVEY_OBJECTS`] objects, or forms nested
/// past [`MAX_FORM_DEPTH`], it stops and marks the survey incomplete.
#[cfg(feature = "native-pdfium")]
fn survey_object(
    object: &PdfPageObject<'_>,
    placement: &Placement,
    frame: &Frame,
    survey: &mut ObjectSurvey,
    depth: usize,
) {
    if survey.incomplete {
        return;
    }
    survey.visited += 1;
    if survey.visited > MAX_SURVEY_OBJECTS || depth > MAX_FORM_DEPTH {
        survey.incomplete = true;
        return;
    }
    if let Some(form) = object.as_x_object_form_object() {
        let inner = placed_within(&form_placement(object), placement);
        for index in form.as_range() {
            if survey.incomplete {
                return;
            }
            if let Ok(child) = form.get(index) {
                survey_object(&child, &inner, frame, survey, depth + 1);
            }
        }
        return;
    }
    let Some(bounds) = object.bounds().map(|bounds| bounds.to_rect()).ok() else {
        if let Some(text) = object.as_text_object() {
            survey.text_objects += 1;
            if text.render_mode() == PdfPageTextRenderMode::Invisible {
                survey.invisible_text_objects += 1;
            }
        }
        return;
    };
    let [left, bottom, right, top] = place(placement, &bounds);
    if let Some(text) = object.as_text_object() {
        survey.text_objects += 1;
        if text.render_mode() == PdfPageTextRenderMode::Invisible {
            survey.invisible_text_objects += 1;
        }
        let bbox = frame.points(left, bottom, right, top);
        // One past the router's bound is enough to say the page has more.
        if bbox[2] > bbox[0] && bbox[3] > bbox[1] && survey.text_boxes.len() <= MAX_SEGMENTS {
            survey.text_boxes.push(bbox);
        }
        return;
    }
    if object.as_image_object().is_some() {
        survey.image_area += (right - left).abs() * (top - bottom).abs();
        survey.keep_image(frame.points(left, bottom, right, top));
        return;
    }
    if let Some(path) = object.as_path_object() {
        survey_path(path, [left, bottom, right, top], frame, survey);
    }
}

/// A path's rules: a thin line across or down, or the four sides of an
/// outlined box, from its box on the page - placed through any form it is
/// drawn in, like text and images, so a ruled table inside a form is ruled
/// where it is drawn.
#[cfg(feature = "native-pdfium")]
fn survey_path(
    path: &PdfPagePathObject<'_>,
    [left, bottom, right, top]: [f32; 4],
    frame: &Frame,
    survey: &mut ObjectSurvey,
) {
    let width = (right - left).abs();
    let height = (top - bottom).abs();
    let thin_across = height <= RULE_THICKNESS && width >= RULE_LENGTH;
    let thin_down = width <= RULE_THICKNESS && height >= RULE_LENGTH;
    if thin_across || thin_down {
        survey.keep_ruling(frame.points(left, bottom, right, top));
    } else if path.is_stroked().unwrap_or(false)
        && path
            .fill_mode()
            .is_ok_and(|mode| mode == PdfPathFillMode::None)
        && width >= RULE_LENGTH
        && height >= RULE_LENGTH
    {
        // An outlined box: its four sides are rules.
        survey.keep_ruling(frame.points(left, top - 0.5, right, top));
        survey.keep_ruling(frame.points(left, bottom, right, bottom + 0.5));
        survey.keep_ruling(frame.points(left, bottom, left + 0.5, top));
        survey.keep_ruling(frame.points(right - 0.5, bottom, right, top));
    }
}

#[cfg(feature = "native-pdfium")]
impl ObjectSurvey {
    /// Keeps an image while it is among the [`MAX_IMAGES`] largest drawn so
    /// far, as the walk goes: a page of a million small images holds 64 of
    /// them, never all. Of images the same size, the earlier drawn is kept.
    fn keep_image(&mut self, image: [u32; 4]) {
        let area = u64::from(image[2].saturating_sub(image[0]))
            * u64::from(image[3].saturating_sub(image[1]));
        let drawn = self.images_drawn;
        self.images_drawn += 1;
        if self.largest_images.len() < MAX_IMAGES {
            self.largest_images.push((area, drawn, image));
            return;
        }
        // The one the bound would drop first: the smallest, and of the
        // smallest the latest drawn.
        let Some((weakest, &(weakest_area, ..))) = self
            .largest_images
            .iter()
            .enumerate()
            .min_by_key(|(_, (area, drawn, _))| (*area, std::cmp::Reverse(*drawn)))
        else {
            return;
        };
        if area > weakest_area {
            self.largest_images[weakest] = (area, drawn, image);
        }
    }

    /// Keeps a rule while the page has fewer than [`MAX_RULINGS`]: the
    /// first ones drawn, as the walk goes.
    fn keep_ruling(&mut self, ruling: [u32; 4]) {
        if self.rulings.len() < MAX_RULINGS {
            self.rulings.push(ruling);
        }
    }

    /// What the survey keeps, held to the router's bounds as it walked: the
    /// largest images, largest first when there were more than the bound
    /// and in drawing order otherwise, and the first rules. The image area
    /// still counts every image.
    fn bounded(mut self) -> Self {
        if self.images_drawn > MAX_IMAGES {
            self.largest_images
                .sort_by_key(|&(area, drawn, _)| (std::cmp::Reverse(area), drawn));
        }
        self.images = std::mem::take(&mut self.largest_images)
            .into_iter()
            .map(|(_, _, image)| image)
            .collect();
        self
    }
}

/// The page's text as runs: characters on one baseline with no gap wider
/// than most of their height between them.
///
/// Each character is asked for its box, its value, whether PDFium generated
/// it, and its weight - four calls - which is why only pages routed to a
/// geometry route are read this way.
///
/// Lines are found on the page as it is displayed: text on a page turned by
/// `/Rotate` runs across the page's own frame, one character above the
/// next, so each character and segment is turned first, and each run is
/// turned back into the page's frame once it is built.
///
/// A page with more runs than the layout analysis takes on
/// ([`MAX_RUNS`]) has none: it is read as its text, and its characters past
/// that are not asked for.
#[cfg(feature = "native-pdfium")]
fn text_runs(text: &PdfPageText<'_>, frame: &Frame, native: &NativePage) -> Vec<TextRun> {
    let (display_width, display_height) = native.display_size();
    let segments = native
        .segments
        .iter()
        .map(|segment| native.to_display(*segment))
        .collect::<Vec<_>>();
    let segments = segments.as_slice();
    let mut runs = character_runs(text, frame, native, segments);
    for run in &mut runs {
        run.bbox = crate::layout::router::from_display(
            run.bbox,
            native.rotation,
            display_width,
            display_height,
        );
    }
    runs
}

/// [`text_runs`] on the page as displayed.
#[cfg(feature = "native-pdfium")]
fn character_runs(
    text: &PdfPageText<'_>,
    frame: &Frame,
    native: &NativePage,
    segments: &[[u32; 4]],
) -> Vec<TextRun> {
    struct Building {
        text: String,
        bbox: [f64; 4],
        /// The run's first character and, if it has more, its last.
        first: usize,
        last: Option<usize>,
    }
    // A run is bold when both its ends are set in a bold face: a label set
    // in bold, not a paragraph that opens with a bold clause number.
    // PDFium's weight is unreliable for the standard fonts, whose names say
    // it instead, and a face's name costs two calls and an allocation, so
    // it is asked of a run's ends rather than of every character.
    //
    // Returns whether a run was kept: one with text, with width or not.
    fn finish(
        building: Option<Building>,
        chars: &PdfPageTextChars<'_>,
        runs: &mut Vec<TextRun>,
    ) -> bool {
        let Some(building) = building else {
            return false;
        };
        let text = building.text.trim_end().to_owned();
        if text.is_empty() {
            return false;
        }
        let bold_at = |index: usize| chars.get(index).is_ok_and(|character| is_bold(&character));
        let bold = bold_at(building.first) && building.last.is_none_or(bold_at);
        let bbox = building.bbox.map(|value| value.round().max(0.0) as u32);
        runs.push(TextRun {
            text,
            bbox,
            bold,
            confidence: None,
        });
        true
    }
    let mut runs = Vec::new();
    // Runs kept so far. The analysis reads only those with width, and past
    // MAX_RUNS none at all; a page of glyphs drawn with no width - text at
    // zero horizontal scale - would otherwise keep a run for every line of
    // them that it never reads. Every run kept counts, so a page holds at
    // most MAX_RUNS of them.
    let mut read = 0_usize;
    let mut current: Option<Building> = None;
    let mut pending_space = false;
    let mut segment = 0_usize;
    let chars = text.chars();
    for character in chars.iter() {
        let index = character.index();
        let Some(value) = character.unicode_char() else {
            continue;
        };
        if value == '\r' || value == '\n' {
            read += usize::from(finish(current.take(), &chars, &mut runs));
            if read > MAX_RUNS {
                return Vec::new();
            }
            pending_space = false;
            continue;
        }
        if value.is_whitespace() {
            pending_space = true;
            continue;
        }
        if character.is_generated().unwrap_or(false) {
            continue;
        }
        let Ok(bounds) = character.loose_bounds() else {
            continue;
        };
        let bbox = native.to_display(frame.rect(&bounds)).map(f64::from);
        if bbox[3] <= bbox[1] {
            continue;
        }
        let height = bbox[3] - bbox[1];
        // Segments are PDFium's text objects in the order their characters
        // come, so a character outside the current one and inside one of the
        // next few starts it: a piece of text the producer placed on its
        // own.
        let center = ((bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0);
        let mut new_object = false;
        if let Some(next) = crate::layout::next_segment(segments, segment, center) {
            segment = next;
            new_object = true;
        }
        let joins = current.as_ref().is_some_and(|run| {
            let run_height = run.bbox[3] - run.bbox[1];
            let middle = (bbox[1] + bbox[3]) / 2.0;
            let run_middle = (run.bbox[1] + run.bbox[3]) / 2.0;
            let same_line = (middle - run_middle).abs() <= height.min(run_height) * 0.4;
            let gap = bbox[0] - run.bbox[2];
            // Words of one text object run together across ordinary
            // spaces; separate objects only across a space or less. Two
            // objects further apart than that are two cells, however
            // narrow the gap - a date that overflows its column into the
            // next one's value.
            let widest = if new_object { 0.25 } else { 0.8 };
            same_line
                && gap >= -height.max(run_height) * 0.3
                && gap <= height.max(run_height) * widest
        });
        if joins {
            let run = current.as_mut().expect("checked above");
            let gap = bbox[0] - run.bbox[2];
            if pending_space || gap > (run.bbox[3] - run.bbox[1]) * 0.15 {
                run.text.push(' ');
            }
            run.text.push(value);
            run.bbox = [
                run.bbox[0].min(bbox[0]),
                run.bbox[1].min(bbox[1]),
                run.bbox[2].max(bbox[2]),
                run.bbox[3].max(bbox[3]),
            ];
            run.last = Some(index);
        } else {
            read += usize::from(finish(current.take(), &chars, &mut runs));
            if read > MAX_RUNS {
                return Vec::new();
            }
            current = Some(Building {
                text: value.to_string(),
                bbox,
                first: index,
                last: None,
            });
        }
        pending_space = false;
    }
    read += usize::from(finish(current.take(), &chars, &mut runs));
    if read > MAX_RUNS {
        return Vec::new();
    }
    runs
}

/// Whether a character is set in a bold face, by its weight or, for the
/// fonts whose weight PDFium does not know, by the face's name.
#[cfg(feature = "native-pdfium")]
fn is_bold(character: &PdfPageTextChar<'_>) -> bool {
    if character
        .font_weight()
        .is_some_and(|weight| font_weight(weight) >= 600)
    {
        return true;
    }
    let name = character.font_name().to_ascii_lowercase();
    ["bold", "black", "heavy", "semibold", "demi"]
        .iter()
        .any(|marker| name.contains(marker))
}

#[cfg(feature = "native-pdfium")]
fn font_weight(weight: PdfFontWeight) -> u32 {
    match weight {
        PdfFontWeight::Weight100 => 100,
        PdfFontWeight::Weight200 => 200,
        PdfFontWeight::Weight300 => 300,
        PdfFontWeight::Weight400Normal => 400,
        PdfFontWeight::Weight500 => 500,
        PdfFontWeight::Weight600 => 600,
        PdfFontWeight::Weight700Bold => 700,
        PdfFontWeight::Weight800 => 800,
        PdfFontWeight::Weight900 => 900,
        PdfFontWeight::Custom(value) => value,
    }
}

/// Every character on the page, in the order the document defines them.
///
/// PDFium bounds a page's text by the page's size as displayed, but its
/// characters sit where the content stream drew them, before `/Rotate`
/// turns the page. On a page turned a quarter - a landscape rate sheet
/// stored as a portrait one - the displayed width is the drawn height, and
/// every character past it was lost: the title cut to "CARRIER RA", the
/// charges and the confirmation date gone. Such a page is bounded by its
/// size as drawn. Every other page is read exactly as it always was.
#[cfg(feature = "native-pdfium")]
fn page_text(page: &PdfPage<'_>, text: &PdfPageText<'_>, rotation: u16) -> String {
    if rotation % 180 == 90 {
        text.inside_rect(PdfRect::new(
            PdfPoints::ZERO,
            PdfPoints::ZERO,
            page.width(),
            page.height(),
        ))
    } else {
        text.all()
    }
}

#[cfg(feature = "native-pdfium")]
fn rotation_degrees(page: &PdfPage<'_>) -> u16 {
    match page.rotation() {
        Ok(PdfPageRenderRotation::Degrees90) => 90,
        Ok(PdfPageRenderRotation::Degrees180) => 180,
        Ok(PdfPageRenderRotation::Degrees270) => 270,
        _ => 0,
    }
}

#[cfg(feature = "native-pdfium")]
impl PdfiumBackend {
    /// [`PdfBackend::inspect`], with the characters of the pages `router`
    /// sends down a geometry route - decided from each page's signals, and
    /// whether the scan rule sends it to OCR - read into runs as well, while
    /// the runs read stay within `run_budget` for the document. A page past
    /// it has its characters read when it is read
    /// ([`PdfBackend::page_runs`]), so what a document holds at once is
    /// bounded, and a page within it is not loaded and read a second time.
    /// The routing calibration in `examples/route_calibration.rs` reads
    /// every page's runs here, to see what the router passed over.
    pub fn inspect_routed(
        &self,
        path: &Path,
        cancel: &CancellationToken,
        router: impl Fn(&crate::layout::RouteSignals, bool) -> crate::layout::PageRoute,
        run_budget: usize,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        cancel.check()?;
        // Loading the document, each page, and its text is reading the
        // format; walking a page's objects and measuring its text for the
        // router is deciding how to read it. The second is timed on its own
        // and the first is what is left, so the page loads PDFium does
        // lazily are counted too.
        let started = Instant::now();
        let mut analysis_micros = 0_u64;
        let mut runs_held = 0_usize;
        // The characters of the pages read ahead into runs: bounded too, as
        // a run holds its text however few runs there are.
        let mut characters_held = 0_usize;
        let inspections = self.with_document(path, |document| {
            if document.pages().len() as usize > MAX_PAGE_COUNT {
                return Err(ExtractionError::resource_limit(
                    "document exceeds 500 pages",
                ));
            }
            let mut inspections = Vec::with_capacity(document.pages().len() as usize);
            for (page_index, page) in document.pages().iter().enumerate() {
                cancel.check()?;
                let text = page.text().map_err(|error| one_line(&error))?;
                let rotation = rotation_degrees(&page);
                let native_text = page_text(&page, &text, rotation);
                // The size at full resolution, unbudgeted: whether a page fits
                // the render cap is the caller's question to ask of it.
                let size = render_size_within(page.width().value, page.height().value, u64::MAX);
                let (width_pixels, height_pixels) = (size.width, size.height);
                let analysis_started = Instant::now();
                let frame = Frame::of(&page, rotation);
                let mut survey = ObjectSurvey::default();
                for object in page.objects().iter() {
                    if survey.incomplete {
                        break;
                    }
                    survey_object(&object, &ON_THE_PAGE, &frame, &mut survey, 0);
                }
                let mut survey = survey.bounded();
                // A page the survey could not finish is not measured for
                // structure: it keeps its text, and nothing is built from a
                // geometry seen only in part.
                let measured = !survey.incomplete;
                if !measured {
                    // What was counted before the walk stopped is a part of
                    // the page, not a share of it: routed on its text alone.
                    // The image area seen is kept - a lower bound on the
                    // page's, it can only make a page with no text to read
                    // less likely to be taken for a scan.
                    survey.text_boxes.clear();
                    survey.images.clear();
                    survey.rulings.clear();
                    survey.text_objects = 0;
                    survey.invisible_text_objects = 0;
                }
                let page_area = page.width().value.abs() * page.height().value.abs();
                let image_coverage = if page_area <= f32::EPSILON {
                    0.0
                } else {
                    (survey.image_area / page_area).clamp(0.0, 1.0)
                };
                // PDFium's own text segments are the same boxes, but each one
                // asked for recounts them all: a page's text objects are its
                // segments at the cost of the walk above.
                let mut native = NativePage {
                    width: Frame::units(frame.width),
                    height: Frame::units(frame.height),
                    rotation,
                    segments: survey.text_boxes,
                    text_objects: survey.text_objects,
                    invisible_text_objects: survey.invisible_text_objects,
                    images: survey.images,
                    rulings: survey.rulings,
                    runs: Vec::new(),
                };
                let signals = measure_signals(&native, &native_text, image_coverage);
                let mut inspection = PdfPageInspection {
                    page_index,
                    native_text,
                    image_coverage,
                    width_pixels,
                    height_pixels,
                    native: None,
                    signals: Some(signals),
                };
                let route = router(&signals, crate::extract::page_needs_ocr(&inspection));
                if measured
                    && needs_runs(route)
                    && runs_held < run_budget
                    && characters_held < MAX_DOCUMENT_CHARS
                    && crate::extract::fits_a_page(&inspection.native_text)
                {
                    native.runs = text_runs(&text, &frame, &native);
                    runs_held = runs_held.saturating_add(native.runs.len());
                    characters_held =
                        characters_held.saturating_add(inspection.native_text.chars().count());
                }
                inspection.native = measured.then_some(native);
                analysis_micros = analysis_micros.saturating_add(micros_since(analysis_started));
                inspections.push(inspection);
            }
            Ok(inspections)
        })?;
        let parse_micros = micros_since(started).saturating_sub(analysis_micros);
        cancel.record(|timings| {
            timings.parse_micros = timings.parse_micros.saturating_add(parse_micros);
            timings.analysis_micros = timings.analysis_micros.saturating_add(analysis_micros);
        });
        Ok(inspections)
    }
}

#[cfg(feature = "native-pdfium")]
impl PdfBackend for PdfiumBackend {
    fn inspect(
        &self,
        path: &Path,
        cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        self.inspect_routed(path, cancel, crate::layout::route_page, MAX_DOCUMENT_RUNS)
    }

    fn page_runs(
        &self,
        path: &Path,
        page_index: usize,
        native: &NativePage,
        cancel: &CancellationToken,
    ) -> Result<Vec<TextRun>, ExtractionError> {
        cancel.check()?;
        self.with_document(path, |document| {
            let page = document
                .pages()
                .get(page_index as i32)
                .map_err(|error| one_line(&error))?;
            let text = page.text().map_err(|error| one_line(&error))?;
            let frame = Frame::of(&page, native.rotation);
            Ok(text_runs(&text, &frame, native))
        })
    }

    fn render_within(
        &self,
        path: &Path,
        page_index: usize,
        max_pixels: u64,
        cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        cancel.check()?;
        let image = self.with_document(path, |document| {
            let page = document
                .pages()
                .get(page_index as i32)
                .map_err(|error| one_line(&error))?;
            let size = render_size_within(page.width().value, page.height().value, max_pixels);
            // PDFium scales to the target width and then, if the height that
            // gives passes the maximum, scales down to that instead, so the
            // bitmap it allocates is never larger than the size computed here.
            let config = PdfRenderConfig::new()
                .set_target_width(size.width as i32)
                .set_maximum_height(size.height as i32);
            Ok(page
                .render_with_config(&config)
                .map_err(|error| one_line(&error))?
                .as_image()
                .map_err(|error| one_line(&error))?
                .into_rgb8())
        })?;
        cancel.check()?;
        Ok(RenderedPage::new(page_index, image.into()))
    }
}

#[cfg(not(feature = "native-pdfium"))]
#[derive(Clone, Debug, Default)]
pub struct PdfiumBackend;

#[cfg(not(feature = "native-pdfium"))]
impl PdfiumBackend {
    pub fn new(_library_directory: impl AsRef<Path>) -> Result<Self, ExtractionError> {
        Err(ExtractionError::native_assets_missing(
            "intern-worker was built without the native-pdfium feature",
        ))
    }
}

#[cfg(not(feature = "native-pdfium"))]
impl PdfBackend for PdfiumBackend {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        Err(ExtractionError::native_assets_missing(
            "PDFium support is unavailable",
        ))
    }

    fn render_within(
        &self,
        _path: &Path,
        _page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        Err(ExtractionError::native_assets_missing(
            "PDFium support is unavailable",
        ))
    }
}

#[cfg(all(test, feature = "native-pdfium"))]
mod survey_bounds {
    use super::*;

    /// Image boxes of a few sizes, many the same, from a fixed sequence.
    fn images(count: usize) -> Vec<[u32; 4]> {
        let mut state = 7_u32;
        (0..count)
            .map(|_| {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                let side = 10 + (state >> 16) % 40;
                let left = (state >> 8) % 500;
                [left, 0, left + side, side]
            })
            .collect()
    }

    /// Every image collected first, then the largest kept, the earlier of a
    /// size first: what the survey did before it bounded as it walked.
    fn collected_then_bounded(images: &[[u32; 4]]) -> Vec<[u32; 4]> {
        let mut all = images.to_vec();
        if all.len() > MAX_IMAGES {
            all.sort_by_key(|image| {
                std::cmp::Reverse(
                    u64::from(image[2].saturating_sub(image[0]))
                        * u64::from(image[3].saturating_sub(image[1])),
                )
            });
            all.truncate(MAX_IMAGES);
        }
        all
    }

    #[test]
    fn a_page_of_many_images_holds_the_largest_as_it_walks_and_keeps_the_same_ones() {
        for count in [
            0,
            1,
            MAX_IMAGES - 1,
            MAX_IMAGES,
            MAX_IMAGES + 1,
            1_000,
            20_000,
        ] {
            let drawn = images(count);
            let mut survey = ObjectSurvey::default();
            for image in &drawn {
                survey.keep_image(*image);
                assert!(survey.largest_images.len() <= MAX_IMAGES);
            }
            assert_eq!(
                survey.bounded().images,
                collected_then_bounded(&drawn),
                "{count} images"
            );
        }
    }

    #[test]
    fn a_page_of_many_rules_holds_the_first_ones() {
        let mut survey = ObjectSurvey::default();
        for index in 0..(MAX_RULINGS as u32 + 5_000) {
            survey.keep_ruling([index, 0, index + 20, 1]);
        }
        let rulings = survey.bounded().rulings;
        assert_eq!(rulings.len(), MAX_RULINGS);
        assert_eq!(rulings[0], [0, 0, 20, 1]);
        assert_eq!(
            rulings[MAX_RULINGS - 1],
            [MAX_RULINGS as u32 - 1, 0, MAX_RULINGS as u32 + 19, 1]
        );
    }
}

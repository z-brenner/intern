use std::path::Path;
#[cfg(feature = "native-pdfium")]
use std::time::Instant;

use crate::extract::{
    CancellationToken, ExtractionError, PdfBackend, PdfPageInspection, RenderedPage,
};
#[cfg(feature = "native-pdfium")]
use crate::limits::{MAX_PAGE_COUNT, render_size_within};
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
#[cfg(feature = "native-pdfium")]
pub struct PdfiumBackend {
    _bound: (),
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
            Ok(_) => Ok(Self { _bound: () }),
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

#[cfg(feature = "native-pdfium")]
fn contains_rendered_image(object: &PdfPageObject<'_>) -> bool {
    if object.as_image_object().is_some() {
        return true;
    }
    if let Some(form) = object.as_x_object_form_object() {
        for index in form.as_range() {
            if form
                .get(index)
                .map(|child| contains_rendered_image(&child))
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "native-pdfium")]
fn rendered_image_area(object: &PdfPageObject<'_>) -> f32 {
    if !contains_rendered_image(object) {
        return 0.0;
    }
    // For a Form XObject, its own transformed bounds describe the stamped area
    // on the containing page. This is conservative for mixed-content forms and
    // avoids incorrectly treating nested child coordinates as page coordinates.
    object
        .bounds()
        .map(|bounds| bounds.width().value.abs() * bounds.height().value.abs())
        .unwrap_or(0.0)
}

#[cfg(feature = "native-pdfium")]
impl PdfBackend for PdfiumBackend {
    fn inspect(
        &self,
        path: &Path,
        cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        cancel.check()?;
        // Loading the document, each page, and its text is reading the
        // format; walking a page's objects for images is deciding whether it
        // is a scan. The second is timed on its own and the first is what
        // is left, so the page loads PDFium does lazily are counted too.
        let started = Instant::now();
        let mut analysis_micros = 0_u64;
        let pdfium = self.pdfium()?;
        let document = pdfium.load_pdf_from_file(path, None).map_err(load_error)?;
        if document.pages().len() as usize > MAX_PAGE_COUNT {
            return Err(ExtractionError::resource_limit(
                "document exceeds 500 pages",
            ));
        }
        let mut inspections = Vec::with_capacity(document.pages().len() as usize);
        for (page_index, page) in document.pages().iter().enumerate() {
            cancel.check()?;
            let native_text = page.text().map_err(|error| one_line(&error))?.all();
            // The size at full resolution, unbudgeted: whether a page fits
            // the render cap is the caller's question to ask of it.
            let size = render_size_within(page.width().value, page.height().value, u64::MAX);
            let (width_pixels, height_pixels) = (size.width, size.height);
            let analysis_started = Instant::now();
            let page_area = page.width().value.abs() * page.height().value.abs();
            let image_area = page
                .objects()
                .iter()
                .map(|object| rendered_image_area(&object))
                .sum::<f32>();
            let image_coverage = if page_area <= f32::EPSILON {
                0.0
            } else {
                (image_area / page_area).clamp(0.0, 1.0)
            };
            analysis_micros = analysis_micros.saturating_add(micros_since(analysis_started));
            inspections.push(PdfPageInspection {
                page_index,
                native_text,
                image_coverage,
                width_pixels,
                height_pixels,
            });
        }
        let parse_micros = micros_since(started).saturating_sub(analysis_micros);
        cancel.record(|timings| {
            timings.parse_micros = timings.parse_micros.saturating_add(parse_micros);
            timings.analysis_micros = timings.analysis_micros.saturating_add(analysis_micros);
        });
        Ok(inspections)
    }

    fn render_within(
        &self,
        path: &Path,
        page_index: usize,
        max_pixels: u64,
        cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        cancel.check()?;
        let pdfium = self.pdfium()?;
        let document = pdfium.load_pdf_from_file(path, None).map_err(load_error)?;
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
        let image = page
            .render_with_config(&config)
            .map_err(|error| one_line(&error))?
            .as_image()
            .map_err(|error| one_line(&error))?
            .into_rgb8();
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

use std::time::Duration;

use crate::extract::ExtractionError;

pub const MAX_SOURCE_BYTES: u64 = 1_073_741_824;
pub const MAX_PAGE_COUNT: usize = 500;
pub const MAX_DECOMPRESSED_OFFICE_BYTES: u64 = 1_073_741_824;
pub const MAX_TEMP_BYTES: u64 = 2_147_483_648;
/// Characters one page may carry into a response.
///
/// Distillation reads a budget of a few tens of thousands of characters, and
/// the largest page any reader produces is a whole Word document rendered as
/// Markdown, so nothing real comes near this. A degenerate file that does is
/// truncated rather than allowed to put hundreds of megabytes through the
/// pipe and into the queue's memory.
pub const MAX_PAGE_CHARS: usize = 2_000_000;
/// Characters a whole document may carry into a response.
///
/// The page cap alone allows five hundred pages of two million characters,
/// and a small crafted workbook can render exactly that: a gigabyte of JSON
/// on one line, read whole into the app's own memory. Four full pages is
/// still a hundred times what distillation reads.
pub const MAX_DOCUMENT_CHARS: usize = 8_000_000;
pub const MAX_PAGE_MEGAPIXELS: u64 = 25;
pub const MAX_PAGE_PIXELS: u64 = MAX_PAGE_MEGAPIXELS * 1_000_000;
/// The largest image file that is decoded at all.
///
/// A phone's 48- and 50-megapixel modes are ordinary now, and a photo of a
/// receipt from one is no harder to read than the 12-megapixel photo the
/// same phone takes by default: it is decoded and then scaled down to
/// [`MAX_PAGE_PIXELS`] before OCR. This cap only refuses an image so large
/// that decoding it is itself the problem - a 100-megapixel RGB image is
/// already 300 MB of pixels.
pub const MAX_IMAGE_FILE_MEGAPIXELS: u64 = 100;
pub const MAX_IMAGE_FILE_PIXELS: u64 = MAX_IMAGE_FILE_MEGAPIXELS * 1_000_000;
/// The resolution a PDF page is rendered at when it fits the pixel budget.
pub const RENDER_DPI: f64 = 300.0;
/// The lowest resolution a page is rendered at to be read.
///
/// A page that is physically huge does not need 300 DPI for Tesseract to
/// read it, so a page over the pixel budget is rendered at whatever
/// resolution fits. Below 50 DPI ordinary body text is a few pixels tall
/// and nothing would come back worth having; a page that large is a
/// degenerate file, not a scan.
pub const MIN_OCR_DPI: f64 = 50.0;
pub const MAX_EXTRACTION_DURATION: Duration = Duration::from_secs(30 * 60);
pub const MAX_RESIDENT_RENDERED_PAGES: usize = 1;
pub const MAX_VISION_LONG_EDGE: u32 = 1_344;
pub const VISION_GRID: u32 = 28;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub max_source_bytes: u64,
    pub max_page_count: usize,
    pub max_decompressed_office_bytes: u64,
    pub max_temp_bytes: u64,
    pub max_page_pixels: u64,
    pub max_image_file_pixels: u64,
    pub max_duration: Duration,
    pub max_resident_rendered_pages: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: MAX_SOURCE_BYTES,
            max_page_count: MAX_PAGE_COUNT,
            max_decompressed_office_bytes: MAX_DECOMPRESSED_OFFICE_BYTES,
            max_temp_bytes: MAX_TEMP_BYTES,
            max_page_pixels: MAX_PAGE_PIXELS,
            max_image_file_pixels: MAX_IMAGE_FILE_PIXELS,
            max_duration: MAX_EXTRACTION_DURATION,
            max_resident_rendered_pages: MAX_RESIDENT_RENDERED_PAGES,
        }
    }
}

impl ResourceLimits {
    pub fn validate_source_size(&self, bytes: u64) -> Result<(), ExtractionError> {
        if bytes > self.max_source_bytes {
            return Err(ExtractionError::resource_limit("source file exceeds 1 GiB"));
        }
        Ok(())
    }

    pub fn validate_page_count(&self, pages: usize) -> Result<(), ExtractionError> {
        if pages > self.max_page_count {
            return Err(ExtractionError::resource_limit(
                "document exceeds 500 pages",
            ));
        }
        Ok(())
    }

    pub fn validate_page_pixels(&self, width: u32, height: u32) -> Result<(), ExtractionError> {
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or_else(|| ExtractionError::resource_limit("page pixel count overflow"))?;
        if pixels > self.max_page_pixels {
            return Err(ExtractionError::resource_limit(
                "rendered page exceeds 25 megapixels",
            ));
        }
        Ok(())
    }

    /// Refuses an image file too large to decode, before any pixel of it is.
    pub fn validate_image_file_pixels(
        &self,
        width: u32,
        height: u32,
    ) -> Result<(), ExtractionError> {
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or_else(|| ExtractionError::resource_limit("image pixel count overflow"))?;
        if pixels > self.max_image_file_pixels {
            return Err(ExtractionError::resource_limit(
                "image exceeds 100 megapixels",
            ));
        }
        Ok(())
    }
}

/// A page's rendered size in pixels, and the resolution that gives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSize {
    pub width: u32,
    pub height: u32,
    pub dpi: f64,
}

/// The size to render a page of the given size in points: [`RENDER_DPI`],
/// or, when that would be more than `max_pixels`, the highest resolution
/// that is not.
///
/// At full resolution each edge is rounded up, as it always has been. Below
/// it each edge is rounded down instead, because two edges rounded up can
/// together pass the budget they were scaled to meet - a 4032 x 3024 point
/// page scaled to exactly 25 megapixels rounds up to 25,007,194 pixels and
/// the render would then be refused for exceeding the cap it was sized for.
pub fn render_size_within(width_points: f32, height_points: f32, max_pixels: u64) -> RenderSize {
    let width_points = f64::from(width_points.abs());
    let height_points = f64::from(height_points.abs());
    let scale = RENDER_DPI / 72.0;
    let width = (width_points * scale).ceil().max(1.0);
    let height = (height_points * scale).ceil().max(1.0);
    if width * height <= max_pixels as f64 {
        return RenderSize {
            width: width as u32,
            height: height as u32,
            dpi: RENDER_DPI,
        };
    }
    let dpi = 72.0 * (max_pixels as f64 / (width_points * height_points)).sqrt();
    let scale = dpi / 72.0;
    RenderSize {
        width: (width_points * scale).floor().max(1.0) as u32,
        height: (height_points * scale).floor().max(1.0) as u32,
        dpi,
    }
}

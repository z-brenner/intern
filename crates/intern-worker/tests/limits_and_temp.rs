use std::time::Duration;

use intern_worker::extract::load_oriented_image;
use intern_worker::limits::{
    MAX_DECOMPRESSED_OFFICE_BYTES, MAX_EXTRACTION_DURATION, MAX_IMAGE_FILE_MEGAPIXELS,
    MAX_PAGE_COUNT, MAX_PAGE_MEGAPIXELS, MAX_PAGE_PIXELS, MAX_RESIDENT_RENDERED_PAGES,
    MAX_SOURCE_BYTES, MAX_TEMP_BYTES, RENDER_DPI, ResourceLimits, render_size_within,
};
use intern_worker::temp::TempWorkspace;
use tempfile::tempdir;

#[test]
fn default_limits_enforce_the_worker_resource_contract() {
    let limits = ResourceLimits::default();

    assert_eq!(MAX_SOURCE_BYTES, 1_073_741_824);
    assert_eq!(MAX_PAGE_COUNT, 500);
    assert_eq!(MAX_DECOMPRESSED_OFFICE_BYTES, 1_073_741_824);
    assert_eq!(MAX_TEMP_BYTES, 2_147_483_648);
    assert_eq!(MAX_PAGE_MEGAPIXELS, 25);
    assert_eq!(MAX_IMAGE_FILE_MEGAPIXELS, 100);
    assert_eq!(MAX_EXTRACTION_DURATION, Duration::from_secs(30 * 60));
    assert_eq!(MAX_RESIDENT_RENDERED_PAGES, 1);
    assert_eq!(limits.max_source_bytes, MAX_SOURCE_BYTES);
    assert_eq!(limits.max_page_count, MAX_PAGE_COUNT);
    assert!(limits.validate_source_size(MAX_SOURCE_BYTES).is_ok());
    assert_eq!(
        limits
            .validate_source_size(MAX_SOURCE_BYTES + 1)
            .unwrap_err()
            .code(),
        "RESOURCE_LIMIT_EXCEEDED"
    );
    assert!(limits.validate_page_count(MAX_PAGE_COUNT).is_ok());
    assert_eq!(
        limits
            .validate_page_count(MAX_PAGE_COUNT + 1)
            .unwrap_err()
            .code(),
        "RESOURCE_LIMIT_EXCEEDED"
    );
    assert!(limits.validate_page_pixels(5_000, 5_000).is_ok());
    assert_eq!(
        limits
            .validate_page_pixels(5_001, 5_000)
            .unwrap_err()
            .code(),
        "RESOURCE_LIMIT_EXCEEDED"
    );
}

#[test]
fn temporary_workspace_is_deleted_on_drop() {
    let path = {
        let workspace = TempWorkspace::create("cleanup-test", MAX_TEMP_BYTES).unwrap();
        let path = workspace.path().to_path_buf();
        std::fs::write(workspace.path().join("page.png"), b"temporary").unwrap();
        assert!(path.exists());
        path
    };

    assert!(!path.exists());
}

#[test]
fn temporary_workspace_stays_inside_its_owned_private_root() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("owned");
    std::fs::create_dir(&root).unwrap();
    let unrelated = temp.path().join("unrelated.txt");
    std::fs::write(&unrelated, b"keep").unwrap();

    let workspace = TempWorkspace::create_in(&root, "privacy-test", MAX_TEMP_BYTES).unwrap();
    assert_eq!(workspace.path().parent(), Some(root.as_path()));
    let workspace_path = workspace.path().to_path_buf();
    drop(workspace);

    assert!(!workspace_path.exists());
    assert_eq!(std::fs::read(unrelated).unwrap(), b"keep");
}

#[test]
fn temporary_workspace_refuses_writes_beyond_budget() {
    let workspace = TempWorkspace::create("budget-test", 3).unwrap();
    let error = workspace.write("too-large.bin", b"1234").unwrap_err();

    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    assert!(!workspace.path().join("too-large.bin").exists());
}

#[test]
fn temporary_workspace_accepts_only_normal_relative_components() {
    let workspace = TempWorkspace::create("path-test", MAX_TEMP_BYTES).unwrap();
    assert!(workspace.write("nested/file.bin", b"ok").is_ok());
    for path in [
        "",
        ".",
        "./file.bin",
        "../file.bin",
        "/rooted.bin",
        r"\rooted.bin",
        r"C:\rooted.bin",
        "C:/rooted.bin",
        r"C:relative.bin",
        r"\\server\share\file.bin",
    ] {
        assert_eq!(
            workspace.write(path, b"no").unwrap_err().code(),
            "PARSE_FAILED",
            "{path}"
        );
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn append_png_chunk(bytes: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut crc_data = kind.to_vec();
    crc_data.extend_from_slice(data);
    bytes.extend_from_slice(&crc_data);
    bytes.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

/// A PNG that declares its size and holds no pixels: 8-bit grey, so a
/// decoder that gets as far as allocating allocates one byte a pixel.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 0, 0, 0, 0]);
    append_png_chunk(&mut bytes, b"IHDR", &header);
    append_png_chunk(&mut bytes, b"IDAT", &[]);
    append_png_chunk(&mut bytes, b"IEND", &[]);
    bytes
}

/// An image file is refused on its declared size before a pixel of it is
/// decoded - but at 100 megapixels, not at the 25 a rendered page is held
/// to. A phone's 48- and 50-megapixel photos are ordinary documents; they
/// are decoded and scaled down to the page cap instead.
#[test]
fn encoded_dimensions_over_one_hundred_megapixels_fail_before_pixel_decode() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("oversized.png");
    std::fs::write(&path, png_header(10_001, 10_000)).unwrap();

    let error = load_oriented_image(&path, &ResourceLimits::default()).unwrap_err();

    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
}

#[test]
fn exactly_one_hundred_megapixels_passes_the_header_limit() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("boundary.png");
    std::fs::write(&path, png_header(10_000, 10_000)).unwrap();

    let error = load_oriented_image(&path, &ResourceLimits::default()).unwrap_err();

    // Past the header check, and refused only because it holds no pixels.
    assert_eq!(error.code(), "PARSE_FAILED");
}

/// What used to be refused outright is now only over the page cap.
#[test]
fn a_photo_over_the_page_cap_is_no_longer_refused_by_its_header() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("phone-photo.png");
    std::fs::write(&path, png_header(8_000, 6_000)).unwrap();

    let error = load_oriented_image(&path, &ResourceLimits::default()).unwrap_err();

    assert_eq!(error.code(), "PARSE_FAILED");
}

/// A page that fits is rendered at 300 DPI exactly as it always was, each
/// edge rounded up.
#[test]
fn a_page_within_the_budget_renders_at_full_resolution() {
    // A4.
    let size = render_size_within(595.28, 841.89, MAX_PAGE_PIXELS);

    assert_eq!((size.width, size.height), (2_481, 3_508));
    assert_eq!(size.dpi, RENDER_DPI);

    // 1,200 points square is 25 megapixels at 300 DPI to the pixel.
    let size = render_size_within(1_200.0, 1_200.0, MAX_PAGE_PIXELS);
    assert_eq!((size.width, size.height), (5_000, 5_000));
    assert_eq!(size.dpi, RENDER_DPI);

    // No budget at all is the full-resolution render.
    let size = render_size_within(4_032.0, 3_024.0, u64::MAX);
    assert_eq!((size.width, size.height), (16_800, 12_600));
}

/// A page over the budget at 300 DPI is rendered at
/// `72 * sqrt(budget / area in points)`, and each edge is rounded down so
/// the two together cannot round back over the budget they were scaled to
/// meet.
#[test]
fn a_page_over_the_budget_renders_at_the_resolution_that_fits() {
    let photo = render_size_within(4_032.0, 3_024.0, MAX_PAGE_PIXELS);

    assert_eq!((photo.width, photo.height), (5_773, 4_330));
    assert!((photo.dpi - 103.098).abs() < 0.001, "{}", photo.dpi);
    assert!(u64::from(photo.width) * u64::from(photo.height) <= MAX_PAGE_PIXELS);

    // A2 at 300 DPI is 34.8 megapixels.
    let a2 = render_size_within(1_190.55, 1_683.78, MAX_PAGE_PIXELS);
    assert!(a2.dpi < RENDER_DPI && a2.dpi > 250.0, "{}", a2.dpi);
    assert!(u64::from(a2.width) * u64::from(a2.height) <= MAX_PAGE_PIXELS);
    assert!(u64::from(a2.width) * u64::from(a2.height) > MAX_PAGE_PIXELS - 10_000);

    // 1,201 points square misses 300 DPI by a hair, and loses no more than
    // the hair: within a pixel of 5,000 a side.
    let just_over = render_size_within(1_201.0, 1_201.0, MAX_PAGE_PIXELS);
    assert!(just_over.dpi < RENDER_DPI);
    assert!(u64::from(just_over.width) * u64::from(just_over.height) <= MAX_PAGE_PIXELS);
    assert!(just_over.width >= 4_999 && just_over.height >= 4_999);
}

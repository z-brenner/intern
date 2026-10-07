use image::{DynamicImage, RgbImage};
use intern_worker::extract::{CancellationToken, OcrBackend, PdfBackend, RenderedPage};
use intern_worker::ocr::TesseractOcr;
use intern_worker::pdf::PdfiumBackend;
use tempfile::tempdir;

#[cfg(all(feature = "native-tesseract", unix))]
fn fake_tesseract(
    directory: &std::path::Path,
    osd_exit_code: i32,
    osd_diagnostic: &str,
) -> TesseractOcr {
    use std::os::unix::fs::PermissionsExt as _;

    let executable = directory.join("tesseract");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$8\" = \"0\" ]; then\n\
           if [ {osd_exit_code} -eq 0 ]; then\n\
             printf 'Orientation in degrees: 270\\nRotate: 90\\n' > \"$2.osd\"\n\
           fi\n\
           printf '{osd_diagnostic}' >&2\n\
           exit {osd_exit_code}\n\
         fi\n\
         {{\n\
           printf 'level\\tpage_num\\tblock_num\\tpar_num\\tline_num\\tword_num\\t'\n\
           printf 'left\\ttop\\twidth\\theight\\tconf\\ttext\\n'\n\
           printf '5\\t1\\t1\\t1\\t1\\t1\\t0\\t0\\t1\\t1\\t88\\tfixture\\n'\n\
         }} > \"$2.tsv\"\n"
    );
    std::fs::write(&executable, script).unwrap();
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&executable, permissions).unwrap();
    let tessdata = directory.join("tessdata");
    std::fs::create_dir(&tessdata).unwrap();
    std::fs::write(tessdata.join("eng.traineddata"), b"fixture").unwrap();
    std::fs::write(tessdata.join("osd.traineddata"), b"fixture").unwrap();
    TesseractOcr::new(executable, tessdata).unwrap()
}

#[test]
fn missing_pdfium_never_reports_successful_extraction() {
    let directory = tempdir().unwrap();
    let error = match PdfiumBackend::new(directory.path()) {
        Ok(backend) => backend
            .inspect(
                directory.path().join("missing.pdf").as_path(),
                &CancellationToken::new(),
            )
            .unwrap_err(),
        Err(error) => error,
    };

    assert_eq!(error.code(), "NATIVE_ASSETS_MISSING");
}

#[test]
fn missing_tesseract_never_reports_successful_ocr() {
    let directory = tempdir().unwrap();
    let executable = directory.path().join("tesseract.exe");
    let tessdata = directory.path().join("tessdata");
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 10)));
    let error = match TesseractOcr::new(executable, tessdata) {
        Ok(backend) => backend
            .recognize(&page, &CancellationToken::new())
            .unwrap_err(),
        Err(error) => error,
    };

    assert_eq!(error.code(), "NATIVE_ASSETS_MISSING");
}

#[cfg(all(feature = "native-tesseract", unix))]
#[test]
fn tesseract_adapter_reads_osd_extension_and_applies_rotation() {
    let directory = tempdir().unwrap();
    let backend = fake_tesseract(directory.path(), 0, "");
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 20)));

    let result = backend.recognize(&page, &CancellationToken::new()).unwrap();

    assert_eq!(result.text, "fixture");
    assert_eq!(result.mean_confidence, 88.0);
    assert_eq!(result.rotation_degrees, 90);
}

#[cfg(all(feature = "native-tesseract", unix))]
#[test]
fn tesseract_osd_exit_one_falls_back_to_zero_degree_ocr() {
    let directory = tempdir().unwrap();
    let backend = fake_tesseract(
        directory.path(),
        1,
        "Too few characters. Skipping this page\\n",
    );
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 20)));

    let result = backend.recognize(&page, &CancellationToken::new()).unwrap();

    assert_eq!(result.text, "fixture");
    assert_eq!(result.rotation_degrees, 0);
}

#[cfg(all(feature = "native-tesseract", unix))]
#[test]
fn tesseract_osd_does_not_turn_cancellation_into_fallback() {
    let directory = tempdir().unwrap();
    let backend = fake_tesseract(
        directory.path(),
        1,
        "Too few characters. Skipping this page\\n",
    );
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 20)));
    let cancel = CancellationToken::new();
    cancel.cancel();

    let error = backend.recognize(&page, &cancel).unwrap_err();

    assert_eq!(error.code(), "CANCELED");
}

#[cfg(all(feature = "native-tesseract", unix))]
#[test]
fn tesseract_spawn_failure_is_propagated_instead_of_falling_back() {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = tempdir().unwrap();
    let backend = fake_tesseract(
        directory.path(),
        1,
        "Too few characters. Skipping this page\\n",
    );
    let executable = directory.path().join("tesseract");
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(executable, permissions).unwrap();
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 20)));

    let error = backend
        .recognize(&page, &CancellationToken::new())
        .unwrap_err();

    assert_eq!(error.code(), "PARSE_FAILED");
    assert!(error.retryable());
}

#[cfg(all(feature = "native-tesseract", unix))]
#[test]
fn tesseract_osd_exit_one_with_initialization_diagnostic_is_not_fallback() {
    let directory = tempdir().unwrap();
    let backend = fake_tesseract(
        directory.path(),
        1,
        concat!(
            "Error opening data file osd.traineddata\\n",
            "Failed loading language osd\\n",
            "Could not initialize tesseract\\n"
        ),
    );
    let page = RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(10, 20)));

    let error = backend
        .recognize(&page, &CancellationToken::new())
        .unwrap_err();

    assert_eq!(error.code(), "NATIVE_ASSETS_MISSING");
    assert!(!error.retryable());
}

/// PDFium keeps process-global state and is not safe to drive from several
/// threads at once, so the tests that drive it take turns.
#[cfg(feature = "native-pdfium")]
static PDFIUM_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(feature = "native-pdfium")]
fn pdfium_turn() -> std::sync::MutexGuard<'static, ()> {
    PDFIUM_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

#[cfg(feature = "native-pdfium")]
fn stream(dictionary: &str, contents: &[u8]) -> Vec<u8> {
    let mut object =
        format!("<< {dictionary} /Length {} >>\nstream\n", contents.len()).into_bytes();
    object.extend_from_slice(contents);
    object.extend_from_slice(b"\nendstream");
    object
}

/// A PDF of these objects, numbered from 1, with its cross-reference table.
#[cfg(feature = "native-pdfium")]
fn pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut pdf = b"%PDF-1.7\n%\x80\x80\x80\x80\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        pdf.extend_from_slice(object);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    pdf
}

#[cfg(feature = "native-pdfium")]
fn nested_image_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] ",
            "/Resources << /XObject << /Fm0 4 0 R >> >> /Contents 5 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream(
            "/Type /XObject /Subtype /Form /BBox [0 0 100 100] /Resources << /XObject << /Im0 6 0 R >> >>",
            b"q 100 0 0 100 0 0 cm /Im0 Do Q",
        ),
        stream("", b"q /Fm0 Do Q"),
        stream(
            "/Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8",
            &[255, 0, 0],
        ),
    ];
    pdf(&objects)
}

/// One page of `width` x `height` points that is nothing but an image - what
/// a tool that wraps a photo in a PDF at 72 DPI produces.
#[cfg(feature = "native-pdfium")]
fn photo_pdf(width: u32, height: u32) -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] \
             /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>"
        )
        .into_bytes(),
        stream(
            "",
            format!("q {width} 0 0 {height} 0 0 cm /Im0 Do Q").as_bytes(),
        ),
        stream(
            "/Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8",
            &[0, 255],
        ),
    ];
    pdf(&objects)
}

/// A landscape page stored as a portrait sheet turned a quarter, with text
/// drawn on both sides of where the displayed width would cut it.
#[cfg(feature = "native-pdfium")]
fn turned_landscape_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Rotate 90 ",
            "/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream(
            "",
            b"BT /F1 12 Tf 300 100 Td (CARRIER RATE CONFIRMATION) Tj ET \
              BT /F1 12 Tf 300 700 Td (Confirmed June 8, 2026) Tj ET",
        ),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    pdf(&objects)
}

/// A letter page a tool wrapped whole in one form XObject - iText's
/// imported pages, pdfpages, macOS - with the sender's logo drawn inside
/// the form beside its text, placed through the form's own map.
#[cfg(feature = "native-pdfium")]
fn wrapped_page_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /XObject << /Fm0 4 0 R >> >> /Contents 5 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream(
            concat!(
                "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Matrix [1 0 0 1 10 0] ",
                "/Resources << /Font << /F1 6 0 R >> /XObject << /Im0 7 0 R >> >>"
            ),
            b"q 72 0 0 36 54 700 cm /Im0 Do Q \
              BT /F1 14 Tf 140 712 Td (WEXCOMBE MILLWORK CO.) Tj ET \
              BT /F1 10 Tf 54 650 Td (Invoice WMC-11047 for stair treads delivered to Lot 17.) Tj ET \
              BT /F1 10 Tf 54 636 Td (Payment is due within thirty days of the invoice date.) Tj ET \
              BT /F1 10 Tf 54 622 Td (Questions go to the billing office at the address above.) Tj ET",
        ),
        stream("", b"q 0.9 0 0 0.9 21 40 cm /Fm0 Do Q"),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        stream(
            "/Type /XObject /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8",
            &[0, 255],
        ),
    ];
    pdf(&objects)
}

/// A page wrapped in a form with a logo inside is a text page: its text
/// objects are its segments, its logo an image the size of the logo, and
/// nothing on it is read twice. It used to be taken for a page-sized image
/// no text overlapped, OCR'd whole, and every line merged in a second time.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_page_wrapped_in_a_form_with_a_logo_reads_its_text_once() {
    use intern_worker::extract::{OcrResult, extract_pdf};
    use intern_worker::layout::PageRoute;
    use intern_worker::limits::ResourceLimits;

    struct NoOcr;
    impl OcrBackend for NoOcr {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, intern_worker::extract::ExtractionError> {
            panic!("a text page wrapped in a form was sent to OCR")
        }
    }

    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("wrapped.pdf");
    std::fs::write(&path, wrapped_page_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();

    let pages = backend.inspect(&path, &CancellationToken::new()).unwrap();
    let native = pages[0].native.as_ref().unwrap();
    assert!(native.segments.len() >= 4, "{:?}", native.segments);
    assert_eq!(native.images.len(), 1);
    // The logo, 72 x 36 points at nine tenths: about 0.4% of the page.
    assert!(
        pages[0].image_coverage < 0.01,
        "{}",
        pages[0].image_coverage
    );
    // Placed through the form's own matrix and the page's where it draws
    // the form - PDFium applies the first to a form's children and keeps
    // the second on the form: the title's box starts at
    // 21 + 0.9 x (10 + 140) = 156 points from the left.
    assert!(
        native
            .segments
            .iter()
            .any(|segment| segment[0].abs_diff(1560) <= 20),
        "{:?}",
        native.segments
    );

    let document = extract_pdf(
        &path,
        &backend,
        &NoOcr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();
    let page = &document.pages[0];
    let route = page.layout.as_ref().unwrap().route;
    assert!(
        matches!(route, PageRoute::Fast | PageRoute::Layout),
        "{route:?}"
    );
    for line in [
        "WEXCOMBE MILLWORK CO.",
        "Invoice WMC-11047 for stair treads delivered to Lot 17.",
        "Payment is due within thirty days of the invoice date.",
    ] {
        assert_eq!(
            page.text.matches(line).count(),
            1,
            "{line}: {:?}",
            page.text
        );
    }
}

/// A rule drawn inside a form XObject that is drawn 400 points lower than
/// the form's own space puts it: 300 by the form's matrix, 100 by the
/// page's where it draws the form.
#[cfg(feature = "native-pdfium")]
fn ruled_form_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /XObject << /Fm0 4 0 R >> >> /Contents 5 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream(
            "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Matrix [1 0 0 1 0 -300]",
            b"1 w 54 700 m 558 700 l S",
        ),
        stream("", b"q 1 0 0 1 0 -100 cm /Fm0 Do Q"),
    ];
    pdf(&objects)
}

/// Rules inside a form are where the page draws them: 300 points from the
/// bottom, 492 from the top - not where the form's own space would put
/// them, 92 from the top, over whatever the page has there.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_rule_inside_a_form_is_where_the_form_is_drawn() {
    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("ruled-form.pdf");
    std::fs::write(&path, ruled_form_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();

    let pages = backend.inspect(&path, &CancellationToken::new()).unwrap();

    let rulings = &pages[0].native.as_ref().unwrap().rulings;
    assert_eq!(rulings.len(), 1, "{rulings:?}");
    let [x0, y0, x1, y1] = rulings[0];
    assert!(
        x0.abs_diff(540) <= 10 && x1.abs_diff(5580) <= 10,
        "{rulings:?}"
    );
    assert!(
        y0.abs_diff(4920) <= 15 && y1.abs_diff(4920) <= 15,
        "{rulings:?}"
    );
}

/// Two columns of capitals set apart, 28 to a line on 144 lines: some
/// eight thousand runs, twice what the layout analysis takes on, on a page
/// the router sends to it.
#[cfg(feature = "native-pdfium")]
fn crowded_columns_pdf() -> Vec<u8> {
    let letters = ["(X)"; 28].join(" -1500 ");
    let mut contents = String::new();
    for line in 0..144 {
        let y = 760 - line * 5;
        for x in [54, 330] {
            contents.push_str(&format!("BT /F1 4 Tf {x} {y} Td [{letters}] TJ ET\n"));
        }
    }
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream("", contents.as_bytes()),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    pdf(&objects)
}

/// A page with more runs than the layout analysis takes on is read as its
/// text: its characters stop being read once there are more, and the page
/// the router sent to the layout route is read on the fast one.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_page_of_more_runs_than_the_analysis_takes_on_keeps_its_text() {
    use intern_worker::extract::{OcrResult, extract_pdf};
    use intern_worker::layout::{PageRoute, route_page};
    use intern_worker::limits::ResourceLimits;

    struct NoOcr;
    impl OcrBackend for NoOcr {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, intern_worker::extract::ExtractionError> {
            panic!("a text page was sent to OCR")
        }
    }

    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("crowded.pdf");
    std::fs::write(&path, crowded_columns_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();
    let cancel = CancellationToken::new();

    let pages = backend.inspect(&path, &cancel).unwrap();
    let signals = pages[0].signals.unwrap();
    assert_eq!(
        route_page(&signals, false),
        PageRoute::Layout,
        "{signals:?}"
    );
    let native = pages[0].native.as_ref().unwrap();
    assert!(
        backend
            .page_runs(&path, 0, native, &cancel)
            .unwrap()
            .is_empty(),
        "a page past the bound has no runs"
    );

    let document =
        extract_pdf(&path, &backend, &NoOcr, &ResourceLimits::default(), &cancel).unwrap();
    let page = &document.pages[0];
    assert_eq!(page.text, pages[0].native_text);
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::Fast);
}

/// Two pages of a few lines each.
#[cfg(feature = "native-pdfium")]
fn two_page_pdf() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /Font << /F1 7 0 R >> >> /Contents 5 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream(
            "",
            b"BT /F1 11 Tf 72 700 Td (Hollowmere Freight Co.) Tj ET \
              BT /F1 11 Tf 72 680 Td (Delivery receipt for pallet 7) Tj ET",
        ),
        stream(
            "",
            b"BT /F1 11 Tf 72 700 Td (Received in good order) Tj ET \
              BT /F1 11 Tf 72 680 Td (Signed at the Brackenridge dock) Tj ET",
        ),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    pdf(&objects)
}

/// Inspection reads a document's characters ahead only within its budget;
/// a page past it has none, and its characters are read when it is.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_document_s_characters_are_read_ahead_only_within_its_budget() {
    use intern_worker::layout::PageRoute;

    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("two-pages.pdf");
    std::fs::write(&path, two_page_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();
    let cancel = CancellationToken::new();
    let geometry = |_: &intern_worker::layout::RouteSignals, _: bool| PageRoute::Layout;

    let every = backend
        .inspect_routed(&path, &cancel, geometry, usize::MAX)
        .unwrap();
    let budgeted = backend.inspect_routed(&path, &cancel, geometry, 1).unwrap();

    let runs = |pages: &[intern_worker::extract::PdfPageInspection], index: usize| {
        pages[index].native.as_ref().unwrap().runs.clone()
    };
    assert!(!runs(&every, 1).is_empty());
    assert_eq!(runs(&budgeted, 0), runs(&every, 0));
    assert!(runs(&budgeted, 1).is_empty(), "past the budget");
    let native = budgeted[1].native.as_ref().unwrap();
    assert_eq!(
        backend.page_runs(&path, 1, native, &cancel).unwrap(),
        runs(&every, 1)
    );
}

/// Text on a quarter-turned page is all read: drawn past the displayed
/// width, it used to be dropped.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_quarter_turned_page_keeps_all_its_text() {
    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("turned.pdf");
    std::fs::write(&path, turned_landscape_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();

    let pages = backend.inspect(&path, &CancellationToken::new()).unwrap();

    let text = &pages[0].native_text;
    assert!(text.contains("CARRIER RATE CONFIRMATION"), "{text:?}");
    assert!(text.contains("Confirmed June 8, 2026"), "{text:?}");
}

#[cfg(feature = "native-pdfium")]
#[test]
fn form_xobject_nested_image_contributes_rendered_coverage() {
    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("nested-image.pdf");
    std::fs::write(&path, nested_image_pdf()).unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();

    let pages = backend.inspect(&path, &CancellationToken::new()).unwrap();

    assert_eq!(pages.len(), 1);
    assert!(
        pages[0].image_coverage >= 0.65,
        "{}",
        pages[0].image_coverage
    );
}

/// A phone photo wrapped in a PDF at 72 DPI is a 4032 x 3024 point page,
/// about 212 megapixels at 300 DPI. PDFium renders it within the 25-megapixel
/// budget instead - at about 103 DPI - and the page is OCR'd rather than the
/// document failing.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_photo_sized_page_renders_within_the_pixel_budget() {
    use image::GenericImageView as _;
    use intern_worker::extract::{OcrResult, PageSource, extract_pdf};
    use intern_worker::limits::{MAX_PAGE_PIXELS, ResourceLimits};

    struct MeasuringOcr(std::sync::Mutex<Vec<(u32, u32)>>);
    impl OcrBackend for MeasuringOcr {
        fn recognize(
            &self,
            page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, intern_worker::extract::ExtractionError> {
            self.0.lock().unwrap().push(page.image.dimensions());
            Ok(OcrResult::new("RECEIPT 0417 TOTAL 42.10", 91.0))
        }
    }

    let Some(runtime) = std::env::var_os("INTERN_RUNTIME_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let path = directory.path().join("receipt.pdf");
    std::fs::write(&path, photo_pdf(4_032, 3_024)).unwrap();
    let backend = PdfiumBackend::new(runtime).unwrap();
    let ocr = MeasuringOcr(std::sync::Mutex::new(Vec::new()));

    let document = extract_pdf(
        &path,
        &backend,
        &ocr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    let sizes = ocr.0.lock().unwrap();
    assert_eq!(sizes.len(), 1);
    let (width, height) = sizes[0];
    assert!(
        u64::from(width) * u64::from(height) <= MAX_PAGE_PIXELS,
        "{width} x {height}"
    );
    assert!(width >= 5_772 && height >= 4_329, "{width} x {height}");
    assert_eq!(document.pages[0].source, PageSource::Ocr);
}

/// The scanned lease through the real PDFium and the real Tesseract: its
/// OCR text keeps the lines Tesseract found, where it used to come back as
/// one line holding the whole page.
#[cfg(all(feature = "native-pdfium", feature = "native-tesseract"))]
#[test]
fn scanned_lease_ocr_text_has_multiple_lines() {
    use intern_worker::extract::{PageSource, extract_pdf};
    use intern_worker::limits::ResourceLimits;

    let Some(runtime) = std::env::var_os("INTERN_RUNTIME_DIR").map(std::path::PathBuf::from) else {
        return;
    };
    let lease = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/scanned-lease.pdf");
    if !lease.is_file() {
        assert!(
            std::env::var_os("INTERN_REQUIRE_GENERATED_FIXTURES").is_none(),
            "required generated fixture is missing: {}",
            lease.display()
        );
        return;
    }
    let _turn = pdfium_turn();
    let ocr = TesseractOcr::new(runtime.join("tesseract.exe"), runtime.join("tessdata")).unwrap();

    let document = extract_pdf(
        &lease,
        &PdfiumBackend::new(&runtime).unwrap(),
        &ocr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    let lines: Vec<&str> = page
        .text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(lines.len() >= 3, "{:?}", page.text);
    assert!(lines[0].starts_with("LEASE AGREEMENT"), "{:?}", page.text);
    assert!(
        !lines[0].contains("PROPERTIES"),
        "the parties ran into the title: {:?}",
        page.text
    );
}

/// Every block of every page the generated fixtures read is a stretch of
/// that page's text, found in it in order - on the fast route, where the
/// text is PDFium's with its `\r\n` line ends, and on every other: a block
/// can always be cited from the page it came from.
#[cfg(feature = "native-pdfium")]
#[test]
fn every_block_of_the_fixtures_is_found_in_its_page_text() {
    use intern_worker::extract::{OcrResult, extract_pdf};
    use intern_worker::limits::ResourceLimits;

    struct CannedOcr;
    impl OcrBackend for CannedOcr {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, intern_worker::extract::ExtractionError> {
            Ok(OcrResult::new("SCANNED PAGE\r\nRead by a stand-in.", 90.0))
        }
    }

    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let fixtures =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let Ok(entries) = std::fs::read_dir(&fixtures) else {
        return;
    };
    let mut pdfs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pdf"))
        .collect::<Vec<_>>();
    pdfs.sort();
    let _turn = pdfium_turn();
    let backend = PdfiumBackend::new(library_directory).unwrap();
    // Scans rendered small: what OCR makes of them is canned anyway.
    let limits = ResourceLimits {
        max_page_pixels: 2_000_000,
        ..ResourceLimits::default()
    };

    let mut checked = 0;
    for path in &pdfs {
        // The encrypted and malformed fixtures have no pages to check.
        let Ok(document) = extract_pdf(
            path,
            &backend,
            &CannedOcr,
            &limits,
            &CancellationToken::new(),
        ) else {
            continue;
        };
        for page in &document.pages {
            let Some(layout) = &page.layout else {
                continue;
            };
            let mut from = 0;
            for block in &layout.blocks {
                let at = page.text[from..].find(&block.text).unwrap_or_else(|| {
                    panic!(
                        "{} page {}: {:?} is not in the page text after byte {from}",
                        path.display(),
                        page.page_number,
                        block.text
                    )
                });
                from += at + block.text.len();
            }
            checked += 1;
        }
    }
    assert!(
        pdfs.is_empty() || checked > 0,
        "no fixture page was checked"
    );
}

/// A page of more objects than the survey visits, every one a rule.
#[cfg(feature = "native-pdfium")]
fn many_objects_pdf() -> Vec<u8> {
    let mut contents = String::from(
        "BT /F1 12 Tf 54 740 Td (A page drawn with far more objects than any document needs.) Tj ET\n",
    );
    for index in 0..(intern_worker::layout::router::bounds::MAX_SURVEY_OBJECTS + 500) {
        let y = 100 + (index % 600);
        contents.push_str(&format!("54 {y} m 300 {y} l S\n"));
    }
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream("", contents.as_bytes()),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    pdf(&objects)
}

/// A page whose first objects are invisible text - somebody's OCR layer -
/// and whose visible text is drawn only after more objects than the survey
/// visits.
#[cfg(feature = "native-pdfium")]
fn hidden_layer_then_many_objects_pdf() -> Vec<u8> {
    let mut contents = String::new();
    for line in 0..10 {
        let y = 700 - line * 14;
        contents.push_str(&format!(
            "BT 3 Tr /F1 12 Tf 54 {y} Td (Hidden layer line {line}.) Tj ET\n"
        ));
    }
    for index in 0..(intern_worker::layout::router::bounds::MAX_SURVEY_OBJECTS + 500) {
        let y = 100 + (index % 400);
        contents.push_str(&format!("54 {y} m 300 {y} l S\n"));
    }
    contents.push_str("BT 0 Tr /F1 12 Tf 54 60 Td (The visible text of the page.) Tj ET\n");
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream("", contents.as_bytes()),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    pdf(&objects)
}

/// A page whose only text is drawn in forms nested `depth` deep.
#[cfg(feature = "native-pdfium")]
fn nested_forms_pdf(depth: usize) -> Vec<u8> {
    let mut objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] ",
            "/Resources << /XObject << /Fm 6 0 R >> >> /Contents 4 0 R >>"
        )
        .as_bytes()
        .to_vec(),
        stream("", b"/Fm Do"),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ];
    // Forms 6 .. 6 + depth - 1, each drawing the next; the last draws text.
    for level in 0..depth {
        let number = 6 + level;
        if level + 1 < depth {
            objects.push(stream(
                &format!(
                    "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << /XObject << /Fm {} 0 R >> >>",
                    number + 1
                ),
                b"/Fm Do",
            ));
        } else {
            objects.push(stream(
                "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >>",
                b"BT /F1 12 Tf 54 740 Td (Text drawn in the innermost form.) Tj ET",
            ));
        }
    }
    pdf(&objects)
}

/// A page with more objects than the survey visits, or forms nested past
/// its depth, is not measured: it keeps its text and is read on the fast
/// route, and the walk over it stops at the bound.
#[cfg(feature = "native-pdfium")]
#[test]
fn a_page_past_the_survey_bounds_is_read_as_its_text() {
    use intern_worker::extract::{OcrResult, extract_pdf};
    use intern_worker::layout::PageRoute;
    use intern_worker::layout::router::bounds::MAX_FORM_DEPTH;
    use intern_worker::limits::ResourceLimits;

    struct NoOcr;
    impl OcrBackend for NoOcr {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, intern_worker::extract::ExtractionError> {
            panic!("a text page was sent to OCR")
        }
    }

    let Some(library_directory) = std::env::var_os("INTERN_PDFIUM_DIR") else {
        return;
    };
    let _turn = pdfium_turn();
    let directory = tempdir().unwrap();
    let backend = PdfiumBackend::new(library_directory).unwrap();
    let cancel = CancellationToken::new();
    for (name, bytes, text) in [
        ("many.pdf", many_objects_pdf(), "far more objects"),
        (
            "deep.pdf",
            nested_forms_pdf(MAX_FORM_DEPTH + 4),
            "innermost form",
        ),
    ] {
        let path = directory.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        let pages = backend.inspect(&path, &cancel).unwrap();
        assert!(
            pages[0].native.is_none(),
            "{name}: the page is not measured"
        );
        let document =
            extract_pdf(&path, &backend, &NoOcr, &ResourceLimits::default(), &cancel).unwrap();
        let page = &document.pages[0];
        assert!(page.text.contains(text), "{name}: {:?}", page.text);
        assert_eq!(
            page.layout.as_ref().unwrap().route,
            PageRoute::Fast,
            "{name}"
        );
    }

    // What the walk counted before it stopped is part of the page, not a
    // share of it: an invisible layer among the first objects says nothing
    // about the page, which is routed on its text.
    let path = directory.path().join("hidden.pdf");
    std::fs::write(&path, hidden_layer_then_many_objects_pdf()).unwrap();
    let pages = backend.inspect(&path, &cancel).unwrap();
    assert!(pages[0].native.is_none());
    let signals = pages[0].signals.unwrap();
    assert_eq!((signals.invisible, signals.segments), (0, 0), "{signals:?}");

    // Forms nested within the bound are followed as before.
    let path = directory.path().join("shallow.pdf");
    std::fs::write(&path, nested_forms_pdf(3)).unwrap();
    let pages = backend.inspect(&path, &cancel).unwrap();
    assert!(
        pages[0]
            .native
            .as_ref()
            .is_some_and(|native| !native.segments.is_empty())
    );
}

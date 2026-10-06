//! What the OCR pass decides between readings of one page. The decisions are
//! here rather than beside the Tesseract adapter because they are the part
//! that has to hold on the platform this ships on, where no Tesseract fixture
//! runs.

use std::collections::HashMap;

use intern_worker::extract::{
    CONFIDENT_READING, ExtractionError, OcrResult, OrientationPasses, better_reading,
    orientation_search_is_worthwhile, read_upright,
};

/// Measured on the corpus: a page read in the orientation OSD asked for
/// scored 44 while the same page read as-is scored 95, with both readings
/// returning eleven words. Volume cannot choose between them; confidence
/// can.
#[test]
fn a_confidently_misdetected_rotation_loses_to_the_page_as_it_was() {
    let oriented = OcrResult::new("O71 TIVL3Y MOGVAW ZLYVNO", 44.1).with_rotation(180);
    let unrotated = OcrResult::new("PACKING SLIP PS-311 DATE JULY 15 2025", 95.2);

    let chosen = better_reading(oriented, unrotated);

    assert_eq!(chosen.text, "PACKING SLIP PS-311 DATE JULY 15 2025");
    assert_eq!(chosen.rotation_degrees, 0);
}

#[test]
fn a_genuinely_rotated_page_keeps_the_rotation_that_read_it() {
    let oriented = OcrResult::new("DELIVERY RECEIPT DR-771", 92.0).with_rotation(270);
    let unrotated = OcrResult::new("gibberish", 31.0);

    let chosen = better_reading(oriented, unrotated);

    assert_eq!(chosen.text, "DELIVERY RECEIPT DR-771");
    assert_eq!(chosen.rotation_degrees, 270);
}

/// A tie keeps the detected orientation rather than silently preferring the
/// unrotated read, so behaviour on a blank page stays predictable.
#[test]
fn an_equal_score_keeps_the_detected_orientation() {
    let oriented = OcrResult::new("", 0.0).with_rotation(90);
    let unrotated = OcrResult::new("", 0.0);

    assert_eq!(better_reading(oriented, unrotated).rotation_degrees, 90);
}

/// Mean word confidence says nothing about how much was read. A rotated page
/// that yields three confident tokens beat a page of three hundred words read
/// just under the confidence bar, and the document came back as three tokens.
#[test]
fn a_sparse_high_confidence_reading_does_not_displace_a_dense_one() {
    let dense = OcrResult::new(
        "Settlement Agreement and Mutual Release ".repeat(50).trim(),
        74.9,
    );
    let sparse = OcrResult::new("INVOICE 4 2", 80.0).with_rotation(90);

    let chosen = better_reading(dense.clone(), sparse);

    assert_eq!(chosen.text, dense.text);
    assert_eq!(chosen.rotation_degrees, 0);
}

/// Density is a floor, not a preference: a fuller reading that is also more
/// confident still wins.
#[test]
fn a_denser_and_more_confident_reading_still_wins() {
    let incumbent = OcrResult::new("REMITTANCE", 60.0);
    let challenger = OcrResult::new("Remittance advice for invoice 4471", 91.0).with_rotation(180);

    let chosen = better_reading(incumbent, challenger);

    assert_eq!(chosen.text, "Remittance advice for invoice 4471");
    assert_eq!(chosen.rotation_degrees, 180);
}

/// A blank page - the back of every sheet of a duplex scan - reads as no
/// words at all, which scores zero confidence. Zero is "not confident", so
/// the orientation search used to buy three more recognition passes and
/// three more full-page PNG encodes to look at the same blank page from
/// three more angles.
#[test]
fn a_blank_page_costs_one_recognition_pass() {
    let blank = OcrResult::new("", 0.0);

    assert!(!orientation_search_is_worthwhile(&blank));
    // What the search used to ask, and why it kept going.
    assert!(blank.mean_confidence < CONFIDENT_READING);
}

#[test]
fn an_unconvincing_reading_is_still_worth_another_orientation() {
    assert!(orientation_search_is_worthwhile(&OcrResult::new(
        "O71 TIVL3Y MOGVAW",
        44.1
    )));
}

#[test]
fn a_confident_reading_is_never_read_again() {
    assert!(!orientation_search_is_worthwhile(&OcrResult::new(
        "PACKING SLIP PS-311",
        95.2
    )));
}

/// One Tesseract pass a page cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
    Recognize(u16),
    DetectOrientation,
}

/// A page whose reading in each orientation, and whose detected
/// orientation, are known in advance, and which keeps a log of every pass
/// it was asked for.
struct ScriptedPage {
    readings: HashMap<u16, OcrResult>,
    detected: u16,
    passes: Vec<Pass>,
}

impl ScriptedPage {
    fn new(readings: impl IntoIterator<Item = (u16, OcrResult)>, detected: u16) -> Self {
        Self {
            readings: readings.into_iter().collect(),
            detected,
            passes: Vec::new(),
        }
    }
}

impl OrientationPasses for ScriptedPage {
    fn recognize(&mut self, rotation_degrees: u16) -> Result<OcrResult, ExtractionError> {
        assert!(
            !self.passes.contains(&Pass::Recognize(rotation_degrees)),
            "the page was read at {rotation_degrees} degrees twice"
        );
        self.passes.push(Pass::Recognize(rotation_degrees));
        Ok(self.readings[&rotation_degrees]
            .clone()
            .with_rotation(rotation_degrees))
    }

    fn detect_orientation(&mut self) -> Result<u16, ExtractionError> {
        self.passes.push(Pass::DetectOrientation);
        Ok(self.detected)
    }
}

/// Nearly every page is upright. Orientation detection used to run first on
/// every one of them - and on the corpus's upright lease it was confidently
/// wrong, which bought three more recognition passes to find the
/// orientation the page already had.
#[test]
fn a_confident_upright_reading_is_the_only_pass() {
    let mut page = ScriptedPage::new(
        [(
            0,
            OcrResult::new(
                "PACKING SLIP PS-311\nDATE JULY 15 2025\nQUARTZ MEADOW RETAIL LLC",
                95.2,
            ),
        )],
        180,
    );

    let reading = read_upright(&mut page).unwrap();

    assert_eq!(page.passes, [Pass::Recognize(0)]);
    assert_eq!(reading.rotation_degrees, 0);
    assert!(reading.text.starts_with("PACKING SLIP"));
}

/// A sideways page read as it came is low-confidence gibberish. Then, and
/// only then, detection is asked, and the search goes on from what it says.
#[test]
fn an_unconvincing_upright_reading_asks_detection_and_then_searches() {
    let mut page = ScriptedPage::new(
        [
            (0, OcrResult::new("O71 TIVL3Y MOGVAW ZLYVNO", 31.0)),
            (90, OcrResult::new("LYV3 0IT NO7 Y3V", 22.0)),
            (
                270,
                OcrResult::new("DELIVERY RECEIPT DR-771 JUNE 12 2025", 92.0),
            ),
        ],
        90,
    );

    let reading = read_upright(&mut page).unwrap();

    assert_eq!(
        page.passes,
        [
            Pass::Recognize(0),
            Pass::DetectOrientation,
            Pass::Recognize(90),
            Pass::Recognize(270),
        ]
    );
    assert_eq!(reading.rotation_degrees, 270);
    assert_eq!(reading.text, "DELIVERY RECEIPT DR-771 JUNE 12 2025");
}

/// The upright reading is one of the search's candidates, and it is
/// compared as it already came back: it is never read a second time, and
/// it still wins when every other orientation reads worse.
#[test]
fn the_upright_reading_is_a_candidate_and_is_never_read_twice() {
    let upright = "SIGNATURE PAGE LUMEN KITE COOPERATIVE SOLSTICE INDEX LLC";
    let mut page = ScriptedPage::new(
        [
            (0, OcrResult::new(upright, 61.5)),
            (90, OcrResult::new("3DA4 3RUTAN9IZ", 20.8)),
            (180, OcrResult::new("3DA4 3RUTAN9IZ NEMUL", 20.8)),
            (270, OcrResult::new("3DA4 3RUTAN9IZ", 20.8)),
        ],
        180,
    );

    let reading = read_upright(&mut page).unwrap();

    assert_eq!(
        page.passes,
        [
            Pass::Recognize(0),
            Pass::DetectOrientation,
            Pass::Recognize(180),
            Pass::Recognize(270),
            Pass::Recognize(90),
        ]
    );
    assert_eq!(reading.rotation_degrees, 0);
    assert_eq!(reading.text, upright);
}

/// When detection agrees with the page as it came, the reading it would
/// buy is the one already in hand.
#[test]
fn detection_that_agrees_with_the_page_buys_no_second_reading() {
    let mut page = ScriptedPage::new(
        [
            (0, OcrResult::new("WORK ORDER WO-312 DATE JULY 16", 70.0)),
            (90, OcrResult::new("x", 10.0)),
            (180, OcrResult::new("x", 10.0)),
            (270, OcrResult::new("x", 10.0)),
        ],
        0,
    );

    let reading = read_upright(&mut page).unwrap();

    assert_eq!(
        page.passes,
        [
            Pass::Recognize(0),
            Pass::DetectOrientation,
            Pass::Recognize(270),
            Pass::Recognize(90),
            Pass::Recognize(180),
        ]
    );
    assert_eq!(reading.rotation_degrees, 0);
}

/// A blank page - the back of every sheet of a duplex scan - is blank in
/// every orientation, so asking which way up it is buys nothing.
#[test]
fn a_blank_page_is_read_once_and_never_detected() {
    let mut page = ScriptedPage::new([(0, OcrResult::new("", 0.0))], 90);

    let reading = read_upright(&mut page).unwrap();

    assert_eq!(page.passes, [Pass::Recognize(0)]);
    assert_eq!(reading.text, "");
}

/// Two confident specks on an otherwise unread page are not an upright
/// page: the word floor sends them on to detection.
#[test]
fn two_confident_specks_are_not_an_upright_page() {
    let mut page = ScriptedPage::new([(0, OcrResult::new("4 2", 91.0))], 0);

    read_upright(&mut page).unwrap();

    assert_eq!(page.passes, [Pass::Recognize(0), Pass::DetectOrientation]);
}

/// The same decisions through the real adapter, with a stand-in Tesseract
/// that logs every invocation: what it was asked for, the image it was
/// given, and the PNG colour type of that image.
#[cfg(all(feature = "native-tesseract", unix))]
mod through_the_adapter {
    use image::{DynamicImage, RgbImage};
    use intern_worker::extract::{CancellationToken, OcrBackend, RenderedPage};
    use intern_worker::ocr::TesseractOcr;

    /// One logged invocation: `recognize` or `osd`, the output name it was
    /// given, and the input's width, height and PNG colour type.
    #[derive(Debug, PartialEq, Eq)]
    struct Invocation {
        mode: String,
        output: String,
        width: u32,
        height: u32,
        colour_type: u8,
    }

    /// A stand-in `tesseract` that logs each invocation and answers
    /// orientation detection with `detected`. A recognition reads as
    /// confident four words when its output name is `confident_output`, and
    /// as four unconvincing ones otherwise.
    fn fake_tesseract(
        directory: &std::path::Path,
        detected: u16,
        confident_output: &str,
    ) -> TesseractOcr {
        use std::os::unix::fs::PermissionsExt as _;

        let log = directory.join("invocations.log");
        let executable = directory.join("tesseract");
        let script = format!(
            r#"#!/bin/sh
ihdr=$(od -An -tu1 -j16 -N10 "$1" | tr -s ' \n' '  ')
name=$(basename "$2")
if [ "$4" = "osd" ]; then
  echo "osd $name $ihdr" >> "{log}"
  printf 'Orientation in degrees: 0\nRotate: {detected}\n' > "$2.osd"
  exit 0
fi
echo "recognize $name $ihdr" >> "{log}"
if [ "$name" = "{confident_output}" ]; then conf=93; else conf=41; fi
{{
  printf 'level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\t'
  printf 'left\ttop\twidth\theight\tconf\ttext\n'
  printf '5\t1\t1\t1\t1\t1\t0\t0\t1\t1\t%s\tDELIVERY\n' "$conf"
  printf '5\t1\t1\t1\t1\t2\t0\t0\t1\t1\t%s\tRECEIPT\n' "$conf"
  printf '5\t1\t1\t1\t2\t1\t0\t0\t1\t1\t%s\tJUNE\n' "$conf"
  printf '5\t1\t1\t1\t2\t2\t0\t0\t1\t1\t%s\t2025\n' "$conf"
}} > "$2.tsv"
"#,
            log = log.display(),
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

    fn invocations(directory: &std::path::Path) -> Vec<Invocation> {
        std::fs::read_to_string(directory.join("invocations.log"))
            .unwrap()
            .lines()
            .map(|line| {
                let fields: Vec<&str> = line.split_whitespace().collect();
                let byte = |index: usize| fields[2 + index].parse::<u32>().unwrap();
                let be = |at: usize| {
                    (byte(at) << 24) | (byte(at + 1) << 16) | (byte(at + 2) << 8) | byte(at + 3)
                };
                Invocation {
                    mode: fields[0].to_owned(),
                    output: fields[1].to_owned(),
                    width: be(0),
                    height: be(4),
                    colour_type: byte(9) as u8,
                }
            })
            .collect()
    }

    fn invocation(mode: &str, output: &str, width: u32, height: u32) -> Invocation {
        Invocation {
            mode: mode.to_owned(),
            output: output.to_owned(),
            width,
            height,
            // PNG colour type 0 is greyscale.
            colour_type: 0,
        }
    }

    /// A 40 x 20 colour page, so a quarter turn shows in the dimensions.
    fn page() -> RenderedPage {
        RenderedPage::new(0, DynamicImage::ImageRgb8(RgbImage::new(40, 20)))
    }

    #[test]
    fn confident_upright_page_skips_osd() {
        let directory = tempfile::tempdir().unwrap();
        let ocr = fake_tesseract(directory.path(), 180, "ocr-upright");

        let reading = ocr.recognize(&page(), &CancellationToken::new()).unwrap();

        assert_eq!(
            invocations(directory.path()),
            [invocation("recognize", "ocr-upright", 40, 20)]
        );
        assert_eq!(reading.rotation_degrees, 0);
        assert_eq!(reading.text, "DELIVERY RECEIPT\nJUNE 2025");
        assert_eq!(reading.mean_confidence, 93.0);
    }

    #[test]
    fn unconvincing_page_runs_osd_on_half_scale_then_search() {
        let directory = tempfile::tempdir().unwrap();
        let ocr = fake_tesseract(directory.path(), 90, "ocr-rotated-270");

        let reading = ocr.recognize(&page(), &CancellationToken::new()).unwrap();

        assert_eq!(
            invocations(directory.path()),
            [
                invocation("recognize", "ocr-upright", 40, 20),
                invocation("osd", "orientation", 20, 10),
                invocation("recognize", "ocr-rotated-90", 20, 40),
                invocation("recognize", "ocr-rotated-270", 20, 40),
            ]
        );
        assert_eq!(reading.rotation_degrees, 270);
        assert_eq!(reading.mean_confidence, 93.0);
    }

    /// Every pass the adapter makes is counted on the request's token, and
    /// its time is split between encoding what Tesseract is handed and
    /// waiting for Tesseract.
    #[test]
    fn the_adapter_counts_its_passes_and_splits_their_time() {
        let directory = tempfile::tempdir().unwrap();
        let ocr = fake_tesseract(directory.path(), 90, "ocr-rotated-270");
        let cancel = CancellationToken::new();

        ocr.recognize(&page(), &cancel).unwrap();

        let timings = cancel.timings();
        assert_eq!(timings.ocr_passes, 3, "{timings:?}");
        assert_eq!(timings.orientation_passes, 1, "{timings:?}");
        assert!(timings.ocr_encode_micros > 0, "{timings:?}");
        assert!(timings.ocr_engine_micros > 0, "{timings:?}");
        // The page itself is the reader's to count, not the adapter's.
        assert_eq!(timings.ocr_pages, 0, "{timings:?}");
    }
}

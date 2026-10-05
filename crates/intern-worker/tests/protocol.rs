use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionError, OcrBackend, OcrResult, PdfBackend,
    PdfPageInspection, RenderedPage, extract_pdf,
};
use intern_worker::limits::{MAX_DOCUMENT_CHARS, MAX_PAGE_CHARS, ResourceLimits};
use intern_worker::protocol::{
    MAX_PROTOCOL_LINE_BYTES, PROGRESS_INTERVAL, handle_line, run_concurrent_worker,
    run_concurrent_worker_observed, run_control_loop,
};

#[test]
fn hello_reports_exact_protocol_version() {
    let response =
        handle_line(r#"{"protocol_version":1,"request_id":"r1","command":{"type":"hello"}}"#)
            .unwrap();

    assert_eq!(
        response,
        r#"{"protocol_version":1,"request_id":"r1","event":{"type":"hello","worker_version":"0.1.0-alpha.11"}}"#
    );
}

#[test]
fn malformed_json_is_an_error_event_and_the_next_request_is_processed() {
    let input = joined_lines([b"not json\n".to_vec(), hello_line("r2"), shutdown_line()]);
    let mut output = Vec::new();
    let mut diagnostics = Vec::new();

    run_control_loop(
        Cursor::new(&input),
        &mut output,
        &mut diagnostics,
        |_request, _sink| unreachable!("no parse request was supplied"),
    )
    .unwrap();

    let lines: Vec<&str> = std::str::from_utf8(&output).unwrap().lines().collect();
    assert_eq!(lines.len(), 2);
    let error: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(error["event"]["type"], "error");
    assert_eq!(error["event"]["code"], "PARSE_FAILED");
    assert_eq!(error["request_id"], "");
    assert_eq!(
        lines[1],
        r#"{"protocol_version":1,"request_id":"r2","event":{"type":"hello","worker_version":"0.1.0-alpha.11"}}"#
    );
    assert!(
        std::str::from_utf8(&diagnostics)
            .unwrap()
            .contains("PARSE_FAILED")
    );
}

#[test]
fn invalid_utf8_is_rejected_and_the_next_request_is_processed() {
    let mut input = vec![0xff, b'\n'];
    input.extend_from_slice(&hello_line("after-utf8"));
    let mut output = Vec::new();

    run_control_loop(
        Cursor::new(input),
        &mut output,
        Vec::new(),
        |_request, _sink| unreachable!(),
    )
    .unwrap();

    let events: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events[0]["event"]["code"], "PARSE_FAILED");
    assert_eq!(events[1]["request_id"], "after-utf8");
    assert_eq!(events[1]["event"]["type"], "hello");
}

#[test]
fn oversized_line_is_drained_and_the_next_request_is_processed() {
    let mut input = vec![b'x'; MAX_PROTOCOL_LINE_BYTES + 1];
    input.push(b'\n');
    input.extend_from_slice(&hello_line("after-large"));
    let mut output = Vec::new();

    run_control_loop(
        Cursor::new(input),
        &mut output,
        Vec::new(),
        |_request, _sink| unreachable!(),
    )
    .unwrap();

    let events: Vec<serde_json::Value> = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events[0]["event"]["code"], "PARSE_FAILED");
    assert_eq!(events[1]["request_id"], "after-large");
    assert_eq!(events[1]["event"]["type"], "hello");
}

#[test]
fn unsupported_protocol_version_returns_stable_version_error() {
    let response =
        handle_line(r#"{"protocol_version":2,"request_id":"r3","command":{"type":"hello"}}"#)
            .unwrap();
    let event: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert_eq!(event["protocol_version"], 1);
    assert_eq!(event["request_id"], "r3");
    assert_eq!(event["event"]["type"], "error");
    assert_eq!(event["event"]["code"], "PROTOCOL_VERSION_UNSUPPORTED");
    assert_eq!(event["event"]["retryable"], false);
}

#[test]
fn stdout_contains_only_flushed_json_lines() {
    struct FlushCountingWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl std::io::Write for FlushCountingWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    let input = joined_lines([hello_line("r4"), shutdown_line()]);
    let mut output = FlushCountingWriter {
        bytes: Vec::new(),
        flushes: 0,
    };
    let mut diagnostics = Vec::new();

    run_control_loop(
        Cursor::new(&input),
        &mut output,
        &mut diagnostics,
        |_request, _sink| unreachable!(),
    )
    .unwrap();

    assert_eq!(output.flushes, 1);
    assert!(
        std::str::from_utf8(&output.bytes)
            .unwrap()
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
    assert!(diagnostics.is_empty());
}

#[test]
fn cancel_interrupts_the_active_request_and_shutdown_joins_it() {
    #[derive(Clone, Default)]
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let input = br#"{"protocol_version":1,"request_id":"parse-1","command":{"type":"parse","path":"scan.pdf"}}
{"protocol_version":1,"request_id":"cancel-1","command":{"type":"cancel","target_request_id":"parse-1"}}
{"protocol_version":1,"request_id":"done","command":{"type":"shutdown"}}
"#;
    let output = SharedWriter::default();
    let captured = output.clone();
    let mut diagnostics = Vec::new();

    run_concurrent_worker(
        Cursor::new(input),
        output,
        &mut diagnostics,
        |_path, cancel| {
            loop {
                cancel.check()?;
                std::thread::sleep(Duration::from_millis(1));
            }
        },
    )
    .unwrap();

    let bytes = captured.0.lock().unwrap().clone();
    let events: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        events
            .iter()
            .any(|event| event["event"]["stage"] == "cancel_requested")
    );
    assert!(
        events
            .iter()
            .any(|event| event["event"]["code"] == "CANCELED")
    );
    assert!(diagnostics.is_empty());
}

#[derive(Clone, Default)]
struct SignalingWriter(Arc<(Mutex<Vec<u8>>, Condvar)>);

impl Write for SignalingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let (output, changed) = &*self.0;
        output.lock().unwrap().extend_from_slice(bytes);
        changed.notify_all();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct TerminalGatedReader {
    chunks: Vec<Vec<u8>>,
    next: usize,
    output: SignalingWriter,
    first_terminal: &'static [u8],
}

impl std::io::Read for TerminalGatedReader {
    fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
        if self.next >= self.chunks.len() {
            return Ok(0);
        }
        if self.next == 1 {
            let (bytes, changed) = &*self.output.0;
            let mut bytes = bytes.lock().unwrap();
            while !bytes
                .windows(self.first_terminal.len())
                .any(|window| window == self.first_terminal)
            {
                bytes = changed.wait(bytes).unwrap();
            }
        }
        let chunk = &self.chunks[self.next];
        assert!(chunk.len() <= target.len());
        target[..chunk.len()].copy_from_slice(chunk);
        self.next += 1;
        Ok(chunk.len())
    }
}

fn empty_document() -> ExtractedDocument {
    ExtractedDocument {
        pages: vec![],
        warnings: vec![],
        truncated: false,
        optional_image: None,
    }
}

fn parse_line(request_id: &str, path: &str) -> Vec<u8> {
    let mut line = serde_json::to_vec(&serde_json::json!({
        "protocol_version": 1,
        "request_id": request_id,
        "command": { "type": "parse", "path": path },
    }))
    .unwrap();
    line.push(b'\n');
    line
}

fn hello_line(request_id: &str) -> Vec<u8> {
    let mut line = serde_json::to_vec(&serde_json::json!({
        "protocol_version": 1,
        "request_id": request_id,
        "command": { "type": "hello" },
    }))
    .unwrap();
    line.push(b'\n');
    line
}

fn shutdown_line() -> Vec<u8> {
    let mut line =
        br#"{"protocol_version":1,"request_id":"done","command":{"type":"shutdown"}}"#.to_vec();
    line.push(b'\n');
    line
}

fn joined_lines(lines: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    lines.into_iter().flatten().collect()
}

#[test]
fn a_parse_sent_after_the_terminal_event_is_not_reported_busy() {
    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("first", "one.txt"),
            joined_lines([parse_line("second", "two.txt"), shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"\"type\":\"parsed\"",
    };

    run_concurrent_worker(reader, captured.clone(), Vec::new(), |_path, _cancel| {
        Ok(empty_document())
    })
    .unwrap();
    let (bytes, _) = &*captured.0;
    let bytes = bytes.lock().unwrap().clone();
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(text.matches("\"type\":\"parsed\"").count(), 2);
    assert!(!text.contains("WORKER_BUSY"));
}

#[test]
fn extractor_panic_is_terminal_and_does_not_leak_the_active_slot() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("panic", "one.txt"),
            joined_lines([parse_line("after", "two.txt"), shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"WORKER_THREAD_PANIC",
    };
    let calls = Arc::new(AtomicUsize::new(0));

    run_concurrent_worker(
        reader,
        captured.clone(),
        Vec::new(),
        move |_path, _cancel| {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                panic!("fixture panic");
            }
            Ok(empty_document())
        },
    )
    .unwrap();
    let (bytes, _) = &*captured.0;
    let bytes = bytes.lock().unwrap().clone();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("WORKER_THREAD_PANIC"));
    assert!(text.contains("\"request_id\":\"after\",\"event\":{\"type\":\"parsed\""));
    assert!(!text.contains("WORKER_BUSY"));
}

#[test]
fn nul_in_request_id_cannot_break_thread_start_or_leak_the_active_slot() {
    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("nul\0id", "one.txt"),
            joined_lines([parse_line("after-nul", "two.txt"), shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"\"type\":\"parsed\"",
    };

    run_concurrent_worker(reader, captured.clone(), Vec::new(), |_path, _cancel| {
        Ok(empty_document())
    })
    .unwrap();

    let (bytes, _) = &*captured.0;
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert_eq!(text.matches("\"type\":\"parsed\"").count(), 2);
    assert!(text.contains(r#""request_id":"nul\u0000id""#));
    assert!(!text.contains("WORKER_THREAD_START_FAILED"));
    assert!(!text.contains("WORKER_BUSY"));
}

/// Every extractor but the spreadsheet reader can hand back a page of any
/// size at all, and the host reads a response line without a bound, so one
/// degenerate file could put hundreds of megabytes through the pipe and into
/// the queue's memory. The cap lives at the boundary so it holds for every
/// reader, including ones added later.
#[test]
fn a_page_longer_than_the_cap_is_truncated_before_it_is_emitted() {
    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("huge", "one.txt"),
            joined_lines([shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"\"type\":\"parsed\"",
    };

    run_concurrent_worker(reader, captured.clone(), Vec::new(), |_path, _cancel| {
        Ok(ExtractedDocument {
            pages: vec![intern_worker::extract::ExtractedPage {
                page_number: 1,
                text: "é".repeat(MAX_PAGE_CHARS + 1_000),
                source: intern_worker::extract::PageSource::Text,
                ocr_confidence: None,
                vision_escalated: false,
            }],
            warnings: vec![],
            truncated: false,
            optional_image: None,
        })
    })
    .unwrap();

    let (bytes, _) = &*captured.0;
    let bytes = bytes.lock().unwrap().clone();
    let parsed = String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|event| event["event"]["type"] == "parsed")
        .unwrap();

    assert_eq!(
        parsed["event"]["document"]["pages"][0]["text"]
            .as_str()
            .unwrap()
            .chars()
            .count(),
        MAX_PAGE_CHARS
    );
    assert_eq!(parsed["event"]["document"]["truncated"], true);
    assert_eq!(parsed["event"]["document"]["warnings"][0], "TEXT_TRUNCATED");
}

fn page(number: usize, text: String) -> intern_worker::extract::ExtractedPage {
    intern_worker::extract::ExtractedPage {
        page_number: number,
        text,
        source: intern_worker::extract::PageSource::AnyDoc,
        ocr_confidence: None,
        vision_escalated: false,
    }
}

/// Every page under its own cap can still add up: five hundred sheets of two
/// million characters is a gigabyte on one response line, read whole into
/// the app. The document as a whole stops at its own cap; the pages past it
/// arrive empty, so page numbers keep their meaning.
#[test]
fn document_char_cap_truncates_later_pages() {
    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("workbook", "book.xlsx"),
            joined_lines([shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"\"type\":\"parsed\"",
    };
    let pages = MAX_DOCUMENT_CHARS / MAX_PAGE_CHARS + 2;

    run_concurrent_worker(
        reader,
        captured.clone(),
        Vec::new(),
        move |_path, _cancel| {
            Ok(ExtractedDocument {
                pages: (1..=pages)
                    .map(|number| page(number, "\u{e9}".repeat(MAX_PAGE_CHARS)))
                    .collect(),
                warnings: vec![],
                truncated: false,
                optional_image: None,
            })
        },
    )
    .unwrap();

    let (bytes, _) = &*captured.0;
    let bytes = bytes.lock().unwrap().clone();
    let parsed = String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|event| event["event"]["type"] == "parsed")
        .unwrap();
    let document = &parsed["event"]["document"];
    let lengths = document["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page["text"].as_str().unwrap().chars().count())
        .collect::<Vec<_>>();

    assert_eq!(lengths.len(), pages);
    assert_eq!(lengths.iter().sum::<usize>(), MAX_DOCUMENT_CHARS);
    assert_eq!(lengths[pages - 1], 0);
    assert_eq!(document["truncated"], true);
    assert_eq!(document["warnings"][0], "TEXT_TRUNCATED");
}

/// Hands a parse request over only once every earlier one has finished, the
/// way the host drives the worker: one document at a time, for as long as
/// the app runs.
struct SequentialReader {
    lines: Vec<Vec<u8>>,
    next: usize,
    output: SignalingWriter,
}

impl std::io::Read for SequentialReader {
    fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
        if self.next >= self.lines.len() {
            return Ok(0);
        }
        let (bytes, changed) = &*self.output.0;
        let mut bytes = bytes.lock().unwrap();
        let finished = |bytes: &[u8]| {
            bytes
                .windows(b"\"type\":\"parsed\"".len())
                .filter(|window| *window == b"\"type\":\"parsed\"")
                .count()
        };
        while finished(&bytes) < self.next {
            bytes = changed.wait(bytes).unwrap();
        }
        drop(bytes);
        let line = &self.lines[self.next];
        assert!(line.len() <= target.len());
        target[..line.len()].copy_from_slice(line);
        self.next += 1;
        Ok(line.len())
    }
}

/// The host keeps one worker for the whole session, and a watched folder can
/// feed it tens of thousands of documents. Each finished extraction thread
/// is joined when the next one starts instead of piling up until shutdown.
#[test]
fn finished_threads_are_reaped() {
    const DOCUMENTS: usize = 24;
    let output = SignalingWriter::default();
    let captured = output.clone();
    let mut lines = (0..DOCUMENTS)
        .map(|index| parse_line(&format!("document-{index}"), "one.txt"))
        .collect::<Vec<_>>();
    lines.push(shutdown_line());
    let reader = SequentialReader {
        lines,
        next: 0,
        output,
    };
    let held = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&held);

    run_concurrent_worker_observed(
        reader,
        captured.clone(),
        Vec::new(),
        |_path, _cancel| Ok(empty_document()),
        move |threads| observed.lock().unwrap().push(threads),
    )
    .unwrap();

    let held = held.lock().unwrap();
    assert_eq!(held.len(), DOCUMENTS);
    // The previous thread may still be returning when the next starts, so
    // two is the steady state; anything near the document count is a leak.
    assert!(
        held.iter().all(|threads| *threads <= 3),
        "threads held after each start: {held:?}"
    );
}

/// A scanned PDF of `SCANNED_PAGES` image-only pages.
struct ScannedPdf;

const SCANNED_PAGES: usize = 40;

impl PdfBackend for ScannedPdf {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        Ok((0..SCANNED_PAGES)
            .map(|page_index| PdfPageInspection {
                page_index,
                native_text: String::new(),
                image_coverage: 1.0,
                width_pixels: 10,
                height_pixels: 10,
            })
            .collect())
    }

    fn render_within(
        &self,
        _path: &Path,
        page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        Ok(RenderedPage::new(
            page_index,
            image::DynamicImage::ImageLuma8(image::GrayImage::new(10, 10)),
        ))
    }
}

/// OCR at 25 ms a page: a 40-page scan takes a second.
struct SteadyOcr;

impl OcrBackend for SteadyOcr {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        std::thread::sleep(Duration::from_millis(25));
        Ok(OcrResult::new("SCANNED PAGE OF THE LEASE", 90.0))
    }
}

/// A 200-page scan used to show 0% for minutes: the worker said it had
/// started and then nothing until it had finished. It now says how many
/// pages it has finished and how many there are - a `reading` event as each
/// page is reached and an `ocr` event as each goes to OCR - and never more
/// than one of either per interval, however fast the pages go by. The count
/// is of pages finished, so the first says nothing is done yet.
#[test]
fn parse_emits_throttled_page_progress() {
    let output = SignalingWriter::default();
    let captured = output.clone();
    let reader = TerminalGatedReader {
        chunks: vec![
            parse_line("scan", "scan.pdf"),
            joined_lines([shutdown_line()]),
        ],
        next: 0,
        output,
        first_terminal: b"\"type\":\"parsed\"",
    };
    let started = Instant::now();

    run_concurrent_worker(reader, captured.clone(), Vec::new(), |path, cancel| {
        extract_pdf(
            &path,
            &ScannedPdf,
            &SteadyOcr,
            &ResourceLimits::default(),
            &cancel,
        )
    })
    .unwrap();

    let elapsed = started.elapsed();
    let (bytes, _) = &*captured.0;
    let events: Vec<serde_json::Value> = String::from_utf8(bytes.lock().unwrap().clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let parsed_at = events
        .iter()
        .position(|event| event["event"]["type"] == "parsed")
        .expect("the scan is parsed");
    let progress = |stage: &str| {
        events
            .iter()
            .enumerate()
            .filter(|(_, event)| {
                event["event"]["type"] == "progress" && event["event"]["stage"] == stage
            })
            .map(|(index, event)| {
                assert!(index < parsed_at, "progress after the terminal event");
                assert_eq!(event["request_id"], "scan");
                assert_eq!(event["event"]["total"], SCANNED_PAGES);
                event["event"]["current"].as_u64().unwrap() as usize
            })
            .collect::<Vec<_>>()
    };
    let reading = progress("reading");
    let ocr = progress("ocr");
    // At most one of each stage per interval, plus the first.
    let allowed = 1 + (elapsed.as_millis() / PROGRESS_INTERVAL.as_millis()) as usize;

    assert_eq!(reading.first(), Some(&0), "{reading:?}");
    assert_eq!(ocr.first(), Some(&0), "{ocr:?}");
    // A second of OCR is long enough to say more than that it started.
    assert!(reading.len() >= 2, "{reading:?} in {elapsed:?}");
    assert!(reading.len() <= allowed, "{reading:?} in {elapsed:?}");
    assert!(ocr.len() <= allowed, "{ocr:?} in {elapsed:?}");
    assert!(reading.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        reading
            .iter()
            .chain(&ocr)
            .all(|finished| (0..SCANNED_PAGES).contains(finished))
    );
}

/// Diagnostics go to a log file, and a write to it can fail (a full disk,
/// an I/O error on the log's volume). That must never stop the worker from
/// answering: the log is never a reason not to work.
#[test]
fn a_diagnostics_log_that_cannot_be_written_never_stops_the_worker() {
    struct FailingLog;
    impl Write for FailingLog {
        fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("no space left on device"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("no space left on device"))
        }
    }

    let input = joined_lines([
        b"not json\n".to_vec(),
        hello_line("after-bad-line"),
        shutdown_line(),
    ]);
    let mut output = Vec::new();
    run_control_loop(
        Cursor::new(&input),
        &mut output,
        FailingLog,
        |_request, _sink| unreachable!("no parse request was supplied"),
    )
    .expect("a failing log is not a failing worker");
    assert!(
        String::from_utf8(output)
            .unwrap()
            .contains("after-bad-line")
    );

    let output = SignalingWriter::default();
    let captured = output.clone();
    run_concurrent_worker(Cursor::new(input), output, FailingLog, |_path, _cancel| {
        Ok(empty_document())
    })
    .expect("a failing log is not a failing worker");
    let (bytes, _) = &*captured.0;
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(
        text.contains("after-bad-line"),
        "hello must still be answered: {text}"
    );
}

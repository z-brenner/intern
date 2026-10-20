//! Indra page assembly against the reference package (spec §8, §17).
//!
//! Each fixture under `tests/fixtures/indra/order/` holds one golden page of
//! `indra-ocr` 1.1.2: the detected boxes and the recognizer's readings fed
//! to `OrtOCR.assemble`, the order network's outputs for the one call it
//! made, and what it returned (lines with boxes on the levelled page, blocks
//! and markdown). `tools/dump_assemble.py` wrote them from the reference's
//! `stages/` goldens. The order network is replayed: the closure checks the
//! features it is handed against the recorded ones and answers with the
//! recorded outputs, so everything else is this crate's own work.
//!
//! The base recognizer's tokens are class ids, read through the vocabulary.
//! While `Vocab::decode` is not built on this branch, the pages are read
//! through `TestVocab`, a transcription of the reference's `Vocab.decode`;
//! once it is, every page is also read through `assemble` and the real
//! `Vocab`, to the same expectations.
//!
//! With `INDRA_REFERENCE_DIR` pointing at the reference goldens
//! (`indra-reference/`), the fixtures are also checked against
//! `stages/*.stages.json` and the dumped order tensors in `intermediate/`.

use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use intern_worker::indra::assemble::{self, AssembleOptions, IdText, pystr};
use intern_worker::indra::npy;
use intern_worker::indra::types::{DetBox, OrderOutputs, PageReading, RecLine, Token, TokenKind};
use intern_worker::indra::vocab::Vocab;
use serde_json::Value;

/// The pages with per-stage goldens (`stages/`, and for four of them
/// `intermediate/`); the other fixtures come from whole runs
/// (`tools/dump_full.py`) of the reference's other inputs.
const STAGES: [&str; 5] = [
    "document-image",
    "multi_column",
    "rotated-low-resolution-scan",
    "table_pmc",
    "tilted_memo",
];

/// Every fixture page, by file stem.
fn pages() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixtures())
        .expect("fixtures")
        .filter_map(|e| {
            let path = e.ok()?.path();
            let stem = path.file_stem()?.to_str()?.to_string();
            (path.extension()? == "json" && stem != "vocab").then_some(stem)
        })
        .collect();
    names.sort();
    names
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/indra/order")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("fixture JSON")
}

fn floats(b64: &Value) -> Vec<f32> {
    let bytes = STANDARD
        .decode(b64.as_str().expect("base64"))
        .expect("base64");
    let (words, rest) = bytes.as_chunks::<4>();
    assert!(rest.is_empty(), "whole float32 values");
    words.iter().map(|w| f32::from_le_bytes(*w)).collect()
}

fn num(v: &Value) -> f64 {
    v.as_f64().expect("number")
}

/// The reference `Vocab.decode` for the base vocabulary (no jamo; LaTeX
/// commands present).
struct TestVocab {
    chars: Vec<String>,
}

impl IdText for TestVocab {
    fn decode(&self, ids: &[u32]) -> String {
        let toks: Vec<&str> = ids
            .iter()
            .filter(|&&i| i > 0 && (i as usize) <= self.chars.len())
            .map(|&i| self.chars[i as usize - 1].as_str())
            .collect();
        let mut out = String::new();
        for (k, t) in toks.iter().enumerate() {
            out.push_str(t);
            let next = toks.get(k + 1).copied().unwrap_or("");
            let head: String = next.chars().take(1).collect();
            if t.chars().count() > 2
                && t.starts_with('\\')
                && pystr::str_isalpha(&t[1..])
                && pystr::str_isalpha(&head)
            {
                out.push(' ');
            }
        }
        out
    }

    fn space_id(&self) -> Option<u32> {
        let i = self.chars.iter().position(|c| c == " ")?;
        Some(i as u32 + 1)
    }
}

struct Page {
    name: String,
    width: usize,
    height: usize,
    boxes: Vec<DetBox>,
    lines: Vec<RecLine>,
    calls: Vec<(usize, Vec<f32>, OrderOutputs)>,
    expected: Value,
}

fn load(name: &str) -> Page {
    let d = read_json(&fixtures().join(format!("{name}.json")));
    let boxes = d["det_boxes"]
        .as_array()
        .expect("boxes")
        .iter()
        .map(|b| {
            let f = |k: usize| num(&b[k]) as f32;
            DetBox {
                x0: f(0),
                y0: f(1),
                x1: f(2),
                y1: f(3),
                score: f(4),
            }
        })
        .collect();
    let lines = d["rec"]
        .as_array()
        .expect("rec")
        .iter()
        .map(|r| RecLine {
            text: r[0].as_str().expect("text").to_string(),
            conf: num(&r[1]) as f32,
            tokens: r[2]
                .as_array()
                .expect("tokens")
                .iter()
                .map(|t| Token {
                    kind: match &t[0] {
                        Value::String(s) => TokenKind::Char(s.clone()),
                        v => TokenKind::Id(v.as_u64().expect("id") as u32),
                    },
                    x: num(&t[1]),
                    p: 1.0,
                })
                .collect(),
        })
        .collect();
    let calls = d["order_calls"]
        .as_array()
        .expect("calls")
        .iter()
        .map(|c| {
            let n = c["n"].as_u64().expect("n") as usize;
            let outputs = OrderOutputs {
                n,
                a: floats(&c["a"]),
                cls: floats(&c["cls"]),
                same: floats(&c["same"]),
            };
            (n, floats(&c["feats"]), outputs)
        })
        .collect();
    Page {
        name: name.to_string(),
        width: d["width"].as_u64().expect("width") as usize,
        height: d["height"].as_u64().expect("height") as usize,
        boxes,
        lines,
        calls,
        expected: d,
    }
}

/// An order closure replaying the recorded calls; it records the largest
/// feature difference it saw.
fn replay<'a>(
    page: &'a Page,
    calls: &'a mut usize,
    max_diff: &'a mut f32,
) -> impl FnMut(&[f32], usize) -> OrderOutputs + 'a {
    move |feats: &[f32], n: usize| {
        let (rn, rf, out) = page
            .calls
            .get(*calls)
            .unwrap_or_else(|| panic!("{}: unexpected order call {}", page.name, *calls));
        *calls += 1;
        assert_eq!(n, *rn, "{}: order call size", page.name);
        assert_eq!(feats.len(), rf.len(), "{}: feature count", page.name);
        for (k, (a, b)) in feats.iter().zip(rf).enumerate() {
            let d = (a - b).abs();
            *max_diff = max_diff.max(d);
            assert!(
                d <= 1e-5 * b.abs().max(1.0),
                "{}: feature {} of line {}: {a} vs {b}",
                page.name,
                k % 16,
                k / 16
            );
        }
        out.clone()
    }
}

/// The page against the reference's lines, blocks and markdown; returns
/// how many boxes and confidences matched exactly.
fn check(page: &Page, got: &PageReading) -> (usize, usize) {
    let e = &page.expected;
    let lines = e["lines"].as_array().expect("lines");
    assert_eq!(got.lines.len(), lines.len(), "{}: line count", page.name);
    let (mut exact_box, mut exact_conf) = (0, 0);
    for (k, (g, x)) in got.lines.iter().zip(lines).enumerate() {
        assert_eq!(
            g.text,
            x["text"].as_str().expect("text"),
            "{}: line {k}",
            page.name
        );
        let bx: Vec<f64> = x["box"].as_array().expect("box").iter().map(num).collect();
        for c in 0..4 {
            assert!(
                (g.bbox[c] - bx[c]).abs() <= 0.05,
                "{}: line {k} box {:?} vs {bx:?}",
                page.name,
                g.bbox
            );
        }
        exact_box += usize::from(g.bbox.to_vec() == bx);
        let conf = num(&x["conf"]);
        assert!(
            (g.conf - conf).abs() <= 1e-4,
            "{}: line {k} conf",
            page.name
        );
        exact_conf += usize::from(g.conf == conf);
    }
    let blocks = e["blocks"].as_array().expect("blocks");
    assert_eq!(got.blocks.len(), blocks.len(), "{}: block count", page.name);
    for (k, (g, x)) in got.blocks.iter().zip(blocks).enumerate() {
        assert_eq!(
            g.class.label(),
            x["label"].as_str().expect("label"),
            "{}: block {k}",
            page.name
        );
        assert_eq!(
            g.text,
            x["text"].as_str().expect("text"),
            "{}: block {k}",
            page.name
        );
        let bx: Vec<f64> = x["bbox"]
            .as_array()
            .expect("bbox")
            .iter()
            .map(num)
            .collect();
        assert_eq!(g.bbox.to_vec(), bx, "{}: block {k} bbox", page.name);
        let joined: Vec<&str> = g
            .lines
            .iter()
            .map(|&i| got.lines[i].text.as_str())
            .collect();
        assert_eq!(joined.join("\n"), g.text, "{}: block {k} lines", page.name);
    }
    assert_eq!(
        got.markdown,
        e["markdown"].as_str().expect("markdown"),
        "{}: markdown",
        page.name
    );
    let text: Vec<&str> = got.lines.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(got.text, text.join("\n"));
    (exact_box, exact_conf)
}

fn vocab_chars() -> Vec<String> {
    let v = read_json(&fixtures().join("vocab.json"));
    v["chars"]
        .as_array()
        .expect("chars")
        .iter()
        .map(|c| c.as_str().expect("char").to_string())
        .collect()
}

#[test]
fn assembles_the_golden_pages() {
    let chars = vocab_chars();
    let test_vocab = TestVocab {
        chars: chars.clone(),
    };
    let vocab = Vocab {
        chars,
        jamo: false,
        has_commands: true,
    };
    // Whether Vocab::decode is built on this branch yet.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let real_decode = std::panic::catch_unwind(|| vocab.decode(&[1])).is_ok();
    std::panic::set_hook(hook);
    if !real_decode {
        eprintln!("note: Vocab::decode is not built yet; pages read through TestVocab only");
    }
    let names = pages();
    assert!(names.len() >= 5, "fixture pages: {names:?}");
    for name in &names {
        let page = load(name);
        let (mut calls, mut max_diff) = (0, 0.0f32);
        let assembly = assemble::assemble_with(
            page.width,
            page.height,
            &page.boxes,
            &page.lines,
            &test_vocab,
            AssembleOptions::default(),
            &mut replay(&page, &mut calls, &mut max_diff),
        );
        assert_eq!(calls, page.calls.len(), "{name}: order calls");
        let (exact_box, exact_conf) = check(&page, &assembly.page);
        eprintln!(
            "{name}: {} lines, {} blocks, markdown identical; boxes exact {exact_box}, \
             conf exact {exact_conf}; feature max |diff| {max_diff:e}",
            assembly.page.lines.len(),
            assembly.page.blocks.len()
        );
        assert_eq!(assembly.tables.len(), assembly.page.blocks.len());
        if real_decode {
            let (mut calls, mut max_diff) = (0, 0.0f32);
            let got = assemble::assemble(
                page.width,
                page.height,
                &page.boxes,
                &page.lines,
                &vocab,
                &mut replay(&page, &mut calls, &mut max_diff),
            );
            check(&page, &got);
        }
    }
}

#[test]
fn table_grids_match_the_markdown_tables() {
    let test_vocab = TestVocab {
        chars: vocab_chars(),
    };
    let page = load("table_pmc");
    let (mut calls, mut max_diff) = (0, 0.0f32);
    let assembly = assemble::assemble_with(
        page.width,
        page.height,
        &page.boxes,
        &page.lines,
        &test_vocab,
        AssembleOptions::default(),
        &mut replay(&page, &mut calls, &mut max_diff),
    );
    let tables: Vec<_> = assembly.tables.iter().flatten().collect();
    assert!(!tables.is_empty(), "table_pmc has tables");
    for grid in tables {
        assert!(assembly.page.markdown.contains(&grid.to_html()));
        for row in &grid.rows {
            assert_eq!(row.iter().map(|c| c.colspan).sum::<usize>(), grid.columns);
        }
    }
}

#[test]
fn keeps_furniture_when_asked() {
    let test_vocab = TestVocab {
        chars: vocab_chars(),
    };
    let page = load("multi_column");
    let (mut calls, mut max_diff) = (0, 0.0f32);
    let kept = assemble::assemble_with(
        page.width,
        page.height,
        &page.boxes,
        &page.lines,
        &test_vocab,
        AssembleOptions {
            keep_furniture: true,
        },
        &mut replay(&page, &mut calls, &mut max_diff),
    );
    let header = kept
        .page
        .blocks
        .iter()
        .find(|b| b.class.label() == "Page-Header")
        .expect("a page header");
    assert!(kept.page.markdown.contains(&header.text));
    assert_ne!(
        kept.page.markdown,
        page.expected["markdown"].as_str().unwrap_or("")
    );
}

#[test]
fn fixtures_agree_with_the_reference_goldens() {
    let Some(dir) = std::env::var_os("INDRA_REFERENCE_DIR") else {
        eprintln!("note: INDRA_REFERENCE_DIR is not set; the reference goldens are not checked");
        return;
    };
    let dir = PathBuf::from(dir);
    for name in pages() {
        let page = load(&name);
        if !STAGES.contains(&name.as_str()) {
            // A whole run: the finished page is in the JSON-lines goldens.
            let file = page.expected["file"].as_str().expect("file");
            let (input, index) = file.split_once('#').expect("file#page");
            let mode = if input.ends_with(".pdf") {
                "ocr"
            } else {
                "auto"
            };
            let golden = std::fs::read_to_string(dir.join(mode).join(format!("{input}.jsonl")))
                .expect("golden");
            let record: Value = golden
                .lines()
                .map(|l| serde_json::from_str::<Value>(l).expect("golden JSON"))
                .find(|r| r["page"].as_u64().map(|p| p.to_string()) == Some(index.to_string()))
                .expect("golden page");
            let texts = |v: &Value, key: &str| -> Vec<Value> {
                v[key]
                    .as_array()
                    .expect("array")
                    .iter()
                    .map(|l| l["text"].clone())
                    .collect()
            };
            assert_eq!(
                texts(&page.expected, "lines"),
                texts(&record, "lines"),
                "{name}"
            );
            assert_eq!(
                texts(&page.expected, "blocks"),
                texts(&record, "blocks"),
                "{name}"
            );
            assert_eq!(page.expected["markdown"], record["markdown"], "{name}");
            continue;
        }
        let stages = read_json(&dir.join("stages").join(format!("{name}.stages.json")));
        let e = &page.expected;
        let texts = |v: &Value, key: &str| -> Vec<String> {
            v[key]
                .as_array()
                .expect("array")
                .iter()
                .map(|l| l["text"].as_str().expect("text").to_string())
                .collect()
        };
        assert_eq!(texts(e, "lines"), texts(&stages, "final_lines"), "{name}");
        assert_eq!(texts(e, "blocks"), texts(&stages, "blocks"), "{name}");
        assert_eq!(e["markdown"], stages["markdown"], "{name}");
        assert_eq!(e["det_boxes"], stages["det_boxes"], "{name}");
        if num(&stages["skew_applied"]) == 0.0 {
            let boxes = |v: &Value, key: &str| -> Vec<Value> {
                v[key]
                    .as_array()
                    .expect("array")
                    .iter()
                    .map(|l| l["box"].clone())
                    .collect()
            };
            assert_eq!(boxes(e, "lines"), boxes(&stages, "final_lines"), "{name}");
        }
        let inter = dir.join("intermediate").join(&name);
        if inter.is_dir() {
            let f32s = |file: &str| {
                npy::read(&inter.join(file))
                    .expect("npy")
                    .as_f32()
                    .expect("f32")
                    .to_vec()
            };
            let (n, feats, outputs) = &page.calls[0];
            assert_eq!(&f32s("order.in.feats.npy"), feats, "{name}: feats");
            assert_eq!(f32s("order.out.a.npy"), outputs.a, "{name}: a");
            assert_eq!(f32s("order.out.cls.npy"), outputs.cls, "{name}: cls");
            assert_eq!(f32s("order.out.same.npy"), outputs.same, "{name}: same");
            assert_eq!(feats.len(), n * 16);
        }
    }
}

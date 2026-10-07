//! InternBench: Intern's whole path, measured over a generated corpus.
//!
//! The corpus evaluator (`intern-evaluate`) asks whether Intern names the
//! fixture corpus right. InternBench asks the same of a larger generated
//! corpus built to be hard - scans, long documents, competing dates,
//! parties only the layout identifies - and also asks what each answer
//! cost: where the time went, stage by stage, and how much memory the
//! worker and the model server took.
//!
//! * [`gold`] reads `bench/gold.json`, the reviewed answers.
//! * [`score`] scores one outcome against them; [`claims`] checks what a
//!   description asserts; [`ocr`] measures OCR against what was drawn.
//! * [`live`] runs the worker and the model; [`replay`] scores a
//!   [`recording`] again without either.
//! * [`report`] summarises the records, [`markdown`] renders them for a
//!   person, [`compare`] diffs two reports, and [`baseline`] gates a run.
//! * [`merge`] replaces or adds documents in a recording, so a few can be
//!   recorded again without the whole corpus.

#![deny(unsafe_code)]

pub mod baseline;
pub mod claims;
pub mod compare;
pub mod gold;
pub mod live;
pub mod machine;
pub mod markdown;
pub mod memory;
pub mod merge;
pub mod ocr;
pub mod record;
pub mod recording;
pub mod replay;
pub mod report;
pub mod run;
pub mod score;
pub mod stats;
pub mod timing;

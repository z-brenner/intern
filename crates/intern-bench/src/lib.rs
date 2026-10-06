//! InternBench: Intern's whole path, measured over a generated corpus.
//!
//! The corpus evaluator (`intern-evaluate`) asks whether Intern names the
//! fixture corpus right. InternBench asks the same of a larger generated
//! corpus built to be hard - scans, long documents, competing dates,
//! parties only the layout identifies.
//!
//! * [`gold`] reads `bench/gold.json`, the reviewed answers.
//! * [`score`] scores one outcome against them; [`claims`] checks what a
//!   description asserts; [`ocr`] measures OCR against what was drawn.

#![deny(unsafe_code)]

pub mod claims;
pub mod gold;
pub mod ocr;
pub mod score;
pub mod stats;

# Independent held-out fidelity gate

Sixteen synthetic tests were authored, hand-checked and frozen before production edits. The same immutable suite ran against the unchanged baseline and the frozen candidate. Answers and per-case results were withheld from the implementation loop until candidate freeze; the first candidate run passed the gate.

| Binary | Passed | Failed | Ignored | Exit |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 4 / 16 | 12 | 0 | 101 |
| Frozen candidate | 16 / 16 | 0 | 0 | 0 |

All twelve baseline failures were resolved and all four baseline controls continued to pass. No held-out tuning occurred.

The suite exercises complete measurements and numeric ranges, multiword units, Unicode numerals, both subject shortening boundaries, incomplete source measurements, explicit sender/recipient controls, and named parties whose routing is unproved. Fifteen cases use real indexing, retrieval, validation and composition; one directly tests a role explicitly marked unsupported. Mentioned quantities must remain complete, source-stated directions must hold, unsupported directions must be absent, and each description must fit 42 words. Whole optional quantities or subjects may be omitted.

Suite: `crates/intern-engine/tests/fidelity/heldout.rs`
SHA256: `18edd9c460c0085c4c266756efbff1177cc640a68e84102f4de08384abe7cce1`

Both native Linux Rust test binaries ran with `heldout_fidelity_ --test-threads=1 --nocapture`. The evaluator performed no compilation, external model calls, VM runs or GPU work. Exact commands, platform, timestamps, binary and candidate source hashes are in [heldout-results.json](heldout-results.json). Full synthetic stdout/stderr are in the four `heldout-*.txt` files.

These deliberately targeted synthetic cases are regression controls, not an estimate of real-document accuracy or model performance. The long-subject cases allow omission of a whole optional subject to satisfy the word budget; they establish quantity integrity, not retention of every source fact. Broader test and build evidence is maintained separately by the parent run.

## Final refinement regression recheck

After the v1 unseen gate passed and the suite was released, a public-corpus refinement retained a sole otherwise unclassified counterparty with optional neutral `also naming` wording. The identical frozen 16 tests were rerun against the final binary: **16 passed, 0 failed, 0 ignored; exit 0**. This is a regression recheck after v1, **not an additional unseen evaluation**. The original baseline/v1 results and transcripts above remain unchanged.

Final binary SHA256: `a25655691ab690131d73daf301b501bcd7351dd1832c9075ddecd8520a185e09`
Held-out SHA256 remains: `18edd9c460c0085c4c266756efbff1177cc640a68e84102f4de08384abe7cce1`

The final source and binary hashes were unchanged during execution. Exact final commands, source hashes, platform and timestamp appear under `final_refinement_regression_recheck` in [heldout-results.json](heldout-results.json); full synthetic output is in `heldout-final-refinement.stdout.txt` and `heldout-final-refinement.stderr.txt`. No compilation, model call or private user-document access was performed for this recheck.

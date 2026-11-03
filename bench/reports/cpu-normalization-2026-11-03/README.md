# Faster ASCII evidence normalization, with unchanged outputs

Measured 2026-11-03. This is a deterministic CPU optimization, **not a new model or live
inference result**. The model-download and Library-transfer blockers remain.

Intern repeatedly normalizes text while indexing, retrieving, validating evidence and
scoring results. Previously every ASCII-only string went through Unicode normalization and
case-folding. ASCII now takes a byte loop with the same case folding and whitespace behavior;
all non-ASCII strings retain the original implementation. Vertical tab is handled explicitly:
Rust's byte whitespace predicate omits it, while the character predicate includes it.

## Controlled replay measurement

Both binaries were built with the same Rust 1.88.0 toolchain, release profile, locked
dependencies and `benchmark-transport` feature. Control source: `a045a34`, in a separate clean
worktree. Candidate source: `6207447`. The actual executable hashes and every measurement are
in [measurements.json](measurements.json). Both run the same committed 77-document recording,
gold and fixture manifest, with no model server. The host is limited to four CPU cores and
16 GiB by cgroups. No compiler or test job ran during the timed sequence.

Six sequential runs alternated control/candidate/candidate/control/control/candidate:

| Measured quantity | Control | Candidate | Change |
|---|---:|---:|---:|
| Median whole replay/scoring process | 9.518 s | 4.911 s | 48.4% less time |
| Whole process range | 9.490-9.945 s | 4.880-4.917 s | All three candidate runs faster |
| Median total indexing across 77 documents | 1,550 ms | 1,008 ms | 35.0% less time |
| Median total retrieval across 77 documents | 60.60 ms | 56.37 ms | 7.0% less time |
| Largest OS process peak RSS | 35,920 KiB | 35,900 KiB | Effectively unchanged |

Five exploratory runs per arm preceded this matched-build sequence; their medians were
9.557 s and 4.895 s. They are not used for the table because the original control's feature
configuration was not recorded, which is why the control was rebuilt explicitly.

**Scope:** full replay includes benchmark scoring and serialization as well as engine work.
Its speedup is not the application's end-to-end speedup. Indexing and retrieval are actual
current engine timings; other latency fields in replay retain the old inference recording's
values and are not used here. No inference, extraction, laptop battery or background
responsiveness measurement was made. The RSS figures describe the replay process only.

## Correctness checks

- All 77 documents in all six timed runs passed the existing baseline with zero regressions.
- Every checked filename, description, readiness, review reason, validated/raw proposal,
  token-confidence value, claim, forbidden description, trap and score was identical between
  arms. The engine accepted all recorded prompt hashes without stale allowances.
- Both fixture-corpus replays (evidence and digest) passed with zero regressions and no
  stale/pending fixtures.
- The engine suite passed **504 tests**, with four existing tests ignored and no failures.
  New differential tests compare the fast path with the untouched Unicode implementation
  for all 128 ASCII bytes, all 16,384 byte pairs in context, deterministic long mixed runs,
  every ASCII whitespace form, and representative Unicode typography and case folding.
- Rust formatting, diff checks and the 20 model-qualification tests passed.

This preserves the baseline's known errors; unchanged recorded outputs do not establish
that the current model is accurate on unseen documents. No model, prompt, grammar, evidence
contract, baseline, confidence threshold, licence or dependency was changed.

## Reproduce

Build control and candidate separately with:

```bash
cargo build --release --locked -p intern-bench --features benchmark-transport
cargo test --release --locked -p intern-engine -- --test-threads=2
```

Retain each executable before building the other. From each source checkout, run the same
command with absolute paths pointing at the same generated corpus and committed inputs:

```bash
PATH_TO_BINARY/intern-bench run --corpus PATH_TO_REPO/bench/generated \
  --gold PATH_TO_REPO/bench/gold.json --manifest PATH_TO_REPO/bench/manifest.json \
  --replay PATH_TO_REPO/bench/recording.json --baseline PATH_TO_REPO/bench/baseline.json \
  --output NEW_REPORT.json
```

Use an external monotonic clock for each process and Linux `wait4` for its CPU time and peak
RSS. Alternate arm order, retain all reports, and compare document outputs and prompt
acceptance before interpreting timings. The exact local measurement harness and all six
reports remain under `target/cpu-normalization/`; this report commits the compact measurements
instead of duplicating the existing recorded documents.

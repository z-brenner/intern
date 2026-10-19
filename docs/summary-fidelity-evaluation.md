# Summary quantity and party-direction fidelity

This follow-up fixes two deterministic composition defects exposed by the native Windows model qualification. Switching models did not prevent either defect: a field subject was cut after eight words, and a sole remaining party was treated as the other side of a directional relationship.

## Resulting behavior

- Field subjects keep their complete source value. A numeric subject is kept intact or omitted as a whole when the existing 42-word budget cannot accommodate it; a word boundary no longer severs a measurement, range or condition.
- A secondary sender or recipient requires a supported role, or an existing explicit billing field. A sole other validated party can appear in an optional “also naming …” clause. Its presence alone never establishes “from” or “to”.
- Supported recipients remain directed. Filename selection, prompts, model choices, downloads and runtime lifecycle are unchanged.

For example, the published scan lease summary now keeps “14,860 rentable square feet” instead of ending at “rentable square”. The capital-call summary no longer describes the general partner as a recipient merely because it is the one other named party. An organization with only an `Other` role is named neutrally rather than assumed to be a notice's sender.

## Evaluation

| Check | Before | Final |
| --- | ---: | ---: |
| Independently frozen targeted synthetic suite | 4 / 16 | 16 / 16 |
| Public focused controls, including neutral name retention | — | 4 / 4 |
| InternBench complete descriptions | 21 / 77 | 23 / 77 |
| InternBench filename accuracy | 54 / 77 | 54 / 77 |
| InternBench unsafe-ready count | 15 | 15 |
| InternBench gated regressions / improvements | — | 0 / 2 |
| Three-run diagnostic replay median wall time | 63.97 s | 63.45 s |
| Diagnostic replay peak RSS | about 45 MiB | about 45 MiB |

The first candidate passed the unseen 16-case suite before answers were released. The existing public corpus then exposed two completeness regressions from omitting useful names; neutral naming resolved both. The final 16-case rerun is regression evidence, not a second unseen evaluation. No gold answers or recorded baseline were changed.

The frozen cases cover field and budget boundaries, multiword units, ranges, Unicode numerals, incomplete source measurements, coissuers, observers, supported recipients and unsupported roles. Fifteen cases use the actual indexing, retrieval, validation and composition path. The long-subject controls permit omission of an entire optional subject; they establish integrity rather than universal fact retention.

Native Linux engine and benchmark tests across all targets passed: **560 passed, 3 ignored**. Build/test checks used a 3 GiB memory limit, zero process swap and at most two CPUs. Installed Rust 1.98.1 strict Clippy reported newer-toolchain lint classes in unchanged code (`while_let_loop`, `cloned_ref_to_slice_refs`, `manual_is_multiple_of`, `chunks_exact_to_as_chunks`); a command-line-only compatibility run passed with those classes allowed and all other warnings denied. Repository CI remains pinned to Rust 1.88 with its original strict Clippy configuration. Source lint policies were not weakened.

All three unchanged and all three final 77-document replay runs completed successfully. These are debug-profile, native Linux engine replay measurements on the i5-13400F PC, using identical recordings and commands. They do not rerun extraction, OCR or model inference. Historical timings and hardware embedded in the recording are excluded from the wall/RSS comparison. Desktop activity and intermittent bounded compilation can affect timing; the small difference supports **no speed improvement claim**.

Full input/binary hashes, replicate measurements, summaries and all 13 changed descriptions are in [replay-comparison.json](../bench/reports/summary-fidelity-2026-10-19/replay-comparison.json). The [independent held-out report](../bench/reports/summary-fidelity-2026-10-19/heldout-results.md), synthetic transcripts and [code review](../bench/reports/summary-fidelity-2026-10-19/independent-review.md) preserve the gate history.

## Reproduction

Generate the established synthetic corpus with `node fixtures/generate-fixtures.mjs` and `node bench/generate.mjs`, then run:

```sh
cargo test --locked -p intern-engine -p intern-bench --all-targets \
  --features intern-bench/benchmark-transport
cargo run --locked -p intern-bench --features benchmark-transport -- \
  run --corpus bench/generated --gold bench/gold.json \
  --manifest bench/manifest.json --replay bench/recording.json \
  --baseline bench/baseline.json --output report.json
```

Use the CI toolchain (Rust 1.88) for the strict compatibility gate. The diagnostic local timing setup disabled debug information and incremental compilation; exact hashes and command are in the JSON report.

## Practical limits and next work

A new local Windows VM run was blocked: root storage had zero user-available space and MemAvailable remained below the existing 12 GiB guest-launch guard. Original VM files and evidence were preserved; no unrelated processes were stopped and no guard was lowered. Fresh hosted Windows checks are tracked on the pull request and are separate from local VM validation. The prior [native Windows model evaluation](bonsai-native-windows-evaluation.md) remains the live-model/OCR evidence.

This focused improvement does not fix the corpus's 15 existing unsafe-ready cases, wrong primary parties, conditional effective dates, every monetary qualifier, or OCR latency. The automatic “factual” scorer checks lexical source support and cannot prove relationship semantics; the targeted assertions supply additional direction checks. Numbers written entirely in words can still be shortened by the ordinary subject rule. Broader semantic cases need their own frozen evaluations.

Bonsai remains experimental under the earlier qualification decision. These results justify fixing model-independent composition; they do not justify a warm 2B/4B router, changing the working backend, or automatically downloading models. Long scans were dominated by extraction in the Windows measurements, so OCR improvements need separate end-to-end timing and quality gates.

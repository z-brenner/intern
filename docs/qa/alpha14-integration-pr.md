# Fix date and organization validation and prepare alpha.14

Intern could mark a report ready under its signer's name or choose a signature,
end date or referenced-agreement date despite stronger document evidence.
Release evaluation also did not enforce the same confidence policy as the app.

This change prefers explicit document dates, distinguishes report staff from
one supported letterhead organization, removes copied trailing date fields
from document types, and avoids needless review when an omitted optional
subject has a uniquely grounded acronym and checked action wording. Missing
or ambiguous ownership, unsupported claims and probabilities below 0.7 still
require review. A model that cannot fit measured available memory waits,
without poisoning verified assets.

Evaluation requests and validates returned critical-value confidence, enforces
the shipping threshold, preserves rejected-run recordings and can compare one
fresh recording against a pinned validator. An ASCII normalization path keeps
matching behavior while reducing replay validation work. It is not claimed as
an inference speedup. The historical benchmark baseline records the corrected
date cases; gold, tolerances and live acceptance limits are unchanged.

All alpha.14 version surfaces and release notes are prepared, with fidelity
and the checklist returned to pending. Model weights, runtime pins, updater
address/signing key, memory floors and mandatory review for the fast tier are
unchanged. Prior alpha.13 fidelity provenance is retained as history; nobody is
named as having reviewed this release's capture.

## Validation

Application-source Windows QA
[41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658)
passed frontend/browser, full Rust and Clippy, parser/OCR/runtime, installer
build and smoke, live acceptance and the complete-recording pair. Its live
19-document result is ten ready, nine reviewed; the pair changes only SOW
readiness with unchanged emitted fields and confidence. The annual report is
covered by guarded tests and the separate PC comparison, not this small live
fixture set.

The PC worker reports that replaying both actual matched 77-document responses
through the same validator at 0.7 gives 9B 73 accepted names, 77 correct dates
and parties, zero strict wrong-ready; 2B remains 67 names, 76 dates, 75 parties
and one wrong-ready. Prompts, raw proposals and probabilities match the original
recordings and no record is stale. This is reported replay evidence, not a new
inference or laptop performance measurement. Original constrained-PC medians
remain 55.77 seconds versus 13.97 seconds, a roughly fourfold latency cost.

Local alpha.14 checks pass: 765 frontend tests in 78 files, types and build;
1,507 portable Rust tests with zero failures and six ignored; strict portable
Clippy, formatting and optimized worker/evaluator/benchmark builds. Version
contracts pass. Fresh preparation-tree Windows QA is running at
[41975965963](https://github.com/z-brenner/intern/actions/runs/41975965963).
Authorized integration, exact-main QA, truthful
capture-specific fidelity review and release preflight. No release is claimed.

## Submission state

This body is prepared locally for review. It has not been submitted as a PR.
The earlier draft-PR submission to z-brenner/intern was denied by automatic
approval review. Available room history confirms that denial but does not
supply its stated reason; no attempt is made to bypass or retry it here.

# Report ownership after a wrapped signature title

This is a validation fix on the isolated `qualification/post-alpha13` branch.
It does not qualify a new model default or release. App fix `234fb6b81d25036afd7303d54ecb7b136239ac6d` has release-input digest
`4bf697c02e68553462f4a5987498a05325968604692118fc1ffc9f73570daade`. Hashes, checks and replay receipts are recorded in the companion JSON.

## Evidence and cause

The parent supplied a completed matched 77-document PC comparison under a
12 GiB memory cap, four CPU cores and no swap. Reported 2B results were 67
accepted filenames, 76 correct dates, one strict wrong-ready and a 13.97-second
median. Reported 9B results were 69 accepted filenames, 74 correct dates,
three strict wrong-ready and a 55.77-second median. The parent verified
byte-identical requests. These are constrained desktop observations, not a
16 GB laptop qualification, and the original PC artifacts have not been
independently inspected on this executor.

The parent then replayed the 9B responses with the date fixes: 72 filenames,
77 dates and one remaining wrong-ready. Its annual report names CEO Casimir
Ellingsen instead of Rookwood Precision Metals Corporation despite minimum
probability 0.792065. The benchmark source has the CEO's standalone name,
`President and Chief Executive Officer`, and the date on successive lines
of one evidence unit. Staff detection already handles inline posts and
labeled signature fields but missed this unlabelled wrapped title. A high
probability cannot establish the person's documentary role.

## Bounded correction

Only record-class documents gain the wrapped-title rule. The name must occupy
its own line and the immediately following line in the same shown evidence
unit must contain only job words and checked modifiers. There is no inference
across pages or adjacent retrieved passages. Explicit subject or recipient
labels keep precedence; a sentence about another president does not count.

When only staff were proposed, recovery requires one organization actually
presented as a name in the first-page letterhead. Plain headings, document-kind
suffixes, heading metadata after a dash and table padding are supported.
Organizations mentioned only in body prose cannot become owners. Multiple
header organizations, including a shared heading, do not resolve ownership;
missing or ambiguous ownership requires `PartyMissing` review. Other document
classes retain their existing recovery. No personal or company name is encoded
in the rule.

Broad tests found and corrected two candidate regressions before publication:
aircraft provider recovery initially lost its letterhead when a table also
named the aircraft owner, and a later stricter check missed table padding in
a vineyard's harvest-log heading. Existing aircraft and museum tests and a
new spreadsheet regression preserve these cases. Gold, baselines, confidence
thresholds and acceptance limits are unchanged.

## Paired experiment and limits

The complete matched PC recording is not present locally. The executed
experiment is explicitly a controlled substitution in the existing 77-document
9B recording: only the annual report's proposed party and minimum probability
are changed to the parent-reported CEO case. Historical source bytes,
extraction, prompts and other replies are unchanged. This is not a full replay
of the captured PC responses.

The frozen control reproduces a ready filename naming the CEO. The corrected
validator emits the company instead at the identical probability, improves
this constructed suite from 72 to 73 accepted filenames and from 76 to 77
correct parties, and removes its sole strict wrong-ready and forbidden party.
Only that document's emitted fields and scores change. Both arms have zero
stale prompts. A separate constructed minimum probability of 0.699 still
requires `LOW_CONFIDENCE` review after the ownership correction.

Original historical 9B and 2B recordings and both fixture pipelines pass
their unchanged baselines and retain identical semantic outputs against the
preceding version. The historical 9B remains 73/77 filenames, 77/77 dates and
parties, zero strict wrong-ready; historical 2B remains 71/77 filenames,
76/77 dates, 75/77 parties and its existing one strict wrong-ready.
Their replay timings and memory fields are inherited; they do not measure
new inference performance. The original 2B recording's existing wrong-ready
and the fixture pipelines' existing errors remain part of the evidence.

The final candidate passes formatting, strict portable workspace Clippy and
release compilation. The full portable Rust suite passes **1,507 tests, zero failed and six ignored**
in a serial run bounded to five minutes: an earlier concurrent attempt stalled in the existing
`default_policy_cancels_header_acquisition_and_joins_network_work` test and was
stopped without changing the downloader or test. That same test passes in the
serial rerun. The interrupted attempt remains recorded, not counted as a pass.
Local full desktop compilation remains unavailable because GTK development
dependencies are absent; Windows QA supplies the native platform checks.

## Default and release implications

The reported matched PC median makes 9B about four times slower than 2B.
The shared date and ownership fixes must be applied to both actual matched
recordings before attributing quality gains to 9B or choosing a default.
A controlled correction of this known case alone cannot establish safe
replacement, generalization, throughput or 16 GB laptop suitability.

The earlier Windows run 41964133224 is preserved. It evaluates the preceding
source, so even a pass cannot qualify this ownership fix. Fresh Windows QA,
the complete matched PC replay for both models, held-out cases, laptop memory
and cold/warm performance, and applicable fidelity review remain necessary.
No PR approval, merge, default-model change or release was performed.

Fresh matching Windows QA is running at
[41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658),
head `a3910ba22e313d5862642748ab8342c4c807a24c`. The earlier run remains active.
This docs-only receipt retains the same release-input digest; neither run's
live acceptance or paired replay has been claimed complete.

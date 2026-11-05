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

## Matched PC replay reported after the controlled experiment

The PC worker subsequently completed both original matched recordings through
`a14b4ef187ab81a931abd58d589d3c150861171c` at the unchanged 0.7 gate. The parent
reports zero stale records and matching prompt hashes, raw proposals and
confidence values against the original recordings:

| Model | Accepted filenames | Correct dates | Correct parties | Strict wrong-ready |
| --- | --- | --- | --- | --- |
| 2B, original → patched | 67 → 67 / 77 | 76 → 76 / 77 | 75 → 75 / 77 | 1 → 1 |
| 9B, original → patched | 69 → 73 / 77 | 74 → 77 / 77 | 76 → 77 / 77 | 3 → 0 |

This is reported full matched-recording evidence, distinct from the controlled
experiment executed here. Reports and verification remain on the PC and have
not been independently inspected on this executor. It is replay of existing
responses, without new inference or fresh latency/memory measurements. Review
rate was not supplied and is not inferred.

## Default and release implications

Applying the same validator to both actual matched recordings now supports
an accuracy advantage for 9B on this set: six more accepted filenames, all
77 dates and parties correct, and zero strict wrong-ready against 2B's one.
The rules encode neither the company nor the CEO's name. The guarded tests
and unchanged historical replays provide additional regression checks, but
this known corpus alone cannot establish generalization.

The original constrained PC inference median remains 55.77 seconds for 9B
against 13.97 seconds for 2B, about four times slower. These timings are
inherited from the original inference experiment, not remeasured after the
validation fixes. The 12 GiB cap and no-swap run support constrained-desktop
feasibility, without establishing actual 16 GB laptop latency, headroom with
other applications or battery behavior. Existing model selection, memory
floors and mandatory review for the fast tier are unchanged.

Fresh matching Windows QA is running at
[41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658),
head `a3910ba22e313d5862642748ab8342c4c807a24c`. At the 05:11 UTC checkpoint,
frontend checks, browser QA, formatting, full Windows workspace strict Clippy,
full Windows workspace tests, verified native assets, parser integration, staged
runtime and PP-OCR checks passed. The installer is building; its smoke test,
live acceptance, paired replay and final evidence binding remain pending.
The earlier [41964133224](https://github.com/z-brenner/intern/actions/runs/41964133224)
is preserved and still in live inference; it covers the preceding source.

The read-only release preflight exits 1 because the existing fidelity sign-off
names the old release-input digest `40df3381aac1a22c7cbfcd50944dc4f5c4dd629ba710e03eaef9f6d0a3864c32`,
while this source has `4bf697c02e68553462f4a5987498a05325968604692118fc1ffc9f73570daade`.
No sign-off fields were edited. [The reviewer rule](../releasing.md#the-reviewer-rule)
requires a named person to accept the exact capture; an assistant cannot invent
that attestation. A new release also needs its own aligned version and notes.

Windows acceptance and the capture-specific preflight are still pending, so
the branch remains unqualified for release. No PR approval, merge,
default-model change or release was performed. These docs-only receipts retain
the same release-input digest and do not restart either QA run.

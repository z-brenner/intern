# Model candidates after the bake-off

[The bake-off](model-bakeoff.md) chose Qwen3.5-2B Q4_K_M behind a GBNF grammar.
This page covers what is worth trying next, what has already been built, and how
each candidate has to be measured before it ships. The numbers here come from the
corpus and from one live run. Nothing is taken from published benchmarks.

## Where the idea came from

Jev (TypeSafe AI, September 2026) is a closed, API-only "decision" model. It is
not autoregressive: you give it a schema of choices and it returns typed values,
each with a calibrated probability. Intern cannot use it, because documents never
leave the machine by default and there are no weights. Two of its ideas do
carry over:

1. **Schema-constrained output.** Intern has had this from the start.
   `prompt::RESPONSE_GRAMMAR` is a GBNF grammar sent as `grammar` on every
   llama-server request. It fixes the field order (evidence before conclusion),
   the date shape, the `date_role` and `party_relation` vocabularies, and the
   three-party cap. A JSON-schema `response_format` would compile down to the
   same sampler constraint and adds nothing over it.
2. **A confidence signal from the model's own probabilities.** This was new. It
   is built and described below.

## 1. Token confidence (built, off by default)

**What it is.** `ModelClient::with_token_confidence(true)` adds `"logprobs": true,
"top_logprobs": 1` to the request. `client::token_confidence` reads the reported
probability of every generated token and keeps only the tokens that overlap the
characters of the `document_date` value and the `parties` values: the inside of
each string, or a bare `null` / `[]`. The analysis then carries
`tokenConfidence: {min, mean, tokens}`.

The scaffolding is skipped on purpose. llama.cpp reports the model's probability
*before* the grammar mask is applied, so a quote mark the grammar forces can come
back at e^-14. That is noise. A date digit the model would rather not have
written is the signal.

**What it does not change.**

* **The recording.** `ModelRequest::sha256` hashes the user turn only. The
  grammar, sampling settings, and the new fields are transport, so the committed
  `fixtures/corpus-recording.json` still replays with 0 regressions and no
  `stale_prompt`.
* **The reply.** Decoding is greedy. Two live runs on a freshly started server,
  one without logprobs and one with, gave byte-identical proposals for all 21
  fixtures.
* **Readiness.** `Engine::with_min_token_confidence(t)` sends a proposal whose
  `min` is below `t` to review with `LOW_CONFIDENCE`. Nothing sets it. The app
  constructs `ModelClient` without token confidence, and `validate.rs` is
  untouched and remains the grounding backstop.

**Live result** (this machine: Ryzen 7 PRO 8840U, llama.cpp b10361 Windows CPU,
8 threads, 8,192 context, packaged runtime). Recorded with `--token-confidence`
to `fixtures/corpus-recording-confidence.json`, through the digest pipeline,
the default then: replay it with `--pipeline digest`.

| Fixture | `min` | Date | Parties | Routed |
| --- | ---: | :---: | :---: | --- |
| scanned-lease.pdf (OCR) | 0.011 | wrong | wrong | review |
| document-image.png (OCR) | 0.395 | wrong | ok | review |
| document-image.tiff (OCR) | 0.484 | wrong | ok | review |
| termination-notice.pdf | 0.502 | ok | ok | ready |
| board-deck.pptx | 0.642 | ok | **wrong** | **ready** |
| nda.docx | 0.660 | ok | ok | ready |
| every other fixture | ≥ 0.672 | | | |

The three lowest scores are exactly the three OCR misreads, but validation
already routes all three to review. Replaying that recording with a threshold:

| `--min-token-confidence` | readiness_match | Review rate | Effect |
| --- | ---: | ---: | --- |
| unset | 18/19 | 42% | as shipped |
| 0.5 | 18/19 | 42% | no change: everything below 0.5 was already in review |
| 0.7 | 15/19 | 58% | catches board-deck's wrong party, but also sends nda and termination-notice to review while they are correct |

**Verdict.** The signal is real and it ranks the misreads lowest. On 19 documents
it does not catch anything the verbatim checks miss without also costing correct
ready files, so it stays data only. Calibrate it again when the corpus has more
"ready but wrong" cases.

**Latency.** Not separable from noise in this run. Mean inference was 20.7 s
without logprobs (first run, cold), 17.2 s with them, and 15.1 s for a repeat
without them on a warm server. llama.cpp sorts the whole vocabulary once per
generated token to report a probability, so expect a cost of a few percent. That
cost should be measured with `llama-bench`-style repeated runs before the flag is
turned on in the app.

**Reproduce.**

```powershell
npm run fixtures
cargo build --locked -p intern-worker --release --features windows-native
cargo build --locked -p intern-engine --bin intern-evaluate
# llama-server with the app's flags (see scripts/record-corpus.ps1), then:
$env:INTERN_RUNTIME_DIR = "$env:LOCALAPPDATA\Intern"
target\debug\intern-evaluate.exe --fixtures fixtures\generated --expected fixtures\expected.json `
  --worker target\release\intern-worker.exe --endpoint http://127.0.0.1:8090/v1/chat/completions `
  --api-key KEY --model-id intern-local --pipeline digest --token-confidence `
  --record fixtures\corpus-recording-confidence.json --output report-confidence.json
# Calibrate offline, no model needed:
target\debug\intern-evaluate.exe --fixtures fixtures\generated --expected fixtures\expected.json `
  --pipeline digest --replay fixtures\corpus-recording-confidence.json --min-token-confidence 0.5
```

The confidence recording is not the recording of record. Use it to calibrate the
threshold, not as the baseline gate. Replayed against
`fixtures/corpus-baseline-digest.json`, which was earned by the Linux recording, it
differs on three scores (nda description facts, ambiguous-note description
specificity, board-deck parties) and improves one (document-image.png parties).
Those differences come from a different machine and OCR build, not from logprobs.

## 2. NuExtract3 as the GGUF (not built)

| | |
| --- | --- |
| Model | `numind/NuExtract3-GGUF`, a 4B fine-tune of Qwen3.5-VL for template extraction |
| License | Apache-2.0 |
| File | about 2.8 GB at Q4_K_M, against 1.18 GiB for the incumbent |
| RAM | expect about 3.5 to 4 GB resident at 8,192 context, about twice the incumbent |
| Speed | a 4B Qwen3.5 measured 48 tok/s prefill and 7.3 tok/s generation here, about 3× slower than the 2B on prefill |

**How it plugs in.** Change the manifest and the template. No new runtime is
needed. NuExtract takes a JSON template with typed slots (`verbatim-string`,
`date-time`, arrays) instead of free instructions. That maps onto Intern's
fields, and its verbatim-string type is the same contract `validate.rs`
enforces. It needs its own prompt builder (a new `prompt` variant, so a new
recording), an adapter from its output to `WireProposal`, and a decision on
whether to keep the GBNF grammar on top of it. Keep the grammar: it is still the
only thing that removes the `due` role.

**Risk.** It is the same size class the bake-off already rejected on speed. It
has to win on accuracy by a clear margin to be worth twice the download and
memory.

## 3. GLiNER2 as a candidate pre-extractor (not built)

| | |
| --- | --- |
| Model | `fastino/gliner2-base-v1`, about 200M parameters, span extraction against labels given at run time |
| License | Apache-2.0 |
| Size | about 0.8 GB fp32, smaller once quantised or exported to ONNX |
| Runtime | not llama.cpp. It needs ONNX Runtime (the `ort` crate) or a Python sidecar, so it is a new pinned runtime asset |

**How it plugs in.** It runs between distill and prompt. GLiNER2 proposes
candidate spans for `party`, `organization`, `person`, and `date` from the
digest. The prompt then lists them as choices ("parties: choose from ..."), and
the grammar can restrict `parties` to those strings, which turns the party
decision into a selection, the way Jev frames it. Every candidate is a literal
span, so it passes the verbatim check by construction.

**Measure it offline first.** The model alone can be scored for *candidate
recall* without touching the LLM. For every text fixture, run GLiNER2 over the
digest that `intern_engine::distill` produces from the recorded extraction in
`fixtures/corpus-recording-digest.json`. Then count how often every reviewed party and
the reviewed date in `fixtures/expected.json` appear among the top-k candidates.
If recall is below 100% on the text fixtures, the pre-extractor would cap
accuracy, and there is no point wiring it in.

### GLiNER2 candidate recall, measured 2026-10-01

**Verdict: don't build it now. The corpus is too small and too easy to justify
it.** On every value that is literally present in the text, GLiNER2 base found
the reviewed answer: 23 of 23 party names and 14 of 14 dates. But the incumbent
already gets every one of those fixtures right, so the corpus leaves the
pre-extractor nothing to fix. All six misses are OCR-garbled fixtures where the
reviewed value is not in the text at all. No span extractor can return those
values, and the incumbent fails them too. Look at this again once the corpus has
documents where the incumbent picks the wrong party from text that contains the
right one.

| `fastino/gliner2-base-v1`, threshold 0.5 (library default) | Parties | Dates |
| --- | --- | --- |
| Recall over all labelled values | 92.0% (23/25 names; 14/16 docs had every party) | 77.8% (14/18 docs) |
| Recall over values literally in the text | **100% (23/23)** | **100% (14/14)** |
| Candidates per document, mean / max | 5.8 / 27 | 2.0 / 7 |
| Same at threshold 0.3 | recall unchanged, 8.2 / 54 | unchanged |
| Same at threshold 0.1 | recall unchanged, 14.9 / 88 | unchanged |
| Alternate labels, threshold 0.5 | recall unchanged, 3.3 / 16 | unchanged |

*Recall over all labelled values* uses the evaluator's lenient
`party_matches`: punctuation-insensitive containment either way. Strict recall
needs the full normalised name, and it came out identical, so the hits are
whole names, not fragments. A date candidate is a GLiNER2 `date` span that
dateutil parses to a full day. It is a hit when it equals `document_date` or
one of the `acceptable_dates`. A value counts as *in the text* when the
normalised party name is a substring of the normalised text, or when the date
is written out in full (month name, ISO, or `m/d/yyyy`) somewhere in the text.

**Misses (identical for both label sets at every threshold):**

| Fixture | Field | Why |
| --- | --- | --- |
| `scanned-lease.pdf` | party | OCR reads `LEDAR FINCH PROPERTIES LLC`. GLiNER2 returned that span, but it is not `Cedar Finch Properties LLC` |
| `scanned-lease.pdf` | date | OCR reads `SEPTEMBER 1 24h24`. GLiNER2 returned `SEPTEMBER 1`, which has no year |
| `rotated-low-resolution-scan.png` | party | OCR reads `PINE ECHO COURTERS LLC` |
| `rotated-low-resolution-scan.png` | date | OCR reads `JUNE 12 2a25`. The span returned was `JUNE 12` |
| `document-image.png` | date | OCR reads `JULY 14 2625`, which parses to the wrong year |
| `document-image.tiff` | date | OCR reads `JULY 16 26275` |

The incumbent's baseline (`fixtures/corpus-baseline.json`) has
`parties_correct` false on the first two fixtures and `date_correct` false on
all four. GLiNER2 matches the incumbent fixture for fixture: it gets nothing
the incumbent misses and misses nothing the incumbent gets.

**What the numbers do not show.**

* **The sample is small.** There are 19 documents with text and labels. Only 23
  party names and 14 dates are actually present in the text. One miss would
  move party recall by about 4 points and date recall by about 7. A perfect 23/23
  only bounds true recall above roughly 87% (rule of three, 95% confidence), and
  14/14 only above roughly 79%. That means this run cannot confirm the 95% gate.
  It can only fail to reject it.
* **Candidate sets on long documents are large.** The three long fixtures carry
  23 to 27 organisation and person spans at the default threshold. Most of them
  are clause boilerplate and people mentioned in passing. `ambiguous-note.pdf`
  has no reviewed parties but still gets 4 candidates. A choose-from-list prompt
  would hand the model those distractors.
* **Dates gain nothing over a regex.** The full-date regex used for the
  "in the text" oracle finds the same 14 dates. GLiNER2's date spans bring no
  information that a deterministic scan of the text does not already have.
* **It ran on the raw text, not the digest.** The run used the page text the
  engine sees before distillation, not the `distill` digest this section's
  protocol names. That is an upper bound on recall. On long documents the digest
  would cut candidate counts, and it could also drop a party.
* **The cost is not small.** The model is 0.83 GB of safetensors. Peak resident
  memory was about 2.0 GB under PyTorch. ONNX would be lower but was not
  measured. On this CPU the median document took 0.26 s. The longest, the
  29,311-character `statement-of-work.pdf`, took 14.3 s, and model load took
  about 6 s warm (42 s on first download). Shipping it also means a new runtime
  (ONNX Runtime or a Python sidecar).
* **`fastino/gliner2-large-v1` was not run.** It is 1.86 GiB, over the size
  budget for this check, and the base model missed nothing a span extractor
  could reach.

**Setup.** Labels were fixed before the first run. The primary set was
`organization`, `person`, `date`, the plain types this section names. The
alternate set, `company`, `person`, `contracting party`, `date`,
`document date`, `effective date`, was run as a sensitivity check. It found the
same values with fewer party candidates (3.3 mean, 16 max), so label phrasing
mostly changes how much noise comes back. Inference used
`extract_entities_long` with 256-word windows and a 64-word overlap. Spans were
extracted once at threshold 0.1 and then filtered offline at 0.3 and 0.5.

Versions: gliner2 2.0.0, torch 2.14.1+cpu, Python 3.12.10, Windows 11, model
revision `f9634218e535`.

```powershell
python -m venv $env:TEMP\g2venv
& $env:TEMP\g2venv\Scripts\python -m pip install torch --index-url https://download.pytorch.org/whl/cpu
& $env:TEMP\g2venv\Scripts\python -m pip install "gliner2[local]" python-dateutil psutil
& $env:TEMP\g2venv\Scripts\python scripts\gliner2-candidate-recall.py --labels primary --out gliner2-primary.json
& $env:TEMP\g2venv\Scripts\python scripts\gliner2-candidate-recall.py --labels alternate --out gliner2-alternate.json
```

`pip install gliner2` on its own installs only the API client. The local model
needs the `[local]` extra and a torch wheel.

No other labelled text corpus is committed. `docs/qa/model-evaluation.json`
has labels and proposals but no document text, and
`fixtures/corpus-recording-confidence.json` holds the same 21 fixtures
recorded on another machine.

## Evaluation protocol for any candidate

Every candidate is held to the same procedure:

1. **Replay first.** `npm run fixtures`, then replay
   `fixtures/corpus-recording.json` against `fixtures/corpus-baseline.json`. It
   must show 0 regressions and no `stale_prompt` before anything live is run.
   This proves the engine around the model is unchanged.
2. **Same-machine control.** Record the incumbent live on the candidate's
   machine, with `--record` to a new file, never the committed one. A recording
   from another machine is not a control: this page's own run differs on three
   scores from the Linux recording with the same model and flags.
3. **Candidate run.** Record the candidate on the same machine with the same
   flags, to its own file, for example `fixtures/corpus-recording-nuextract.json`.
4. **Compare.** Write a baseline from the control recording, then replay the
   candidate recording against it with `--baseline`. This lists every
   per-fixture regression and improvement.
5. **Repeat once.** Run each arm twice. A second arm on a warm server drifted on
   7 of 21 proposals in this run (descriptions, evidence quotes), so a
   one-fixture difference is noise until it reproduces.

**Acceptance.** A candidate replaces the incumbent only if all of these hold:

* zero `date_forbidden` and `party_forbidden`,
* no regression in `date_correct` or `parties_correct` against the control,
* a strict gain on at least one of dates, parties, or `readiness_match`,
* median inference no more than 1.5× the control,
* peak memory that still fits the 8 GB laptops the app targets.

If it passes, the change ships with a re-recorded `fixtures/corpus-recording.json`
and `fixtures/corpus-baseline.json` made on the packaged Windows runtime
(`scripts/record-corpus.ps1`), and with a new row in
[the bake-off](model-bakeoff.md).
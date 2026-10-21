# Model and pipeline bake-off

Every number here was measured on one machine with real local inference over the
whole generated corpus. Nothing is extrapolated from published benchmarks.

## Bonsai 2: the production model

Intern's local model is now Prism ML's **Ternary Bonsai 2 4B** (PQ2_0,
`Ternary-Bonsai-2-4B-PQ2_0.gguf`, 1,146,465,120 bytes, 1.07 GiB), with
**Ternary Bonsai 2 2B** (PQ2_0, 517,312,992 bytes, 493 MiB) as an opt-in for
machines short of memory. It replaces unsloth's Qwen3.5-2B Q4_K_M, the model
this bake-off chose; everything below this section is that earlier,
historical measurement and is kept as it was.

**What changed.** `model-manifest.json` pins both variants (schema 3) by an
immutable Hugging Face revision, size and SHA-256; only the one a person
chooses is downloaded, one is resident at a time, and switching unloads the
running model before loading the other. PQ2_0 needs PrismML's llama.cpp fork,
pinned at `prism-b10777-e3de715`. The local evidence pipeline sends the
request profile the evaluation selected: one fixed worked example (a
fictional dispatch note) before the document, greedy decoding, seed 17,
thinking off. Existing installations keep their Qwen file until the person
removes it with **Remove the previous model (1.19 GiB)**; an update asks for
the new download first.

**Why.** On the [native Windows evaluation](bonsai-native-windows-evaluation.md)
- twelve synthetic held-out documents, a 4 GiB / 2-vCPU Windows guest, one
pass - Bonsai 4B named 11 of 12 strict filenames correctly against Qwen's 9
and Bonsai 2B's 8, with one wrong-but-ready filename against Qwen's two, and
resolved the amendment's last-signature date Qwen missed. Its peak model
memory was 1.96 GB against Qwen's 2.21 GB; 2B's was 0.94 GB.

**What it costs, and what it does not fix.** Read these before relying on it:

- **Latency.** Bonsai 4B's median document took 54.1 s against Qwen's 26.2 s
  on that guest - about twice as long - and its summed inference 2.2 times
  Qwen's. Bonsai 2B was the fastest (22.5 s median) but named fewer
  documents correctly. The 25-page scan is OCR-bound (about 125 s of its
  143-170 s) whichever model reads it.
- **Semantic failures remain** ([4B audit](../bench/reports/bonsai-native-windows-2026-10-18/bonsai4-semantic-audit.md),
  [2B audit](../bench/reports/bonsai-native-windows-2026-10-18/bonsai2-semantic-audit.md)):
  4B filed an annual report "to Fellow Shareholders" without its issuer and
  marked it ready; it described a capital call as going to the general
  partner that issued it (Qwen did too), reversed supplier and distributor
  into buyer and seller in its raw facts, invented a recipient for a meeting
  notice (sent to review), named an invoice's subject from a table header,
  and mislabelled several date roles. 2B omitted a meeting notice's issuer
  and marked it ready, and preferred an underlying agreement's date to the
  controlling signature date. Shared engine composition still truncates
  "square feet" and overstates conditional effective dates. The review flow
  stays as it is, and nothing routes between the two models.
- **As recorded, Intern's own corpus and InternBench scored 4B below Qwen on
  filenames and dates** (tables below); deterministic checks added since
  close most of that gap and leave 4B with fewer trap dates and
  wrong-but-ready names than Qwen had ("Deterministic checks" below). On the corpus, recorded live on Linux
  (below), Bonsai 4B named 9 of 18 corpus fixtures exactly right against
  Qwen's 13, and 16 of 19 party sets against 18. Its losses: a type that
  absorbs the effective date ("Lease Agreement Effective September 1 2024",
  ready), an invented sender for meeting minutes ("The Fictional Meridian
  Committee", ready), a spurious party on a 100-page journal ("Fictional
  observation 001", ready), and two OCR'd scans whose types copy the OCR's
  casing or an identifier (both sent to review, as before). It newly names
  the multi-date invoice correctly instead of sending it to review. Dates,
  types and the absence of trap dates and parties are unchanged, and every
  document it named carried the right date.
- **Not yet measured on Windows.** The corpus and InternBench recordings of
  record were made on Linux x86-64 with PrismML's ubuntu-x64 build, 2
  threads. The Windows recording on the packaged runtime, Windows memory and
  latency on a real laptop, and a clean-VM check of the VC++ runtime the fork
  (like upstream) needs are follow-ups. The release gate's floors were set
  from the Linux run (`scripts/validate-model-evaluation.mjs`).

### Corpus, Qwen3.5-2B and Bonsai 4B

Both replayed through the same engine; the Qwen recording is the one
committed before the switch (Linux, 4 threads), the Bonsai one
`fixtures/corpus-recording.json` (Linux, 2 threads). The worker read every
fixture identically for both, so every difference is the model's and its
request profile's.

| Score | Qwen3.5-2B | Bonsai 4B |
| --- | ---: | ---: |
| Filename exactly right | 13/18 | 9/18 |
| Date right | 16/18 | 16/18 |
| Type right | 17/18 | 17/18 |
| Parties right | 18/19 | 16/19 |
| Description specific | 17/19 | 18/19 |
| Routing (ready vs review) right | 16/19 | 17/19 |
| Named (ready) | 13/19 | 14/19 |
| Trap date or party | 0 | 0 |
| Named with the wrong date | 0 | 0 |

### InternBench, Qwen3.5-2B and Bonsai 4B

All 77 documents, `bench/recording.json` against the Qwen recording it
replaced; the worker read all 77 identically for both. Rates over the
documents each score applies to.

| Score | Qwen3.5-2B | Bonsai 4B |
| --- | ---: | ---: |
| Filename exactly right | 70.1% | 64.9% |
| Date right | 97.4% | 90.9% |
| Date role right | 56.8% | 70.0% |
| Type right | 93.5% | 90.9% |
| Parties right | 87.0% | 90.9% |
| Party roles right | 87.7% | 86.8% |
| Documents naming a forbidden party | 8 | 1 |
| Trap dates filed | 1 | 6 |
| Wrong filename marked ready ("unsafe ready") | 15 | 21 |
| Needless review | 3.8% | 9.4% |
| Routing right | 77.2% | 79.0% |
| Description specific | 96.1% | 100% |

Bonsai 4B names parties better and almost never a forbidden one, but it
files more documents under a trap date (a due, deadline or underlying
agreement's date: `sow-under-msa-5p`, `demand-letter`,
`scan-patient-intake-form`, `benefits-change-checkbox-form`,
`industrial-lease-dated-in-schedule-25p` among them) and marks more wrong
filenames ready. Thirty documents lost at least one reviewed score and
85 scores improved; the commit that records it lists them. Its median
document took 65.5 s here (2 threads, Linux, on a machine shared with other
work), so do not read it as a laptop figure.

### Deterministic checks added for what Bonsai got wrong

The tables above are the replies as recorded. Each of Bonsai 4B's trap
dates and wrong-but-ready filenames was traced to a check the engine
should have made and did not; these were added to validation (`facts.rs`,
`infer.rs`), not to the prompt, which stays byte-identical to the evaluated
profile. Each rule reads the document's own text and names no fixture.
Measured by replaying the same recordings - Bonsai's and, through
`--allow-stale --allow-other-model`, Qwen's - before and after:

| Rule | Documents it changes |
| --- | --- |
| "No later than" / "not later than" before a date makes it a deadline | `demand-letter` (Bonsai): trap date 2026-07-22 withheld, sent to review |
| A date in a table cell whose column (or row) label names another event - coverage, a requested effective date, birth, hire, due, delivery - is not the document's; the one date a column names as its own ("Date signed", "Date") replaces it, else it is withheld. New reason `DATE_IS_OTHER_KIND` | `scan-patient-intake-form` (Bonsai): trap 2026-01-01 withheld, review. `benefits-change-checkbox-form` (Bonsai): trap 2026-07-01 replaced by the signed date 2026-06-29, review. An earlier draft that treated any non-issue label as "other" also hit `purchase-order` ("PO Date") and `field-condition-report` ("Date of examination") in both models; it now requires a label naming another event, and neither changes |
| A date stated only in signature blocks, or cited only from them, is a signing date: it gives way to a single stated effective date, else to a single "Date of this ..." date; an agreement or amendment's earlier signature gives way to the latest | `sow-under-msa-5p` (Bonsai): 2026-06-01 -> 2026-05-26, now right. `industrial-lease-dated-in-schedule-25p` (Bonsai): 2026-04-16 -> 2026-04-14, right. `mixed-signature-region` (Bonsai and Qwen): 2026-08-18 -> 2026-08-21, right for Bonsai; Qwen's date is fixed, its parties are still wrong |
| A type loses a date phrase ("Effective September 1 2024", a leading "2025") or a trailing document number ("PS-311", "DR-771", "#10422") written into it, when a kind is left | corpus `scanned-lease.pdf`, `rotated-low-resolution-scan.png`, `document-image.jpg` (Bonsai): filenames now right. `annual-report-excerpt-8p`: "2025 Annual Report" -> "Annual Report" |
| A bare "Letter", "Document" or "Agreement" is no type ("Never ... alone"): review (`TYPE_MISSING`) | `engagement-letter` (Bonsai): wrong ready name -> review. `agreement-two-column-footnotes` (Qwen): wrong ready name -> review |
| An audience or a body - "Fellow Shareholders", "Board of Directors", "Human Resources" - is no party: dropped, review (`PARTY_UNSUPPORTED`) | `annual-report-excerpt-8p` (Bonsai): issuer now Rookwood Precision Metals Corporation and the filename right, but sent to review (a needless review). `board-resolution` (Bonsai): wrong ready name -> review. `benefits-change-checkbox-form`: party now Annika Solberg-Reyes, filename right, review (its date rule sends it there anyway) |
| A type ending "of [the] <filename party>" drops it | `newsletter-three-column` (Bonsai): "Newsletter of the ... Association for ... Association" -> "Newsletter for ... Association"; still not the gold's "from" |

After them, replaying the same recordings:

| Score | Qwen3.5-2B before | Qwen3.5-2B after | Bonsai 4B before | Bonsai 4B after |
| --- | ---: | ---: | ---: | ---: |
| InternBench filename right | 54/77 | 54/77 | 50/77 | 55/77 |
| InternBench date right | 75/77 | 76/77 | 70/77 | 74/77 |
| InternBench trap dates filed | 1 | 0 | 6 | 0 |
| InternBench wrong filename marked ready | 15 | 14 | 21 | 12 |
| InternBench needless review | 2 | 2 | 5 | 7 |
| InternBench routing right | 44/57 | 43/57 | 45/57 | 40/57 |
| Corpus filename right | 13/18 | 13/18 | 9/18 | 12/18 |

Routing falls because five documents whose filenames are wrong, or
dated by a withheld trap, now go to review where the gold expected a
ready one, and two right filenames go to review with them (the needless
reviews above). The digest pipeline's replay is unchanged. Still wrong
and ready for Bonsai: role confusions the text cannot settle on its own
(a vendor form for the county that issues it, a loss notice from the
insurer, a certificate for its broker), a product name in a type, a
committee and a journal entry read as parties, and the generic-audience
and relation cases above.

### The recordings of record: Bonsai 4B read through Indra OCR

Everything above was measured over the routing worker's PP-OCR text. The
committed recordings (`fixtures/corpus-recording.json`, `bench/recording.json`)
were then made again, live, with the worker that ships - Indra OCR first,
PP-OCR re-reading any page under 75 confidence, Tesseract last - and the
same model, runtime, request profile and rules, on the same Linux machine.
Indra changed the text of 6 corpus fixtures and 20 InternBench documents,
all scans; Bonsai's reply to every document whose text did not change was
byte for byte the same, so each difference below is the OCR's.

| Score (rules on) | Corpus, PP-OCR text | Corpus, Indra text | InternBench, PP-OCR text | InternBench, Indra text |
| --- | ---: | ---: | ---: | ---: |
| Filename right | 12/18 | 11/18 | 55/77 | 55/77 |
| Date right | 16/18 | 18/18 | 74/77 | 75/77 |
| Type right | 17/18 | 15/18 | 70/77 | 71/77 |
| Parties right | 16/19 | 15/19 | 73/77 | 74/77 |
| Trap dates filed | 0 | 0 | 0 | 0 |
| Wrong filename marked ready | - | - | 12 | 14 |
| Documents naming a forbidden party | 0 | 0 | 1 | 0 |

What Indra's text changed:

- Better: corpus `document-image.png` and `document-image.tiff` are dated now
  (PP-OCR read `20z5` and lost a line); `document-image.png`'s filename is
  right. InternBench `scan-low-res-receipt`'s type is read, `scan-faint-letter`
  names the right person, and `scan-patient-intake-form` and
  `scan-bill-of-lading` lose an unsupported fact or a wrong date.
- Worse: corpus `scanned-lease.pdf` and `rotated-low-resolution-scan.png`.
  Indra reads that clean-room bitmap font's V as U (`EFFECTIUE`, `DELIUERY`,
  `CEDRR`; `fixtures/README.md`), so the type the model spells correctly is
  not stated in the text and the filename falls back to "Document", in
  review. It is one synthetic font, not a general reading error, so no rule
  reads around it. On InternBench, `scan-faint-letter` now names the right
  person but with "-" rather than "from" and is ready, and
  `scan-patient-intake-form` now carries its right date, so nothing sends it
  to review, but it still names the clinic instead of the patient: the two
  wrong-but-ready additions (12 -> 14). `scan-upside-down-po` (a description
  fact), `scan-skewed-notice` (an unsupported fact) and
  `scan-certificate-of-insurance` (a date role) lose one score each.

The seven rules were checked again on the Indra text, rules off against on:
InternBench trap dates 4 -> 0, wrong-but-ready 21 -> 14, filenames 51 -> 55,
dates 72 -> 75, needless review 5 -> 7, routing 45 -> 40; corpus filenames
10 -> 11. The documents each rule changes are the ones listed above, less
those whose new text no longer needs it.

The release gate's floors now follow this run
(`scripts/validate-model-evaluation.mjs`): date 95%, type 80%, parties 75%,
review at most 45% - type and parties lower than over PP-OCR's text, for
the two scans above.


**Machine:** AMD Ryzen 7 PRO 8840U, 8 cores / 16 threads, 14.7 GB usable RAM, no
discrete GPU, Windows 11, ordinary background applications running.
**Runtime (historical):** llama.cpp `b10361`, CPU only, 8 threads, 8,192-token context.
**Corpus:** `fixtures/generated`, 21 fixtures. Thirteen are text-bearing documents
that reach the model; two are intentionally unreadable (encrypted, malformed);
six require OCR and were not scored on this machine, which has PDFium but no
Tesseract build.

## Raw model speed

`llama-bench`, prompt of 2,048 tokens and 128 generated tokens, 8 threads:

| Model | File size | Prefill (tok/s) | Generation (tok/s) |
| --- | ---: | ---: | ---: |
| **Qwen3.5-2B Q4_K_M** | **1.18 GiB** | **157** | **17.5** |
| Qwen3-1.7B Q4_K_M | 1.03 GiB | 199 | 20.8 |
| Qwen2.5-VL-3B Q4_K_M *(incumbent)* | 1.79 GiB | 75 | 11.3 |
| Qwen3.5-2B Q5_K_M | 1.33 GiB | 78 | 15.9 |
| Qwen3.5-4B Q4_K_M | 2.54 GiB | 48 | 7.3 |

Q5 costs half the prefill speed for no measurable accuracy gain on this corpus
and was dropped. The 4B model is roughly three times slower than the 2B on
prefill and was never a candidate for a laptop that has to stay usable.

## Quality

The bake-off below compares models and pipelines over the same twelve text
documents, which is what makes the rows comparable. The full corpus, including the
six scanned fixtures, is measured separately in
[the section after it](#the-full-corpus). `dates` counts the reviewed answer or a
listed acceptable alternative; `trap dates` counts documents filed under a date the
corpus explicitly marks as wrong (a referenced master agreement's date, a
signature date where an effective date exists, a payment due date).

| Run | Dates | Trap dates | Type | Parties | Description facts | Review rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Qwen2.5-VL-3B, **old pipeline** *(what shipped)* | 9/11 | 1 | 3/11 | 8/12 | 8/11 | 25% |
| Qwen3.5-2B, old pipeline | 9/11 | 2 | 4/11 | 7/12 | 6/11 | 42% |
| Qwen2.5-VL-3B, new pipeline | 9/11 | 2 | 9/11 | 7/12 | 8/11 | 42% |
| Qwen3-1.7B, new pipeline | 11/11 | 0 | 9/11 | 7/12 | 9/11 | 33% |
| **Qwen3.5-2B, new pipeline** *(selected)* | **11/11** | **0** | **8/11** | **9/12** | **9/11** | **25%** |

The two comparisons that matter:

* **Same model, old vs new pipeline** isolates the redesign. Document type goes
  from 4/11 to 8/11 and trap dates from 2 to 0. The old pipeline's filenames
  frequently had no type at all (`2025-02-14.pdf`, `2026-12-29 - notice.pdf`)
  because its schema asked for a "subject" and a separate loosely-defined type,
  and its evidence rule discarded whichever the model could not quote exactly.
* **Same pipeline, old vs new model** isolates the model. Dates go from 9/11 with
  2 traps to 11/11 with none, at roughly half the latency and 40% less peak
  memory.

### Before and after

| Document | Old pipeline | New pipeline |
| --- | --- | --- |
| 14-page statement of work | `2026-04-09 - Statement of Work.pdf` *(signature date)* | `2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf` |
| Termination notice | `2026-12-29 - notice.pdf` | `2026-12-29 Notice of Termination - John Smith.pdf` |
| Consulting amendment | `2025-09-14 - amendment - FIRST AMENDMENT TO CONSULTING AGREEMENT.pdf` | `2025-09-14 First Amendment to Consulting Agreement between Contoso Worldwide, Inc and Jane Ellery.pdf` |
| Vendor invoice | `2026-01-05 - invoice.pdf` | `2026-01-05 Invoice between Contoso Worldwide, Inc and Acme Corporation.pdf` |
| Employment agreement | `2025-02-14.pdf` | `2025-02-14 Employment Agreement between Northstar Lantern Works LLC and Mira Vale.pdf` |
| Order form | `2026-01-14 - Order Form.docx` *(signature date)* | `2026-02-01 Order Form between Tessellate Analytics Ltd and Contoso Worldwide, Inc.docx` |
| Ambiguous note | `Document.pdf`, needs review | `Document between Rowan and Priya.pdf`, needs review |

Every filename above is copied from a scored run, including the two that are not
flattering. The vendor invoice reads `between` its two sides where `from Acme
Corporation` would be right, and it leads with the party that was billed; the
direction is under-determined by the document's own layout and the model does not
recover it. An earlier revision of this table showed the correct-looking
`2026-01-05 Invoice from Acme Corporation.pdf`, which the pipeline does not
produce.

### The full corpus

Eighteen documents scored with real inference, including the six scanned fixtures,
which had never been scored at all because the machine had no Tesseract build:

| | Result |
| --- | ---: |
| Date correct, text documents | 13/13 |
| Date correct, whole corpus | 13/17 |
| Date correct where Intern named the file | **9/9** |
| Filed under a corpus-marked trap date | **0** |
| Document type | 13/17 |
| Parties | 13/18 |
| Named a party the corpus marks as not defining | 1/18 |
| Description specific | 18/18 |
| Agreed with the corpus on review-or-name | 16/18 |
| Review rate | 50% here, 55.6% on the release runner |
| Date *role* correct | **6/13** |

Three of these deserve to be read carefully rather than skimmed.

**The review rate is not stable across machines, and that is not a defect.**
Scoring the same commit on the GitHub release runner sent one more document to
review: the model described the statement of work as carrying "a total fee of
$248,000", literal-evidence validation could not support that figure, and the
document went to review instead of being renamed. The filename and date were
identical and correct on both machines, and date-correct-where-named stayed at
100%. llama.cpp is not bit-identical across machines — thread count changes the
order of floating-point reductions, and greedy decoding does not remove that —
so one document of drift on an 18-document corpus moves this number 5.6 points.
The acceptance ceiling allows for it; the correctness gates do not move.

**The four corpus-wide date misses are all scans, and all four are the product
working.** On the lease, the model read `SEPTEMBER 1 24h24` and inferred September
1, 2024 - which is right - and validation threw it out, because "2024" appears
nowhere in the document literally. No date was invented and the file went to
review. Corpus-wide date accuracy therefore has a ceiling set by OCR fidelity, not
by document understanding, which is why the release gate holds the narrower and
absolute promise instead: every file Intern actually renamed carried the right
date.

**The date role is mostly wrong, and it is the one place the model is not
understanding what it reads.** It answers `effective` for every document and has
never once used `notice`, `amendment`, `invoice`, or `issuance`, though all four
are in its grammar. It picks the defining date correctly and cannot say why. The
role never reaches the filename, so no output is wrong because of it, but it was
included as evidence of understanding and it does not currently serve that
purpose. It is now scored on every run rather than being invisible.

The statement of work and the order form are the two documents built to punish a
pipeline that reaches for the easiest date. The old pipeline took the signature
date on both. The new one takes the effective date buried on page five of a
29,000-character contract, and the subscription start date from a table rather
than the "Signed on" line beside it.

## Replayed against the recorded corpus

Everything above was scored live, on the machine named at the top, and could
not be re-scored without it. Since then the corpus has a committed recording
(`fixtures/corpus-recording.json`: what the worker read from every fixture and
what the model replied, keyed by the hash of the prompt) and a baseline
(`fixtures/corpus-baseline.json`) that CI replays on every push -
[`evaluation.md`](evaluation.md) explains the mechanism. The recording was made
with the pinned Qwen3.5-2B Q4_K_M model and llama.cpp `b10361` built from
source, CPU only, 4 threads, 8,192-token context, on a Linux machine with the
parser worker built against PDFium `chromium/7881` and the pinned
`tessdata_fast` files under the distribution's Tesseract 5.3.4 rather than the
installer's vcpkg 5.5.2; the recording's `note` field says so. The corpus gained a
three-slide PowerPoint deck in round six and was re-recorded; every other
fixture scored exactly as before. The engine as it stands scores that
recording as follows.

| | Text documents (13) | Scanned (6) | Whole corpus |
| --- | ---: | ---: | ---: |
| Date correct | 12/12 | 2/6 | 14/18 |
| Filed under a corpus-marked trap date | **0** | 0 | **0** |
| Date *role* correct | **12/12** | 2/2 | **14/14** |
| Document type | **12/12** | 6/6 | **18/18** |
| Parties | **13/13** | 3/6 | **16/19** |
| Named a party the corpus marks as not defining | 0/13 | 0/6 | 0/19 |
| Description covers the listed facts | 10/12 | 6/6 | 16/18 |
| Agreed with the corpus on review-or-name | 12/13 | 6/6 | 18/19 |
| Review rate | 15% | 100% | 42% |

Live inference on that machine took a median of 13.9 seconds per document
(7.0 to 56.7), which is the recording's only number that replay cannot
reproduce.

The first replay of the recording scored four fixtures lower than this, and
the difference is the point of having one: each was a defect in the
deterministic stages after the model, found in seconds and fixed against the
model's real replies rather than a guess at them.

* The statement of work lost its second party. The model read it correctly
  as `between` Ridgeline and Contoso; the invoice repair matched the word
  "statement" in its type, found Contoso on a line with "Client", and dropped
  it as a customer. An account statement is an issued document; a statement
  of work is an agreement, and the type list now says so.
* The vendor invoice kept both parties and `between`. It has a "Bill To" line
  naming the customer *and* a "Remit To" line naming itself, and the repair
  wanted exactly one cue. Cues that agree on the issuer now settle it; cues
  that contradict each other still leave the model's answer alone.
* The amendment's date role came back `effective`. The PDF wraps "is dated"
  and "as of September 14, 2025" onto different lines, and the role was read
  from the second line alone. Wrapped sentences are now rejoined before the
  wording is read - only where the previous line did not end a sentence or a
  label and the next carries one on, so a header's "To:" and "From:" lines,
  and a "Date of this Notice:" line, keep their own cues.
* The 100-page journal was typed `Journal` under a heading that reads
  "MOONLIT ARCHIVE PROJECT JOURNAL". A supported type the title completes is
  now completed from the title, whole - never with an exhibit label, a party's
  name, or the "No" a stripped number leaves behind.

Text-document date role went from the 6/13 reported above to 12/12, and
document type from 13/17 to 17/17, on the same model. What remains is listed
under known misses.

## Performance

Measured over the same runs, per document, including extraction:

| | Qwen2.5-VL-3B, old pipeline | Qwen3.5-2B, new pipeline |
| --- | ---: | ---: |
| Median total time | 23.6 s | 12.4-27.7 s |
| Slowest document | 115 s | 51-56 s |
| Peak model process memory | 4,215 MB | 2,470-2,590 MB |
| First-run download | 3.05 GiB *(model + projector)* | 1.19 GiB *(text model only)* |

The new pipeline's spread is wide because it is a laptop measurement, not a
benchmark: repeated runs of the same corpus on the same machine produced medians
of 12.4, 16.6, 19.6, and 27.7 seconds depending on what else was running. The
slowest run was the one taken while other work was competing for the same eight
threads. Quote the range, never one figure.

Extraction is not the cost. A one-page invoice extracts in 16 ms, a 14-page
contract in 59 ms, a 100-page journal in 24 ms. Distillation is 0.03–13.5 ms.
Across the corpus the median extraction is 37 ms. The exception is a scanned
page: OCR is seconds, and the slowest document spends 6.6 s there because a page
that will not read confidently is re-read in other orientations. Essentially all
the time is the model:

| Document | Pages | Source chars | Digest chars | Distill | Inference |
| --- | ---: | ---: | ---: | ---: | ---: |
| Vendor invoice | 1 | 641 | 845 | 0.3 ms | 25.4 s |
| 100-page journal | 100 | 9,592 | 103 | 0.4 ms | 13.7 s |
| Settlement agreement | 7 | 14,837 | 12,282 | 6.7 ms | 54.8 s |
| Statement of work | 14 | 29,285 | 12,771 | 13.5 ms | 56.3 s |

Every figure in that table is read from `docs/qa/model-evaluation.json`, the
scored run this release ships. An earlier revision of the table reported 903 and
13,178 digest characters and inference times of 9.7 s and 42.3 s — numbers from a
superseded run, and the two inference figures were faster than the 13.0 s
minimum recorded anywhere in the shipping corpus.

That run's median document takes 19.9 s end to end and its slowest takes 56.4 s.
Repeated runs on the same machine moved the median between 12.4 s and 27.7 s
depending on what else was running; treat a single figure as noise.

Short documents are dominated by *generation* — the structured reply is about
240 tokens at 17.5 tokens/second. Long documents add prefill at 157
tokens/second, so the 12,000-character budget costs roughly 19 seconds on the
biggest files. That budget is the tuning lever: shrinking it trades accuracy for
seconds, and 12,000 was where date accuracy stopped improving.

The digest can exceed its budget by one block when the mandatory set alone is
larger, which is why the statement of work reports 12,771 characters. Very short
documents can report a ratio above 1: the page markers and the date index cost
more than the text saved, which is the correct trade when there is nothing to
compress.

## Compression

| Document | Source | Digest | Ratio |
| --- | ---: | ---: | ---: |
| Vendor invoice (1 page) | 641 | 845 | passthrough |
| Settlement agreement (7 pages) | 14,837 | 12,282 | 1.2× |
| Statement of work (14 pages) | 29,285 | 12,771 | 2.3× |
| Project journal (100 pages) | 9,592 | 103 | 93× |

Compression is a consequence of the document, not a configured ratio. The
journal compresses 93× because 100 pages of it differ only by an observation
number; the settlement agreement compresses barely at all because it is already
near the budget.

## What was rejected

* **LLMLingua-2.** A BERT-class encoder plus an ONNX or Python runtime, several
  hundred megabytes, to preprocess for a model that is 1.19 GiB. More decisively,
  it emits a pruned token sequence rather than document text, which destroys
  literal evidence checking and can separate a date from the words that give it
  meaning. See `docs/architecture.md`.
* **Q5_K_M and Q8.** Half the prefill speed for no measured accuracy gain here.
* **Qwen3.5-4B.** Three times slower on prefill; a 29,000-character contract
  would take minutes.
* **A vision projector.** `mmproj-Qwen3.5-2B-F16.gguf` is 668,227,264 bytes —
  637 MiB — downloaded and held resident for a path that a text-bearing corpus
  never takes. Rejected outright: none is pinned, downloaded, or loaded, and the
  request type has no field for an image. An earlier revision of this line said
  it was "installed and loaded on demand instead", which described a capability
  this build does not have.

## Known misses

Reported rather than tuned away, because thirteen documents is a small corpus and
fitting a prompt to it is not the same as being right. What the recorded
corpus still misses, as of the baseline above:

* The 100-page journal is typed, dated, and named correctly, and goes to
  review because the model reports low confidence and asks for it.
* The meeting minutes are described without the word "minutes", which the
  corpus lists as a required fact; the document itself never uses the word.
* The scanned fixtures read digits badly enough (`24h24`, `2625`) that four
  of six cannot be dated from their own text, exactly as the corpus expects
  of them; see the paragraph on OCR fidelity below.
* The settlement agreement's description omits a fact the corpus lists.

The bullets that follow predate the recording and are kept as the history of
how the numbers above were reached:

* Two documents (`meeting-minutes.md`, a 100-page project journal) get no
  document type at all and are named `<date> Document.<ext>`. Both are the
  least contract-like fixtures in the corpus. `nda.docx` answers
  `Non-Disclosure Agreement` where the document says `Mutual Non-Disclosure
  Agreement` - correct but less specific than the text supports. Since then,
  a document the model leaves untyped takes its type from its own title when
  the title names one (`Board Meeting Minutes`, `MUTUAL NDA`) and goes to
  review flagged `TYPE_INFERRED`; the rule is unit-tested on the corpus's
  titles, and the corpus has not been re-scored against it.
* The vendor invoice reads `between` its two sides rather than `from` the party
  that issued it. The direction is under-determined by the document's own layout.
  The queue now repairs `between` to `from` for invoice-shaped types when the
  layout names a customer (`Bill to`) or an issuer (`Remit to`), which covers
  this fixture; a document with neither cue keeps what the model said.
* `date_role_correct` was the weakest metric (6 of 13 on the after run): the
  model finds the right date and mislabels it. The role is now read from the
  document's wording around the date - `Effective as of`, `Invoice date`,
  `Notice is hereby given ... on` - with the model's label used only when the
  document says nothing. Each corpus date line is covered by a unit test;
  the metric itself has not been re-run.
* The six OCR fixtures are not scored for naming quality. They have since been
  run through the packaged worker against a real Tesseract, and the clean-room
  bitmap font reads at 62-95 mean confidence with digits corrupted often enough
  (`2024` as `24h24`, `2025` as `2625`) that literal-evidence validation cannot
  confirm a date. Every scanned fixture therefore goes to review, which is what
  `expected.json` expects of them and what should happen to a poor scan, but it
  means the corpus does not measure how well Intern names scanned documents.
  `fixtures/README.md` records the measured per-fixture fidelity.
* Running that path for the first time found a real defect rather than a fixture
  problem: Tesseract's orientation detection, which is trained on prose with
  ascenders and descenders, reported a confident but 180-degree-wrong rotation on
  these all-caps forms, and OCR then returned a full page of gibberish with the
  same word count as a real reading. A page that does not read confidently is now
  re-read in the other orientations and the most confident reading wins. On the
  rotated fixture that moved mean confidence from 14-23 to 76 and turned
  unusable output into the document's actual text. Upright pages, which is
  effectively every real document, still cost exactly one OCR pass.
* Those OCR measurements come from UB-Mannheim Tesseract 5.4.0, not the pinned
  vcpkg 5.5.2 that ships; treat the exact confidences as indicative.
* Repeated boilerplate used to consume digest budget. Near-duplicate blocks
  were dropped by comparing a digit-masked prefix, so `25. Assignment.` and
  `41. Assignment.` collapsed to one - but only when the repeats began the same
  way, and never when the block was mandatory. In the 14-page statement of
  work the same assignment and force-majeure clauses reappeared several times:
  some copies started with the numbered heading, some mid-clause, and every
  copy named the parties and the type, which made every copy mandatory. Two
  changes since: a block whose text is identical to one already kept (clause
  number aside) is dropped whether or not it is mandatory, and a
  non-mandatory body block that ends on the same 80 characters as one already
  kept is dropped too. Blocks that differ only in their digits are still both
  kept, because the digits may be the date. This is verified on synthetic
  documents in the distillation tests; the corpus has not been re-scored
  against it, and the release evaluation will say what it buys.

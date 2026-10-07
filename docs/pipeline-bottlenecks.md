# Where a document goes, and where it slows down or goes wrong

This page traces one file from the moment it enters the queue to the moment
it has a name and a sentence. For every stage it names the functions
responsible and what each costs. It then lists the bottlenecks: the places
that cost the most time, or that lose the facts a filename needs. Every
claim about cost is read from the code, measured by InternBench
([`internbench.md`](internbench.md)), or quoted from an earlier measurement
named where it is used. Each measured claim says which.

## The path

| # | Stage | Where | What it does |
| --- | --- | --- | --- |
| 1 | Intake | `intern-intake` (`watcher.rs`, `scan.rs`), then `intern-queue/src/pipeline.rs` `Pipeline::admit_for_enqueue` / `enqueue_admitted` | A watched folder or a drop adds a path. Admission reads the whole file to hash it (SHA-256, `files.fingerprint`) and checks exact duplicates against the queue's history (`flag_if_completed_duplicate`, `flag_if_filed_elsewhere`). |
| 2 | Claim | `Pipeline::run_next_inner` → `claim_next`, `LeaseKeeper` | One document at a time. A lease guards it against a second machine on a shared folder. |
| 3 | Snapshot | `intern-worker/src/extract.rs` `snapshot_source` | The worker copies the file into a private temp workspace, so a file saved over mid-read cannot be half-read. |
| 4 | Route | `intern-worker/src/main.rs` `extract_path`, `ROUTES` | By extension: PDF, Office (AnyDoc → Markdown), workbooks, CSV, email, text, images. |
| 5a | PDF parse + page analysis | `pdf.rs` `PdfiumBackend::inspect` | Loads the PDF once. For every page, takes PDFium's text (`page.text().all()`, in content-stream order) and adds up the area covered by images. |
| 5b | OCR decision | `extract.rs` `page_needs_ocr` | Fewer than 20 meaningful characters under heavy image cover, a stamped scan, or more than 3% replacement glyphs. |
| 5c | Render | `pdf.rs` `render_within` | Rasterises a page that needs OCR at 300 DPI, capped at 25 MP. It **reloads the PDF from disk for every page.** |
| 5d | OCR | `ocr.rs` `TesseractOcr::recognize` → `extract.rs` `read_upright` | Converts to grey, **encodes a PNG**, and **spawns `tesseract`** as a new process for each pass. A reading that is not confident (mean below 75) triggers orientation detection on a half-scale copy (another process and PNG) and then re-reads up to three more orientations. Pages are OCR'd one after another. |
| 5e | Vision image | `extract.rs` `normalize_vision_image` | For every PNG, JPEG or TIFF file, however confidently it was OCR'd (`extract_image`), and in a PDF for the first page that either reads below OCR confidence 75 or is a native page `page_needs_vision` flags, which costs that page an extra render: a Lanczos resize, an RGB PNG encode, base64, the JSON pipe, and a base64 decode on the host. The engine only checks **whether** an image exists (`engine.rs` `barely_readable`). Nothing ever reads its pixels. |
| 5f | Other readers | `extract_anydoc`, `sheet.rs`, `delimited.rs`, `email.rs`, `extract_text`, `extract_image` | Office files become Markdown; sheets become Markdown tables (200 × 30 window); emails become a header block plus body. |
| 6 | Host adapter | `intern-engine/src/worker.rs` `SupervisedWorker::extract`, `adapt_document` | Reads the JSON line (64 MiB bound). Turns low OCR confidence and truncation into field-affecting parser warnings. |
| 7 | Distil | `distill.rs` `distill` → `segment`, `collapse_running_lines`, `score_block`, `select`, `date_lines`, `emit` | Documents up to 12,000 characters pass through. Larger ones become a verbatim digest: blocks scored on date, party, type, and subject cues; mandatory blocks first; greedy by score into the budget; emitted in document order. A `SECTIONS:` outline goes in front, then an index of dated lines (**at most 14, in document order, from kept blocks only**). |
| 8 | Fit to context | `engine.rs` `Engine::analyze` | Estimates prompt tokens (one per digit or CJK character). If the prompt plus 1,024 reply tokens would not fit 8,000, it distils again smaller, up to twice. If the server says the prompt does not fit, it halves once more. |
| 9 | Prompt | `prompt.rs` `build_prompt`, `client.rs` `ModelRequest::from_digest` | A fixed system turn. The user turn opens with one sentence that **varies with whether the digest was condensed**, then about 4,000 characters of fixed instructions, then the document between delimiters. |
| 10 | Inference | `client.rs` `ModelClient::propose_once`, llama-server | One chat completion: greedy, GBNF grammar, `cache_prompt`, thinking off, at most 1,024 reply tokens. The grammar's field order puts a verbatim `type_evidence` quote before `document_type` and a `date_evidence` quote before `document_date`; `party_evidence` (a copied line per party, up to three) comes **after** `parties`, so it justifies the parties rather than leading to them. A reply that will not parse is retried once. |
| 11 | Validate | `validate.rs` `validate` → `validate_document_type`, `validate_date`, `validate_parties`, `validate_description`, plus `infer.rs` `infer_date_role`, `infer_document_type`, `repair_issued_relation` | Every fact is checked against the **digest**, not the source. The date must be stated in an ordinary form, must not belong to another document, and must not be only a deadline. A type the model left out is taken from the title. The issuer of an invoice is repaired to `from`. |
| 12 | Name | `engine.rs` `finish` → `naming.rs` `compose_filename`, `evidence.rs` `stated_dates` | `YYYY-MM-DD <type> <relation> <party>[ and <party>].<ext>`, made Windows-safe, collision-suffixed. |
| 13 | Queue side | `pipeline.rs` `analyze_with_deadline`, `near_duplicate_of`, `compose_for_target`, house style, own names, `apply_if_unchanged` | Applies the 15-minute deadline and the near-duplicate check (simhash), recomposes the name for the destination, applies learned spellings and the organisation's own names, then renames or routes to review. |

## Latency: where the time goes

What the code says, before measurement:

1. **Generation is the floor on every document.** The grammar makes the model
   write `type_evidence` before the type and `date_evidence` before the date,
   then the parties followed by one `party_evidence` per party (each a copied
   line). Then it writes the description. Roughly 150–250 generated tokens
   per document run at ~17 tokens/s on an 8-thread laptop (`llama-bench`,
   [`model-bakeoff.md`](model-bakeoff.md)), and slower with fewer threads. That is
   10–15 s even for a one-page invoice. The grammar's `string` is `char*`,
   so nothing bounds a quote: a model that copies a whole opening paragraph
   as evidence pays for every token of it. (`prompt.rs` `RESPONSE_GRAMMAR`
   and the reply shape; `client.rs` `MAX_REPLY_TOKENS`.)
2. **Prefill scales with the digest, and the prompt prefix is not stable.**
   The digest is capped at 12,000 characters (about 3,000 tokens of prose; a
   ledger can cost more). The fixed instructions are about 1,000 tokens.
   llama.cpp's `cache_prompt` can reuse a common prefix with the previous
   request. But `build_prompt` puts the condensed-or-complete sentence
   **before** the fixed instructions, so whenever consecutive documents
   differ in that respect, the whole instruction block is prefilled again.
   (`prompt.rs` `build_prompt`.) InternBench records `cached_tokens` per
   document, which shows how often this happens. The measurement below shows
   that on the pinned model the cache is rarely reused at all, for a reason
   reordering would not fix.
3. **OCR pays process and encoding overhead on every pass.** Each Tesseract
   pass is a new process that reloads its language model and decodes a PNG
   the worker has just encoded. A full-page 300-DPI grey PNG is about 8 MP to
   compress. A page that does not read confidently pays for up to five
   passes. Pages are sequential, on a machine where the model server leaves
   half the cores idle during extraction. (`ocr.rs` `recognize_at`,
   `detect_orientation`, `write_png`; `extract.rs` `read_upright`.)
4. **Rendering re-opens the PDF for every OCR'd page.** `render_within` calls
   `load_pdf_from_file` each time. That is negligible for a one-page scan and
   linear extra parsing for a 100-page one. (`pdf.rs`.)
5. **Scans and image files build an image nobody reads.**
   `normalize_vision_image` resizes with Lanczos3, encodes an RGB PNG, and
   base64s it into the reply. Every PNG, JPEG or TIFF pays for it, however
   confidently it was read; a PDF pays for it once, for its first
   low-confidence OCR page or vision-flagged native page (which is rendered
   only for this). The host decodes it. Only its presence is used.
   (`extract.rs` `extract_image`, `extract_pdf`; `engine.rs` `barely_readable`.)
6. **Everything is serial.** One document at a time; within it, extraction
   then inference. While the model generates, the worker is idle, and so are
   the cores the model does not use.
7. **Small, fixed costs.** The file is read at admission (hash) and again
   copied by the snapshot. Distillation, validation and naming are
   milliseconds.

### Measured: the InternBench baseline

The baseline in [`bench/reports/`](../bench/reports/) is one live run of all
52 InternBench documents. It used the pinned model through llama.cpp at 4
threads with an 8,192-token context, on an otherwise idle 4-core Xeon
(2.8 GHz, AVX-512) with 16 GB. This machine is slower than the laptop in
[`model-bakeoff.md`](model-bakeoff.md): about 59 prompt tokens per second
and 12 generated tokens per second, against 157 and 17.5. Read the shares
below; the absolute seconds will differ on other hardware.

| Where the time went (51 completed documents, 2,970 s) | Share |
| --- | ---: |
| Prefill (reading the prompt) | 74.8% |
| Generation (writing the reply) | 21.6% |
| Extraction, all of it | 2.9% |
| of which OCR | 2.5% |
| Distillation, validation, naming | < 0.1% |

Per document:

| | p50 | p95 | max |
| --- | ---: | ---: | ---: |
| Total | 52.8 s | 115.5 s | 142.5 s |
| Prefill | 38.6 s | 86.8 s | 105.7 s |
| Generation | 12.5 s | 17.5 s | 18.8 s |
| Prompt tokens | 2,210 | 5,051 | 5,783 |
| Tokens reused from the prompt cache | 46 | 1,166 | 1,219 |
| Generated tokens | 139 | 207 | 216 |

Four things follow.

1. **The prompt cache does almost nothing for this model.** Only 8 of the 51
   documents reused more than 800 cached tokens; the median reused 46, which
   is the system turn. The pinned model is a hybrid with recurrent layers.
   llama.cpp cannot rewind recurrent state to an arbitrary prefix. It can only
   restore a saved checkpoint, and the server log shows one about 512 tokens
   before the end of the previous prompt. Once a prompt is longer than about
   1,500 tokens, that checkpoint lies past the fixed instructions. The next
   document then prefills the whole ~1,000-token instruction block again:
   about 17 s per document on this machine and 6 s on the laptop. Reordering
   the prompt cannot fix that; a shorter fixed block can.
2. **The document itself is the rest of the prefill.** A median prompt of
   2,210 tokens is about 1,200 tokens of document under 1,000 of
   instructions. Long documents send up to 14,000 characters of digest, and
   that is where p95 comes from.
3. **Generation is a floor of 10–18 s on every document,** a one-page
   receipt included. The reply copies three evidence quotes before the facts.
   Its median is 139 tokens, 216 at most. One reply, on the 10-page data
   processing addendum, ran to the 1,024-token limit and failed as
   `MODEL_REPLY_TRUNCATED`.
4. **OCR is cheap by comparison.** Even the 25-page scan spent 38 s on OCR
   against 82 s of prefill. A more accurate OCR engine that is slower per
   page costs little end to end. Keeping easy text documents on the fast
   path matters more than shaving seconds from OCR.

Memory: the parser worker peaked at 121 MB, on the noisy scan. The model
server ran at 3.6 GB typical and 5.5 GB at its peak, well above the
1.28 GB model file. The excess is the server's own context and prompt-cache
state, not Intern's. It deserves its own look before Intern raises the
context size.

## Accuracy: where facts are lost

1. **Reading order on multi-column PDFs.** `page.text().all()` returns text in
   content-stream order. A producer that writes a two-column page row by row
   interleaves the columns, and nothing downstream can unscramble it.
   (`pdf.rs` `inspect`.)
2. **The digest decides what the model and the validator may know.** Facts
   outside the digest cannot be quoted, and validation checks only the
   digest (`validate.rs`). On long documents, the selection is greedy by
   score with "mandatory" blocks first (`distill.rs` `select`). "Mandatory"
   includes every block with a date-role cue and every block on page 1 with
   a date. A 50-page agreement can therefore fill the budget with dated
   boilerplate and covenant deadlines, while the one definition that carries
   the defining date competes on equal terms. InternBench's `digest_recall`
   measures this directly and deterministically.
3. **The date index is truncated in document order.** `date_lines` keeps the
   first 14 dated lines of the kept blocks. A statement, a schedule, or a
   long agreement whose defining date comes after 14 other dated lines has
   that date left out of the index. That index is the device that took the
   original corpus from 9/11 to 11/11 dates.
4. **OCR digits.** Literal-evidence validation is right to refuse a date it
   cannot find. But a scan that turns `2024` into `24h24` can never be dated
   from its own text. OCR fidelity on digits and names caps accuracy on
   scans. InternBench measures it separately: date, name and identifier
   accuracy.
5. **Tables and layout.** A date in a table cell and parties identified only
   by layout (letterhead, "Bill To" blocks, form boxes) depend on how the
   reader linearises the page. PDFium gives no table structure, so a header
   row and its values can be far apart in the text.
6. **One inference, no second look.** A low-confidence or contradictory reply
   goes to review rather than being re-asked with a narrower question. This
   is safe, but on long and complex documents the review rate is the cost.

### Measured: where Intern is right and wrong

| | Result |
| --- | ---: |
| Whole filename right | 14/52 (27%) |
| Document type | 38/52 (73%) |
| Defining date | 43/52 (83%) |
| Parties | 37/52 (71%) |
| Joining word right, where parties were named | 19/51 (37%) |
| Routing (ready vs review) agrees with the reviewed answer | 28/40 (70%) |
| Description states every listed fact | 8/52 (15%), 57% of facts on average |
| Description states nothing the document does not | 47/51 (92%); 4 of 167 checkable claims unsupported |
| Gold evidence present in the digest / in the prompt sent | 100% / 100% |
| **Filed without review under a wrong name** | **20/52** |
| Filed under a date the corpus marks as a trap | 7/52 |

The safety line is the one to read first. 20 documents would have been
renamed without anyone looking, under a name the reviewed answer disagrees
with. 6 of those sprang a trap:

* the master agreement's date on a statement of work issued under it;
* the start date on an offer letter;
* a design-freeze date on a launch plan;
* the first entry's date on a maintenance record;
* a patient's date of birth on an intake form;
* the bill-to customer named as a party to an invoice.

The other 14 are names that are close but not right. Most often the joining
word is missing; sometimes the title is shortened ("Minutes" for "Board
Meeting Minutes"). These are not dangerous one at a time, but they are
exactly what a person has to fix by hand.

Where the misses come from:

* **The joining word.** The model answered `none` for 21 of the 51 documents
  that should name a relation, so the filename reads `- Party`. It almost
  never chose `for` or `from` unprompted. The vocabulary is offered and not
  used.
* **Who the parties are.** A notice was named for the landlord who sent it
  instead of the tenant it is about. A prior authorization was named for the
  health plan instead of the patient. An insurance declarations page was
  named for the agency, an upside-down purchase order for the supplier. Each
  time the model found a real name and gave it the wrong role.
* **Types.** 14 types were unsupported, generic or wrong:
  * "Document";
  * "Approval of Minutes" for a written consent;
  * the delivery line "via Email and Certified Mail, Return Receipt
    Requested" taken as the type of a demand letter.
  The fallback that takes the type from the title then picked the wrong
  heading.
* **Dates.** Every miss was a reading error, not a retrieval error: the right
  date was in the digest every time. The 100-page report, dated only on page
  41, was dated correctly.
* **Scans.** OCR fidelity explains two misses outright. The skewed notice
  lost three paragraphs (64.6% CER), and the noisy statement read at 38% CER
  with the issuer's name destroyed. The other scans read at 0–2.5% CER and
  failed for the same reasons digital documents do.

On this corpus, digest recall and prompt recall are 100%. Distillation is
**not** losing the facts. Long documents miss because of how the facts are
read and labelled, not because they never reach the model.

The existing corpus in `fixtures/`, run live on the same machine the same
evening, scored what its committed baseline says:
* 11/18 filenames, 14/18 dates and 0 trap dates;
* 18/18 types, 16/19 parties and 11/16 relations;
* 18/19 routing, with a 42% review rate.

Its documents are shorter, and its median document took 20.3 s end to end.
Every miss on that corpus is one InternBench also shows: a joining word, a
party seen through OCR, or a date a scan does not state legibly.

## The highest-value changes

Ranked by what the measurements say they are worth.

### Complex document understanding

1. **Ask for roles, derive the joining word.** The model reliably finds
   names and unreliably labels them. Capture each party's role (issuer,
   customer, landlord, tenant, employer, employee, …) and compose `from`,
   `for`, `to`, `with` or `between` deterministically. On the baseline, 32 of
   51 joining words are wrong and most of them are `none`.
2. **Give the model the layout it lost.** Multi-column reading order (the
   interleaved declarations page), labelled values (`Bill To` blocks, form
   fields) and table rows decide who is who. A structured representation with
   block identifiers lets the prompt say what is a heading, a table cell or a
   key-value pair.
3. **Constrain the type to the document's own titles and labels.** The
   fallback picked delivery lines and agenda items. Rank title candidates
   structurally (first heading, largest font, a type noun) rather than taking
   the first heading.

### OCR

1. **A modern recognizer.** PP-OCRv5 through ONNX Runtime, measured on the
   same 13 InternBench scans, cut character errors from 2.5% to 0.5%. It
   raised date accuracy from 0.90 to 0.97 and identifier accuracy from 0.80
   to 0.93. It costs more per page, but OCR is 2.5% of the time.
2. **Deskew and orientation that do not drop text.** The 3° skewed page lost
   three paragraphs.
3. **Second readings only where it matters.** Re-read a low-confidence line
   only when it carries a date, an amount, an identifier or a name, instead
   of re-reading whole pages.
4. **Keep the native layer when it is trustworthy, and only then.** The
   invisible, OCR-corrupted text layer was trusted and the date could not be
   confirmed.

### Long documents

1. **Retrieve by field, not by one global score.** Recall is already 100%,
   so the gain is in size. One global compressor sends up to 14,000
   characters to make sure everything survives. Field-specific retrieval
   (type, date, parties, subject, identifiers), each with its own small
   budget, sends the evidence for each question and little else.
2. **Never let a reply run away.** Evidence by block identifier, and bounded
   strings, would have saved the one long document that failed on a
   truncated reply.

### Latency

1. **Shrink the fixed instructions.** They cannot be cached on this model,
   so every token of them costs prefill on every document: about 17 s here
   and 6 s on the laptop.
2. **Cite evidence by identifier instead of copying it.** Generation is
   10–18 s, mostly quotes; identifiers are a few tokens each.
3. **Send less document.** Field retrieval with budgets, sized to the
   document.
4. **Keep OCR off the critical path** with bounded page-level parallelism.
   Leave easy text documents on the fast native path.

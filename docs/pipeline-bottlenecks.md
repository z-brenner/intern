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
   document, which shows how often this happens.
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

<!-- MEASURED-LATENCY -->

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

<!-- MEASURED-ACCURACY -->

<!-- ANALYSIS -->

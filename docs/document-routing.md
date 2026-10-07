# Document routing and page layout

Every page the parser worker sends now carries two things: the text the
engine has always read, and a **layout** — the same page as blocks in the
order a person reads them, each with a stable id. This page explains how a
PDF page is routed to the reader that builds them, what the router measures,
where each threshold comes from, what it costs, and what was checked to show
that easy documents read exactly as before.

The code is `crates/intern-worker/src/layout.rs` (the representation),
`layout/router.rs` (signals and thresholds), `layout/geometry.rs` (the
analysis of where text sits), `layout/text.rs` (blocks from text), and the
routed reading loop in `extract.rs`. The engine mirrors the types in
`crates/intern-engine/src/structure.rs`, and `structured(&DocumentSource)`
there returns blocks for every page of every document.

## What a layout is

A page layout is the page's blocks in reading order. A block is a heading,
paragraph, list item, table, labelled values (`key_value`), caption, running
header, or running footer. Its id is `p{page}.b{n}`, `n` counting blocks in
reading order from 1; a table's rows are `p{page}.b{n}.r{k}` and its cells
`….c{j}`; labelled values are fields `….f{k}`, each a key and a value. Every
block names the heading it falls under (`section`), across pages, says where
its text came from (`native` or `ocr`), and carries OCR's mean confidence in
it when OCR read it. Where the reader knows geometry, every block, line,
cell, key and value has a box: `[x0, y0, x1, y1]` in tenths of a PDF point
from the top left of the page as displayed, so everything is an integer and
serialises the same way every time.

Every reader produces one:

| Reader | Route | Blocks from |
| --- | --- | --- |
| PDF, text page | `fast` / `layout` / `ocr_regions` | see below |
| PDF, scanned page; images | `ocr` | the OCR engine's lines (Tesseract's TSV lines today) |
| Office (Markdown), sheets, CSV, email, text | `fast` | the text's own structure: Markdown headings, blank lines, `\|` table rows, list markers, `Key: value` lines, short lines in capitals; no boxes |

Running headers and footers are marked across the document on every route:
a short block at the top or bottom of a page whose text, figures aside,
heads or foots at least a quarter of the pages (two at the least, three
words or more), or that is only a page number, becomes `page_header` or
`page_footer`. Only its kind changes; the text and ids stay. On InternBench
that marks the annual reports' running titles and page numbers and the
agreements' `Page 2 of 10` footers, and leaves a first page's letterhead
alone.

A page past the protocol's layout budget (16 MiB of serialised layouts per
document, a page whose text was cut) is sent without one, and the engine's
`structured()` segments it from its text with the same id scheme, using the
distiller's own heading and table rules. So does a page stored before layouts
existed. Phase 3 can therefore index every page of every document.

## The four routes

A PDF page goes one of four ways, decided per page from signals PDFium hands
over while inspection already has the page open:

* **`ocr`** — the scan rule pages have always been routed by
  (`page_needs_ocr`: almost no meaningful characters under heavy image
  coverage, a stamp-only text layer on a full-page image, or more than 3%
  replacement glyphs), plus one new case: an *invisible* text layer over a
  full-page image whose words look like a bad prior OCR. That page is read
  again, and the fresh reading replaces the layer only if it is confident
  (mean ≥ 75) or has fewer OCR-error words; otherwise the layer is kept. The
  page text is the OCR engine's text, exactly as before.
* **`ocr_regions`** — trustworthy native text with an image of at least 15%
  of the page that no text overlaps (a pasted scan of a signature block, a
  stamped certificate). The page is rendered once, the regions cropped and
  read by OCR, and lines read at confidence ≥ 50 (regions at mean ≥ 60) are
  placed into the page's geometry and analysed with its native text. The page
  text is the blocks' linearisation.
* **`layout`** — side-by-side flows, table rows, grids of labelled values, or
  overprinted text: the page's characters are read into runs (characters on
  one baseline with no wide gap) and the page is rebuilt from their geometry —
  reading order by a recursive cut into columns and bands, tables from cells
  that line up and from ruling lines, labelled values paired with their
  labels, headings by size and weight, running headers and footers by
  position. The page text is the blocks' linearisation: blocks separated by a
  blank line, tables as `| a | b |` rows (which the distiller already treats
  as tables), labelled values as `Key: value` lines.
* **`fast`** — everything else. The page text is **exactly**
  `page.text().all()`, byte for byte as before, and blocks are built cheaply
  from that text; a line gets a box when the page's text objects fall into
  exactly as many lines as its text has, and is left without one otherwise.

Only the `layout` and `ocr_regions` routes read characters one by one
(four PDFium calls per character), which is why the router decides from
cheaper signals first.

## Signals

All signals are integers (ratios per mille) and travel with the layout, so a
benchmark can say why a page went the way it did. They come from one walk of
the page's objects — the same walk that measured image coverage before — and
from the page text.

| Signal | What it is |
| --- | --- |
| `chars` | Non-whitespace, non-control, non-replacement characters of the native text. |
| `segments` | Text objects (PDFium's runs of one style), with their boxes, in content-stream order. A form XObject's text counts as one box where the form is drawn. |
| `image_coverage` | Per mille of the page covered by images (as before). |
| `replacement` | Per mille of non-whitespace characters that are U+FFFD. |
| `invisible` | Per mille of text objects drawn in render mode 3 — how an OCR layer sits under a scan. |
| `garbage` | Per mille of words that look like OCR errors: a digit between letters (`Ca1der`), a confusable letter among digits (`2O26`, `$69.9O`), a word starting `0`/`1` then letters (`0strander`), case flipping inside a word. Identifiers are judged part by part, so `INV-2026-0042` is clean. |
| `columns` | Side-by-side flows: 1 plus the number of whitespace bands at least a line height wide that run down a part of the page (parts are split at gaps of 1.25 lines) with text on both sides in three or more rows. Table columns count too. |
| `interleave` | Per mille of consecutive segments, in content-stream order, that jump across the widest such band: high when columns are written row by row, near zero when column by column. Diagnostic. |
| `aligned_rows` | Rows of three or more segment cells, two of which line up (left or right edge) with a cell of the row above or below. |
| `key_values` | Text lines that read as `Label: value` or a bare `Label:`. Diagnostic. |
| `key_value_grid` | Text lines holding two or more labels (`Invoice No: 4471   Date: May 1`, `Name: …   Name: …` across signature blocks). |
| `overlap` | Per mille of segments covering half of another — overprinted text. |
| `font_sizes` | Distinct segment heights to the half point. Diagnostic. |
| `rulings` | Thin path objects (and the sides of stroked rectangles): rules. |
| `image_region` | Per mille of the page in the largest image that text overlaps by at most a tenth of its area. |
| `rotation` | The page's `/Rotate`. Boxes are turned into the displayed frame. |

## Thresholds and the measurements behind them

The calibration (`cargo run --release -p intern-worker --features
windows-native --example route_calibration -- FILE.pdf …`) prints, for every
page, the route, every signal, and what the full geometry analysis finds when
it reads the page's characters anyway: tables of two or more rows, labelled
values set apart from their labels (a value under its label, or one that
wraps), and a reading order that jumps back up the page. A page where the
analysis finds any of those, and the native text would run together, "needs
layout". It was run on every PDF of InternBench (36) and of the clean-room
corpus (11 readable), 414 pages, plus one synthetic mixed page (below).

Of the 371 pages with native text:

| Signal | Pages that need layout | Pages that do not |
| --- | --- | --- |
| `columns` = 1 | 4 | 207 |
| `columns` = 2 | 25 | 0 |
| `columns` ≥ 3 | 135 | 0 |
| `aligned_rows` = 0 | 26 | 207 |
| `aligned_rows` 1–2 | 2 | 0 |
| `aligned_rows` 3–4 | 12 | 0 |
| `aligned_rows` ≥ 5 | 124 | 0 |
| `key_value_grid` = 0 | 149 | 204 |
| `key_value_grid` = 1 | 8 | 3 |
| `key_value_grid` ≥ 2 | 7 | 0 |
| `key_values` ≤ 5 | 153 | 206 |
| `key_values` ≥ 6 | 11 | 1 |
| `interleave` = 0 | 4 | 207 |
| `interleave` ≥ 100 | 157 | 0 |

So the thresholds (`layout::router::thresholds`) are:

| Threshold | Value | Why |
| --- | --- | --- |
| `columns` → layout | ≥ 2 | Separates the classes completely on this corpus: no page that reads well as it is has a second flow, and every page but four that needs layout has one. It also catches column-ordered two-column pages (lease page 1), whose text is in order but whose blocks and tables are not. |
| `ALIGNED_ROWS` | 3 | Zero on every page that does not need layout; 3 or more on 136 that do. Kept as its own trigger for a table whose columns are too close for a band (no page in this corpus depends on it alone). |
| `KEY_VALUE_GRID` | 2 | Every page with two or more such lines needs layout; three pages with one do not. The first draft routed on `key_values ≥ 6` (one label to a line); that sent the corpus invoice and the statement of work's signature page to the layout route, which gained nothing - the fast route's blocks already pair one label to a line - and tore the invoice's unaligned line items apart. |
| `OVERLAP` | 100 ‰ | No page in either corpus has any overlap; the guard is for overprinted text, which no fast reading can order. |
| `INVISIBLE_LAYER`, `FULL_PAGE_IMAGE`, `GARBAGE_LAYER` | 500 ‰, 900 ‰, 40 ‰ | The OCR-corrupted invoice: invisible 1000, image coverage 1000, garbage 127. On all 371 native pages `garbage` is at most 3 and `invisible` is 0, so a clean layer is never re-read. Its fresh reading (95% confident) replaces it: `Wexcornbe Mi11work` becomes `Wexcombe Millwork`, `O3/lO/2O26` becomes `03/10/2026`. |
| `IMAGE_REGION`, `REGION_MIN_CHARS` | 150 ‰, 20 chars | No InternBench or corpus page has native text beside a large image (the spec's "mixed PDF" documents are not in the corpus yet). Verified on a synthetic page: native amendment text with the scanned signature block of `scan-clean-lease-2p.pdf` pasted below it (21.8% of the page) routes `ocr_regions`; the landlord, tenant, names and title come back as OCR key-value blocks after the native text, and the whole page takes 0.37 s. |

What the router gets wrong on this corpus, all on the fast route:

* `annual-report-100p.pdf` page 34, `credit-agreement-50p.pdf` page 3 and
  `board-minutes.pdf` page 1 have labelled values that wrap onto the next
  line ("Directors present: …" over three lines). The fast route keeps the
  continuation as the following line, in order, so nothing is lost; only the
  field's value is shorter than it could be.
* `vendor-invoice.pdf` (corpus): the analysis finds a "table" in line items
  whose cells are not aligned at all; read on the layout route they were
  torn apart, and the fast text keeps each item on its line.

Against InternBench's categories (`multi_column`, `table`, `form`,
`layout_parties`, `complex_pdf`, `date_in_table` need layout;
`simple_digital` does not), at the document level: all 23 native PDFs with a
layout category have their layout pages routed `layout`; every corpus text
PDF (nine, and the two duplicate invoices) and `board-minutes.pdf` stays
entirely on the fast route; five
native PDFs with no layout category route some pages to layout, and each of
those pages has a table or side-by-side signature blocks the categories do
not mention (the fee table in the engagement letter, the rent table in the
notice of rent increase, the test plans in the SOW, the signature pages of
the assignment and services agreement).

Per page, over all 414: 211 fast, 160 layout, 43 OCR (every scanned page,
plus the corrupted invoice's re-read), and the synthetic page `ocr_regions`.

## The fast route reads exactly as before

`scripts/compare-worker-pages.mjs --before OLD --after NEW FILE…` runs two
worker builds over the same files and fails if any page the new one read on
the fast route differs from the old one's in text, source, OCR confidence or
page-image flag, or if a document's outcome, page count, warnings or
truncation changed. Run with a worker built from the commit before this work
(`9cd57a2`) against the final one:

* Every file that is not a scan - InternBench's 40 and the corpus's 15
  readable ones, every format (PDF, Office, sheets, CSV, email, text) -
  **228 of 228 fast-route pages byte-identical**, with warnings, truncation
  and page-image presence unchanged.
* The corpus's encrypted, malformed, lock-file and unsupported inputs fail
  with the same code and message.
* 18 scanned and mixed files (31 pages): every page OCR read before reads
  byte-identical, at the same confidence, with the same warnings; the fast
  pages of the mixed documents are identical. (The 25-page scanned lease was
  left out of these runs for time.)

The existing corpus replay gate (`intern-evaluate --replay`) reads the
recorded extractions, which carry no layout, so its prompts are unchanged by
construction; the engine's `distill` does not read layouts at all.

## Layout quality

What the layout route makes of the documents it exists for (all fictional):

* `lease-two-column.pdf`: page 1's full-width title, parties paragraph and
  basic-information table are read first, then the left column's sections
  2–6, then the right column's 7–10; the table keeps each term with its
  provision, wrapped provisions joined (`| Base Rent | Year 1: $34.00 … |`).
  Page 2's columns are already written column by column and the page stays
  on the fast route. Page 3's two signature blocks become one block each.
* `declarations-interleaved.pdf`: the policy fields read as fields
  (`NAMED INSURED` = `Wexcombe Bicycle Cooperative 2214 Kingsfold Avenue
  Westharrow, MI 49103`, `DATE ISSUED` = `06/18/2026`), the agent block
  beside them separately, and the coverage table as a table with its total;
  PDFium's text ran the two columns together line by line.
* `invoice-date-in-table.pdf`: the header grid becomes fields (`Invoice
  Date` = `03/04/2026`, `PO Date`, `Ship Date`, `Due Date`); line items are
  a five-column table whose totals (`Subtotal`, `Freight`, `Sales tax`,
  `TOTAL DUE (USD)`) are rows of their own under the amount column; BILL TO
  and SHIP TO are a two-column table.
* `purchase-order.pdf`: the PO grid becomes six fields; the line-item table
  keeps its header and every row, and `Order Total (USD)` is its own row.
* `change-order-form.pdf`: `TO OWNER` / `TO CONTRACTOR` become fields with
  the names under them; the change-order grid five fields; the changes a
  table whose summary lines (`The original Contract Sum was` …) are two-cell
  rows; the signature grid a three-column table.
* `account-statement.pdf`: the statement fields, the account summary and
  the transaction tables on both pages, with an empty cell where a row has
  a credit and no debit, so every figure stays under its heading.
* `vendor-registration-form.pdf`: numbered form fields set as small labels
  over values become fields (`3. Federal employer identification number
  (EIN)` = `84-0293157`), sections are headings, and a section heading
  after a table ends it.
* Scans (Tesseract): every scanned page gets blocks from Tesseract's lines,
  marked `ocr` with their confidence — the 10-page agreement 3 to 12 blocks a
  page with headings, key-values and page footers; the patient intake form 7
  tables and its fields. Tesseract reads a page lying on its side as
  vertical text without being asked; its boxes are turned upright so the
  rotated invoice's layout is the invoice (fields `Invoice No.`, `Invoice
  Date`, `Bill To`, totals) and not its lines stacked sideways.
* Mixed: `scan-mixed-amendment.pdf` reads page 1 fast, page 2 (a table) on
  the layout route, page 3 by OCR; `mixed-signature.pdf` page 1 fast and
  page 2 by OCR.

The checks that drove the fixes (scripts kept beside the calibration, not in
the repository): for every page off the fast route, every old line of three
or more words must still be on one line of the new text (45 of 4,044 are
not, all of them two signature blocks or two form columns that PDFium had
run together), and the words must be the same multiset (the only changes are
four labels that gain their colon and the re-read invoice).

## What it costs

Measured on the shared development container (4 cores, other builds and
benchmarks running; all timings contended), release build.

Per native page (414-page calibration): signals median 44 µs (p90 92 µs),
fast-route blocks median 9 µs, the geometry analysis of a page's runs median
48 µs (p90 140 µs). The 100-page annual report: signals 4.5 ms in all, fast
blocks 0.9 ms, geometry on its 93 layout pages 10.3 ms; inspection's
analysis (object walk, signals, characters of layout pages) 0.20 ms a page.
The 100-page corpus document (all fast): signals 0.36 ms in all.

Whole extraction, worker-reported, minimum of five alternating runs:

| Document | Pages | Routes | Before ms | After ms |
| --- | ---: | --- | ---: | ---: |
| long-document-100-pages.pdf | 100 | fast 100 | 4.7 | 6.3 |
| statement-of-work.pdf | 14 | fast 14 | 10.6 | 11.4 |
| settlement-agreement.pdf | 7 | fast 7 | 6.5 | 6.3 |
| vendor-invoice.pdf | 1 | fast 1 | 1.1 | 1.2 |
| annual-report-100p.pdf | 100 | fast 7, layout 93 | 59.4 | 94.8 |
| credit-agreement-50p.pdf | 50 | fast 39, layout 11 | 39.5 | 47.2 |
| asset-purchase-agreement-25p.pdf | 25 | fast 14, layout 11 | 19.1 | 26.4 |
| data-processing-agreement-10p.pdf | 10 | fast 3, layout 7 | 10.9 | 16.5 |
| sow-under-msa-5p.pdf | 5 | layout 5 | 6.8 | 11.6 |
| lease-two-column.pdf | 3 | fast 1, layout 2 | 3.8 | 5.1 |
| invoice-date-in-table.pdf | 1 | layout 1 | 1.8 | 2.4 |
| all 52 digital documents | | | 254.4 | 341.2 |

Scanned documents (one run each; Tesseract 5.3.4):

| Document | Pages | Before ms | After ms |
| --- | ---: | ---: | ---: |
| scan-agreement-10p.pdf | 10 | 13,695 | 6,005 |
| scan-clean-lease-2p.pdf | 2 | 2,229 | 1,639 |
| scan-noisy-statement.pdf | 1 | 5,564 | 5,103 |
| ocr-corrupted-invoice.pdf | 1 | 2 | 906 (re-read) |
| all 18 scanned files | 31 | 34,778 | 25,051 |

A PDF's scanned pages are rendered one at a time (PDFium is not
thread-safe; inspection and rendering share one open document) and read by
a pool of OCR workers through a bounded queue: as many as the engine offers
(`OcrBackend::concurrency`; for Tesseract half the logical cores, at most
four). Pages are put back in order and every order-dependent decision (which
page becomes the page image, warning order, which error is reported) is the
one a sequential read makes; reading with one worker gives the same
document. Tesseract is started with `OMP_THREAD_LIMIT=1`: a Tesseract built
with OpenMP starts a thread per core in every process, and two pages at once
then took 128 s instead of 1.5 s on this machine. The optional page image,
whose pixels nothing reads, is a 256-pixel grey thumbnail.

## Known limits

* Only the OCR engine's lines are geometry on scanned pages; a PP-OCR
  backend that fills `OcrResult::lines` in upright page pixels gets the same
  analysis. OCR page text stays the engine's own text (Tesseract's, exactly
  as before); the blocks are built from the same words.
* Tables are found from alignment and rules, not from a model; borderless
  tables whose columns never line up (the corpus invoice) stay on the fast
  route as lines.
* `ocr_regions` has no corpus document yet; its thresholds are the spec's
  15% and the synthetic page above.
* A heading inside a table that is neither larger, bold, nor capitalised
  stays a row of the table.
* Running blocks are found by repetition: the corpus's synthetic 100-page
  journal, whose every page is a title, a date and one line
  `Fictional observation 042`, has that line marked a footer.

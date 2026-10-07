# Document routing and page layout

Every page the parser worker sends now carries two things: the text the
engine has always read, and a **layout** — the same page as blocks in the
order a person reads them, each with a stable id. This page explains how a
PDF page is routed to the reader that builds them, what the router measures,
where each threshold comes from, what it costs, and what was checked of what
easy documents read.

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
| PDF, scanned page; images | `ocr` | the OCR engine's lines (PP-OCR's where its runtime is installed, Tesseract's TSV lines otherwise) |
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
  engine's lines go through the same analysis as a `layout` page's runs, and
  the page text is the blocks' linearisation. An engine's own text reads an
  unruled table down its columns or across a row of boxes and pairs no label
  with its value; on InternBench's 72 documents, reading the blocks instead
  took labelled values from 90.5% to 99.3%, with table rows (98.2%), reading
  order (100%) and every date, name and identifier the same. A reading with
  no lines keeps the engine's text. OCR measures a line's height by its
  letters, not its type size - a heading in capitals measures smaller than
  the prose under it - so on these pages a line over another is taken for
  a form's caption over its value only if its text reads as one: not a
  part's heading (`ARTICLE 4 - …`), not over a numbered clause (`4.1 …`),
  not a line that holds a label already or over one that opens with one
  (`By:`, `Title:`), and not a name - a person's, a firm's, a title - unless
  a word of it names a field (`Member ID`, `Invoice Date`). Otherwise the
  page text would gain colons the document does not have
  (`Palisade Tower Partners LLC: By: …`).
* **`ocr_regions`** — trustworthy native text with an image of at least 15%
  of the page that no text overlaps (a pasted scan of a signature block, a
  stamped certificate). The page is rendered once, the regions cropped and
  read by OCR, and lines read at confidence ≥ 50 (regions at mean ≥ 60) are
  placed into the page's geometry and analysed with its native text. The page
  text is the blocks' linearisation. At most four regions are read, largest
  first, and an image a quarter or more of which lies under a larger region
  already chosen is not read again - a scan drawn with a copy over it, a
  thumbnail on its full-size image - since what lies under both would be
  merged into the page twice. A quarter of the smallest region the router
  asks for is still several lines of text; images that only touch, or a
  stamp over a corner of a scan, overlap far less and are both read.
* **`layout`** — side-by-side flows, table rows, grids of labelled values, or
  overprinted text: the page's characters are read into runs (characters on
  one baseline with no wide gap) and the page is rebuilt from their geometry —
  reading order by a recursive cut into columns and bands, tables from cells
  that line up and from ruling lines, labelled values paired with their
  labels, headings by size and weight, running headers and footers by
  position. The page text is the blocks' linearisation: blocks separated by a
  blank line, tables as `| a | b |` rows (which the distiller already treats
  as tables), labelled values as `Key: value` lines.
* **`fast`** — everything else. The page text is **exactly** what PDFium
  reads for the page: `page.text().all()`, except on a page turned a quarter
  by `/Rotate`, whose text is the text inside the page's size as drawn
  (`inside_rect`). PDFium bounds `all()` by the size as displayed while the
  characters sit where they were drawn, so a landscape sheet stored as a
  portrait one lost every character past the displayed width. Blocks are
  built cheaply from that text, and each block's text is an exact stretch
  of the page text - from its first line's start to its last line's end,
  `\r\n` line ends and all - so a block can always be found in the page
  it came from. A line gets a box when the page's text objects fall into
  exactly as many lines as its text has, and is left without one otherwise.

Only the `layout` and `ocr_regions` routes read characters one by one
(four PDFium calls per character), which is why the router decides from
cheaper signals first. Inspection reads them while it has the page open,
as long as the runs it holds for the document stay under 100,000
(`MAX_DOCUMENT_RUNS`, 15 times the corpus's largest document); a page past
that has its characters read when it is read - once it is planned, or, for
an `ocr_regions` page, once its regions' readings are back - one page at a
time, at the cost of loading the page again (0.74 ms a page, median,
contended). Reading every page that way doubled the analysis time of the
layout-heavy documents (the annual report's 47 ms to 99 ms); reading
nothing ahead at all would hold no bound. A page with more runs than the
analysis takes on (below) has none read past that.

## Signals

All signals are integers (ratios per mille) and travel with the layout, so a
benchmark can say why a page went the way it did. They come from one walk of
the page's objects — the same walk that measured image coverage before — and
from the page text.

| Signal | What it is |
| --- | --- |
| `chars` | Non-whitespace, non-control, non-replacement characters of the native text. |
| `segments` | Text objects (PDFium's runs of one style), with their boxes, in content-stream order. A form XObject's text objects count one by one, each placed where the form draws it: the form's matrix, composed down any nesting of forms. |
| `image_coverage` | Per mille of the page covered by image objects, each at the size it is drawn, inside forms too. A form holding an image counts as that image, not as the form's box: a page wrapped in one form with a logo inside is a text page with a logo. |
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
| `rulings` | Thin path objects (and the sides of stroked rectangles): rules, inside forms too, placed where the form draws them. |
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

## What the fast route keeps

`scripts/compare-worker-pages.mjs --before OLD --after NEW FILE…` runs two
worker builds over the same files and fails if any page the new one read on
the fast route differs from the old one's in text, source, OCR confidence or
page-image flag, or if a document's outcome, page count, warnings,
truncation or page image changed. It was run with a worker built from the
commit before layouts (`9cd57a2`) against one built from `1ff92d8`, with the
same runtime (PDFium and Tesseract, and PP-OCR, which only the new worker
reads scans with):

* InternBench's 72 documents, every format: **102 of 102 fast-route pages
  byte-identical**, beside 174 pages read on the layout route, 53 by OCR
  and one `ocr_regions` page.
* The 21 generated fixtures (`fixtures/generated`): **133 of 133 fast-route
  pages byte-identical**, beside 6 OCR pages; the encrypted and malformed
  inputs fail as they did.
* What it reports is OCR's, as expected: the two-frame fax TIFF reads both
  frames (one before), and PP-OCR reads three scans at a confidence on the
  other side of the warning threshold from Tesseract's, so the
  low-confidence warning and the page image come or go with it
  (`scan-noisy-statement.pdf`, `mixed-signature.pdf`,
  `rotated-low-resolution-scan.png`). OCR page text is not compared: it is
  now the blocks' linearisation, and PP-OCR's.

The fast route's text is what PDFium reads, as above: `page.text().all()`,
or the text inside the size as drawn on a page turned a quarter (no page of
either set is both turned a quarter and on the fast route). What changed on
it is its blocks: each block's text is now the stretch of the page text it
came from, `\r\n` line ends and all, where it was its lines joined with
`\n`. Over both sets, every block of every page, on every route, is found
in its page's text in order; at `6f2bca9` that held on 17 of InternBench's
102 fast pages.

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
(`OcrBackend::concurrency`; for PP-OCR its configured workers, for Tesseract
half the logical cores, at most four). Rendering never runs further ahead
of OCR than the queue allows: the rendered pages held at once are the one
being rendered, at most `MAX_QUEUED_RENDERED_PAGES` (one) waiting, and one
per worker being read - six with four workers, about 450 MB at the
25-megapixel cap, and three with one. Pages are put back in order and every order-dependent decision (which
page becomes the page image, warning order, which error is reported) is the
one a sequential read makes; reading with one worker gives the same
document. Tesseract is started with `OMP_THREAD_LIMIT=1`: a Tesseract built
with OpenMP starts a thread per core in every process, and two pages at once
then took 128 s instead of 1.5 s on this machine. The optional page image,
whose pixels nothing reads, is a 256-pixel grey thumbnail.

## What a page may cost at most

The router and the layout analysis compare text with the text around it,
so a page built to be slow - thousands of one-character runs on a line,
tens of thousands of text objects, a page hundreds of inches wide - could
keep them busy for minutes. Each step is held to a bound
(`layout::router::bounds`) many times what any page of InternBench's 72
documents or the generated fixtures has. The calibration, rerun over their
442 pages (68 PDFs), measured the largest of each:

| Bound | Value | Largest measured |
| --- | ---: | --- |
| Text objects a page's survey keeps (`MAX_SEGMENTS`) | 5,000 | 226 (the ruled inspection log) |
| Images a page's survey keeps, the largest first (`MAX_IMAGES`) | 64 | 1 |
| Rules a page's survey keeps (`MAX_RULINGS`) | 2,000 | 300 (the inspection log) |
| Runs a page's layout is built from (`MAX_RUNS`) | 4,000 | 226 |
| Runs on one line, within five points down the page (`MAX_RUNS_PER_LINE`) | 250 | 8 |
| Runs inspection reads ahead for a document (`MAX_DOCUMENT_RUNS`) | 100,000 | 6,735 (the 100-page annual report) |
| Candidate gutters one region of a page tries (`MAX_GUTTER_CANDIDATES`) | 64 | 7 gutters on a page |
| Bands of whitespace counted in one part of a page (`MAX_GUTTERS`) | 64 | 7 |
| Points across a part of a page searched for gutters (`MAX_WIDTH_POINTS`) | 15,000 | a PDF page is at most 14,400 |

A page past a bound keeps its text and is read on the fast route. A page
with more text objects than the survey keeps is measured for no structure;
one with more runs than the analysis takes on has its characters read no
further than that and no geometry layout. Rows are built checking each run
only against the runs that start near it, two rows' free stretches
intersect in one pass, and gutter coverage is counted from where cells
start and end, so the rest grows about linearly with the runs. Between the
regions of a page the analysis asks whether the request was canceled or its
time is up; if so it stops, and the document fails as out of time rather
than coming back half-read.

On the calibration's pages (contended): the geometry analysis takes 64 µs
a page at the median and 524 µs at the most; the signals 222 µs at the
most; reading a page's characters 0.74 ms at the median.

## Known limits

* Only the OCR engine's lines are geometry on scanned pages (PP-OCR and
  Tesseract both fill `OcrResult::lines` in upright page pixels). An OCR
  page's text is its blocks' linearisation, built from those lines; a
  reading with no lines, or with more than the analysis takes on, keeps
  the engine's own text.
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

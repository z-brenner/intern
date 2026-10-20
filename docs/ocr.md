# OCR

A page with no usable text layer - a scan, a photo, an image file - is read
by an OCR engine in the parser worker. This page says which engines there
are, how one is chosen, what each does to a page, and what it costs. Every
number on it is measured; how to run the engines locally is at the end.

## Three engines, one chosen at run time

The worker reads scans with the first of three engines that is installed and
loads:

1. **Indra OCR** - Datalab's compact document OCR (`indra-ocr` 1.1.2): a line
   detector, a page orientation classifier, a recognizer with a character
   language model, and a reading-order and layout network, run through
   **ONNX Runtime**. The worker's reader is a Rust port of the reference
   pipeline (`crates/intern-worker/src/indra/`); no Python runs.
2. **PP-OCR** - a text-detection network, an English text-recognition
   network and a page-orientation classifier, run through the same ONNX
   Runtime.
3. **Tesseract** - as it has always been read, through `tesseract.exe` and
   its English and orientation data.

The Windows package carries all three. The worker looks for their files in
its runtime directory - `INTERN_RUNTIME_DIR`, or the directory it runs from -
exactly as it looks for PDFium:

| File | Engine | What it is |
| --- | --- | --- |
| `onnxruntime.dll` (`libonnxruntime.so` on Linux) | Indra, PP-OCR | ONNX Runtime 1.30.0, Microsoft's CPU build |
| `indra-models/det.onnx` | Indra | the line detector |
| `indra-models/orient.onnx` | Indra | the page orientation classifier |
| `indra-models/rec.onnx` | Indra | the Latin, Cyrillic and Greek line recognizer |
| `indra-models/order.onnx` | Indra | the reading-order and layout network |
| `indra-models/lm.npz` | Indra | the recognizer's character language model |
| `indra-models/meta.json` | Indra | the recognizer's vocabulary and the networks' sizes |
| `ocr-models/text-detection.onnx` | PP-OCR | the text-line detector |
| `ocr-models/text-recognition.onnx` | PP-OCR | the English line recognizer; its character list is compiled into the worker |
| `ocr-models/page-orientation.onnx` | PP-OCR | the page orientation classifier (optional) |
| `tesseract.exe`, `tessdata/` | Tesseract | Tesseract 5.5.2 and its English and orientation data |

Indra is chosen when ONNX Runtime and all six of its files are there and it
loads; PP-OCR when Indra's files are not there, or Indra does not load, and
ONNX Runtime and PP-OCR's detector and recognizer are there and load;
Tesseract otherwise. A runtime directory with none of an engine's files - a
development machine, a Linux CI runner - passes that engine over and says
nothing. One that has an engine's files and cannot load them, or has only
some of them (an antivirus quarantine, a damaged update), writes one warning
line to the worker's log for each engine passed over, naming the engine
given up on, the engine read with instead, and the missing file or the
error, never a document:

```json
{"level":"warning","code":"OCR_ENGINE_FALLBACK","from":"indra","to":"paddle","message":"Indra OCR is incomplete (C:\\...\\indra-models\\lm.npz missing); reading scans with PP-OCR"}
```

`from` and `to` are `indra`, `paddle` or `tesseract`. The choice is made
once, the first time a page needs OCR: a text PDF never loads an engine.

**Choosing an engine.** `INTERN_OCR_ENGINE=indra`, `paddle` or `tesseract`
starts the search at that engine instead, for comparing engines on a
benchmark and for support: `paddle` passes Indra over, `tesseract` passes
both ONNX engines over, and each falls back down the list as above. An
engine asked for and not installed is reported as a fallback. Unset, empty
or `auto` is the default; any other value is logged once as an
`OCR_ENGINE_UNKNOWN` warning and the default is used. The installed app
never sets it.

Both ONNX engines are built only with the `onnx-ocr` cargo feature, which
`windows-native` includes. Without it nothing of ONNX Runtime is compiled
in, and a runtime directory holding Indra's or PP-OCR's files reports a
fallback to Tesseract. With it nothing is linked or downloaded at build time
either: the `ort` crate (pinned `=2.0.0-rc.13`, default features off, `std`
and `load-dynamic`) loads the runtime once per process, shared by both
engines, from an absolute path in the runtime directory - never by bare
name, which Windows would resolve through a DLL search order that includes
directories anyone can write to. ONNX Runtime's telemetry is turned off.
Neither engine makes a network request: the models are read from the
runtime directory, and the page never leaves the worker.

## Indra OCR

### What happens to a page under Indra

Each step below is the reference package's, at the pinned code and model
revisions, with its constants; the worker's port is checked against the
reference package's own outputs on the same pages.

1. **Skew and orientation.** A small grey copy of the page is turned by
   candidate angles from -45 to 45 degrees, coarse to fine, and the angle
   whose rows of ink are sharpest is the page's skew. The orientation
   classifier looks at a 768 x 768 copy and names the quarter turn that
   rights the page; the turn is taken only when it is more than 95% sure.
   The page is turned and levelled before anything is read; the boxes the
   reading reports are in the upright page, not levelled.
2. **Line detection.** The page, its longer side scaled to 1,024 pixels, is
   given to the detector as an ink map. For every pixel it scores whether
   it is the core of a text line and how far the line's top and bottom are;
   each connected core becomes one line's box. A page whose lines come out
   under 9 pixels tall is detected again at up to 2,048 pixels, so small
   print is not lost.
3. **Recognition.** Each line is cut out of the full page with a window
   three line heights tall - the recognizer sees the lines above and below,
   which tells it the line's size and baseline - and read in batches of
   lines of similar width. The recognizer reports, for every time step, its
   best character and that character's probability, and its eight best
   candidates. A line's confidence is the mean probability of the
   characters it emits; a line under 0.5 keeps its box and has no text.
4. **Language model.** A line the recognizer is unsure of - confidence from
   0.5 up to 0.9 - is decoded again by a beam search over its eight best
   candidates at every step, scored by a five-character back-off language
   model (`lm.npz`), so a reading that spells a word is preferred over one
   that does not. The line's confidence stays the recognizer's.
5. **Reading order and layout.** The lines are split where a wide gap or a
   column gutter runs through them, and a list marker read as a line of its
   own is joined to its item. The reading-order network takes sixteen
   numbers a line - where it is, its size, and a few things about its text
   - and scores every pair of lines for which comes first and whether the
   two share a block, and labels each line with one of ten block classes:
   text, section header, page header, page footer, list group, table,
   caption, footnote, table of contents and form. Lines are clustered into
   blocks, the blocks put in order, and the lines inside each block put in
   order - top to bottom for tables, forms and tables of contents, by
   cutting the block into columns and bands otherwise.

The reading comes back as an `OcrResult`: the page's text, every line
(`OcrLine`: text, box `[x0, y0, x1, y1]` in the upright page's pixels,
confidence 0-100), and the blocks (`OcrBlock`: label, box, and the indices
of its lines in reading order).

**From Indra's blocks to Intern's.** A page Indra read keeps Indra's order:
its blocks as it orders them, and its lines as it orders them inside each.
Its labels become block kinds; nothing is re-derived from the geometry that
Indra has already decided:

| Indra label | Intern block |
| --- | --- |
| Text | Paragraph; KeyValue when every line of it is a labelled value |
| Section header | Heading, its level from how much larger than the body text it is |
| Page header, page footer | PageHeader, PageFooter |
| List group | one ListItem a list item: an item starts at the group's first line and at each line that starts with a bullet, `1.`, `(2)`, `a)` or `(b)` |
| Table | Table, with its rows and cells (below) |
| Caption | Caption |
| Form, footnote | KeyValue for each stretch of labelled values (`Invoice date: 2026-04-01`, or `Bill to:` with its value on the lines after it), Other for the rest |
| Table of contents | Other |

A table block becomes a table with rows and cells. When the layout
analysis that native pages get reads the block's lines as one table, its
table is kept: it knows wrapped cells, header rows, and a header over one
row of values as labelled values. Otherwise the block is assembled as
Indra assembles it: lines grouped into rows by their vertical centres, a
row closer under the one above than the table's rows are to each other
joined to it as a wrapped cell, columns anchored on the rows with the most
cells, and each line placed in the columns it covers more than half of. A
first row that is one line across most of the table is its title, a
Caption before it; the first remaining row is the header. A cell across
several columns keeps them, empty, so the cells after it stay under their
headings; a line that starts in a column such a cell covers joins that
cell, where Indra's own table drops it. A table of one line is a paragraph.

Everything else is as for any page read by OCR: confidences are the lines'
own, the source is OCR, boxes are in tenths of a point, a small block at the
very top or bottom of the page is a running header or footer, read first or
last, running headers and footers are marked across the document, headings
open sections, blocks are numbered in reading order, and the page's text is
its blocks in order. A reading whose blocks do not account for every line
with text - a line in two blocks, in none, or not there - is laid out from
its lines as PP-OCR's and Tesseract's are, by the layout analysis.

Indra's lines carry no per-character positions into the layout, so a line
holding two table cells closer together than the line splitting's gap stays
one cell.

## Indra OCR measurements

The measurements of Indra OCR on InternBench's scans and the research sets,
against PP-OCR and Tesseract on the same pages, follow in the table below.

| Engine | CER | WER | Dates | Names | Identifiers | CPU s | Wall s | Peak RSS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

## PP-OCR

PP-OCR was the engine for scans before Indra, and is the engine the worker
reads with when Indra's files are not installed or do not load.

### What happens to a page under PP-OCR

1. **Orientation.** The classifier looks at a 224 x 224 thumbnail and names
   the quarter turn that rights the page; the page is read at that turn.
   Every reading still has to convince on its own terms - at least three
   words at a confidence of 75 or more, the same bar the Tesseract path
   uses - and the search over the other turns runs if it does not. A page
   the classifier gets wrong is caught that way. So is the failure PP-OCR
   has and Tesseract does not: a page lying on its side reads as confident
   text, because a line taller than it is wide is read as vertical text.
   The detector's boxes give it away - most of the text runs down the
   page - and the search turns it a quarter. Without the classifier the
   page is read upright first and turned only when that reading is
   unconvincing.
2. **Detection.** The page is shrunk so its longer side is 960 pixels and
   the detector marks every text line on it. Each line becomes the smallest
   rectangle, at any angle, around its blob, grown back to the ink the
   detector was trained to shrink it from (the reference DB
   post-processing, reimplemented in `paddle/db.rs`).
3. **Skew.** The page's skew is the length-weighted median slope of its long
   lines. Each line is cut out along its own slope, so a crooked page reads
   as well as a level one; only the boxes the reading reports are levelled.
   Past 10 degrees the page is turned level and detected again.
4. **Recognition.** Each line is cut out of the full-resolution page,
   48 pixels tall, and read in batches of six lines of similar length. CTC
   decoding gives the text and a probability for every character.
5. **Second reading.** A line whose least certain character is under 0.9
   and that holds a critical field - a date, an amount, an identifier, a
   capitalised name of two or more words - or whose date or amount is
   damaged however sure the recognizer was (`0.7/01/2026`, `$25,o00`,
   `June 31, 2026`), is cut out again with more margin, made black and
   white at Otsu's threshold, and read again. At most 12 lines a page. The
   second reading replaces the first only if it reads the same line
   (within a quarter of its characters), loses no date or amount and
   damages none; one that repairs a damaged field wins outright, otherwise
   the more confident reading does.
6. **Output.** The page's text, lines that share a row joined with a space,
   rows with a newline, and a gap taller than a line as a blank line - the
   shape Tesseract's text has, which distillation finds paragraphs and
   headings in. The reading also carries every line (`OcrLine`: text, box
   `[x0, y0, x1, y1]` in the upright page's pixels, confidence 0-100) for
   the layout analysis to build blocks from.

**Confidence** is the mean, over the page's words, of each word's least
certain character, times a hundred. A word is as wrong as its weakest
character, and that is how Tesseract's word confidences behave too, so the
75 bar keeps its meaning across the engines. Real readings score 92-100;
the same pages read upside down score 23-46. The recognizer's own line
score, the mean over every character, crowds every real reading into
98-100 and could not tell them apart.

**Threads and memory.** Each page's networks use two intra-op threads. On a
machine with eight or more logical cores a second page can be read at the
same time, with its own sessions, so OCR never uses more than half the
cores and never more than two pages' worth of memory - about 570 MB each
at their peak. A page that arrives while every set of sessions is busy
waits for one. A document's pages share their sessions, and ONNX Runtime's
arena keeps each page's buffers for the next run instead of allocating them
again. When the document has been read, the worker drops the sessions,
arena and all, and the next scanned document builds them again. The worker
outlives every document and waits for the next one beside the model server,
which by then is reading this one. Measured on InternBench's 20 scans on a
quiet 4-core machine, alternating:

| Sessions | OCR, 20 scans | Worker idle after a 10-page scan | Worker peak while reading |
| --- | ---: | ---: | ---: |
| dropped after each document (shipped) | 126 s | 96 MB | 660-840 MB |
| kept, arena on | 122 s | 527 MB | about 740 MB |
| kept, arena off | 151 s | 156 MB | 430-480 MB |

Without the arena, OCR takes a fifth longer, and the dense forms and the
10-page scan take half as long again. Rebuilding the sessions for each
document costs about 3%. A second set of sessions
that cannot be built - memory is short - makes the page wait for the first
instead of failing it, and a run that fails on networks that loaded is
reported as worth trying again. Lines are cut into crops one batch at a
time, so a page of a thousand long lines never holds them all, and a page
many times longer than wide shows the orientation classifier only its
central square.
Spinning threads are off: between these networks' short layers they would
burn a core the user is working on.

**Determinism.** Same page, same configuration, same output - across runs,
across processes, and across pages read at once: every benchmark run below
that differs only in threads or pages at once produced byte-identical
text. `readings_are_deterministic_across_runs_and_threads` holds it.

### PP-OCR measurements

Measured on the 13 scanned InternBench documents (46 pages: 300 DPI office
scans, a 25-page 200 DPI lease, turned 90 and 180 degrees, 3 degrees of
skew, a 100 DPI receipt, speckle and blur, faint light, a fax, a dense
intake form), scored by intern-bench's own OCR measure against the text
drawn on each page. Two research sets back it up: 30 synthetic business
pages in clean, scanned and fax quality (dates, amounts and identifiers
scored as exact strings), and 50 FUNSD form scans (noisy, low resolution,
order-free word scoring). FUNSD is a research set: it was used for this
measurement and is not committed. intern-bench's OCR measure folds
typographic quotes and apostrophes (`‘ ’ “ ”`) to `'` and `"` on both
sides before comparing: PP-OCR's recognition dictionary has no curly
quotes, so it emits every one straight, and a quote's glyph style carries
no filing information. Nothing else is folded - a misread such as
`0ccurrence` still counts. The tables below were measured before the fold,
so their PP-OCR word error rates include words that differ only by a
straightened quote.

Each scanned page was rendered once through the worker's own PDFium path
(300 DPI, as OCR receives it) and the same pixels were handed to every
configuration, one configuration at a time, through the engines' own
`OcrBackend` implementations. InternBench's own runs score the shipped
worker end to end.

**Timings are from a machine shared with other work** and say only how
configurations compare when run back to back. CPU is the more trustworthy
of the two. They should be measured again on an idle machine before
anyone quotes an absolute speed.

#### Engines

InternBench, 46 pages. CPU and wall are per page; PP-OCR on 2 threads,
Tesseract on 1.

| Engine | CER | WER | Dates | Names | Identifiers | CPU s | Wall s | Peak RSS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Tesseract (today) | 2.50% | 3.79% | 27/30 | 27/27 | 12/15 | 1.10 | 1.14 | 57 MB |
| **PP-OCR, detector 960** | **0.50%** | **1.83%** | **29/30** | **27/27** | **14/15** | 2.96 | 1.65 | 593 MB |
| PP-OCR, detector 1280 | 0.49% | 1.82% | 29/30 | 27/27 | 14/15 | x1.10 | | 755 MB |
| PP-OCR, detector 1600 | 0.49% | 1.83% | 29/30 | 27/27 | 13/15 | x1.20 | | 998 MB |
| newer tiny detector, 960 | 0.50% | 1.93% | 29/30 | 27/27 | 14/15 | 2.87 | 1.59 | 491 MB |
| newer small detector, 960 | 0.51% | 1.96% | 29/30 | 27/27 | 14/15 | 3.09 | 1.75 | 603 MB |

The 1280 and 1600 rows were measured in an earlier, busier period; their
cost is given relative to the 960 run of that period. "Newer" detectors are
the next generation's tiny (1.8 MB) and small (9.9 MB) text detectors, each
with the same English recognizer; the newer generation's own recognizers
merge widely spaced table columns into one token ("13.14100,436.73") and
were not considered.

Per document, CER and critical fields found:

| Document | Tesseract | PP-OCR 960 |
| --- | --- | --- |
| skewed notice (3 degrees) | 64.56% (4/6) | 0.00% (6/6) |
| rotated 90 degrees | 0.00% (5/5) | 0.00% (5/5) |
| upside down | 0.00% (5/5) | 0.11% (5/5) |
| 100 DPI receipt | 15.85% (6/6) | 12.36% (6/6) |
| speckle and blur | 38.30% (6/6) | 1.28% (6/6) |
| faint light | 13.39% (5/5) | 0.49% (5/5) |
| fax, two frames | 0.16% (3/6) | 0.00% (4/6) |
| intake form | 0.85% (8/8) | 0.34% (8/8) |
| mixed amendment, scanned page | 2.29% (6/6) | 2.50% (6/6) |
| clean 2-page lease | 0.79% (6/6) | 0.79% (6/6) |
| 10-page agreement | 0.12% (6/7) | 0.42% (7/7) |
| 25-page lease, 200 DPI | 0.08% (6/6) | 0.30% (6/6) |

Tesseract drops three paragraphs of the skewed notice and most of the
noisy statement; PP-OCR reads both. On clean pages the two are close, and
Tesseract keeps punctuation slightly better. The receipt's error is
entirely its rules of dashes, which neither engine reads; every word and
figure on it is right. The fax's missing fields are on its second frame,
which the worker does not read.

Synthetic business pages (30), exact-string recall:

| Engine | Word F1 | Dates | Amounts | Identifiers | Fax tier: dates / amounts / identifiers |
| --- | --- | --- | --- | --- | --- |
| Tesseract | 0.946 | 0.867 | 0.855 | 0.850 | 0.600 / 0.600 / 0.575 |
| PP-OCR, detector 960 | 0.982 | 0.944 | 0.968 | 0.933 | 0.833 / 0.916 / 0.825 |
| **PP-OCR 960, binarised second reading (shipped)** | **0.983** | **0.989** | **0.970** | **0.942** | **0.967 / 0.929 / 0.850** |
| PP-OCR, detector 1600 | 0.979 | 0.922 | 0.970 | 0.942 | 0.767 / 0.916 / 0.850 |
| newer tiny detector, 960 | 0.979 | 0.956 | 0.959 | 0.917 | 0.867 / 0.884 / 0.775 |
| newer small detector, 1600 | 0.972 | 0.911 | 0.948 | 0.933 | 0.733 / 0.862 / 0.825 |

FUNSD forms (50), order-free word scoring:

| Engine | Word F1 | Word recall | Digit-word recall | CPU s |
| --- | --- | --- | --- | --- |
| Tesseract (2 threads) | 0.535 | 0.475 | 0.225 | 2.78 |
| PP-OCR, detector 960 | 0.777 | 0.750 | 0.453 | 2.81 |
| PP-OCR 960, binarised second reading (shipped) | 0.776 | 0.749 | 0.455 | 3.49 |
| PP-OCR 960, second reading both ways | 0.778 | 0.751 | 0.457 | 4.08 |
| PP-OCR, detector 1280 | 0.783 | 0.759 | 0.454 | 2.91 |
| newer tiny detector, 960 | 0.786 | 0.768 | 0.451 | 2.70 |
| newer small detector, 960 | 0.782 | 0.760 | 0.442 | 2.78 |

FUNSD's forms are small (about 750 x 1000 pixels) and noisy, and its
answers are words in any order, so it rewards finding every scrap of text
more than reading a date exactly. The newer tiny detector finds about one
word in a hundred more there; on the business pages, where the critical
fields are, it reads fewer of them (identifiers 0.917 against 0.933, and
the fax tier's amounts 0.884 against 0.916).

#### Second reading

At the 960 detector. Critical fields are InternBench's dates / names /
identifiers found, of 30 / 27 / 15; the synthetic-page columns are dates /
amounts / identifiers found, of 90 / 634 / 120 (fax tier: of 30 / 225 /
40). CPU is InternBench / synthetic pages, relative to no second reading
in the same period.

| Second reading | InternBench CER | WER | Critical fields | Synthetic pages | Fax tier | CPU |
| --- | --- | --- | --- | --- | --- | --- |
| none | 0.50% | 1.83% | 29 / 27 / 14 | 85 / 614 / 112 | 25 / 206 / 33 | |
| as first written (unguarded), both preprocessings | 0.63% | 1.94% | 28 / 27 / 14 | | | +41% |
| contrast-stretched and binarised | 0.48% | 1.74% | 29 / 27 / 14 | 87 / 620 / 113 | 27 / 213 / 34 | +38% / +42% |
| contrast-stretched only | 0.49% | 1.80% | 29 / 27 / 14 | 87 / 620 / 113 | 27 / 211 / 34 | +22% / +22% |
| **binarised only** | **0.49%** | **1.78%** | **29 / 27 / 14** | **89 / 615 / 113** | **29 / 209 / 34** | **+20% / +22%** |
| both, least certain character under 0.8 | | | | 87 / 621 / 113 | 27 / 212 / 34 | +29% synthetic |
| both, at most 6 lines a page | | | | 86 / 619 / 113 | 26 / 211 / 34 | +23% synthetic |

The first version of the pass made InternBench worse. A full stop after a
date (`by 09/30/2026.`) made the date look broken, so its line was re-read,
and four characters of noise replaced it: holding no date, they held no
wrong date. Readings that turned `$6.85` into `S6.85` and `$145.00` into
`$145.0` won on a higher mean probability. The guards described above -
same line, no field lost or damaged, amounts checked as dates are - fixed
both, and the pass then repaired `0.7/01/2026`, `$25,o00` and `Ml 49101`
without breaking anything else measured.

How well the pass picks its lines, on InternBench's first readings: of the
59 lines that hold a critical field and are not read exactly, the 0.9 bar
with at most 12 lines a page chooses 53, among 306 chosen in all (6.7 a
page). A bar of 0.8 chooses 52 of 271; 0.7, 47 of 226; 0.5, 12 of 23.

On FUNSD's forms the second reading changes next to nothing: word F1
0.776 binarised and 0.778 both ways, against 0.777 without, digit-word
recall 0.455 and 0.457 against 0.453, for 24% and 45% more time. Its
answers carry no dates or amounts to check, and its noise is in the
detection, not in a character here and there.

#### Orientation

The classifier named the right turn for every InternBench page in all
four turns (184 of 184), every synthetic page (120 of 120) and 199 of 200
FUNSD pages; the one miss was read upright, found
unconvincing and searched. On InternBench's two turned documents it saves
the passes the search would spend:

| Document | Classifier first | Upright first, no classifier | Tesseract |
| --- | --- | --- | --- |
| rotated 90 degrees | 1 pass, 0.98 s | 2 passes, 1.86 s | 1 pass, 0.70 s |
| upside down | 1 pass, 1.37 s | 4 passes, 6.08 s | 2 passes, 2.33 s |

Both strategies read both documents exactly as well. The classifier is
6.8 MB, and it ships.

#### Skew

The skewed InternBench notice (3 degrees) reads perfectly. To find where
levelling starts to matter, five InternBench pages (the agreement's first
page, the lease's third, the faint letter, the noisy statement, the intake
form) were turned 2, 4, 6 and 10 degrees and read three ways:

| | 2 degrees | 4 degrees | 6 degrees | 10 degrees | CPU, 20 pages |
| --- | --- | --- | --- | --- | --- |
| never level and detect again | 0.99% | 0.70% | 0.66% | 0.67% | 62.4 s |
| level and detect again past 5 degrees | 0.99% | 0.70% | 0.64% | 0.71% | 73.0 s |
| level and detect again past 2 degrees | 0.68% | 0.72% | 0.64% | 0.71% | 78.2 s |

Levelling a page and detecting it again cost about a quarter more time on
each page it was done to. Unturned, the same pages read at about the same
error. All of the difference at 2 degrees is one noisy page (3.62% against
1.60%), and levelling was worse at 4; every critical field was within one
either way. Cutting each line out along its own slope is enough, so the
page is levelled and detected again only past 10 degrees, the largest
angle measured.

#### Threads and pages at once

PP-OCR with the 960 detector over InternBench's 46 pages, run back to
back:

| Pages at once x threads each | Whole set, wall | Per page, wall | CPU, whole set | Peak RSS |
| --- | --- | --- | --- | --- |
| 1 x 1 | 134.7 s | 2.91 s | 133.6 s | 593 MB |
| **1 x 2** | **77.0 s** | **1.65 s** | **137.1 s** | **593 MB** |
| 2 x 1 | 69.4 s | 2.93 s | 134.9 s | 1,148 MB |
| 2 x 2 | 42.8 s | 1.80 s | 145.8 s | 1,159 MB |

Two threads read a page 1.8 times as fast as one for 3% more processor
time. Two pages at once on one thread each are only 10% quicker over the
set than one page at a time on two, take as long again over each page, and
double the memory: each set of sessions peaks at about 570 MB, on a batch
of full-width lines from a 300 DPI page. So each page gets two threads,
and a machine reads as many pages at once as half its cores allow, but
never more than two: on a 4-core machine one page at a time, on 8 or more
two. The output is the same in every row.

#### Model bytes

| File | Bytes |
| --- | --- |
| text detector (shipped) | 4,826,518 |
| English recognizer (shipped) | 7,848,423 |
| orientation classifier (shipped) | 6,788,069 |
| ONNX Runtime 1.30.0 DLL (shipped) | 16,462,648 |
| newer tiny detector (not shipped) | 1,780,590 |
| newer small detector (not shipped) | 9,880,512 |

#### Decision (PP-OCR against Tesseract)

**PP-OCR replaces Tesseract as the engine for scans**, with Tesseract as
the fallback. On InternBench it finds 29 of 30 dates and 14 of 15
identifiers to Tesseract's 27 and 12, and all 27 names as Tesseract does,
at a fifth of its character error; on the business pages it finds 89
dates of 90 to Tesseract's 78 and 113 identifiers of 120 to its 102; on
FUNSD's forms it finds half again as many words. It costs more: about 2.7
times Tesseract's processor time per page before the second reading and
3.2 times with it, and 0.6 GB of memory while a page is read, against
57 MB.

Tesseract is not used as a second voter on cropped fields. On InternBench
it found no field PP-OCR missed. On the business pages it found 12 that
PP-OCR missed - 8 amounts and 4 identifiers, no dates - against 107 the
other way. A vote could win some of those 12 back at the price of a
Tesseract process per crop; it is the next thing to measure if amounts
come to matter.

The shipped configuration:

| Setting | Value | Why |
| --- | --- | --- |
| text detector | the mobile detector of the generation before the newest | best critical fields on InternBench and the business pages; the newer tiny detector is 3 MB smaller and a little cheaper, and reads fewer identifiers and fax-tier amounts |
| recognizer | English, same generation | the newer generation's recognizers merge widely spaced table columns into one token |
| detector input | 960 on the longer side | 1280 and 1600 find nothing more, cost 10-20% more, and 30-70% more memory |
| orientation | classifier first, then the search | one pass for a turned page instead of two to four |
| levelling | past 10 degrees only | lines are cut along their own slopes; levelling bought nothing up to 10 |
| second reading | binarised, least certain character under 0.9, at most 12 lines a page | most dates for the time; +20% |
| threads | 2 a page; pages at once up to half the cores, at most 2 | 1.8 times as fast as one thread; memory caps the pages at once |
| recognition batch | 6 lines | reading lines one at a time drops trailing full stops and commas at the end of long lines |

Shipped together: InternBench CER 0.49%, WER 1.78%, dates 29/30, names
27/27, identifiers 14/15, 3.54 s of processor time and 1.95 s wall a page
on two threads (contended), 599 MB peak.

## Packaging

`src-tauri/resources/runtime-assets.json` pins every file by URL, size and
SHA-256, and `scripts/fetch-windows-assets.ps1` fetches and verifies each:

* **ONNX Runtime**: the `Microsoft.ML.OnnxRuntime` 1.30.0 NuGet package
  from `api.nuget.org/v3-flatcontainer` (157 MB, every platform's build).
  Only `runtimes/win-x64/native/onnxruntime.dll`, `LICENSE` and
  `ThirdPartyNotices.txt` are taken out of it, and the DLL is checked
  against its own size and SHA-256 as well as the package's. It is staged
  beside the executables like the other sidecar DLLs. It needs the Visual
  C++ runtime (`MSVCP140.dll`, `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll`),
  as `llama-server` does.
* **Indra OCR's models**: `det.onnx`, `orient.onnx`, `rec.onnx`,
  `order.onnx`, `lm.npz` and `meta.json` from
  `huggingface.co/datalab-to/indra-ocr`, every one pinned to commit
  `a1283c4b42a273c09480ad50e71461daff50fcc7` (the `v1.1.0` tag the
  `indra-ocr` 1.1.2 package pins), staged under `indra-models/`. Only the
  Latin set ships: not the multilingual recognizers and language models
  (`script.onnx`, `rec_cjk`, `rec_rtl`, `rec_world`, `lm_cjk`, `lm_world`),
  and not `model.safetensors`, the recognizer's float weights, which
  nothing reads. The bundle takes `.npz` and `.json` files from
  `indra-models/` and nowhere else.
* **PP-OCR's models**: the official ONNX exports on Hugging Face, each
  pinned to a commit (`resolve/<sha>/inference.onnx`), staged under
  `ocr-models/`.
* **Licences**: ONNX Runtime's MIT license and third-party notices; the
  Apache-2.0 text from the PaddleOCR repository at its v3.7.0 tag (the
  PP-OCR model repositories carry no license file of their own); Indra's
  model license (a modified AI Pubs Open RAIL-M license) from the same
  model commit, as `licenses/indra-ocr-models-LICENSE.txt`; and the
  Apache-2.0 license of the Indra code from `github.com/datalab-to/indra`
  at commit `0e96b93c6810f17cf8352acbb4487cc9e45b66d5`, as
  `licenses/indra-LICENSE.txt`. `THIRD_PARTY_NOTICES.md` lists every file
  and summarises the terms of Indra's model license, which differ from the
  other components': read the license itself before shipping or using the
  models.

`scripts/verify-assets.mjs` holds the pins, including the DLL's, and refuses
a bundled OCR file whose digest is not its download's pin. It refuses an
inventory that carries part of PP-OCR's runtime without the rest, or part of
Indra's models without the rest; Indra's models without `onnxruntime.dll` or
without both of their licences; and a release inventory without all of
both engines. The installer smoke requires every one of these files. The
worker smoke fails on a runtime that carries part of Indra's models, and on
any `OCR_ENGINE_FALLBACK` line from a runtime that carries ONNX Runtime, so
a package built with Indra must read its scans with Indra. QA and release
run each ONNX engine's own tests against the staged runtime:
`INTERN_REQUIRE_INDRA_OCR=1 cargo test -p intern-worker --test indra_ocr
--features windows-native` and the same with `INTERN_REQUIRE_PP_OCR=1` and
`--test paddle_ocr`.

Indra's six files are 24,684,875 bytes, and its two licences 26,108 more:
the install grows by 24.7 MB (23.6 MiB) over a package with PP-OCR, whose
files grew it by 36.3 MB (34.6 MiB): 16.5 MB of runtime, 19.5 MB of models
and 0.36 MB of licence text. The installer compresses with LZMA; Indra's
files compressed with LZMA (`xz -9`) come to 22.7 MB (21.7 MiB) - its
language model is compressed already - and PP-OCR's with the runtime to
22.3 MB (21.2 MiB), which is about what the installer grows by. The
integrator's build will say exactly.

## Running it locally

On Linux x86-64, `scripts/fetch-ocr-runtime.sh RUNTIME_DIR [CACHE_DIR]`
stages ONNX Runtime 1.30.0 (from its PyPI wheel, pinned by size and
SHA-256, and the library inside checked against its own digest), Indra's
six files and the three PP-OCR models (by the same pins as the Windows
package), and Indra's two licences, into a runtime directory. Add PDFium
and Tesseract as `docs/evaluation.md` describes, and point
`INTERN_RUNTIME_DIR` at it. Then:

```sh
cargo test -p intern-worker --features native-pdfium,onnx-ocr --test indra_ocr
cargo test -p intern-worker --features native-pdfium,onnx-ocr --test paddle_ocr
```

reads the generated fixtures through the real engines (`npm run fixtures`
first). Those tests return early where the runtime is absent;
`INTERN_REQUIRE_INDRA_OCR=1` and `INTERN_REQUIRE_PP_OCR=1` make its absence
a failure. The Indra tests also compare the port with the reference
package's own output (`indra-ocr` 1.1.2, Latin recognizer only): for
Intern's fixtures against output committed under
`crates/intern-worker/tests/fixtures/indra/`, and for Indra's example pages
against a directory of the reference package's output that
`INDRA_REFERENCE_DIR` names, with the pages themselves in
`INDRA_INPUTS_DIR` (by default `indra-inputs` beside it). Without
`INDRA_REFERENCE_DIR` those comparisons are skipped.
`bench/tools/indra/README.md` says how the reference output is made. (On Linux,
`windows-native` also builds the Tesseract path, which needs Tesseract's
libraries; `native-pdfium,onnx-ocr` does not.)

To read a benchmark with one engine and then another, set
`INTERN_OCR_ENGINE` for the worker: `indra`, `paddle` or `tesseract`
([InternBench](internbench.md)).

Everything in `crates/intern-worker/src/indra/` but its engine - image
operations, detection decoding, cropping and batching, CTC decoding, the
language model and beam search, reading order, block and table assembly -
and everything in `crates/intern-worker/src/paddle/` but its engine is plain
Rust over pixels and probabilities, tested on every platform without the
runtime.

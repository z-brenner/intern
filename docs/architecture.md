# Intern architecture

Intern turns a document into a filename, a one-sentence description, and the
evidence behind both. This document explains how, why the pieces are shaped the
way they are, and what the design costs on an ordinary Windows laptop.

Everything below runs locally. The engine speaks only to `127.0.0.1`.

## The stages

```text
DocumentSource ─▶ distill ─▶ prompt ─▶ one local inference ─▶ validate ─▶ compose name
```

There are five stages and no decision tree. Each has one job:

| Stage | Input | Output | Where |
| --- | --- | --- | --- |
| Extract | a file path | pages of text or Markdown | `intern-worker` (separate process) |
| Distill | pages | a verbatim digest under a character budget | `intern-engine::distill` |
| Prompt | digest | one user turn plus a GBNF grammar | `intern-engine::prompt` |
| Infer | prompt | one JSON reply | `intern-engine::client` + llama.cpp |
| Validate | reply + digest | checked facts and review reasons | `intern-engine::validate` |
| Name | checked facts | a Windows-safe filename | `intern-engine::naming` |

`intern-queue` decides *which* document runs and what happens to the result;
`intern-core` makes the state and the file operations survive a crash. Neither
knows anything about models.

## Extraction

Native text first, always. PDFium supplies each page's text and how much of the
page is covered by images. A page goes to OCR only when it has fewer than 20
meaningful characters under heavy image coverage, when it has fewer than 200 on
a page that is essentially all image — a scan whose text layer is a Bates
number or a "CONFIDENTIAL" stamp and nothing else — or when more than 3% of its
characters came back as replacement glyphs. Word processing and presentation
files — `.docx`, `.docm`, legacy `.doc`, `.rtf`, `.odt`, `.pptx`, `.pptm`,
`.ppsx`, legacy `.ppt`, `.odp` — go through AnyDoc to Markdown, which preserves
headings and tables, and only when the container's content is not something
else: routing here is by extension, so a workbook renamed `.docx` — which
AnyDoc would otherwise render through its uncapped Excel path — is a routing
failure for review rather than a document. Content of the same kind as its
extension is read as what it is: Word has saved RTF under `.doc` for decades,
and a `.doc` that is really a Word 2007 package (or a `.pptx` that is really a
97-2003 deck) reaches no reader without the caps it would have had. A file
saved with a password to open is reported as `PASSWORD_PROTECTED` rather than
as damage: an encrypted Office package is an OLE compound file holding
`EncryptionInfo` and `EncryptedPackage` streams, recognised before the zip
pre-pass would have called it a corrupt archive; a legacy workbook carries a
`FilePass` record; and PDFium's password error on a PDF maps to the same code.
Every error message is one line, never a debug dump.

Plain text and Markdown are read directly, by byte-order mark: UTF-16 in
either order and a marked UTF-8 file all decode, and the mark itself never
reaches the text. Unmarked text that is not UTF-8 is, on the Windows machines
these files come from, almost always Windows-1252, and is read as that; only
a result holding C1 control characters — bytes Windows-1252 leaves undefined —
carries a corruption warning. A text file is never read past four times the
page cap in bytes, which always fills a page.

Workbooks — `.xlsx`, `.xlsm`, Excel 97-2003 `.xls`, and OpenDocument `.ods` —
are read sheet-per-page as Markdown tables, capped at 200 rows by 30 columns
per sheet with an elision marker so a large workbook cannot flood
distillation, and a cell's text at 1,000 characters. That window is a design
choice, marked where it applies, so it is reported as `CONTENT_ELIDED`, which
the host treats as a note rather than a reason for review: a ledger whose
facts sit in its first rows can be Ready however long it runs. Text that was
actually lost — a page cut at the size cap, unread TIFF frames — is still
`TEXT_TRUNCATED` and still forces review. A `.xlsx` is streamed cell by cell;
calamine reads a binary `.xls` whole as it opens it, so its record stream is
first surveyed for what that would allocate (dense ranges spanning a sheet's
corners, forged `Dimensions` claims, shared strings copied into every cell
that names them) and refused if calamine would hold more than 512 MiB at once,
counting the sheets already built and the one being built; a year of monthly
ledgers stays far inside that. Its formulas' token streams are then emptied
(only cached values are rendered, and spelling out a formula can be far longer
than its record), and it is handed to calamine in a fresh compound file. An
`.ods` is parsed by AnyDoc, whose OpenDocument reader charges repeated rows
and cells against a fixed expansion budget, and each sheet's table is then cut
to the same window rather than rendered whole. CSV exports are read through
the same window: the delimiter is the one of comma, semicolon, tab, and pipe
that splits the leading records most consistently, fields that are not UTF-8
read as Windows-1252, and a UTF-16 export is transcoded as it streams.

A page that has to be OCR'd is rendered at 300 DPI or, when that would pass
the 25-megapixel render cap, at the highest resolution that fits it. A phone
photo that some tool wrapped in a PDF at 72 DPI is a 4032 × 3024 point page,
about 212 megapixels at 300 DPI; it is rendered at about 103 DPI, which
Tesseract reads perfectly well, instead of failing the document. Only a page
that would have to go below 50 DPI to fit is a resource limit, and the size of
what was actually rendered is still checked against the cap. A standalone
image is decoded up to 100 megapixels — a phone's 48- and 50-megapixel modes
are ordinary — and up to 400 MB of decoded pixels, which holds a scanner's
16-bit colour mode to the memory an 8-bit image takes. It is scaled down to
the page cap before OCR by averaging the pixels each page pixel covers,
straight into the page-sized copy, and only then turned upright, so a
48-megapixel photo costs the worker about 220 MB at its peak rather than the
760 MB a filtered resample's floating-point intermediate took.

OCR text keeps the layout Tesseract found. The worker rebuilds it from
Tesseract's TSV output: words on a line joined with a space, lines with a
newline, and a new block or paragraph with a blank line, the way Tesseract's
own text output separates them. It used to be one line per page, which left
distillation no headings, no labelled lines, and no date lines on exactly the
documents where dates go missing; the scanned lease's evidence for its date
was its whole page.

A standalone image is OCR'd as one page; a TIFF holding a frame per page — a
fax, a batch scan — yields its first frame and reports the rest as truncated
rather than dropping them silently. `.eml` emails and Outlook `.msg` messages
emit a fixed-order header block followed by the body and a listing (never an
extraction) of attachments. The `Date:` line is the message's own `Date`
header, verbatim — for a `.msg`, the one in the transport headers it
travelled with — so the sent date is checkable against the document like any
other fact, in the sender's own offset. A `.msg` that never travelled (a
draft, a sent item) has only a UTC submit time, and is dated in this
machine's zone with its offset written out. No second, UTC rendering of the
date is added: for every evening email west of Greenwich it falls on the
next day, and validation would accept it because it is in the text. An HTML
body is read the way it renders: whitespace in its text collapses to a space
and only its structure starts a line, so neither Outlook's indented, wrapped
source nor a receipt laid out in one big table cell loses or gains a line.
Table cells are kept apart (` | `), comments and the head dropped (its title
kept as the first line), and named and hexadecimal entities decoded;
Outlook's binary HTML property is decoded from the hex
msg_parser hands over, by its declared charset. Strings an ANSI `.msg` stores
in its code page, which msg_parser drops when they are not UTF-8, are read
back and decoded in the code page the message declares.

Whatever the reader, a page carries at most two million characters and a
document eight million, and the host reads the worker's reply lines with a
64 MiB bound, failing a longer one as a crashed worker rather than allocating
it in the app's own process.

What "OCR only when necessary" means is enforced in code rather than
documented as intent:

* The OCR engine is constructed the first time a page actually needs it. A text
  PDF is never delayed by, and never fails because of, an OCR engine that is
  missing or slow to start.
* PDFium is bound once per process and shared. Binding it per document made
  every PDF after the first one in a queue fail as "native assets missing";
  `one_pdf_backend_parses_every_document_in_a_queue` keeps that fixed.
* A page is read as it came first, in grey: Tesseract binarises whatever it is
  given, and grey is a third of the bytes to encode for every pass. A reading
  of at least three words at a mean confidence of 75 or more is done — one
  recognition pass and no orientation detection — and so is a reading that
  found nothing, since a blank page, every other sheet of a duplex scan, is
  blank in every orientation. Orientation detection used to run first on every
  page, and on the corpus's upright lease it was confidently 180 degrees wrong
  and bought three more recognition passes to find the orientation the page
  already had. Reading upright first took that page from four passes to one
  and from 3.8 s to 0.6 s, and the median upright scan in the corpus from
  1.2 s to 0.6 s (whole document, Linux, Tesseract 5.3.4 on one thread).
* A page that does not read confidently asks orientation detection, on a
  half-scale copy, and is then re-read in the other orientations. Tesseract's
  orientation detection is trained on prose with ascenders and descenders; on
  a dense all-caps form it can be confidently 180 degrees wrong, and OCR then
  returns a full page of gibberish with the same word count and shape as a
  real reading. Volume cannot tell those apart, so mean word confidence
  arbitrates: one corpus page scored 23, 14, 14, and 76 across the four
  orientations. Confidence is a mean, though, so a reading only displaces
  another when it read a comparable amount; three confident tokens are not a
  better reading of a page than three hundred words just under the bar. The
  upright reading is one of the candidates, compared as it already came back
  rather than read again.

A PDF reports its progress as it goes: a `reading` event as each page is
reached and an `ocr` event as each goes to OCR, carrying how many pages are
finished and the page count, at most four of each a second; a standalone
image reports none of its one page finished as it goes to OCR. The window
shows that as a whole percentage. A 200-page scan used to sit at 0% until it
was done.

## Distillation

The model has a context window and a CPU budget; a 30,000-character contract
has neither. The old pipeline solved this by sending the first 14,000 characters
and the last 8,000 and discarding the middle. That throws away exactly the part
of a long agreement where its term, its fees, and often its effective date live.

Distillation instead reads the whole document:

1. **Segment.** Pages become blocks: headings, paragraphs, table groups.
   Paragraphs longer than 700 characters are split on sentence boundaries so a
   salient sentence can survive independently of the prose around it.
2. **Collapse running lines.** A short block whose digit-masked shape repeats on
   at least half the pages is a running header or footer; only its first
   appearance is kept.
3. **Score.** Each block is scored on cues that answer the three questions a
   filename needs: date cues and date-role phrases ("effective as of", "date of
   this notice", "invoice date"), party cues ("by and between", corporate
   suffixes, `To:`/`From:`), document-type cues, subject cues, signature cues,
   money, and identifiers. Standard clause bodies — governing law, severability,
   entire agreement, counterparts, and their relatives — are demoted hard.
   Position matters a little: the opening names a document and the closing signs
   it.
4. **Select.** Mandatory blocks first (the opening, anything carrying a date with
   a stated role, anything naming parties and a type, subject lines, signature
   blocks), then the highest-scoring remainder, until the budget is spent.
   A block whose text repeats one already kept (clause number aside) is never
   kept twice, and near-duplicate blocks — the same opening or, for long body
   text, the same closing 80 characters with digits masked — never compete
   for budget with text that appears once.
5. **Emit.** Kept blocks are written back **in document order**, with `[Page N]`
   markers, `[...]` where text was removed, a `SECTIONS:` outline of every
   heading found anywhere in the document, and an index of every sentence that
   carries a date. The date index is what turns "which of these dates defines
   the document" from a scanning problem into a reading problem; adding it took
   the corpus from 9 of 11 dates correct to 11 of 11, and eliminated the last
   two cases of filing a document under a referenced agreement's date.

Three properties are load-bearing and each has a test:

* **Nothing is unreachable.** `a_fact_buried_in_the_middle_of_a_long_document_survives`
  builds an eight-page agreement whose effective date is on page five and
  asserts it is in the digest.
* **Kept text is verbatim.** `distillation_never_invents_text` asserts every
  emitted segment is a substring of the source. This is what makes evidence
  checking meaningful.
* **The digest is deterministic.** The same document always produces the same
  digest, so a re-run is reproducible and a cached prompt prefix stays warm.

### Why not LLMLingua-2

LLMLingua-2 was the obvious candidate and was rejected for two reasons, one
practical and one fatal.

The practical one is deployment cost. It is a BERT-class token classifier: an
ONNX runtime plus a 400 MB–1 GB encoder, or a Python runtime, added to a product
whose entire point is to fit comfortably beside Windows on a 16 GB laptop. That
is a large fraction of the main model's footprint spent on preprocessing.

The fatal one is that it deletes tokens. Its output is a compressed token
sequence, not document text — which means no excerpt the model quotes can be
checked against the original, and Intern's anti-hallucination guarantee
disappears. It also, by construction, breaks the relationships the redesign
exists to preserve: "effective as of" and the date it governs can be separated.

Structure-aware extractive distillation gives the same compression on the
documents that matter, keeps text verbatim, costs no extra download, no extra
process, and no measurable memory, and runs in well under a millisecond. It is
implemented in Rust with no dependencies beyond the standard library.

### Budgets

| Source size | Behaviour |
| --- | --- |
| ≤ 12,000 characters | passed through untouched |
| > 12,000 characters | distilled to ≤ 12,000 characters |

Compression is therefore adaptive by construction: the ratio follows the
document rather than a configured number. A one-page invoice is untouched; a
15,000-character settlement agreement compresses 1.2×; a 29,000-character
statement of work compresses 2.2×; a 100-page journal whose pages differ only by
an observation number compresses 93×, to the four lines that actually differ.

The digest can overshoot the budget by one block when the mandatory set alone is
larger than the budget — dropping evidence to hit a round number would be the
wrong trade.

12,000 characters is roughly 3,000 tokens. It was chosen from measurement, not
taste: prefill on the target machine runs at about 160 tokens/second, so every
1,000 characters of budget costs about 1.5 seconds of wall clock on every
document. A larger budget buys nothing on the corpus and costs seconds per file.

"Roughly 3,000 tokens" holds for prose. Qwen's tokenizer reads every digit, and
every CJK, Hangul, or Kana character, as a token of its own, so a bank statement
or a Chinese contract inside the character budget can come to 8,000 tokens or
more and no longer fit the 8,192-token context. Before a prompt is sent the
engine estimates it - one token per digit or wide-script character, one per
three and a half characters of anything else - and when the estimate plus the
reply's 1,024 tokens passes 8,000 it distills again, scaled towards 6,500
tokens and condensed even if the source was small enough to pass through, at
most twice. If the server still answers that the prompt does not fit, the
document is condensed to half once more and sent once more; after that it fails
on its own as `MODEL_INPUT_TOO_LARGE`, without restarting the server. Only
prompts that did not fit change, so every recorded prompt is sent as it was.
The estimate guards the local server's context only: a hosted model's is many
times larger, so it is sent the digest whole, and condensed only if it answers
that the prompt did not fit.

The SECTIONS line that opens a condensed digest lists at most 40 headings in at
most 1,500 characters, ending in ` | …` when cut. A table row is never a
heading, however capitalised: a 600-row ledger used to put 600 of them there.

## The prompt and the grammar

One inference per document. The reply is constrained by a GBNF grammar, so
whole classes of mistake are impossible rather than filtered afterwards:

* `document_date` can only be `YYYY-MM-DD`.
* `date_role` has no "due", "deadline", or "renewal" member. The model has no
  vocabulary for a payment due date as the document's date; it can still pick
  one and call it something else, which validation catches (below).
* `parties` is capped at three entries.
* The reply contains no whitespace at all. Pretty-printing costs generated
  tokens, and generation is the slowest thing on a CPU.

The prompt teaches date *meaning* rather than a priority order: an agreement is
defined by its effective date, a notice by its notice date or by the termination
it brings about, an invoice by its invoice date, an amendment by its own date and
never by the date of the agreement it amends. A signature date loses to a stated
effective date.

Hybrid-reasoning models are switched out of thinking mode
(`chat_template_kwargs.enable_thinking = false`). Intern needs a form filled in,
not a chain of thought, and thinking tokens are pure latency here.

## Validation

The goal is calibration, not timidity. A proposal goes to review only when a
*specific* thing is wrong with it.

| Fact | Accepted when |
| --- | --- |
| Date | it is a real calendar date **and** is written, in some ordinary human form, in the document — `April 1, 2026`, `1st April 2026`, `01/04/2026`, `01.04.2026`, `4/1/26`, `01/04/26`, `01.04.26`, and their relatives, matched as whole tokens so `12/1/2026` never supports February 1. A two-digit year is read in either order; dotted or dashed, only padded, so a section number like `1.4.26` is never a date |
| Type | at least 60% of its significant words appear in the document |
| Party | the name appears in the document, verbatim or with punctuation disregarded (`Contoso Worldwide Inc` for a document that writes `Contoso Worldwide, Inc.`, and `&` read as `and`); the words themselves are never loosened. Two spellings of one name (`ACME CORP`, `Acme Corp.`) are one party, but a name is never merged into a longer one (`Acme`, `Acme Holdings`) |
| Description | one sentence, 6–42 words, and every number and capitalised name in it appears in the document, allowing a possessive, a thousands separator, a hyphen the sentence added, or a date the document states written another way (`January 5, 2026` for `01/05/2026`) |

The date rule is deliberately about the *date*, not about the model's quoted
line. Small models paraphrase their own quotes — answering
"This Agreement is effective as of February 14, 2025" for a document whose line
reads "Effective date: February 14, 2025". The first version of this validation
gated on the quoted wrapper and threw away correct dates on half the corpus. What
must be true is that the date is really in the document, and that is what is
checked. The model's quoted line is still stored and shown to the reviewer.

A date the document states is not yet the document's date, so four more
checks read the document around it:

* **Another document's date.** "the Master Services Agreement dated June 2,
  2023" dates somebody else's agreement. A date stated only that way is
  withheld, or replaced by the one date the document states on an effective
  or commencement line. The determiner on the nearest document noun decides:
  `This Consulting Agreement dated`, and a defined term like `(the
  "Agreement")` standing for it, are the document dating itself. Document
  nouns are whole words, plural and `sub-` forms included (`the Loan
  Agreements`, `the Subcontract`), so a contractor or a border is none. A
  citation (`issued under`, `pursuant to`) taints only a date it runs
  straight into, with no clause punctuation in between; a comma that only
  sets off the cited date (`the Master Services Agreement, as amended,
  effective June 2, 2023`) does not end it, while one followed by a clause
  of its own (`..., your employment will terminate effective`) does.
* **A deadline.** When every statement of the chosen date is labelled a
  deadline (`Due Date:`, `Payment due`, `Expires`, `Renewal Date`), the one
  date the document labels as its issue date (`Invoice Date:`, `Dated`, a
  bare `Date:` - not `Ship Date:` or `Order Date:`) replaces it; with none
  or several, the date is withheld for a person, and the model's date is
  offered to them. Either way the proposal goes to review with
  `DATE_IS_DEADLINE`. `payable` and `return` do not set it off, and a
  numeric date that reads either way round counts as two - unless the
  document's other numeric dates show which way round it writes them, as
  they do for the date chips.
* **An implausible year.** A year more than ten years ahead or before 1900 is
  usually an OCR misread the model copied faithfully (2625 for 2025). The
  date is kept and the proposal goes to review with `DATE_IMPLAUSIBLE`.
* **A date that reads either way round.** `04/01/2026` is 1 April in London
  and 4 January in New York, and both are dates the document prints. When
  the day and the month are both 12 or under and differ, the document
  writes the date only in numbers (never in words, never year first), and
  nothing settles the document's order - no other numeric date that can
  only be read one way round, like `30/01/2026` - the date is kept and the
  proposal goes to review with `DATE_AMBIGUOUS`. A date read against the
  order the document does show (`03/04/2026` as 4 March beside a
  `30/01/2026`) goes to review the same way.

All of this is string handling over text nobody controls. A panic in it -
one slice inside an `é` once panicked on every French invoice - fails that
one document as `ANALYSIS_FAILED`, with a fixed message that never carries
the document's text, instead of taking the model thread down with it.

Self-reported confidence below 0.60 also routes to review, as does any
fact-affecting parser warning, and any document with no defining date or no
specific type.

Three things are then decided from the document rather than from the model,
because a two-billion-parameter model is good at finding facts and poor at
labelling them consistently:

* **The date's role.** The wording in the ninety-odd characters before the
  validated date decides whether it is an effective, execution, invoice,
  notice, termination, amendment, filing, or issuance date - `Effective as
  of`, `Invoice date`, `Notice is hereby given ... on`, `signed on`. A bare
  `Date:` label defers to the document type (an invoice's bare date is its
  invoice date; an amendment's is its own date). The model's stated role is
  used only when the document's wording says nothing. The wording is read
  across a PDF's line wraps - "is dated" on one line and "as of September
  14, 2025" on the next are one sentence - but a line that ended a sentence
  or a label keeps its cue to itself, so a header's `Date of this Notice:`
  never lends `notice` to the sentence under it.
* **A missing type, or a partial one.** When the model offers no document
  type, or one the document does not support, the document's own title - the
  first outline heading, at most eight words, containing a type noun
  (`agreement`, `invoice`, `minutes`, `NDA` ...) - becomes the type, and the
  document is routed to review with `TYPE_INFERRED` so a person confirms the
  title names the document. A document with no such title still gets no
  type. A supported type the title merely completes - `Journal` under
  "MOONLIT ARCHIVE PROJECT JOURNAL" - is completed from the title, whole,
  without a review flag, provided the extra words are plain: not an exhibit
  label, not a party's name, not the `No` a stripped number leaves behind.
* **Who issued an invoice.** An invoice, receipt, account statement, or
  quote is *from* whoever issued it, not *between* its two sides. When the
  model says `between` for one of those types and the document's own layout
  names a customer (`Bill to`, `Sold to`, `Attn`) or an issuer (`Remit to`,
  `From:`), the relation is repaired to `from` the issuing party. Every cue
  nominates an issuer; one nominee settles it, however many cues agree, and
  two leave the model's answer alone. A statement *of work* is an agreement,
  not a statement, and is never repaired this way.

Each is deterministic, unit-tested against the corpus's own date lines and
titles, and never invents a fact: a role, a type, or a relation is inferred
only from text the validation has already found in the document.

## The filename

```text
YYYY-MM-DD <document type> <relation> <party>[ and <party>].<ext>
```

`<relation>` is one of `between`, `for`, `with`, `from`, `to`, or — when the model
declines to state one — a bare `-`, which keeps a validated party in the name
without asserting a relationship the document never established. Only `between`
takes two names; the others take the first, and a stated one-sided relation
keeps only that one on the validated proposal too - `to John Smith and
Northstar Lantern Works LLC` would assert a relationship the notice never
stated. A declined relation asserts nothing about anybody, so it keeps every
validated name for the reviewer to read. Real names from the scored corpus:

```text
2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf
2026-12-29 Notice of Termination - John Smith.pdf
2025-04-30 Invoice from Nimbus Orchard Supply Co.pdf
Lease Agreement with Orion Glass Studio Inc.pdf
```

Those are outputs, not illustrations. The last one carries no date because the
scan gave up no readable one, so it goes to review rather than borrowing a date
from somewhere else in the page. Nor does it become a rename as it stands:
every applied name must begin with a date (`DATE_REQUIRED`, refused at
approval and again at the apply), so the reviewer types one or accepts the
model's unverified reading. The analysis keeps the model's reply beside the
validated facts for exactly that offer — a date the document never states
verbatim is withheld from the name, not lost — and lists every date the
document does state, so a document the model could not date is dated from
its own page in a click, with the file's last-modified date as the labelled
last resort. A date printed only in numbers is listed when it can be read
one way: `30/01/2026`, a year-first date, or `03/04/2026` in a document
whose other numeric dates show which way round it writes them. One that
could be either, and any two-digit year, is left off rather than offered
under a guess. Whatever date the applied name carries is the date the queue
files under: the layout's year folder and the description record read it
from the name, never from the fact validation withheld.

The party clause is composed from a validated relation and validated names, not
from free text, so every name in a filename has been found in the document.
`between` needs two names: when a house-style merge or an unprintable name
leaves one, the clause reads `with`. A type or a party a letterhead or a scan
printed in capitals is title-cased for the name — `ORION GLASS STUDIO INC.`
becomes `Orion Glass Studio Inc.` — when it has no lowercase letter and at
least two words of four letters or more, so `IBM` and `KPMG LLP` stay as they
are; company suffixes read as usual (`Inc`, `Corp`, `Ltd`, `GmbH`, while
`LLC`, `LLP`, `PLC` stay in capitals), a word with a full stop is an
abbreviation (`No. 2`, `St. Louis`), common short words are cased (`Bank of
New York`, `Wage and Tax Statement`), other words of three letters or fewer
and vowel-less initialisms (`ABC`, `HSBC`) stay in capitals, and a word with
a digit, an apostrophe, or a `Mc`/`Mac` surname prefix is left alone
(`MACHINES` is a word, not a surname). Only the name changes; the evidence
and the description keep the document's casing.
Names longer than 120 characters shed the second party, then the party clause,
then truncate the type — detail is lost from the least identifying end first.
Typographic ligatures (`ﬁ`) and full-width letters (`ＡＣＭＥ`) are folded to the
letters a person types and every segment is composed to NFC, so two spellings
of one accented name make one filename. Windows-hostile characters, reserved
device names, trailing dots and spaces, and
invisible formatting characters — the bidirectional controls, a soft hyphen, a
zero-width space, a byte-order mark — are removed; the original extension is
always preserved; collisions get a ` (2)` suffix, judged the way Windows
compares names. The engine checks collisions
against the only folder it knows, the document's own; the queue recomposes the
name against the folder the document is actually going to, so a suffix means
a real collision at the destination and never a phantom one at the source -
nor one with the document itself. Filed into the folder it is already in (no
destination, and a flat layout), a document's own name is left out of the
comparison, so a document already named the way Intern names documents is
proposed under that name rather than as ` (2)`. Filing it under its own name,
or that name in other letter case, which Windows takes for the same file,
completes it without touching the file and records `ALREADY_NAMED`; this
release makes no case-only renames.

Where a document lands is the destination folder plus, optionally, a
subfolder the queue derives from the validated facts: the year, the year and
type, the type, or the first party (`2026/Statement of Work/`). A folder
name is cut to 80 characters and never ends in a space or a dot, which the
queue's verbatim paths would otherwise create as written. A fact the
layout needs but the document lacks sends it to `Undated` or `Unsorted`, never
the root. Folders are created on first use and removed by the undo that
empties them; the destination itself is never removed, and neither is
anything above the folders the layout in force could have made - a
destination changed to a folder that contains the old one leaves everything
already filed inside the new root, and those folders are somebody's filing
rather than Intern's scaffolding. An undo puts the
document back and leaves it waiting for a person, not ready to file: ready is
the state the scheduler files from, and with automatic renaming on the same
name would be applied again within the minute, undoing the undo.

Only one document is worked on at a time, so an approval made while the queue
is busy cannot be applied on the spot. It is remembered on the proposal and
applied by the scheduler between documents, under the name the reviewer typed
- a busy queue is not something wrong with the document, and never sends it to
review, and a spelling rule learned while it waits does not rebuild it. An undo does not wait for a document being read: it moves nothing but
its own filed document, so the only thing it waits for is another rename.

A rename or undo that fails partway is settled rather than left: its receipt
records how far it got, and a reconciliation - straight after the failure, on
the recovery pass, or when a person asks to check again - finishes it or rolls
it back. A rename refused because the document is open in another program
moved nothing, so it is rolled back after a plain read of the original, and
approving again once the program lets go files it. A file that appeared at
the destination name in the meantime is somebody else's, so the rename is
rolled back and the document waits in review; only the same file or the same
bytes at both names is ambiguous enough to hold, and an item whose files a
person has sorted out by hand can then be removed once they confirm it.
Approving a document with an unfinished operation checks its files first. If
that finishes the earlier rename, the document is filed under the name that
rename gave it, and an approval that asked for another name or sentence is
refused with that name rather than reported as applied. A rename rolled back
into review takes the approval off its proposal, and a document read again
since its old rename was journalled stays in review when that rename is
rolled back. Every file operation and the reconciliation after it run one at
a time, so the recovery pass - every 65 seconds by the clock, however often
new documents wake the scheduler - never reconciles an operation still in
flight, and an operation that has been taken over stops before its rename
rather than after. A drain that finds nothing it may claim because a rename
or an undo is running waits for it and carries on, instead of leaving the
backlog until the next wake. An undo that cannot reach the filed document -
an offline share - says so; only a document the file system reports missing
is called moved or deleted.

A document that changed after it was read - signed, edited, saved over - is
never filed under a name that described the earlier version: approving it
sends it back to review as `FILE_CHANGED` and says so. **Re-analyze** reads it
again from the start under its new fingerprint, dropping the earlier proposal
and any approval in it; it is refused while an operation of the document
never finished, because what is on disk is an open question until that is
checked again. A filing to another volume is a verified copy rather than a
rename, and the copy keeps what a rename would: the document's modified and
accessed times (and its creation time on Windows) and its Mark-of-the-Web,
the `Zone.Identifier` stream that keeps Office's Protected View on for a
downloaded or e-mailed file.

### House style

The document's words are not always the words a person files under.
"Contoso Worldwide, Inc." is "Contoso" to everyone who works there, and a
reviewer who fixes that in every name is teaching something the model cannot
learn and validation must not: validation checks that a name is *in* the
document, and "Contoso" alone would pass that check for the wrong reason.

So house style is a separate, deterministic stage that runs after validation
and before naming. A rule maps a spelling as the document writes it (matched
with case, punctuation, and spacing disregarded, words never loosened) to the
spelling the reviewer wrote, for one party or for the document type. The
queue applies the rules in force to the validated proposal, composes the name
from the result, and records which rules fired beside the proposal. A rule's
spelling is the reviewer's, so the name carries it exactly as typed -
capitals included - while the document's own words beside it are
title-cased as usual. The
engine's analysis is untouched: the evidence panel still shows the document's
words, and the description record and the layout folder follow the styled
proposal, so the name, the folder, and the record agree.

Rules are learned only from edits, and only from edits that respell exactly
one field. The approved name is read with the grammar that composed the
proposed one - type, connecting word, party, `and`, party - after stripping
the extension, the date, and any collision suffix, so a reviewer who typed a
date and shortened a party in one go still teaches the party. An edit that
touches two fields, the connecting word, or a name the engine did not compose
teaches nothing: it is a decision about that document. A name an earlier
version proposed is read the way that version composed it, so a document
still waiting when Intern is upgraded teaches as before. Reading the grammar
rather than diffing matters, because the smallest edit lies: "Acme and
Contoso" becomes "Acme Corp and Contoso Inc" by inserting text one character
into the connector, and a diff would credit it all to one party.

A rule takes effect on the second identical edit (`EDITS_TO_LEARN`), or at
once when a person says "Use now" in Settings, and every document still
waiting is recomposed under it so the queue shows the change immediately -
every document but one whose name a person approved - just now, or earlier
and still waiting to be filed - which is the reviewer's own text and would
lose whatever the validated facts do not carry, the date they typed with it
most of all.
Respelling a spelling Intern applied maps back to the document's word - the
person changed their mind about the word, not about Intern - and restoring
the document's own spelling retracts the rule. The whole memory is the list
in Settings; nothing is learned that cannot be seen and forgotten there.

## Model and runtime

| | |
| --- | --- |
| Model | Qwen3.5-2B-Instruct, Q4_K_M GGUF (1.19 GiB) |
| Runtime | llama.cpp `b10361`, CPU only |
| Context | 8,192 tokens |
| Threads | half the logical processors, clamped to 2–12 |
| Vision | none. No projector is pinned, downloaded, or loaded, and the request type has no field for an image |

The model is text-first. Essentially every business document has usable text,
and a vision projector costs hundreds of megabytes for a capability used on a
small minority of files. Intern starts the server with `--no-mmproj` and never
starts it any other way: `LlamaServer::start` has exactly one call site, and it
passes `None` for the projector. A page that neither text extraction nor OCR can
read goes to review.

This paragraph used to describe the runtime reloading once with a projector when
a document arrived with an image and little text. No such path exists — the
manifest pins one file, `ModelRole` has one variant, and the table above already
said so. Both statements could not be true.

Threads are half the logical processors on purpose. llama.cpp scales with
physical cores rather than SMT threads, and taking every core makes the rest of
Windows stutter — the product's premise is that it runs while you work. For the
same reason both sidecars run at below-normal priority on Windows: they still
get every cycle nothing else wants, and the window in front never waits on them.

A reply may run to 1,024 tokens. The grammar's closing brace ends it long
before that - the corpus's longest is about 200 - but a contract whose opening
paragraph names the date and both parties is quoted as evidence three times
over, and at the old 420 that reply was cut off mid-string. A reply that does
hit the limit is reported as `MODEL_REPLY_TRUNCATED` and not sent again:
decoding is greedy, so the same request stops at the same token. Only a reply
that finished but cannot be read gets its one second attempt.

What llama-server and `intern-worker` print on standard error - a missing CPU
feature, a quarantined runtime library, a model that will not load - is kept in
`llama-server.log` and `worker.log` under the app's local data folder
(`%LOCALAPPDATA%\com.intern.app\logs`), each emptied when it is next opened
past 256 KiB. Setup's messages for a runtime that will not start, will not
become ready, or fails its self-test point there rather than at a download the
checksum has already vouched for. At its default verbosity llama-server logs no
prompt text and not its `--api-key`, and it is never started with `-v`; the
worker's panic hook writes where a panic happened and how long its message
was, never the message, which can quote the document.

The model download, a hosted model, and Microsoft Graph trust the operating
system's root certificates as well as the Mozilla set bundled into the binary.
Firms that inspect TLS install their own root in the Windows store, and with
the bundled set alone the first-run download failed behind such a proxy with
nothing saying why. A store with nothing usable in it would stop a client from
being built at all, so each client is built again without it rather than not
at all. The clients for the local server never read the store: they speak plain
HTTP to this machine.

The server is started once and kept warm between documents, so how it stops
matters as much as how it starts: it holds well over a gigabyte, and a second
copy started by the next launch would hold another. Stopping it is not left to
`Drop`. Every child Intern spawns — the model server and `intern-worker` both —
joins a Windows job object marked kill-on-close, whose last handle is Intern's
own and is closed by the kernel however Intern ends. A crash, a panic, and the
`std::process::exit` that the window close and the updater's install step both
leave through therefore all reap the children, and the updater never asks NSIS
to overwrite a binary that is still running. Deliberate exits do better than
that: Tauri's exit events stop the pipeline while it is still whole, and the
updater's before-exit hook — Tauri's `cleanup_before_exit`, reached through a
guard parked in the app's resource table — does the same before the installer
is launched. The job object is the backstop, not the plan.

Launch never reads the model file. Tauri's setup hook asks only whether it is
there at the size the manifest pins; the digest is checked behind the window,
on the setup thread, and used to be read twice in the hook, a white window for
five seconds or more on an office laptop. A model that has passed its digest
and the semantic self-test is stamped in `models/.verified.json` with its size
and modification time, the digest it was checked against, the llama-server
binary's size and date, and the version of Intern that checked it. While all of
that still matches, the next launch starts the server with neither check; when
any of it changes, the model is hashed and self-tested once and stamped again,
and a failed self-test removes the stamp. Within a session the digest is read
at most once: restarts - a cancel, a recovery - start a file that still looks
exactly as it did when it was checked, and refuse one that does not.

Starts and stops are serialized, and every deliberate stop - a cancel, a hosted
model chosen, shutdown - moves a generation counter before it stops anything.
A cancel interrupts a request by restarting the server under it, and that
request used to fail like a server that had died: it was recovered, restarting
the server again, and the canceled document was read a second time, while two
servers could load at once. Now a request that fails after a cancel reports
`MODEL_CANCELED`, a recovery for a failure older than the last restart does
nothing, the queue skips recovery for a request it has itself canceled, and
the canceled item no longer pauses the queue through the lease its cancel took
away. A document the model failed on its own terms - too large, a reply cut
off or unreadable - fails without a restart. A cancel no longer holds the
queue while it restarts the server, so the next document can reach the model
before the new server has loaded; a request that finds a restart under way - a
cancel's, or the start that choosing the local model again begins - waits for
it, and is never sent to a server a stop is about to take away. One that still
finds no server running is handed back to the queue as `MODEL_NOT_READY`, which
ends that pass with nothing counted against the document; counted, it used to
fail as a file error the second time it met a restart.

Choosing a hosted model stops the local server, and a launch with a hosted
model chosen never starts it: it would hold 1.3 to 2.6 GB with nothing to ask
it. Choosing the local model again starts and verifies it as a launch does, and
the queue waits until that has finished. A request the switch interrupts goes
back to be read again, by the hosted model. While a model downloads, the window
hears about it at most about four times a second, besides every change of
status and the final byte; it used to hear about every network chunk.

### A hosted model

The inference is local by default and the local server is the product. The
same position in the pipeline can be filled by a hosted model behind an API
key: the engine's `Proposer` is the one seam, the local client and the hosted
client both implement it, and the distillation, prompt, validation, and naming
on either side do not know which answered.

The hosted client speaks two wire formats — Anthropic's Messages API, and the
chat-completions shape OpenAI defined and most providers and local servers
copy — and sends only what every server understands: the model, the system
instruction, the prompt, and (for Anthropic, where it is required) an output
cap. No sampling knobs, because a parameter one provider rejects is a document
that never gets filed. What goes out is the distilled digest of the document,
condensed but verbatim; what comes back is read through the same JSON
recovery and the same evidence checks as a local reply. A refusal from the
model - Anthropic's `refusal`, OpenAI's `refusal` field, a `content_filter`
finish - is reported as one and fails that document on its own, never re-routed
elsewhere and never sent again; a missing or rejected key, an unreachable
service, a model name the service does not know, an address that has moved or
cannot be used, and an account out of credit (`HOSTED_MODEL_BILLING`: a 402,
Anthropic's `billing_error` or its 400 about the credit balance, OpenAI's
`insufficient_quota`) all pause the queue rather than failing the backlog one
item at a time, and the queue says which; a busy service earns one retry before
it pauses. That retry waits as long as the service's `Retry-After` asked, up to
a minute, and otherwise about eight seconds, spread by a fifth either way. A
request to a service on the internet may take three minutes; one to a server on
this machine - LM Studio or Ollama on a laptop CPU - may take 400 seconds, so
that a request that timed out, the wait, and its one retry all end inside the
queue's fifteen-minute deadline rather than running on past it.

### When a document fails

Each failure is stored under a code that says what went wrong, and the code
decides what happens next. A failure that would come out the same on a second
attempt fails the document at once: a password-protected file, a file that is
not what its extension says, a document past the extraction limits or too long
for the model, a damaged file the parser rejects, an internal failure on that
one document, and a model refusal. Asking again would only re-read the file —
thirty minutes of it, for a scan that hit the time limit — or re-send and
re-bill the request. A worker crash, a failure the worker says may pass, and a
model request that timed out or was called off get one more attempt. A failure
that every following document would share pauses the queue and names the
reason in the window: the hosted-model failures above, a local model server
that cannot be started or recovered, and missing text-recognition files. A
model reply that cannot be used fails that document; three documents in a row
with such a reply pause the queue, since by then the model is the problem, and
any document read successfully starts the count again. A document that meets
no model at all — the moment while Settings switches between the hosted and
the local one, before the local server's start has begun — goes back to wait
with nothing counted against it. A request that finds a start or a restart
under way waits for it instead (see above), so a model that is missing after
that did not come up. Still missing four minutes later, far longer than a
switch takes to begin its start, it is a start or restart that failed: the
queue pauses and says the local model stopped responding, rather than reading
the same document again on every pass with nothing on screen.
The banner for a pause says to resume the queue; a document's own sentence
for the same code, which says to retry that document, would name the wrong
action.

These codes are new in alpha.11, and alpha.10 fails its whole queue listing
on a code it does not know. A database in which alpha.11 has recorded a
failure - a failed document, or one waiting for its second attempt - or a
document the folder watcher set aside (`INTAKE_WITHDRAWN`, on a canceled row)
therefore shows an empty queue if alpha.10 is reinstalled over it: remove the
failed and set-aside documents, and let the waiting ones finish, before going
back. From alpha.11 on, an unknown code reads as none, and a row or an
operation record with a status, direction or stage a newer build wrote is left
out of the listing rather than failing it.

The key is stored in the operating system's credential store under Intern's
name, never in the settings file, and never travels anywhere but the address
that was configured — redirects are refused. Plain HTTP is accepted only to
this machine, so a local server can be used without a certificate and a
remote one cannot be used without one; the same judgement takes the machine's
proxy out of the path for an address on this machine, because a proxy would
otherwise receive in cleartext the key and the document text that plain HTTP
was allowed for on the grounds that neither leaves the machine. A service on
the internet is still reached through the proxy. **Test connection** sends the same
calibration document setup uses to check the local model, so a wrong key,
model name, or address is found before a real document is sent. It sends to
the address on screen rather than the saved one, so a new address can be
tried before it is saved — but a key that was already on the machine goes
only to the address the settings name. Typing the key is what admits a new
address, and nobody can type a key they do not have.

### The same document twice

Exact duplicates are a hash comparison before analysis. The duplicates people
make are not exact: a second scan of the same page, a PDF exported twice from
the same message, a copy saved again by a program that rewrote its metadata.
So the engine also fingerprints everything the extractor read - a 64-bit
simhash over five-character shingles of the normalised text, hashed with
FNV-1a spelled out in the crate so the value is identical on every machine
and in every build, because the shared filed index carries it between
teammates. Similar text gives similar bits; a second scan with a handful of
misread characters lands within six bits, and two unrelated documents sit
about thirty-two apart.

The queue holds every analysis against the fingerprints of its own filings
and asks the duplicate oracle about other machines. Closeness alone does not
decide: this month's statement and last month's share almost every word, and
a fingerprint barely sees the date and the figures that differ. So the dates
have to agree - the filed name's leading date against the date the analysis
found or the model read - and without a date on one side only a
near-identical text counts. Every filing within the
distance is considered, not only the nearest one: last year's renewal of an
agreement can be nearer in text than this year's second scan of it is, and
looking only at the nearest hid the filing the document really repeats behind
a date that said "another document". A match sends the document to review with
`NEAR_DUPLICATE`, named after the filing it repeats and the machine that made
it; it is never filed on its own, and an undo forgets the fingerprint.

## Measuring it

Every stage after the model is deterministic, which is what makes accuracy
measurable without the model. `intern-evaluate` records a live run - the text
the worker extracted from each fixture and the reply the model gave, keyed by
the hash of the prompt - and replays it in seconds: distillation, validation,
inference of roles and types, house style, and naming run for real over the
recorded reply, and the corpus is scored against `fixtures/expected.json`. A
committed baseline turns that into a gate: CI replays on every push and fails
when a reviewed answer that was right is now wrong, and a prompt change makes
the recording stale rather than silently scoring replies to a question the
engine no longer asks. [`evaluation.md`](evaluation.md) has the workflow.

## What it costs

Measured on an AMD Ryzen 7 PRO 8840U with 14.7 GB usable RAM, CPU only, with
ordinary applications running:

| | |
| --- | --- |
| Extraction | 13 ms for a one-page invoice, 33 ms for a 14-page contract; 38 ms median across the corpus |
| Extraction, scanned page | seconds, and up to 6.6 s when a page has to be re-read in other orientations |
| Distillation | 0.3 ms to 9 ms |
| Median document, end to end | 12.4-27.7 s across four runs of the same corpus on the same machine |
| 29,000-character contract | 42 s |
| Peak model process memory | 2,470-2,590 MB |
| First-run download | 1.19 GiB, the text model and nothing else |

Quote the latency as a range. Four runs of the same corpus on this machine gave
medians of 12.4, 16.6, 19.6, and 27.7 seconds depending on what else was
competing for the eight threads, and any single figure from that spread is noise.

Almost all of the time is the model, and on short documents most of that is
*generation*, not reading: the structured reply is about 240 tokens at 17.5
tokens per second. The previous pipeline and model took 23.6 s on the median
document and 115 s on its worst, with 4,215 MB of peak memory.

`docs/qa/model-evaluation.json` records one full-corpus evaluation - all 18
scorable fixtures, real inference, the pinned model verified by size and digest -
bound to the commit and release-input hash that produced it. It comes from a
development laptop, not the pinned release runner, and cannot satisfy a release
gate: the release workflow rescores the corpus itself and
`validate-release-evidence.mjs` requires the evidence to name the live run.
`docs/model-bakeoff.md` has the measurements behind the model and pipeline choice,
including what was rejected and what still misses.

## The boundary

`intern-engine` has one entry point:

```rust
let analysis = engine.analyze(&source, "pdf", &existing_names)?;
```

`DocumentSource` in, `DocumentAnalysis` out — filename, description, status,
review reasons, validated facts with evidence, and local timings.
`ENGINE_CONTRACT_VERSION` versions that shape.

`intern-analyze` is that call as a command-line program. The desktop app, the
CLI, and the watched intake folder are all callers of the same function; none
of them can change how documents are understood. Adding a new host means
adding a caller, not touching the engine. Adding a model means implementing
`Proposer`, which is what the hosted client is.

The watched intake folder — including shared OneDrive/SharePoint intake
folders, network shares, and the multi-machine claim protocol behind them —
lives in `intern-intake` and is documented in
[`shared-intake.md`](shared-intake.md). It sits entirely on the queue side of
this boundary: it decides *which* documents enter the local queue and records
what happened to them, and knows nothing about models.

The queue reports every completed rename to a *filing sink*, and the desktop
app's sinks write the description records that let a SharePoint column carry
the sentence — see [`sharepoint-descriptions.md`](sharepoint-descriptions.md)
— and the filed markers of the shared intake folder. A sink hears about a
rename only after it has succeeded and cannot undo it; a record that fails to
write is reported in Settings, and the rename stands. A rename the applier
had to settle afterwards - an ambiguous failure finished by a reconciliation,
here, on the next retry, or on the next recovery pass - is reported the same
way, because what is reported is read from the queue's own record of the
operation rather than from whichever call happened to finish it.

The mirror image is the *duplicate oracle*: before analysing a document the
queue checks its own history for the same content, then asks the oracle,
which in the desktop app reads the shared folder's filed markers. Either
answer routes the document to review as a duplicate, naming what the content
was filed as and, for a teammate's filing, by which machine. Analysis never
runs on a duplicate unless a person asks for it.

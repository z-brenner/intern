# Intern on held-out real letters, notices and invoices: November 2026

Status: measured on 2026-11-02. [The first real-document set](../realdocs-2026-11-02/README.md)
had six rules written from it (#51). That set no longer measures how those rules carry, so this
one does. It is 60 more real documents, none in the first set, and no rule was written from
them. Their gold was written before Intern read any of them.

**The answer:**
- **The rules carried.** On these unseen documents, #51 takes Intern from 12 to 14 names
  right, and from 7 to 5 wrong names marked ready. One change went the wrong way, in a name
  that goes to review either way (below).
- **Intern is still not safe to file this kind of mail unreviewed.** Half the names it marks
  ready are wrong.

## The set

60 real documents, pinned in `scripts/realdocs-eval/sample-heldout.json` by source URL, size
and SHA-256. `python3 -I scripts/realdocs-eval/fetch.py OUT sample-heldout.json` fetches and
checks them. All 60 download from their sources with the script's own User-Agent; none needs a
local copy.

The gold follows the first set's conventions. Four agents found and read the documents, under
the first set's brief plus these rules:
- no document, URL or (where possible) issuer of the first set;
- no sec.gov, and no personal contact in any request;
- no workaround for any site that refused scripted access.

| Kind | n | Formats |
|---|---:|---|
| Letter | 25 | scanned PDF 6, PDF 6, ODT 4, DOC 3, DOCX 2, JPEG 2, MSG 2 |
| Notice | 20 | PDF 7, scanned PDF 5, DOCX 2, TXT 2, XLSX, DOC, ODT, RTF |
| Invoice | 11 | TIFF 4, PDF 3, JPEG 2, scanned PDF 2 |
| Statement | 2 | TIFF, scanned PDF |
| Receipt | 2 | JPEG 2 |

It adds the first real `.xlsx` (a UK offshore regulator's improvement notice) and the first real
`.msg` files (two emails from a California agency's public records release). No real single
invoice was found as a spreadsheet; the search covered about 26,000 GOV.UK publications.

**Sources** (`sample-heldout.json` gives each document's page and licence):

| Source | Licence |
|---|---|
| Federal Reserve, DOE, CPSC, FDA, FEC, EPA, PHMSA, FCC; FDIC and FERC notices from the Federal Register's public-inspection copies; the National Weather Service | US government works |
| Federal court filings via CourtListener's RECAP storage | public court records |
| GOV.UK: DCLG, OFT, BEIS, HM Treasury, MoD, No. 10, DfT, Ofqual, OPRED, Environment Agency, Planning Inspectorate, ACMD, MAC, DWP, ACOBA, Cabinet Office | Open Government Licence v3.0 |
| Australia's Federal Register of Legislation | CC BY 4.0 |
| UCSF Industry Documents (tobacco litigation, via the Internet Archive mirror) | made public through the Master Settlement Agreement |
| Library of Congress: Bell and Lincoln papers; California Revealed | public domain |
| California Building Standards Commission's 2016 public-records release, published by Public.Resource.Org | public records |

**Caveats:**
- **The California records carry no open licence.** Their basis is California public-records
  law (*County of Santa Clara v. Superior Court*, 2009) and Public.Resource.Org's
  public-domain statement.
- **Court filings were found indirectly.** CourtListener's own website answers scripted
  requests with 403, so the agents located filings through the Internet Archive's copy of RECAP
  docket data. Each file was then fetched from CourtListener's public file storage, which
  serves it directly.

**Run.** One live InternBench run, with the same model, runtime and prompt as the first set's:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, the app's flags, 4 threads;
- review gate: token confidence at 0.5.

#51 changes validation only, never the prompt, so the one recording was replayed with each
version of the code:
- main as #50 left it (before);
- #51 (after);
- #51 with this change's fix (afterwards).

## Results

Each name is adjudicated by the first set's rule:
- A name with no joining word counts as right only when its party issued the document.
- A party name shortened only by a legal-form word or a fund's series name counts as right.

Four names the scorer marks wrong are right on this rule; `readings.json` says why for each.

| | Main (#50) | **With #51** |
|---|---:|---:|
| Right, scorer | 8/60 | 10/60 |
| Right, adjudicated | 12/60 (20%) | **14/60 (23%)** |
| Marked ready | 12 | 10 |
| **Wrong names ready** | **7** | **5** |

Against the first set, unseen: 11/58 right, and 12 of 19 names marked ready wrong. This set is
as hard as the first.

**What #51 changed here**:

| Document | Main | With #51 |
|---|---|---|
| Federal Reserve letter to BNP Paribas's counsel | "Letter to Fortis Bank Belgium", a bank it is about | "Letter to Cleary Gottlieb Steen & Hamilton LLP", its addressee (review: bare "Letter") |
| DOE enforcement letter | "Letter - Dr. Thomas Burns, Jr." | "Letter to Savannah River Mission Completion, LLC" (review) |
| KUSA-TV invoice | "Invoice - KUSA-TV", ready, wrong date | review, no party |
| Environment Agency consultation notice | "Notice for Environment Agency", ready | review (bare "Notice") |
| Notice of deposition | "from Girafa.com" | between Girafa.com and Amazon.com (its date is still wrong) |
| Notice of entry of order | "from American Service Insurance" | between the case's two sides |
| EPA notice of violation | "for David Geanacopoulos", a person | "to Volkswagen Group of America", still with no date (review) |
| GOV.UK invoice | "Invoice - Mike Murphy", ready | "Invoice from Our Bank", ready: still wrong, see below |
| An 1882 bill | "from Charles Williams Jr.", its issuer, with no date (review) | "from National Bell Telephone Co.", a company its billhead names (review) |

The last row is the one change for the worse. The reply called the bill's issuer, Charles
Williams Jr., its buyer. #51's rule for an invoice that names only its customer counts that
unconfirmed "buyer", so it took the organization at the head of the page as the issuer. That
overrode the billhead's "BOUGHT OF CHARLES WILLIAMS, Jr.", which names the seller. Both versions
go to review. It is fixed in this change (below).

**By kind and text layer, with #51** (adjudicated right / marked ready / wrong ready):

| Kind | Right | Ready | Wrong ready |
|---|---:|---:|---:|
| Letters (25) | 4 | 1 | 1 |
| Notices (20) | 8 | 8 | 3 |
| Invoices (11) | 2 | 1 | 1 |
| Statements (2), receipts (2) | 0 | 0 | 0 |

| Text layer | Right | Ready | Wrong ready |
|---|---:|---:|---:|
| Native PDF (16) | 4 | 5 | 4 |
| Office: DOCX, DOC, ODT, RTF, XLSX, MSG (17) | 3 | 2 | 0 |
| Scanned PDF (14) | 4 | 2 | 1 |
| Image: JPEG, TIFF (11) | 2 | 0 | 0 |
| Text (2) | 1 | 1 | 0 |

**Why the 46 wrong names are wrong** (with #51; a name can fail more than one way):

| Class | Names | Ready |
|---|---:|---:|
| Type generic or missing: "Document", a bare "Letter" or "Notice" | 22 | 0 |
| A person named instead of the organization | 13 | 0 |
| No date read | 11 | 0 |
| No joining word, and the party is not the issuer ("Advisory Opinion - Chicago Mercantile Exchange") | 6 | 1 |
| An invoice's customer named, or no issuer | 5 | 0 |
| Type names another document or the wrong kind | 5 | 0 |
| A date the document mentions, not its own | 5 | 2 |
| A party the document only mentions, or a label read as a name | 4 | 1 |
| OCR: a misread name or a hand it cannot read | 3 | 0 |
| The file could not be opened | 2 | 0 |
| A court filing's sides not as its caption gives them | 2 | 1 |

**The 5 wrong names still ready:**
- **Two notices whose own "Dated:" stands at their end.** The deposition notice and FERC's
  notice of technical conference were dated by the deposition and the conference. #51's date
  rule reads only the first page, so that an exhibit's "Dated:" can never date the document it
  is attached to; these two notices' own lines sit on page 2.
- **A GOV.UK invoice's "Our Bank:" field read as its issuer.**
- **A notice of related action on numbered pleading paper**, filed "for" the defendant: the
  caption's sides were not read.
- **An FEC advisory opinion** named "- Chicago Mercantile Exchange", its addressee, with no
  joining word.

**Two documents could not be opened:** tobacco-litigation TIFF scans whose Group 4 data is
stored with FillOrder 2, least significant bit first. The `tiff` decoder Intern uses reads
them as corrupt; libtiff reads them. That is a worker fix of its own.

## Afterwards: one fix this set showed (in-sample for it)

The 1882 bill's lost issuer came from one widening in #51. That widening let the reply's
unconfirmed "customer" role count when an invoice seems to name only its customer, so the
organization at the head of the page was taken as the issuer instead. It is reverted here, and
only a role the document confirms counts again. On every recording, the widening's only effect
had been to change wrong names into other wrong names. #51's other rule still stands: an
invoice whose first party the reply calls the customer names no one.

| | With #51 | With this fix |
|---|---:|---:|
| This set: right, adjudicated | 14/60 | 14/60 |
| This set: wrong names ready | 5 | **4** |
| First set: wrong names ready | 3 | **2** |

**What changed:**
- **This set:**
  - The 1882 bill names Charles Williams Jr. again; it stays in review, with no date.
  - The GOV.UK invoice's "Invoice from Our Bank" goes to review.
- **First set:** the spreadsheet of outstanding invoices goes to review.
- **Everywhere else:** InternBench (ten recordings) and CUAD (three samples) are unchanged.

## Afterwards: a notice dated at its end (in-sample)

Two of the wrong names still ready were notices dated by an event: a deposition and a
technical conference. Each dates itself at its end, on page 2, and #51's own-date rule read only
the first page. It now reads the document's own pages, up to the first page that opens as an
attachment ("EXHIBIT A", "SCHEDULE A", "Attachment 1"). An exhibit's or a schedule's "Dated:"
still never dates the document. The rest of the rule stands:
- the reply's date must have no field;
- exactly one own date must be found;
- a letter's dateline always stands.

| | Before | After |
|---|---:|---:|
| This set: right, adjudicated | 14/60 | **16/60** |
| This set: wrong names ready | 4 | **3** |

**What changed:**
- **This set:**
  - FERC's notice of technical conference is right and ready.
  - The Lehman notice of bankruptcy is right; it stays in review.
  - The deposition notice, the KUSA-TV invoice and (in the first set) a notice of appeal now
    carry their own dates; their other faults remain.
- **Everywhere else:** CUAD and InternBench are unchanged.

**Speed.** p50 13.3 s per document, p95 37 s, at most 75 s, on a 4-core x86-64 host.

## Reproduce

```bash
python3 -I scripts/realdocs-eval/fetch.py runs/heldout sample-heldout.json   # 60 documents
CORPUS=runs/heldout/corpus GOLD=runs/heldout/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... THREADS=4 \
  scripts/run-internbench.sh runs/heldout/run -- --min-token-confidence 0.5
```

`readings.json` has each document's reading before and after #51, both verdicts and the failure
classes.

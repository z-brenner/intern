# Intern on a third, unseen set of real letters, notices and invoices: November 2026

Status: measured on 2026-11-03. The two earlier sets, [the first](../realdocs-2026-11-02/README.md)
and [the held-out set](../realdocs-heldout-2026-11-02/README.md), now each have rules written from
them (#51, #54, #55), so neither measures how those rules carry. This set does. It is 59 more real
documents, none in either earlier set and mostly from other issuers, gathered after those rules
were merged. Its gold was written before Intern read any of them.

**The answer:**
- **The rules carried, but only a little.** From main before #51 to main as of #55, the names
  right go from 6 to 7 of 58. The wrong names marked ready go from 7 to 5.
- **Intern is not close to filing this kind of mail unreviewed.** It names 7 of 58 right (12%),
  and 5 of the 8 names it marks ready are wrong.
- **This set is harder than the earlier two.** Before any rule, main named 19% and 20% of the
  earlier sets right, and 10% of this one. A fifth of it is historic: bills and receipts from
  1776 to 1916, most handwritten, and 19th-century posters. A quarter is office files.

## The set

59 real documents, pinned in `scripts/realdocs-eval/sample-heldout2.json` by source URL, size and
SHA-256. `python3 -I scripts/realdocs-eval/fetch.py OUT sample-heldout2.json` fetches and checks
them. All 59 download from their sources with the script's own User-Agent.

Four agents gathered and read the documents under the earlier sets' brief, plus these rules:
- no document, URL or (where possible) issuer of the earlier sets;
- at most 4 documents per issuer, and at most 4 court filings;
- no sec.gov, no personal contact in any request, no workaround for any site that refused
  scripted access.

| Kind | n | Formats |
|---|---:|---|
| Letter | 23 | PDF 10 (3 scanned), DOCX 4, ODT 2, MSG 2, TIFF 2, JPEG, XLSX, TXT |
| Notice | 19 | PDF 11 (5 scanned), DOC 2, RTF 2, JPEG 2, TIFF, TXT |
| Invoice | 11 | PDF 7 (4 scanned), JPEG 3, TIFF |
| Statement | 4 | PDF 2 (1 scanned), XLSX, CSV |
| Receipt | 2 | JPEG 2 |

It adds the first real CSV. It also has more historic hands than the earlier sets: 1776 to 1916
bills and receipts, and 19th-century posters.

**Sources** (`sample-heldout2.json` gives each document's page and licence):

| Source | Licence |
|---|---|
| NHTSA, the Copyright Office, the Justice Department's Civil Rights Division, NSF; OSTI (DOE contractor letters) | US government works |
| Federal court filings (N.D. Ohio, D. Md., D. Conn.) via CourtListener's RECAP storage | public court records |
| California Energy Commission, CPUC, the LA Regional Water Quality Control Board; the Massachusetts Attorney General | state public records |
| Ofgem, the Office of Rail and Road, the Drinking Water Inspectorate, Scottish Government FOI releases, GOV.UK departments and tribunals | Open Government Licence v3.0 |
| ACER (the EU's energy regulators' agency) | reuse with acknowledgement |
| Four TV and radio stations' FCC political files (WISN, KSAZ, KMVA, KFWB) | public records (47 C.F.R. 73.1943) |
| The London Datastore; BC and Canada open data | the Datastore's terms; Open Government Licence – BC and – Canada |
| Library of Congress (Wright, Greeley, Washington, Jefferson papers; printed ephemera), Wellcome Collection, Digital Commonwealth, an Internet Archive museum upload | public domain or no known restrictions |
| The FreeBSD Project's errata notice | the project's documentation licence |

**Caveats:**
- **One letter was gathered and left out.** Its text came from www.gutenberg.org, whose terms
  forbid automated access.
- **The London Datastore's API refused Python's default User-Agent** and served curl's. Its files
  and API both answer `fetch.py`'s declared User-Agent, so nothing was worked around.
- **notice-10 prints no date.** Its gold has none, it should go to review, and it does. It is left
  out of the filename score, as such documents were in the earlier sets.
- **The 1916 invoice relies on its museum's Public Domain Mark.** As an unpublished business
  record it may strictly stay in copyright until 2036, though it has little original content.

**Run.** One live InternBench run of main as of #55:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, the app's flags, 4 threads;
- review gate: token confidence at 0.5;
- worker: built with `--features windows-native` (PDFium and PP-OCR), on a 4-core x86-64 host.

The same recording was replayed with main before #51 to show what the rules did. No prompt differs
between the two on this set.

## Results

Each name is adjudicated by the earlier sets' rule:
- a name with no joining word counts as right only when its party issued the document;
- a party name shortened only by a legal-form word counts as right.

One name the scorer marks wrong is right on this rule: "Notice of Change of Company Name - The
Company Names Tribunal", whose tribunal issued it. `readings.json` gives every reading, both
verdicts and the failure classes.

| | Main before #51 | **Main as of #55** |
|---|---:|---:|
| Right, scorer | 4/58 | 6/58 |
| Right, adjudicated | 6/58 (10%) | **7/58 (12%)** |
| Marked ready | 9 | 8 |
| **Wrong names ready** | **7** | **5** |

**What the rules changed:**
- **Gains:**
  - A notice of appeal named "from" the appellant is now between the case's two sides, right and
    ready (#51's caption rule).
  - A notice of removal is now "with" the plaintiff the gold names, right but in review.
  - An Islington consultation response, wrongly filed as "Notice from" the council it answers,
    went from ready to review.
  - Ofgem's penalty notice lost a dangling "Pursuant" from its type (#55); it is still wrong,
    named "- Farringdon Energy Limited" with no joining word.
- **Loss:** a TV station's invoice, right in substance as "Invoice - Wisn TV", now names no party.
  The reply called the station the recipient and its ad agency the sender, and #51's rule names no
  one when an invoice's first party is the reply's receiving side. It goes to review either way.

**By kind and text layer, as of #55** (adjudicated right / marked ready / wrong ready):

| Kind | Right | Ready | Wrong ready |
|---|---:|---:|---:|
| Letters (23) | 1 | 0 | 0 |
| Notices (18 with a name) | 3 | 5 | 4 |
| Invoices (11) | 1 | 0 | 0 |
| Statements (4), receipts (2) | 2 | 3 | 1 |

| Text layer | Right | Ready | Wrong ready |
|---|---:|---:|---:|
| Native PDF (16) | 3 | 3 | 1 |
| Scanned PDF (13) | 3 | 2 | 1 |
| Image: JPEG, TIFF (12) | 0 | 0 | 0 |
| Office: DOCX, DOC, ODT, RTF, XLSX, MSG (14) | 1 | 1 | 1 |
| Text: TXT, CSV (3) | 0 | 2 | 2 |

**Why the 51 wrong names are wrong** (as of #55; a name can fail more than one way):

| Class | Names | Ready |
|---|---:|---:|
| Type generic or missing: "Document", a bare "Letter" or "Invoice" | 23 | 0 |
| No date read | 16 | 0 |
| Type names another document or the wrong kind ("Strategic Plan", "Contract") | 12 | 0 |
| A person named instead of the organization | 11 | 0 |
| A party the document only mentions, or a label read as a name | 8 | 0 |
| OCR: a historic hand it cannot read | 8 | 0 |
| No party named | 6 | 2 |
| No joining word, and the party is not the issuer | 5 | 3 |
| A date the document mentions, not its own | 4 | 2 |
| An invoice's agency named as its issuer | 1 | 0 |

**The 5 wrong names still ready:**
- **ORR's improvement notice to Chiltern Railways**, as "Improvement Notice - The Chiltern Railway
  Company Limited", dated by its compliance date. Its own issue date is handwritten.
- **A notice of removal**, as "Notice of Removal - Olymbec USA LLC". The party is the plaintiff
  and the date is wrong.
- **A company-name tribunal's notice**, as "Notice of Change of Company Name - Aston Car Group
  Limited". The party is the company it is about, with no joining word. The same tribunal's other
  notice is right, naming the tribunal.
- **FreeBSD's errata notice** and **a Canadian tribunal service's statement of financial
  position**: named with no party at all.

The validation rules catch none of the three classes that dominate. Generic types, a person for
the organization, and no date all go to review, so they cost names, not safety. What remains unsafe
is narrow: a notice or statement named with its subject but no joining word, or with no party.

## What this suggests

- **Stop writing rules from one set and measuring on the same set.** Every rule written from the
  first two sets carried less here than on them. A new rule needs a held-out set to show it
  works, and this one is now used.
- **Generic types are the largest lever.** 23 of the 51 wrong names are "Document", or a bare kind,
  for documents that say plainly what they are. Most of those are letters whose gold accepts a
  plain "Letter to X". A letter laid out as one (dateline, inside address, salutation) could name
  that kind when the reply's type is not in the document. Any such change should be measured on
  all three sets, with this one as the check.
- **Historic hands are beyond the OCR.** None of the 12 images is named right, and the six
  handwritten bills and receipts among them, from 1776 to 1868, read as noise. They go to review,
  which is right, but they measure the OCR, not the pipeline's logic.

**Speed.** p50 16.0 s per document, p95 30 s, at most 45 s, on a 4-core x86-64 host.

## Afterwards: a letter's layout names it a letter (in-sample)

The largest class above is a generic type. A rule now names one case of it. When the reply's
type is not in the document and no title stands in for it, a document laid out as a letter is
named a "Letter": it has a "Dear ..." on its first page, with a dateline or a "Date:" field above
it. The dateline may sit in a paragraph that extraction ran the letter's head into, so lines are
read, not units. A bare "Letter" still goes to review, so no name becomes ready. What changes is
the name a person reviews: "Letter to Target Corporation" in place of "Document to Target
Corporation".

The rule was written after this set was measured, so the figures below are in-sample on all
three sets.

| | Before | After |
|---|---:|---:|
| This set: right, adjudicated | 7/58 | **9/58** |
| First set: right, scorer | | +1 |
| Held-out set: right, scorer | | +2 |
| Wrong names ready, any set | | unchanged |

**What changed:**
- **This set:** Ofgem's decision letter is now "Letter to Good Energy Limited", and the attorneys
  general's letter is "Letter to Target Corporation". Two GOV.UK letters go from "Document" to
  "Letter", but still name a person.
- **First set:** the FTC's closing letter is now "Letter to Loeb & Loeb LLP".
- **Held-out set:**
  - DOE's enforcement letter is now "Letter to Packaging Specialties, Inc.".
  - An 1891 letter is now "Letter from The Columbia Phonograph Company".
- **Everywhere else:** InternBench's ten recordings keep every count; a few "Document - X" names
  there become "Letter - X", still wrong for their other reasons and still in review. CUAD is
  unchanged.

## Reproduce

```bash
python3 -I scripts/realdocs-eval/fetch.py runs/heldout2 sample-heldout2.json   # 59 documents
CORPUS=runs/heldout2/corpus GOLD=runs/heldout2/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... THREADS=4 \
  scripts/run-internbench.sh runs/heldout2/run -- --min-token-confidence 0.5
```

`readings.json` has each document's reading on main before #51 and as of #55, both verdicts and the
failure classes.

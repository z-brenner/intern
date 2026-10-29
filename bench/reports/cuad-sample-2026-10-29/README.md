# Intern on real contracts: a CUAD sample, October 2026

Status: measured on 2026-10-29. Every rule Intern has was written from failures on InternBench's
77 synthetic documents, and scored on the same 77. This is the first measurement on documents
nobody wrote for Intern: 42 real commercial contracts filed with the SEC.

**Sample.** From CUAD v1 (The Atticus Project; CC BY 4.0): every 12th of its 510 contracts,
sorted by file name, 42 with a PDF. They run from 1 to 60 pages (median about 14) and include
amendments, joint filing agreements with many parties, templates with blanks, and scanned
exhibits. The list is pinned in `scripts/cuad-eval/sample.json` by dataset revision, size and
SHA-256, with CUAD's own answers: Document Name, Parties, Agreement Date and Effective Date.

**Run.** A live InternBench run:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, with the app's flags and 4 threads;
- review gate: token-confidence at 0.5.

The replies were recorded, so every rule change below is replayed against the same replies.

**Scoring.** `scripts/cuad-eval/score.py` checks three things:
- the type against CUAD's Document Name;
- the date against its agreement or effective date (a blank there means undated);
- the parties against CUAD's list.

CUAD's answers are spans of the filings as annotated, so they carry the filings' typos and
blanks. Seven readings the scorer marks wrong are Intern right and CUAD's annotation off. They
are listed under [CUAD's annotations](#cuads-annotations), and the adjudicated row counts them
as right.

## Results

| | Before (PR #44's rules) | After (this change) |
|---|---:|---:|
| Names right, adjudicated | 28/42 (67%) | **30/42 (71%)** |
| Wrong names marked ready, adjudicated | 5 (12%) | **2 (5%)** |
| Type exactly CUAD's | 34 | **37** |
| Marked ready | 26 | 24 |
| Scorer as is: right / wrong but ready | 23 / 9 | 23 / 7 |

On InternBench the same model names 69/77 (90%) and marks 2 wrong names ready (3%). Real
contracts are harder, and the failures are of kinds the synthetic set does not have.

## What real contracts broke, and what was done

| Contract | What went wrong (ready unless said) | Fix | Now |
|---|---|---|---|
| cuad-42 | The model named "Calm.com, Inc."; Intern filed "Account Name Calm.com, Inc.", because the index read the field label and the name as one organization and the name was "completed" with it | A field's or a role's label ("Name", "Account", "Plaintiff", ...) never completes a name | Right |
| cuad-13 | Dated by a definition of *another* agreement: "'Master Services Agreement' the agreement executed between the Parties on the 21st day of February, 2017". The supply agreement itself is undated | A defined term of two words or more ending on a document noun dates that document, unless it is this document's own alias. It also changes what retrieval shows, so this one was re-run live | Date withheld, to review |
| cuad-21 | A three-party sponsorship agreement filed "with" one party | An agreement naming one party goes to review: `PARTY_MISSING` ("Only one side of the agreement was found.") | To review |
| cuad-00, -07, -39, -42 | "Co-Branding and Advertising Agreement" filed as "Advertising Agreement"; likewise "License and Hosting", "Operation and Maintenance", "Product Sale and Marketing" | An agreement's title reads back across "and" ("Purchase and Sale Agreement"); other kinds still stop there ("Availability and Price List" is a price list). A too-long one shortens to the agreement it ends on, never to an amendment's subject | Full titles (00, 07, 42 ready; 39 to review on a parser warning) |
| cuad-08 | "Software License, Customization and Maintenance Agreement" lost its comma once the title read across "and" | A title keeps its inner punctuation | Type right; to review, as no date was found |

**Tried and rejected: a date stated only on a running header or footer.** cuad-10 is dated by the
EDGAR line "Source: DATA CALL TECHNOLOGIES, SB-2/A, 9/18/2006", which repeats on all five pages.
Treating any date found only on such a line as the copy's date, not the document's, did not
catch it: the line runs into a different paragraph on each page. A line-by-line version
withheld cuad-40's correct date, because that contract repeats its own date in its header. A
header that dates the document is as real as a filing stamp, so there is no rule here.

**Still wrong and ready (2):**
- **cuad-06.** "AMENDMENT NO. 1 TO STREMICK'S HERITAGE FOODS, LLC and PREMIER NUTRITION
  CORPORATION MANUFACTURING AGREEMENT" is filed as a bare "Amendment". A title stops at "No." (an
  invoice's number), and the parties' names stand between the amendment and the agreement it
  amends.
- **cuad-10.** The filing stamp's date, above.

**To review, and right to be:**
- **Templates with a blank party or date** (01, 02, 04, 31). The model sometimes fills a blank
  party with the law firm named in a notice clause (04).
- **Joint filing agreements** with 5 to 12 filers (23, 27). Intern names two, one of them the
  issuer whose stock they report, which is no party.

## CUAD's annotations

| Contract | CUAD says | The filing says (Intern) |
|---|---|---|
| cuad-05 | "Bank of America Corporatino, Banc of America ..." (one span, comma) | BANK OF AMERICA CORPORATION; BANC OF AMERICA PREFERRED FUNDING CORPORATION |
| cuad-09 | "Howmedica Ostenonics Corp." | Howmedica Osteonics Corp. |
| cuad-29 | "AB Wately, Inc." | A.B. Watley, Inc. |
| cuad-33 | Agreement date "04/30/[]" | "Issue Date: 30 April 2020" |
| cuad-39 | "OPERATIONS AND MAINTENANCE AGREEMENT" | Title: OPERATION AND MAINTENANCE AGREEMENT |
| cuad-40 | "The John Hopkins University"; "NON-EXCLUSIVE LICENSE AGREEMENT- FOR SALES" | The Johns Hopkins University; title NON-EXCLUSIVE LICENSE AGREEMENT |
| cuad-41 | Agreement date "09/14/200[]" | "DATED SEPTEMBER 14, 2005" |

`readings.json` has every contract's reading before and after.

## Reproduce

```bash
python3 -I scripts/cuad-eval/fetch-sample.py runs/cuad            # 42 PDFs (6.7 MB), checked by SHA-256
CORPUS=runs/cuad/corpus GOLD=runs/cuad/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... INTERN_BENCH_TOKEN_CONFIDENCE=1 THREADS=4 \
  scripts/run-internbench.sh runs/cuad/run -- --min-token-confidence 0.5
python3 -I scripts/cuad-eval/score.py runs/cuad/run/report.json
# A rule change, against the same replies:
target/release/intern-bench run --corpus runs/cuad/corpus --gold runs/cuad/gold.json \
  --replay runs/cuad/run/recording.json --min-token-confidence 0.5 --output runs/cuad/replay.json
```

(`intern-bench` built with `--features benchmark-transport`.) The live run took 16 minutes on
a 4-core x86-64 host: 15.3 s per contract median, 199 s for the slowest.

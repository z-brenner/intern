# Intern on real letters, notices and invoices: November 2026

Status: measured on 2026-11-02, before any rule was written from these documents.

Every real-world check so far was a contract: CUAD's SEC filings. There, Intern names 81% right
on a sample no rule was written from, with no wrong name ready. This set asks the same question
of what an office receives besides contracts:
- letters;
- notices;
- invoices;
- a few receipts and a statement.

It also covers every format the worker reads.

**The answer is not yet.** On these 60 documents Intern names 11 of 58 right. **12 wrong names
were marked ready**, of the 19 it marked ready. The rest went to review, as they should.

## The set

60 real documents, pinned in `scripts/realdocs-eval/sample.json` by source URL, size and SHA-256.
`scripts/realdocs-eval/fetch.py` fetches them and checks each one; no document is committed.
Each gold answer was written by reading the document, in InternBench's schema and conventions:
- a letter is "to" its addressee;
- a court filing is "between" its case's two sides;
- an agency's notice is "for" the party it is about;
- an invoice is "from" its issuer.

| Kind | n | Formats |
|---|---:|---|
| Letter | 22 | scanned PDF 6, PDF 6, DOCX 3, TXT 3, DOC, RTF, ODT, XLS |
| Notice | 18 | PDF 8, scanned PDF 5, TXT 2, DOCX, DOC, EML |
| Invoice | 17 | TIFF 4, PDF 4, XLS 4, JPEG 3, scanned PDF 2 |
| Receipt | 2 | JPEG 2 |
| Statement | 1 | XLS |

By text layer:
- 18 native PDFs;
- 14 office files;
- 13 scans;
- 9 images;
- 6 plain-text or email files.

**Sources**, all public records or openly licensed:

| Source | Documents | Licence |
|---|---|---|
| US federal agencies: FTC, DOJ Antitrust, EPA, PHMSA, FCC, IRS | letters, notices | US government works, 17 U.S.C. 105 |
| Federal court filings via CourtListener's RECAP archive | notices | public court records |
| UK government via GOV.UK: CMA, DCMS, MoJ, CSPL | letters, a notice | Open Government Licence v3.0 |
| Broadcast stations' FCC political files: KCNC, WPXI, WPTV, WRAL, WBTV | invoices | public records (47 C.F.R. 73.1943) |
| Internet Archive public-domain collections, 1859–1983 | invoices, receipts | Public Domain Mark |
| UCSF Industry Documents (tobacco litigation) | invoices | public through the Master Settlement Agreement |
| SheetJS's `enron_xls` and EDRM's Enron email set | spreadsheets, an email | CC0 and CC BY 3.0 US |

`sample.json` names each document's exact source page and licence.

The set is chosen to be hard in the ways real mail is:
- a closing letter addressed to the company's outside counsel;
- court captions with three plaintiffs;
- an invoice whose "Remit To" is a bank;
- a GOV.UK letter that names the document it encloses;
- a footnoted FCC notice that cites a dozen other orders by date.

Scans run from 2006 court notices to 2018 agency letters. The 19th-century invoices are
printed billheads filled in by hand.

**Caveats**:
- **Two FTC letters need a local copy.** ftc.gov refuses scripted downloads (HTTP 403, even to
  curl's default agent), so real-letter-01 and real-letter-08 were fetched in a browser. The
  fetch script accepts a local copy only when its bytes match the pinned SHA-256.
- **real-notice-12 is a published sample**, not a real notice: the IRS's CP161 with placeholder
  data. It has a real layout but no real addressee.
- **SEC EDGAR documents are not in the set.** EDGAR serves only requests whose User-Agent
  names a contact email, and the fetch script sends none.

**Run.** One live InternBench run of main at #49:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, the app's flags, 4 threads;
- review gate: token confidence at 0.5.

The worker was built with `--features windows-native`, on a 4-core x86-64 host.

## Results

The scorer counts a name right only when it is one of the gold's names, letter for letter.
Five names it marks wrong are Intern right, read against the document. A sixth was right too,
and the gold has been corrected for it.

**Adjudicated right although the scorer marks them wrong**:
- notice-06: the plaintiff's dismissal, named with it alone.
- notice-08: the dismissal of one named defendant, between the plaintiff and that defendant.
- invoice-15 and office-12: "Invoice - WBTV" and "Invoice - Varibus Corporation", from their
  issuers.
- letter-12: the defendants' titled letter to the court.
- office-15, the corrected one: "Email from UPS", the email UPS sent. The gold now accepts
  "Email" for it, as InternBench's own email gold does; that is the one gold change.

`readings.json` gives each document's reading, both verdicts and the failure classes.

| | Right, scorer | Right, adjudicated | Marked ready | Wrong names ready |
|---|---:|---:|---:|---:|
| **All (58 with a name; 2 should go to review and did)** | 6 | **11 (19%)** | 19 | **12** |
| Letters (21) | 0 | 1 | 3 | 3 |
| Notices (18) | 3 | 6 | 10 | 4 |
| Invoices (16) | 2 | 4 | 5 | 4 |
| Receipts (2) and the statement (1) | 0 | 0 | 1 | 1 |

| Text layer | Right, adjudicated | Wrong names ready |
|---|---:|---:|
| Native PDF (18) | 5 | 3 |
| Office: DOCX, DOC, RTF, ODT, XLS (14) | 2 | 3 |
| Scanned PDF (13) | 1 | 3 |
| Image: JPEG, TIFF (8) | 1 | 1 |
| Text: TXT, EML (5) | 2 | 2 |

Against CUAD's third sample (81% right, none ready wrong), this is a different class of
document, not a regression.

**Why the names are wrong.** These are the 47 wrong names; a name can fail more than one way.
Ready means a wrong name filed without review.

| Class | Names | Ready | Examples |
|---|---:|---:|---|
| Type generic or missing: "Document", a bare "Letter" or "Notice" | 21 | 1 | the FTC closing letter as "Document"; the IRS CP161 as "Notice" |
| A person named instead of the organization | 8 | 2 | PHMSA's warning letter to "Jeffery Coleman", Toledo Refining's manager; the DOJ's letters "from" the official who signed |
| An invoice's customer named instead of its issuer | 8 | 4 | "Invoice - Carl Malamud", the customer on C-SPAN's invoice; Enron's invoice as "Invoice - NorthWestern Generation I, LLC" |
| A court filing's two sides not read from its caption | 8 | 2 | a notice of appeal filed by the defendant named "from" the first plaintiff; a caption laid out as a table |
| A date the document mentions, not its own | 7 | 3 | an FCC notice dated by the violation it describes (2023-10-20), not its release (2024-10-17); an invoice by its air date, not its "Invoice Date" |
| No date read | 7 | 0 | historic invoices; a letter whose date the text never states |
| OCR: a misread name, or a hand it cannot read | 6 | 1 | "Vernon Computer Rentals & Leaging" |
| Type names another document the letter is about or answers | 4 | 3 | an FCC letter ending an "Order to Show Cause" filed as that order; a letter about a "Statement of Conduct" filed as the statement |

**The unsafe ones.** Eleven of the 12 wrong names marked ready fall into a class the engine
can see in the document's own text. The twelfth is an OCR misreading:

- **Invoices (4):** two named the customer. The model gave it the role of customer, but the role
  went unconfirmed, so the name was filed as "Invoice - Customer". One took the air date for the
  invoice's date. One carries the OCR misreading "Leaging".
- **Notices (4):** the caption went unread, the date was an event the notice recounts, or a bare
  "Notice" was filed.
- **Letters (3):** a type that names another document, or a person in place of the company.
- **The spreadsheet statement (1):** filed as one of the invoices it lists.

These are the next work, measured on this set and on a held-out one.

## Afterwards: rules written from this set (in-sample)

Six deterministic validation rules followed, one for each class the unsafe names showed. They
are measured on the same recording, replayed, so this set no longer measures how they carry:
a held-out set of the same kinds will.

| Rule | Unsafe names it sends to review or corrects |
|---|---|
| **An invoice's customer is never named as its issuer.** When the first party the document names is one the reply called the customer, and nothing confirms an issuer, no party is named and the name goes to review (`PARTY_MISSING`). A party-less invoice always goes to review. | invoice-11, office-13 to review; on InternBench, the remittance advice filed under its recipient on four recordings, and a party-less invoice on one |
| **A bare "Notice" goes to review**, as a bare "Letter" does. | notice-12 to review; InternBench's rent-increase notice on two weaker models' recordings |
| **A letter, notice or invoice is dated by its own date field.** This applies when the reply's date has no field anywhere ("On October 20, 2023, ESPN transmitted ...") and the first page gives exactly one own-date field: "Released:", "Dated:", "Date:", an "Invoice Date" column, two-digit years too. A date in any field stands ("PO Date"); a line item's dated row does not count; a later page's "Dated:" (an exhibit) never does. | notice-14 and invoice-13 right and ready |
| **A type read below a letter's salutation**, other than a kind of letter, **goes to review**: the letter is talking about it. | letter-14 ("Order to Show Cause") and office-08 ("Statement of Conduct") to review; InternBench's letter enclosing a trade credit application on two recordings |
| **A caption is read as a block** when no label stands beside a name: the parties before "v." down to "Plaintiff(s)", and those after it down to "Defendant(s)", including captions laid out as tables. | notice-01 and notice-06 right and ready; notice-09 right |
| **A letter to a person at an organization is to the organization**, the one its inside address names under the person's name and title. | letter-05 right and ready ("Warning Letter to Toledo Refining Company, LLC") |

| | Before | After (in-sample) |
|---|---:|---:|
| Right, scorer | 6/58 | 12/58 |
| Right, adjudicated | 11/58 | 15/58 |
| Marked ready | 19 | 13 |
| **Wrong names ready** | **12** | **3** |

**The cost.** invoice-15 ("Invoice - WBTV") was right in substance and goes to review without a
party: the reply called WBTV, its issuer, the recipient.

**The 3 still ready and wrong:**
- invoice-07: OCR's "Leaging".
- notice-10: arguable; the movants who gave notice rather than the debtor.
- office-14: a spreadsheet of outstanding invoices filed as one of them.

**No regressions elsewhere:**
- **CUAD:** no name changes on any of the three samples.
- **InternBench:** no right name is lost on any of the ten recordings. Unsafe-ready falls on
  seven of them; on the committed recording it goes from 2 to 1, and `bench/baseline.json`
  moves with it.

**What the run also showed.** Retrieval never gave the model the line holding the document's
own date. notice-14's "Released:" line and invoice-13's invoice-date row were not in its
context, which is why it took a date it could see. Validation now reads the whole first page
for it; retrieval keeping a first page's date fields would let the model see it too. That is a
prompt change, so it needs a live run.

**Speed.** p50 23.9 s per document, p95 54 s. The slowest was a 23-page scanned letter, at
116 s: OCR alone took 101 s of it. Office, text and spreadsheet files parse in milliseconds.

## Reproduce

```bash
python3 -I scripts/realdocs-eval/fetch.py runs/realdocs    # 60 documents; copy the two FTC PDFs in if ftc.gov refuses
CORPUS=runs/realdocs/corpus GOLD=runs/realdocs/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... THREADS=4 \
  scripts/run-internbench.sh runs/realdocs/run -- --min-token-confidence 0.5
```

InternBench's own report then gives the strict filename score and its unsafe-ready count. The
adjudication is `readings.json`.

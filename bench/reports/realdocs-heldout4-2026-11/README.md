# Intern on a fifth real-document set, held out from every rule: November 2026

Status: measured on 2026-11-09. Every rule in the engine has now been written, or tuned, against
the four earlier real sets ([the first](../realdocs-2026-11-02/README.md),
[the second](../realdocs-heldout-2026-11-02/README.md),
[the third](../realdocs-heldout2-2026-11-03/README.md) and the fourth, `real4`, of the
[model qualification](../model-qualification-2026-11/README.md)). This set has had no rule written
from it. It is 57 more real documents from issuers and collections none of those sets used,
gathered on 2026-11-09, and their gold was written before Intern read any of them. The run below
is main's engine (`ebc5d8f`, v0.1.0-alpha.14) with the shipped 9B, so this is main's held-out
score.

**The answer:**
- **Main names 16 of 52 right (31%)** at the gate it ships with (0.7), against 42%, 50%, 30% and
  45% on the four earlier sets. Leaving out the six 19th-century images and the two scanned forms
  from 1982 and 1990, it names 16 of the 44 modern documents scored right (36%).
- **It files little, and 2 of the 4 names it files are wrong.** At 0.7, 52 of the 56 recorded
  names go to review; the two wrong ones filed are an OFAC enforcement release named "between
  U.S. Department of the Treasury and Gracetown, Inc." with the release date inside the type, and
  an OMB memorandum named "for Heads of Executive Departments and Agencies". At the workflow's
  gate, 0.5, the same run would file 9, and 7 of them wrong.
- **The model reads modern documents' types and dates well and their parties badly.** Of the 48
  modern documents read, the type is right in 44, the date in 43 of the 44 that print one, the
  parties in 29. The names are lost on parties and on exact type wording: memoranda named for
  their addressees, letters named the wrong way round, a release's heading (date included)
  copied into the type, people named instead of their bodies.
- **Images and scanned forms are lost entirely**: none of the eight is named right, and each goes
  to review, which is right.
- **All six documents whose gold expects review went to review.**

## The set

57 real documents, pinned in `scripts/realdocs-eval/sample-heldout4.json` by source URL, size and
SHA-256 (45.2 MB in all). `python3 -I scripts/realdocs-eval/fetch.py OUT sample-heldout4.json`
fetches and checks them; on 2026-11-09 all 57 downloaded from their sources with the script's own
User-Agent and matched their pins. The candidate benchmark workflow takes the set as `real5`.

**How it was gathered.** One agent gathered and read every document, under the earlier sets' brief
and these rules:
- no document, issuer or source collection used by `sample.json`, `sample-heldout.json`,
  `sample-heldout2.json` or `sample-heldout3.json`. That rules out GOV.UK, CourtListener, the FCC
  political files, the Internet Archive, the Library of Congress, Public.Resource.Org, MuckRock,
  the NRC, the FTC, the EPA and every other agency those sets drew on;
- licences that plainly permit redistribution where possible (US government works, the Open
  Government Licence, Creative Commons), so that this set's recording can be committed later;
- at most four documents per issuer; no sec.gov; a generic User-Agent; no workaround for any bot
  check or access block; nothing with a private individual's identity, bank, card or medical
  details, nothing about a minor, no family-law or criminal matter.

| Kind | n | Formats |
|---|---:|---|
| Letter | 22 | PDF 17, scanned PDF 4, JPEG 1 |
| E-mail | 1 | PDF (an Outlook message printed to PDF) |
| Notice | 11 | PDF 11 |
| Invoice, receipt, statement | 8 | JPEG 5, PDF 2, XLSX 1 |
| Form | 3 | scanned PDF 2, DOCX 1 |
| Memorandum, minutes, report | 12 | PDF 9, scanned PDF 1, DOCX 2 |

By text layer: 40 native PDFs, 7 scanned PDFs with an OCR layer, 6 JPEG images, 4 office files.

**Sources** (each entry in the sample gives the document's page and exact licence):

| Source | Documents | Licence |
|---|---:|---|
| US federal: CFTC (two staff letters and two companies' request letters it published), NTSB, CMS, HUD, OMB, NLRB, OFAC, FRTIB, NIST, NARA (two scanned records-disposition forms), FAA, TreasuryDirect | 26 | US government works, except the two request letters, which are the companies' (no licence stated) |
| US district courts (S.D.N.Y., N.D. Tex., W.D. Tenn., D.S.C.): public notices | 4 | US government works |
| New Zealand: Commerce Commission warning letters, the New Zealand Gazette, the Climate Change Commission, the Ministry for Regulation, MFAT | 10 | Crown copyright under CC BY 4.0 or CC BY 3.0 NZ (MFAT's own statement not checked); invoices attached to OIA responses are third-party |
| Welsh Government (gov.wales, not GOV.UK): two FOI responses, minutes (DOCX), statistics (XLSX), and an e-mail and an invoice it released under FOI | 6 | Open Government Licence v3.0; the released e-mail and invoice are third-party with no stated terms |
| Northern Ireland: Department for the Economy (a DOCX bulletin), Department for Communities (a DOCX template) | 2 | Open Government Licence v3.0 |
| EU: ESMA, the European Data Protection Board | 3 | reuse authorised with acknowledgement |
| Penn Libraries (Kaplan Collection): four billheads 1882-1896, a receipt 1891, a letter 1883 | 6 | No Copyright - United States |

**The gold** was written from each document alone (read with `pdftotext`, LibreOffice or by eye on
the image; Intern's engine and no model read any of them before the gold was fixed), with the
earlier sets' conventions: the defining date and its role, the date traps with their reasons,
the parties and their relation, every defensible alternative type and party set, and the parties
a careless reading would name. Six documents expect review because a careful clerk could not name
them without guessing:
- two court notices that print no date of their own (W.D. Tenn., D.S.C.);
- a bulletin dated by month only;
- an FAA letter whose rubber-stamped year is smudged (2017 or 2011; both accepted);
- a blank declaration template;
- a statistics workbook that prints no document type.

Three documents name nobody: the Treasury debt report and the Welsh minutes print no issuer, and
the template has none.

**Second opinion.** Before the run, 15 documents chosen at random (seed 20261109: invoice-06,
letter-01, -04, -07, -09, -12, -14, -19, memo-02, notice-01, -05, -07, -09, office-02, -04) were
re-read against their gold. Two changed:
- letter-04 (NTSB): the notes said the file's later letters went to "two other recipients"; they
  go to Triton Logistics Incorporated and the Commercial Vehicle Safety Alliance, now named in the
  notes and as traps.
- letter-14 (ESMA): the notes now record the salutation "Dear Mr Berrigan, dear Sean," (no change
  to the answer).
The other 13 stood. Two more checks changed answers:
- Before the second opinion, while writing the gold: parties accepted for the Treasury report and
  the Welsh minutes and workbook that the text never prints (only the file metadata or the
  publisher's page does) were removed, and the 1883 letter's addressee, whose surname cannot be
  read, is no longer guessed.
- After it, a check that every gold party occurs in its document's text found that NLRB
  memorandum GC 25-05 prints only "Office of the General Counsel" and the National Labor
  Relations Act, never the Board. Its gold still names the Board, inferred, but now expects
  either reading rather than a ready one. This was committed after the run was dispatched; the
  figures below use the corrected gold, and no name's verdict depends on it.

**Caveats:**
- **No openly licensed EML or MSG file was found.** The one e-mail is an Outlook message the Welsh
  Government printed to PDF.
- **Office files are few (4).** Without GOV.UK, few open-licence publishers post Word or
  spreadsheet originals; two of the four are review cases.
- **Sites refused, and were left alone:** UK Parliament committees and the Senedd (a bot check),
  the ICO, Ofwat, GAO, CBO, oversight.gov, the Department of Education, USDA and the Army Corps of
  Engineers (403), the Ministry for the Environment and EECA in New Zealand (an Incapsula check),
  and the Wikimedia Commons API (429, rate limited).
- **Left out for the content rules:** an ASIC infringement notice describing a claimant's medical
  condition, a BIS charging letter to a private individual, a Pensions Regulator notice naming
  individuals in a pension-liberation case, two Commerce Commission letters about children's
  nightwear, and an NTSB letter about a worker's death.
- **Historic material is 8 of the 57 documents** (the six Penn items, 1882-1896, and the 1982 and
  1990 forms); the rest is from 2015 to 2026.

## The run

One live run on a GitHub-hosted runner (`candidate-bench.yml`, run 42577779669, branch
`zack/realdocs5` at `d21399c`, whose engine is main's `ebc5d8f`; only data was added):
- model: Qwen3.5 9B Q4_K_M (SHA-256 `03b74727...e8`), the shipped accurate model;
- runtime: the pinned upstream llama.cpp build the Linux package stages, the app's flags,
  4 threads; worker built with `--features windows-native`;
- runner: AMD EPYC 7763, 4 vCPUs, 16 GB;
- review gate: the run used the workflow's 0.5; the figures here replay its recording with main's
  `intern-bench` at 0.7, the gate the 9B ships with, and at 0.5 for comparison. The two replays
  name every document the same way; only readiness differs.

**One document was not read.** `real5-notice-06`'s host (the Northern District of Texas)
answered 404 to the runner at 07:41 UTC, though it served the same bytes to this session's
`fetch.py` before (07:05) and after (09:05). The run scored the other 56; it stays in the set, and
a later run may need it saved by hand, as `fetch.py` allows.

## Results

Each name is adjudicated by the earlier sets' rule (a name with no joining word is right only when
its party issued the document; a party shortened only by a legal-form word is right). No name the
scorer marks wrong is right on that rule here, so the adjudicated and scorer counts agree. Two come
close and stay wrong: "Letter to Mr. Moritz Körner" (an honorific added) and "Letter to
Directorate General Financial Services" (the Directorate-General's name cut short).

| | Gate 0.7 (shipped) | Gate 0.5 |
|---|---:|---:|
| Read | 56 of 57 | 56 of 57 |
| **Right** | **16/52 (31%)** | 16/52 |
| Type / date / parties right | 44/56, 43/52, 32/56 | same |
| Marked ready | 4 | 9 |
| **Wrong names ready** | **2** | **7** |
| Gold expects review, sent to review | 6/6 | 6/6 |

For comparison, the same engine and model at 0.7 on the earlier sets (the round's replay kit,
`kit/main-ebc5d8f/summary.txt`): 23/55, 28/56, 16/53 and 25/55 right, with 2, 0, 0 and 0 wrong
names ready.

**By kind and text layer, at 0.7** (right / scored, ready, wrong ready):

| Kind | n | Right | Ready | Wrong ready |
|---|---:|---:|---:|---:|
| Letters and the e-mail | 23 | 10/23 | 0 | 0 |
| Notices | 11 | 2/8 | 2 | 1 |
| Invoices, receipts, statements | 8 | 0/8 | 0 | 0 |
| Forms | 3 | 0/2 | 0 | 0 |
| Memoranda, minutes, reports | 12 | 4/11 | 2 | 1 |

| Text layer | n | Right | Ready | Wrong ready |
|---|---:|---:|---:|---:|
| Native PDF | 40 | 13/37 | 4 | 2 |
| Scanned PDF | 7 | 2/7 | 0 | 0 |
| Image (JPEG) | 6 | 0/6 | 0 | 0 |
| Office (DOCX, XLSX) | 4 | 1/2 | 0 | 0 |

**The 16 right:** two of the three Commerce Commission warning letters, an NTSB letter, two CFTC
no-action letters ("CFTC Letter from CFTC" and "from Commodity Futures Trading Commission"), all
three CMS approval letters, both Welsh FOI responses ("Letter from Welsh Government"), two Gazette
notices, both HUD mortgagee letters, one set of FRTIB minutes, and the Welsh minutes (no party, as
the gold wants). All but two (a Gazette liquidation notice and the FRTIB minutes) are in review.

**Why the 36 wrong names are wrong** (`readings.json` gives each; a name can fail more than one
way; the last column counts names the 0.5 gate would have filed):

| Class | Names | Ready at 0.5 |
|---|---:|---:|
| Type generic or missing ("Document", a bare "Letter" where the gold needs more) | 16 | 0 |
| No date read (all on images and scans) | 9 | 0 |
| OCR: a 19th-century hand or a scanned form | 8 | 0 |
| A memorandum's addressees named instead of its issuer ("for Heads of Executive Departments and Agencies", "to Regional Directors, ...") | 4 | 3 |
| Relation reversed (a request letter named "from" the agency it is addressed to; a reply named "to" its own sender) | 4 | 0 |
| A heading copied into the type with its date or weekday ("Enforcement Release December 4, 2025", "Minutes of the Tuesday") | 5 | 3 |
| A person named instead of the organization | 3 | 0 |
| No party named | 3 | 0 |
| Enforcement releases named "between U.S. Department of the Treasury and" their subject | 2 | 2 |
| A party the document only mentions (an attachment's issuer, a request's subject) | 3 | 0 |
| An invoice's customer named as its issuer | 1 | 1 |
| Other (a type cut short, a parent office or agency, OCR noise or a source code read as a party, a form's addressee, a party cut short, an honorific) | 9 | 1 |

**The 2 wrong names filed at 0.7:**
- **OFAC's Gracetown, Inc. enforcement release**, as "2025-12-04 Enforcement Release December 4,
  2025 between U.S. Department of the Treasury and Gracetown, Inc.". Its type repeats the date and
  its parties add the Treasury, so the name is wrong twice. The other two OFAC releases are
  named the same way and wait for review.
- **OMB memorandum M-25-14**, as "2025-01-29 Memorandum for Heads of Executive Departments and
  Agencies": the addressee line, not the Office of Management and Budget that issues it.

At 0.5 the gate would also file the ShapeShift release, both NLRB memoranda (to their addressee
group), the Welsh invoice (from the customer, Neath Port Talbot Council) and a set of NIST minutes
named "Minutes of the Tuesday for NIST".

**Evidence in the prompt.** The gold's date form and parties reached the prompt in 0.78 of the
documents on average (real4: 0.90). Twenty documents lost some gold evidence, and 17 of the 19
scored among them are named wrong. Ten of the twenty are images and scans whose OCR missed the
date or party; five are gold artifacts (an issuer printed only in a letterhead graphic, as with
HUD, or not printed at all, as with the NLRB, or printed letter-spaced, as with the S.D.N.Y.); the
rest are retrieval losses on readable text.

**Speed.** p50 72.2 s per document, p95 132 s, at most 228 s; 74.5 minutes for the run
(model load included); the server's peak resident memory was 8.5 GiB.

## What this suggests

- **The held-out score is about where the hardest earlier set was.** 31% right here, against
  30% to 50% on sets the rules were written from. The rules carried to new issuers only as far
  as the earlier sets' document shapes go; memoranda, enforcement releases and minutes, which
  those sets barely had, are where the names are lost.
- **The unsafe cases are new shapes, not old ones.** Both wrong names filed at 0.7, and five of
  the seven at 0.5, are memoranda and enforcement releases: a memorandum is "from" its issuer even
  when its first line says "MEMORANDUM FOR ...", and a release's heading date is not part of its
  type. Any rule for them must be written now from this set, which then stops being held out; a
  sixth set would be needed to measure it.
- **Parties, not types or dates, are the 9B's weak field on modern documents** (29 of 48 against
  44 and 43). Relation direction on request and reply letters, and naming a person where the body
  issues, are the recurring causes.
- **The review gate held:** every document whose gold expects review went there, and at 0.7 the
  gate filed 2 wrong names of 4.

## Reproduce

```bash
# The set (57 documents, 45.2 MB), checked against the pins:
python3 -I scripts/realdocs-eval/fetch.py runs/real5 sample-heldout4.json
# The live run on a runner (the shipped 9B, 4 threads):
gh workflow run candidate-bench.yml --ref zack/realdocs5 -f sets=real5 -f threads=4 \
  -f candidates='[{"label":"qwen35-9b-q4km","url":"https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q4_K_M.gguf","sha256":"03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8"}]'
# Its recording, replayed at the shipped gate with main's engine:
cargo build --release --locked -p intern-bench
target/release/intern-bench run --corpus /nonexistent --gold runs/real5/gold.json \
  --replay recording.json --min-token-confidence 0.7 --output report.json --markdown report.md
```

The recording (`recording.json`, 3.3 MB) is the run's artifact
`candidate-qwen35-9b-q4km-real5-42577779669`, kept 14 days. It is not committed here; most of the
set is openly licensed, so it could be, with the documents' licences listed beside it.
`readings.json` gives each document's name at both gates, the gold name, the verdicts, the prompt
recall and the failure classes.

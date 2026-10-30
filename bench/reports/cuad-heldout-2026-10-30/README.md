# Intern on real contracts it was not tuned on: a held-out CUAD sample, October 2026

Status: measured on 2026-10-30. The CUAD rules of #45 were written from the 42 contracts of
[the first sample](../cuad-sample-2026-10-29/README.md), so that sample no longer says how well
they carry to contracts nobody looked at. This one does.

**Sample.** From CUAD v1 (The Atticus Project; CC BY 4.0): every 12th of its 510 contracts,
sorted by file name, starting at the 7th. That gives 41 contracts with a PDF, none of them in
the first sample. It is pinned in `scripts/cuad-eval/sample-heldout.json` by dataset revision,
size and SHA-256, with CUAD's own answers. `scripts/cuad-eval/make-sample.py` builds either
sample; at offset 0 it reproduces the first exactly.

**Run.** One live InternBench run of main at #41 (the rules of #45), replayed for every change
below:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, the app's flags, 4 threads;
- review gate: token confidence at 0.5.

**Scoring.** `scripts/cuad-eval/score.py --sample sample-heldout.json`, as for the first sample.
Three readings it marks wrong are Intern right, all filed without review, and the adjudicated
rows count them as right:
- **cuad-18.** CUAD names "J. Walter Thompson"; the contract's party is "Ford Motor Media, a
  division of J. Walter Thompson ("FMM")", which Intern names.
- **cuad-22.** CUAD's type is the preamble's "MARINE TRANSPORTATION AGREEMENT". The contract's
  own heading is "TRANSPORTATION SERVICES AGREEMENT", which Intern now uses.
- **cuad-26.** CUAD names "Melavante Corporation". The contract says "Metavante" 516 times and
  "Melavante" never.

## Results

| | Main (#45's rules) | This change |
|---|---:|---:|
| Names right, adjudicated | 24/41 (59%) | **32/41 (78%)** |
| Wrong names marked ready, adjudicated | 0 | **0** |
| Marked ready | 16 | 22 |
| Type exactly CUAD's | 36 | 38 |
| Scorer as is: right / wrong but ready | 23 / 1 | 29 / 3 |

**The rules generalize on safety, not yet on coverage.** On contracts no rule was written from,
main filed no wrong name. It named 59% right, against 71% on the sample its rules came from.

The fixes below were written from this sample's failures, so the 78% after them is no longer a
held-out figure; the first sample is unchanged by them (30/42, adjudicated). The next honest
measure of these rules is a third sample, at another offset.

**What the held-out sample showed, and what was done:**

| Contracts | What went wrong (all to review) | Fix | Now |
|---|---|---|---|
| cuad-15, -22, -32 | The model gave a bare "Agreement". The title it cited was cut at an organization's noun: "SPONSORSHIP AND SERVICES AGREEMENT", "TRANSPORTATION SERVICES AGREEMENT" and "SUPPLY AGREEMENT" each read as "Agreement", because "Services" and "Supply" also end company names | An agreement's or a contract's title stops only at a company's legal form ("Inc", "LLC", "Co", "Corporation", ...). Other kinds still stop at an organization's noun ("Wrenfield Restaurant Supply Invoice" is an invoice) | Full titles, ready |
| cuad-20 | A cover page's title wrapped over two headings, "JOINT DEVELOPMENT AND MARKETING" and "AGREEMENT"; the model cited the second | A heading that is only the kind of document is read with the heading just above it, when that is in the same capitals | "Marketing Agreement", shortened to fit both parties' names; right |
| cuad-34, -11 | An agreement filed "with" one party: the model left out the other, or named it as the contract never writes it | When an agreement keeps one side and that side is one of the two organizations its own "between" clause names (one that "This ... Agreement" opens, naming two organizations and no third), the clause's other organization is the other side. A recital's "between" is never used, nor a clause that also names a side's parent, and with no side left nothing is taken | Both parties named; 34 ready, 11 to review on low confidence |
| cuad-08, -26 | The model gave the right date, and Intern withheld it as not in the document: "is made this 22 day of June 2018", "made as of the 26 day of January, 2007". A filled-in blank's day had to carry its ordinal ("22nd") to count | A day of a month without its ordinal supports the date too. The forms also guide retrieval, so cuad-26's prompt changed and it was re-run live | Dated; 26 ready, 08 to review on low confidence |

The title fixes are not CUAD-specific: on InternBench they complete
`agreement-two-column-footnotes` on 7 of the 10 recordings, and
`msa-effective-date-in-definitions-12p` on one of them. No name is lost and no wrong name is
newly marked ready on any of the 10.

**Still wrong (all to review):**
- **A day the contract leaves out** (16): "executed this Agreement on the st day of JUNE ,
  2006". The model guessed the 1st, as CUAD's annotator did; Intern withholds it.
- **Wrong date or party, caught by the gate** (09, 19, 21). cuad-21 named the two companies of
  a share purchase its recitals mention; cuad-19 named a signatory for the other company.
- **One party, the other unreachable:**
  - cuad-14 ran two names together ("CyberShop Excite, Inc.");
  - cuad-38 lists its parties on a cover page, one per heading;
  - cuad-35 and -41 are templates with a blank party.
- **Many-party filings** (30): a joint filing agreement with five filers.
- **CUAD blanks** (35, 41): CUAD has no date for these; Intern gives the one the draft or exhibit
  header states. Both are in review.

`readings.json` has every contract's reading before and after.

## Reproduce

```bash
python3 -I scripts/cuad-eval/fetch-sample.py runs/cuad-heldout sample-heldout.json   # 41 PDFs
CORPUS=runs/cuad-heldout/corpus GOLD=runs/cuad-heldout/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... THREADS=4 \
  scripts/run-internbench.sh runs/cuad-heldout/run -- --min-token-confidence 0.5
python3 -I scripts/cuad-eval/score.py runs/cuad-heldout/run/report.json --sample sample-heldout.json
```

The worker must be built with `--features windows-native` (its PDF reader); without it every
contract fails as `NATIVE_ASSETS_MISSING`. The live run took 17 minutes on a 4-core x86-64 host:
25.4 s per contract median, 51 s for the slowest. Replays ran beside it for part of the run, so
these times are an upper bound.

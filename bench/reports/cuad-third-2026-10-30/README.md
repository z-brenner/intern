# Intern on a third set of real contracts, written from by no rule: October 2026

Status: measured on 2026-10-30. #45's CUAD rules were written from [the first
sample](../cuad-sample-2026-10-29/README.md) and #47's from [the second](../cuad-heldout-2026-10-30/README.md),
so neither still measures how those rules carry to contracts nobody looked at. This sample
does, and it is the honest figure for Intern on real contracts today.

**Sample.** From CUAD v1 (The Atticus Project; CC BY 4.0): every 12th of its 510 contracts,
sorted by file name, starting at the 4th. That gives 42 contracts with a PDF, none in either
earlier sample. It is pinned in `scripts/cuad-eval/sample-third.json` by dataset revision, size
and SHA-256 (`make-sample.py` with offset 3).

**Run.** One live InternBench run of main at #47, replayed for the change below:
- model: Qwen3.5-2B Q4_K_M;
- runtime: upstream llama.cpp b11946, the app's flags, 4 threads;
- review gate: token confidence at 0.5.

**Scoring.** `scripts/cuad-eval/score.py --sample sample-third.json`. Five readings it marks wrong
are Intern right, and the adjudicated rows count them as right. (The scorer read an answer that
opens with "(" as missing its first party; it no longer does, which made cuad-01 right.)

| Contract | CUAD says | The contract says (Intern) |
|---|---|---|
| cuad-04 | "Berkely Lights, Inc." | "Berkeley Lights" three times, "Berkely" never |
| cuad-28 | "COMPLEHON AGREEMENT AND UQITDITV MAINTENANCE AGREEMENT" (OCR) | COMPLETION AND LIQUIDITY MAINTENANCE AGREEMENT |
| cuad-34 | "STRATEGIC ALLIANCE AGREEMENT is" | Title: STRATEGIC ALLIANCE AGREEMENT |
| cuad-37 | "TRANSPORTATION SERVICE AGREEMENT Contract Identification FT18150" | Title: TRANSPORTATION SERVICE AGREEMENT; the identification is the next line |
| cuad-38 | "Pifzer Inc." | Pfizer Inc. |

## Results

| | Main (#47's rules) | This change |
|---|---:|---:|
| Names right, adjudicated | **33/42 (79%)** | 34/42 (81%) |
| Wrong names marked ready, adjudicated | 2 | **0** |
| Marked ready | 25 | 24 |
| Scorer as is: right / wrong but ready | 28 / 5 | 29 / 3 |

**How the rules carried.** Before #47, the second sample, unseen then, read 59% right. Here,
unseen, main reads 79%: the title, other-side and date rules written from one sample of
contracts held on another. Safety held as well. The two wrong names main filed were patterns
the earlier samples had shown, and this change sends both to review:

| Contract | What went wrong (ready) | Fix | Now |
|---|---|---|---|
| cuad-15 | Filed as a bare "Amendment". Its title is "AMENDMENT NO. 3", and it names the agreement it amends only in its first paragraph. The first sample's cuad-06 failed the same way | A bare "Amendment" or "Addendum" goes to review, as a bare "Agreement", "Letter" or "Document" already did: which agreement it changes is the name's point | To review (first sample's cuad-06 too) |
| cuad-39 | A Schedule 13G "JOINT FILING STATEMENT" filed for one of its five filers. "Statement" made it a record, which names one party | A joint filing statement or agreement is an agreement among its filers (Rule 13d-1(k)), so it is named between two of them, or goes to review with one | Between two of its filers |

**Afterwards, from this sample (so in-sample for it).** cuad-18 was typed a bare "Agreement",
citing page 2's heading "AGREEMENT", under the cover page's "Strategic Alliance Agreement". A
bare kind was completed from the document's title only when the reply's word stood in running
text; a cited heading that is itself the bare kind now completes it too. cuad-18 is right and
ready. Neither earlier sample nor InternBench changes. That makes this sample 35/42 (83%)
adjudicated; 79% stays the unseen figure.

**Still wrong (all to review, 7):**
- **A bare "Amendment"** (15), above.
- **One party, the other unreachable** (14, 08, 29): the party's name is not as the contract
  writes it, or the contract is a schedule or a statement of work under another agreement.
- **Parties** (21): the model wrote both parties as the contract never does.
- **A date a week off** (31), and a garbled reading (40); the gate sent both to review.

`readings.json` has every contract's reading before and after.

## Across the three samples

| Sample | Rules written from it | Right, adjudicated | Wrong names ready, adjudicated |
|---|---|---:|---:|
| First (`sample.json`, 42) | #45 | 30/42 (71%) | 1 (cuad-10's EDGAR filing stamp) |
| Second (`sample-heldout.json`, 41) | #47 | 32/41 (78%) | 0 |
| **Third (`sample-third.json`, 42)** | **none (this change's two came from patterns the first two showed)** | **34/42 (81%)** | **0** |

## Reproduce

```bash
python3 -I scripts/cuad-eval/fetch-sample.py runs/cuad-third sample-third.json   # 42 PDFs
CORPUS=runs/cuad-third/corpus GOLD=runs/cuad-third/gold.json MANIFEST= MODEL=.../Qwen3.5-2B-Q4_K_M.gguf \
LLAMA_SERVER=.../llama-server INTERN_RUNTIME_DIR=... THREADS=4 \
  scripts/run-internbench.sh runs/cuad-third/run -- --min-token-confidence 0.5
python3 -I scripts/cuad-eval/score.py runs/cuad-third/run/report.json --sample sample-third.json
```

The worker must be built with `--features windows-native`. The live run took 15.3 s per
contract median on a 4-core x86-64 host, 64 s for the slowest.

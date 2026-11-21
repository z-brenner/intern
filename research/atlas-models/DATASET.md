# ATLAS training data

Status, 2026-11-21: the pipeline is built and verified locally on 1,000 generated documents
(all 1,000 labels confirmed by the engine). The first CI build of the data is part of training
run `2026-11-21-lora1` (see TRAINING.md). No dataset file is committed: the generator is
deterministic, so any split is rebuilt from its seed, and CI keeps each build as a 30-day
workflow artifact.

## What an example is

One example is exactly one exchange the engine has with the model, as ATLAS sends it today:

- the system turn (SYSTEM_INSTRUCTION + COMPACT_INSTRUCTIONS) and the user turn (the evidence
  lines with their `[p1.b4.f2]` handles), captured byte for byte from `intern-bench` running the
  real worker over the document;
- the GBNF grammar the engine sent;
- the reply: the compact JSON form (`type`, `date`, `parties`, optional `subject`), each fact
  citing the handle of the evidence line that prints it.

So a model trained on it learns the production prompt, not a paraphrase of it, and the reply
already passes the engine's grammar and validation.

## How it is built

| Step | Tool | What it does |
|---|---|---|
| 1. Generate | `dataset/generate.mjs` | Writes N synthetic PDF/DOCX/TXT documents and an InternBench gold file. 3 in 5 PDF, 1 in 5 DOCX, 1 in 5 TXT. File names are `scanNNNNN.ext`, so the prompt carries no file-name hint. |
| 2. Capture | `dataset/fake_server.py` + `intern-bench` | Runs the real worker and engine over the corpus against a stand-in server that records each request. No model runs. |
| 3. Label | `dataset/build_targets.py answers` | Builds the reply from the gold and the captured evidence lines only. |
| 4. Verify | `dataset/verify.sh` | Serves those replies back, lets `intern-bench` parse, validate, compose the file name and score it. A document is kept only when the engine's own name is scored right. Party roles are tried in up to nine orders, because the engine's composition (not the label script) decides "to", "from", "for" or "between". Orders 0-2 keep only names that join the gold's own party set; 3-8 also accept the gold's documented alternative sets. |
| 5. Export | `build_targets.py export` | Writes `sft.jsonl`: `{id, family, format, messages, grammar, filename}`. |

Local result on 1,000 documents (seed `atlas-2026-11`, split `train`): 920 kept at order 0,
55 at order 2, 25 only under an acceptable alternative (mostly company Notification Letters the
engine composes "from" the sender, and receipts). 1,000 of 1,000 verified in about 2.5 minutes
on 4 CPUs.

## What the documents are

Seven families, weighted to the mix of the real development sets:

| Family | Weight | Gold name | Traps in the body |
|---|---|---|---|
| Agency letter to a company | 30 | to recipient, issuance date | inspection date, response-by date, earlier letter, named contact, signer, cc list, sometimes outside counsel |
| Company letter to an agency | 12 | to agency (from sender acceptable) | the company's own people, dates of what it answers |
| Court notice | 12 | between plaintiff and defendant, filing date | complaint filing date, hearing date |
| Enforcement notice | 14 | for respondent, notice date (to acceptable) | violation and inspection dates |
| Invoice | 18 | from vendor, invoice date | due date, service period, attention contact, ship-to |
| Receipt | 6 | from vendor (for payer acceptable) | payment method and dates |
| Statement | 8 | from vendor (for customer acceptable) | period, due date |

Every name, organisation, address and event is invented from word pools in the generator.

## Contamination controls

- **Test names refused.** The generator reads `bench/gold.json` and every
  `scripts/realdocs-eval/sample*.json` gold (real1-real8) at start and refuses any document
  whose party, role name, forbidden party or acceptable party matches one of them. 4 of the
  first 1,004 candidates were refused this way.
- **No shared templates.** It imports only `bench/lib` (PDF, DOCX and gold writers), never
  `bench/docs`, so no InternBench document's text or layout is reused.
- **Validation never repeats a training entity.** The validation split is generated with
  `--avoid train/gold.json` and refuses any document whose answer parties appear as answer
  parties in the training split (17 of 217 candidates for a 200-document split).
- **Real documents are never training data.** No real document's text was copied.
  real7 and real8 are never screened and are refused by the workflows.

## Honest limits

- The engine verifies that each label is consistent with the gold and with how the engine
  composes names. It cannot catch a convention error in the generator itself: if the generator's
  idea of the right party for some family is wrong, every label of that family is consistently
  wrong. The real sets are the check on that.
- The families and naming conventions were modelled on what the real1-real3 gold expects, so
  real1-real3 are not clean held-out sets for a model trained on this data. real4-real6 are
  cleaner (their conventions were not read to write the generator), which is why training runs
  report on real4-real6 and the synthetic validation split.
- Synthetic documents are cleaner than scans. Nothing here exercises OCR; it trains the
  reading of the evidence lines the worker already extracted.

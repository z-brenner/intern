# Other models, rules, speed and corporate records: research, November 2026

Status: research summary, 2026-11-15, after v0.1.0-alpha.19. Three questions were asked:

- is there another local model that would be better and faster;
- can regular expressions or other deterministic rules make reading faster or more accurate;
- can the standard for renaming without a person be lowered, for a user who keeps a
  company's books and records in-house.

Five research passes ran, and four of them were then checked by an independent pass whose
job was to find what was wrong. Each finding says whether it was **measured here** (counted
from this repository's recordings or code), **checked** (a fact read from its primary page
on 2026-11-15) or **reported** (a vendor's or author's claim nobody here has run). The
research passes ran no model. One screen was run afterwards, of the one candidate they
found, and its result is in section 1. The third question has its own report:
[`bench/reports/review-levels-2026-11`](../bench/reports/review-levels-2026-11/README.md).

Development documents here means InternBench (77) and the real sets 1 to 6. The two held-out
sets were not opened by any research pass.

## The short answers

| Question | Answer | Confidence |
|---|---|---|
| A better model? | None is known. No large lab has released a small general model since the 2026-11-03 survey. | Checked, not exhaustive |
| A faster model? | No. The one 4B-class family never run here, Spark-X, was screened on 2026-11-15: 60 of 77 names right against 69 for Qwen3.5 4B and 72 for the 9B in the same run, and more wrong names filed at every gate. | Measured here, one run, InternBench only |
| A faster runtime? | No untried flag and no newer llama.cpp build is a free win. Two things have never been measured and need a real laptop: Windows throttling a minimised model on battery, and the integrated GPU. | Measured here / checked |
| Rules instead of the model? | No. Rules alone name 14 of 25 documents exactly where they can name one at all; the pipeline names 17 of the same 25. At most 15% of model time would be saved. | Measured here |
| Rules as a veto or an override? | No. An override spoils more than it fixes; a veto would have held 7 right names and caught none of the wrong ones. | Measured here |
| Where rules do help | They already do: the engine's rule layer lifts the type from 202 to 224 right of 247. What is left wrong is mostly the party, which rules read poorly. | Measured here |
| Is the product measured on this user's paper? | Barely. 338 of 422 development documents are vendor, regulator, court or public-sector paper; 14 of 41 corporate record types have no example at all. | Measured here |

## What to do next, in order

1. **Build a corporate-records development set** (about 74 documents; plan below) and measure
   the shipped engine on it with "Your organization's names" set. Every other decision here,
   a model switch included, is a guess until this exists. Cost: about one earlier real set's
   work, plus an own-names option in the bench.
2. **A path for the company's own records.** Charters, bylaws, consents, minutes, policies
   and plans have one party, the company itself, and often no date; today they are held for
   review by construction (read from the code, not run). Probably the largest single change
   in what files by itself for this user. Needs set 1 to measure.
3. ~~Screen Spark-X2.6-4B against same-run 4B and 9B controls.~~ Done on 2026-11-15: it reads
   worse than the 4B it would replace (section 1). No model change is proposed.
4. **Measure on a laptop**: ten documents in front on mains, ten minimised on mains, ten
   minimised on battery. An afternoon; possibly the largest speed number in this document, in
   either direction.
5. **Do not build** a model-free naming path, a rule veto, a rule override, a leaner reply
   format or a smaller prompt for mid-size documents now. Reasons below.

## 1. Other models

### What a candidate has to meet (measured here, from the code)

- Memory: 8,192 MiB must be available before the accurate model loads and 6,656 MiB before
  the quick one (`tiers.rs`). Both downloads together stay under 10 GiB (`manifest.rs`;
  8.42 GB today).
- Runtime: upstream llama.cpp b11946, CPU only, an 8,192-token context. Every request carries
  a GBNF grammar, token probabilities, temperature 0 and thinking off (`client.rs`).
- Licence: commercial use at any company size.
- The review gate does not carry over between models: the 2B shipped at 0.5, the 9B needed
  0.7 and then 0.8, and the 4B filed about one wrong name per right one at every threshold.
  A new model needs its own sweep.

### The bar (measured here)

On the alpha.17 engine at a 0.7 gate, InternBench plus real sets 1 to 5: the 9B named 200 to
202 of 335 right and filed 5 to 7 wrong of 59 to 60; two 9B runs differed by up to 2 right
names. The 4B named 159 to 165 of 291; two 4B runs differed by 6. A 4B reading takes 52% to
58% of a 9B reading's time on the same CPU model, and 53% to 59% of its memory (not "under
half": only the file is).

### What is new since the last survey (checked 2026-11-15)

- No new small general model from Qwen, Google, Microsoft, Mistral, Meta, DeepSeek, IBM,
  NVIDIA or Liquid AI. The pinned Qwen files are unchanged upstream and their hashes equal
  the manifest's.
- **Spark-X** (Hub organisation XHToken), which the earlier survey missed. Apache-2.0.
  - Spark-X2.5-4B: 4.11B parameters; Q4_K_M 2,600,224,352 bytes.
  - Spark-X2.6-4B, released 2026-10-19, "exactly the same architecture" with further
    training, supersedes it. Plain Q4_K_M from bartowski: 2,621,104,768 bytes, SHA-256
    `30b77e2ede85c37da7071048c91fcf47c3e8cbfad51971640530f8ba9753301a`, commit
    `17682e38297a4510899b12a95ab3d8dcebc674e5`. The vendor's own file of the same name is
    0.94 GB larger and is not a plain Q4_K_M.
  - Reported, on the vendor's card: instruction following 93.0 against 91.5 (Qwen3.5 9B) and
    89.8 (4B). On the rows nearest to reading a document it is level with the 4B: long-context
    reasoning 56.3 against 57.0 (4B) and 63.0 (9B). All its figures are in thinking mode at
    temperature 1.0; the app runs thinking off at temperature 0 under a grammar, and this
    repository has already found that such figures predicted nothing.
  - Both open questions about running it were settled by the screen below: b11946 loads it
    with the app's flags, and the server reuses the cached instruction block (for LFM3 it did
    not, at 3 to 5 s a document).
- **NuExtract3** (a Qwen3.5-4B fine-tune for extraction, Apache-2.0) is still unrun here and
  cannot be screened as the workflow stands. Its chat template picks its mode from a
  `template` argument the app does not send; without it the model runs in its
  document-to-Markdown mode. A fair run needs its own prompt builder, an output adapter and a
  recording: days, not hours. Its reported lead over Qwen3.5 9B (0.651 against 0.479) is
  mostly invalid JSON from the general models (170 and 229 failed outputs against 27), a
  failure the app's grammar already prevents.
- **Bonsai 2 9B** (ternary Qwen3.5 9B, Q2_0, 2.56 GB): the 2B and 4B of this family were
  measured here and rejected (37 and 48 of 77 against the 2B's 52, and slower). The 9B was
  never run. It would be a quick-tier candidate for machines under 8 GiB, not a faster one;
  a run needs a newer llama.cpp with controls on the same build and the `bonsai_2` prompt
  profile. Low priority.
- **Decision models** (small models that score a yes or no question) are new since
  September. The only measurement of the idea here is one run in October on the 2B's names
  (8 of 14 wrong names held for 3 of 46 right, threshold chosen on the same data). Nobody has
  measured it on the 9B's output, and it only ever adds holds, which is the opposite of what
  was asked. The one small enough to sit beside the 9B is `knowledgator/gliclass-v4-edge`
  (0.44B, Apache-2.0, 8-bit ONNX 647 MB; the worker already ships ONNX Runtime). Worth an
  offline script before anything else, not a commitment.
- Not worth a run: Gemma 4 E4B, Nemotron 3.5 Bolt E4B, Granite 4.1 and 4.2 8B (screened
  2026-11-03, none led); LFM3, LFM2.5 and d1 (revenue cap in the licence); MiMo-V2.6 distil
  (a 9B, so no faster, tuned for agents and code); anything 27B and up.

### The screen, as run (measured here, 2026-11-15)

`candidate-bench.yml` run 43750632377 on `main` (v0.1.0-alpha.19), InternBench only, four
threads, llama.cpp b11946, input `candidates`:

```json
[
  {"label": "spark-x2.6-4b-q4-k-m", "url": "https://huggingface.co/bartowski/Spark-X2.6-4B-GGUF/resolve/17682e38297a4510899b12a95ab3d8dcebc674e5/Spark-X2.6-4B-Q4_K_M.gguf", "sha256": "30b77e2ede85c37da7071048c91fcf47c3e8cbfad51971640530f8ba9753301a"},
  {"label": "control-qwen3.5-4b-q4-k-m", "url": "https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/resolve/e87f176479d0855a907a41277aca2f8ee7a09523/Qwen3.5-4B-Q4_K_M.gguf", "sha256": "00fe7986ff5f6b463e62455821146049db6f9313603938a70800d1fb69ef11a4"},
  {"label": "control-qwen3.5-9b-q4-k-m", "url": "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q4_K_M.gguf", "sha256": "03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8"}
]
```

The three recordings were then replayed through the alpha.19 bench at three gates. A gate
changes which names are ready, not the names.

| Of 77 | Spark-X2.6-4B | Qwen3.5 4B | Qwen3.5 9B |
|---|---:|---:|---:|
| Names right | 60 | 69 | 72 |
| Type right | 72 | 73 | 75 |
| Date right | 73 | 76 | 74 |
| Parties right | 74 | 76 | 77 |
| Relation right (who sent it, who it is for) | 67 of 74 | 76 of 77 | 77 of 77 |
| Ready at 0.5, and wrong among them | 52, 5 | 47, 3 | 63, 2 |
| Ready at 0.7, and wrong among them | 39, 4 | 26, 1 | 37, 1 |
| Ready at 0.8, and wrong among them | 33, 4 | 15, 1 | 28, 0 |
| Peak server memory | 3.29 GB | 3.34 GB | 6.23 GB |

The ready rows show what the gate alone would do. In the app a quick-tier reading is never
ready, whatever the gate.

What it shows:

- **Spark-X2.6-4B is below the 4B on every fact**, and 9 names below it, where the switch
  rule at the end of this section asks for more than 6 above. It named 12 documents wrong
  that the 4B named right and 3 the other way. Of the 12: four dates (two long reports left
  undated, one given a bare year, one form given a different day), five relations (four name
  the party the document is for or to as its sender, one calls a bank statement an agreement
  with the bank), two invoices with the sender left out, one newsletter named by its masthead.
- **Its confidence marks its mistakes worse.** The four wrong names still ready at 0.8 are
  those four senders: the customer on a statement, the lender of a loaned painting, the
  insurer on a loss notice, the supplier paid on a remittance advice, with a least token
  probability of 0.82 to 0.92. The 9B has no wrong name ready at 0.8 and the 4B one. Spark
  files no wrong name only from a gate of 0.93, which leaves 15 ready against the 9B's 28 at
  its shipped 0.8, and that 0.93 is read off these same 77 documents.
- **It runs.** b11946 loads it with the app's flags and grammar, all 77 readings completed, and
  the server reused the cached instruction block (508 cached tokens at the median; this was
  the open question after LFM3).
- **Speed was not measured like for like.** The three jobs landed on three CPU models (Xeon
  6975P-C for Spark, EPYC 9V45 for the 4B, EPYC 9V74 for the 9B). Spark took 32.3 s a
  document, the 4B 18.0 s and the 9B 45.1 s; Spark read 56 prompt tokens a second and wrote
  7.2, the 4B 98 and 10.9. The one figure in the same run that does not depend on the model,
  OCR time, was 28% longer on Spark's machine, which accounts for some of that gap and not
  all of it. Spark also writes more: 106 reply tokens a document against 90. Nothing here
  suggests it is faster than the 4B; whether it is slower was not established.

Decided: no second pass on the real sets and no model change. The 4B-class slot it was a
candidate for is the quick tier, and it reads worse than the model already there.

Limits of this screen: one run; InternBench only, which is synthetic and is the set the
engine's rules were fitted on with Qwen models, so exact names favour the incumbents
([`model-candidates.md`](model-candidates.md) says to read type, date and parties for that
reason, and those are lower too, relation most of all); two 4B runs have differed by 6 names
before. A 9-name deficit with a worse relation score and worse-marked mistakes was judged
not worth a second pass (this one took 2.1 runner-hours for 77 documents). That is a
judgement, and the workflow input above reruns it.

The rule for any later candidate stands: a 4B-class model replaces the 9B only if, on
InternBench plus real sets 1 to 6 and in two runs, it names at least as many right as the
same-run 9B, files no more wrong names and takes at most 0.6 of its time. It replaces the
quick tier if it beats the same-run 4B by more than the 4B's own run-to-run spread (6 names)
in both runs. Never screen on `real7` or `real8`.

## 2. Speed without a new model

### Where a document's time goes (measured here)

One document is one model exchange: 1,204 of 1,204 recorded document-runs, no retries. With
the shipped flags (`--no-repack`, alpha.18 engine, 209 documents on four runner CPUs):

| Part | Share of time |
|---|---:|
| Reading the prompt (prefill), about 860 new tokens | 59.8% |
| Writing the reply, about 91 tokens | 35.6% |
| Extraction and OCR | 4.5% |

The split moves with the processor: prefill is 45% to 77% across the recorded runs, and on
the Zen 5 runner the reply already costs more than the prompt. Both costs are linear
in tokens with no fixed part. The 477-token instructions are served from the cache on every
document. Every one of these numbers is a Linux cloud runner at 4 threads on 2 cores: there
is no Windows, laptop or 8-thread measurement in the repository.

### What is already in the product, or already rejected

Cached instructions, `--no-repack`, every logical processor up to eight and slot restore are
shipped. Measured and rejected, with reports under `bench/reports/speed-linux-cpu-*`: two
documents at once, speculative decoding in five forms, shorter line ids, batch and polling
flags, an 8-bit KV cache, Q4_K_S, Q5 and Q8 files, prompt compression. The quick-then-accurate
cascade stays off: a quick reading costs 0.52 to 0.58 of an accurate one and waits for a
person whenever it is kept, so it would pay only if more than half of documents stopped at
the quick model, and stopping there is the opposite of filing without a person.

### Levers considered

| Lever | Ceiling | Why not now |
|---|---|---|
| A leaner reply (56% of reply tokens are line ids, punctuation and keys) | 9% of a document if every id went, 20% if every id, key and punctuation token went | The one measured attempt (ordinal ids) cut reply tokens 24%, saved 1% and lost 8 right names. Unresolved, and needs a full live accuracy run. |
| A smaller prompt for mid-size documents | 8.2% of all time for a hard cap at 1,000 new tokens | The retrieval sweep in [`evidence-retrieval.md`](evidence-retrieval.md) already found fact recall falling from 99.2 to 96.9 for a 4% smaller context. Worst for long agreements. |
| Skipping the model where rules settle the name | 15% of model time | The rule names are worse (section 3). |
| A newer llama.cpp | None expected | The x86 Q4_K kernel work is still open upstream; one change merged since the pin is reported 7% to 28% slower on AVX2 for one path. Re-check when pull request 29397 merges. |
| A 4,096-token context | Memory, not time | The largest recorded prompt is 2,827 tokens. Would let more machines keep the accurate model. Testable today with `extra_server_args`; the memory freed was not measured. |

### Two things nobody has measured (checked: the mechanisms exist)

- **Windows power throttling.** Microsoft documents that the descendants of a minimised or
  hidden window get low quality of service, and that on battery this means the most efficient
  frequency and the efficiency cores. The model server is a child of the app, started below
  normal priority, and the app lives in the background. If this bites, the fix is small (tag
  the server, behind a "full speed on battery" setting). It cannot be tested on a runner.
- **The integrated GPU.** llama.cpp ships OpenVINO and Vulkan builds of the same server for
  Windows. The OpenVINO backend lists this model family as passing on CPU and GPU and failing
  on the NPU, gives no speed figures, and does not support state save and restore, so the
  cached instructions and the slot file might be lost. A first look is by hand on a laptop;
  adopting it would be a second runtime package and a full accuracy run.

## 3. Regular expressions and other rules

### What the engine already does with rules (measured here)

The model proposes the type, the date, up to three parties and a subject. Rules alone read
the amount and the identifier and compose the relation word, the filename and the
description. Rules check every fact the model gives against the text, and stand in for it in
a few cases (a title as the type, an email header's date, the other side of a between
clause). This layer lifts the type from 202 to 224 right of 247 on the first four
development sets and leaves the date about where the model had it.

### A prototype of rules as the reader (measured here)

About 400 lines of standard-library Python (date, type and party readers), written on
InternBench and real sets 1 to 3, frozen, then run once on real sets 4 to 6. Those three sets
were new to the prototype but not to the engine, so the engine's side of each comparison is
in-sample.

| Fact | Rule fires (of 161) | Rule right when it fires | Rule right, pipeline wrong | Pipeline right, rule wrong |
|---|---:|---:|---:|---:|
| Date | 93 | 87 (94%) | 2 | 2 |
| Type | 105 | 102 (97%) | 4 | 1 |
| Party | 30 | 17 (57%) | 2 | 10 |

- **Skipping the model.** All three rules fire on 25 of 161 documents. Their name is exactly
  right on 14; the pipeline's is right on 17 of the same 25. Those documents carry 15% of
  model time. Not worth a second naming path.
- **Overriding the model.** It would fix 2 dates and spoil 2, fix 4 types and spoil 1, fix 2
  parties and spoil 10. On the first four sets it fixes 0 dates and spoils 5.
- **Vetoing a ready name.** Of 13 ready names on these sets one was wrong. A rule disagreed
  with two, both right, and agreed with the wrong one: the blank invoice template whose date
  cell holds `TODAY()`. The rule read the same cached value the model did. A second reading
  of the same text is not an independent check.
- **Confirming a fact so its hold can be dropped.** Among names held with a type reason, the
  type is right 48 of 50 times when a rule agrees and 2 of 23 when no rule fires. That is a
  real signal about the type. But dropping the type holds on agreement makes 0 names ready on
  the first four sets and 3 on the other three, because the same names are held for the token
  probability or the party as well. For the date there is nothing to clear: on 0 of the 63
  documents with a date reason does the rule agree with the pipeline.
- **Kinds of document where the rules are confidently wrong:** blank forms and templates
  (the rules cannot say "this has no date": 0 of 2 right), and minutes ("CALL TO ORDER" read
  as an order; the date the earlier minutes were approved taken for the meeting's).

### The limit is the party (measured here)

Of the 13 really wrong names on real sets 4 to 6 whose date and type a rule confirmed, 12 are
wrong in the party: a person where the organisation was wanted, or the sender where the
addressee was. Rules read the organisation in a letter's inside address right 13 of 16 times
and a person there 2 of 10. The next accuracy work belongs here, and a small entity model for
person against organisation is the candidate the repository already lists
([`model-candidates.md`](model-candidates.md), section 3). New since then (checked): a Rust
crate, `gliner2` 0.1.3, loads that model on the CPU with no ONNX Runtime. Its licence and its
accuracy on legal text are unverified, and it would add a model, not remove one.

### For agreements, the rules look strong and are unproven

On the 20 agreements in development the between clause gives the parties 19 of 19 times and
the opening clause the date 15 of 16. All 20 are synthetic. The one miss is a statement of
work whose opening clause carries the master agreement's date, which is exactly the mistake a
contracts file cannot afford. Real agreements with their text are the first thing the
corporate-records set would supply.

## 4. Fit to corporate books and records

### What the development documents are (measured here)

| Kind | Of 422 |
|---|---:|
| Public-sector letters, notices, forms and reports | 159 (38%) |
| Vendor paper: invoices, bills, receipts, orders, quotes | 95 (23%) |
| Regulator letters and notices | 58 (14%) |
| Litigation paper | 26 (6%) |
| Everything else | 84 (20%) |

Among the 84: minutes 12 (one of them a company board's), memoranda 10, emails 8 (every real
one a public body's), services and supply agreements 6, leases 6, and one each of a
separation agreement, an offer letter, a data processing agreement and a company written
consent, all four synthetic. No example at all of: a charter, bylaws or an operating
agreement, an officer's certificate, an org chart, a cap table or stock ledger, a stockholder
or investor agreement, a SAFE, note, warrant or term sheet, an equity plan or grant, an NDA,
terms of service, an invention assignment, a privacy policy, an internal policy. The three
contract samples from CUAD are 125 commercial contracts and no governance, equity, employment
or policy record.

"Your organization's names", the setting an in-house user depends on, is scored by no
benchmark: the bench has no reference to it.

### Where the engine will struggle (read from the code, not run)

1. **The company's own records are held by the party rules.** A record with one party of
   unsettled role is sent to review, and an agreement with one party is held as one-sided.
   "Terms of Service" is an agreement by keyword. A static copy of three engine rules over
   141 typical corporate titles gives the class "unknown" to 33: bylaws, articles, board
   resolutions, proxy, org chart, cap table, stock ledger, power of attorney, SAFE, warrant,
   term sheet, 83(b) election, handbook, code of conduct among them.
2. **Dates.** The date cues have no "adopted", "last updated", "as amended through" or
   "approved by the board", and "Last updated:" is deliberately not a date label (written for
   a price list). Org charts, cap tables, bylaws and templates often have no date, and an
   undated document is never renamed without a person. For a privacy policy "last updated" is
   usually the date to file by: that is a decision for the owner.
3. **Many parties.** A name carries at most two. A stockholders agreement becomes "between
   the company and the first investor", and "with the first investor" once own names are set.
4. **Titles refused as parts.** A heading that starts with "Confidential" or "Section" is
   never a title, which refuses the invention assignment agreement every employee signs and
   the 83(b) election.
5. **Emails.** The name is "Email to (organisation)"; fifty emails from one colleague share
   it. The subject reaches only the description.
6. **Formats.** `.html` (a policy saved from the website) and `.vsdx` (a Visio org chart) are
   not read.
7. **Affiliates.** Own names match by prefix, so a contract signed by a subsidiary loses
   which entity signed.
8. **Drafts, signature pages, stand-alone exhibits.** Nothing in a name says the file is one.
   A draft filed looking like the signed agreement is the really wrong case worth holding.

### A corporate-records development set

About 74 documents: charters 3, bylaws 3, written consents and resolutions 4, board minutes
3, a notice of stockholder meeting, officer's and good standing certificates 2, org charts 4,
cap tables and ledgers 3 (synthetic: no public source was found), stockholder agreements 4,
SAFE, note, warrant and term sheet 3, equity plan and grants 4, NDAs 3, services agreements
2, statements of work and an order form 3, an amendment and an amended and restated
agreement, DPAs 2, a lease, offer letters and an employment agreement 3, invention
assignments 2, separation and termination papers 4, privacy policies and terms 4, internal
policies 3, emails 6, a memorandum, a board deck, and a draft, a signature page and a
stand-alone exhibit.

Sources checked on 2026-11-15:

- Openly licensed: `basecamp/policies` (CC BY 4.0), `Automattic/legalmattic` (CC BY-SA 4.0),
  `github/site-policy` (CC0), Common Paper's standard agreements (CC BY 4.0), ContractNLI's
  607 NDAs (CC BY 4.0), CUAD (CC BY 4.0).
- Fetch by URL, never committed: SEC EDGAR exhibits. The full-text search answered for all 18
  phrases tried (charter, investors' rights, stockholders agreement, master services
  agreement, statement of work, separation agreement, offer letter, option grant, insider
  trading policy, code of ethics, DPA, written consent, organizational chart, office lease,
  NDA, indemnification agreement, subsidiaries). The exhibit pages refused an undeclared
  client, so none was opened: the fetcher must declare itself as the SEC asks.
- Licence not verified: NVCA model documents, Y Combinator's forms, Bonterms, the UCSF
  industry documents library, the Enron email archive.

Method as for the real sets ([`internbench.md`](internbench.md)): a sample file pins each
document by URL, licence, size and hash; gold is written after the rules are frozen and read
a second time blind. Two additions: a second gold name as filed with own names set, and a
recorded answer on what an in-house filer accepts for an undated record. The limit: public
exhibits are public-company paper with conformed signatures. No private company's minute book
is public. A listing of a few hundred real filenames from the owner's folders would correct
the inventory faster than anything here.

## What was not verified

- One model was run, once, on InternBench (section 1). Every other speed and memory figure
  for a candidate is scaled from this repository's measurements of Qwen3.5 files. The screen's
  three jobs ran on three CPU models, so its times compare nothing.
- llama.cpp's release notes and pull requests were read through a summarising fetch tool, not
  as diffs; GitHub pages were not reachable for the checkers.
- The rules prototype is one draft and one revision, not the engine. The engine's own readers
  were not measured on their own.
- Engine gaps 1 to 8 are read from the code and a static copy of three rules. None was
  observed in the app.
- Licences were read from model cards and README files, not from licence texts.
- The corporate inventory is general practice for an in-house team, not the owner's folders.

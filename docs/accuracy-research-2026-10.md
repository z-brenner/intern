# Making Intern more accurate: research, October 2026

Status: research summary plus the measured changes of 2026-10-29. The question was
"there has to be something we can do model-wise or pipeline-wise to increase accuracy".
Three research passes covered:
- other models;
- fine-tuning;
- pipeline and prompting techniques.

Their findings were checked against Intern's own data. Each finding below says whether it
was **measured here**, **checked** (a fact verified on the web or in the code) or **reported**
(a researcher's citation, not independently re-read: treat its numbers as leads, not facts).

## Where the errors are (measured here)

Qwen3.5-2B, production prompt, 77-document InternBench, before the change below: **52/77
names right; 15 wrong names marked ready** (filed without review). The gold type, date and
party were present in the model's context for **77/77** documents, so the errors are
choices, not missing information.

| Bucket | Wrong names | Of which ready |
|---|---:|---:|
| A person who works for the organization named instead of it (buyer contact, board president, conservator, inspector, signatories, winemaker, CEO) | 9 | 7 |
| Generic or wrong type ("Letter", "Agreement", "email"; prompt words echoed: "Evidence") | 7 | 1 |
| Relation word: 3 "to" where the gold says "for", 2 direction reversed | 5 | 5 |
| Date | 1 | 0 |
| Wording | 3 | 2 |

Speed was measured the same day, in
[`bench/reports/speed-linux-cpu-2026-10-29`](../bench/reports/speed-linux-cpu-2026-10-29/README.md).
Shorter ids, two documents at once and speculative decoding do not pay on a 4-core CPU. The
upstream llama.cpp runtime reads prompts 36% faster than the packaged one. Its batch size and
polling were measured again on the AVX2 and AVX-VNNI paths laptops load: every setting fell
within run-to-run noise, so the app's flags stay.

## What was tried and measured

| Change | Result | Verdict |
|---|---|---|
| **Organization over staff, record relation "for"** (deterministic, `facts.rs`/`compose.rs`, PR #40) | Qwen production runtime **52 → 59** names, unsafe **17 → 11**; committed recording 54 → 62, 14 → 7; no right name lost on any of 8 recorded runs × 77 documents (5 models), apart from one name the earlier code had right by luck | **Shipped** |
| **Issuers: an order from its buyer, a header's organization before the kind of document, a title's type** (deterministic) | **+3 names and 2 fewer unsafe on every Qwen run** (committed 62 → 65, 7 → 5; production runtime 59 → 62, 11 → 9); no name lost on any of 9 runs | **Shipped** |
| **An insurance document is for its insured** (deterministic, `facts.rs`): "Named insured", "Insured", "Policyholder", "Member", "Patient" and "Subscriber" support the client role, as a field label or a table's column header ("Crew member" and other staff groups never do) | **+1–2 names on all five Qwen runs, 1 fewer unsafe on three** (committed 65 → 66, 5 → 5; production runtime 61 → 63, 8 → 7; upstream 63 → 65, 6 → 5); +1 name on 4 of the 5 other models' runs; no name lost on any of 10 runs. A loss notice on the insurer's form is no longer filed "from" the insurer. | **Shipped** |
| **A court document is filed between the two sides of its case** (deterministic, `facts.rs`/`compose.rs`): a plaintiff, petitioner, appellant or claimant and a defendant, respondent or appellee, each labelled beside its name, in a document with "A v. B"; letters and emails about a case are unchanged. A role word no longer completes a name ("Plaintiff Gilchrist ... LLC") | The court hearing notice right on 9 of 10 runs (8 were wrong and ready); +1 name and 1 fewer unsafe on all five Qwen runs (committed 66 → 67, 5 → 4); no other document changed | **Shipped** |
| **A record or a notice is for the side its label names** (deterministic, `facts.rs`): a reply's bare "recipient" gives way to a client, tenant or other side-it-is-about role that the document's own label gives the party ("Prepared for ...") | The watershed report right on the 3 runs that had it wrong and ready; committed 67 → 68, 4 → 3; no other document changed on 10 runs | **Shipped** |
| **A record its staff made comes from the organization they act for** (deterministic, `facts.rs`): when the reply named staff (an examiner, an inspector) and another organization, the first page's organization is added as the record's issuer | The loan condition report right on 6 of 10 runs (all were wrong and ready), and one maintenance record; committed 68 → 69, 3 → 2; no other document changed | **Shipped** |
| **Real contracts (CUAD sample, 42 SEC filings; `bench/reports/cuad-sample-2026-10-29`): a field's label never completes a name; a defined term dates the document it defines; an agreement naming one party goes to review (`PARTY_MISSING`); an agreement's title reads across "and" and shortens to the agreement it ends on** (deterministic) | CUAD, adjudicated: names 28 → 30 of 42, **wrong names ready 5 → 2**, exact types 34 → 37. InternBench unchanged on all 10 recordings (Qwen runs exactly; three other models' runs shorten one title the gold also accepts) | **Shipped** |
| **Held-out real contracts (41 more CUAD filings, none in the first sample; `bench/reports/cuad-heldout-2026-10-30`): an agreement's title stops only at a legal form, so "Supply", "Services" and "Agency" stay in it; a title wrapped over two headings is read whole; an agreement left with one side takes the other from its own "between" clause; "the 22 day of June 2018" states a date; an email's header block supports the type "Email"** (deterministic) | Main on the held-out sample, adjudicated: 24/41 (59%) right, **no wrong name ready**. After: **32/41 (78%), still none**; ready 16 → 22. First sample unchanged. InternBench committed recording 69 → 71 (the seed agreement's title; the approval email), unsafe unchanged | **Shipped** |
| **A third set of real contracts, no rule written from it (42 CUAD filings; `bench/reports/cuad-third-2026-10-30`): a bare "Amendment" or "Addendum" goes to review; a joint filing statement is an agreement among its filers** (deterministic) | Main, unseen: **33/42 (79%) right, 2 wrong names ready**, against 59% on the second sample before #47: the rules carried. After: 34/42 (81%), **none ready wrong**; the first sample's bare "Amendment" (cuad-06) goes to review too, leaving its EDGAR stamp as the one wrong name ready across 125 contracts | **Shipped** |
| **Real letters, notices and invoices, no rule written from them (60 public documents in ten formats: scanned and native PDF, JPEG, TIFF, DOCX, DOC, RTF, ODT, XLS, TXT, EML; `bench/reports/realdocs-2026-11-02`)** | Main, unseen: **11/58 (19%) right, adjudicated; 12 of the 19 names marked ready are wrong**, against 81% and none on contracts. The wrong names: a generic type 21, a person for the organization 8, an invoice's customer for its issuer 8, a court caption unread 8, a date the document mentions rather than its own 7 | **Measured; the next work** |
| **What the real letters, notices and invoices showed** (deterministic, `facts.rs`/`compose.rs`/`validate.rs`; in-sample, `bench/reports/realdocs-2026-11-02`): an invoice's customer is never named as its issuer; a bare "Notice" goes to review; a letter, a notice or an invoice is dated by its own first-page date field ("Released:", an "Invoice Date" column) when the reply's date has none; a type read below a letter's salutation goes to review; a caption is read as a block ("Plaintiffs, v. ... Defendant"); a letter to a person at an organization is to the organization | Real set, in-sample: **15/58 right (from 11), wrong names ready 12 → 3**. InternBench: no right name lost on 10 recordings, unsafe down on 7 (committed 2 → 1). CUAD: unchanged on all three samples | **Shipped** |
| **Held-out real letters, notices and invoices, no rule written from them (60 more public documents, the first real `.xlsx` and `.msg` among them; `bench/reports/realdocs-heldout-2026-11-02`)** | Unseen: main **12/60 right, 7 wrong names ready**; with #51, **14/60 and 5**. The rules carried, with one change for the worse, in review: a widening that let the reply's unconfirmed "customer" role pick an invoice's issuer from the page's head, now reverted (wrong names ready 5 → 4 here and 3 → 2 on the first set; InternBench and CUAD unchanged). Two TIFFs stored with FillOrder 2 could not be opened | **Measured; the revert shipped** |
| **A notice dated at its end** (deterministic, `facts.rs`; in-sample): a letter's, notice's or invoice's own-date field counts on any of its own pages, up to the first page that opens as an attachment ("EXHIBIT A", "SCHEDULE A"), not on the first page alone | Held-out set: 14 → **16/60** right, wrong names ready 4 → **3** (FERC's notice of technical conference right and ready); three more documents carry their own date; CUAD and InternBench unchanged | **Shipped** |
| **The document's own date in the model's context** (retrieval, `index.rs`/`infer.rs`; a prompt change, re-run live): a "Released:" or "Notice date" field labels the own date; a two-digit year is read under a label that names the issue date ("Invoice Date", "Dated:"), never a bare "Date:" or a line item | Context date recall on the first real set 51 → 52/59; five prompts change on the real sets and CUAD, none on InternBench. Live: the model's own date right on 2 more of the 5 (validation had already corrected both names); no name changes except one right name sent to review | **Shipped** |
| **A letter's inside address names its addressee; a title stops at "pursuant"** (deterministic, `facts.rs`/`phrases.rs`; in-sample): a document of no known class laid out as a letter is read like a letter; a line under the addressed person that is a party the reply named counts as their organization; the inside address overrides a reply that called that organization the sender | Held-out set: 16 → **19/60** right, wrong names ready 3 → **1**; first set, CUAD and InternBench: no right name lost | **Shipped** |
| **A third real set, unseen by every rule (59 letters, notices and invoices from new issuers, gathered after #51, #54 and #55; `bench/reports/realdocs-heldout2-2026-11-03`)** | Main before #51: **6/58 right, 7 wrong names ready**; as of #55: **7/58 (12%), 5**. The rules carried, but only a little. The set is harder than the earlier two (historic hands, office files). The dominant failures are a generic type (23 names), no date (16), a type naming another document (12) and a person for the organization (11); all go to review | **Measured; the next work** |
| **A letter's layout names it a letter** (deterministic, `facts.rs`; in-sample): when the reply's type is not in the document and no title stands in, a document with a dateline or "Date:" field over a "Dear ..." is a "Letter", still in review | Third set: 7 → **9/58** right; first set +1, held-out +2; wrong names ready unchanged everywhere; InternBench and CUAD unchanged | **Shipped** |
| **The reply's roles stand in where the document supports none; a title's identifier is no part of the type** (deterministic, `compose.rs`/`facts.rs`/`phrases.rs`; measured by replay on every recording, three models): when the rules on the roles the document supports would name no side, the roles the reply proposed are read, for every class but an agreement, unless the document's own role for any party contradicts them, and a name that rests on them always waits for a person (`PARTY_UNSUPPORTED`); an invoice's or notice's number, a dash-set subtitle, a written date or a code before the kind is cut from the type ("Invoice No. 2248", "Quarterly Business Review - Q3 2026") | Qwen3.5-2B: real sets 12/16/8 → **16/17/11** right, wrong names ready 3/2/5 → **1/1/2**; InternBench, the three CUAD samples and the fixtures corpus unchanged, no name lost anywhere. Qwen3.5-4B: real sets 12/11/7 → 18/16/12, InternBench 56 → 60, wrong names ready 10/8/10 → 4/4/3. Qwen3.5-9B: real sets 12/10/7 → **20/19/11**, wrong names ready 11/13/9 → 4/7/2. A bigger model labels roles right far more often than the document's wording confirms them, and the rules written for the 2B threw those labels away | **Shipped** |
| **A stated type reads as its title's kind phrase; a machine stamp is no type; a form is for the party it is filled in for, and the reply's roles stand in where the rules only defaulted** (deterministic, `facts.rs`/`compose.rs`; measured by replay on every recording, three models; PR #60): the field a title line runs into is cut ("CHANGE ORDER Distribution: ..." → "Change Order"), a line's own kind phrase stands for it ("Spring Wholesale Availability and Price List" → "Price List"), never across a connector ("Approval of Closure Plan" stays); a type ending "generated" or "printed" is unsupported and the heading above it read; a form is for its client, customer, tenant or employee, else the counterpart of a supported issuer | Qwen3.5-9B: InternBench 61 → **70** right, wrong names ready 9 → **1**; real sets 20/19/11 → 20/**21**/11, wrong ready unchanged. Qwen3.5-4B: InternBench 60 → **64**, 8 → **5**. Qwen3.5-2B unchanged on every recording; no name lost on any set for any model. Rejected after measurement: reading the reply's roles wherever the rules only defaulted (2B real2 17 → 15), and a kind-phrase cut across a connector (a type right → wrong on real1) | **Shipped** |
| **Qwen3.5 9B (accurate) and Qwen3.5 4B (fast) replace Qwen3.5 2B and Bonsai 2 2B** (`bench/reports/model-qualification-2026-11`): six 4B–9B candidates (Qwen3.5 4B and 9B, Gemma 4 E4B QAT, Nemotron 3.5 Bolt E4B, Granite 4.1 8B and 4.2 8B), each Q4_K_M, run live on InternBench and the three real sets on GitHub runners (`candidate-bench.yml`); the UD-Q4_K_XL and Q4_K_S quants, MTP speculation and n-gram speculation measured and rejected; the 9B's review gate set again on the four real sets (0.7); the 4B's readings held for a person | Qwen3.5 9B, strict names: real sets **22/21/11** of 60/60/59 (2B 16/17/11); the third CUAD sample 19/42 (2B 14); InternBench 70 (2B 71); the fixtures corpus 10/18 (2B 13/18), its three losses all in review. Fourth real set (58 documents), unseen by every rule: 9B **15/58**, 4B 13, 2B 13; parties right 41/38/30; the 9B's losses are types copied from a header or a person's title, all in review. The gate: at 0.5 the 9B files 16 wrong names of 42 on the four real sets, at 0.6 9 of 28, at **0.7 2 of 15** (shipped; InternBench 37 of 77 filed, 1 wrong); the 4B files about one wrong for every right at any gate (0.6: 14 of 25; 0.7: 5 of 12), so every quick reading is held for a person as in alpha.12 Same 26 contract requests on one 4-core box, p50: 2B 15 s, 4B 37 s, 9B 68 s; resident 9B 8.9 GB (3.4 GB anonymous), 4B 4.7 GB, 2B 2.1 GB | **Shipped** |
| **A letter in every layout, and a class as the stand-in type** (deterministic, `facts.rs`; measured by replay on every recording, three models, four real sets): a letter is known by a greeting in any of its forms ("Dear" alone, "Gentlemen:", "To Whom It May Concern", a greeting a table's cells split), a dateline written short ("Dec. 26, 1907", "18th October 2019") or a sign-off over a signature ("Yours faithfully,", "Very respectfully,"), with or without a greeting; an email by its header block and a notice by a heading that opens on the word stand in for a kind the reply could not find, each a bare kind that waits for a person | About fifty real documents had been named "Document" where the gold accepts "Letter", "Email" or "Notice" with their parties. Qwen3.5 9B: real sets 20/21/11/15 to 21/23/14/18 right, the 4B 18/16/12/13 to 20/17/14/13; no name lost, wrong names ready unchanged | **Shipped** |
| **A type copied past its line** (deterministic, `facts.rs`): a bigger model copies a title and runs on past its line, into a wrapped heading's next line or the paragraph under it, until the reply's room for a type runs out ("Final enforcement notice to Euro Car Parks Limited for failure to comply with an"); the part of the reply its line holds, read as a title, is the type, when the reply's next words are the document's next words | InternBench's board minutes right on the 2B, 4B and 9B recordings (69 to 70, 64 to 65, 70 to 71 right); a notice of technical conference's type right on the 9B; no name lost. A reply whose tail is the id it ran into ("OFFICIAL PUBLIC RECORD,p1.b1],") is not a copy and is left as before | **Shipped** |
| **A letter is to its client** (deterministic, `compose.rs`): the Letter class read a supported client, licensee or tenant as "for"; a warning letter is written to the company it warns, so the relation is "to" | Six letters right across the 2B, 4B and 9B (real-letter-05 on both bigger models, real3-office-03, real4-letter-02 twice, real-office-01); no name lost | **Shipped** |
| **A letter's organizations from its inside address, its signature block and its letterhead** (deterministic, `facts.rs`): the inside address names the organization under the person in more ways than an organization's word: a name set in capitals ("OFCOM"), a name written as one ("Daily Telegraph", "Office of Management and Budget"), the organization a post is held at ("Chair, Home Affairs Select Committee"), the organization's line over the person; a British letter's address stands over its dateline; the first person the address names is the addressee, whatever role the reply gave them, and a second addressee's organization never takes their place; a person who signs is replaced by the organization under their post in the signature block, or the letterhead's organization when the signature names none; a person's line, a city's, a street's and a "cc:" line never name an organization | Qwen3.5 9B: real sets 21/23/14/18 to 23/28/16/25 right (the fourth set 15 to 25 of 58); the 4B 20/17/14/13 to 22/19/15/15; the 2B real sets 15/17/13 to 19/20/15. One name lost, in review: a business review request letter to the Antitrust Division, named to the division rather than to the department it is part of (the gold names both). A first version that read any short line of capitalized words as an organization lost 32 names (titles, cities, a typist's initials, a wrapped department's first line) and was tightened until it lost none | **Shipped** |
| **A caption set "against" its sides, with no label beside them, or missing a side; a side's possessive, a tag before a comma, a verb or an aside cut from a title; a trading name cut from a party** (deterministic, `facts.rs`, `phrases.rs`): a caption's sides are read across "- against -" and a table's cells, and with no "Plaintiff" or "Defendant" label at all on either side of its "against"; a notice whose reply kept one side of the case takes the caption's other side; "Plaintiff's Notice of Motion" is a "Notice of Motion", "White Finding, Notice of Violation" a "Notice of Violation", "Notice IS Hereby Given" a "Notice", "Notice of Variation and Consolidation with introductory note" stops at "with" (an agreement keeps its "with"); "Farringdon Energy Limited trading as Champion Energy" is Farringdon Energy Limited | Ten names right: three notices of settlement, motion and appeal filed between their sides, the Britvic notice on three recordings, the Tennessee Valley Authority notice, the Farringdon notice; the 2B's one wrong name ready on the first real set (the PA Advisors dismissal) is right; no name lost | **Shipped** |
| An invoice, a receipt or a statement as the stand-in type when the word stands at the head of the first page | Changed no name on any recording: the historic scans it was written for have no date the engine accepts, and a heading that says "INVOICE" was already read as a title | Rejected (not shipped) |
| A date stated only on a running header or footer is the copy's, not the document's (CUAD's EDGAR "Source:" stamp) | Missed the stamp it targeted, which runs into a different paragraph on each page; read line by line, it withheld a correct date a contract repeats in its own header | Rejected |
| Always file a record "for" its recipient (drop the "to" when a third party may be its subject) | Fixed the property policy on 4 runs, but a certificate whose insured is unlabelled would be filed "for" its holder, which no gold accepts | Rejected |
| **Review gate on token probability** (least probable date or party token below 0.5 goes to review; `docs/model-candidates.md`) | Unsafe 9 → 3 and 8 → 4 on two production-runtime runs, for 9 and 12 right names sent to review; about +1–2% time | **Shipped** |
| Short ordinal line ids (`[3]` for `[p1.b4.r2]`) | Prompt −5%, reply −24% tokens, but −1% time per document; names 58 → 50, trap dates 0 → 5. The model cited line 1 for most facts. | Rejected |
| Plainer prompt scaffold ("1 page, every line:" and `<<<`/`>>>` for "The whole 1-page document follows" and BEGIN/END EVIDENCE) | Names 58 → 58 with 19 of 77 names changed in both directions; unsafe 10 → 12. The echo it targeted is now caught by the issuer rules instead. | Rejected |
| Whole documents instead of retrieval (64K context) | 0 fixed, 2 new unsafe errors (a trap date, a CEO as party); 2–20x slower; the 60–100-page documents timed out or did not fit | Rejected |
| Thinking mode, 1,024-token budget (LFM3 and Qwen) | Both worse (Qwen 52 → 47, unsafe 15 → 21), ~5x slower; neither ever finished reasoning in budget | Rejected (`bench/reports/lfm3-retest-linux-cpu-2026-10-28/`) |
| LFM3-1.7B QAD-Q4_0 as the namer | 51/77, ties Qwen; LFM licence caps commercial use at US$10M revenue | Rejected |

## Pipeline techniques, ranked by expected gain per cost

Ranking from the research pass, adjusted for what was measured since.

1. **Organization outranks a person acting for it.** Shipped in PR #40, measured above.
2. **Class-specific relation words.** The record part shipped in PR #40. Two directional misses
   remain, and they may be the model mislabelling roles:
   - a purchase order "from" its supplier;
   - a loss notice "from" the insurer.
3. **Better routing from token log-probabilities.** Shipped as a review gate (measured above). On
   the recorded replies, the minimum over date and party tokens beat the alternatives tried:
   party names alone, each name's first token, the type, the role words. Reported: log-prob
   confidence ranks errors but does not verify them. First-token aggregation ranked best on
   12–27B models (ConfBench, arXiv 2608.01792), and a single misattributing sentence flipped
   high-confidence extractions in 96–100% of counterfactuals (arXiv 2610.20417).
4. **Party names constrained by the grammar to candidate strings.** This adds a typed
   candidate block, organizations first. Reported: a candidate grammar took LLaMA-7B entity
   disambiguation from 37.4 to 77.2 (arXiv 2305.13971). It is less likely to help here,
   since Intern's errors are choices between real names, not invented ones. Cost +50–150
   prompt tokens.
5. **Organization-first output key and per-class role lists in the grammar.** The direction of
   the effect is inconsistent in the literature; it needs a live run.
6. **Remove words the model echoes from the prompt scaffolding.** Measured and rejected (above):
   any change to the prompt moved about a quarter of the names for no net gain.
7. **Type chosen from the document's title candidates.** Rules out a bare "Letter",
   "Agreement" or "Report". Targets about 5–7 held misses.
8. **A small NER model to tell people from organizations.** GLiNER family, Apache-2.0, ONNX.
   Only if 1 and 4 misclassify.
9. **Same-model yes/no verification.** Reported: P(True) did not help Qwen3.5-2B rank candidates
   (ΔAUROC +0.008), though it helped Qwen3.5-4B (arXiv 2609.33142). A two-step "validate the
   first answer" pass helped 8–70B models but hurt 3–4B ones (arXiv 2607.26780). Low priority.
10. **Better wording of each field's question.** Reported: optimizing the extraction questions
    alone moved clinical extraction F1 by about 18.6 points across five models, more than a
    larger model did (arXiv 2608.31058). It is worth doing only against a held-out set:
    tuning wording on these 77 documents would fit them, not documents in general.
11. **Not recommended:**
    - self-consistency sampling (N× cost, little routing gain);
    - one call per field (+9 s per field);
    - few-shot examples chosen per document type (breaks the cached instruction prefix);
    - thinking (measured worse).

## Models (checked: licences, repos, file hashes)

Qwen has released no small model since Qwen3.5. Qwen3.6 and 3.8 start at 27B (reported). The
models worth benchmarking under Intern's limits (CPU, ~3 GB RAM, commercial licence) are below.
All three files are downloaded and verified against the Hub's SHA-256.

| Model | Licence | Why | Risk |
|---|---|---|---|
| **MiniCPM5-2B** (`openbmb/MiniCPM5-2B-GGUF` @ `2079a22`, Q4_K_M, 1.56 GB) | Apache-2.0 | Vendor IFEval 86.7 and IFBench 66.3, against Qwen3.5-2B's 77.5 and 46.0; plain Llama architecture | Thinking-first post-training; non-thinking quality unproven here; ~1.0–1.25x Qwen's time |
| **Granite-4.0-1B** (`ibm-granite/granite-4.0-1b-GGUF` @ `b27c2fe`, 1.02 GB) | Apache-2.0 | Non-reasoning instruct model, IFEval 77.4; ~0.8x Qwen's time | Weaker knowledge; nothing compares it with Qwen3.5-2B |
| **NuExtract-2.0-2B** (`numind/NuExtract-2.0-2B-GGUF` @ `d038fea`, 0.99 GB) | MIT | Trained for verbatim extraction | 2024 base (Qwen2-VL); no published scores |
| Intern-Decision-2B (`internlm/Intern-Decision-2B`) | Apache-2.0 | A Qwen3.5-2B fine-tune for calibrated yes/no and choice answers; could fill d1's review-gate role without d1's licence | Trained on agent tasks; its official readout is Transformers-only |

Measured live (2026-10-29): 77 documents, upstream llama.cpp b11946, the same machine, scored
with the rules above. MiniCPM5 and Granite were also run with the fast tier's few-shot prompt
profile (`bonsai_2`), which shows a small model the shape of a reply.

| Model, prompt | Names right | Wrong names ready | Time per document (median) |
|---|---:|---:|---:|
| **Qwen3.5-2B, production prompt** | **65** | **5** | 18.8 s |
| MiniCPM5-2B, production prompt | 48 | 2 (4 ready at all) | 24.4 s |
| MiniCPM5-2B, few-shot | 50 | 16 | 22.6 s |
| Granite-4.0-1B, production prompt | 0 (null replies) | 0 | 12.8 s |
| Granite-4.0-1B, few-shot | 47 | 10 | 18.5 s |

Neither replaces Qwen: both name fewer documents and mark more wrong names ready, and neither
is faster once its replies are usable. MiniCPM5's zero-shot replies echoed the prompt's words
as the type and sent nearly everything to review. NuExtract-2.0-2B was not run: it takes its
own `template` request argument instead of Intern's grammar.

Rejected:
- 3–4B dense models (Granite 4.x 3B, Gemma 4 E4B, Ministral 3B, SmolLM3, Phi-4-mini): too
  slow.
- Gemma 4 E2B: ~4.5 GB RSS.
- Small-active MoEs: 3.8–4.8 GB files.
- Hunyuan: its licence excludes the EU, the UK and South Korea.
- Llama 3.2: branding and acceptable-use terms.
- NuExtract-2.0-4B: non-commercial.
- LFM: revenue cap.

## Fine-tuning (reported; needs a decision and a budget)

This is the largest remaining model-side lever, and the one that needs the most care.

- **Evidence (reported).** A LoRA fine-tune of Qwen3.5-2B on 6,125 examples took an extraction
  task from F1 0.49–0.55 to 0.952 (arXiv 2606.08051). NuExtract3, a Qwen3.5-4B fine-tune,
  scored 0.651 against 0.417 for base 4B and 0.479 for 9B. Intern already has a grammar, a
  validator and retrieval, so its headroom is smaller. Estimate: 52 → 58–66 names; no gain
  if the training data does not resemble real documents.
- **Data.** Do not reuse InternBench's generator: new seeds would produce near-duplicates of
  the benchmark. Permissive sources:
  - CUAD (CC BY 4.0);
  - SEC EDGAR exhibits and letters (sec.gov content may be redistributed);
  - US federal documents (public domain).

  FUNSD, DocILE, Kleister and RVL-CDIP are non-commercial or unlicensed.
- **Teacher.** Anthropic, OpenAI and Google terms restrict training competing or distributed
  models on their outputs; a model Intern ships is distributed. Use self-hosted Apache-2.0
  teachers instead (a large Qwen3.5 or gpt-oss-120b). Keep a label only when two labellers
  and Intern's validator agree.
- **Mechanics.**
  - Train a bf16 LoRA with Unsloth (Qwen3.5 support confirmed by the vendor; about 5 GB
    VRAM).
  - Merge, convert to GGUF with the converter of the llama.cpp tag the app ships, and
    quantize to Q4_K_M.
  - Training prompts must be Intern's own byte for byte. Capture them with
    `intern-bench --wire-output`.
- **Cost.**
  - GPUs: a few hundred dollars.
  - Labelling: 40–60 hours of human adjudication.
  - Timeline: a 1-week evaluation set first, a 2–3-week pilot, then 4–6 weeks for the full
    run.
- **Gate.** A held-out set of about 300 real documents, labelled before any training. About
  140 documents can detect +10 points, and about 400 can detect +5. InternBench alone is
  too small and synthetic: 11 vs 12 discordant names give McNemar p = 1.0.

## What the failures are (reported, and consistent with what is measured here)

A preregistered audit of LLM entity extraction (arXiv 2610.20417) found hallucination
behind only 1.2% of false positives. 65.6% were real strings from the wrong context, and
19.9% fell outside the gold set's scope. That matches Intern's errors: the right names are
in the context, and the model picks a person who works for the party, the wrong side of an
order, or a word from the prompt. Its conclusion matches the measurements above. Repairs work
when matched to the failure mode (the deterministic role rules), and confidence can set
review priorities but never settle truth (the gate sends a reading to a person; it does not
change the name).

## Still to measure

- A held-out set of real documents. The first CUAD sample (42 contracts) found failures the
  synthetic set does not have (filing stamps, definitions of other agreements, templates,
  many-party filings). A second, held out from the rules written for the first, measured how
  they carry: no wrong name filed, 59% named right against 71% on the first. A third, held out
  from both, read 79% before any rule came from it: the rules carry. The next honest check is a
  fourth sample, or documents that are not contracts. Both cover only contracts;
  letters, invoices, notices and forms have no real-document check yet.

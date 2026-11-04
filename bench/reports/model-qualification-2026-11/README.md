# Qwen3.5 9B and 4B replace Qwen3.5 2B: the qualification, November 2026

Status: measured 2026-11-03 and 2026-11-04. This is the record of how Intern's local models were
chosen for v0.1.0-alpha.13, what each measurement found, and what was rejected. `readings.json`
holds every document's reading by the three Qwen3.5 models on every set below, with the gold.

**The answer:**
- **Qwen3.5 9B (Q4_K_M, 5.68 GB) is the accurate model and Qwen3.5 4B (Q4_K_M, 2.74 GB) the
  fast one.** Of six candidates in the 4B-9B class, they were the only two that named more real
  documents right than the 2B, and the 9B named the most.
- **On real documents the 9B names about a quarter more right than the 2B**: 22, 21 and 11 of the three sets' 60, 60 and 59 documents named right,
  whole filename, against the 2B's 16, 17 and 11, and 30, 31 and 17 with type, date and parties
  right against 22, 24 and 13.
- **It is surer of its wrong names, so its review gate is stricter and it files less.** At the
  2B's gate the 9B would have filed 16 wrong names of 42 on the four real sets; at 0.7, where it
  ships, it files 2 of 15 (the 2B at 0.5: 5 of 28). The 4B filed about one wrong name for every
  right one at any gate, so, as in alpha.12, every quick reading waits for a person.
- **On a fourth real set that no rule was written from**, the lead shrinks: 15 names right of 58
  against the 2B's 13, with 23 against 18 right apart from the joining word, and 2 wrong names
  filed against 1. The 9B reads the parties better (41 right against 30) and the type worse (38
  against 44): it copies a page header or names a person where the 2B's rules found a kind.
- **It is four to five times slower**, and needs about 10 GB of memory free. The trade was made on
  purpose: a right name in review beats a fast wrong one. Where it does not fit, the 4B reads,
  its names wait for a person, and the 9B reads them again when memory comes back.
- **Two rounds of engine rules were needed to see any of this.** As found, the 9B named fewer
  InternBench documents right than the 2B (55 against 71), because the rules had been fitted to
  the 2B's habits. With the rules of #59 and #60 it names 70, and no 2B name was lost.

## What was asked

A model that is more accurate than Qwen3.5 2B, runs without a GPU on a 16 GB laptop beside a
browser and an office suite, in a quantization llama.cpp reads, under a licence Intern can ship.
Speed may fall several times; review may rise. The question was accuracy first.

## Candidates

The survey (2026-11-03) covered every open model of 3B to 10B parameters released since Qwen3.5
with a usable GGUF and a licence that permits shipping: Qwen3.5 4B and 9B (Apache 2.0),
Gemma 4 E4B (Apache 2.0), NVIDIA Nemotron 3.5 Bolt E4B (OpenMDW 1.1), IBM Granite 4.1 8B and
4.2 8B (Apache 2.0), NuExtract3 (a Qwen3.5 4B fine-tune for structured extraction, Apache 2.0),
Ministral 3B, SmolLM3, Phi-4-mini, and LFM3. Six went to the live screen. Those left out:
- NuExtract3 answers its own JSON template, not Intern's grammar and facts reply; adopting it
  means a new prompt and pipeline, measured separately if at all.
- Dense 3B and 4B models (Granite 4.x 3B, Ministral 3B, SmolLM3, Phi-4-mini) publish nothing
  that puts them above Qwen3.5 4B, which is the same cost.
- LFM3 and the 2B class were measured in October (`docs/accuracy-research-2026-10.md`) and
  did not beat the 2B.

Every vendor figure is a thinking-mode figure; Intern runs every model with thinking off, so none
of them predicted this screen. The one like-for-like document-extraction comparison published
(NuMind's) put Gemma 4 E4B above Qwen3.5 9B, which the screen contradicts for Intern's task.

## The screen: six models live on InternBench and three real sets

Every candidate ran live, in parallel, on GitHub-hosted runners (`candidate-bench.yml`, run
41718479983; 4 vCPUs each, AMD EPYC and Intel Xeon of several generations, so timings across jobs
are not comparable): llama.cpp b11946 with the app's own flags, 4 threads, an 8,192-token context,
thinking off, the review gate at 0.5. The engine was main at `c427aff`, before any rule was
written for a larger model. Strict names are whole filenames right; "wrong ready" counts names
Intern would have filed that are wrong.

| Model (Q4_K_M unless noted) | InternBench (77) | First real set | Second | Third | Wrong ready, real sets |
|---|---:|---:|---:|---:|---:|
| Qwen3.5 2B (shipping) | **71** | 12 | 16 | 8 | 3 / 2 / 5 |
| Qwen3.5 4B | 56 | 12 | 11 | 7 | 10 / 8 / 10 |
| Qwen3.5 9B | 55 | 12 | 10 | 7 | 11 / 13 / 9 |
| Gemma 4 E4B (Google QAT, Q4_0) | 48 | 11 | 13 | 6 | 6 / 8 / 6 |
| Nemotron 3.5 Bolt E4B | 52 | 12 | 15 | 6 | 17 / 19 / 14 |
| Granite 4.2 8B | 56 | 14 | 11 | 7 | 10 / 13 / 9 |
| Granite 4.1 8B | 38 | 11 | 9 | 4 | 8 / 8 / 3 |

Read with the lenient score (type, date and parties right, the joining word ignored), the picture
was different: the 9B led the real sets (28 / 28 / 17 against the 2B's 23 / 27 / 14), and the 4B
and Granite 4.2 were close behind. The larger models were reading the documents better and
being named worse: the rules threw their readings away. Section "Adapting the engine" is what
followed. Gemma 4 E4B, Nemotron Bolt and the Granites were set aside here: none led on any set
under either score, Nemotron filed the most wrong names of all, and the Granites and Gemma need
about 10 GB resident on the runners' measurement, with no accuracy to pay for it.

**Quantizations and decoding tricks, measured and rejected** (same workflow, runs 41730932267,
41730957300, 41730968866):
- Unsloth's UD-Q4_K_XL of the 9B and the 4B named no more InternBench documents right than
  Q4_K_M (53 against 55 for the 9B; 54 against 56 for the 4B), and Q4_K_S no more either. On
  x86 only Q4_0, Q4_K and IQ4_NL weights get llama.cpp's repacked GEMM kernels, so a Q5 or Q6
  file would be slower as well as larger. Q4_K_M ships.
- Multi-token prediction (`--spec-type draft-mtp`, the 9B's MTP GGUF) changed the replies (52 of
  77 right against 55) and was no faster on the same processor class (p50 56.8 s against 52.4 s);
  n-gram speculation likewise. Neither is used.
- The KV cache is not worth quantizing: a Qwen3.5 model keeps one for 8 of its 32 layers, 256 MiB
  at f16 for 8,192 tokens, for the 4B and the 9B alike.

## Adapting the engine

Both rounds were written by replaying every recording Intern has (InternBench, the three real
sets, three CUAD samples, the fixtures corpus) for all three models, and accepting a rule only
when no model lost a name on any set. Round one (#59): the reply's roles stand in where the
document supports none, held for review; a title's identifier is no part of the type. Round two
(#60): a stated type reads as its title's kind phrase; a machine stamp is no type; a form is for
the party it is filled in for, and the reply's roles stand in where the rules only defaulted.

| Strict names, wrong ready | 2B (gate 0.5) | 4B (gate 0.6, if filed) | 9B (gate 0.6 → shipped 0.7) |
|---|---:|---:|---:|
| InternBench (77), as found → after | 71, 1 → 71, 1 | 56, 15 → **64, 5** | 55, 17 → **70, 1** (0.7: 70, 1; 37 ready) |
| First real set (60) | 12, 3 → 16, 1 | 12, 10 → 18, 3 | 12, 11 → **22, 2** (0.7) |
| Second real set (60) | 16, 2 → 17, 1 | 11, 8 → 16, 4 | 10, 13 → **21, 0** (0.7) |
| Third real set (59) | 8, 5 → 11, 2 | 7, 10 → 12, 2 | 7, 9 → **11, 0** (0.7) |
| Third CUAD sample (42) | 14, 12 → 14, 12 | 17, 8 (screen) | 19, 12 → 19, 8 |

The 2B gained too, from rules written for the larger models, and lost nothing. The 4B's real-set
figures are on the documents its runner job could fetch (58, 56 and 55 of 60, 60 and 59; the
rest count as not named). The 9B's runner jobs missed 3, 4 and 5 documents the same way; those
were read live on a 4-core Xeon afterwards and merged into its recordings, so its figures are on
the whole sets. Two of them (two FTC letters the 2B named right) the 9B named right too; the rest
are the historic handwritten bills no model reads. The 9B column is at the gate it ships with,
0.7; at 0.6, the gate the rounds were measured at, it is 22, 3; 21, 3; 11, 1.

**The review gate** sends a name to review when the model's least probable date or party token
fell under a threshold. The 2B ships at 0.5. The sweep below, on the four real sets and
InternBench with the shipped engine, is why the 9B ships at 0.7 and why the 4B's names are held
whatever the gate says (names filed / wrong among them):

| Gate | 9B, real sets 1-4 | 9B, InternBench | 4B, real sets 1-4 | 4B, InternBench |
|---|---:|---:|---:|---:|
| 0.5 | 42 / 16 | 65 / 3 | | |
| 0.6 | 28 / 9 | 47 / 1 | 25 / 14 | 36 / 5 |
| 0.7 | **15 / 2** | **37 / 1** | 12 / 5 | 31 / 5 |
| 0.8 | 7 / 1 | 27 / 0 | 10 / 3 | 18 / 2 |
| 2B at 0.5 | 28 / 5 | 65 / 1 | | |

Each tenth of the 9B's gate above 0.6 halves what it files; 0.7 is where the wrong filings fall
fastest (9 to 2) for the names given up. The 4B files a wrong name for about every right one at
every gate, so `tiers::hold_fast_reading` holds its readings for a person, as alpha.12 held
Bonsai's, and the 9B reads them again when memory allows. The two the 9B still files wrong at 0.7
are both on the first set: a notice named for one side of its case, and an invoice named from its
customer.

## The fourth real set, unseen by every rule

The fourth set (`scripts/realdocs-eval/sample-heldout3.json`, 58 modern business documents: 25
letters, 17 notices, 14 invoices, 2 statements; 35 native PDFs, 10 scanned, 11 office files, 2
text files) was gathered after both rounds of rules were written, from issuers the earlier sets
mostly do not have, and its gold was written before any model read it. All three models ran on it
live on runners (run 41749074250; every document completed for every model) and were replayed
with the shipped engine at each model's gate. The scorer's strict verdict, unadjudicated:

| | Qwen3.5 2B (0.5) | Qwen3.5 4B (0.6, if filed) | Qwen3.5 9B (0.6) |
|---|---:|---:|---:|
| Names right, whole filename | 13/58 | 13/58 | **15/58** |
| Type, date and parties right (joining word ignored) | 18 | 20 | **23** |
| Parties right | 30 | 38 | **41** |
| Type right | **44** | 39 | 38 |
| Date right | 45 | 47 | 47 |
| Marked ready | 6 | 10 | 9 |
| **Wrong names ready** | **1** | 5 | **2** |
| Marked ready / wrong at 0.7, the 9B's shipped gate | 1 / 0 | 4 / 1 | 3 / 0 |

**What it says.** The 9B's lead over the 2B on unseen documents is real but small on whole names:
two more right, five more with type, date and parties right, eleven more with the parties right.
At 0.6 it files one more wrong name than the 2B: a notice of appeal named "from" the city that is
one side of the case (the gold wants both sides), and a notice of violation whose type is a
fragment of its title ("White Finding, Notice of Violation"). Both are the 9B being sure of a
reading the rules let through; at 0.7, the gate it ships with, neither files, and three right names
are filed. The 4B would have filed five wrong names here at 0.6 and one at 0.7, which is part of why
its readings are held for a person.

**Where the 9B loses on this set is the type**, not the parties: 44 types right for the 2B
against 38. Eleven FOIA and Official Information Act responses are "Document" for the 9B where the
gold wants "Freedom of Information Act Response" or a plain "Letter"; the letter-layout rule that
names the 2B's reading a "Letter" did not fire for the 9B because it cited a person as the
addressee ("Document - Mr. Carl Malamud"), and three Gazette notices are "Official Public Record",
the page's header, where the 2B read the notice's own kind. Every one of those goes to review, so
they cost names, not safety, and they are the next lever: the type a bigger model copies from a
page is a header or a person's title more often than the 2B's was. The 9B gains where the 2B named
no one or the wrong person: two WFAA invoices "from WFAA" (the 2B: "Invoice"), a letter to
Public.Resource.Org (the 2B: to Carl Malamud), two Oregon and NRC notices to the company they
penalise (the 2B: to a person, or to the agency).

Replay figures for the 4B are on this set complete; its earlier real-set figures are on the
documents its runner jobs fetched.

## The fixtures corpus and CI

The 21 generated fixtures (`fixtures/`) are the one place the 2B still leads: 13 of 18 names
right against the 9B's 10, through the evidence pipeline. The 9B's three losses all go to review:
a packing slip's and a delivery receipt's OCR'd code kept in the type ("Packing Slip PS-311"), and
a committee named "for" meeting minutes that name no party. The same three names are wrong and
ready for both models (a long document's title, a termination notice, a board deck's client). The
pattern of a bare code after a kind appears once on the real sets, so no rule was written for it.
Through the digest pipeline, which only a hosted model uses, the 9B names 11 of 17 and one reply
ran past the reply cap (`MODEL_REPLY_TRUNCATED` on the settlement agreement), recorded as the
failure it was.

The QA workflow's release gate (`scripts/validate-model-evaluation.mjs`) accepts the 9B's live
evidence run: 19 evaluated, 12 named, every named document's date right. The recordings of
record are now the 9B's: `bench/recording.json` (the runner's InternBench recording),
`fixtures/corpus-recording.json` and `fixtures/corpus-recording-digest.json` (recorded on a 4-core
Xeon on 2026-11-04), with their baselines written at the 0.7 gate, which CI's replays now pass.

## Speed and memory

The same 26 recorded contract requests (every third InternBench document) against each model on
one otherwise idle 4-core Xeon, llama.cpp b11946, 4 threads, the app's flags:

| | Qwen3.5 2B | Qwen3.5 4B | Qwen3.5 9B |
|---|---:|---:|---:|
| Latency per request, p50 | 14.9 s | 37.3 s | 68.2 s |
| Prompt processing, p50 (860 tokens) | 8.7 s | 23.7 s | 42.7 s |
| Decode, p50 | 12.9 tok/s | 5.9 tok/s | 3.6 tok/s |
| llama-server resident (RSS) after the run | 2.1 GB | 4.7 GB | 8.9 GB |
| of which anonymous (not the mapped file) | 0.9 GB | 2.0 GB | 3.4 GB |

On InternBench's 77 documents on the runner the 9B took p50 80 s, p95 148 s, at most 163 s a
document (wall 1 h 52 min); the fixtures corpus through the evidence pipeline took it a median
30 s a document on the Xeon. The app's thresholds follow the RSS: the 9B loads with 10 GiB free
and is estimated at 8.3 GiB resident, the 4B with 6.5 GiB free at 4.7 GiB; the download cap
rises to 10 GiB for the two files together.

## What this does not settle

- **Wrong names still file.** The 9B's wrong names ready on the real sets are one side of a
  court case, a title copied whole, an invoice's customer as its issuer, an OCR-cut name, an
  email typed by its subject line. Each is a class of one or two documents; none is a rule yet.
- **The date a lease is dated by.** The 9B's one wrong name ready on InternBench cites a lease's
  signature date where the gold wants the commencement date in its particulars; a rule preferring
  a labelled commencement date needs measuring on CUAD, where the 2B's recordings would change.
- **Windows.** Every figure here is from Linux x86-64. The QA workflow scores the fixtures corpus
  on a Windows runner with the shipped runtime before a release.
- **Speed on a laptop.** A 4-core Xeon at 2.1 GHz is slower than most laptops; a 2,000-token page
  on four AVX2 laptop cores is still a minute or two of prompt processing for the 9B.

## Reproduce

```bash
# The screen and the real-set runs: Actions > Candidate model benchmark on Linux > Run workflow,
# with the candidates JSON from docs/internbench.md and sets internbench,real1,real2,real3,real4.
# Replays of every recording with the shipped engine, at the app's gates:
cargo run --release --locked -p intern-bench -- run --corpus bench/generated --gold bench/gold.json \
  --manifest bench/manifest.json --replay bench/recording.json --min-token-confidence 0.7 \
  --output report.json --markdown report.md
cargo run --release --locked -p intern-engine --bin intern-evaluate -- --fixtures fixtures/generated \
  --expected fixtures/expected.json --replay fixtures/corpus-recording.json --min-token-confidence 0.7
```

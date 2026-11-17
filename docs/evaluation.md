# Measuring accuracy: the corpus, the recording, and the baseline

Every claim this project makes about accuracy is a number from
`intern-evaluate` over the clean-room corpus in `fixtures/`. This page is how
those numbers are produced, and how CI keeps them from quietly getting worse.

## Three ways to run the evaluator

```text
intern-evaluate --fixtures fixtures/generated --expected fixtures/expected.json ...
```

| Mode | Needs | Takes | Use it for |
| --- | --- | --- | --- |
| **Live** (`--worker`, `--endpoint`, `--api-key`) | the parser worker, its runtime, and a running `llama-server` | most of an hour on a laptop CPU | the truth: what the model actually says |
| **Record** (live, plus `--record PATH`) | the same | the same | keeping what a live run saw, for replay |
| **Replay** (`--replay PATH`) | nothing but the repository | seconds | scoring an engine change without a model |

A recording (`fixtures/corpus-recording.json`) holds, per fixture, the text
the worker extracted and the reply the model gave, keyed by the SHA-256 of
the exact prompt that reply answers. Replay indexes the recorded text again
and retrieves its evidence (or, for a digest recording, distills it),
rebuilds the prompt to check it is still the one the reply answers, and then
re-runs everything *after* the model - validation, evidence checks, date-role
and type inference, composition, house style, naming - from that text and
that reply, so a change to those stages is scored the way a live run would
score it.

The evaluator runs the evidence pipeline, the default, unless asked for
another (`--pipeline digest`, or `legacy` live). A recording says which
pipeline made it and is replayed only through that one:
`fixtures/corpus-recording.json` and `corpus-baseline.json` are the evidence
pipeline's, recorded live with Qwen3.5 9B after the second engine adaptation
(`e64581e`, merged as `a7d3021`), and `fixtures/corpus-recording-digest.json`
and `corpus-baseline-digest.json` the digest pipeline's, which a hosted model
reads through, recorded the same day. Both are replayed with the accurate
model's review gate, `--min-token-confidence 0.8`
(`REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE` in `tiers.rs`), as the app reads,
and their baselines were written at that gate. CI replays both:

```text
intern-evaluate ... --replay fixtures/corpus-recording.json --baseline fixtures/corpus-baseline.json \
  --min-token-confidence 0.8
intern-evaluate ... --pipeline digest --replay fixtures/corpus-recording-digest.json \
  --baseline fixtures/corpus-baseline-digest.json --min-token-confidence 0.8
```

## What replay can and cannot tell you

Replay is honest about its own limits. Before scoring a fixture it rebuilds
the prompt from the recorded text and compares the hash with the one recorded:

* **The hashes match.** The model was asked exactly this. The reply is real
  and the score is the score a live run would give.
* **They differ** because the prompt wording or the distillation changed.
  The recorded reply answers a question the engine no longer asks, and
  scoring it would measure nothing. The fixture is reported as
  `stale_prompt`, the run exits 2, and the fix is to re-record. Pass
  `--allow-stale` to score anyway while iterating locally; the records are
  marked `"stale": true` and CI never uses this flag.
* **The fixture bytes changed** (the generator was edited without
  re-recording): `stale_fixture`, same treatment.

So: a change to `facts.rs`, `compose.rs`, `validate.rs`, `evidence.rs`,
`infer.rs`, `house_style.rs`, or `naming.rs` is measured for free. A change
to `prompt.rs` costs one live recording on a machine with the runtime - and
so, almost always, does a change to `index.rs` or `retrieve.rs`, which
choose the evidence lines the prompt carries, or, for the digest pipeline,
to `distill.rs`. The hash is taken over the whole prompt, and the
digest distillation builds is most of the prompt: a heuristic that keeps a
different block, orders the date index differently, or trims one more
character changes the hash, and every fixture it touches replays as
`stale_prompt` until the corpus is re-recorded. Only a distillation change
that leaves every corpus prompt byte-for-byte the same replays clean, and
then it has changed nothing replay can see. That cost is the point. A prompt
change nobody has run the model over is a prompt change nobody has measured.

### A fixture waiting for a recording

A fixture added to the corpus, or regenerated, before anyone could make a
live recording of it would otherwise fail every replay as `unrecorded` or
`stale_fixture`. Mark it in the gold definition in
`fixtures/generate-fixtures.mjs` with `recording: 'pending'` (and update
`expected.json` with `--update-gold`, as `fixtures/README.md` describes).
Replay then reports it with status `pending`: it is not scored, it is not a
regression, it does not make the run exit 2, and `summary.pending` counts it
so it is not forgotten. A baseline written meanwhile leaves it out, so once
it is recorded - and the mark removed - it arrives as a new fixture rather
than as a regression from "pending". A live run ignores the mark.

## The baseline gate

`fixtures/corpus-baseline.json` is the per-fixture score sheet the committed
recording earns. CI replays the corpus on every push and compares:

* any score that was right and is now wrong is a **regression**, and the job
  fails (exit 2);
* a trap score (`date_forbidden`, `party_forbidden`) that was clear and now
  fires is a regression too;
* a fixture whose status changed - it scored before and is now stale or
  unrecorded - is a regression;
* a score that was wrong and is now right is an **improvement**, reported in
  the log and never required;
* a `pending` fixture is listed and compared with nothing.

Only the score keys the baseline already holds are compared. A score added
to the evaluator is reported from its first run but gates nothing until a
baseline is written with it: replay first against the old baseline, which
must show no regressions, then write the baseline, and check in its diff
that the only changes are the new keys and the improvements you expected.

`ready` on its own is not compared: readiness is a routing decision, and
`readiness_match` already scores whether it was the right one.

To accept a new state of the world - a better engine, or a deliberately
different trade - run replay with `--write-baseline fixtures/corpus-baseline.json`
and commit the result with the change that earned it. Reviewers can see, in
the diff of that file, exactly which fixtures changed and in which direction.

## Recording on Windows

The shipped runtime is Windows-only (PDFium, Tesseract, and llama.cpp are
pinned there by `src-tauri/resources/runtime-assets.json`), so the recording of
record is made on Windows:

```powershell
./scripts/fetch-windows-assets.ps1 -CacheDirectory $env:TEMP\intern-assets
cargo build --locked -p intern-worker --release --features windows-native
Copy-Item target\release\intern-worker.exe src-tauri\binaries\intern-worker-x86_64-pc-windows-msvc.exe
./scripts/stage-windows-runtime.ps1 -Destination C:\intern-stage
npm run fixtures
./scripts/record-corpus.ps1 -RuntimeDirectory C:\intern-stage -ModelPath "$env:LOCALAPPDATA\Intern\models\Ternary-Bonsai-2-4B-PQ2_0.gguf"
```

The committed recordings were made with the model and request of the build
that recorded them, without a prompt profile, and CI replays them that way.
The Bonsai 2 models an earlier build pinned were asked through the `bonsai_2`
profile; a recording of that request is made with `-PromptProfile bonsai_2`
(`--prompt-profile bonsai_2` to `intern-evaluate`, `intern-analyze` and
InternBench), and replays only with the same option, since the profile is part
of every prompt's identity. Neither pinned model has a profile.

The script refuses a model whose SHA-256 is not the one `model-manifest.json`
pins, starts `llama-server` with the flags the app itself uses (one slot, CPU,
8,192-token context, the model's own chat template, the prompt cache off,
`-ub 256`), records with the token probabilities the review gate reads
(`--token-confidence`), writes the baseline at the accurate model's gate
(`--min-token-confidence 0.8`, as CI replays it), and stops the server. It records the evidence pipeline, the
default; the digest pipeline's recording is made the same way by
`intern-evaluate --pipeline digest --record fixtures/corpus-recording-digest.json
--write-baseline fixtures/corpus-baseline-digest.json`. Commit
`fixtures/corpus-recording.json`, `fixtures/corpus-baseline.json`, and the
report's summary in `docs/model-bakeoff.md`.

## Recording elsewhere

The engine and the worker are portable; only the packaged runtime is not. The
committed recordings were made on Linux with the pinned accurate model and an
upstream llama.cpp CPU build. The 77-document evaluation
(`bench/reports/bonsai-linux-cpu-2026-10-27`) found the llama.cpp fork the
package then carried read the model it measured the same way as a build from
source, within run-to-run noise, and the same run on upstream `b11946` - the
one build the package carries now, which both pinned models run on - read it
no less accurately
(`bench/reports/speed-linux-cpu-2026-10-29`); the digest pipeline's with PDFium
`chromium/7881` for Linux, the same pinned `tessdata_fast` files, and the
distribution's Tesseract 5.3.4 rather than the vcpkg 5.5.2 the installer
ships, the evidence pipeline's with the routing reader and PP-OCR; the
`note` field of each recording says so. OCR output can differ by a character between Tesseract
builds, which is why the OCR fixtures are marked `needs_review` in the gold
corpus and scored on routing rather than on the digits they misread. A
recording made on the packaged Windows runtime supersedes it; the workflow is
the same, with `INTERN_RUNTIME_DIR` pointing at a directory holding
`libpdfium.so` and `tessdata/`; the worker uses the Tesseract on `PATH` unless
the directory holds a `tesseract` of its own. `scripts/fetch-linux-assets.sh`
and `scripts/stage-linux-runtime.sh` build exactly such a directory, the Linux
package's runtime ([Linux](linux.md)).

## Re-recording only some fixtures

A change to how the worker reads one kind of document - how scans are OCR'd,
say - changes the prompts of only the fixtures of that kind, and only those
need recording again. The rest of the committed recording cannot simply be
recorded again with them: its text fixtures were edited by hand after they
were recorded (see the `note`), and a live run does not reproduce those
replies byte for byte. So record the corpus live into a scratch file and
splice in just the fixtures the change touched:

```text
intern-evaluate ... --record /tmp/live-recording.json --output /tmp/after.json
node scripts/splice-recording.mjs fixtures/corpus-recording.json /tmp/live-recording.json \
  --note "what was re-recorded, when, and on what" scanned-lease.pdf document-image.png ...
intern-evaluate ... --replay fixtures/corpus-recording.json --write-baseline fixtures/corpus-baseline.json
```

The script refuses recordings made for a different model or budget, a
fixture recorded from different bytes (the fixture changed, so everything
needs re-recording), and a live run that *read* any fixture it was not asked
to splice differently - that is a finding about the change, not something to
commit. A fixture whose reply alone differs is reported and left as it was.
It splices the file as text, so every entry it did not touch stays
byte-for-byte as `intern-evaluate` wrote it. The OCR fixtures were last
re-recorded this way, after OCR began keeping Tesseract's line structure.

## After a retrieval or prompt change

A change to what retrieval sends, or to the prompt, changes the request of
every document it touches, and those documents replay as `stale_prompt` in
both recordings of record until they are recorded again. Neither can be
patched: `intern-bench merge-recordings` refuses a recording made with
another retrieval configuration or prompt, and a splice would leave the
fixture recording's header naming the configuration it no longer holds. So
both are recorded again whole, live, with the pinned accurate model. The
`--allow-stale` flag is for looking while iterating, never for CI.

The widening of the evidence in November 2026
([`evidence-retrieval.md`](evidence-retrieval.md#the-chosen-configuration))
is such a change. Until the recordings are made again, CI's
`corpus-replay` job reads:

| Step | Recording | Stale | Result |
| --- | --- | --- | --- |
| Replay the corpus and compare with the baseline | `fixtures/corpus-recording.json` | `statement-of-work.pdf`, `settlement-agreement.pdf`: the two fixtures long enough to be excerpted | 2 regressions, exit 2 |
| Replay the corpus through the digest pipeline and compare with its baseline | `fixtures/corpus-recording-digest.json` | none: the digest pipeline neither retrieves nor shows a file name | passes |
| Replay InternBench and compare with its baseline | `bench/recording.json` | 19 of 77, every document Stage 4 excerpted: `account-statement`, `annual-report-100p`, `annual-report-excerpt-8p`, `asset-purchase-agreement-25p`, `credit-agreement-50p`, `data-processing-agreement-10p`, `industrial-lease-dated-in-schedule-25p`, `inspection-log-ruled-2p`, `lease-two-column`, `msa-effective-date-in-definitions-12p`, `payroll-register`, `promissory-note`, `property-policy-declarations-mid-60p`, `scan-agreement-10p`, `scan-lease-25p`, `services-agreement`, `sow-under-msa-5p`, `term-loan-parties-apart-40p`, `watershed-monitoring-report-100p` | 19 regressions, exit 2 |

The other 58 InternBench documents and 19 fixtures replay unchanged: they
went whole and still do. The file name the prompt shows stales nothing:
neither evaluator passes one - the corpora's file names are their ids, and
name the answer - and a request without a name is byte for byte the
request from before names were shown.

**InternBench, on GitHub's runners.** The candidate benchmark workflow
records InternBench live with the app's flags (see
[`internbench.md`](internbench.md#candidates-on-github-runners)):

1. Run it on the branch with the change (Actions > Candidate model
   benchmark on Linux > Run workflow, or
   `gh workflow run candidate-bench.yml --ref BRANCH -f sets=internbench -f candidates='[...]'`),
   with the pinned accurate model as the candidate - its entry in
   `src-tauri/resources/model-manifest.json`:

   ```json
   [{"label": "qwen3.5-9b",
     "url": "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q4_K_M.gguf",
     "sha256": "03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8"}]
   ```

   and the defaults otherwise: four threads, no extra server or bench
   flags. The job takes several hours; it records with token
   probabilities, since it scores at a gate.
2. Download the artifact `candidate-qwen3.5-9b-internbench-RUN_ID`, copy its
   `recording.json` over `bench/recording.json`, and write in its `note`
   what made it (model, runtime, runner CPU, workflow run, commit), as the
   note it replaces does.
3. Write the baseline at the app's gate from it, and compare it with a
   replay of the old recording made at the commit before the change:

   ```sh
   cargo run --release --locked -p intern-bench -- run --corpus bench/generated \
     --gold bench/gold.json --manifest bench/manifest.json \
     --replay bench/recording.json --min-token-confidence 0.8 \
     --write-baseline bench/baseline.json --output after.json --markdown after.md
   cargo run --release --locked -p intern-bench -- compare \
     --before before.json --after after.json --markdown diff.md
   ```

4. Commit `bench/recording.json` and `bench/baseline.json` with the change.

On a Linux machine instead, `scripts/run-internbench.sh` makes the same
recording (`MODEL`, `LLAMA_SERVER` and `INTERN_RUNTIME_DIR` set, and
`-- --min-token-confidence 0.8` so it records token probabilities) into
`target/internbench/recording.json`.

**The fixture corpus, on a Linux machine.** No workflow records it; the
committed recording was made on a four-core Linux machine with the staged
Linux runtime (see [Recording elsewhere](#recording-elsewhere)):

```sh
scripts/fetch-linux-assets.sh /tmp/intern-assets
scripts/stage-linux-runtime.sh /tmp/intern-stage
cargo build --release --locked -p intern-worker --features windows-native
cargo build --release --locked -p intern-engine --bin intern-evaluate
npm run fixtures
# llama-server with the app's flags, as scripts/run-internbench.sh starts it
KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
/tmp/intern-stage/runtime/llama-upstream/llama-server --host 127.0.0.1 --port 18090 \
  --api-key "$KEY" --model Qwen3.5-9B-Q4_K_M.gguf --parallel 1 --ctx-size 8192 \
  --n-gpu-layers 0 --threads 4 --threads-batch 4 --jinja --no-webui --no-mmproj \
  --cache-ram 0 -ub 256 --prio -1 --poll 0 --no-repack &
INTERN_RUNTIME_DIR=/tmp/intern-stage/runtime target/release/intern-evaluate \
  --fixtures fixtures/generated --expected fixtures/expected.json \
  --worker target/release/intern-worker \
  --endpoint http://127.0.0.1:18090/v1/chat/completions --api-key "$KEY" \
  --model-id intern-local --token-confidence --min-token-confidence 0.8 \
  --note "Qwen3.5-9B-Q4_K_M.gguf (manifest sha256 03b74727), upstream llama.cpp b11946 ... (what, where, when, from which commit)" \
  --record fixtures/corpus-recording.json --output live.json
target/release/intern-evaluate --fixtures fixtures/generated --expected fixtures/expected.json \
  --replay fixtures/corpus-recording.json --baseline fixtures/corpus-baseline.json \
  --min-token-confidence 0.8 --output before-baseline.json
target/release/intern-evaluate --fixtures fixtures/generated --expected fixtures/expected.json \
  --replay fixtures/corpus-recording.json --write-baseline fixtures/corpus-baseline.json \
  --min-token-confidence 0.8 --output corpus-report.json
```

The replay against the old baseline lists every fixture that moved, in
either direction, for the commit message; then the baseline is written.
Commit `fixtures/corpus-recording.json` and `fixtures/corpus-baseline.json`
with the change. The digest pipeline's recording needs nothing.

## Reading a report

`--output report.json` writes the full report; without it, the report goes to
standard output. `summary` carries every boolean score as
`{correct, total, rate}`, the count of records by status, the review rate,
and inference and total-time percentiles (zero in replay, where nothing is
inferred). `records` carries one entry per fixture with the composed
filename, the description, the validated proposal, the review reasons, and
the scores. In replay every record says `"replayed": true`.

The scores are the ones `fixtures/README.md` describes: `date_correct` counts
the reviewed date or a listed acceptable one; `date_forbidden` counts a date
the corpus marks as a trap; `type_correct` accepts any answer carrying every
meaningful word of the reviewed type; `parties_correct` needs every reviewed
party and no spurious one; `description_covers_facts` needs every listed fact
in the sentence; `readiness_match` compares the routing decision with the
reviewed one.

Two scores judge the name a person actually sees:

* `relation_correct` - the connecting word (`between`, `for`, `from`, ...)
  is the reviewed `party_relation`. Scored where the corpus states one and
  the run produced parties, since a relation attached to nobody says
  nothing.
* `filename_correct` - the proposed filename is one the reviewed answer
  composes to: the reviewed type (or an acceptable one), the reviewed date
  (or an acceptable one), the reviewed parties and relation, run through the
  engine's own naming with the fixture's extension, and compared the way
  Windows compares names (case, trailing dots and spaces, and Unicode
  composition disregarded). Scored where the corpus gives a date and a type.
  Every other score can be right while this one is wrong - "Contoso
  Worldwide Inc" for "Contoso Worldwide, Inc.", a lost date, the wrong
  connecting word - which is why it exists.

Each miss is also printed on standard error as `file: filename: expected X,
got Y`, with X the reviewed answer's own name, so the log says what a person
would have seen without opening the report.

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
and retrieves its evidence (or, for a digest recording, distils it),
rebuilds the prompt to check it is still the one the reply answers, and then
re-runs everything *after* the model - validation, evidence checks, date-role
and type inference, composition, house style, naming - from that text and
that reply, so a change to those stages is scored the way a live run would
score it.

The evaluator runs the evidence pipeline, the default, unless asked for
another (`--pipeline digest`, or `legacy` live). A recording says which
pipeline made it and is replayed only through that one:
`fixtures/corpus-recording.json` and `corpus-baseline.json` are the evidence
pipeline's, recorded live from `902c7fe`, and
`fixtures/corpus-recording-digest.json` and `corpus-baseline-digest.json`
the digest pipeline's, which a hosted model still reads through. CI replays
both:

```text
intern-evaluate ... --replay fixtures/corpus-recording.json --baseline fixtures/corpus-baseline.json
intern-evaluate ... --pipeline digest --replay fixtures/corpus-recording-digest.json \
  --baseline fixtures/corpus-baseline-digest.json
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
./scripts/record-corpus.ps1 -RuntimeDirectory C:\intern-stage -ModelPath "$env:LOCALAPPDATA\Intern\models\Qwen3.5-2B-Q4_K_M.gguf"
```

The script refuses a model whose SHA-256 is not the one `model-manifest.json`
pins, starts `llama-server` with the flags the app itself uses (one slot, CPU,
8,192-token context, the model's own chat template), records, writes the
baseline, and stops the server. It records the evidence pipeline, the
default; the digest pipeline's recording is made the same way by
`intern-evaluate --pipeline digest --record fixtures/corpus-recording-digest.json
--write-baseline fixtures/corpus-baseline-digest.json`. Commit
`fixtures/corpus-recording.json`, `fixtures/corpus-baseline.json`, and the
report's summary in `docs/model-bakeoff.md`.

## Recording elsewhere

The engine and the worker are portable; only the packaged runtime is not. The
committed recordings were made on Linux with the same pinned model and a
llama.cpp CPU build from source; the digest pipeline's with PDFium
`chromium/7881` for Linux, the same pinned `tessdata_fast` files, and the
distribution's Tesseract 5.3.4 rather than the vcpkg 5.5.2 the installer
ships, the evidence pipeline's with the routing reader and PP-OCR; the
`note` field of each recording says so. OCR output can differ by a character between Tesseract
builds, which is why the OCR fixtures are marked `needs_review` in the gold
corpus and scored on routing rather than on the digits they misread. A
recording made on the packaged Windows runtime supersedes it; the workflow is
the same, with `INTERN_RUNTIME_DIR` pointing at a directory holding
`libpdfium.so`, a `tesseract.exe` symlink to the Tesseract binary, and
`tessdata/`.

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

# Intern alpha.14 rendered-fidelity ledger

**Rendered sign-off status: accepted for alpha.14, by Claude Code under the
maintainer's standing instruction; no person has looked at this capture.**
`docs/qa/rendered-fidelity-signoff.json` is bound to release-input digest
`c233331e1444f3edb9e3d65e7eef8f5e726cbd19926dc7830886fa134c9f0d25` and to the
1536x1024 capture `docs/qa/latest-implementation.png`, SHA-256
`e5f68d84302602dfb63feecf3518223dfa51fa3559409ccb19a000d0cd34f3b0`, taken by
Whole-product QA evidence run 42129423153 (attempt 1) at commit
`1710056f98df097b42e96f8cf4b03b57d3d9d3bf`, `main` after the alpha.14
release preparation.

## alpha.14 record

Run 42129423153 passed every gate on Windows:

- frontend checks (77 Vitest files, 753 tests passed and 1 skipped), and the
  browser QA suite (30 tests) including its contrast assertions;
- `cargo fmt --check`, workspace clippy with warnings denied, and the Rust
  workspace tests (1,759);
- native fixture parsing with the pinned assets (4 tests on the 21 gold
  fixtures), and the verified pinned runtime (76 runtime files, 27 license
  files) carrying both llama.cpp builds: the PrismML release
  `prism-b10826-c9c34bc` and upstream build 11946 for the Qwen3.5 models;
- an NSIS build, and installer and uninstall smoke with user data retained,
  with both installed llama-server builds started;
- an accepted whole-corpus evaluation with real inference: 19 documents on the
  pinned Qwen3.5 9B model (the accurate tier, `Qwen3.5-9B-Q4_K_M.gguf`), on the
  two threads the hosted evaluation sets for itself on the four-core runner,
  median 15.8 s and at most 30.5 s of inference a document, 329.6 s in all,
  peak model memory 8,124 MiB.

The corpus scores are the same as alpha.13's on every count: date role 16/16,
descriptions covering the expected facts 16/18, type 17/18, parties 15/19,
description specific 16/19, dates right for 16 of 18, the expected filename
matched exactly for 9 of 18, a review rate of 42.1% at the 0.7
token-confidence gate, and nothing filed under a date or a party the corpus
marks as a trap. This synthetic corpus does not exercise what this release
changes (letters, notices and captions are measured on the real-document
sets cited in the release notes and in
`docs/accuracy-research-2026-10.md`), so an unchanged score here is the
expected result. The run was faster than alpha.13's (median 21.7 s, 490.8 s
in all); this record does not credit that to the release, since the hosted
evaluation sets its own thread count and the runners differ.

The capture was inspected against the accepted concept:

- The counts strip and toolbar, the queue table and the review panel read as
  three distinct planes.
- The header states the privacy posture (Private, On this device).
- Date-first proposed filenames and em-dashes for absent values stay consistent
  down the column; the model's confidence is a right-aligned percentage beside
  its track in the panel.
- Ready, Needs review, Proposing a name and Waiting are each distinguishable by
  icon as well as color.
- The panel opens with "What to check" and the reason for review; evidence
  stays attributed under Date and Type headings, each value on its own line.
- The approve and keep actions sit in a pinned bar at the foot of the panel,
  with their keyboard shortcuts.
- No collisions, clipping, illegible copy, excessive density or ambiguous focus
  were observed, and the run's automated contrast assertions passed.

What changed in the frame: nothing of substance. This release changes the
engine's reading rules, the thread count and the memory floor, none of which
this screen shows. Compared pixel by pixel with the accepted alpha.13 capture,
the frame differs in 15 pixels on the outline of the Queue and Completed tabs,
which is anti-aliasing, and is otherwise identical. The frame was still taken
again and reviewed rather than carried over, because the sign-off binds the
capture to the release inputs and those changed.

The alpha.13 observation stands, and is not a defect of this frame: at this
size Parties sits below the fold under the pinned action bar and is reached by
scrolling the panel.

### Reviewer

Reviewed by Claude Code, which opened the capture named above from the run's
artifact at full size, checked it against the concept, compared it with the
alpha.13 capture, and accepted it. No person has looked at this capture. The
maintainer, Zachary Brenner, instructed in advance that this release be cut
without further questions and that the work be reviewed and approved by Claude
Code; this record is written under that standing instruction and says so,
rather than naming a person who did not look. The maintainer can look at the
same capture and re-sign this record without a new QA run, since neither the
digest nor the capture hash changes.

### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs. Any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.13 record

The alpha.13 sign-off was accepted by Claude Code, under the maintainer's
standing instruction, for the capture from run 41777820194 at commit
`d900c11604efefaa56616230f0b0818c26df3911`, bound to release-input digest
`40df3381aac1a22c7cbfcd50944dc4f5c4dd629ba710e03eaef9f6d0a3864c32`; the
capture SHA-256 was
`a95052639f75b6e2bdad0438da20645b59490271748ca2a9dab901fdb31cbd25`. Pull
requests #65 through #68 have changed the non-QA release inputs since, so that
digest no longer describes what would ship. Its record is kept below for
history.

### alpha.13 run

Run 41777820194 passed every gate on Windows:

- frontend checks (77 Vitest files, 753 tests passed and 1 skipped), and the
  browser QA suite (30 tests) including its contrast assertions;
- `cargo fmt --check`, workspace clippy with warnings denied, and the Rust
  workspace tests (1,753);
- native fixture parsing with the pinned assets, and the verified pinned runtime
  (76 runtime files, 27 license files), now carrying both llama.cpp builds: the
  PrismML release `prism-b10826-c9c34bc` and upstream build 11946 for the
  Qwen3.5 models;
- an NSIS build, and installer and uninstall smoke with user data retained,
  with both installed llama-server builds started;
- an accepted whole-corpus evaluation with real inference: 19 documents on the
  pinned Qwen3.5 9B model (the accurate tier, `Qwen3.5-9B-Q4_K_M.gguf`), on two
  threads of the four-core runner, median 21.7 s and at most 51.1 s of
  inference a document, 490.8 s in all, peak model memory 8,132 MiB.

Against alpha.12's hosted run on the same 19 documents, which scored the
Qwen3.5 2B: date role 16/16 (15/16), descriptions covering the expected facts
16/18 (15/18), type 17/18 (17/18), and nothing filed under a date or a party
the corpus marks as a trap, in both. Dates were right for 16 of 18. Two counts
are lower: parties 15/19 (18/19) and description specific 16/19 (17/19), and
the review rate is 42.1% (26.3%): eight of the 19 readings were held for the
user at the 0.7 token-confidence gate this release adopts, and the expected
filename was matched exactly for 9 of 18 (the local qualification run of the
same model on this corpus scored 10). Every gate is above its floor. This
synthetic corpus is the one set in the qualification battery where the 9B
scores below the 2B; the release notes and
`bench/reports/model-qualification-2026-11/` cite the real-document sets where
it was chosen, and the gate's own numbers are recorded here as they are.

The capture was inspected against the accepted concept:

- The counts strip and toolbar, the queue table and the review panel read as
  three distinct planes.
- The header states the privacy posture (Private, On this device).
- Date-first proposed filenames and em-dashes for absent values stay consistent
  down the column; the model's confidence is a right-aligned percentage beside
  its track in the panel.
- Ready, Needs review, Proposing a name and Waiting are each distinguishable by
  icon as well as color.
- Evidence stays attributed under Date and Type headings, each value on its own
  line.
- The approve and keep actions sit in a pinned bar at the foot of the panel,
  with their keyboard shortcuts.
- No collisions, clipping, illegible copy, excessive density or ambiguous focus
  were observed, and the run's automated contrast assertions passed.

What changed in the frame is this release's own:

- The sidebar is gone. Queue (8) and Completed (1) are tabs above the table,
  beside the "1 needs review" and "3 ready" counts; help and settings moved to
  the header, and Pause is labeled.
- The table's Confidence column is a Check column, which says "All checks
  passed" or gives the reason for review; the confidence itself is in the
  panel. "Rename all ready" replaces "Apply all ready".
- A progress line ("Reading Q1 Financials.pdf, 0 of 4 done") stands where the
  alpha.12 frame showed the drop zone.
- The panel opens with "What to check", carrying the reason for review above
  the fold, which the alpha.11 and alpha.12 records left as a follow-up.
- Evidence is introduced as the model's reading in its own words rather than
  lines matched in the document, and its values are no longer quoted.

One observation, not a defect of this frame: at this size Parties now sits
below the fold under the pinned action bar and is reached by scrolling the
panel, where alpha.12 showed it; the reason for review took that space. The
"Read by" line in the Details panel is not in this frame, which shows the
review panel; it is held by unit tests.

#### Reviewer

Reviewed by Claude Code, which opened the capture named above from the run's
artifact at full size, checked it against the concept, and accepted it. No
person has looked at this capture. The maintainer, Zachary Brenner, instructed
in advance that this release be cut without further questions and that the
work be reviewed and approved by Claude Code; this record is written under
that standing instruction and says so, rather than naming a person who did not
look. The maintainer can look at the same capture and re-sign this record
without a new QA run, since neither the digest nor the capture hash changes.

#### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs. Any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.12 record

The alpha.12 sign-off was accepted by the maintainer for the capture from run
40566145844 at commit `cd058571ebe529d4bf434c1abcdebf6fddbbeb57`, bound to
release-input digest
`649494076b200fc7546e3aa97b605ed77c07626adb917715951d9732f2213126`, and then
carried, because run 40618344348's capture was byte-identical, to the SBOM-fix
commit `33082af6ebb2a4ad96b1cf299e502c164ab3834e` and its digest
`1dbe634135b2f85c92b532d7bfcdb5376945a0b5308c3ef338b6db5a09c31a84`, which is
the record alpha.12 shipped with. The capture SHA-256 was
`2ea19abf6f400ead3d9fa25a4452bed2d44d78f44642f846317d64bd9b9ea52d`. Pull
requests #37 through #63 have changed the non-QA release inputs since, so
neither digest describes what would ship. Its record is kept below for history.

### alpha.12 run

Run 40566145844 passed every gate on Windows:

- frontend checks (71 Vitest files, 716 tests passed and 1 skipped), and the
  browser QA suite (29 tests) including its contrast assertions;
- `cargo fmt --check`, workspace clippy with warnings denied, and the Rust
  workspace tests (1,684);
- native fixture parsing with the pinned assets, and the verified pinned runtime
  (54 runtime files, 26 license files), now with the PrismML llama.cpp release
  `prism-b10826-c9c34bc`;
- an NSIS build, and installer and uninstall smoke with user data retained;
- an accepted whole-corpus evaluation with real inference: 19 documents on the
  pinned Qwen3.5 2B model (the accurate tier), median 7.6 s and at most 24.2 s
  of inference a document, peak model memory 2,126 MiB.

Against alpha.11's hosted run on the same 19 documents: parties 18/19 (16/19),
review rate 26.3% (42.1%), and nothing filed under a date or a party the corpus
marks as a trap, in both. Two counts are one document lower: type 17/18
(18/18) and description specific 17/19 (19/19); descriptions covering the
expected facts are 15/18 (16/18). Date role is 15/16, on a denominator the
evidence pipeline counts differently from alpha.11's 14/14. The release notes'
accuracy figures come from InternBench and are cited there; these are the
release gate's own corpus, and every gate is above its floor.

The capture was inspected against the accepted concept:

- Sidebar counts, the queue table and the review panel read as three distinct
  planes.
- The header states the privacy posture.
- Date-first proposed filenames, right-aligned confidence and em-dashes for
  absent values stay consistent down the column.
- Ready, Needs review, Proposing a name and Waiting are each distinguishable by
  icon as well as color.
- Evidence stays attributed under DATE, TYPE and PARTIES headings, with each
  party on its own line.
- The approve and keep actions sit in a pinned bar at the foot of the panel,
  with their keyboard shortcuts.
- No collisions, illegible copy, excessive density or ambiguous focus were
  observed.

What changed in the frame is this release's own: one Queue view, with a
"1 needs review" count where the separate Needs Review page was; Copy table and
Details in the toolbar; "Copied into Input" in the header; and a drop zone that
copies what is dropped into the input folder.

As in alpha.11, the panel's Reason for review sits below the fold under the
pinned action bar and is reached by scrolling the panel; bringing it above the
fold for an item in review is still a follow-up.

### Reviewer

Reviewed by the maintainer, Zachary Brenner, who looked at the capture named
above and accepted it. Claude Code inspected it first and wrote this record and
the sign-off at the maintainer's direction.

### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs. Any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.11 record

The alpha.11 sign-off was bound to release-input digest
`7cc3645acd565a349bc58c8cfeab086792c52d282e03704beb0744e17ac1025d` at commit
`6aa24a522a78475ba7cff02b6b756159b0d67b9b`, with the capture SHA-256
`f4bf894357019c77d251b0f0ef9c379e71c2665d67b901c10c8976526bc0aee4`. Pull
requests #16 through #34 have changed the non-QA release inputs since, so that
digest no longer describes what would ship. Its record is kept below for
history.

### alpha.11 run

Run 37365868306 passed every gate on Windows:

- frontend checks, and the browser QA suite (23 tests) including its contrast
  assertions;
- `cargo fmt --check`, workspace clippy with warnings denied, and the Rust
  workspace tests (1,073);
- native fixture parsing with the pinned assets, and the verified pinned runtime
  (51 runtime files, 23 license files);
- an NSIS build, and installer and uninstall smoke with user data retained;
- an accepted whole-corpus evaluation with real inference: 19 documents on the
  pinned model, median 23 s and at most 136 s a document, peak model memory
  2,074 MB.

The evaluation's scores equal alpha.10's: date role 14/14, type 18/18,
description specific 19/19, descriptions covering the expected facts 16/18,
parties 16/19, review rate 42.1%, and nothing filed under a date or a party the
corpus marks as a trap. This release's naming changes (accented text near a
date no longer stopping the queue, a document's own date recognized, due,
implausible and ambiguous dates sent to review, title-cased letterhead names)
are held by unit tests and the corpus replay rather than moved by these
aggregate counts. The release notes' figure of parties rising from 15 to 16 of
19 came from a local Linux rig comparing the branches; on this hosted Windows
measurement parties are 16/19 in alpha.9 and in alpha.11, and the notes are
being corrected to say so.

The capture was inspected against the accepted concept:

- Sidebar counts, the queue table and the review panel read as three distinct
  planes.
- The header states the privacy posture.
- Date-first proposed filenames, right-aligned confidence and em-dashes for
  absent values stay consistent down the column.
- Ready, Needs review, Proposing a name and Waiting are each distinguishable by
  icon as well as color.
- Evidence stays attributed under DATE, TYPE and PARTIES headings, with each
  party on its own line.
- The approve and keep actions sit in a pinned bar at the foot of the panel,
  now with their keyboard shortcuts.
- No collisions, illegible copy, excessive density or ambiguous focus were
  observed.

What changed in the frame is this release's own: the drop zone lists the new
formats; a document being read says what is happening ("Proposing a name…")
rather than a percentage; the panel counts what is left to decide and offers
Next undecided, Open and Show in folder; and the filename field wraps, with the
extension shown apart.

Two observations, neither a defect of this frame:

- The previous capture showed a "for Vistage" brand tag beside the name. The
  header has not rendered one since before alpha.10 (`AppHeader` is unchanged
  from the alpha.10 release commit), so that frame was older than the code it
  was bound to. This one is a faithful render of what ships.
- At this size the panel's Reason for review sits below the fold, under the
  pinned action bar, and is reached by scrolling the panel. The bar is pinned
  so that Approve is always on screen; bringing the reason above the fold for
  an item in review is a follow-up.

### Reviewer

Reviewed by the maintainer, Zachary Brenner, who looked at the capture named
above and accepted it. Claude Code inspected it first and wrote this record and
the sign-off at the maintainer's direction.

### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs. Any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.10 record

The alpha.10 sign-off was bound to release-input digest
`5e0cc31c1184aaa033a7066e65c336394787aa65889c72792a819e660ac73885` at commit
`f7fd55c56ade21b2fac8091b76993aa51f104245`, with the capture SHA-256
`ee46abcfe76433b6ed25b6f7a53c80056bf36918b8b44009ae8f5511c54167a4`. Pull
requests #4 through #14 and the alpha.11 version bump have changed the non-QA
release inputs since, so that digest no longer describes what would ship. Its
record is kept below for history.

### alpha.10 run

Run 35084991785 passed every gate on Windows:

- frontend checks, and the browser QA suite including its contrast assertions;
- `cargo fmt --check`, workspace clippy with warnings denied, and the Rust
  workspace tests;
- native fixture parsing with the pinned assets, and the verified pinned runtime;
- an NSIS build, and installer and uninstall smoke with user data retained;
- an accepted whole-corpus evaluation with real inference.

That evaluation reproduces alpha.9's naming and date scores exactly: date role
14/14, type 18/18, description specific 19/19, parties 16/19, review rate 42.1%,
and nothing filed under a date the corpus marks as a trap. Descriptions covered
the expected facts for 16 of 18 documents, one more than alpha.9's 15. This
release changes no naming behavior, and the numbers show it did not.

The capture is byte-identical to the accepted alpha.9 frame, with the same SHA-256.
That is expected. The visible additions in this release are the guided SharePoint
setup and the managed SharePoint connection card in Settings, and both appear
only in a build provisioned with a SharePoint deployment. This build is not
provisioned. The model-download retry and the automatic update check add nothing
to this frame. The frame was re-taken because the release inputs changed, not
because it was expected to look different.

The onboarding screens are covered by automated gates rather than this frame:

- unit tests;
- the Playwright fake-deployment journey, which asserts heading focus, visible
  focus rings, no identifier or path inputs, and no horizontal overflow at every
  step.

The capture was inspected against the accepted concept:

- Sidebar counts, the queue table and the review drawer read as three distinct
  planes.
- The header states the privacy posture beside the brand tag.
- Date-first proposed filenames, right-aligned confidence and em-dashes for
  absent values stay consistent down the column.
- Ready, Needs review, Processing and Waiting are each distinguishable by icon as
  well as color.
- Evidence stays attributed under DATE, TYPE and PARTIES headings, with each
  party on its own line.
- The approve and keep actions sit in a pinned bar at the foot of the panel.
- No clipping, collisions, illegible copy, excessive density or ambiguous focus
  were observed.

### Reviewer

Reviewed by the maintainer, Zachary Brenner, who accepted the capture named
above. Claude Opus 5 (Claude Code) inspected it first and wrote this record and
the sign-off at the maintainer's direction.

### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs. Any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.9 record

The alpha.9 sign-off was bound to release-input digest
`c4db219b6db91a98eab8f9fae4e359860825a0f7dacf86772f1c44cb5d5471fd` at commit
`a9a96a482b099c4b848a00df8ee66bb4013caf18`. Pull requests #37 through #40 and the
alpha.10 version bump have changed the non-QA release inputs since, so that
digest no longer describes what would ship. Its record is kept below for history.


Run 34538797800 passed every gate on Windows: frontend checks, the browser QA
suite including its contrast assertions, `cargo fmt --check`, workspace clippy
with warnings denied, the Rust workspace tests, native fixture parsing with the
pinned assets, the verified pinned runtime, an NSIS build, installer and
uninstall smoke with user data retained, and an accepted whole-corpus
evaluation with real inference. That evaluation reproduces alpha.8's scores
exactly - date role 14/14, type 18/18, description specific 19/19, parties
16/19, review rate 42.1%, nothing filed under a date the corpus marks as a trap
- which is the point: this release changes no naming behavior, and the numbers
show it did not.

The capture is materially unchanged from alpha.8's, deliberately. This
release's one visible change is that a window of 1100 pixels or less no longer
opens inside a modal review drawer over an inert queue, and this frame is 1536
wide, where the inspector has always been a side panel. The narrow behavior is
held by the 1024-pixel browser gate and by a unit test rather than by this
picture; the frame was re-taken because the release inputs changed, not because
it was expected to look different.

Everything else in the release sits beneath the window and cannot be
photographed: a shared folder that recovers when a teammate's machine crashes
mid-document, a private-local-intake setting that is re-derived per folder
rather than trusted for ever, coordination files that survive an antivirus or
sync-client hold, description records found whichever way Windows spells the
folder, and a settings file that reports what it could not read instead of
silently starting with defaults.

The capture was inspected against the accepted concept. Core hierarchy and
interaction emphasis align: sidebar counts, queue table and review drawer read
as three distinct planes, and the header states the privacy posture beside the
brand tag. Date-first proposed filenames, right-aligned confidence and
em-dashes for absent values stay consistent down the column. Ready, Needs
review, Processing and Waiting are each distinguishable by icon as well as
color. Evidence stays attributed under DATE, TYPE and PARTIES headings with
each party on its own line, the approve and keep actions sit in a pinned bar at
the foot of the panel, and the selected row carries an accent bar as well as a
tint. No clipping, collisions, illegible copy, excessive density or ambiguous
focus were observed.

This is not a pixel-equality assertion: the reference and implementation can
differ where supported behavior requires it. The native title bar, the
1024-pixel layout, hover states and motion remain covered by the hosted
automated browser and installed-app gates rather than by a claim that one frame
captures every state.

#### Reviewer

Reviewed by the maintainer, Zachary Brenner, who inspected the capture named
above and accepted it. Claude Opus 5 (Claude Code) inspected it first and wrote
this record and the sign-off at the maintainer's direction, the same standing
under which the alpha.6 and alpha.8 records were written.

#### Freshness boundary

The sign-off is accepted only for the digest and screenshot named above.
`scripts/hash-release-inputs.mjs` derives that digest from the committed non-QA
release inputs; any relevant source change invalidates this record and requires
a new capture and review. Exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a tag
or publication is allowed.

## Superseded alpha.6 record

The alpha.6 sign-off was reviewed on 2026-09-02 and bound to the non-QA
release-input digest
`fe016f3944a190dd932d001367fdcc44a360b5cb6fd2030d6fe233e290a1c05c` at commit
`9e91511c12ad9a3e727e3fa56e15b7197de191d2`. Eight pull requests and the alpha.7
version bump have changed the non-QA release inputs since, so that digest no
longer describes what would ship.

## Evidence inspected

- Accepted concept: `docs/design/intern-primary-screen.png`, 1536×1024,
  SHA-256 `c8cf322da777d77bc490b855fd18c5a70fe24192a343505e677d34d925a30de8`.
- Fresh implementation capture: hosted Whole-product QA evidence run
  `33572364279`, Windows/X64, commit
  `9e91511c12ad9a3e727e3fa56e15b7197de191d2`. The capture is 1536×1024 and
  SHA-256 `1bc7bb1707c743fc87106eb3a2a914d234128cefdc20e92a77c88fa7a6b7fb76`.
- The capture differs substantially from alpha.5's: alpha.6 rebuilds the review
  panel. Evidence is now attributed quotations under labeled DATE, TYPE, and
  PARTIES headings, each party on its own line so it can be checked separately;
  the model's confidence appears in the panel at all, with an exact percentage
  beside a proportional track; the approve and keep actions sit in a pinned bar
  at the foot of the panel; the selected row carries an accent bar as well as a
  tint, so it is no longer confusable with hover; and a help affordance sits
  beside the settings gear. The run's automated contrast assertions passed.
- That run completed every substantive automated gate: 19 Vitest files / 146
  tests, four Playwright tests, `cargo fmt --check`, `cargo clippy -D
  warnings`, workspace Rust tests, native fixture tests, verified pinned
  Windows assets, an NSIS build, installed-app smoke, and an accepted
  whole-corpus evaluation with real inference - dates 76.5%, types 88.2%,
  every gate above its floor and zero documents filed under a forbidden
  date, with the new reference-date guard converging the hardware-dependent
  model picks. Its final evidence binding correctly failed closed because the
  prior sign-off was bound to the alpha.5
  `f338fbbe5532f18175dff9d95b2d8fe8f225dba30c2f2da775239b4ca7ef0b89` digest;
  this post-run review supplies the accepted replacement record.

## Review conclusion

The fresh capture was inspected against the accepted concept. Core hierarchy
and interaction emphasis align: sidebar counts, queue table, and review drawer
read as three distinct planes, and the header states the privacy posture in the
chrome beside the new brand tag. Date-first proposed filenames, right-aligned
confidence, and em-dashes for absent values stay consistent down the column.
Ready, Needs review, Processing, and Waiting are each distinguishable by icon
as well as color, and this run's automated contrast assertions for the review
and waiting statuses passed at 4.5:1 on the brand-tinted selection.

No clipping, collisions, illegible copy, excessive density, or ambiguous
focus/selection were observed. Focus is a ring on the filename input and
selection is a tint on the row the drawer describes, so neither is in doubt.
The filename field's right-edge truncation is an input scrolled to offset zero,
which the capture test arranges deliberately so the head of the proposed name
is visible rather than its tail.

This is not a pixel-equality assertion: the reference and implementation can
differ where alpha.6's supported behavior requires it. The native title bar,
1024-pixel layout, hover states, and motion remain supported by the hosted
automated browser and installed-app gates rather than an assertion that the
single 1536×1024 frame captures every state.

## Reviewer

Reviewed by Claude Opus 5 (Claude Code) at the maintainer's direction, who
inspected the capture named above. The reviewer field in the sign-off record
names the same, so a later reader can tell who looked and with what standing.

## Freshness boundary and residuals

The sign-off is accepted only for the digest above and its exact screenshot.
`scripts/hash-release-inputs.mjs` derives that digest from the committed
non-QA release inputs; a relevant source change invalidates the sign-off and
requires a new QA capture and review. The remaining release work is not a
fidelity discrepancy: exact-main validation and the deliberately dispatched
release workflow must still reproduce and accept their own evidence before a
tag or publication is allowed.

# Windows memory and the tier plan: what was measured, November 2026

Status: measured on 2026-11-09 on GitHub-hosted `windows-latest` runners
(4 logical processors, 16 GiB, a 2.9 GiB pagefile), three passes in two runs
of `.github/workflows/runtime-speed.yml` with `probe: memory`. The question
was whether, on Windows, the tier plan judges a loaded 9B not to fit beside
itself and swaps it for the 4B, and the 4B back for the 9B, before every
document. By `plan()`'s arithmetic on the measured numbers it does, for
documents read back to back on a 16 GiB machine with about 10,000 to
13,200 MiB (9.8 to 12.9 GiB) available before any model is loaded. The plan
now credits the live server's working set instead, and asks a loaded 4B for
1 GiB more than the floor before it makes way for the 9B; with the measured
numbers the swap is gone. Figures are in MiB unless they say otherwise.

## The mechanism

The plan runs before every document (`plan()` in
`src-tauri/src/commands.rs`). With a model loaded it asks whether the
accurate model fits in the available memory plus what unloading the loaded
one would give back, against `MIN_AVAILABLE_FOR_ACCURATE` (8,192 MiB). Up to
alpha.14 that credit was an estimate, `tiers::unload_gives_back`: the
resident high-water mark less the model file, 8,300 − 5,417 = 2,883 MiB for
the 9B and 4,700 − 2,614 = 2,086 MiB for the 4B. That is right on Linux,
where `MemAvailable` counts the mapped weights as page cache. On Windows the
available memory is `GlobalMemoryStatusEx`'s `ullAvailPhys`, the free, zero
and standby lists, and a page in a process's working set is on none of them:
the weights the server has touched are not available until it exits or the
system trims them. The `tiers.rs` comment said the standby list made the two
alike. It does not.

## Method

`scripts/probe-windows-memory.ps1` starts upstream llama-server `b11946`
with the app's flags (`server.rs` `arguments_for`: `--parallel 1
--ctx-size 8192 --n-gpu-layers 0 --threads 4 --threads-batch 4 --jinja
--no-webui --cache-ram 0 -ub 256 --prio -1 --poll 0 --no-mmproj`, four
threads by `default_threads`, below-normal priority) for the pinned 9B, then
the pinned 4B, and samples `ullAvailPhys`, the memory lists, and the
server's working set, private working set, private bytes and peak working
set (`GetProcessMemoryInfo` on the process handle, the call the app now
makes) at six points: before the load, after `/health`, after one recorded
request (`speed-linux-cpu-2026-10-29/requests/0001`, 1,295 prompt tokens),
after 60 s idle, and 3 s and 15 s after the server exits. It then applies
`plan()`'s arithmetic with the constants read from `tiers.rs`. The third
pass ran with a second process holding 2 GiB in its working set (it held
2,171 MiB), as an open browser does, so the machine started with less
available, as a laptop does.

| Pass | Run | Runner CPU | Available before any model |
| --- | --- | --- | ---: |
| 1 | 42573531984 | AMD EPYC 9V74 | 13,205 MiB |
| 2 | 42574460663 | AMD EPYC 7763 | 12,873 MiB |
| 3, 2 GiB held | 42574460663 | AMD EPYC 7763 | 11,461 MiB |

Raw output: `run-42573531984/` and `run-42574460663/{memory,memory-balloon}/`.
`memory-probe.json` holds every sample, the verdicts and the request
timings (`replies`); `summary.md` holds the tables the probe printed. Both
were written before the probe relabelled the estimate: their
`shipped_credit_mib` and "Shipped credit" are what the probe now calls
`estimate_credit_mib` and "Estimate (fallback)". The server's own logs are
not committed (the repository ignores `*.log`); they are in each run's
`windows-memory-probe` artifact until 2026-12-09.

## What the loaded servers hold (MiB)

| Pass | Model | Stage | Available | Working set | Private working set | Private bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 1 | 9B | after `/health` | 5,382 | 7,896 | 3,021 | 3,375 |
| 1 | 9B | after a document | 5,410 | 8,110 | 3,231 | 3,540 |
| 1 | 9B | after 60 s idle | 6,081 | 7,528 | 3,177 | 3,540 |
| 2 | 9B | after `/health` | 5,154 | 7,896 | 3,021 | 3,375 |
| 2 | 9B | after a document | 5,570 | 8,109 | 3,230 | 3,540 |
| 2 | 9B | after 60 s idle | 6,924 | 6,685 | 3,176 | 3,540 |
| 3 | 9B | after `/health` | 3,571 | 7,896 | 3,021 | 3,376 |
| 3 | 9B | after a document | 3,535 | 8,107 | 3,230 | 3,539 |
| 3 | 9B | after 60 s idle | 5,865 | 6,332 | 3,176 | 3,539 |
| 1 | 4B | after a document | 9,165 | 4,513 | 1,896 | 2,193 |
| 2 | 4B | after a document | 9,044 | 4,513 | 1,896 | 2,192 |
| 3 | 4B | after a document | 8,234 | 4,514 | 1,897 | 2,193 |
| 3 | 4B | after 60 s idle | 8,815 | 4,445 | 1,829 | 2,193 |

- **The 9B's working set right after a document is 8,107-8,110 MiB
  (7.9 GiB)** in every pass (peak 8,142-8,143), matching the 8,124 MiB peak the
  alpha.14 QA measured (`docs/qa/fidelity-ledger.md`). Of it, about
  3,230 MiB is private (the repacked weights, the KV cache, the
  checkpoints: Linux measured 3,317 MiB of `RssAnon`) and about 4,880 MiB
  is the mapped file: all of it but about the size of the token
  embeddings, which are read a row at a time. On Linux the 9B's working set
  under a memory limit was about 6.2 GB (`speed-linux-cpu-2026-11-05`),
  because Linux reclaims the repack originals first; Windows holds them
  until it trims.
- **Windows trims an idle server, but not between documents in a queue.**
  60 s after the document, the 9B's working set had fallen by 581, 1,424
  and 1,776 MiB in the three passes, nearly all of it file-backed (the
  private working set fell by 54 MiB), moved to the standby list and so
  counted available again. The 4B lost 0 to 69 MiB. The free list was at
  2-86 MiB whenever the 9B was loaded: everything available was standby.
- **The working set is what an exit gives back.** In all six stops, the
  available memory 3 s after the exit equalled the available memory before
  it plus the working set before it, to within 83 MiB (9B: +50, +55, +2;
  4B: +2, −1, +83).

## The plan's verdicts with the estimate alone

| Pass | Loaded | Stage | Available | Credit | Plan sees | Accurate model |
| --- | --- | --- | ---: | ---: | ---: | --- |
| 1 | 9B | after `/health` | 5,382 | 2,883 | 8,264 | fits |
| 1 | 9B | after a document | 5,410 | 2,883 | 8,292 | fits |
| 2 | 9B | after `/health` | 5,154 | 2,883 | **8,037** | **does not fit** |
| 2 | 9B | after a document | 5,570 | 2,883 | 8,452 | fits |
| 3 | 9B | after `/health` | 3,571 | 2,883 | **6,453** | **does not fit** |
| 3 | 9B | after a document | 3,535 | 2,883 | **6,417** | **does not fit: the 4B is loaded** |
| 3 | 9B | after 60 s idle | 5,865 | 2,883 | 8,748 | fits |
| 2 | 4B | after a document | 9,044 | 2,086 | 11,130 | fits: the 9B is loaded again |
| 3 | 4B | after a document | 8,234 | 2,086 | **10,320** | **fits: the 9B is loaded again** |

Sums are of the unrounded figures, with the credit the app works out from
the pinned files' sizes: 2,882.6 MiB for the 9B, 2,086.0 for the 4B. The
probe rounded the credit to whole MiB, so its own tables and the committed
`plan_sees_mib` run up to 0.4 MiB higher; no verdict differs. Which stage a
plan sees depends on when it runs. A plan made with a model loaded usually
follows a document, so it sees the after-a-document stage. The exception is
the first document after a launch with a model already vouched for (stamped
by an earlier self-test): setup starts the model the launch's plan chose
and returns, so with documents already waiting, the first one is planned
against a server that has just started, about the after-`/health` stage. A
model not yet vouched for reads the self-test's probes before setup
returns, so its first plan sees a server that has already read. (After an
idle unload the plan is made with nothing loaded and the document is read
right after the start, with no plan between.)

On the runner as it comes, with 12,873-13,205 MiB available and nothing
else running, the loaded 9B passes after a document by 100 and 260 MiB.
Right after a start it passed by 72 MiB in pass 1 and failed by 155 MiB in
pass 2: there, with documents waiting at a launch with the 9B vouched for,
the plan would have read
the first with the 4B, provisionally, loaded the 9B again for the second
(with the 4B loaded it sees 11,130 MiB), and kept it after that. With
2 GiB held, as on a laptop with a browser open, documents read back to back
go: before the second (or, at launch, the first) the plan sees 6.3 GiB and
loads the 4B, whose reading is provisional; before the next it sees
10.1 GiB with the 4B loaded and loads the 9B again; and so on. Each pair of
documents costs a 9B load, a 4B load, and a 4B reading that is provisional
and is read again by the 9B later. A minute's pause lets Windows trim the
9B enough to stay (the plan sees 8,748 MiB after 60 s idle), so the swap is
a queue's, not a single document's.

The bands below extrapolate from one load per pass: each pass's drop in
available memory from before the load to after a document is taken to be
the same whatever the machine has available. That is a linear
extrapolation; Windows trims harder when less is available, so the edges
are approximate. On that basis a machine with A MiB available before any
model is loaded, reading documents back to back (edges rounded to the
nearest MiB):

| Pass | Swap before every document | 9B reads one document, the 4B the rest |
| --- | --- | --- |
| 1 | 10,610 ≤ A < 13,105 | 8,192 ≤ A < 10,610 |
| 2 | 10,729 ≤ A < 12,612 | 8,192 ≤ A < 10,729 |
| 3 | 10,035 ≤ A < 13,236 | 8,192 ≤ A < 10,035 |

In the second band the 4B, once loaded, never sees room for the 9B and
reads everything provisionally until the idle unload, ten minutes after the
last document. Only above 12,612-13,236 MiB (12.3-12.9 GiB) available,
depending on the pass, a 16 GiB machine with almost nothing else open, did
the estimate keep the 9B after a document. Planned right after the start,
as the first document after a launch with the 9B vouched for is, the same
arithmetic on the after-`/health` drop stands the 9B down below 13,133,
13,028 and 13,200 MiB in the three passes.

## The change

`plan()` credits what the live server holds that the machine does not
already count (`tiers::memory_released_by`, read through
`LlamaServer::memory_released`): on Windows its working set
(`K32GetProcessMemoryInfo` on the handle `Child` owns), on Linux its
`RssAnon` from `/proc/<pid>/status`. The estimate stays as the fallback
when the process cannot be read or a start or stop holds the server's slot.

Credited the working set, the plan sees what the machine would have once
the loaded server exits: at the six exits the probe made, each after the
idle stage, the plan's figure just before and the available memory 3 s
after differed by 83 MiB at most. It is not exactly what the machine had
before the model was loaded: over the 18 samples with a model loaded, the
plan's figure ran from 110 MiB under that to 1,098 MiB over it. With pass
3's numbers:

| Loaded | Stage | Available | Working set | Plan sees | Accurate model | Before the load | 3 s after the exit |
| --- | --- | ---: | ---: | ---: | --- | ---: | ---: |
| 9B | after `/health` | 3,571 | 7,896 | 11,466 | fits | 11,461 | |
| 9B | after a document | 3,535 | 8,107 | 11,642 | fits | 11,461 | |
| 9B | after 60 s idle | 5,865 | 6,332 | 12,197 | fits | 11,461 | 12,198 |
| 4B | after a document | 8,234 | 4,514 | 12,748 | fits | 12,162 | |
| 4B | after 60 s idle | 8,815 | 4,445 | 13,260 | fits | 12,162 | 13,343 |

So with a model loaded the plan makes about the test a launch with nothing
loaded makes, whichever model is loaded, but not exactly, and not alike for
the two models. In pass 3 the plan's figure came out higher with the 4B
loaded than with the 9B: 585 MiB over the memory before the load with the
4B right after a document, and 1,098 MiB after a minute idle, against 181
MiB with the 9B right after a document. Carried to other machines as the
bands above are, and credited the working set alone, a machine with A MiB
available before any model is loaded, reading back to back, has the loaded
9B stood down (A + 181 < 8,192) and the loaded 4B brought back (A + 585 ≥
8,192) for 7,607 ≤ A < 8,011: a swap before every document again, in a band
about 404 MiB wide instead of about 3,200. With a minute or more between
documents the 4B's figure is A + 1,098, and the band is 7,094 ≤ A < 8,011,
a swap at every pause. Passes 1 and 2 have no such band: there the 9B's
figure was the higher (+314 and +806 after a document, against +9 and −110
for the 4B). The band is a squeeze: the 9B's load took 7,926 MiB of the
available memory, so carried below A = 7,926 the extrapolation leaves the
9B less than nothing, and the figures there say little more than that the
4B's can come out the higher. What they do say is that without a margin the
plan would load the 9B, from the 4B, onto a machine that cannot hold it.

The plan therefore also asks a loaded 4B for 1,024 MiB more than the floor
before it makes way for the 9B (`tiers::UPGRADE_MARGIN`, applied in
`tiers::memory_for_plan`). With pass 3's numbers the 4B then comes back
only for A ≥ 8,631 after a document, or A ≥ 8,118 after a minute idle, both
above the 8,011 under which the 9B is stood down, so neither band is left.
The loaded 9B is still stood down at the floor itself: the margin keeps the
smaller model longer, never the larger one. It also covers a machine whose
memory drifts around the floor, which the plan without it would swap back
and forth on noise: the 9B is stood down once and comes back only with
1,024 MiB to spare. The cost is that a machine whose 4B is loaded and which
has between 0 and 1,024 MiB more than the floor by the plan's figure keeps
the 4B, its readings provisional, until memory frees up or the idle unload;
a launch with nothing loaded still loads the 9B at the floor.

On Linux the credit for the 9B moves from 2,883 to about 3,317 MiB (its
`RssAnon` in `bench/reports/speed-linux-cpu-2026-11-05`) and for the 4B
from 2,086 to a little less: a research run for this work, not committed
here, read 1,704 MiB of `RssAnon` after the load and 1,900 MiB after one
request on the AVX2 (`haswell`) backend. That is a few hundred MiB either
way, and the margin applies on Linux too.

Tests (`src-tauri/src/commands.rs`, a fake server reporting a working set):
`the_windows_numbers_no_longer_swap_the_models_document_after_document`
(pass 3's numbers: the estimate, with the pinned files' sizes, would swap;
the runtime credits the server's own figure and keeps the 9B over three
documents; and a loaded 4B makes way for it),
`a_full_machine_still_stands_the_quick_model_in` (a machine just short with
nothing loaded gets the 4B, and keeps it, by the server's own figure),
`a_machine_near_the_floor_keeps_the_quick_model_through_the_queue` (pass
3's numbers on a machine with 7,950 MiB available with nothing loaded, 242
MiB short of the floor: the 9B is stood down, the 4B stays through three
documents though the working set alone would bring the 9B back, and the 9B
comes back at the floor and the margin, not a byte before), and
`a_server_that_cannot_say_what_it_holds_is_credited_the_estimate` (the
fallback, at the floor and the margin and a byte under). `tiers.rs` tests
the margin's arithmetic
(`only_a_loaded_fast_model_is_asked_for_the_margin`) and carries pass 3's
numbers to every A from 7,000 to 9,999 MiB
(`a_loaded_fast_model_makes_way_only_with_room_to_spare`): credited the
working set alone the models swap for 7,607-8,010 MiB after a document and
7,094-8,010 MiB after a minute idle; with the margin, at none. The test
fixtures' model files are a few bytes, so in those runtimes the fallback
estimate is nearly the whole resident estimate, not the pinned models'
2,883 and 2,086 MiB: the arithmetic with the pinned sizes is what shows the
estimate swapping. `tiers.rs` tests the `RssAnon` parser and reads a real
child process on Linux and Windows.

## Not measured

- A real laptop. The runner has no browser, no office suite, no antivirus
  scan of its own and a 2.9 GiB pagefile; the 2 GiB process stands in for
  everything a person has open.
- A squeeze. Nothing here ran with less available than the 9B needs, so
  how Windows trims and pages the server when memory is genuinely short,
  and whether the 8 GiB floor is right there, is still open (research G3).
  The bands above are extrapolated, not measured at each level. Under a
  squeeze more of the server's private memory may sit on the modified list,
  which its working set does not count, the available memory does not
  count either, and its exit frees: the credit could then run short.
- The margin's size. It covers the largest gap measured between the two
  models' figures, 917 MiB (pass 3, the 4B a minute idle against the 9B
  right after a document), with about 100 MiB to spare, on one load per
  model per pass. A machine where the gap is wider could still swap the
  models, at a pause rather than before every document if it is the idle
  4B's figure that is higher.
- The app itself. The verdicts are `plan()`'s arithmetic on measured
  numbers, and the change is covered by unit tests with a fake server; no
  installed build was run on Windows with both models.
- More than one request. The probe reads one document per model; how the
  working set moves over a long queue was not followed.

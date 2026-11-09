# Windows memory and the tier plan: what was measured, November 2026

Status: measured on 2026-11-09 on GitHub-hosted `windows-latest` runners
(4 logical processors, 16 GB, a 2.9 GB pagefile), three passes in two runs
of `.github/workflows/runtime-speed.yml` with `probe: memory`. The question
was whether, on Windows, the tier plan judges a loaded 9B not to fit beside
itself and swaps it for the 4B, and the 4B back for the 9B, before every
document. It does, on a 16 GB machine with about 10 to 13 GB available
before any model is loaded. The plan now credits the live server's working
set instead, and with the measured numbers the swap is gone.

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

Raw output: `run-42573531984/` and `run-42574460663/{memory,memory-balloon}/`
(`memory-probe.json` holds every sample, `summary.md` the verdicts, the
`*-server.log` files the server's own log).

## What the loaded servers hold (MiB)

| Pass | Model | Stage | Available | Working set | Private working set | Private bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 1 | 9B | after `/health` | 5,382 | 7,896 | 3,021 | 3,375 |
| 1 | 9B | after a document | 5,410 | 8,110 | 3,231 | 3,540 |
| 1 | 9B | after 60 s idle | 6,081 | 7,528 | 3,177 | 3,540 |
| 2 | 9B | after a document | 5,570 | 8,109 | 3,230 | 3,540 |
| 2 | 9B | after 60 s idle | 6,924 | 6,685 | 3,176 | 3,540 |
| 3 | 9B | after a document | 3,535 | 8,107 | 3,230 | 3,539 |
| 3 | 9B | after 60 s idle | 5,865 | 6,332 | 3,176 | 3,539 |
| 1 | 4B | after a document | 9,165 | 4,513 | 1,896 | 2,193 |
| 2 | 4B | after a document | 9,044 | 4,513 | 1,896 | 2,192 |
| 3 | 4B | after a document | 8,234 | 4,514 | 1,897 | 2,193 |
| 3 | 4B | after 60 s idle | 8,815 | 4,445 | 1,829 | 2,193 |

- **The 9B's working set right after a document is 8.1 GB** in every pass
  (8,107-8,110 MiB; peak 8,142), matching the 8,124 MiB peak the alpha.14
  QA measured (`docs/qa/fidelity-ledger.md`). Of it, 3.2 GB is private (the
  repacked weights, the KV cache, the checkpoints: Linux measured 3.3 GB of
  `RssAnon`) and 4.9 GB is the mapped file: all of it but about the size of
  the token embeddings, which are read a row at a time.
  On Linux the 9B's working set under a memory limit was 6.2 GB
  (`speed-linux-cpu-2026-11-05`), because Linux reclaims the repack
  originals first; Windows holds them until it trims.
- **Windows trims an idle server, but not between documents in a queue.**
  60 s after the document, the 9B's working set had fallen by 0.6, 1.4 and
  1.8 GB in the three passes, nearly all of it file-backed (the private
  working set fell by 54 MiB), moved to the standby list and so counted
  available again. The 4B lost 0 to 69 MiB. The free list was at 2-86 MiB whenever the
  9B was loaded: everything available was standby.
- **The working set is what an exit gives back.** In all six stops, the
  available memory 3 s after the exit equalled the available memory before
  it plus the working set before it, to within 83 MiB (9B: +50, +55, +2;
  4B: +2, −1, +83).

## The plan's verdicts with the shipped credit

| Pass | Loaded | Stage | Available | Credit | Plan sees | Accurate model |
| --- | --- | --- | ---: | ---: | ---: | --- |
| 1 | 9B | after a document | 5,410 | 2,883 | 8,293 | fits |
| 2 | 9B | after `/health` | 5,154 | 2,883 | 8,037 | does not fit |
| 2 | 9B | after a document | 5,570 | 2,883 | 8,453 | fits |
| 3 | 9B | after a document | 3,535 | 2,883 | **6,418** | **does not fit: the 4B is loaded** |
| 3 | 9B | after 60 s idle | 5,865 | 2,883 | 8,748 | fits |
| 3 | 4B | after a document | 8,234 | 2,086 | **10,320** | **fits: the 9B is loaded again** |

On the runner as it comes, with 12.9-13.2 GB available and nothing else
running, the loaded 9B passes by 100-260 MiB. With 2 GiB held, as on a
laptop with a browser open, documents read back to back go: the 9B reads
the first; before the second the plan sees 6.4 GB and loads the 4B, whose
reading is provisional; before the third it sees 10.3 GB with the 4B loaded
and loads the 9B again; and so on. Each pair of documents costs a 9B load, a
4B load, and a 4B reading that is provisional and is read again by the 9B
later. A minute's pause lets Windows trim the 9B enough to stay (the plan
sees 8.7 GB after 60 s idle), so the swap is a queue's, not a single
document's.

From the measured loads (each pass's drop in available memory after a
document), a machine with A MiB available before any model is loaded
swaps before every document for:

| Pass | Swap before every document | 9B reads one document, the 4B the rest |
| --- | --- | --- |
| 1 | 10,610 ≤ A < 13,104 | 8,192 ≤ A < 10,610 |
| 2 | 10,729 ≤ A < 12,612 | 8,192 ≤ A < 10,729 |
| 3 | 10,035 ≤ A < 13,235 | 8,192 ≤ A < 10,035 |

In the second band the 4B, once loaded, never sees room for the 9B and
reads everything provisionally until the idle unload, ten minutes after the
last document. Only above about 13 GB available, a 16 GB machine with
almost nothing else open, did the shipped credit keep the 9B.

## The change

`plan()` credits what the live server holds that the machine does not
already count (`tiers::memory_released_by`, read through
`LlamaServer::memory_released`): on Windows its working set
(`K32GetProcessMemoryInfo` on the handle `Child` owns), on Linux its
`RssAnon` from `/proc/<pid>/status`. The estimate stays as the fallback
when the process cannot be read or a start or stop holds the server's slot.
The plan then sees what the machine had with no model loaded, whichever
model is loaded, and makes the same test a launch with nothing loaded
makes:

| Pass | Loaded | Stage | Available | Working set | Plan sees | Before any model |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 3 | 9B | after a document | 3,535 | 8,107 | 11,642: fits | 11,461 |
| 3 | 9B | after 60 s idle | 5,865 | 6,332 | 12,197: fits | 11,461 |
| 3 | 4B | after a document | 8,234 | 4,514 | 12,748: fits | 12,162 |

A machine that is short with nothing loaded is short with either model
loaded, so the 4B still stands in there. On Linux the credit for the 9B
moves from 2,883 to about 3,317 MiB (its measured `RssAnon`) and for the 4B
from 2,086 to about 1,900 (AVX2 backend), a few hundred MiB either way.

Tests (`src-tauri/src/commands.rs`, a fake server reporting a working set):
`the_windows_numbers_no_longer_swap_the_models_document_after_document`
(pass 3's numbers: the shipped estimate would swap, the measured credit
keeps the 9B over three documents, and a loaded 4B makes way for it),
`a_full_machine_still_stands_the_quick_model_in` (a machine just short
with nothing loaded gets the 4B, and keeps it, where the estimate would
have kept the 9B), and
`a_server_that_cannot_say_what_it_holds_is_credited_the_estimate` (the
fallback, at the floor and a byte under). `tiers.rs` tests the `RssAnon`
parser and reads a real child process on Linux and Windows.

## Not measured

- A real laptop. The runner has no browser, no office suite, no antivirus
  scan of its own and a 2.9 GB pagefile; the 2 GiB process stands in for
  everything a person has open.
- A squeeze. Nothing here ran with less available than the 9B needs, so
  how Windows trims and pages the server when memory is genuinely short,
  and whether the 8 GiB floor is right there, is still open (research G3).
- The app itself. The verdicts are `plan()`'s arithmetic on measured
  numbers, and the change is covered by unit tests with a fake server; no
  installed build was run on Windows with both models.
- More than one request. The probe reads one document per model; how the
  working set moves over a long queue was not followed.

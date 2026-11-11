# Two OCR pages at once on four logical processors, November 2026

Status: measured on 2026-11-09, on a GitHub-hosted runner and on the
shared 4-core box. The question: from four logical processors the worker
now reads two scanned pages at once, two threads each, where alpha.14 read
one (`paddle::thread_budget`); how much sooner are scans read, at what
memory, and is anything read differently?

**How.** `scripts/bonsai-eval/ocr-pages.py` sends every InternBench
document (77, of which 20 with OCR, 56 pages) to two parser workers over
the worker's own protocol, one document at a time, and alternates them:
base, head, head, base, base, head. Base is the worker at `ebc5d8f`
(alpha.14), head the worker at `d24fc11`; both built from source with
`--features windows-native` on the machine that ran them, reading the same
runtime (ONNX Runtime, PDFium and the pinned OCR models). Each document is
timed from the parse command to the parsed reply; each worker's peak
resident memory over a run is its `VmHWM`; each document's reading (every
page's number, text, source and OCR confidence, and the document's
warnings) is hashed. Every run's per-document figures are in the JSON files
here, beside `ocr-pages.json`, the summary.

Both machines had far more than 5 GiB available at every document's start
(at least 13.9 GiB on the runner and 7.8 GiB on the box), so head read two
pages at once throughout (`paddle::pages_at_once`).

## The runner: two cores, four threads

`speed-battery.yml` run 42613236166, the `ocr_pages` job
([`runner-42613236166/`](runner-42613236166/)): an Intel Xeon Platinum
8573C, 2 cores with 2 threads each, nothing else running.

| Worker | Whole set | The 20 documents with OCR | `scan-lease-25p` | `scan-agreement-10p` | Peak resident |
| --- | ---: | ---: | ---: | ---: | ---: |
| base: 1 page x 2 threads | 120.1, 116.5, 117.3 s | 118.3, 115.6, 116.4 s | 53.5, 52.6, 53.1 s | 24.6, 23.5, 23.8 s | 779, 826, 915 MB |
| head: 2 pages x 2 threads | 108.9, 108.4, 105.5 s | 107.9, 107.5, 104.6 s | 47.1, 46.6, 44.9 s | 21.1, 21.5, 20.7 s | 1,317, 1,304, 1,312 MB |

The documents with OCR were read 8.7% sooner (116.8 s against 106.7 s on
the mean), the 25-page lease 13% sooner, in every round; the rounds spread
by at most 3 s. Every document was read the same in all six runs.

## The shared box: four cores, one thread each

The 4-core Xeon these reports use ([`local-4core/`](local-4core/)), shared
with other work; `load.txt` is its one-minute load average every 30 s
through the runs, of which the worker's own threads are about 2 (base) and
4 (head).

| Worker | Whole set | The 20 documents with OCR | `scan-lease-25p` | `scan-agreement-10p` | Peak resident |
| --- | ---: | ---: | ---: | ---: | ---: |
| base: 1 page x 2 threads | 138.0, 153.0, 142.1 s | 136.3, 151.7, 140.8 s | 61.4, 67.0, 63.1 s | 27.2, 32.1, 26.7 s | 747, 816, 813 MB |
| head: 2 pages x 2 threads | 97.2, 130.2, 139.1 s | 95.9, 128.8, 137.9 s | 33.9, 58.0, 56.1 s | 14.8, 16.3, 30.0 s | 1,382, 1,368, 1,421 MB |

The first round ran with the least other work on the box (load 2.1-3.7):
head read the documents with OCR 30% sooner than base and the lease 45%
sooner. Head's second and third rounds ran while other work took the load
to 5-6 and gained little; over all three rounds the documents with OCR
were read 15% sooner. Every document was read the same in all six runs.

## What it says

Two pages at once gain what the processors a page leaves idle can give: up
to about 30% on four idle cores, about 9% on two cores with two threads
each, and little on a machine busy with other work. The readings do not
change. The worker's peak rises from 0.75-0.92 GB to 1.30-1.42 GB, which is
why the second page waits for 5 GiB of available memory: the tier plan
counts one page (`tiers::WORKER_PEAK`).

The figures that review could not reproduce (30% over two rounds on the
shared box, from runs whose output was not kept) are replaced by these.

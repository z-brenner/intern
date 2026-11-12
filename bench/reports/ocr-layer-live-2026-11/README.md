# Somebody else's OCR layer, read again: live names, November 2026

alpha.18 re-reads, with PP-OCR, a scanned page whose own invisible OCR layer
would read badly - every line repeated, or prose torn into a cell per word
([document routing](../../../docs/document-routing.md)). Over the 448
development files that changed the extracted text of 28 documents and of no
other. A replay cannot measure what that does to names, since the model is
asked about different text, so the four development sets that hold 27 of the
28 were read live.

## The runs

`candidate-bench.yml` run 43302007759: the alpha.18 candidate at `343ee39`,
Qwen3.5 9B with `--no-repack`, four threads, token traces, sets real3-6, one
AMD EPYC runner per set (real2, which holds the 28th document, was not run).
Each recording was replayed through the release engine at the gate of 0.8
and compared, document by document, with alpha.17's own recordings of the
same sets replayed through alpha.17's engine (the traced 9B recordings of
2026-11-11 for real3-5, run 43065240563 for real6).

The two sides differ in three ways at once: the text of the 28 documents,
the engine, and the runner that gave the replies. The other documents of the
same sets, whose text did not change, show what the runner and the engine
alone do.

## Results

| | Documents | Names right, alpha.17 → alpha.18 | Filed without review, right / wrong, alpha.17 → alpha.18 |
| --- | ---: | --- | --- |
| Text changed by the re-read | 27 | 7 → 9 | 1 / 0 → 4 / 0 |
| Text unchanged | 166 | 83 → 83 | 5 / 1 → 7 / 1 |

On the changed documents, names gained: real4-notice-13, real5-letter-01,
real6-letter-11, real6-notice-07 (and real4-notice-14, real5-letter-02 and
real6-notice-11 are now right and filed without review where alpha.17 held
them); names lost: real4-notice-15 (the re-read of the certified-mail
receipt on its last page reads "OTLO Columbia Plywood Corporation", and the
model took that line's name) and real5-letter-03. Both lost names are held
for review.

### The one wrong name, found and fixed before release

The candidate itself filed real4-notice-14 without review as
"2025-10-28 Notice of Civil Penalty Assessment and Order **Case** from
Department of Environmental Quality". The re-read joined the subject line
and its case number into one line ("Re: Notice of Civil Penalty Assessment
and Order Case No. WQ-SW-WR-2023-626"), and the engine took the type from
the cited line up to "No.", keeping the number's label. The name is otherwise
one the gold accepts. Fixed in the engine (`phrases::labels_the_number`): a
word on a closed list of words that label a reference number - case, docket,
file, matter, cause, index, project, reference, control, tracking - standing
right after the type and before its number is no part of the type. The table
above is the fixed engine's. The fix was written from real4, a development
set; over every recording it changes only the three DEQ notices of real4
(two names gained, one wrong filing gone), and nothing on the held-out
seventh set.

A second cause was examined and left: the notice is laid out as a letter,
and validation does not read the organization in its inside address as the
recipient, as it does for a letter. Reading it there gained no name on any
recording, and reviewers found seven ways it could file a wrong one (a "cc"
line, an agent "c/o", a class named like a firm), so it was not made.

## What this does not show

- 27 documents on one runner per set: a name or two either way is within what
  two runners' replies differ by. The unchanged documents moved by 0 names
  right and 2 right filings across the same runs.
- Time: the re-read adds 0.5-5.5 s to each of 18 of the 28 documents and saves
  up to a second on 10 (`docs/document-routing.md`); not re-measured here.

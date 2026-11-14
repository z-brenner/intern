# The seventh real set, held out, November 2026

What [`prereg.md`](prereg.md) fixed before any model read the set, measured.
As of the run below, no rule, threshold, prompt or flag in alpha.18 was
written from these documents, and nothing was chosen by these results. One
more round of engine rules followed it; what that round changed on this set,
and what it knew of this set, is at the [end](#after-the-last-engine-round-14-november-2026).

## The run

`candidate-bench.yml` run 43302007759, job "a18-9b on real7": the alpha.18
release candidate at `343ee39` (the pull request's head after the four
alpha.18 packages were merged), Qwen3.5 9B Q4_K_M as the model manifest pins
it (`03b74727...`), `--no-repack` as the app launches it, four threads, token
traces kept, on an AMD EPYC 9V45 runner; 1,597 s for the set.

54 of the 55 documents were fetched and read. `real7-invoice-05` was not
fetched: its source no longer matches the size and SHA-256 the sample pins.
51 of the 54 have a scored filename (gold sends three to review with no
name).

The recording was replayed through both engines on the same replies, at the
gate of 0.8: alpha.18's (`0c5f3e9`) and alpha.17's (`v0.1.0-alpha.17`,
`e47a756`). No replay was stale. The documents' text is alpha.18's worker's
in both: the replay reads the recording's own extracted text, so a document
alpha.17's worker would have read differently is not told apart by
staleness; alpha.18's changes to extraction (the re-read of somebody else's
OCR layer, a blank page, the orientation stop) are the only ways they could
differ.

## Results

Both engines read every document of the set identically: the same name,
readiness and reasons on all 54.

| | alpha.18 | alpha.17 |
| --- | ---: | ---: |
| Documents fetched / read / scored | 54 / 54 / 51 | 54 / 54 / 51 |
| Names right | 19 of 51 (37%) | 19 of 51 |
| Type right | 46 of 54 (85%) | 46 of 54 |
| Date right | 47 of 51 (92%) | 47 of 51 |
| Parties right | 33 of 54 (61%) | 33 of 54 |
| Filed without review at 0.8, right / wrong | 3 / 1 | 3 / 1 |
| Held for review | 50 of 54 (93%) | 50 of 54 |

Names right by kind (alpha.18): letters 5 of 15, notices 4 of 9, invoices 2
of 6, forms 2 of 6, minutes 2 of 5, memoranda 1 of 6, contracts 1 of 2,
receipts 1 of 2, statements 1 of 2, purchase orders 0 of 1.

### The names filed without review

| Document | Least token probability | Name | Right |
| --- | ---: | --- | --- |
| real7-minutes-05 | 0.994 | 2026-08-20 Board Meeting Minutes from Cedar Valley Water Conservancy | yes |
| real7-letter-07 | 0.933 | 2024-03-19 Review of Large Energy Users Connection Policy for Uisce Éireann | **no** |
| real7-notice-08 | 0.885 | 2026-08-12 Notice Of Conclusion of Audit from Noak Bridge Parish Council | yes |
| real7-contract-01 | 0.879 | 2025-10-01 Lease of City Real Property between City of San Leandro and As Kneaded Bakery | yes |

`real7-letter-07` is Uisce Éireann's response to a consultation of the
Commission for Regulation of Utilities. The model took the consultation's
title for the document's type and called the respondent a "client"; the
engine named it "for" the respondent as a record's counterpart. The gold
names it a response or letter from Uisce Éireann, or to the Commission.
alpha.17's engine files the same name from the same reply, so the
pre-registration's condition (alpha.18 filing a wrong name alpha.17 would
hold) is not met. Nothing was changed for it: a change written from it would
be fitted to this set.

### The review gate, out of sample

The gate of 0.8 was chosen on the sixth set (`bench/reports/review-gate-2026-11`).
On this one, at the 0.7 alpha.16 used, 8 names would have been filed
without review, 5 right and 3 wrong (`real7-letter-13` at 0.750 and
`real7-minutes-04` at 0.754 besides `real7-letter-07`); at 0.8, 3 right and
1 wrong. The gate held two of the three wrong names and two of the five
right ones.

## What this does not show

- 51 scored documents cannot certify a rate; one or two filings either way is
  within what two runners' replies differ by.
- alpha.18 and alpha.17 differ here only through the engine, which reads this
  set the same; alpha.18's other changes (the quicker model's launch,
  startup, review choices, re-validation, the OCR layer re-read) are not
  measured by it. 15 of the 54 were read by OCR, wholly or in part; the bench
  records each page's route but not why it went to OCR, so whether any of
  them took the re-read of somebody else's OCR layer is not known from this
  run.
- The names right (37%) are lower than on the sixth set under alpha.17 (19 of
  45, 42%): different documents, not a change in the engine, which reads both
  sets identically to alpha.17.

## Reproducing

Download the run's `candidate-a18-9b-real7-43302007759` artifact, write the
gold with `scripts/realdocs-eval/fetch.py OUT sample-heldout6.json` (or from
the sample's own entries for the 54 documents read), and replay:

    intern-bench run --corpus <empty dir> --gold gold.json \
      --replay recording.json --min-token-confidence 0.8 --allow-stale \
      --output report.json

## After the last engine round (14 November 2026)

After the run above, one more round of engine rules was written from the
development recordings (the rows measured on 2026-11-14 in
[`docs/accuracy-research-2026-10.md`](../../../docs/accuracy-research-2026-10.md)):
rules of validation and naming only, with the prompt unchanged, so the same
recording replays through them. It was replayed once through the merged
engine (`9aa8204`, gate 0.8, no stale reply), and no rule was changed after
that replay.

| | Before the round (as above) | After |
| --- | ---: | ---: |
| Names right | 19 of 51 | 19 of 51 |
| Type right | 46 of 54 | 47 of 54 |
| Date right | 47 of 51 | 47 of 51 |
| Parties right | 33 of 54 | 33 of 54 |
| Joining word right | 32 of 54 | 34 of 54 |
| Filed without review at 0.8, right / wrong | 3 / 1 | 3 / 0 |
| Held for review | 50 of 54 (93%) | 51 of 54 (94%) |

Seven documents read differently:

| Document | Before | After | |
| --- | --- | --- | --- |
| real7-letter-07 | filed: Review of Large Energy Users Connection Policy for Uisce Éireann | in review: Letter from Commission for Regulation of Utilities | wrong both; no longer filed |
| real7-notice-05 | SBA Procedural Notice to All SBA Employees | SBA Procedural Notice from SBA | wrong, now right; in review |
| real7-notice-01 | Notice of hearing for ATCO Gas and Pipelines Ltd | Notice of hearing from ATCO Gas and Pipelines Ltd | right, now wrong; in review |
| real7-notice-04 | Notice of Public Hearing for Murray City Planning Commission | ... from Murray City Planning Commission | right both; in review |
| real7-notice-02 | Notice of Proposal to Make Cork City for Cork City Council | ... from Cork City Council | wrong both; in review |
| real7-notice-06 | Notice for Mt. Sterling Water & Sewer | Notice from Mt. Sterling Water & Sewer | not scored; in review |
| real7-invoice-07 | Document - CONSTANTIN | the same, with `PARTY_UNSUPPORTED` added | wrong both; in review |

What this is, and is not:

- The people and processes that proposed, built and reviewed the round's
  rules did not open this set, its gold or its replays; the round's brief
  forbade it and each reported that it had not. They did not work blind to
  it, though, because the brief they worked from was written by the round's
  coordinator, who had read the result above:
  - the brief to the analysts of names filed without review described the
    class the wrong name of this set belongs to ("a letter whose 'Re:' line
    names what the letter is about (a consultation, a review, another
    document)"), giving two development documents as its known instances;
  - the brief to the analysts of types and dates quoted two wrong names
    from this set's readings as examples of mechanisms ("Attachment B
    Invoice", real7-invoice-04; "OrdinanceResolution", real7-form-04). The
    analyst found neither in the development recordings and proposed no
    rule from them; both readings are unchanged;
  - seven of the twelve analysts and skeptics reported having seen this
    set's totals in the subject of the commit that added the report above.
- **The round's coordinator had read the result above before the round
  began**, including that real7-letter-07 was filed wrong and why. One of
  the round's rules (a record at the head of a letter's subject line is the
  letter's matter, not its type) holds exactly that kind of name. It was
  proposed from, and tested on, a document of the sixth set with the same
  shape (real6-letter-12), which is a development set. By the
  pre-registration's own rule, this set does not count as held out for that
  rule: its hold on real7-letter-07 is reported here, not claimed as a blind
  confirmation. The name it now waits under is still not the reference's:
  the letter is from Uisce Éireann, to the Commission.
- The other six differences are blind. The rule that names a notice "from"
  the one party the model called its issuer, where the engine said "for" by
  default, moved four notices: one right name became wrong, one stayed
  right, and the joining word became right on two names that are wrong or
  unscored for other reasons. The rule that names a document addressed to
  a class of recipients from the office it comes from made one name right.
  All of them were in review before and after.
- So on documents the rules were not written from, the round's net effect
  on names is nothing (one gained, one lost), where the development
  recordings it was written from show 33 more right of 1,224 readings
  (three of them after the reference accepted one party's name as its
  document spells it, a change made after seeing the result). The
  difference is what fitting to a set is, and is why this set is kept.


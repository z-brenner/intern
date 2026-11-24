# A whole-value token score for the review gate, November 2026

Status: measured on 2026-11-24 on development data only (real sets 1 to 9, all already
read). **No threshold is chosen here.** The score ships off by default
(`Engine::with_whole_value_confidence`, `intern-bench --confidence-score whole-value`), and the
plan is to fix a threshold before real set 10 is read, then confirm it on that set.

## The question

Today's gate holds a reading whose *least probable single token* in its date or party names is
below 0.7 (`TokenConfidence::min`). On InternBench, that token was usually a choice the name does
not depend on: which party the model writes first, how the JSON closes after "Corp.", or the
capitalisation of a word. On real set 9, the check held 13 of the 15 right names that strict held.

The **whole-value score** (`client::whole_value_confidence`) is the lowest, over the date and each
party, of the *product* of the probabilities of that value's tokens. At each token, any
alternatives the server listed that would have written the same value are added back:

- **Closing token:** an alternative that writes the same characters of the value and closes it
  in the same place (`.","` against `.",`).
- **Case and spacing:** an alternative that differs only in case or spacing (`art` against `ART`).

**Party order is not merged.** Merging it (the "T1 order" rule) let wrong names through on real
sets 1 and 5 in [`review-gate-2026-11`](../review-gate-2026-11/README.md), and it does worse
again here (below).

## Data

One traced 9B recording per set, the same ones as PR #91's verifier, plus real set 9. Every
reading was replayed through main's engine at strict:

| Set | Recording |
|---|---|
| real1 to real4 | candidate-bench run 42983156560 (`--no-repack`) |
| real5 to real7 | run 43302007759 (alpha.18) |
| real8 | run 43666145989 (alpha.19) |
| real9 | the PR #94 live run (SHA-256 `fe08419f…b9866b`) |

The gate only decides readings that strict would otherwise file. That pool is the readings ready
at strict with no token gate (`--min-token-confidence 0`): **133 readings** (93 right, 25 minor,
15 really wrong, on the [review-levels severity scale](../review-levels-2026-11/policy/severity.py)).
Today, at 0.7, the gate files 41 right, 6 minor and 2 really wrong of them.

## Results

**Thresholds chosen with each set held out.** For each set, the threshold was chosen on the other
eight: the one that files the most right names without filing more really wrong names than today's
0.7 filed on those eight. It was then applied to the held-out set:

| Score | Right | Minor | Really wrong |
|---|---:|---:|---:|
| Least token at 0.7 (today) | 41 | 6 | 2 |
| **Whole value, formatting merged** | **56** | **10** | **2** |
| Whole value, nothing merged | 49 | 9 | 2 |
| Least token, formatting merged | 48 | 9 | 3 |
| Whole value, formatting and order merged | 47 | 9 | 2 |

By set (whole value, formatting merged; threshold chosen without that set, against today at 0.7):

| Set | Threshold | Right / minor / wrong | Today |
|---|---:|---|---|
| real1 | 0.594 | 11 / 0 / 0 | 6 / 0 / 0 |
| real2 | 0.594 | 8 / 1 / 0 | 6 / 0 / 0 |
| real3 | 0.594 | 3 / 1 / 0 | 3 / 0 / 0 |
| real4 | 0.594 | 8 / 0 / 0 | 4 / 0 / 0 |
| real5 | 0.594 | 4 / 0 / 0 | 4 / 0 / 0 |
| real6 | 0.667 | 5 / 0 / 1 | 6 / 0 / 2 |
| real7 | 0.595 | 6 / 3 / 0 | 5 / 2 / 0 |
| real8 | 0.587 | 4 / 3 / 1 | 4 / 2 / 0 |
| real9 | 0.594 | 7 / 2 / 0 | 3 / 2 / 0 |

**Does it fail the way the November formatting rules did?** Not on these sets. On real1 and real5,
the two sets where the earlier rules let wrong names through, it files no really-wrong name (11
and 4 right). The two really-wrong names it files across all nine sets are `real6-form-06` (a
blank invoice template that today's gate files too, at 0.90) and `real8-notice-11` (0.649). Today
files `real6-form-06` and `real6-notice-10` instead.

**The same threshold on every set (in-sample), for scale.** At 0.594 the built engine files 58
right, 10 minor and 2 really wrong across the nine sets. Today at 0.7 files 41, 6 and 2. Both
figures come from end-to-end replays through this branch's `intern-bench`.

AUC (right against really wrong, within the pool): least token 0.729, whole value with
formatting merged 0.778.

## What this does not show

- **Small numbers.** The pool holds 15 really-wrong names, and both scores file 2. The
  difference between them is 15 right names; the risk side is unmeasured beyond "no worse on these
  sets". Minor names filed rise from 6 to 10.
- **All nine sets have been read.** Real sets 7 to 9 were held-out sets for earlier decisions
  and are development data now. The held-out thresholds above are also chosen on sets that shaped
  earlier rules. Only a fresh set can confirm a threshold.
- **Not a calibration.** A product is lower than its least factor, so the whole-value score's
  threshold sits lower (about 0.59 here against 0.7). The two numbers are not comparable, and
  setting `--min-token-confidence 0.7` with `--confidence-score whole-value` would hold *more* than
  today.
- **One mismatch with the analysis.** On 132 of 133 readings the engine's figures equal the
  analysis scripts' figures. On `real3-office-14` the scripts map the token trace onto the reply
  differently, and the engine's reading (0.616) is used above.
- **The app is unchanged.** The desktop app asks for one token alternative and never sets the
  option. Turning the score on would mean asking for two alternatives (`top_logprobs: 2`, which
  generates no token differently) and choosing a threshold.

## Reproducing

From the recordings at commit `3622d8be` (`bench-data/recordings/*.zip`) and real set 9's
recording, with the gold from `scripts/realdocs-eval/sample*.json` (minus each entry's
`source`) in `gold-realN.json`, and `sets.txt` listing `realN <recording dir>`:

    intern-bench run --corpus EMPTY --gold gold-realN.json --replay DIR/recording.json \
      --min-token-confidence 0 --allow-stale --output g0-realN.json
    python3 wv.py WORKDIR          # scores every variant from the traces -> wv-rows.json
    python3 curve.py WORKDIR/wv-rows.json
    python3 loso.py WORKDIR/wv-rows.json

and for the engine's own figures:
`intern-bench run ... --confidence-score whole-value --min-token-confidence X`.

# Review levels: a looser standard for renaming without a person, November 2026

Status: measured on 2026-11-15. The owner asked for a lower standard for automatic renaming
and descriptions: a name should wait for a person only when it is really wrong. A looser
level, **balanced**, was designed on the development documents, fixed, and then read once on
the two held-out sets. It did not meet the bar written down beforehand. It ships as a choice
in Settings; **strict**, today's standard, stays the default.

| | Strict (today) | Balanced |
|---|---|---|
| Development (408 documents): renamed without a person | 49 (12%) | 206 (50%) |
| ... of those, really wrong | 0 | 14 (6.8%) |
| Held-out (109 documents): renamed without a person | 7 (6%) | 45 (41%) |
| ... of those, really wrong | 0 | 7 (15.6%); 5 (11.1%) read by hand |
| The bar fixed in advance for the held-out sets | | no more than 10% |

In plain terms: on documents nobody had tuned anything on, balanced renames about six times
as many documents by itself, and about one name in seven of those is really wrong, mostly in
who the document is from or to. In development it looked like one in fifteen.

## What "ready" means today

A reading is ready only when validation raised no reason at all and the least probable token
of its date and party names is at least 0.8 (`tiers::REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE`,
decided in [`review-gate-2026-11`](../review-gate-2026-11/README.md) by a rule that put any
wrong name filed ahead of any right name filed). The quick model's readings are never ready.
The queue adds holds of its own: a near-duplicate, a learned spelling nobody has confirmed, a
name with no leading date. Nothing in the data has changed since that decision. The criterion
has: the owner now accepts minor errors and some wrong names in exchange for less review.

## A scale for how wrong a name is

- **right**: the bench's exact match to a gold filename.
- **really wrong**: no date in the name; a date that is not the gold's date or an acceptable
  one; a type whose head noun differs from every gold type ("Invoice" for an invoice dispute
  letter); a party in the name that the gold forbids for a reason other than being the person
  who signs or is addressed, or that the gold does not know (a near spelling or a class of
  addressee aside).
- **minor**: everything else: another accepted party, a joining word, a qualifier on the
  type, a person named for their organisation.

Applied by [`policy/severity.py`](policy/severity.py). The 159 not-right names of the
accurate model in development were read by hand against it, and an independent check then
read all 76 it called minor: it found one fault (a party the gold forbids passed when the
gold also listed it by role), repaired in the copy here. On a stricter scale that counts a
person named for the organisation as really wrong, every figure below is worse; it is given
beside.

## Development evidence

408 documents with a completed reading by the accurate model: InternBench (77) and real sets
1 to 6 (331), each counted once, from its newest recording, replayed through alpha.19's
engine. By the scale above: 249 right, 70 minor, 89 really wrong (36 of them have no date in
the name and can never be filed).

**Today's gate holds mostly good names.** It files 49, all right. Of the 359 it holds, 200
are right and 70 are minor.

**Each hold, on its own** (documents held; right / minor / really wrong as first scored):

| Hold | Held | r / m / w | What it mostly holds |
|---|---:|---|---|
| Token probability below 0.8 | 325 | 183 / 69 / 73 | good names; the only hold on 98 (79 / 10 / 9) |
| `TYPE_MISSING` | 120 | 71 / 27 / 22 | good names |
| `TYPE_INFERRED` | 98 | 59 / 21 / 18 | good names |
| `TYPE_UNSUPPORTED` | 121 | 55 / 33 / 33 | good names |
| `PARTY_UNSUPPORTED` | 162 | 67 / 50 / 45 | mixed |
| `DESCRIPTION_UNSUPPORTED` | 28 | 12 / 8 / 8 | good names |
| `DATE_UNSUPPORTED` | 36 | 3 / 1 / 32 | really wrong names |
| `DATE_MISSING` | 7 | 0 / 0 / 7 | really wrong names |
| `DESCRIPTION_INVALID` | 14 | 0 / 2 / 12 | really wrong names |
| `PARSER_WARNING` | 12 | 2 / 0 / 10 | really wrong names |
| A type of "Document" | 39 | 0 / 12 / 27 | names that say nothing |

Below 0.8 the token probability does not separate good from bad in development: among dated
names with no other reason, 2 of 19 are really wrong at 0.7 to 0.8, 1 of 24 at 0.6 to 0.7,
3 of 28 at 0.5 to 0.6, 3 of 18 at 0.4 to 0.5, 0 of 9 below 0.4. The model's own reported
confidence is not a signal at all: the reply has no such field and the engine writes 1.0.

**The levels considered** (repaired scale; the first scoring gave 12 and 15 for the two
looser rows, the independent check 13 and 16):

| Level | Renamed without a person | right / minor / really wrong | Really wrong share |
|---|---:|---|---:|
| Strict (today) | 49 (12%) | 49 / 0 / 0 | 0% |
| Balanced | 206 (50%) | 171 / 21 / 14 | 6.8% |
| Relaxed (balanced, and an uncertain party is filed when the token probability is at least 0.6) | 246 (60%) | 199 / 30 / 17 | 6.9% |
| The party check off | 318 (78%) | 232 / 59 / 27 as first scored | 8% |

On the stricter scale: 0, 21 (10.2%) and 26 (10.6%) for the first three.

Two warnings were already visible in development. Splitting the data into an older part
(InternBench, sets 1 to 4) and a newer one (sets 5 and 6), the party check looked nearly free
on the older part and cost 13 really wrong names of 73 filed on the newer. And among the 23
agreement-like documents, all synthetic, balanced files 11 more than strict and 3 of those 11
carry a wrong date, each a real date in the document that is not the document's own (a
counterparty's signature date, a master agreement's date).

Of the 14 really wrong names balanced files in development, 8 are such dates. No existing
check sees them.

## The rule, fixed before the held-out sets were read

[`prereg.md`](prereg.md), written before any level was applied to the held-out sets. The
levels and the bar came from the development analysis, made without opening either set. In
short: balanced becomes the default if no more than 10% of the names it files on the two
held-out sets together are really wrong; relaxed is offered if it is no more than 5 points
worse than balanced; a level that fails is not tuned. The number that decides is the larger
of the script's count and a count by hand.

The person who ran it knew how both sets read at today's gate, from the alpha.18 and
alpha.19 reports. The levels were not theirs.

## The held-out result

real7 (54 documents read; the alpha.18 live recording, run 43302007759, replayed through
alpha.19's engine) and real8 (55; the alpha.19 live recording, run 43666145989). Accurate
model. By the scale: 35 right, 35 minor, 39 really wrong (13 of them undated).

| Level | Renamed without a person | right / minor / really wrong | Really wrong share | Stricter scale |
|---|---:|---|---:|---:|
| Strict (today) | 7 of 109 (6%) | 6 / 1 / 0 | 0% | 0 of 7 |
| Balanced | 45 (41%) | 23 / 15 / 7 | **15.6%** | 10 of 45 (22%) |
| Relaxed | 65 (60%) | 30 / 24 / 11 | 16.9% | 17 of 65 (26%) |

| Set | Strict | Balanced | Relaxed |
|---|---|---|---|
| real7 (54) | 3: 3 / 0 / 0 | 26: 13 / 8 / 5 | 36: 17 / 13 / 6 |
| real8 (55) | 4: 3 / 1 / 0 | 19: 10 / 7 / 2 | 29: 13 / 11 / 5 |

- **Balanced fails the bar**: 7 of 45, 15.6%, against 10%. Reading the 22 not-right names by
  hand, two of the seven are a class of addressee, which the scale calls minor, so the hand
  count is 5 of 45, 11.1%. It fails either way. One more name swaps
  sender and recipient, which the scale counts as minor and a filing clerk would not.
- **What the seven are**: five name a party that is not one (an address, a generic kind of
  sender, an office holder the gold does not know, two classes of addressee), one gives a
  title that misdescribes the document, one puts a date on a document that has none of its own. Five of the seven
  carry no review reason at all: the token probability was their only hold, and it was low
  (0.28 to 0.46 for six of them, 0.67 for the seventh).
- **What balanced still holds**: 64 documents, of which 32 are really wrong, 20 minor and 12
  right. Of the 26 really wrong names that have a date, it holds 19. `PARTY_UNSUPPORTED` is
  on 55 of the 64 and is the only hold on 24 good names.
- **Relaxed** is within 5 points of balanced and worse on every count. With balanced failing
  it is not offered.

The difference from development is the kind of error. In development the looser level's
wrong names were mostly dates; here they are mostly parties the engine's party check did not
question. On these two sets, unlike in development, a low token probability did mark most of
them. That is an observation made after the fact on the sets it would have to be tested on:
it is not built into a level, and a floor on the token probability at balanced would need a
set nobody has read under these levels.

## What was decided

- The setting ships with two levels. **Strict is the default.** Balanced is there for a
  person who chooses it, described in Settings in words, without these numbers.
- The level is not tuned to the held-out sets, and relaxed is not built.
- The quick model's readings wait at both levels (in development it would file 19 really
  wrong names of 159 at balanced).
- Both held-out sets have now been read under these levels. Any later change to a level, and
  any new level, has to be reported as made with both in view, and confirmed on a new set.

What balanced is, in engine terms: a reading is ready when it has a date; every reason on it
is one of `TYPE_MISSING`, `TYPE_UNSUPPORTED`, `TYPE_INFERRED`, `DESCRIPTION_UNSUPPORTED` and
`LOW_CONFIDENCE` (the token probability); it has a document type (a name that would say
"Document" waits); and its title did not stand in for a type the document did not support
(`TYPE_UNSUPPORTED` with `TYPE_INFERRED` and without `TYPE_MISSING`). Every date reason,
every party reason, `DESCRIPTION_INVALID`, `PARSER_WARNING`, the quick model's hold and the
queue's own holds keep a document waiting at both levels.

## What this does not tell you

- **Almost nothing here is corporate paper.** The documents are mostly public letters,
  notices, invoices, forms and minutes. Development holds 16 contracts, all synthetic, and no
  stockholder agreement, policy, org chart or equity record
  ([`docs/research-2026-11.md`](../../../docs/research-2026-11.md), section 4); the held-out
  sets were gathered the same way. How either level behaves on an in-house file is not known.
- **Small counts.** 7 of 45 has a 95% interval of about 8% to 29%. The two sets differ (5 of
  26 and 2 of 19). One run each; a reading repeated on the same document gave a different
  name for 56 of 339 development documents.
- **Descriptions** are composed from the same facts as the name and filed with it; there is
  no separate gate. They were scored only for repeating a wrong party: of the 12 really wrong
  names balanced files in development as first scored, that scoring called 10 descriptions
  right, and at least 5 of those 10 state something false (the wrong date again, or the
  sides the wrong way round).
- **The queue's side** (learned spellings, near-duplicates, name clashes, a level changed
  with documents waiting) is covered by tests, not by this measurement: the bench has no
  queue.
- A date that comes from a spreadsheet's `TODAY()` cannot be seen in a replay of a recording
  made before the worker reported formulas; one development document is counted as held on
  the strength of the code, not of a run.

## Files

- [`prereg.md`](prereg.md): the rule, as written before the run.
- [`policy/`](policy/): `severity.py` (the scale), `levels.py` (the levels; `dev` and
  `heldout`), `load.py` (one record per document from the newest recording),
  `SHA256SUMS` (the scripts as run; written after the run, not with the pre-registration).
  They read intern-bench reports replayed from the live recordings; neither the recordings
  nor the gold for the real sets are in the repository.

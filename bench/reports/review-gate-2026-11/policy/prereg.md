# Pre-registration: the review gate, decided on the sixth real-document set

Written 2026-11-11T07:35:25Z, before any model has read real6 (its sample file is still being
assembled; no live run of it exists). Policy code as of this moment: policies.py, trace.py, features.py
(SHA-256 in prereg.sha256).

## What is known going in (development data, seen)
- Current engine, 9B, gate 0.7, live recordings of 2026-11-09: real1-4 file 11 names, all right;
  real5 (then unseen by any rule) filed 6, 4 of them wrong.
- Fresh traced recording of 2026-11-11 (same prompt, other runners): real1 files 2/0 (right/wrong),
  real5 files 2/2. On real5, 14 names carry no validation reason; 10 of those are wrong.
- Token-trace candidates (ignore a name's first token when the runner-up begins another reply party; ignore
  tokens that only close a name; read only the parties the filename uses) each released wrong names on
  real1 or real5 at 0.7. REJECTED before real6; not candidates.

## Candidates (exact predicates: policies.py POLICIES)
- today@0.70: no validation reason and least date/party token probability >= 0.70 (what ships).
- today@0.80: the same at 0.80.
- R1@0.70 / R1@0.80: as today, but PARTY_UNSUPPORTED alone does not hold a one-party "from" name, and a
  name with a party but no joining word is never filed.
- C1@0.70 / C1@0.80: as today, but a validation-clean two-party "between" notice is filed whatever its
  token probability.

## Decision rule
1. A candidate is eligible only if, on the traced development recordings (real1-5 and InternBench, 9B),
   it files no more wrong names than today@0.70.
2. On real6 (9B, live, replayed through the release-candidate engine): choose the eligible candidate with
   the fewest wrong names filed; ties go to the most right names filed; if today@0.70 is among the tied
   best, nothing changes.
3. Whatever is chosen, the release notes report real6's right/wrong filed for today@0.70 and for the
   choice, with counts, and say that one set of about 57 documents cannot certify a rate.

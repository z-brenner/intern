# Independent review of bounded summary fidelity change

Reviewed the uncommitted `facts.rs` / `compose.rs` changes against base `6173151e0309f3f31b927e883175e0b607dfe335`, including all four public and sixteen released heldout tests. No production edits, model runs, or compilation were performed by this reviewer.

## Verdict

No correctness, security, performance, or regression blocker found in the proposed implementation. Approval is limited to this bounded change, subject to completing the existing regression/replay gate. The old notice-description expectation has been updated as described below; production code remains unchanged from the freeze. This review does not authorize merging, model-default changes, or rollout.

Keeping the complete field value avoids severing quantities at eight words. Numeric subjects bypass the six-word shortening step and can still be dropped as a whole by the fixed description-budget loop. The added work is linear in an already bounded context unit, with no unbounded iteration or new process/network behavior.

Directional secondary parties now require a supported role or the existing explicit billing field. Removing the sole-other fallback prevents a coissuer, observer, or referenced organization from becoming a sender/recipient. Explicit invoice billing direction and letter sender/addressee controls remain passing.

## Independent verification

- All four source hashes and the candidate test-binary SHA-256 matched `candidate-freeze.json` before execution.
- Frozen candidate binary, `--test-threads=1 fidelity`: **20 passed, 0 failed**, including all sixteen released heldouts.
- Prebuilt baseline binary, `--test-threads=1 heldout_fidelity`: **4 passed, 12 failed**. Failures independently reproduced incomplete pressure/area/flow units, truncated temperature ranges, Unicode-digit unit loss, false sender/recipient assertions, and use of an unsupported recipient role.
- Inspected all new tests and the surrounding validation, role selection, field extraction, and description budget code. Tests assert source-derived units/direction and include positive controls for true recipients. Long numeric-subject controls deliberately accept complete omission.

## Existing test expectation

`a_notice_whose_reply_named_no_addressee_is_to_its_to_field` previously expected Northstar Lantern Works LLC to be described as sender even though its supplied validated role is `Other`. The candidate conservatively omits that sender while retaining the explicit `To: John Smith` addressee and filename parties. The public expected description and explanatory comment were updated after the freeze; this test-only adjustment is consistent with the fix. Restoring sole-other sender inference would restore the demonstrated failures. Evidence-backed sender-company recovery can be separate work.

## Limits

The new atomicity guard recognizes Unicode numeric characters; it does not newly solve quantities spelled entirely in words. Conservative omission can reduce summary detail. This small synthetic suite establishes the targeted behaviors, not general semantic accuracy. Full existing regression and replay results are owned by the parent task and were not duplicated here.

## Neutral naming refinement (candidate v2)

The 77-case corpus replay exposed completeness loss in two v1 descriptions. The refinement appends `, also naming NAME` when a sole validated other party remains and no supported directional secondary party or explicit billed recipient was emitted. It excludes all primary relation names and obeys the existing `Part::SecondParty` budget. Keeping even a supported coissuer is sound here: the clause asserts only source presence, without labeling that party sender or recipient. No correctness/security/performance blocker found in this refinement.

The public coissuer regression was strengthened to require retaining the useful name, and `neutral-red.log` records its failure before the refinement. The notice expectation now preserves Northstar neutrally while keeping John Smith as the true addressee. Reviewed `neutral-green.log`, which records all 384 library tests passing. Independently reran the refined binary's focused `fidelity` filter: all 20 tests passed.

V1's sixteen heldouts remain its first unseen evaluation. V2's rerun is regression evidence after a corpus-informed refinement, not a new unseen evaluation or a general accuracy estimate. Approval remains qualified on final full-regression/replay completion, with no merge/model rollout authorization.

Reviewed v2 source SHA-256 values:

- `facts.rs`: `3192d88ce08b8a702d712450faf28cc45524ad7b3c69852b534786265a95cc64`
- `compose.rs`: `5554090a1cffa4eeb824c95a405d91419da2100563166ecbafa009edf839c9c1`
- `public.rs`: `1a5655874f1d7d37ff31223c78baf31635787a3d02e9f0427570fcf504791fbe`
- `heldout.rs`: `18edd9c460c0085c4c266756efbff1177cc640a68e84102f4de08384abe7cce1`

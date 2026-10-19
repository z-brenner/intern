# Bonsai2 independent semantic audit

All 12 retained held-out results were reviewed. The automated totals remain **8/12 strict filenames, 3 review-required outputs, 1 strict unsafe-ready output**; this audit does not rewrite frozen scores. Median measured document time is 22.493 seconds; nearest-rank p95 is 143.283 seconds. This is a single Windows CPU pass, not a model comparison or production qualification.

For each case, the scored raw reply was selected by matching `request.native_request_sha256` to the recording exchange's `prompt_sha256`. There is exactly one scored match per case; four extra wire captures are unscored warmups. Review used source pages, actual forwarded evidence, raw replies, final reports and frozen gold. It did not rerender every original page or rerun OCR.

## Main findings

- **No accepted wrong date value or invented party was found in the final filenames.** The four strict misses are omissions: annual-report date, remittance type, meeting issuer, and amendment date. The annual report, remittance and amendment were flagged for review. The meeting was left ready.
- **The assignment's $314,000 deposit is supported.** Page 1 §1 explicitly transfers the assignor's interest in the security deposit held by the landlord. “Transferred security deposit” would be clearer than “deposit,” but the amount is not fabricated and the description does not claim a new payment.
- **The lease measurement is malformed.** Source/OCR retains “14,860 rentable square feet”; the final description ends the phrase at “square.” The raw model only supplied “Office Lease” as subject, so this is an engine inference/composition defect, not model-generated measurement hallucination or OCR unit loss.
- **Date-role labels need separate scrutiny.** The promissory note retains “issuance” despite an explicit Effective Date. The account statement changes the model's correct “issuance” to “invoice.” The separation agreement's April 6 as-of date is labeled “effective,” but §6 makes effectiveness conditional on the eighth day after an undated signature without revocation. Frozen gold also labels April 6 effective, masking that semantic issue. The lease's April 22 as-of date must not be conflated with its June 1 commencement; its effective label is more definite than the source establishes.
- **Financial summaries can be supported yet unhelpful.** The capital-call description correctly labels $5 million as capital commitment, but omits the actual $210,000 call and its limited partner. “No. RGP3-0148” is the investor ID, whereas the notice is No. 7.
- **The party-role metric is narrower than its name suggests.** `score.rs:647` checks relation/name membership, including whether both holders occur for “between.” Its 10/10 eligible result does not establish the correctness of raw issuer/recipient/borrower/etc. labels.

## All 12 cases

| Case | Final naming/date/parties | Description and attribution |
|---|---|---|
| Promissory note | Correct date value and borrower/lender; engine removes the model's signatory-as-borrower. Effective date remains mislabeled issuance. | Correct $250,000 principal. “For Copper Flats” treats a city as subject and can suggest an unsupported purpose. Interest and maturity omitted. Raw subject/role mistakes partly survive. |
| Capital call | Correct issuer/date; engine repairs execution→notice and removes GP/CFO candidates. | Supported commitment, but misses $210,000 call and recipient; investor ID generically labeled “no.” Engine fact selection adds these details. |
| Assignment | Correct principals and June 1 effective date; engine repairs execution→effective. | Underlying lease date and $314,000 security deposit supported. Generic raw roles and minor punctuation reduce precision, not factual identity. |
| Annual report | Missing date; review. Model invents “8-PAGE DOCUMENT” and January 1 from the report year despite March 12/December 31 evidence in the prompt. Engine rejects them. | Circular “Annual Report ... covering 2025 Annual Report”; no financial content. Safe omission after blocking raw inventions. |
| Separation agreement | Correct names/as-of date/full title; effective-date metadata conflicts with conditional §6. | $48,750 salary continuation supported, but conditions omitted. Engine introduces effective label; frozen gold shares this limitation. |
| Distribution agreement, 10-page scan | Correct November 10 effective date and companies. Model calls supplier “contractor” and distributor “recipient”; final between relation hides that weakness. | Title-and-names only; products and territory omitted. OCR preserves relevant facts. |
| Office lease, 25-page scan | Correct companies and allowable April 22 as-of date; as-of/effective/commencement distinction matters. | Suite/deposit supported; missing “feet” is an engine composition defect. OCR contains the full unit. |
| Noisy statement | Correct issuer, statement date and $809.95 balance. Engine recovers omitted issuer but regresses date role to invoice. | Account holder omitted. OCR punctuation noise does not corrupt final amount. Ready differs from frozen expected-review, without demonstrating a false final amount. |
| Low-DPI remittance | Generic Document, correct payment date and sender; review. Model chooses department-like ACCOUNT PAYABLE despite readable remittance header, and adds routing bank as issuer. | Correct $47,016.96 net payment. Engine drops bank but also valid payee. Missing type is substantive, not a capitalization mismatch. |
| Right-aligned invoice | Correct issuer/date; engine changes issuance→invoice and keeps customer in description. | Invoice ID and $3,006.42 total supported. “For Test” is a table-header subject, not an informative service summary. |
| Interleaved meeting notice | Correct date/type, issuer absent, still ready. Issuer appears clearly in source and prompt. | Truthful but ambiguous “regarding merger plan.” Strict unsafe-ready is an unflagged identifying omission, not a fabricated institution/date. |
| Mixed-signature amendment | Correct type/parties, no date, review. Model picks the underlying 2024 agreement date; engine blocks it. Actual prompt includes both 2026 signature dates. | Generic underlying-agreement subject; modules, $64,800/year fee and service changes omitted. No evidence that OCR lost the governing August 21 date. |

## Limits and protocol

These results support a distinction between **grounded but underspecific outputs**, **raw mistakes successfully filtered**, and **accepted semantic/composition defects**. Automated `description_factual=true` is not sufficient proof of an adequate summary. No prompt, model setting, gold file, source code or production behavior was changed.

The separate Qwen vendor-profile correction is recorded as a pre-inference change in `artifacts/qwen-vendor-profile-correction.json`, with new nqvcal IDs and old plans retained. It is not a Bonsai2 held-out adjustment and provides no Qwen performance result. This audit relies on that separate record for its source/timing claim.

Detailed per-case evidence, raw-versus-engine attribution and input hashes are in [bonsai2-independent-semantic-audit.json](bonsai2-independent-semantic-audit.json).

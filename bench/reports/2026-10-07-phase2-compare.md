# InternBench comparison

- **Before:** replay run 2026-10-07T07:58:32Z at `9e1f5270cad525dd03165a515fe8ac7460c35730-dirty` (72 documents) on Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS) · timings recorded (recording `1366aa8f4354`, made 2026-10-07T04:54:20Z), not measured
- **After:** replay run 2026-10-07T09:09:44Z at `7278aac9e07d1f003797304393456e6bf251496b-dirty` (72 documents) on Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS) · timings recorded (recording `8f84a28c15e2`, made 2026-10-07T05:45:28Z), not measured
- **Compared:** every score, rate and count over the 72 documents both runs scored (completed, or failed and scored as a miss), each score over the documents that have it in both runs; latency over the 71 both completed.

## Scores

| Score | Before | After | Change |
| --- | ---: | ---: | ---: |
| `filename_correct` | 19/72 (26.4%) | 24/72 (33.3%) | +6.9 pts better |
| `type_correct` | 55/72 (76.4%) | 57/72 (79.2%) | +2.8 pts better |
| `date_correct` | 57/72 (79.2%) | 61/72 (84.7%) | +5.5 pts better |
| `date_role_correct` | 36/53 (67.9%) | 37/53 (69.8%) | +1.9 pts better |
| `parties_correct` | 47/72 (65.3%) | 50/72 (69.4%) | +4.2 pts better |
| `relation_correct` | 28/70 (40.0%) | 33/70 (47.1%) | +7.1 pts better |
| `party_role_correct` | 29/35 (82.9%) | 27/35 (77.1%) | -5.7 pts worse |
| `readiness_match` | 32/52 (61.5%) | 35/52 (67.3%) | +5.8 pts better |
| `description_complete` | 5/72 (6.9%) | 9/72 (12.5%) | +5.6 pts better |
| `description_factual` | 66/71 (93.0%) | 70/71 (98.6%) | +5.6 pts better |
| `description_specific` | 71/72 (98.6%) | 72/72 (100.0%) | +1.4 pts better |
| `unsafe_ready` | 26/72 (36.1%) | 29/72 (40.3%) | +4.2 pts worse |
| `date_forbidden` | 11/72 (15.3%) | 11/72 (15.3%) | 0 pts |
| `party_forbidden` | 7/72 (9.7%) | 8/72 (11.1%) | +1.4 pts worse |
| `needless_review` | 1/48 (2.1%) | 2/48 (4.2%) | +2.1 pts worse |
| `date_exact` | 55/72 (76.4%) | 58/72 (80.6%) | +4.2 pts better |
| `date_present` | 68/72 (94.4%) | 72/72 (100.0%) | +5.6 pts better |
| `type_present` | 64/72 (88.9%) | 66/72 (91.7%) | +2.8 pts better |
| `description_completeness` (mean) | 47.7% | 53.0% | +5.3 pts better |
| `description_specificity` (mean) | 96.8% | 99.5% | +2.8 pts better |
| `evidence_recall` (mean) | 50.0% | 53.0% | +3 pts better |
| `digest_recall` (mean) | 98.2% | 100.0% | +1.9 pts better |
| `prompt_recall` (mean) | 98.2% | 100.0% | +1.9 pts better |
| `kv_accuracy` (mean) | 57.6% | 99.0% | +41.3 pts better |
| `ocr_cer` (mean) | 19.1% | 5.0% | -14.1 pts better |
| `ocr_cer_ci` (mean) | 19.1% | 4.9% | -14.1 pts better |
| `ocr_date_accuracy` (mean) | 87.2% | 100.0% | +12.8 pts better |
| `ocr_identifier_accuracy` (mean) | 90.6% | 100.0% | +9.4 pts better |
| `ocr_mean_confidence` (mean) | 91.8 | 97.5 | +5.7 better |
| `ocr_name_accuracy` (mean) | 94.4% | 100.0% | +5.6 pts better |
| `ocr_wer` (mean) | 23.8% | 5.4% | -18.5 pts better |
| `reading_order_accuracy` (mean) | 64.9% | 100.0% | +35.1 pts better |
| `table_cell_recall` (mean) | 91.5% | 99.4% | +7.9 pts better |
| `table_row_accuracy` (mean) | 65.1% | 96.8% | +31.7 pts better |

## Safety counts

| Count | Before | After | Change |
| --- | ---: | ---: | ---: |
| unsafe_ready | 26 | 29 | +3 worse |
| trap_dates | 11 | 11 | 0 |
| forbidden_parties | 7 | 8 | +1 worse |
| spurious_parties | 26 | 24 | -2 better |
| forbidden_descriptions | 0 | 0 | 0 |
| unsupported_claims | 5 | 1 | -4 better |
| claims | 223 | 240 | +17 |
| review_rate | 38.0% | 32.4% | -5.6 pts |
| unsupported_fact_rate | 2.2% | 0.4% | -1.8 pts better |

## Broken: 33 score(s) in 19 document(s)

- **account-statement**: unsafe_ready
- **prior-authorization**: parties_correct
- **capital-call-notice**: readiness_match
- **lease-two-column**: date_exact
- **declarations-interleaved**: unsafe_ready
- **vendor-registration-form**: party_forbidden
- **credit-agreement-50p**: needless_review
- **scan-fax-two-frames**: readiness_match
- **scan-patient-intake-form**: type_correct, type_present
- **meeting-notice-interleaved**: date_correct, date_exact, date_forbidden
- **rate-confirmation-rotated**: unsafe_ready
- **price-list-unruled**: unsafe_ready
- **invoice-label-above**: filename_correct, parties_correct, party_forbidden, party_role_correct
- **invoice-right-aligned**: needless_review, readiness_match
- **benefits-change-checkbox-form**: date_forbidden, unsafe_ready
- **loss-notice-boxed-fields**: party_role_correct
- **scan-remittance-advice-120dpi**: date_correct, date_exact, date_forbidden
- **mixed-signature-region**: date_forbidden, parties_correct, party_forbidden, party_role_correct, unsafe_ready
- **scan-bill-of-lading**: relation_correct

## Fixed: 67 score(s) in 27 document(s)

- **invoice-date-in-table**: relation_correct
- **account-statement**: readiness_match, type_correct, type_present
- **sow-under-msa-5p**: description_complete
- **notice-of-default**: date_correct, date_exact, date_forbidden
- **explanation-of-benefits**: parties_correct
- **capital-call-notice**: unsafe_ready
- **engagement-letter**: relation_correct
- **declarations-interleaved**: description_factual, parties_correct, party_forbidden, readiness_match
- **change-order-form**: filename_correct, relation_correct, unsafe_ready
- **aircraft-maintenance-log**: parties_correct, party_forbidden
- **data-processing-agreement-10p**: date_correct, date_exact, date_present, description_specific, filename_correct, parties_correct, party_role_correct, readiness_match, relation_correct, type_correct, type_present
- **credit-agreement-50p**: filename_correct, relation_correct
- **scan-skewed-notice**: parties_correct
- **scan-noisy-statement**: filename_correct, relation_correct
- **scan-faint-letter**: description_complete, description_factual
- **ocr-corrupted-invoice**: date_correct, date_exact, date_present, description_factual, filename_correct
- **scan-fax-two-frames**: filename_correct, type_correct, type_present
- **scan-patient-intake-form**: date_correct, date_exact, date_forbidden, unsafe_ready
- **meeting-notice-interleaved**: description_complete
- **meeting-notice-reversed**: description_complete
- **rate-confirmation-rotated**: date_correct, date_exact, date_forbidden, readiness_match
- **price-list-unruled**: description_factual, readiness_match
- **invoice-label-above**: date_role_correct, needless_review
- **benefits-change-checkbox-form**: date_present, readiness_match
- **loss-notice-boxed-fields**: parties_correct
- **mixed-signature-region**: date_present
- **scan-certificate-of-insurance**: date_correct, date_exact, date_forbidden

## Worse past the extract-only tolerance: 8 score(s)

| Document | Score | Before | After |
| --- | --- | ---: | ---: |
| scan-certificate-of-insurance | `ocr_char_distance` | 188 | 491 |
| scan-certificate-of-insurance | `ocr_char_distance_ci` | 188 | 477 |
| scan-certificate-of-insurance | `ocr_word_distance` | 31 | 78 |
| scan-certificate-of-insurance | `table_cell_recall` | 96.7% | 93.3% |
| scan-certificate-of-insurance | `table_row_accuracy` | 80.0% | 60.0% |
| scan-bill-of-lading | `ocr_char_distance` | 117 | 477 |
| scan-bill-of-lading | `ocr_char_distance_ci` | 115 | 466 |
| scan-bill-of-lading | `ocr_word_distance` | 26 | 70 |

## Better past the extract-only tolerance: 79 score(s)

| Document | Score | Before | After |
| --- | --- | ---: | ---: |
| invoice-date-in-table | `kv_accuracy` | 42.9% | 100.0% |
| invoice-date-in-table | `table_cell_recall` | 97.4% | 100.0% |
| invoice-date-in-table | `table_row_accuracy` | 85.7% | 100.0% |
| lease-two-column | `table_cell_recall` | 75.0% | 100.0% |
| lease-two-column | `table_row_accuracy` | 50.0% | 100.0% |
| declarations-interleaved | `kv_accuracy` | 83.3% | 100.0% |
| declarations-interleaved | `reading_order_accuracy` | 62.5% | 100.0% |
| scan-clean-lease-2p | `ocr_char_distance` | 18 | 8 |
| scan-clean-lease-2p | `ocr_char_distance_ci` | 18 | 8 |
| scan-clean-lease-2p | `ocr_word_distance` | 5 | 2 |
| scan-mixed-amendment | `ocr_word_distance` | 4 | 2 |
| scan-skewed-notice | `ocr_char_distance` | 816 | 0 |
| scan-skewed-notice | `ocr_char_distance_ci` | 816 | 0 |
| scan-skewed-notice | `ocr_date_accuracy` | 33.3% | 100.0% |
| scan-skewed-notice | `ocr_word_distance` | 144 | 0 |
| scan-low-res-receipt | `ocr_char_distance` | 100 | 79 |
| scan-low-res-receipt | `ocr_char_distance_ci` | 100 | 79 |
| scan-low-res-receipt | `ocr_word_distance` | 23 | 3 |
| scan-noisy-statement | `ocr_char_distance` | 358 | 9 |
| scan-noisy-statement | `ocr_char_distance_ci` | 358 | 9 |
| scan-noisy-statement | `ocr_word_distance` | 138 | 13 |
| scan-faint-letter | `ocr_char_distance` | 163 | 5 |
| scan-faint-letter | `ocr_char_distance_ci` | 163 | 5 |
| scan-faint-letter | `ocr_word_distance` | 32 | 1 |
| scan-agreement-10p | `ocr_char_distance` | 18 | 11 |
| scan-agreement-10p | `ocr_char_distance_ci` | 18 | 11 |
| scan-agreement-10p | `ocr_identifier_accuracy` | 50.0% | 100.0% |
| scan-agreement-10p | `ocr_word_distance` | 6 | 3 |
| scan-lease-25p | `ocr_char_distance` | 30 | 10 |
| scan-lease-25p | `ocr_char_distance_ci` | 30 | 10 |
| scan-lease-25p | `ocr_word_distance` | 8 | 4 |
| scan-fax-two-frames | `ocr_char_distance` | 817 | 37 |
| scan-fax-two-frames | `ocr_char_distance_ci` | 817 | 37 |
| scan-fax-two-frames | `ocr_date_accuracy` | 50.0% | 100.0% |
| scan-fax-two-frames | `ocr_identifier_accuracy` | 0.0% | 100.0% |
| scan-fax-two-frames | `ocr_word_distance` | 119 | 3 |
| scan-patient-intake-form | `kv_accuracy` | 0.0% | 100.0% |
| scan-patient-intake-form | `ocr_char_distance` | 9 | 5 |
| scan-patient-intake-form | `ocr_char_distance_ci` | 9 | 5 |
| meeting-notice-interleaved | `reading_order_accuracy` | 27.3% | 100.0% |
| meeting-notice-reversed | `reading_order_accuracy` | 9.1% | 100.0% |
| rate-confirmation-rotated | `digest_recall` | 50.0% | 100.0% |
| rate-confirmation-rotated | `kv_accuracy` | 80.0% | 100.0% |
| rate-confirmation-rotated | `table_cell_recall` | 60.7% | 100.0% |
| rate-confirmation-rotated | `table_row_accuracy` | 0.0% | 100.0% |
| invoice-label-above | `kv_accuracy` | 0.0% | 100.0% |
| invoice-boxed-grid | `kv_accuracy` | 9.1% | 100.0% |
| benefits-change-checkbox-form | `kv_accuracy` | 0.0% | 100.0% |
| benefits-change-checkbox-form | `table_cell_recall` | 77.3% | 100.0% |
| benefits-change-checkbox-form | `table_row_accuracy` | 70.6% | 100.0% |
| loss-notice-boxed-fields | `kv_accuracy` | 0.0% | 100.0% |
| scan-rotated-page-in-pdf | `ocr_char_distance` | 329 | 1 |
| scan-rotated-page-in-pdf | `ocr_char_distance_ci` | 325 | 1 |
| scan-rotated-page-in-pdf | `ocr_word_distance` | 62 | 1 |
| scan-rotated-page-in-pdf | `table_cell_recall` | 86.1% | 100.0% |
| scan-rotated-page-in-pdf | `table_row_accuracy` | 0.0% | 100.0% |
| scan-remittance-advice-120dpi | `kv_accuracy` | 0.0% | 100.0% |
| scan-remittance-advice-120dpi | `ocr_char_distance` | 416 | 93 |
| scan-remittance-advice-120dpi | `ocr_char_distance_ci` | 415 | 92 |
| scan-remittance-advice-120dpi | `ocr_word_distance` | 60 | 16 |
| scan-remittance-advice-120dpi | `table_row_accuracy` | 0.0% | 85.7% |
| scan-mixed-middle-page | `ocr_char_distance` | 227 | 18 |
| scan-mixed-middle-page | `ocr_char_distance_ci` | 226 | 18 |
| scan-mixed-middle-page | `ocr_word_distance` | 40 | 3 |
| scan-mixed-middle-page | `table_cell_recall` | 91.7% | 100.0% |
| scan-mixed-middle-page | `table_row_accuracy` | 0.0% | 100.0% |
| mixed-signature-region | `digest_recall` | 66.7% | 100.0% |
| mixed-signature-region | `ocr_char_distance` | 206 | 1 |
| mixed-signature-region | `ocr_char_distance_ci` | 206 | 1 |
| mixed-signature-region | `ocr_date_accuracy` | 0.0% | 100.0% |
| mixed-signature-region | `ocr_name_accuracy` | 0.0% | 100.0% |
| mixed-signature-region | `ocr_word_distance` | 30 | 1 |
| mixed-signature-region | `reading_order_accuracy` | 20.0% | 100.0% |
| scan-certificate-of-insurance | `digest_recall` | 50.0% | 100.0% |
| scan-certificate-of-insurance | `kv_accuracy` | 0.0% | 80.0% |
| scan-certificate-of-insurance | `ocr_date_accuracy` | 85.7% | 100.0% |
| scan-bill-of-lading | `kv_accuracy` | 80.0% | 100.0% |
| scan-bill-of-lading | `table_cell_recall` | 74.2% | 100.0% |
| scan-bill-of-lading | `table_row_accuracy` | 20.0% | 100.0% |

## Status changes

- **data-processing-agreement-10p**: model_failed → completed

## Filenames that changed (29)

- **invoice-date-in-table**: `2026-03-04 Invoice between Quillon Ridge Bakery, Inc and Halvorsen Fixture Works LLC.pdf` → `2026-03-04 Invoice from Quillon Ridge Bakery, Inc.pdf`
- **account-statement**: `2026-03-01 Document - Kingsfold Community Bank.pdf` → `2026-03-01 Statement - Kingsfold Community Bank.pdf`
- **notice-of-default**: `2025-07-01 Notice of Default and Reservation of Rights with Basalt Commercial Credit Corp.pdf` → `2025-10-14 Notice of Default and Reservation of Rights - Basalt Commercial Credit Corp.pdf`
- **prior-authorization**: `2026-03-09 Notice of Prior Authorization Approval for Silverlode Health Partners.pdf` → `2026-03-09 Notice of Prior Authorization Approval - Silverlode Health Partners.pdf`
- **capital-call-notice**: `2026-02-06 Capital Call Notice for Ravensmoor Growth Partners III, L.P.pdf` → `2026-02-06 CAPITAL CALL NOTICE No for Ravensmoor Growth Partners III, L.P.pdf`
- **engagement-letter**: `2026-01-09 Letter with Halloran Ostrowski CPAs LLP.pdf` → `2026-01-09 Letter between Silverbirch Ceramics Inc and Halloran Ostrowski CPAs LLP.pdf`
- **letterhead-letter**: `2026-03-19 Document between Moonrake Paper & Packaging Co and Briarport Coffee Roasters LLC.pdf` → `2026-03-19 Document - Moonrake Paper & Packaging Co.pdf`
- **lease-two-column**: `2026-05-18 Retail Lease between Oakhaven Retail Properties LLC and Sorrel & Thistle Tea House LLC.pdf` → `2026-07-01 Retail Lease between Oakhaven Retail Properties LLC and Sorrel & Thistle Tea House LLC.pdf`
- **declarations-interleaved**: `2026-06-18 Commercial Package Policy Declarations - Kingsfold Insurance Agency, Inc.pdf` → `2026-06-18 Commercial Package Policy Declarations - Northfell Mutual Insurance Company.pdf`
- **vendor-registration-form**: `2026-04-22 Vendor Registration Form.pdf` → `2026-04-22 Vendor Registration Form - Emberglow Coatings Ltd.pdf`
- **change-order-form**: `2026-05-07 Change Order - Briarport Library District.pdf` → `2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf`
- **aircraft-maintenance-log**: `2026-08-10 Maintenance Record between Halden Aero Works and HA-180.pdf` → `2026-08-10 Maintenance Record - Highmeadow Air Charter LLC.pdf`
- **credit-agreement-50p**: `2026-06-12 Credit Agreement - Marrowfield Packaging Holdings, Inc.pdf` → `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`
- **scan-skewed-notice**: `2026-08-03 Notice of Nonrenewal of Lease - Harrowgate Apartments LLC.tiff` → `2026-08-03 Notice of Nonrenewal of Lease for Harrowgate Apartments LLC.tiff`
- **scan-noisy-statement**: `2026-08-31 Statement of Account - Copper Flats Dental Group PLLC.pdf` → `2026-08-31 Statement of Account for Copper Flats Dental Group PLLC.pdf`
- **ocr-corrupted-invoice**: `Invoice from Wexcombe Millwork Co.pdf` → `2026-03-10 Invoice from Wexcombe Millwork Co.pdf`
- **scan-fax-two-frames**: `2026-08-12 Document to Highmeadow Air Charter LLC.tiff` → `2026-08-12 Quotation to Highmeadow Air Charter LLC.tiff`
- **scan-patient-intake-form**: `1990-11-23 New Patient Intake Form - Ione Kowalczyk.png` → `2026-08-27 Document - Calder Way Family Medicine.png`
- **meeting-notice-interleaved**: `2026-08-03 Notice of Special Meeting of Members - Larchmont Valley Federal Credit Union.pdf` → `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf`
- **meeting-notice-reversed**: `2026-09-11 Notice of Special Meeting of Members - Larchmont Valley Federal Credit Union.pdf` → `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf`
- **rate-confirmation-rotated**: `2026-06-10 Confirmation between Copperline Freight Brokerage LLC and Halden Ridge Trucking Inc.pdf` → `2026-06-08 Invoice between Copperline Freight Brokerage LLC and Halden Ridge Trucking Inc.pdf`
- **invoice-label-above**: `2026-05-11 Invoice from Quarrystone Signs & Graphics.pdf` → `2026-05-11 Invoice from Bellhaven Physical Therapy PLLC.pdf`
- **invoice-boxed-grid**: `2026-07-09 Service Invoice from Whitlock & Sons Plumbing LLC.pdf` → `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf`
- **benefits-change-checkbox-form**: `Benefits Enrollment Change Form - Brewhouse operations.pdf` → `2026-06-20 Benefits Enrollment Change Form from Brewhouse operations.pdf`
- **loss-notice-boxed-fields**: `2026-02-16 Property Loss Notice between Harrowmere Mutual Insurance Company and Delacroix Bakehouse LLC.pdf` → `2026-02-16 Property Loss Notice for Harrowmere Mutual Insurance Company.pdf`
- **scan-remittance-advice-120dpi**: `2026-07-17 Invoice from Tolliver Grain & Feed Cooperative.png` → `2026-05-28 Invoice from Tolliver Grain & Feed Cooperative.png`
- **mixed-signature-region**: `First Amendment to Consulting Agreement between Corvane Analytics Inc and Pellworth County Water Authority.pdf` → `2026-08-18 First Amendment to Consulting Agreement between Rosalind Achterberg and Barnaby Quist.pdf`
- **scan-certificate-of-insurance**: `2026-04-01 Certificate of Liability Insurance - Ostergaard Roofing Contractors LLC.pdf` → `2026-06-23 Certificate of Liability Insurance from Ostergaard Roofing Contractors LLC.pdf`
- **scan-bill-of-lading**: `2026-09-08 Bill of lading from Bill of lading no. BL-2026-118745.pdf` → `2026-09-08 Bill of lading between Corriveau Millwork Supply Co and Tamberlane Builders Supply.pdf`

## Latency

A replayed run reports its recording's timings, taken when and where the recording was made, not measured by that run; a difference involving one is not a measured change. Latency is not compared: compare two live runs made on the same machine.


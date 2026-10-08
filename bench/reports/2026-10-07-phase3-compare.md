# InternBench comparison

- **Before:** replay run 2026-10-08T17:18:01Z at `f7ee8de2b25a3bb5e65048497c202ff1d20ec289` (77 documents) on Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS) · timings recorded (recording `1874e95a476a`, made 2026-10-07T05:45:28Z), not measured but for `index_ms` and `retrieval_ms`
- **After:** replay run 2026-10-08T17:18:06Z at `f7ee8de2b25a3bb5e65048497c202ff1d20ec289` (77 documents) on Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS) · timings recorded (recording `bb64876b233b`, made 2026-10-07T16:54:53Z), not measured but for `index_ms` and `retrieval_ms`
- **Compared:** every score, rate and count over the 77 documents both runs scored (completed, or failed and scored as a miss), each score over the documents that have it in both runs; latency over the 76 both completed.

## Phase 3 scorecard

| Figure | Before | After | Change |
| --- | ---: | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 75.0% | +8.3 pts better |
| Complex-document filename accuracy | 33.9% | 73.2% | +39.3 pts better |
| Description completeness | 53.5% | 53.7% | +0.2 pts better |
| Unsupported-fact rate (documents) | 27.3% | 11.7% | -15.6 pts better |
| Review rate | 32.9% | 15.8% | -17.1 pts |
| Evidence recall | 51.7% | 66.0% | +14.3 pts better |
| Total latency p50 | – | – | – |
| Total latency p95 | – | – | – |
| Generation latency p50 | – | – | – |
| Generation latency p95 | – | – | – |
| Generated tokens p50 | – | – | – |
| Generated tokens p95 | – | – | – |
| Prompt tokens p50 | – | – | – |

Over the documents both runs scored; a slice's accuracy over its documents. Latency and tokens are percentiles over the documents both runs completed, shown only when both runs measured their timings.

## Scores

| Score | Before | After | Change |
| --- | ---: | ---: | ---: |
| `filename_correct` | 26/77 (33.8%) | 54/77 (70.1%) | +36.4 pts better |
| `type_correct` | 61/77 (79.2%) | 72/77 (93.5%) | +14.3 pts better |
| `date_correct` | 65/77 (84.4%) | 75/77 (97.4%) | +13 pts better |
| `date_role_correct` | 43/63 (68.2%) | 37/63 (58.7%) | -9.5 pts worse |
| `parties_correct` | 53/77 (68.8%) | 67/77 (87.0%) | +18.2 pts better |
| `relation_correct` | 34/75 (45.3%) | 61/75 (81.3%) | +36 pts better |
| `party_role_correct` | 29/39 (74.4%) | 37/39 (94.9%) | +20.5 pts better |
| `readiness_match` | 37/57 (64.9%) | 44/57 (77.2%) | +12.3 pts better |
| `description_complete` | 11/77 (14.3%) | 21/77 (27.3%) | +13 pts better |
| `description_factual` | 75/76 (98.7%) | 76/76 (100.0%) | +1.3 pts better |
| `description_specific` | 76/77 (98.7%) | 74/77 (96.1%) | -2.6 pts worse |
| `unsafe_ready` | 30/77 (39.0%) | 15/77 (19.5%) | -19.5 pts better |
| `date_forbidden` | 11/77 (14.3%) | 1/77 (1.3%) | -13 pts better |
| `party_forbidden` | 9/77 (11.7%) | 8/77 (10.4%) | -1.3 pts better |
| `needless_review` | 3/53 (5.7%) | 2/53 (3.8%) | -1.9 pts better |
| `date_exact` | 62/77 (80.5%) | 74/77 (96.1%) | +15.6 pts better |
| `date_present` | 76/77 (98.7%) | 76/77 (98.7%) | 0 pts |
| `type_present` | 70/77 (90.9%) | 74/77 (96.1%) | +5.2 pts better |
| `unsupported_fact_doc` | 21/77 (27.3%) | 9/77 (11.7%) | -15.6 pts better |
| `description_completeness` (mean) | 53.5% | 53.7% | +0.2 pts better |
| `description_specificity` (mean) | 98.3% | 93.9% | -4.3 pts worse |
| `evidence_recall` (mean) | 51.7% | 66.0% | +14.3 pts better |
| `digest_recall` (mean) | 99.4% | 99.4% | 0 pts |
| `prompt_recall` (mean) | 99.4% | 100.0% | +0.7 pts better |
| `context_date_recall` (mean) | 100.0% | 100.0% | 0 pts |
| `context_fact_recall` (mean) | 98.5% | 98.5% | 0 pts |
| `context_party_recall` (mean) | 100.0% | 100.0% | 0 pts |
| `context_recall` (mean) | 100.0% | 100.0% | 0 pts |
| `context_subject_recall` (mean) | 96.8% | 96.8% | 0 pts |
| `context_type_recall` (mean) | 100.0% | 100.0% | 0 pts |
| `kv_accuracy` (mean) | 99.1% | 99.1% | 0 pts |
| `ocr_cer` (mean) | 5.0% | 5.0% | 0 pts |
| `ocr_cer_ci` (mean) | 4.9% | 4.9% | 0 pts |
| `ocr_date_accuracy` (mean) | 100.0% | 100.0% | 0 pts |
| `ocr_identifier_accuracy` (mean) | 100.0% | 100.0% | 0 pts |
| `ocr_mean_confidence` (mean) | 97.5 | 97.5 | 0 |
| `ocr_name_accuracy` (mean) | 100.0% | 100.0% | 0 pts |
| `ocr_wer` (mean) | 5.4% | 5.4% | 0 pts |
| `reading_order_accuracy` (mean) | 74.9% | 74.9% | 0 pts |
| `route_correct` (mean) | 100.0% | 100.0% | 0 pts |
| `table_cell_recall` (mean) | 99.4% | 99.4% | 0 pts |
| `table_row_accuracy` (mean) | 96.8% | 96.8% | 0 pts |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 66.7% → 75.0% (+8.3 pts) | 91.7% → 100.0% (+8.3 pts) | 83.3% → 100.0% (+16.7 pts) | 60.0% → 100.0% (+40 pts) | 2 → 3 | – | – |
| complex | 56 | 33.9% → 73.2% (+39.3 pts) | 85.7% → 96.4% (+10.7 pts) | 66.1% → 89.3% (+23.2 pts) | 65.2% → 87.0% (+21.7 pts) | 21 → 9 | – | – |

`long`: 10 pages or more. `complex`: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. A document may be in both.

## Safety counts

| Count | Before | After | Change |
| --- | ---: | ---: | ---: |
| unsafe_ready | 30 | 15 | -15 better |
| trap_dates | 11 | 1 | -10 better |
| forbidden_parties | 9 | 8 | -1 better |
| spurious_parties | 25 | 9 | -16 better |
| forbidden_descriptions | 0 | 0 | 0 |
| unsupported_claims | 1 | 0 | -1 better |
| claims | 264 | 286 | +22 |
| review_rate | 32.9% | 15.8% | -17.1 pts |
| unsupported_fact_rate | 0.4% | 0.0% | -0.4 pts better |

## Broken: 55 score(s) in 27 document(s)

- **notice-of-default**: date_role_correct
- **explanation-of-benefits**: description_specific, readiness_match, unsupported_fact_doc
- **engagement-letter**: parties_correct, party_forbidden, readiness_match, relation_correct, type_present, unsupported_fact_doc
- **annual-report-excerpt-8p**: date_correct, date_present, readiness_match, unsupported_fact_doc
- **board-minutes**: parties_correct, party_forbidden
- **field-condition-report**: parties_correct, party_forbidden
- **aircraft-maintenance-log**: description_specific, parties_correct, party_forbidden
- **credit-agreement-50p**: date_role_correct
- **demand-letter**: date_role_correct, unsafe_ready
- **board-resolution**: party_forbidden
- **quarterly-business-review**: filename_correct, relation_correct, unsafe_ready
- **product-launch-plan**: needless_review, readiness_match, unsupported_fact_doc
- **harvest-log**: date_role_correct, parties_correct, party_forbidden
- **scan-low-res-receipt**: readiness_match
- **scan-noisy-statement**: readiness_match
- **scan-faint-letter**: description_complete
- **agreement-two-column-footnotes**: filename_correct, type_correct, unsafe_ready
- **meeting-notice-columns**: description_complete
- **meeting-notice-interleaved**: description_complete
- **meeting-notice-reversed**: description_complete
- **price-list-unruled**: date_role_correct, description_specific, readiness_match, unsupported_fact_doc
- **invoice-label-above**: date_role_correct, needless_review
- **scan-remittance-advice-120dpi**: relation_correct
- **scan-certificate-of-insurance**: date_role_correct
- **scan-bill-of-lading**: date_role_correct, party_forbidden
- **industrial-lease-dated-in-schedule-25p**: description_complete
- **watershed-monitoring-report-100p**: unsafe_ready

## Fixed: 218 score(s) in 58 document(s)

- **notice-rent-increase**: filename_correct, party_role_correct, readiness_match, relation_correct, unsupported_fact_doc
- **invoice-date-in-table**: description_complete, party_forbidden
- **invoice-layout-only**: description_complete
- **account-statement**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, relation_correct, unsafe_ready
- **second-amendment**: filename_correct, type_correct, unsafe_ready
- **sow-under-msa-5p**: date_correct, date_exact, date_forbidden, filename_correct, unsafe_ready
- **notice-of-default**: description_complete, filename_correct, parties_correct, readiness_match, relation_correct, unsupported_fact_doc
- **offer-letter**: date_correct, date_exact, date_forbidden, filename_correct, relation_correct, unsafe_ready
- **prior-authorization**: description_complete, filename_correct, parties_correct, readiness_match, relation_correct, unsupported_fact_doc
- **explanation-of-benefits**: unsafe_ready
- **capital-call-notice**: filename_correct, party_role_correct, readiness_match, relation_correct, unsupported_fact_doc
- **engagement-letter**: description_complete, unsafe_ready
- **letterhead-letter**: parties_correct
- **lease-two-column**: date_exact
- **declarations-interleaved**: filename_correct, relation_correct, unsafe_ready
- **annual-report-excerpt-8p**: relation_correct, unsafe_ready
- **vendor-registration-form**: filename_correct, parties_correct, party_forbidden, relation_correct, unsafe_ready
- **board-minutes**: relation_correct, type_correct
- **aircraft-maintenance-log**: date_correct, date_exact, date_forbidden, relation_correct
- **asset-purchase-agreement-25p**: description_complete
- **credit-agreement-50p**: needless_review, readiness_match, unsupported_fact_doc
- **annual-report-100p**: date_role_correct, filename_correct, relation_correct, unsafe_ready
- **separation-agreement**: description_complete
- **demand-letter**: readiness_match, type_correct, unsupported_fact_doc
- **board-resolution**: relation_correct, type_correct
- **product-launch-plan**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, relation_correct, unsafe_ready
- **payroll-register**: date_role_correct, filename_correct, relation_correct, unsafe_ready
- **ap-aging-report**: filename_correct, parties_correct, readiness_match, relation_correct, type_correct, type_present, unsupported_fact_doc
- **email-approval-thread**: parties_correct, party_role_correct, relation_correct
- **court-hearing-notice**: description_factual
- **scan-rotated-90-invoice**: description_complete
- **scan-upside-down-po**: description_complete
- **scan-skewed-notice**: filename_correct, party_role_correct, relation_correct, unsupported_fact_doc
- **scan-low-res-receipt**: filename_correct, parties_correct, relation_correct, type_correct, type_present, unsupported_fact_doc
- **scan-noisy-statement**: description_complete, unsupported_fact_doc
- **scan-faint-letter**: filename_correct, type_correct, type_present
- **scan-lease-25p**: description_complete
- **scan-patient-intake-form**: filename_correct, relation_correct, type_correct, type_present, unsupported_fact_doc
- **newsletter-three-column**: filename_correct, relation_correct, unsafe_ready
- **meeting-notice-columns**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, party_forbidden, readiness_match, relation_correct, unsupported_fact_doc
- **meeting-notice-interleaved**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, party_forbidden, readiness_match, relation_correct, unsupported_fact_doc
- **meeting-notice-reversed**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, party_forbidden, readiness_match, relation_correct, unsupported_fact_doc
- **rate-confirmation-rotated**: description_complete, filename_correct, type_correct, unsafe_ready
- **inspection-log-ruled-2p**: filename_correct, parties_correct, relation_correct, unsafe_ready
- **price-list-unruled**: unsafe_ready
- **invoice-label-above**: description_complete, filename_correct, parties_correct, party_forbidden, party_role_correct
- **invoice-right-aligned**: description_complete, needless_review, readiness_match, unsupported_fact_doc
- **benefits-change-checkbox-form**: date_correct, date_exact, date_forbidden, filename_correct, parties_correct, party_role_correct, relation_correct, unsafe_ready
- **loss-notice-boxed-fields**: filename_correct, party_role_correct, relation_correct, unsafe_ready
- **scan-rotated-page-in-pdf**: filename_correct, parties_correct, party_forbidden, relation_correct, unsupported_fact_doc
- **scan-cancellation-notice-150dpi**: description_complete, parties_correct, unsupported_fact_doc
- **scan-remittance-advice-120dpi**: date_correct, date_exact, date_forbidden, type_correct
- **mixed-signature-region**: type_correct
- **scan-certificate-of-insurance**: filename_correct, party_role_correct, relation_correct, unsafe_ready
- **industrial-lease-dated-in-schedule-25p**: filename_correct, readiness_match, relation_correct
- **term-loan-parties-apart-40p**: needless_review, readiness_match
- **property-policy-declarations-mid-60p**: parties_correct, party_forbidden
- **watershed-monitoring-report-100p**: date_correct, date_exact, date_present, description_specific, parties_correct, readiness_match, type_correct, type_present

## Status changes

- **watershed-monitoring-report-100p**: model_failed → completed

## Filenames that changed (55)

- **notice-rent-increase**: `2026-05-12 Notice of Rent Increase for Cresthaven Court Holdings LLC.pdf` → `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`
- **invoice-date-in-table**: `2026-03-04 Invoice from Quillon Ridge Bakery, Inc.pdf` → `2026-03-04 Invoice.pdf`
- **account-statement**: `2026-03-01 Statement - Kingsfold Community Bank.pdf` → `2026-04-01 Statement from Kingsfold Community Bank.pdf`
- **purchase-order**: `2026-01-29 Purchase Order between Lamplighter Robotics Inc and Kestrel Instruments Ltd.pdf` → `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`
- **second-amendment**: `2025-09-29 Second Amendment to Software License and Support with Umberlee Imaging Software Inc.pdf` → `2025-09-29 Second Amendment to Software License and Support Agreement with Umberlee Imaging Software Inc.pdf`
- **sow-under-msa-5p**: `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` → `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`
- **notice-of-default**: `2025-10-14 Notice of Default and Reservation of Rights - Basalt Commercial Credit Corp.pdf` → `2025-10-14 Notice of Default and Reservation of Rights for Glasswing Ceramics LLC.pdf`
- **offer-letter**: `2026-03-16 Offer of Employment - Corvid Data Systems Inc.pdf` → `2026-02-10 Offer of Employment from Corvid Data Systems Inc.pdf`
- **prior-authorization**: `2026-03-09 Notice of Prior Authorization Approval - Silverlode Health Partners.pdf` → `2026-03-09 Notice of Prior Authorization Approval to Florian Okonkwo.pdf`
- **explanation-of-benefits**: `2026-01-15 Statement of Benefits - Paloma Achterberg.pdf` → `2026-01-15 Meadowlark Health Plan from Paloma Achterberg.pdf`
- **capital-call-notice**: `2026-02-06 CAPITAL CALL NOTICE No for Ravensmoor Growth Partners III, L.P.pdf` → `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`
- **engagement-letter**: `2026-01-09 Letter between Silverbirch Ceramics Inc and Halloran Ostrowski CPAs LLP.pdf` → `2026-01-09 Document - Joaquin Sandoval.pdf`
- **letterhead-letter**: `2026-03-19 Document - Moonrake Paper & Packaging Co.pdf` → `2026-03-19 Document - Briarport Coffee Roasters LLC.pdf`
- **lease-two-column**: `2026-07-01 Retail Lease between Oakhaven Retail Properties LLC and Sorrel & Thistle Tea House LLC.pdf` → `2026-05-18 Retail Lease between Oakhaven Retail Properties LLC and Sorrel & Thistle Tea House LLC.pdf`
- **declarations-interleaved**: `2026-06-18 Commercial Package Policy Declarations - Northfell Mutual Insurance Company.pdf` → `2026-06-18 Commercial Package Policy Declarations from Northfell Mutual Insurance Company.pdf`
- **annual-report-excerpt-8p**: `2025-12-31 2025 Annual Report - Rookwood Precision Metals Corporation.pdf` → `Annual Report from Rookwood Precision Metals Corporation.pdf`
- **vendor-registration-form**: `2026-04-22 Vendor Registration Form - Emberglow Coatings Ltd.pdf` → `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`
- **board-minutes**: `2025-09-17 Minutes - Cairnfield Cooperative Grocers.pdf` → `2025-09-17 Minutes of the Regular Meeting of the Board of Directors for Beckett Halloran.pdf`
- **field-condition-report**: `2025-10-02 Loan Condition Report - Haverford Family Collection.pdf` → `2025-10-02 Loan Condition Report for Delphine Moncrieff.pdf`
- **aircraft-maintenance-log**: `2026-08-10 Maintenance Record - Highmeadow Air Charter LLC.pdf` → `2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf`
- **annual-report-100p**: `2026-09-18 Annual Report - Tamsin Valley Farmers Cooperative.pdf` → `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`
- **demand-letter**: `2026-07-08 via Email and Certified Mail, Return Receipt Requested - Fernvale Cider Works LLC.docx` → `2026-07-08 Demand for Payment - Fernvale Cider Works LLC.docx`
- **board-resolution**: `2025-11-03 Approval of Minutes - Summerhill Robotics, Inc.docx` → `2025-11-03 Unanimous Written Consent of the Board of Directors for Celeste Fontaine.docx`
- **quarterly-business-review**: `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx` → `2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx`
- **product-launch-plan**: `2026-04-24 Fieldnote Product Launch Plan - Quarrystone Audio Labs Inc.pptx` → `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`
- **payroll-register**: `2026-07-15 Payroll Register - Wrenfield Bakehouse LLC.xlsx` → `2026-07-15 Payroll Register from Wrenfield Bakehouse LLC.xlsx`
- **ap-aging-report**: `2026-08-31 Document - Lowmarsh Farm Equipment Co.csv` → `2026-08-31 Accounts Payable Aging Summary from Lowmarsh Farm Equipment Co.csv`
- **harvest-log**: `2026-10-02 Harvest Log - Rowanbrae Vineyards.xlsx` → `2026-10-02 Harvest Log from Esme Varga.xlsx`
- **email-approval-thread**: `2026-03-17 Document between Pemberly Falls Distribution Co and Basalt Telemetry Inc.eml` → `2026-03-17 Document from Signe Holmqvist.eml`
- **court-hearing-notice**: `2026-07-29 Notice of Hearing - Gilchrist Harbor Marine Supply, LLC.txt` → `2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt`
- **scan-skewed-notice**: `2026-08-03 Notice of Nonrenewal of Lease for Harrowgate Apartments LLC.tiff` → `2026-08-03 Notice of Nonrenewal of Lease to Wendell Okafor.tiff`
- **scan-low-res-receipt**: `2026-05-09 Document - Quarry Bend Hardware & Feed.png` → `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`
- **scan-noisy-statement**: `2026-08-31 Statement of Account for Copper Flats Dental Group PLLC.pdf` → `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`
- **scan-faint-letter**: `2026-03-02 Document from Mireille Saltonstall.png` → `2026-03-02 Letter of Resignation from Mireille Saltonstall.png`
- **scan-fax-two-frames**: `2026-08-12 Quotation to Highmeadow Air Charter LLC.tiff` → `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`
- **scan-patient-intake-form**: `2026-08-27 Document - Calder Way Family Medicine.png` → `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`
- **newsletter-three-column**: `2026-09-14 Newsletter - Saltmarsh Point Homeowners Association.pdf` → `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`
- **agreement-two-column-footnotes**: `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` → `2026-03-02 Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`
- **meeting-notice-columns**: `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` → `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`
- **meeting-notice-interleaved**: `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` → `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`
- **meeting-notice-reversed**: `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` → `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`
- **rate-confirmation-rotated**: `2026-06-08 Invoice between Copperline Freight Brokerage LLC and Halden Ridge Trucking Inc.pdf` → `2026-06-08 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf`
- **inspection-log-ruled-2p**: `2026-05-19 Fire Extinguisher Inspection Report - Emberwatch Fire Protection Inc.pdf` → `2026-05-19 Fire Extinguisher Inspection Report from Emberwatch Fire Protection Inc.pdf`
- **invoice-label-above**: `2026-05-11 Invoice from Bellhaven Physical Therapy PLLC.pdf` → `2026-05-11 Invoice from Quarrystone Signs & Graphics.pdf`
- **invoice-boxed-grid**: `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf` → `2026-07-09 Service Invoice from Whitlock & Sons Plumbing LLC.pdf`
- **benefits-change-checkbox-form**: `2026-06-20 Benefits Enrollment Change Form from Brewhouse operations.pdf` → `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`
- **loss-notice-boxed-fields**: `2026-02-16 Property Loss Notice for Harrowmere Mutual Insurance Company.pdf` → `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`
- **scan-rotated-page-in-pdf**: `2026-04-27 Notice of Freight Claim - Redgate Interstate Freight Inc.pdf` → `2026-04-27 Notice of Freight Claim from Hollin Brook Furniture Gallery.pdf`
- **scan-cancellation-notice-150dpi**: `2026-08-12 Notice of Cancellation for Nonpayment of Premium with Kestrelmoor Casualty Company.pdf` → `2026-08-12 Notice of Cancellation for Nonpayment of Premium - Brannock Tool & Die Inc.pdf`
- **scan-remittance-advice-120dpi**: `2026-05-28 Invoice from Tolliver Grain & Feed Cooperative.png` → `2026-07-17 Remittance Advice - Sablewood Packaging Corp.png`
- **mixed-signature-region**: `2026-08-18 First Amendment to Consulting Agreement between Rosalind Achterberg and Barnaby Quist.pdf` → `2026-08-18 First Amendment to Software License Agreement between Rosalind Achterberg and Barnaby Quist.pdf`
- **scan-certificate-of-insurance**: `2026-06-23 Certificate of Liability Insurance from Ostergaard Roofing Contractors LLC.pdf` → `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`
- **scan-bill-of-lading**: `2026-09-08 Bill of lading between Corriveau Millwork Supply Co and Tamberlane Builders Supply.pdf` → `2026-09-08 Straight Bill of Lading - Odile Corriveau.pdf`
- **industrial-lease-dated-in-schedule-25p**: `2026-04-14 Industrial Lease - Ardent Quay Industrial Properties LP.pdf` → `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`
- **property-policy-declarations-mid-60p**: `2026-07-01 Commercial Property Policy - Hollowmere Craft Brewing Cooperative.pdf` → `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf`

## Latency

A replayed run reports its recording's timings, taken when and where the recording was made, not measured by that run (all but `index_ms` and `retrieval_ms`, which it measures); a difference involving one is not a measured change. Latency is not compared: compare two live runs made on the same machine.


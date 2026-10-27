Documents: 77; prompt SHA-256 identical 2B vs 4B: 77/77

| Run | Filename | Ready | Review | Unsafe ready | Needless review | Readiness match | Type | Date | Parties | Relation | Trap date | Forbidden party | Failed | Sum inference s | Sum extraction s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| b2 | 37/77 | 54 | 23 | 23 | 2 | 39 | 66 | 67 | 55 | 46 | 5 | 1 | 0 | 1085 | 189 |
| b4 | 48/77 | 63 | 14 | 22 | 4 | 45 | 70 | 70 | 68 | 61 | 6 | 2 | 0 | 2729 | 197 |
| qwen_replay | 54/77 | 65 | 12 | 15 | 2 | 44 | 72 | 75 | 67 | 61 | 1 | 8 | 0 | 1492 | 131 |
| qwen_live | 52/77 | 66 | 11 | 18 | 3 | 44 | 70 | 75 | 64 | 62 | 1 | 10 | 0 | 1585 | 181 |

4B fixes the 2B name on 17: account-statement, aircraft-maintenance-log, ap-aging-report, declarations-interleaved, explanation-of-benefits, harvest-log, inspection-log-ruled-2p, invoice-boxed-grid, invoice-layout-only, meeting-notice-columns, meeting-notice-interleaved, meeting-notice-reversed, notice-of-default, notice-rent-increase, payroll-register, scan-remittance-advice-120dpi, scan-upside-down-po

4B breaks a correct 2B name on 6: board-minutes, credit-agreement-50p, invoice-date-in-table, product-launch-plan, scan-faint-letter, scan-fax-two-frames

2B unsafe ready (23): aircraft-maintenance-log, ap-aging-report, benefits-change-checkbox-form, declarations-interleaved, explanation-of-benefits, field-condition-report, harvest-log, industrial-lease-dated-in-schedule-25p, invoice-boxed-grid, loss-notice-boxed-fields, meeting-notice-columns, meeting-notice-interleaved, meeting-notice-reversed, newsletter-three-column, notice-of-default, notice-rent-increase, property-policy-declarations-mid-60p, quarterly-business-review, scan-certificate-of-insurance, scan-patient-intake-form, scan-upside-down-po, sow-under-msa-5p, vendor-registration-form

| Policy | Escalated | Filename | Ready | Review | Unsafe ready | Needless review | Readiness match | Sum inference s | Sum total s | + per-doc swaps s |
|---|---|---|---|---|---|---|---|---|---|---|
| 2B always | 0 | 37 | 54 | 23 | 23 | 2 | 39 | 1085 | 1275 | 1275 |
| 4B always | 77 | 48 | 63 | 14 | 22 | 4 | 45 | 2729 | 2918 | 2918 |
| 2B, escalate if review | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if route_ocr | 20 | 37 | 56 | 21 | 24 | 2 | 38 | 1625 | 1814 | 1934 |
| 2B, escalate if pages>=5 | 16 | 35 | 55 | 22 | 26 | 2 | 40 | 1814 | 2003 | 2099 |
| 2B, escalate if pages>=10 | 12 | 36 | 54 | 23 | 24 | 2 | 39 | 1683 | 1873 | 1945 |
| 2B, escalate if not_pdf | 17 | 38 | 59 | 18 | 26 | 2 | 42 | 1666 | 1856 | 1958 |
| 2B, escalate if no_date | 1 | 37 | 54 | 23 | 23 | 2 | 39 | 1138 | 1327 | 1333 |
| 2B, escalate if no_type | 1 | 38 | 55 | 22 | 23 | 2 | 40 | 1110 | 1299 | 1305 |
| 2B, escalate if parties<2 | 21 | 44 | 56 | 21 | 21 | 5 | 41 | 1818 | 2007 | 2133 |
| 2B, escalate if parties>=3 | 26 | 38 | 57 | 20 | 24 | 2 | 40 | 2013 | 2202 | 2358 |
| 2B, escalate if date_role_other | 1 | 37 | 54 | 23 | 23 | 2 | 39 | 1138 | 1327 | 1333 |
| 2B, escalate if date_role_execution | 7 | 38 | 54 | 23 | 21 | 2 | 39 | 1332 | 1521 | 1563 |
| 2B, escalate if prompt>1500 | 10 | 38 | 54 | 23 | 22 | 2 | 39 | 1654 | 1843 | 1903 |
| 2B, escalate if facts_unsupported>0 | 17 | 39 | 67 | 10 | 31 | 1 | 48 | 1645 | 1834 | 1936 |
| 2B, escalate if unknown_or_miscited_ids>0 | 61 | 49 | 63 | 14 | 22 | 5 | 44 | 3213 | 3402 | 3768 |
| 2B, escalate if cited_support<1 | 59 | 49 | 61 | 16 | 20 | 5 | 42 | 3126 | 3315 | 3669 |
| 2B, escalate if r:TYPE_UNSUPPORTED | 11 | 38 | 62 | 15 | 29 | 1 | 45 | 1446 | 1635 | 1701 |
| 2B, escalate if r:TYPE_INFERRED | 9 | 38 | 62 | 15 | 27 | 1 | 46 | 1381 | 1570 | 1624 |
| 2B, escalate if r:DATE_AMBIGUOUS | 3 | 37 | 54 | 23 | 23 | 2 | 39 | 1155 | 1344 | 1362 |
| 2B, escalate if r:DATE_UNSUPPORTED | 4 | 38 | 58 | 19 | 26 | 2 | 42 | 1229 | 1419 | 1443 |
| 2B, escalate if r:PARTY_UNSUPPORTED | 1 | 37 | 54 | 23 | 23 | 2 | 39 | 1116 | 1305 | 1311 |
| 2B, escalate if r:DATE_MISSING | 1 | 37 | 54 | 23 | 23 | 2 | 39 | 1138 | 1327 | 1333 |
| 2B, escalate if conf_min<0.3 | 10 | 39 | 59 | 18 | 26 | 2 | 42 | 1449 | 1638 | 1698 |
| 2B, escalate if conf_min<0.5 | 16 | 38 | 59 | 18 | 27 | 2 | 42 | 1708 | 1897 | 1993 |
| 2B, escalate if conf_min<0.7 | 40 | 44 | 59 | 18 | 22 | 5 | 41 | 2586 | 2775 | 3015 |
| 2B, escalate if conf_min<0.8 | 52 | 46 | 60 | 17 | 22 | 5 | 43 | 3011 | 3201 | 3513 |
| 2B, escalate if conf_min<0.9 | 65 | 48 | 62 | 15 | 22 | 5 | 44 | 3459 | 3648 | 4038 |
| 2B, escalate if conf_min<0.95 | 69 | 48 | 63 | 14 | 22 | 4 | 45 | 3585 | 3774 | 4188 |
| 2B, escalate if conf_mean<0.9 | 4 | 37 | 55 | 22 | 24 | 2 | 40 | 1261 | 1450 | 1474 |
| 2B, escalate if conf_mean<0.95 | 17 | 41 | 59 | 18 | 24 | 2 | 42 | 1792 | 1981 | 2083 |
| 2B, escalate if conf_mean<0.98 | 50 | 47 | 60 | 17 | 22 | 5 | 43 | 2915 | 3105 | 3405 |
| 2B, escalate if review OR route_ocr | 36 | 41 | 68 | 9 | 31 | 1 | 50 | 2191 | 2380 | 2596 |
| 2B, escalate if review OR pages>=5 | 36 | 39 | 68 | 9 | 32 | 1 | 49 | 2410 | 2599 | 2815 |
| 2B, escalate if review OR pages>=10 | 33 | 40 | 68 | 9 | 31 | 1 | 49 | 2310 | 2500 | 2698 |
| 2B, escalate if review OR not_pdf | 32 | 41 | 69 | 8 | 31 | 1 | 50 | 2156 | 2345 | 2537 |
| 2B, escalate if review OR no_date | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if review OR no_type | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if review OR parties<2 | 37 | 46 | 66 | 11 | 26 | 4 | 47 | 2292 | 2481 | 2703 |
| 2B, escalate if review OR parties>=3 | 40 | 41 | 67 | 10 | 30 | 1 | 49 | 2493 | 2682 | 2922 |
| 2B, escalate if review OR party_role_other | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if review OR date_role_other | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if review OR date_role_execution | 29 | 43 | 69 | 8 | 29 | 1 | 50 | 2046 | 2235 | 2409 |
| 2B, escalate if review OR prompt>1500 | 31 | 42 | 68 | 9 | 29 | 1 | 49 | 2281 | 2470 | 2656 |
| 2B, escalate if review OR facts_unsupported>0 | 23 | 41 | 69 | 8 | 31 | 1 | 50 | 1822 | 2011 | 2149 |
| 2B, escalate if review OR unknown_or_miscited_ids>0 | 66 | 48 | 65 | 12 | 23 | 4 | 46 | 3386 | 3575 | 3971 |
| 2B, escalate if review OR cited_support<1 | 66 | 48 | 65 | 12 | 23 | 4 | 46 | 3386 | 3575 | 3971 |
| 2B, escalate if review OR conf_min<0.3 | 26 | 41 | 69 | 8 | 31 | 1 | 50 | 1933 | 2122 | 2278 |
| 2B, escalate if review OR conf_min<0.5 | 29 | 40 | 68 | 9 | 31 | 1 | 49 | 2094 | 2284 | 2458 |
| 2B, escalate if review OR conf_min<0.7 | 47 | 46 | 64 | 13 | 24 | 4 | 45 | 2777 | 2966 | 3248 |
| 2B, escalate if review OR conf_min<0.8 | 57 | 46 | 63 | 14 | 24 | 4 | 45 | 3147 | 3337 | 3679 |
| 2B, escalate if review OR conf_min<0.9 | 67 | 48 | 63 | 14 | 22 | 4 | 45 | 3510 | 3699 | 4101 |
| 2B, escalate if review OR conf_min<0.95 | 69 | 48 | 63 | 14 | 22 | 4 | 45 | 3585 | 3774 | 4188 |
| 2B, escalate if review OR conf_mean<0.9 | 25 | 41 | 69 | 8 | 31 | 1 | 50 | 1888 | 2077 | 2227 |
| 2B, escalate if review OR conf_mean<0.95 | 32 | 43 | 68 | 9 | 28 | 1 | 49 | 2207 | 2396 | 2588 |
| 2B, escalate if review OR conf_mean<0.98 | 55 | 46 | 63 | 14 | 24 | 4 | 45 | 3052 | 3241 | 3571 |
| oracle: escalate exactly where 4B fixes the name | 17 | 54 | 56 | 21 | 11 | 5 | 39 | 1664 | 1853 | 1955 |

| Signal (from the 2B pass) | Fires | on 2B-wrong | on 4B-fixes | on 4B-breaks | on 2B-unsafe |
|---|---|---|---|---|---|
| review | 23 | 17 | 5 | 1 | 0 |
| route_ocr | 20 | 6 | 2 | 2 | 3 |
| pages>=5 | 16 | 7 | 0 | 2 | 4 |
| pages>=10 | 12 | 5 | 0 | 1 | 3 |
| not_pdf | 17 | 11 | 4 | 3 | 4 |
| no_date | 1 | 1 | 0 | 0 | 0 |
| no_type | 1 | 1 | 1 | 0 | 0 |
| parties<2 | 21 | 18 | 9 | 2 | 11 |
| parties>=3 | 26 | 13 | 4 | 3 | 7 |
| date_role_other | 1 | 1 | 0 | 0 | 0 |
| date_role_execution | 7 | 3 | 2 | 1 | 3 |
| prompt>1500 | 10 | 6 | 2 | 1 | 4 |
| facts_unsupported>0 | 17 | 14 | 3 | 1 | 0 |
| unknown_or_miscited_ids>0 | 61 | 33 | 16 | 4 | 19 |
| cited_support<1 | 59 | 31 | 16 | 4 | 19 |
| r:TYPE_UNSUPPORTED | 11 | 10 | 1 | 0 | 0 |
| r:TYPE_INFERRED | 9 | 6 | 2 | 1 | 0 |
| r:DATE_AMBIGUOUS | 3 | 0 | 0 | 0 | 0 |
| r:DATE_UNSUPPORTED | 4 | 4 | 1 | 0 | 0 |
| r:PARTY_UNSUPPORTED | 1 | 1 | 0 | 0 | 0 |
| r:DATE_MISSING | 1 | 1 | 0 | 0 | 0 |
| conf_min<0.3 | 10 | 8 | 2 | 0 | 2 |
| conf_min<0.5 | 16 | 12 | 2 | 1 | 4 |
| conf_min<0.7 | 40 | 27 | 12 | 5 | 15 |
| conf_min<0.8 | 52 | 33 | 15 | 6 | 19 |
| conf_min<0.9 | 65 | 40 | 17 | 6 | 23 |
| conf_min<0.95 | 69 | 40 | 17 | 6 | 23 |
| conf_mean<0.9 | 4 | 3 | 0 | 0 | 1 |
| conf_mean<0.95 | 17 | 15 | 5 | 1 | 7 |
| conf_mean<0.98 | 50 | 34 | 15 | 5 | 19 |

Paired bootstrap (4000 resamples of the documents), difference in correct filenames:
- bootstrap_4b_vs_2b: mean +11.08, 95% interval [2, 20]
- bootstrap_review_escalation_vs_2b: mean +4.01, 95% interval [0, 9]
- bootstrap_review_escalation_vs_4b: mean -7.07, 95% interval [-15, 1]

| Doc | kind | layer | route | pages | 2B name ok | 2B ready | 4B name ok | 4B ready | 2B reasons | 2B conf min | 2B filename | 4B filename |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| notice-rent-increase | notice | native | layout | 1 | False | True | True | True |  | 0.89 | 2026-05-12 Notice for Imogen Castellanos.pdf | 2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf |
| invoice-date-in-table | invoice | native | layout | 1 | True | True | False | False |  | 0.60 | 2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf | 2026-03-04 Invoice.pdf |
| invoice-layout-only | invoice | native | layout | 1 | False | False | True | True | DATE_UNSUPPORTED | 0.00 | INVOICE from Ironvale Marine Coatings Co.pdf | 2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf |
| account-statement | statement | native | layout | 2 | False | False | True | True | TYPE_INFERRED | 0.54 | 2026-03-01 Statement from Kingsfold Community Bank.pdf | 2026-04-01 Statement from Kingsfold Community Bank.pdf |
| sow-under-msa-5p | sow | native | layout | 5 | False | True | False | True |  | 0.86 | 2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf | 2026-06-01 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf |
| notice-of-default | notice | native | layout | 2 | False | True | True | True |  | 0.77 | 2025-10-14 Loan Agreement between Basalt Commercial Credit Corp and Glasswing Ceramics LLC.pdf | 2025-10-14 Notice of Default and Reservation of Rights for Glasswing Ceramics LLC.pdf |
| explanation-of-benefits | statement | native | layout | 1 | False | True | True | True |  | 0.68 | 2026-01-15 Meadowlark Health Plan.pdf | 2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf |
| engagement-letter | letter | native | layout | 2 | False | False | False | True | TYPE_UNSUPPORTED | 0.80 | 2026-01-09 Document from Halloran Ostrowski Cpas LLP.pdf | 2026-01-09 LETTER from Halloran Ostrowski Cpas LLP.pdf |
| letterhead-letter | letter | native | layout | 1 | False | False | False | False | TYPE_UNSUPPORTED | 0.49 | 2026-03-19 Document - Moonrake Paper & Packaging Co.pdf | 2026-03-19 Document from Moonrake Paper & Packaging Co.pdf |
| declarations-interleaved | insurance | native | layout | 2 | False | True | True | True |  | 0.51 | 2026-07-01 Commercial Package Policy Declarations for Commercial General Liability Coverage Part Declarations.pdf | 2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf |
| annual-report-excerpt-8p | annual_report | native | layout | 8 | False | False | False | True | TYPE_UNSUPPORTED|TYPE_INFERRED|DATE_UNSUPPORTED | 0.24 | Annual Report from Rookwood Precision Metals Corporation.pdf | 2026-03-12 2025 Annual Report to Fellow Shareholders.pdf |
| vendor-registration-form | form | native | layout | 1 | False | True | False | True |  | 0.22 | 2026-04-22 Vendor Registration Form.pdf | 2026-04-22 Vendor Registration Form for Hartwell County.pdf |
| board-minutes | minutes | native | fast | 2 | True | True | False | True |  | 0.77 | 2025-09-17 Minutes of the Regular Meeting of the Board of Directors.pdf | 2025-09-17 MINUTES for Yara Delacroix-Hale.pdf |
| field-condition-report | condition_report | native | layout | 2 | False | True | False | True |  | 0.70 | 2025-10-02 Loan Condition Report for Anselm Greaves.pdf | 2025-10-02 Loan Condition Report for Anselm Greaves.pdf |
| aircraft-maintenance-log | maintenance_record | native | layout | 1 | False | True | True | True |  | 0.56 | 2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf | 2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf |
| credit-agreement-50p | contract | native | layout | 50 | True | True | False | False |  | 0.31 | 2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf | 2026-06-12 Credit Agreement between The Lenders Party Hereto and Halden Bay National Bank, N.A.pdf |
| annual-report-100p | annual_report | native | layout | 100 | False | False | False | False | DATE_MISSING | 0.07 | Annual Report.pdf | Annual Report for Farmers and Ranchers.pdf |
| demand-letter | letter | office | fast | 1 | False | False | False | True | TYPE_UNSUPPORTED | 0.59 | 2026-07-22 Document - Fernvale Cider Works LLC.docx | 2026-07-22 LETTER - Fernvale Cider Works LLC.docx |
| board-resolution | resolution | office | fast | 1 | False | False | False | True | TYPE_UNSUPPORTED|TYPE_INFERRED | 0.37 | 2025-11-03 Unanimous Written Consent of the Board of Directors for The undersigned.docx | 2025-11-03 RESOLUTION for Celeste Fontaine.docx |
| quarterly-business-review | presentation | office | fast | 11 | False | True | False | True |  | 0.13 | 2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx | 2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx |
| product-launch-plan | presentation | office | fast | 8 | True | True | False | True |  | 0.69 | 2026-06-03 Product Launch Plan.pptx | 2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx |
| payroll-register | payroll | sheet | fast | 3 | False | False | True | True | DESCRIPTION_INVALID | 0.73 | 2026-07-15 Payroll Register.xlsx | 2026-07-15 Payroll Register for Wrenfield Bakehouse LLC.xlsx |
| ap-aging-report | report | sheet | fast | 1 | False | True | True | True |  | 0.62 | 2026-08-31 Accounts Payable Aging Summary.csv | 2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv |
| harvest-log | log | sheet | fast | 4 | False | True | True | True |  | 0.63 | 2026-10-02 Summary.xlsx | 2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx |
| email-approval-thread | email | email | fast | 1 | False | False | False | False | TYPE_UNSUPPORTED|PARTY_UNSUPPORTED | 0.57 | 2026-03-17 Document - Signe Holmqvist.eml | 2026-03-17 Document - Signe Holmqvist.eml |
| court-hearing-notice | notice | text | fast | 1 | False | False | False | True | TYPE_UNSUPPORTED|TYPE_INFERRED | 0.30 | 2026-07-30 Notice of Hearing - Halden County Superior Court Clerk.txt | 2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt |
| scan-upside-down-po | purchase_order | scan | ocr | 1 | False | True | True | True |  | 0.86 | 2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf | 2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf |
| scan-low-res-receipt | receipt | scan | ocr | 1 | False | False | False | False | TYPE_UNSUPPORTED | 0.85 | 2026-05-09 Document.png | 2026-05-09 Document from Quarry Bend Hardware & Feed.png |
| scan-faint-letter | letter | scan | ocr | 1 | True | False | False | False | TYPE_INFERRED | 0.63 | 2026-03-02 Letter of Resignation from Mireille Saltonstall.png | 2026-03-02 Letter of Resignation - Dr. Rufus Pemberton.png |
| scan-fax-two-frames | quotation | scan | ocr | 2 | True | True | False | True |  | 0.66 | 2026-08-12 QUOTATION from Kestrel Ridge Aero Services.tiff | 2026-08-12 QUOTATION from Kestrel Ridge Aero Services - Parts and Quotes.tiff |
| scan-patient-intake-form | form | scan | ocr | 1 | False | True | False | True |  | 0.72 | 2026-01-01 New Patient Intake Form.png | 2026-01-01 New Patient Intake Form for Calder Way Family Medicine.png |
| newsletter-three-column | newsletter | native | layout | 1 | False | True | False | True |  | 0.89 | 2026-09-14 The Tidewater Ledger.pdf | 2026-09-14 Newsletter of the Saltmarsh Point Homeowners Association for Saltmarsh Point Homeowners Association.pdf |
| meeting-notice-columns | notice | native | layout | 1 | False | True | True | False |  | 0.67 | 2026-08-03 Notice of Special Meeting of Members.pdf | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf |
| meeting-notice-interleaved | notice | native | layout | 1 | False | True | True | False |  | 0.67 | 2026-08-03 Notice of Special Meeting of Members.pdf | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf |
| meeting-notice-reversed | notice | native | layout | 1 | False | True | True | False |  | 0.67 | 2026-08-03 Notice of Special Meeting of Members.pdf | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf |
| inspection-log-ruled-2p | inspection_report | native | layout | 2 | False | False | True | True | TYPE_INFERRED | 0.75 | 2026-05-19 Fire Extinguisher Inspection Report.pdf | 2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf |
| price-list-unruled | price_list | native | layout | 1 | False | False | False | True | TYPE_UNSUPPORTED|TYPE_INFERRED | 0.59 | 2026-03-16 Price List.pdf | 2026-03-16 Spring Wholesale Availability and Price List from Thistledown Wholesale Nursery.pdf |
| invoice-boxed-grid | invoice | native | layout | 1 | False | True | True | True |  | 0.66 | 2026-07-09 Service Invoice.pdf | 2026-07-09 Service Invoice from Whitlock & Sons Plumbing LLC.pdf |
| benefits-change-checkbox-form | form | native | layout | 1 | False | True | False | True |  | 0.61 | 2026-06-29 Benefits Enrollment Change Form for Westbrook Kettle Brewing Co.pdf | 2026-07-01 Benefits Enrollment Change Form for Human Resources -.pdf |
| loss-notice-boxed-fields | form | native | layout | 1 | False | True | False | True |  | 0.71 | 2026-02-16 Property Loss Notice.pdf | 2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf |
| scan-remittance-advice-120dpi | remittance_advice | scan | ocr | 1 | False | False | True | True | TYPE_UNSUPPORTED | 0.24 | 2026-07-17 Document - Tolliver Grain & Feed Cooperative.png | 2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png |
| mixed-signature-region | amendment | mixed | ocr_regions | 2 | False | False | False | True | DATE_UNSUPPORTED | 0.82 | First Amendment to Software License Agreement between Corvane Analytics Inc and Pellworth County Water Authority.pdf | 2026-08-18 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf |
| scan-certificate-of-insurance | certificate | scan | ocr | 1 | False | True | False | True |  | 0.36 | 2026-06-23 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf | 2026-06-23 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf |
| industrial-lease-dated-in-schedule-25p | contract | native | layout | 25 | False | True | False | True |  | 0.52 | 2026-01-09 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf | 2026-04-16 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf |
| property-policy-declarations-mid-60p | insurance | native | layout | 60 | False | True | False | True |  | 0.47 | 2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf | 2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf |
| watershed-monitoring-report-100p | report | native | layout | 100 | False | False | False | True | DATE_UNSUPPORTED | 0.12 | Annual Watershed Monitoring Report.pdf | 2026-06-19 Annual Watershed Monitoring Report to Cobalt Basin Water Authority.pdf |

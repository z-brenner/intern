# d1 as a review gate over ready names

Documents asked: 60 (0 failed attempts). Ready documents with a party and an answer: 60 (14 wrong names, 46 right).
d1 time per document: median 14.8 s, max 58.6 s.

## Leave-one-out threshold, worth holding 2 right name(s) to hold one wrong one

| Signal | Wrong names held | Right names held |
| --- | ---: | ---: |
| max_party_relation | 8/14 | 5/46 |
| party_main | 8/14 | 5/46 |

## Leave-one-out threshold, worth holding 1 right name(s) to hold one wrong one

| Signal | Wrong names held | Right names held |
| --- | ---: | ---: |
| max_party_relation | 8/14 | 5/46 |
| party_main | 8/14 | 5/46 |

## Every threshold (fitted on all ready documents; optimistic)

### max_party_relation

| Threshold | Wrong held | Right held |
| ---: | ---: | ---: |
| 0.161 | 9 | 11 |
| 0.179 | 9 | 10 |
| 0.182 | 9 | 9 |
| 0.184 | 9 | 8 |
| 0.212 | 9 | 7 |
| 0.228 | 9 | 6 |
| 0.310 | 9 | 5 |
| 0.320 | 8 | 5 |
| 0.336 | 7 | 5 |
| 0.354 | 6 | 5 |
| 0.370 | 5 | 5 |
| 0.387 | 4 | 5 |
| 0.395 | 3 | 5 |
| 0.396 | 2 | 5 |
| 0.458 | 2 | 4 |
| 0.474 | 1 | 4 |
| 0.496 | 1 | 3 |
| 0.512 | 1 | 2 |
| 0.517 | 1 | 1 |
| 0.580 | 0 | 1 |

### party_main

| Threshold | Wrong held | Right held |
| ---: | ---: | ---: |
| 0.161 | 9 | 11 |
| 0.179 | 9 | 10 |
| 0.182 | 9 | 9 |
| 0.184 | 9 | 8 |
| 0.212 | 9 | 7 |
| 0.228 | 9 | 6 |
| 0.310 | 9 | 5 |
| 0.320 | 8 | 5 |
| 0.336 | 7 | 5 |
| 0.354 | 6 | 5 |
| 0.370 | 5 | 5 |
| 0.387 | 4 | 5 |
| 0.395 | 3 | 5 |
| 0.396 | 2 | 5 |
| 0.458 | 2 | 4 |
| 0.474 | 1 | 4 |
| 0.496 | 1 | 3 |
| 0.512 | 1 | 2 |
| 0.517 | 1 | 1 |
| 0.580 | 0 | 1 |

## Per document

| Document | Wrong | Name | max_party_relation | party_main |
| --- | --- | --- | ---: | ---: |
| aircraft-maintenance-log | yes | 2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf | 0.31 | 0.31 |
| annual-report-100p | yes | 2026-09-18 Annual Report for Ilse Chakrabarti.pdf | 0.35 | 0.35 |
| board-minutes | yes | 2025-09-17 Minutes of the Regular Meeting of the Board of Directors for Beckett Halloran.pdf | 0.32 | 0.32 |
| court-hearing-notice | yes | 2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt | 0.34 | 0.34 |
| field-condition-report | yes | 2025-10-02 Loan Condition Report for Delphine Moncrieff.pdf | 0.52 | 0.52 |
| harvest-log | yes | 2026-10-02 Report for Esme Varga.xlsx | 0.15 | 0.15 |
| loss-notice-boxed-fields | yes | 2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf | 0.12 | 0.12 |
| mixed-signature-region | yes | 2026-08-21 First Amendment to Software License Agreement between Rosalind Achterberg and Barnaby Quist.pdf | 0.39 | 0.39 |
| property-policy-declarations-mid-60p | yes | 2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf | 0.04 | 0.04 |
| purchase-order | yes | 2026-01-29 Purchase Order from Juniper Okonkwo.pdf | 0.37 | 0.37 |
| quarterly-business-review | yes | 2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx | 0.08 | 0.08 |
| scan-remittance-advice-120dpi | yes | 2026-07-17 Remittance Advice - Sablewood Packaging Corp.png | 0.39 | 0.39 |
| scan-upside-down-po | yes | 2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf | 0.46 | 0.46 |
| watershed-monitoring-report-100p | yes | 2026-06-19 Annual Watershed Monitoring Report to Cobalt Basin Water Authority.pdf | 0.13 | 0.13 |
| ap-aging-report |  | 2026-08-31 Accounts Payable Aging Summary from Lowmarsh Farm Equipment Co.csv | 0.15 | 0.15 |
| asset-purchase-agreement-25p |  | 2026-03-30 Asset Purchase Agreement between Kingsfold Specialty Foods Inc and Bramblecrest Preserves LLC.pdf | 0.08 | 0.08 |
| assignment-assumption |  | 2026-06-01 Assignment and Assumption of Lease between Brindle Logistics Inc and Vantage Peak Freight LLC.pdf | 0.18 | 0.18 |
| benefits-change-checkbox-form |  | 2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf | 0.50 | 0.50 |
| capital-call-notice |  | 2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf | 0.04 | 0.04 |
| change-order-form |  | 2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf | 0.08 | 0.08 |
| credit-agreement-50p |  | 2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf | 0.02 | 0.02 |
| data-processing-agreement-10p |  | 2026-02-02 Data Processing Addendum between Thistledown Learning Inc and Skylark Analytics Ltd.pdf | 0.09 | 0.09 |
| declarations-interleaved |  | 2026-06-18 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf | 0.04 | 0.04 |
| industrial-lease-dated-in-schedule-25p |  | 2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf | 0.11 | 0.11 |
| inspection-log-ruled-2p |  | 2026-05-19 Fire Extinguisher Inspection Report from Emberwatch Fire Protection Inc.pdf | 0.13 | 0.13 |
| invoice-boxed-grid |  | 2026-07-09 Service Invoice from Whitlock & Sons Plumbing LLC.pdf | 0.06 | 0.06 |
| invoice-layout-only |  | 2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf | 0.03 | 0.03 |
| invoice-right-aligned |  | 2026-05-04 Invoice from Mossgiel Laboratory Services Inc.pdf | 0.01 | 0.01 |
| lease-two-column |  | 2026-05-18 Retail Lease between Oakhaven Retail Properties LLC and Sorrel & Thistle Tea House LLC.pdf | 0.15 | 0.15 |
| meeting-notice-columns |  | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf | 0.01 | 0.01 |
| meeting-notice-interleaved |  | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf | 0.01 | 0.01 |
| meeting-notice-reversed |  | 2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf | 0.01 | 0.01 |
| msa-effective-date-in-definitions-12p |  | 2026-03-02 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf | 0.09 | 0.09 |
| newsletter-three-column |  | 2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf | 0.01 | 0.01 |
| notice-of-default |  | 2025-10-14 Notice of Default and Reservation of Rights for Glasswing Ceramics LLC.pdf | 0.21 | 0.21 |
| notice-rent-increase |  | 2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf | 0.58 | 0.58 |
| offer-letter |  | 2026-02-10 Offer of Employment from Corvid Data Systems Inc.pdf | 0.04 | 0.04 |
| payroll-register |  | 2026-07-15 Payroll Register for Wrenfield Bakehouse LLC.xlsx | 0.01 | 0.01 |
| prior-authorization |  | 2026-03-09 Notice of Prior Authorization Approval to Florian Okonkwo.pdf | 0.23 | 0.23 |
| promissory-note |  | 2025-09-08 Promissory Note between Foxhallow Outfitters LLC and Ashgrove Capital Partners LLC.pdf | 0.06 | 0.06 |
| rate-confirmation-rotated |  | 2026-06-08 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf | 0.02 | 0.02 |
| scan-agreement-10p |  | 2025-11-10 Exclusive Distribution Agreement between Basalt Ridge Optics Inc and Saltash Point Scientific Supply LLC.pdf | 0.07 | 0.07 |
| scan-bill-of-lading |  | 2026-09-08 Straight Bill of Lading from Corriveau Millwork Supply Co.pdf | 0.04 | 0.04 |
| scan-certificate-of-insurance |  | 2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf | 0.04 | 0.04 |
| scan-clean-lease-2p |  | 2026-02-20 Residential Lease Agreement between Dahlquist Family Rentals LLC and Paloma Iwasaki.pdf | 0.16 | 0.16 |
| scan-fax-two-frames |  | 2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff | 0.02 | 0.02 |
| scan-lease-25p |  | 2026-04-22 Office Lease between Palisade Tower Partners LLC and Fennel Court Software Inc.pdf | 0.06 | 0.06 |
| scan-mixed-amendment |  | 2026-07-15 First Amendment to Distribution Agreement with Hollowell Kitchenware Inc.pdf | 0.10 | 0.10 |
| scan-mixed-middle-page |  | 2026-03-18 Lease Renewal Agreement between Ashcombe Commercial Properties LLC and Vellacott Optometry PC.pdf | 0.18 | 0.18 |
| scan-noisy-statement |  | 2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf | 0.47 | 0.47 |
| scan-patient-intake-form |  | 2026-08-27 New Patient Intake Form for Ione Kowalczyk.png | 0.40 | 0.40 |
| scan-rotated-90-invoice |  | 2026-04-14 Invoice from Wrenfield Restaurant Supply Co.png | 0.05 | 0.05 |
| scan-rotated-page-in-pdf |  | 2026-04-27 Notice of Freight Claim from Hollin Brook Furniture Gallery.pdf | 0.18 | 0.18 |
| scan-skewed-notice |  | 2026-08-03 Notice of Nonrenewal of Lease to Wendell Okafor.tiff | 0.51 | 0.51 |
| second-amendment |  | 2025-09-29 Second Amendment to Software License and Support Agreement with Umberlee Imaging Software Inc.pdf | 0.10 | 0.10 |
| separation-agreement |  | 2026-04-06 Separation Agreement and General Release between Mossgiel Biomedical Inc and Corinne Abernathy.docx | 0.06 | 0.06 |
| services-agreement |  | 2026-01-12 Master Services Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf | 0.05 | 0.05 |
| sow-under-msa-5p |  | 2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf | 0.01 | 0.01 |
| term-loan-parties-apart-40p |  | 2026-08-14 Term Loan Agreement between Oakhurst Mutual Savings Bank and Saltbrook Cold Storage Partners LP.pdf | 0.13 | 0.13 |
| vendor-registration-form |  | 2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf | 0.02 | 0.02 |

# InternBench: live run

- **Run:** 2026-11-21T04:54:30Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 47 documents · 47 completed · gold `24b6a6a3da`
- **Wall time:** 4380.99 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 20/42 (47.6%) |
| Document type | 37/47 (78.7%) |
| Date | 36/42 (85.7%) |
| Date role (when the reviewed date was chosen) | 19/36 (52.8%) |
| Parties | 34/47 (72.3%) |
| Relation word (when parties were named) | 32/42 (76.2%) |
| Party in the right role (when parties were named) | 28/35 (80.0%) |
| Ready / review routing | 8/29 (27.6%) |
| Description states nothing false | 47/47 (100.0%) |
| Description is specific | 33/47 (70.2%) |
| Description specificity | 61.7% (mean of 47) |
| Evidence recall (model's quotes) | 73.3% (mean of 43) |
| Digest recall (facts reaching the digest) | 88.4% (mean of 43) |
| Prompt recall (facts reaching the prompt sent) | 87.2% (mean of 43) |
| Sent to review | 95.7% of 47 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 74.5% | 47 |
| Review rate | 95.7% | 47 |
| Evidence recall | 73.3% | 43 |
| Total latency p50 | 82.71 s | 47 |
| Total latency p95 | 173.09 s | 47 |
| Generation latency p50 | 17.56 s | 47 |
| Generation latency p95 | 22.06 s | 47 |
| Generated tokens p50 | 86 | 47 |
| Generated tokens p95 | 102 | 47 |
| Prompt tokens p50 | 984 | 47 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 42 |
| Trap date chosen | 3 of 47 |
| Forbidden party named | 8 of 47 |
| Spurious parties named | 13 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 125 (0.0%) |
| Right name sent to review anyway | 10 of 23 |
| Sent to review for an unsupported fact | 35 of 47 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 1/2 | 2/2 | 2/5 | 3/3 | 100.0% | 113.04 s | 205.15 s |
| invoice | 4 | 2/4 | 2/4 | 3/4 | 1/2 | 75.0% | 70.18 s | 126.31 s |
| letter | 18 | 8/18 | 16/18 | 13/18 | 0/11 | 100.0% | 82.71 s | 169.90 s |
| memo | 3 | 0/2 | 2/2 | 0/3 | 1/2 | 100.0% | 48.53 s | 67.51 s |
| minutes | 6 | 3/6 | 6/6 | 6/6 | 1/5 | 83.3% | 148.52 s | 282.45 s |
| notice | 10 | 6/10 | 8/10 | 9/10 | 1/5 | 100.0% | 54.18 s | 154.03 s |
| statement | 1 | – | – | 1/1 | 1/1 | 100.0% | 27.19 s | 27.19 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 2/4 | 2/4 | 3/4 | 0/1 | 100.0% | 43.98 s | 66.90 s |
| native | 24 | 11/23 | 22/23 | 16/24 | 3/18 | 91.7% | 83.25 s | 173.09 s |
| office | 4 | 1/1 | 1/1 | 2/4 | 3/3 | 100.0% | 53.00 s | 143.55 s |
| scan | 15 | 6/14 | 11/14 | 13/15 | 2/7 | 100.0% | 84.69 s | 205.15 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 9/18 | 12/18 | 16/22 | 6/11 | 95.5% | 48.53 s | 85.54 s |
| 2-4 | 14 | 8/13 | 13/13 | 12/14 | 1/8 | 100.0% | 113.04 s | 205.15 s |
| 5-9 | 5 | 1/5 | 5/5 | 2/5 | 0/4 | 100.0% | 91.18 s | 156.53 s |
| 10-24 | 4 | 1/4 | 4/4 | 2/4 | 1/4 | 75.0% | 67.51 s | 173.09 s |
| 25-49 | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 148.52 s | 282.45 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 2/6 | 6/6 | 4/6 | 1/6 | 1 of 6 | 82.71 s | 282.45 s | 18.08 s | 22.60 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 2/3 | 2/3 | 2/3 | 0 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 0/1 | 0 |
| agency_guidance | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 0/1 | 1/1 | 0/1 | 1 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachment_with_own_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| attachments | 2 | 0/2 | 2/2 | 0/2 | 1 |
| attendance_list | 3 | 1/3 | 3/3 | 3/3 | 0 |
| attendee_list | 2 | 2/2 | 2/2 | 2/2 | 0 |
| australia | 6 | 3/5 | 4/5 | 6/6 | 1 |
| bilingual_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 2 | 0/2 | 0/2 | 1/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 1/4 | 2 |
| board_minutes | 4 | 2/4 | 4/4 | 4/4 | 0 |
| born_digital | 22 | 10/21 | 20/21 | 15/22 | 6 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 2/2 | 2/2 | 2/2 | 0 |
| canada | 2 | 2/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 35 | 18/35 | 31/35 | 27/35 | 7 |
| confirmation_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copier_searchable_pdf | 2 | 1/2 | 2/2 | 1/2 | 1 |
| corporate_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| council_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| counsel_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 1 | 1/1 | 1/1 | 1/1 | 0 |
| customer_first | 4 | 2/4 | 2/4 | 3/4 | 1 |
| date_at_end | 4 | 1/4 | 2/4 | 3/4 | 2 |
| date_at_foot | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_at_top | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_title | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_range | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| day_first_date | 4 | 2/3 | 2/3 | 4/4 | 2 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deadline_in_body | 6 | 3/6 | 6/6 | 4/6 | 2 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docket_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 1/1 | 1/1 | 2/4 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| enclosure | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 1/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 1/1 | 0/1 | 1 |
| file_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 0/1 | 0 |
| filled_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 3/4 | 3/4 | 5/5 | 1 |
| footnotes | 3 | 0/3 | 3/3 | 2/3 | 1 |
| form | 6 | 2/3 | 3/3 | 3/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form_revision_date | 2 | 1/1 | 1/1 | 1/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 2/3 | 3/3 | 2/5 | 0 |
| government_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_annotations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 2 | 0/2 | 1/2 | 2/2 | 0 |
| handwritten_dates | 2 | 1/2 | 2/2 | 1/2 | 0 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 1/4 | 1/4 | 2/4 | 1 |
| historical | 2 | 2/2 | 2/2 | 2/2 | 0 |
| image | 4 | 2/4 | 2/4 | 3/4 | 0 |
| image_only | 4 | 0/3 | 1/3 | 4/4 | 1 |
| image_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 2/4 | 2/4 | 4/5 | 2 |
| ireland | 2 | 1/2 | 2/2 | 1/2 | 0 |
| irregular_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 1/1 | 1/1 | 1/2 | 1 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 1/1 | 1/1 | 1/1 | 0 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 8/18 | 16/18 | 13/18 | 3 |
| letter_format | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_company | 3 | 2/3 | 2/3 | 2/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead | 8 | 4/8 | 7/8 | 6/8 | 1 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 2/2 | 0 |
| letterhead_logo | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| local_government | 1 | 0/1 | 1/1 | 1/1 | 0 |
| logo_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 2/4 | 4/4 | 4/4 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_addressees | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_companies_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_dates | 7 | 3/7 | 7/7 | 5/7 | 1 |
| many_named_organisations | 3 | 1/3 | 3/3 | 3/3 | 0 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 2/4 | 4/4 | 3/4 | 0 |
| memo | 3 | 0/2 | 2/2 | 0/3 | 3 |
| memorandum | 3 | 0/2 | 2/2 | 0/3 | 3 |
| metadata_date_trap | 14 | 6/10 | 8/10 | 11/14 | 3 |
| minutes | 6 | 3/6 | 6/6 | 6/6 | 0 |
| misprinted_year | 1 | 0/1 | 1/1 | 0/1 | 1 |
| mixed_scan_and_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 41 | 17/36 | 33/36 | 30/41 | 10 |
| month_only_date | 2 | 1/1 | 1/1 | 1/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 1/2 | 1/2 | 2/2 | 1 |
| named_officials | 8 | 2/7 | 6/7 | 5/8 | 3 |
| named_person_at_org | 8 | 5/8 | 8/8 | 7/8 | 1 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 3 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 2/3 | 3 |
| no_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_signatory | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 2 | 0/2 | 1/2 | 1/2 | 1 |
| no_title | 1 | – | – | 1/1 | 1 |
| noisy_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| nonprofit | 2 | 1/2 | 2/2 | 1/2 | 1 |
| notice | 10 | 6/10 | 8/10 | 9/10 | 2 |
| notice_of_decision | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 1/1 | 0/1 | 1 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 10 | 6/10 | 10/10 | 9/10 | 0 |
| office | 4 | 1/1 | 1/1 | 2/4 | 2 |
| open_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| opinion_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 1/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 0/1 | 1/1 | 1/3 | 3 |
| printed_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_notice | 6 | 3/6 | 4/6 | 5/6 | 2 |
| purchase_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 1 |
| received_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 3 | 3/3 | 3/3 | 3/3 | 0 |
| reference_number | 9 | 5/9 | 9/9 | 5/9 | 3 |
| reference_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 5 | 2/5 | 5/5 | 4/5 | 1 |
| regulator | 4 | 2/4 | 4/4 | 3/4 | 0 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 1/1 | 0 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 1/1 | 1/1 | 1/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 1/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 1/1 | 1 |
| running_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 15 | 6/14 | 11/14 | 13/15 | 2 |
| scanned_signature_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| security_marking | 1 | – | – | 1/1 | 1 |
| senate_minutes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 2/2 | 2/2 | 3/3 | 1 |
| short | 8 | 1/7 | 4/7 | 6/8 | 4 |
| signatory_trap | 2 | 1/2 | 1/2 | 2/2 | 1 |
| signatory_vs_org | 3 | 1/3 | 3/3 | 3/3 | 0 |
| signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 2/5 | 5/5 | 4/5 | 1 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 1/1 | 1/1 | 1/1 | 0 |
| source_title_mismatch | 3 | 2/3 | 3/3 | 3/3 | 0 |
| south_africa | 3 | 1/2 | 1/2 | 2/3 | 1 |
| staff_names | 7 | 5/7 | 7/7 | 7/7 | 0 |
| stamp_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| stamped_date | 2 | 0/2 | 1/2 | 0/2 | 1 |
| statement | 1 | – | – | 1/1 | 1 |
| statute_date | 1 | – | – | 0/1 | 0 |
| statutory_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| successor_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| table | 2 | 1/1 | 1/1 | 2/2 | 1 |
| table_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| table_of_contents | 2 | 1/2 | 2/2 | 2/2 | 0 |
| tables | 3 | 0/2 | 2/2 | 2/3 | 1 |
| tax_invoice | 2 | 2/2 | 2/2 | 2/2 | 0 |
| template_attachment | 1 | 0/1 | 1/1 | 0/1 | 1 |
| third_party_named_in_body | 1 | 0/1 | 1/1 | 1/1 | 0 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 1/2 | 1/2 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_day_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 2 | 1/1 | 1/1 | 2/2 | 1 |
| two_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 2/2 | 1 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 1/2 | 1/2 | 1/2 | 1 |
| university | 4 | 1/3 | 3/3 | 1/4 | 3 |
| university_foundation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_date_format | 4 | 2/4 | 4/4 | 3/4 | 0 |
| us_federal | 10 | 5/9 | 9/9 | 8/10 | 1 |
| us_local | 6 | 3/5 | 4/5 | 6/6 | 1 |
| us_state | 7 | 2/7 | 7/7 | 4/7 | 2 |
| warn_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| withdrawn | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 134 · layout 32 · ocr 29 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 12 · ocr 20 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 47 | 82.71 s | 173.09 s | 282.45 s |
| `extraction_wall_ms` | 47 | 577.0 ms | 4.67 s | 13.48 s |
| `worker_total_ms` | 47 | 576.7 ms | 4.67 s | 13.48 s |
| `worker_snapshot_ms` | 47 | 0.33 ms | 0.61 ms | 1.0 ms |
| `worker_parse_ms` | 47 | 6.5 ms | 61.4 ms | 631.7 ms |
| `worker_analysis_ms` | 47 | 0.72 ms | 5.5 ms | 7.5 ms |
| `worker_render_ms` | 47 | 0.00 ms | 593.2 ms | 1.52 s |
| `worker_image_decode_ms` | 47 | 0.00 ms | 23.6 ms | 55.7 ms |
| `worker_ocr_ms` | 47 | 0.00 ms | 7.46 s | 25.49 s |
| `worker_ocr_encode_ms` | 47 | 0.00 ms | 500.0 ms | 1.15 s |
| `worker_ocr_engine_ms` | 47 | 0.00 ms | 6.69 s | 24.02 s |
| `worker_vision_ms` | 47 | 0.00 ms | 10.3 ms | 17.7 ms |
| `analyze_wall_ms` | 47 | 80.58 s | 170.47 s | 282.39 s |
| `distill_ms` | 47 | 2.4 ms | 29.3 ms | 44.0 ms |
| `prompt_ms` | 47 | 0.03 ms | 0.07 ms | 0.07 ms |
| `inference_ms` | 47 | 80.57 s | 170.43 s | 282.32 s |
| `prefill_ms` | 47 | 61.73 s | 154.71 s | 259.70 s |
| `generation_ms` | 47 | 17.56 s | 22.06 s | 23.22 s |
| `validation_ms` | 47 | 6.5 ms | 21.1 ms | 30.0 ms |
| `naming_ms` | 47 | 0.23 ms | 3.4 ms | 5.4 ms |
| `index_ms` | 47 | 2.1 ms | 27.1 ms | 40.3 ms |
| `retrieval_ms` | 47 | 0.04 ms | 2.3 ms | 3.2 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 47 | 15.9 tok/s | 16.0 tok/s | 16.1 tok/s |
| `generation_tok_per_s` | 47 | 4.8 tok/s | 5.0 tok/s | 5.0 tok/s |
| `prompt_tokens` | 47 | 984 | 2454 | 4079 |
| `cached_tokens` | 47 | 477 | 477 | 477 |
| `generated_tokens` | 47 | 86 | 102 | 109 |
| `estimated_prompt_tokens` | 47 | 1652 | 3240 | 3365 |
| `model_requests` | 47 | 1 | 1 | 1 |
| `redistillations` | 47 | 0 | 0 | 0 |
| `source_characters` | 47 | 2961 | 48744 | 68987 |
| `digest_characters` | 47 | 3125 | 7921 | 8432 |
| `prompt_characters` | 47 | 5174 | 9976 | 10487 |
| `worker_ocr_pages` | 47 | 0 | 2 | 8 |
| `worker_ocr_passes` | 47 | 0 | 2 | 8 |
| `worker_orientation_passes` | 47 | 0 | 2 | 8 |
| `worker_rendered_pixels` | 47 | 0 | 16830000 | 67320000 |
| `index_units` | 47 | 32 | 284 | 347 |
| `context_units` | 47 | 21 | 79 | 90 |
| `context_tokens` | 47 | 1064 | 2650 | 2777 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 12 | 5.4 ms | 72.2 ms | 5.0 ms | 70.9 ms | 61.4 ms | 6.0 ms | 0.00 ms | 0.00 ms |
| layout | 12 | 10.7 ms | 639.6 ms | 10.2 ms | 638.3 ms | 631.7 ms | 5.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.32 s | 2.42 s | 1.32 s | 2.42 s | 3.5 ms | 0.91 ms | 150.6 ms | 2.21 s |
| ocr | 20 | 2.36 s | 6.28 s | 2.36 s | 6.28 s | 36.1 ms | 3.6 ms | 928.5 ms | 11.37 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.32 s | 3.83 s | 1.32 s | 3.83 s | 28.5 ms | 0.91 ms | 593.2 ms | 3.79 s |
| 2-4 | 14 | 13.7 ms | 6.28 s | 13.2 ms | 6.28 s | 31.3 ms | 2.9 ms | 334.9 ms | 11.37 s |
| 5-9 | 5 | 20.9 ms | 13.48 s | 20.4 ms | 13.48 s | 631.7 ms | 5.1 ms | 1.52 s | 25.49 s |
| 10-24 | 4 | 27.2 ms | 2.62 s | 26.2 ms | 2.62 s | 61.4 ms | 7.5 ms | 423.8 ms | 4.35 s |
| 25-49 | 2 | 59.9 ms | 60.5 ms | 58.3 ms | 58.8 ms | 51.5 ms | 6.0 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 6.6 ms | 13.48 s | 6.1 ms | 13.48 s | 67.2 ms | 3.6 ms | 1.52 s | 25.49 s |
| invoice | 4 | 1.67 s | 2.36 s | 1.67 s | 2.36 s | 12.4 ms | 1.3 ms | 593.2 ms | 2.04 s |
| letter | 18 | 639.6 ms | 6.28 s | 638.3 ms | 6.28 s | 631.7 ms | 5.1 ms | 928.5 ms | 11.37 s |
| memo | 3 | 4.1 ms | 18.2 ms | 3.8 ms | 17.4 ms | 14.2 ms | 2.1 ms | 0.00 ms | 0.00 ms |
| minutes | 6 | 59.9 ms | 2.62 s | 58.3 ms | 2.62 s | 51.5 ms | 7.5 ms | 423.8 ms | 4.35 s |
| notice | 10 | 577.0 ms | 4.11 s | 576.7 ms | 4.11 s | 29.0 ms | 1.2 ms | 254.0 ms | 3.89 s |
| statement | 1 | 922.7 ms | 922.7 ms | 922.4 ms | 922.4 ms | 0.74 ms | 0.15 ms | 131.9 ms | 776.3 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 443 MB | 1254 MB | real6-form-03 |
| Model server | 6078 MB | 6085 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (22)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter from The Marine Mammal Commission.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-01 Letter to Nevada Nuclear Waste Task Force, Incorporated.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-04**: expected `2017-11-14 Letter to The President.pdf`, got `2017-11-14 Letter from The President.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Opinion to Matthew E. Minick.pdf` · trap: party Matthew E. Minick (addressee, the county attorney as a person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for IF Leco Corporation.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Javier Martínez.pdf` · trap: party Javier Martínez (Speaker of the House, addressed with the Members on the House's behalf) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to CEO.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter to His Excellency General The Hon Sir Peter Cosgrove AK MC (Retd) Governor-General.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document - Reh Docplals.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter Tech Companies Must Resist Unlawful Subpoenas for User Data.pdf` · trap: party Amazon (one of the seven companies, 'and others', the open letter is sent to; too many to name one) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document - National Capital Planning Commission.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Document from Conway Board of Health.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2024-03-13 Public Notice of Special Meeting from Maggie Thompson.pdf` · trap: party Maggie Thompson (Assistant Secretary who signs) · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document from Meade Brothers.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `2026-04-06 Delivery Order from WEST Department of Administration.pdf` · review: LOW_CONFIDENCE
- **real6-memo-01**: expected `2004-06-01 Memorandum from Office of Government Ethics.pdf`, got `2004-06-01 Memorandum to Designated Agency Ethics Officials.pdf` · trap: party Designated Agency Ethics Officials (class of recipients named in the title) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order from Port of Vancouver USA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Record of the Regular Meeting of the.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-07-16 Confirmed Minutes Confirmed Minutes from Shire of Woodanilling.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

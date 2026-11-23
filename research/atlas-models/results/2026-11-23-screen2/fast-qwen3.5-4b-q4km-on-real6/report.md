# InternBench: live run

- **Run:** 2026-11-23T16:47:42Z · commit `70ce2164a1`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 2672.17 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 18/45 (40.0%) |
| Document type | 40/50 (80.0%) |
| Date | 38/45 (84.4%) |
| Date role (when the reviewed date was chosen) | 20/38 (52.6%) |
| Parties | 35/50 (70.0%) |
| Relation word (when parties were named) | 30/45 (66.7%) |
| Party in the right role (when parties were named) | 27/38 (71.0%) |
| Ready / review routing | 10/30 (33.3%) |
| Description states nothing false | 49/50 (98.0%) |
| Description is specific | 35/50 (70.0%) |
| Description specificity | 62.7% (mean of 50) |
| Evidence recall (model's quotes) | 75.0% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 88.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 62.0% | 50 |
| Review rate | 88.0% | 50 |
| Evidence recall | 75.0% | 46 |
| Total latency p50 | 47.30 s | 50 |
| Total latency p95 | 99.60 s | 50 |
| Generation latency p50 | 10.26 s | 50 |
| Generation latency p95 | 14.09 s | 50 |
| Generated tokens p50 | 85 | 50 |
| Generated tokens p95 | 112 | 50 |
| Prompt tokens p50 | 1003 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 4 of 45 |
| Trap date chosen | 4 of 50 |
| Forbidden party named | 8 of 50 |
| Spurious parties named | 14 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 151 (0.7%) |
| Right name sent to review anyway | 10 of 24 |
| Sent to review for an unsupported fact | 31 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 2/2 | 4/5 | 3/3 | 100.0% | 64.78 s | 115.40 s |
| invoice | 4 | 2/4 | 2/4 | 3/4 | 0/2 | 100.0% | 40.54 s | 70.95 s |
| letter | 18 | 9/18 | 16/18 | 13/18 | 2/11 | 88.9% | 46.96 s | 95.64 s |
| memo | 5 | 0/4 | 4/4 | 2/5 | 1/3 | 80.0% | 36.75 s | 53.83 s |
| minutes | 6 | 4/6 | 6/6 | 4/6 | 2/5 | 66.7% | 83.25 s | 161.68 s |
| notice | 11 | 3/11 | 8/11 | 9/11 | 1/5 | 90.9% | 33.95 s | 87.42 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 19.66 s | 19.66 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 1/4 | 2/4 | 2/4 | 0/1 | 100.0% | 26.10 s | 44.09 s |
| native | 25 | 9/24 | 22/24 | 17/25 | 4/19 | 84.0% | 48.02 s | 99.60 s |
| office | 4 | 1/1 | 1/1 | 2/4 | 3/3 | 100.0% | 29.64 s | 80.65 s |
| scan | 17 | 7/16 | 13/16 | 14/17 | 3/7 | 88.2% | 49.96 s | 115.40 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 7/18 | 11/18 | 15/22 | 5/11 | 95.5% | 29.05 s | 49.09 s |
| 2-4 | 16 | 7/15 | 15/15 | 12/16 | 2/9 | 93.8% | 64.58 s | 115.40 s |
| 5-9 | 6 | 2/6 | 6/6 | 5/6 | 1/4 | 66.7% | 52.49 s | 97.58 s |
| 10-24 | 4 | 1/4 | 4/4 | 2/4 | 1/4 | 75.0% | 36.75 s | 99.60 s |
| 25-49 | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 50.0% | 83.25 s | 161.68 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 2/6 | 6/6 | 3/6 | 2/6 | 2 of 6 | 46.96 s | 161.68 s | 10.02 s | 14.80 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 2/3 | 2/3 | 2/3 | 0 |
| addressee_is_office_holder | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 1/1 | 0 |
| agency_enforcement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_guidance | 2 | 0/2 | 2/2 | 1/2 | 0 |
| agency_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 0/1 | 1/1 | 1/1 | 0 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachment_with_own_date | 2 | 1/2 | 2/2 | 2/2 | 0 |
| attachments | 3 | 0/3 | 3/3 | 3/3 | 0 |
| attendance_list | 3 | 1/3 | 3/3 | 1/3 | 0 |
| attendee_list | 2 | 2/2 | 2/2 | 2/2 | 0 |
| australia | 6 | 3/5 | 4/5 | 4/6 | 2 |
| bilingual_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 2 | 0/2 | 0/2 | 1/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 2 |
| board_minutes | 4 | 3/4 | 4/4 | 3/4 | 0 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 23 | 8/22 | 21/22 | 16/23 | 4 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| canada | 2 | 1/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 38 | 15/38 | 33/38 | 28/38 | 7 |
| confirmation_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| copier_searchable_pdf | 2 | 1/2 | 1/2 | 1/2 | 2 |
| corporate_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| council_minutes | 1 | 0/1 | 1/1 | 0/1 | 0 |
| counsel_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| cover_sheet | 2 | 1/2 | 2/2 | 2/2 | 0 |
| customer_first | 4 | 2/4 | 2/4 | 3/4 | 1 |
| date_at_end | 5 | 1/5 | 3/5 | 5/5 | 1 |
| date_at_foot | 1 | 0/1 | 0/1 | 0/1 | 2 |
| date_at_top | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_range | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| day_first_date | 4 | 2/3 | 2/3 | 3/4 | 3 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deadline_in_body | 7 | 0/7 | 7/7 | 3/7 | 3 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docket_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 1/1 | 1/1 | 2/4 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 1/3 | 3/3 | 2/3 | 0 |
| enclosure | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 1/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 0/1 | 2 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 1/1 | 0 |
| filled_form | 1 | 0/1 | 1/1 | 1/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 3/4 | 3/4 | 4/5 | 2 |
| footnotes | 3 | 1/3 | 3/3 | 3/3 | 0 |
| form | 6 | 1/3 | 3/3 | 5/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form_revision_date | 2 | 0/1 | 1/1 | 2/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 1/3 | 3/3 | 4/5 | 0 |
| government_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_annotations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 2/3 | 2/3 | 0 |
| handwritten_dates | 2 | 1/2 | 2/2 | 2/2 | 0 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 1/4 | 1/4 | 2/4 | 1 |
| historical | 2 | 2/2 | 2/2 | 2/2 | 0 |
| image | 4 | 1/4 | 2/4 | 2/4 | 1 |
| image_only | 4 | 0/3 | 1/3 | 3/4 | 2 |
| image_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 2/4 | 2/4 | 4/5 | 2 |
| ireland | 2 | 1/2 | 2/2 | 1/2 | 0 |
| irregular_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 1/1 | 1/1 | 1/2 | 1 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 1/1 | 0/1 | 1 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 9/18 | 16/18 | 13/18 | 2 |
| letter_format | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_from_company | 3 | 2/3 | 2/3 | 2/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead | 9 | 4/9 | 8/9 | 6/9 | 1 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 2/2 | 0 |
| letterhead_logo | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 2 | 1/2 | 2/2 | 2/2 | 0 |
| local_government | 1 | 0/1 | 1/1 | 0/1 | 0 |
| logo_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 2/4 | 4/4 | 2/4 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_dates | 8 | 2/8 | 8/8 | 5/8 | 0 |
| many_named_organisations | 3 | 2/3 | 3/3 | 3/3 | 0 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 2/4 | 4/4 | 3/4 | 0 |
| memo | 5 | 0/4 | 4/4 | 2/5 | 2 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 0/4 | 4/4 | 2/5 | 2 |
| metadata_date_trap | 14 | 5/10 | 8/10 | 9/14 | 4 |
| minutes | 6 | 4/6 | 6/6 | 4/6 | 0 |
| misprinted_year | 1 | 0/1 | 0/1 | 0/1 | 2 |
| mixed_scan_and_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 44 | 15/39 | 35/39 | 31/44 | 11 |
| month_only_date | 2 | 1/1 | 1/1 | 2/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 0/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 0/2 | 1/2 | 1/2 | 2 |
| named_officials | 9 | 3/8 | 7/8 | 5/9 | 3 |
| named_person_at_org | 9 | 4/9 | 9/9 | 7/9 | 1 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 4 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 1/3 | 4 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_signatory | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 2 | 0/2 | 1/2 | 1/2 | 1 |
| no_title | 1 | – | – | 0/1 | 2 |
| noisy_ocr_layer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| nonprofit | 2 | 1/2 | 2/2 | 2/2 | 0 |
| notice | 11 | 3/11 | 8/11 | 9/11 | 4 |
| notice_of_decision | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 0/1 | 2 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 7/12 | 12/12 | 11/12 | 0 |
| office | 4 | 1/1 | 1/1 | 2/4 | 2 |
| open_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| opinion_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 1/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 0/1 | 1/1 | 2/3 | 2 |
| printed_form | 1 | 0/1 | 1/1 | 1/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_notice | 6 | 1/6 | 3/6 | 4/6 | 4 |
| purchase_order | 1 | 0/1 | 1/1 | 1/1 | 0 |
| re_line_references_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| received_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 3 | 3/3 | 3/3 | 3/3 | 0 |
| reference_number | 11 | 4/11 | 11/11 | 8/11 | 3 |
| reference_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 3/6 | 6/6 | 5/6 | 0 |
| regulator | 5 | 1/5 | 5/5 | 3/5 | 0 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 1/1 | 0 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 1/1 | 1/1 | 1/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 0/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 2 |
| running_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 17 | 7/16 | 13/16 | 14/17 | 3 |
| scanned_signature_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| security_marking | 1 | – | – | 0/1 | 2 |
| senate_minutes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 0/2 | 2/2 | 2/3 | 2 |
| short | 8 | 1/7 | 3/7 | 5/8 | 6 |
| signatory_trap | 2 | 0/2 | 1/2 | 1/2 | 2 |
| signatory_vs_org | 3 | 2/3 | 3/3 | 3/3 | 0 |
| signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 3/5 | 4/5 | 3/5 | 2 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 1/1 | 1/1 | 0 |
| source_title_mismatch | 3 | 2/3 | 3/3 | 3/3 | 0 |
| south_africa | 3 | 0/2 | 1/2 | 1/3 | 2 |
| staff_names | 8 | 4/8 | 8/8 | 5/8 | 1 |
| stamp_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| stamped_date | 3 | 0/3 | 2/3 | 2/3 | 1 |
| statement | 1 | – | – | 0/1 | 2 |
| statute_date | 1 | – | – | 1/1 | 0 |
| statutory_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| successor_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| table | 2 | 0/1 | 1/1 | 1/2 | 2 |
| table_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| table_of_contents | 2 | 1/2 | 2/2 | 1/2 | 0 |
| tables | 3 | 0/2 | 2/2 | 2/3 | 1 |
| tax_invoice | 2 | 2/2 | 2/2 | 2/2 | 0 |
| template_attachment | 1 | 0/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 2/3 | 0 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 1/2 | 2/2 | 0 |
| transmittal_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_day_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 2 | 1/1 | 1/1 | 1/2 | 2 |
| two_senders | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 1/1 | 1/1 | 1/2 | 2 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 1/2 | 1/2 | 1/2 | 1 |
| university | 4 | 1/3 | 2/3 | 1/4 | 4 |
| university_foundation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_date_format | 4 | 2/4 | 4/4 | 3/4 | 0 |
| us_federal | 12 | 5/11 | 11/11 | 11/12 | 0 |
| us_local | 6 | 4/5 | 4/5 | 6/6 | 1 |
| us_state | 8 | 1/8 | 8/8 | 4/8 | 2 |
| warn_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| withdrawn | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 47.30 s | 99.60 s | 161.68 s |
| `extraction_wall_ms` | 50 | 586.6 ms | 4.87 s | 14.42 s |
| `worker_total_ms` | 50 | 586.2 ms | 4.87 s | 14.42 s |
| `worker_snapshot_ms` | 50 | 0.33 ms | 0.63 ms | 1.2 ms |
| `worker_parse_ms` | 50 | 6.4 ms | 64.5 ms | 960.9 ms |
| `worker_analysis_ms` | 50 | 0.75 ms | 5.7 ms | 7.6 ms |
| `worker_render_ms` | 50 | 0.00 ms | 587.5 ms | 1.51 s |
| `worker_image_decode_ms` | 50 | 0.00 ms | 30.7 ms | 56.4 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 7.84 s | 27.12 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 516.6 ms | 990.3 ms |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 7.06 s | 25.77 s |
| `worker_vision_ms` | 50 | 0.00 ms | 10.3 ms | 17.6 ms |
| `analyze_wall_ms` | 50 | 46.88 s | 96.68 s | 161.62 s |
| `distill_ms` | 50 | 2.5 ms | 29.5 ms | 44.1 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.07 ms | 0.07 ms |
| `inference_ms` | 50 | 46.83 s | 96.64 s | 161.55 s |
| `prefill_ms` | 50 | 34.90 s | 86.61 s | 146.74 s |
| `generation_ms` | 50 | 10.26 s | 14.09 s | 15.06 s |
| `validation_ms` | 50 | 6.3 ms | 25.2 ms | 28.6 ms |
| `naming_ms` | 50 | 0.26 ms | 3.4 ms | 5.4 ms |
| `index_ms` | 50 | 2.2 ms | 27.2 ms | 41.1 ms |
| `retrieval_ms` | 50 | 0.04 ms | 2.3 ms | 3.3 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 28.6 tok/s | 29.1 tok/s | 29.4 tok/s |
| `generation_tok_per_s` | 50 | 8.3 tok/s | 8.6 tok/s | 8.6 tok/s |
| `prompt_tokens` | 50 | 1003 | 2454 | 4079 |
| `cached_tokens` | 50 | 477 | 477 | 477 |
| `generated_tokens` | 50 | 85 | 112 | 119 |
| `estimated_prompt_tokens` | 50 | 1669 | 3240 | 3365 |
| `model_requests` | 50 | 1 | 1 | 1 |
| `redistillations` | 50 | 0 | 0 | 0 |
| `source_characters` | 50 | 2970 | 48744 | 68987 |
| `digest_characters` | 50 | 3190 | 7921 | 8432 |
| `prompt_characters` | 50 | 5239 | 9976 | 10487 |
| `worker_ocr_pages` | 50 | 0 | 2 | 8 |
| `worker_ocr_passes` | 50 | 0 | 2 | 8 |
| `worker_orientation_passes` | 50 | 0 | 2 | 8 |
| `worker_rendered_pixels` | 50 | 0 | 16846830 | 67320000 |
| `index_units` | 50 | 32 | 284 | 347 |
| `context_units` | 50 | 23 | 79 | 90 |
| `context_tokens` | 50 | 1081 | 2650 | 2777 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 12 | 5.6 ms | 75.8 ms | 5.2 ms | 74.5 ms | 64.5 ms | 6.0 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 10.6 ms | 968.7 ms | 10.0 ms | 967.4 ms | 960.9 ms | 5.7 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.38 s | 2.38 s | 1.38 s | 2.38 s | 3.8 ms | 1.0 ms | 144.4 ms | 2.17 s |
| ocr | 22 | 2.35 s | 6.78 s | 2.35 s | 6.78 s | 36.5 ms | 3.7 ms | 919.6 ms | 12.06 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.38 s | 3.94 s | 1.38 s | 3.94 s | 24.9 ms | 1.0 ms | 587.5 ms | 3.90 s |
| 2-4 | 16 | 13.7 ms | 6.78 s | 13.2 ms | 6.78 s | 30.8 ms | 2.8 ms | 334.7 ms | 12.06 s |
| 5-9 | 6 | 21.0 ms | 14.42 s | 20.3 ms | 14.42 s | 960.9 ms | 5.1 ms | 1.51 s | 27.12 s |
| 10-24 | 4 | 27.6 ms | 2.92 s | 26.6 ms | 2.92 s | 64.5 ms | 7.6 ms | 422.4 ms | 4.81 s |
| 25-49 | 2 | 59.8 ms | 67.3 ms | 58.1 ms | 65.6 ms | 57.9 ms | 6.0 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 6.6 ms | 14.42 s | 6.2 ms | 14.42 s | 66.2 ms | 3.7 ms | 1.51 s | 27.12 s |
| invoice | 4 | 1.67 s | 2.35 s | 1.67 s | 2.35 s | 14.2 ms | 1.1 ms | 587.5 ms | 2.11 s |
| letter | 18 | 968.7 ms | 6.78 s | 967.4 ms | 6.78 s | 960.9 ms | 5.1 ms | 919.6 ms | 12.06 s |
| memo | 5 | 8.3 ms | 4.75 s | 7.9 ms | 4.75 s | 13.6 ms | 2.1 ms | 233.4 ms | 6.98 s |
| minutes | 6 | 59.8 ms | 2.92 s | 58.1 ms | 2.92 s | 57.9 ms | 7.6 ms | 422.4 ms | 4.81 s |
| notice | 11 | 1.19 s | 4.19 s | 1.19 s | 4.19 s | 25.5 ms | 2.0 ms | 259.0 ms | 3.98 s |
| statement | 1 | 937.3 ms | 937.3 ms | 937.0 ms | 937.0 ms | 0.80 ms | 0.13 ms | 128.5 ms | 789.9 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 465 MB | 1251 MB | real6-form-03 |
| Model server | 3265 MB | 3266 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (27)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter from The Marine Mammal Commission.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-01 Letter to The Members of the Nuclear Waste Technical Review Board.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Opinion from KEN PAXTON.pdf` · trap: party Ken Paxton (Attorney General who signs, a person) · **filed without review**
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice from State of Michigan.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Javier Martínez.pdf` · trap: party Javier Martínez (Speaker of the House, addressed with the Members on the House's behalf) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Industry Letter to CEO.pdf` · **filed without review**
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter for John A. Fraser.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter Tech Companies Must Resist Unlawful Subpoenas for User Data to ACLU of Northern California.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-03**: expected `2026-06-04 Notice of Exemption for Northeast Texas Railway Company.pdf`, got `2026-06-04 Verified Notice of Exemption for Northeast Texas Railway Company.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document from National Park Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Public Hearing Notice from Conway Board of Health.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-08**: expected `2025-08-20 Revised Notice of Public Hearing and Participant Funding for NexGen Energy Ltd.pdf`, got `2025-08-20 Revised Notice of Public Hearing for The Canadian Nuclear Safety Commission.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `2026-06-16 Public Notice for MAGODI.jpg` · trap: party P Magodi (Municipal Manager in whose name it is signed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-09-03 Rapid Response from Perdue Farms.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting for Maggie Thompson.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces); party Maggie Thompson (Assistant Secretary who signs) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document - To M. A. MEADE.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `2019-12-23 Federal Financial Report from US Election Assistance Commission.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `2026-04-06 Delivery Order from WEST Department of Administration State of West Virginia.pdf` · review: TYPE_INFERRED, LOW_CONFIDENCE
- **real6-memo-01**: expected `2004-06-01 Memorandum from Office of Government Ethics.pdf`, got `2004-06-01 Memorandum to Designated Agency Ethics Officials from Office of Government Ethics.pdf` · review: LOW_CONFIDENCE
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Duke Energy Florida, LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · **filed without review**
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Board of Regents Meeting Record for President Bogost.pdf` · **filed without review**
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-07-16 Confirmed Minutes Confirmed Minutes for Ordinary Council.pdf` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

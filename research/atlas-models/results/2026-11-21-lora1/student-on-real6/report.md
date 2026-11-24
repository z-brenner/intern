# InternBench: live run

- **Run:** 2026-11-21T09:37:46Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 9V74 80-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `99e3ad6efd92`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 366.36 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 11/45 (24.4%) |
| Document type | 32/50 (64.0%) |
| Date | 34/45 (75.6%) |
| Date role (when the reviewed date was chosen) | 26/33 (78.8%) |
| Parties | 32/50 (64.0%) |
| Relation word (when parties were named) | 26/44 (59.1%) |
| Party in the right role (when parties were named) | 22/37 (59.5%) |
| Ready / review routing | 6/30 (20.0%) |
| Description states nothing false | 50/50 (100.0%) |
| Description is specific | 27/50 (54.0%) |
| Description specificity | 53.3% (mean of 50) |
| Evidence recall (model's quotes) | 54.4% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 98.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 80.0% | 50 |
| Review rate | 98.0% | 50 |
| Evidence recall | 54.4% | 46 |
| Total latency p50 | 6.02 s | 50 |
| Total latency p95 | 16.58 s | 50 |
| Generation latency p50 | 1.82 s | 50 |
| Generation latency p95 | 2.63 s | 50 |
| Generated tokens p50 | 75 | 50 |
| Generated tokens p95 | 104 | 50 |
| Prompt tokens p50 | 1003 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 45 |
| Trap date chosen | 4 of 50 |
| Forbidden party named | 6 of 50 |
| Spurious parties named | 16 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 113 (0.0%) |
| Right name sent to review anyway | 8 of 24 |
| Sent to review for an unsupported fact | 40 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 1/2 | 2/5 | 3/3 | 100.0% | 7.04 s | 20.26 s |
| invoice | 4 | 1/4 | 2/4 | 2/4 | 0/2 | 100.0% | 4.69 s | 9.38 s |
| letter | 18 | 5/18 | 14/18 | 11/18 | 0/11 | 100.0% | 5.69 s | 12.70 s |
| memo | 5 | 1/4 | 4/4 | 3/5 | 1/3 | 100.0% | 4.59 s | 9.60 s |
| minutes | 6 | 1/6 | 6/6 | 5/6 | 0/5 | 100.0% | 10.10 s | 18.55 s |
| notice | 11 | 3/11 | 7/11 | 9/11 | 1/5 | 90.9% | 6.12 s | 9.55 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 3.43 s | 3.43 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 0/4 | 2/4 | 1/4 | 0/1 | 100.0% | 4.69 s | 7.97 s |
| native | 25 | 8/24 | 21/24 | 15/25 | 1/19 | 100.0% | 5.67 s | 13.63 s |
| office | 4 | 0/1 | 1/1 | 3/4 | 3/3 | 100.0% | 3.59 s | 8.87 s |
| scan | 17 | 3/16 | 10/16 | 13/17 | 2/7 | 94.1% | 7.49 s | 20.26 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3/18 | 10/18 | 14/22 | 5/11 | 100.0% | 4.37 s | 7.97 s |
| 2-4 | 16 | 3/15 | 14/15 | 10/16 | 1/9 | 100.0% | 8.87 s | 16.58 s |
| 5-9 | 6 | 3/6 | 4/6 | 4/6 | 0/4 | 83.3% | 6.12 s | 20.26 s |
| 10-24 | 4 | 2/4 | 4/4 | 3/4 | 0/4 | 100.0% | 4.59 s | 13.63 s |
| 25-49 | 2 | 0/2 | 2/2 | 1/2 | 0/2 | 100.0% | 10.10 s | 18.55 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 2/6 | 6/6 | 4/6 | 0/6 | 0 of 6 | 6.02 s | 18.55 s | 2.49 s | 2.64 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 0/3 | 2/3 | 1/3 | 0 |
| addressee_is_office_holder | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 0/1 | 0 |
| agency_enforcement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_guidance | 2 | 1/2 | 2/2 | 1/2 | 0 |
| agency_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annotation_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 1/1 | 0 |
| attached_order_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachment_with_own_date | 2 | 1/2 | 2/2 | 1/2 | 0 |
| attachments | 3 | 1/3 | 2/3 | 3/3 | 0 |
| attendance_list | 3 | 0/3 | 3/3 | 2/3 | 1 |
| attendee_list | 2 | 1/2 | 2/2 | 2/2 | 0 |
| australia | 6 | 1/5 | 4/5 | 4/6 | 1 |
| bilingual_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 2 | 0/2 | 0/2 | 1/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 0 |
| board_minutes | 4 | 1/4 | 4/4 | 3/4 | 1 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 23 | 8/22 | 20/22 | 13/23 | 5 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 0/2 | 2/2 | 1/2 | 0 |
| canada | 2 | 2/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 0/1 | 1/1 | 0/1 | 1 |
| cc_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 38 | 9/38 | 29/38 | 25/38 | 8 |
| confirmation_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copier_searchable_pdf | 2 | 0/2 | 1/2 | 2/2 | 1 |
| corporate_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| council_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| counsel_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 2/2 | 2/2 | 0 |
| customer_first | 4 | 1/4 | 2/4 | 2/4 | 1 |
| date_at_end | 5 | 1/5 | 2/5 | 3/5 | 2 |
| date_at_foot | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_at_top | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_range | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| day_first_date | 4 | 1/3 | 2/3 | 2/4 | 2 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 1 |
| deadline_in_body | 7 | 1/7 | 7/7 | 3/7 | 2 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docket_number | 1 | 0/1 | 1/1 | 0/1 | 1 |
| docx | 4 | 0/1 | 1/1 | 3/4 | 0 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 0/1 | 1 |
| effective_dates | 3 | 1/3 | 3/3 | 2/3 | 0 |
| enclosure | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 0/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 1/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 1/1 | 1 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 0/1 | 0 |
| filled_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 1/4 | 3/4 | 3/5 | 1 |
| footnotes | 3 | 1/3 | 2/3 | 1/3 | 1 |
| form | 6 | 0/3 | 1/3 | 3/6 | 0 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form_revision_date | 2 | 0/1 | 1/1 | 0/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 0/3 | 1/3 | 2/5 | 0 |
| government_sender | 1 | 0/1 | 1/1 | 0/1 | 1 |
| handwritten | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_annotations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 2/3 | 3/3 | 0 |
| handwritten_dates | 2 | 0/2 | 0/2 | 2/2 | 0 |
| handwritten_signature | 2 | 0/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 0/4 | 1/4 | 1/4 | 1 |
| historical | 2 | 1/2 | 2/2 | 2/2 | 0 |
| image | 4 | 0/4 | 2/4 | 1/4 | 0 |
| image_only | 4 | 0/3 | 0/3 | 3/4 | 2 |
| image_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 1/4 | 2/4 | 3/5 | 1 |
| ireland | 2 | 0/2 | 2/2 | 0/2 | 1 |
| irregular_spacing | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 0/1 | 1/1 | 2/2 | 0 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 1/1 | 0/1 | 0 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 5/18 | 14/18 | 11/18 | 3 |
| letter_format | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_from_company | 3 | 0/3 | 2/3 | 1/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead | 9 | 3/9 | 7/9 | 7/9 | 1 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 1/2 | 1 |
| letterhead_logo | 2 | 0/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 2 | 0/2 | 2/2 | 0/2 | 0 |
| local_government | 1 | 0/1 | 1/1 | 1/1 | 0 |
| logo_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 1/4 | 4/4 | 3/4 | 1 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_addressees | 1 | 0/1 | 0/1 | 0/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_dates | 8 | 2/8 | 6/8 | 5/8 | 1 |
| many_named_organisations | 3 | 1/3 | 3/3 | 1/3 | 2 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 1/4 | 3/4 | 4/4 | 0 |
| memo | 5 | 1/4 | 4/4 | 3/5 | 1 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 1/4 | 4/4 | 3/5 | 1 |
| metadata_date_trap | 14 | 2/10 | 8/10 | 9/14 | 2 |
| minutes | 6 | 1/6 | 6/6 | 5/6 | 1 |
| misprinted_year | 1 | 0/1 | 0/1 | 1/1 | 1 |
| mixed_scan_and_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 44 | 10/39 | 31/39 | 29/44 | 9 |
| month_only_date | 2 | 0/1 | 0/1 | 1/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| municipal | 2 | 0/2 | 1/2 | 1/2 | 1 |
| named_officials | 9 | 3/8 | 7/8 | 8/9 | 0 |
| named_person_at_org | 9 | 3/9 | 9/9 | 7/9 | 2 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 1 |
| no_explicit_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 1/3 | 1 |
| no_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_signatory | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_specific_addressee | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_text_layer | 2 | 0/2 | 0/2 | 1/2 | 2 |
| no_title | 1 | – | – | 0/1 | 1 |
| noisy_ocr_layer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| nonprofit | 2 | 1/2 | 1/2 | 1/2 | 0 |
| notice | 11 | 3/11 | 7/11 | 9/11 | 3 |
| notice_of_decision | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 1/1 | 1 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 3/12 | 10/12 | 10/12 | 1 |
| office | 4 | 0/1 | 1/1 | 3/4 | 0 |
| open_letter | 1 | 0/1 | 0/1 | 0/1 | 0 |
| opinion_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 1/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 1/1 | 1/1 | 3/3 | 0 |
| printed_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 0 |
| public_notice | 6 | 2/6 | 3/6 | 5/6 | 2 |
| purchase_order | 1 | 0/1 | 0/1 | 1/1 | 0 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| received_stamp | 1 | 0/1 | 0/1 | 1/1 | 1 |
| redacted | 1 | 0/1 | 1/1 | 1/1 | 0 |
| redactions | 3 | 1/3 | 3/3 | 2/3 | 0 |
| reference_number | 11 | 7/11 | 10/11 | 9/11 | 0 |
| reference_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 0/1 | 1 |
| referenced_dates | 6 | 3/6 | 6/6 | 4/6 | 1 |
| regulator | 5 | 2/5 | 5/5 | 4/5 | 0 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 1 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 1/1 | 0/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 1/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 1 |
| running_header | 1 | 0/1 | 1/1 | 0/1 | 1 |
| scanned | 17 | 3/16 | 10/16 | 13/17 | 4 |
| scanned_signature_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| security_marking | 1 | – | – | 0/1 | 1 |
| senate_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 0 |
| service_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| service_period | 3 | 0/2 | 2/2 | 1/3 | 0 |
| short | 8 | 0/7 | 3/7 | 6/8 | 4 |
| signatory_trap | 2 | 0/2 | 1/2 | 1/2 | 1 |
| signatory_vs_org | 3 | 1/3 | 2/3 | 3/3 | 1 |
| signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 0/5 | 3/5 | 5/5 | 1 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 1/1 | 0/1 | 0 |
| source_title_mismatch | 3 | 2/3 | 2/3 | 3/3 | 1 |
| south_africa | 3 | 0/2 | 1/2 | 1/3 | 1 |
| staff_names | 8 | 1/8 | 8/8 | 4/8 | 1 |
| stamp_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| stamped_date | 3 | 0/3 | 1/3 | 2/3 | 1 |
| statement | 1 | – | – | 0/1 | 1 |
| statute_date | 1 | – | – | 0/1 | 0 |
| statutory_notice | 1 | 0/1 | 1/1 | 0/1 | 0 |
| successor_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| table | 2 | 0/1 | 1/1 | 0/2 | 1 |
| table_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| table_of_contents | 2 | 0/2 | 2/2 | 1/2 | 1 |
| tables | 3 | 0/2 | 1/2 | 3/3 | 0 |
| tax_invoice | 2 | 1/2 | 2/2 | 1/2 | 0 |
| template_attachment | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 1/3 | 1 |
| through_line | 1 | – | – | 1/1 | 0 |
| trading_name | 2 | 0/2 | 0/2 | 2/2 | 0 |
| transmittal_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| two_day_meeting | 1 | 0/1 | 1/1 | 0/1 | 1 |
| two_digit_year | 2 | 0/1 | 1/1 | 0/2 | 1 |
| two_senders | 1 | 0/1 | 0/1 | 0/1 | 0 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 1/2 | 1 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 0/2 | 1/2 | 0/2 | 1 |
| university | 4 | 0/3 | 2/3 | 3/4 | 2 |
| university_foundation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_date_format | 4 | 1/4 | 2/4 | 4/4 | 0 |
| us_federal | 12 | 4/11 | 11/11 | 6/12 | 3 |
| us_local | 6 | 1/5 | 3/5 | 6/6 | 0 |
| us_state | 8 | 2/8 | 7/8 | 6/8 | 0 |
| warn_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 0/1 | 1/1 | 1/1 | 0 |
| withdrawn | 1 | 0/1 | 1/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 6.02 s | 16.58 s | 20.26 s |
| `extraction_wall_ms` | 50 | 173.6 ms | 3.98 s | 11.36 s |
| `worker_total_ms` | 50 | 172.5 ms | 3.98 s | 11.35 s |
| `worker_snapshot_ms` | 50 | 0.26 ms | 0.51 ms | 0.84 ms |
| `worker_parse_ms` | 50 | 5.1 ms | 51.3 ms | 166.9 ms |
| `worker_analysis_ms` | 50 | 0.64 ms | 4.5 ms | 6.3 ms |
| `worker_render_ms` | 50 | 0.00 ms | 528.2 ms | 1.25 s |
| `worker_image_decode_ms` | 50 | 0.00 ms | 25.0 ms | 40.7 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 6.19 s | 21.42 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 524.2 ms | 1.16 s |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 5.46 s | 20.01 s |
| `worker_vision_ms` | 50 | 0.00 ms | 8.6 ms | 15.0 ms |
| `analyze_wall_ms` | 50 | 5.15 s | 11.31 s | 18.51 s |
| `distill_ms` | 50 | 2.0 ms | 23.8 ms | 35.6 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.06 ms | 0.06 ms |
| `inference_ms` | 50 | 5.12 s | 11.27 s | 18.45 s |
| `prefill_ms` | 50 | 3.31 s | 8.77 s | 15.80 s |
| `generation_ms` | 50 | 1.82 s | 2.63 s | 2.87 s |
| `validation_ms` | 50 | 5.0 ms | 19.1 ms | 20.8 ms |
| `naming_ms` | 50 | 0.22 ms | 2.7 ms | 4.4 ms |
| `index_ms` | 50 | 1.8 ms | 22.6 ms | 32.6 ms |
| `retrieval_ms` | 50 | 0.03 ms | 1.9 ms | 2.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 295.3 tok/s | 304.9 tok/s | 309.8 tok/s |
| `generation_tok_per_s` | 50 | 40.9 tok/s | 43.3 tok/s | 43.5 tok/s |
| `prompt_tokens` | 50 | 1003 | 2454 | 4079 |
| `cached_tokens` | 50 | 477 | 477 | 477 |
| `generated_tokens` | 50 | 75 | 104 | 114 |
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
| fast | 12 | 4.3 ms | 60.8 ms | 3.9 ms | 59.7 ms | 51.3 ms | 5.0 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 8.2 ms | 173.6 ms | 7.7 ms | 172.5 ms | 166.9 ms | 4.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.08 s | 2.00 s | 1.08 s | 2.00 s | 2.8 ms | 0.81 ms | 124.9 ms | 1.82 s |
| ocr | 22 | 1.97 s | 5.68 s | 1.97 s | 5.68 s | 28.0 ms | 3.2 ms | 787.4 ms | 10.35 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.08 s | 3.15 s | 1.08 s | 3.15 s | 23.1 ms | 0.81 ms | 528.2 ms | 3.12 s |
| 2-4 | 16 | 11.4 ms | 5.68 s | 10.9 ms | 5.68 s | 26.4 ms | 2.3 ms | 281.7 ms | 10.35 s |
| 5-9 | 6 | 20.7 ms | 11.36 s | 16.4 ms | 11.35 s | 166.9 ms | 4.2 ms | 1.25 s | 21.42 s |
| 10-24 | 4 | 21.4 ms | 2.32 s | 20.6 ms | 2.32 s | 51.3 ms | 6.3 ms | 343.3 ms | 3.84 s |
| 25-49 | 2 | 46.7 ms | 47.1 ms | 45.3 ms | 45.6 ms | 39.5 ms | 5.0 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 5.4 ms | 11.36 s | 5.0 ms | 11.35 s | 56.1 ms | 3.2 ms | 1.25 s | 21.42 s |
| invoice | 4 | 1.39 s | 1.97 s | 1.39 s | 1.97 s | 11.5 ms | 0.92 ms | 528.2 ms | 1.72 s |
| letter | 18 | 173.6 ms | 5.68 s | 172.5 ms | 5.68 s | 166.9 ms | 4.2 ms | 787.4 ms | 10.35 s |
| memo | 5 | 6.3 ms | 3.98 s | 5.8 ms | 3.98 s | 10.7 ms | 1.8 ms | 214.1 ms | 5.83 s |
| minutes | 6 | 46.7 ms | 2.32 s | 45.3 ms | 2.32 s | 39.5 ms | 6.3 ms | 343.3 ms | 3.84 s |
| notice | 11 | 966.9 ms | 3.37 s | 966.3 ms | 3.37 s | 24.9 ms | 1.6 ms | 213.6 ms | 3.19 s |
| statement | 1 | 752.7 ms | 752.7 ms | 752.4 ms | 752.4 ms | 0.45 ms | 0.13 ms | 110.0 ms | 629.0 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 442 MB | 1242 MB | real6-form-03 |
| Model server | 846 MB | 848 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (34)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter to Power Systems & Supplies of Alaska (PSSA).pdf` · trap: party Power Systems & Supplies of Alaska (applicant whose authorization the letter comments on; neither sender nor recipient) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-02**: expected `1980-05-12 Opinion Letter from Pension Benefit Guaranty Corporation.pdf`, got `1980-05-12 Document from Pension Benefit Guaranty Corporation.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-07 Nuclear Waste Technical Review Board for Nevada Nuclear Waste Task Force, Incorporated.pdf` · trap: date 2002-11-07 (the Board's receipt stamp, not the letter's date) · review: LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for IF Leco Corporation.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - House Executive Message.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-10**: expected `2025-06-23 Letter to Mike Lee.pdf`, got `2025-06-23 One Big Beautiful Bill from United States Senate.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-11**: expected `2023-11-14 Letter from Department of Public Expenditure NDP Delivery and Reform.pdf`, got `2023-11-14 Letter to Office for the Irish Human Rights and Equality Commission.pdf` · trap: party Irish Human Rights and Equality Commission (body whose Accounting Officer is appointed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to Bosca PO 559 Baile Átha Cliath.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter from Secretary to the Treasury.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-15**: expected `2025-05-12 Letter from ANZ.pdf`, got `2025-05-12 Letter to ANZ.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-17**: expected `1915-03-25 Letter from Poro College.jpg`, got `1915-03-25 Letter to A.H. PROP.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2025-09-16 Open Letter.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real6-notice-03**: expected `2026-06-04 Notice of Exemption for Northeast Texas Railway Company.pdf`, got `2026-06-04 Document from Northeast Texas Rural Rail Transportation District.pdf` · trap: party Northeast Texas Rural Rail Transportation District (owner of the line, named in the caption; right only beside the railway in a 'between' name) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document from National Park Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Document from Conway Board of Health.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-07**: expected `2026-11-09 Notice of Exemption from San Bernardino County Department of Public Works.pdf`, got `Notice of Exemption from Office of Land Use and Climate Innovation.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `2026-06-16 Public Notice for Meyerton 1960.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-09-03 Announcement of Planned Action from Department of Workforce Development.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting for NC State University.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces) · review: LOW_CONFIDENCE
- **real6-invoice-02**: expected `2025-07-30 Tax Invoice from Department of Parliamentary Services.pdf`, got `2025-07-30 House Services Order Form - s 22.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document - M. A. Meade.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `2019-09-30 Massachusetts Narrative Report on Expenditure of Hava Section from Federal Agency.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `Delivery Order from Purchasing Division.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Duke Energy Florida, LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: LOW_CONFIDENCE
- **real6-minutes-01**: expected `2026-08-18 Board Meeting Minutes for Federal Accounting Standards Advisory Board.pdf`, got `2026-08-18 Fasab Meeting Minutes for Department of War.pdf` · trap: party Department of War (department that briefed the Board) · review: LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order from Port of Vancouver USA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Record of the Regular Meeting of the from University of Wisconsin System Board of Regents.pdf` · review: LOW_CONFIDENCE
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-07-16 Document from Shire of Woodanilling.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-06**: expected `2025-10-08 Meeting Minutes for University Senate.docx`, got `2025-10-08 University Senate Meeting Minutes from UO Senate.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

# InternBench: live run

- **Run:** 2026-11-21T09:38:44Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `97291f77f59a`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 589.90 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 8/45 (17.8%) |
| Document type | 33/50 (66.0%) |
| Date | 28/45 (62.2%) |
| Date role (when the reviewed date was chosen) | 5/27 (18.5%) |
| Parties | 23/50 (46.0%) |
| Relation word (when parties were named) | 24/44 (54.5%) |
| Party in the right role (when parties were named) | 14/32 (43.8%) |
| Ready / review routing | 6/30 (20.0%) |
| Description states nothing false | 50/50 (100.0%) |
| Description is specific | 30/50 (60.0%) |
| Description specificity | 53.3% (mean of 50) |
| Evidence recall (model's quotes) | 20.6% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 100.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 82.0% | 50 |
| Review rate | 100.0% | 50 |
| Evidence recall | 20.6% | 46 |
| Total latency p50 | 9.72 s | 50 |
| Total latency p95 | 26.51 s | 50 |
| Generation latency p50 | 3.01 s | 50 |
| Generation latency p95 | 3.92 s | 50 |
| Generated tokens p50 | 94 | 50 |
| Generated tokens p95 | 122 | 50 |
| Prompt tokens p50 | 1003 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 45 |
| Trap date chosen | 7 of 50 |
| Forbidden party named | 8 of 50 |
| Spurious parties named | 25 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 114 (0.0%) |
| Right name sent to review anyway | 6 of 24 |
| Sent to review for an unsupported fact | 41 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 2/2 | 2/5 | 3/3 | 100.0% | 12.90 s | 28.35 s |
| invoice | 4 | 1/4 | 1/4 | 1/4 | 0/2 | 100.0% | 9.12 s | 15.49 s |
| letter | 18 | 4/18 | 11/18 | 10/18 | 0/11 | 100.0% | 9.27 s | 17.67 s |
| memo | 5 | 1/4 | 4/4 | 2/5 | 1/3 | 100.0% | 7.50 s | 13.90 s |
| minutes | 6 | 0/6 | 5/6 | 1/6 | 0/5 | 100.0% | 15.81 s | 30.99 s |
| notice | 11 | 2/11 | 5/11 | 7/11 | 1/5 | 100.0% | 8.37 s | 16.03 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 5.58 s | 5.58 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 0/4 | 1/4 | 0/4 | 0/1 | 100.0% | 7.49 s | 12.26 s |
| native | 25 | 6/24 | 17/24 | 12/25 | 1/19 | 100.0% | 9.27 s | 20.71 s |
| office | 4 | 0/1 | 1/1 | 2/4 | 3/3 | 100.0% | 6.51 s | 15.61 s |
| scan | 17 | 2/16 | 9/16 | 9/17 | 2/7 | 100.0% | 12.89 s | 28.35 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3/18 | 6/18 | 9/22 | 5/11 | 100.0% | 7.37 s | 12.26 s |
| 2-4 | 16 | 1/15 | 12/15 | 9/16 | 1/9 | 100.0% | 13.90 s | 26.51 s |
| 5-9 | 6 | 2/6 | 5/6 | 3/6 | 0/4 | 100.0% | 10.30 s | 28.35 s |
| 10-24 | 4 | 2/4 | 3/4 | 2/4 | 0/4 | 100.0% | 7.50 s | 20.71 s |
| 25-49 | 2 | 0/2 | 2/2 | 0/2 | 0/2 | 100.0% | 15.81 s | 30.99 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 2/6 | 5/6 | 2/6 | 0/6 | 0 of 6 | 9.27 s | 30.99 s | 2.92 s | 4.09 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 0/1 | 0 |
| addressed_to_person | 3 | 0/3 | 2/3 | 1/3 | 0 |
| addressee_is_office_holder | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 1/1 | 0 |
| agency_enforcement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_guidance | 2 | 1/2 | 2/2 | 1/2 | 0 |
| agency_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| attachment_with_own_date | 2 | 0/2 | 1/2 | 1/2 | 0 |
| attachments | 3 | 1/3 | 3/3 | 2/3 | 0 |
| attendance_list | 3 | 0/3 | 2/3 | 0/3 | 1 |
| attendee_list | 2 | 0/2 | 2/2 | 1/2 | 0 |
| australia | 6 | 1/5 | 1/5 | 2/6 | 3 |
| bilingual_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| billhead | 2 | 0/2 | 0/2 | 0/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 2 |
| board_minutes | 4 | 0/4 | 4/4 | 0/4 | 0 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 23 | 6/22 | 17/22 | 11/23 | 6 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| canada | 2 | 1/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 0/1 | 1/1 | 1 |
| certificate_of_service | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 38 | 6/38 | 23/38 | 18/38 | 10 |
| confirmation_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| copier_searchable_pdf | 2 | 0/2 | 0/2 | 1/2 | 2 |
| corporate_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| council_minutes | 1 | 0/1 | 0/1 | 0/1 | 1 |
| counsel_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 0/1 | 0/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 0/1 | 0/1 | 1 |
| cover_sheet | 2 | 0/2 | 1/2 | 2/2 | 1 |
| customer_first | 4 | 1/4 | 1/4 | 1/4 | 1 |
| date_at_end | 5 | 1/5 | 2/5 | 4/5 | 1 |
| date_at_foot | 1 | 0/1 | 0/1 | 0/1 | 2 |
| date_at_top | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_range | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| day_first_date | 4 | 1/3 | 1/3 | 1/4 | 3 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 1 |
| deadline_in_body | 7 | 0/7 | 6/7 | 4/7 | 2 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| docket_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 0/1 | 1/1 | 2/4 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 1/3 | 2/3 | 2/3 | 1 |
| enclosure | 1 | 0/1 | 0/1 | 1/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 0/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 0/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 0/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 0/1 | 2 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 1/1 | 0 |
| filled_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| first_name_salutation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| foi_release | 5 | 1/4 | 1/4 | 2/5 | 2 |
| footnotes | 3 | 1/3 | 2/3 | 2/3 | 1 |
| form | 6 | 0/3 | 2/3 | 3/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 0/1 | 0 |
| form_revision_date | 2 | 0/1 | 1/1 | 1/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 0/3 | 2/3 | 2/5 | 0 |
| government_sender | 1 | 0/1 | 0/1 | 0/1 | 1 |
| handwritten | 2 | 0/2 | 0/2 | 0/2 | 0 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 1/3 | 1/3 | 1 |
| handwritten_dates | 2 | 0/2 | 1/2 | 1/2 | 0 |
| handwritten_signature | 2 | 0/2 | 0/2 | 1/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| hearing_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 0/4 | 1/4 | 0/4 | 1 |
| historical | 2 | 0/2 | 0/2 | 2/2 | 1 |
| image | 4 | 0/4 | 1/4 | 0/4 | 0 |
| image_only | 4 | 0/3 | 1/3 | 1/4 | 2 |
| image_page | 1 | 0/1 | 0/1 | 0/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 0/1 | 0/1 | 1/1 | 0 |
| invoice | 5 | 1/4 | 1/4 | 2/5 | 2 |
| ireland | 2 | 0/2 | 1/2 | 0/2 | 1 |
| irregular_spacing | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 0/1 | 1/1 | 1/2 | 1 |
| issuer_in_image | 1 | 0/1 | 0/1 | 0/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 0/1 | 0/1 | 0 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 0/1 | 0/1 | 0 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 4/18 | 11/18 | 10/18 | 4 |
| letter_format | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letter_from_company | 3 | 0/3 | 1/3 | 1/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 0/1 | 1/1 | 0/1 | 0 |
| letterhead | 9 | 3/9 | 8/9 | 6/9 | 0 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 0/2 | 1 |
| letterhead_logo | 2 | 0/2 | 1/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 0/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 0/1 | 1/1 | 1 |
| line_item_dates | 2 | 0/2 | 1/2 | 0/2 | 0 |
| local_government | 1 | 0/1 | 0/1 | 0/1 | 1 |
| logo_issuer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| long_document | 4 | 1/4 | 3/4 | 1/4 | 1 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_dates | 8 | 0/8 | 6/8 | 2/8 | 1 |
| many_named_organisations | 3 | 1/3 | 3/3 | 1/3 | 1 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 1/4 | 4/4 | 2/4 | 0 |
| memo | 5 | 1/4 | 4/4 | 2/5 | 2 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 1/4 | 4/4 | 2/5 | 2 |
| metadata_date_trap | 14 | 1/10 | 4/10 | 6/14 | 6 |
| minutes | 6 | 0/6 | 5/6 | 1/6 | 1 |
| misprinted_year | 1 | 0/1 | 0/1 | 0/1 | 2 |
| mixed_scan_and_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 44 | 8/39 | 27/39 | 21/44 | 13 |
| month_only_date | 2 | 0/1 | 0/1 | 2/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 0/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 0/2 | 0/2 | 0/2 | 1 |
| named_officials | 9 | 3/8 | 7/8 | 5/9 | 2 |
| named_person_at_org | 9 | 3/9 | 6/9 | 7/9 | 3 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 4 |
| no_explicit_addressee | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 1/3 | 4 |
| no_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_signatory | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_text_layer | 2 | 0/2 | 1/2 | 0/2 | 1 |
| no_title | 1 | – | – | 0/1 | 2 |
| noisy_ocr_layer | 1 | 0/1 | 0/1 | 1/1 | 1 |
| nonprofit | 2 | 0/2 | 1/2 | 1/2 | 0 |
| notice | 11 | 2/11 | 5/11 | 7/11 | 4 |
| notice_of_decision | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 0/1 | 2 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 2/12 | 8/12 | 8/12 | 2 |
| office | 4 | 0/1 | 1/1 | 2/4 | 2 |
| open_letter | 1 | 0/1 | 0/1 | 1/1 | 0 |
| opinion_letter | 1 | 0/1 | 0/1 | 1/1 | 1 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 0/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| placeholders | 3 | 1/1 | 1/1 | 2/3 | 2 |
| printed_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 0 |
| public_notice | 6 | 1/6 | 2/6 | 3/6 | 3 |
| purchase_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| received_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 1 |
| redactions | 3 | 1/3 | 1/3 | 2/3 | 0 |
| reference_number | 11 | 6/11 | 10/11 | 8/11 | 1 |
| reference_stamp | 1 | 0/1 | 0/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 3/6 | 5/6 | 4/6 | 2 |
| regulator | 5 | 1/5 | 5/5 | 4/5 | 0 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 1 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 1/1 | 0/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 0/1 | 0 |
| revised | 1 | 0/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 2 |
| running_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 17 | 2/16 | 9/16 | 9/17 | 5 |
| scanned_signature_page | 1 | 0/1 | 0/1 | 0/1 | 1 |
| script_font_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| security_marking | 1 | – | – | 0/1 | 2 |
| senate_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 0/2 | 1/2 | 1/3 | 1 |
| short | 8 | 0/7 | 2/7 | 2/8 | 6 |
| signatory_trap | 2 | 0/2 | 0/2 | 0/2 | 1 |
| signatory_vs_org | 3 | 1/3 | 2/3 | 2/3 | 1 |
| signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_on_later_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 0/5 | 3/5 | 3/5 | 2 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 1/1 | 0/1 | 0 |
| source_title_mismatch | 3 | 0/3 | 2/3 | 1/3 | 0 |
| south_africa | 3 | 0/2 | 0/2 | 0/3 | 1 |
| staff_names | 8 | 0/8 | 6/8 | 1/8 | 1 |
| stamp_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| stamped_date | 3 | 0/3 | 2/3 | 1/3 | 1 |
| statement | 1 | – | – | 0/1 | 2 |
| statute_date | 1 | – | – | 1/1 | 0 |
| statutory_notice | 1 | 0/1 | 0/1 | 0/1 | 0 |
| successor_name | 2 | 0/2 | 0/2 | 0/2 | 1 |
| table | 2 | 0/1 | 1/1 | 0/2 | 2 |
| table_layout | 1 | 0/1 | 1/1 | 0/1 | 0 |
| table_of_contents | 2 | 0/2 | 1/2 | 0/2 | 1 |
| tables | 3 | 0/2 | 2/2 | 2/3 | 1 |
| tax_invoice | 2 | 1/2 | 1/2 | 1/2 | 0 |
| template_attachment | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 1/3 | 1 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 1/2 | 0/2 | 0 |
| transmittal_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| two_day_meeting | 1 | 0/1 | 1/1 | 0/1 | 0 |
| two_digit_year | 2 | 0/1 | 0/1 | 0/2 | 2 |
| two_senders | 1 | 0/1 | 0/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 0/2 | 2 |
| typed | 1 | 0/1 | 0/1 | 1/1 | 0 |
| typewritten | 2 | 0/2 | 1/2 | 0/2 | 1 |
| university | 4 | 0/3 | 2/3 | 1/4 | 4 |
| university_foundation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_date_format | 4 | 1/4 | 3/4 | 2/4 | 0 |
| us_federal | 12 | 3/11 | 9/11 | 7/12 | 2 |
| us_local | 6 | 1/5 | 3/5 | 5/6 | 1 |
| us_state | 8 | 2/8 | 8/8 | 4/8 | 1 |
| warn_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| watermark | 1 | 0/1 | 0/1 | 1/1 | 1 |
| withdrawn | 1 | 0/1 | 0/1 | 1/1 | 1 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 9.72 s | 26.51 s | 30.99 s |
| `extraction_wall_ms` | 50 | 197.4 ms | 4.63 s | 13.75 s |
| `worker_total_ms` | 50 | 196.0 ms | 4.63 s | 13.75 s |
| `worker_snapshot_ms` | 50 | 0.32 ms | 0.63 ms | 0.98 ms |
| `worker_parse_ms` | 50 | 6.3 ms | 66.4 ms | 189.0 ms |
| `worker_analysis_ms` | 50 | 0.73 ms | 5.6 ms | 7.6 ms |
| `worker_render_ms` | 50 | 0.00 ms | 599.4 ms | 1.46 s |
| `worker_image_decode_ms` | 50 | 0.00 ms | 23.5 ms | 55.3 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 7.35 s | 25.90 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 537.6 ms | 960.4 ms |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 6.59 s | 24.62 s |
| `worker_vision_ms` | 50 | 0.00 ms | 10.3 ms | 17.6 ms |
| `analyze_wall_ms` | 50 | 9.12 s | 18.07 s | 30.93 s |
| `distill_ms` | 50 | 2.5 ms | 29.5 ms | 44.2 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.07 ms | 0.07 ms |
| `inference_ms` | 50 | 9.11 s | 18.02 s | 30.86 s |
| `prefill_ms` | 50 | 6.04 s | 15.37 s | 26.75 s |
| `generation_ms` | 50 | 3.01 s | 3.92 s | 4.09 s |
| `validation_ms` | 50 | 6.9 ms | 26.1 ms | 31.1 ms |
| `naming_ms` | 50 | 0.25 ms | 3.4 ms | 5.5 ms |
| `index_ms` | 50 | 2.2 ms | 28.2 ms | 40.6 ms |
| `retrieval_ms` | 50 | 0.04 ms | 2.3 ms | 3.3 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 165.2 tok/s | 169.7 tok/s | 170.8 tok/s |
| `generation_tok_per_s` | 50 | 31.3 tok/s | 34.1 tok/s | 34.5 tok/s |
| `prompt_tokens` | 50 | 1003 | 2454 | 4079 |
| `cached_tokens` | 50 | 477 | 477 | 477 |
| `generated_tokens` | 50 | 94 | 122 | 126 |
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
| fast | 12 | 5.5 ms | 81.9 ms | 5.1 ms | 80.6 ms | 70.8 ms | 5.9 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 10.7 ms | 197.4 ms | 10.3 ms | 196.0 ms | 189.0 ms | 5.6 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.33 s | 2.43 s | 1.33 s | 2.43 s | 4.8 ms | 1.0 ms | 148.3 ms | 2.22 s |
| ocr | 22 | 2.32 s | 6.26 s | 2.32 s | 6.26 s | 36.1 ms | 3.6 ms | 918.4 ms | 11.23 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.33 s | 3.78 s | 1.33 s | 3.78 s | 23.3 ms | 1.0 ms | 599.4 ms | 3.75 s |
| 2-4 | 16 | 13.7 ms | 6.26 s | 13.2 ms | 6.26 s | 30.7 ms | 2.8 ms | 334.9 ms | 11.23 s |
| 5-9 | 6 | 20.9 ms | 13.75 s | 20.3 ms | 13.75 s | 189.0 ms | 5.6 ms | 1.46 s | 25.90 s |
| 10-24 | 4 | 28.1 ms | 2.65 s | 26.9 ms | 2.64 s | 70.8 ms | 7.6 ms | 429.6 ms | 4.30 s |
| 25-49 | 2 | 59.8 ms | 61.0 ms | 58.1 ms | 59.4 ms | 52.3 ms | 5.9 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 6.6 ms | 13.75 s | 6.2 ms | 13.75 s | 66.4 ms | 3.6 ms | 1.46 s | 25.90 s |
| invoice | 4 | 1.66 s | 2.32 s | 1.66 s | 2.32 s | 14.5 ms | 1.1 ms | 599.4 ms | 2.07 s |
| letter | 18 | 197.4 ms | 6.26 s | 196.0 ms | 6.26 s | 189.0 ms | 5.6 ms | 918.4 ms | 11.23 s |
| memo | 5 | 7.6 ms | 4.57 s | 7.2 ms | 4.57 s | 13.6 ms | 2.1 ms | 261.1 ms | 6.64 s |
| minutes | 6 | 59.8 ms | 2.65 s | 58.1 ms | 2.64 s | 52.3 ms | 7.6 ms | 429.6 ms | 4.30 s |
| notice | 11 | 1.14 s | 4.05 s | 1.14 s | 4.05 s | 25.3 ms | 2.0 ms | 250.9 ms | 3.83 s |
| statement | 1 | 919.4 ms | 919.4 ms | 919.1 ms | 919.1 ms | 0.52 ms | 0.13 ms | 116.4 ms | 787.4 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 454 MB | 1253 MB | real6-form-03 |
| Model server | 846 MB | 853 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (37)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter to Power Systems & Supplies of Alaska.pdf` · trap: party Power Systems & Supplies of Alaska (applicant whose authorization the letter comments on; neither sender nor recipient) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-02**: expected `1980-05-12 Opinion Letter from Pension Benefit Guaranty Corporation.pdf`, got `1977-07-18 Letter from Pension Benefit Guaranty Corporation.pdf` · trap: date 1977-07-18 (date of a letter to a Congressman, in a footnote) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-01 Letter to The Members.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-05**: expected `1969-06-28 Letter to Lewis Research Center.pdf`, got `Letter to National Aeronautics and Space Administration.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for EGLE.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Michelle Lujan Grisham.pdf` · trap: party Michelle Lujan Grisham (Governor who signs; the office is the sender and the office-holder is a trap, as in the earlier samples) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-10**: expected `2025-06-23 Letter to Mike Lee.pdf`, got `2025-06-23 Letter to Garfield County Colorado Board of County Commissioners.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-11**: expected `2023-11-14 Letter from Department of Public Expenditure NDP Delivery and Reform.pdf`, got `Letter to Deirdre Malone.pdf` · trap: party Deirdre Malone (the Accounting Officer appointed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to CEO.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter to Ohn A. faen.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-15**: expected `2025-05-12 Letter from ANZ.pdf`, got `Letter to Reserve Bank of Australia.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-17**: expected `1915-03-25 Letter from Poro College.jpg`, got `1915-03-25 Letter from A.H. PROP.jpg` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `Open Letter to ACLU of Northern California.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-03**: expected `2026-06-04 Notice of Exemption for Northeast Texas Railway Company.pdf`, got `2026-06-01 Document - Northeast Texas Railway Company.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document from Commission Action.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Public Hearing Notice from Conway Board of Health.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-07**: expected `2026-11-09 Notice of Exemption from San Bernardino County Department of Public Works.pdf`, got `Notice of Exemption for San Bernardino County.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-08**: expected `2025-08-20 Revised Notice of Public Hearing and Participant Funding for NexGen Energy Ltd.pdf`, got `2025-08-20 record for NexGen Energy Ltd.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `Public Notice for Municipal Manager.jpg` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac for Yours in Governance.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-10-10 Rapid Response - Perdue Farms.pdf` · trap: date 2025-10-10 (date of the layoff and the employees' last work date) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting from Maggie Thompson Assistant Secretary.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces); party Maggie Thompson (Assistant Secretary who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-02**: expected `2025-07-30 Tax Invoice from Department of Parliamentary Services.pdf`, got `Document.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document from In every style known in theart, M ablalte.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `2019-12-23 Federal Financial Report from Federal Agency.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `2026-04-06 Document from Fisher, Cindy.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Public Service Commission.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: LOW_CONFIDENCE
- **real6-minutes-01**: expected `2026-08-18 Board Meeting Minutes for Federal Accounting Standards Advisory Board.pdf`, got `2026-08-18 Fasab Meeting Minutes for Board Meeting Minutes.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order - Commissioner Burkman.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Record of the Regular Meeting of the for Regent Manydeeds.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-06-30 Document - Chief Executive Officer.pdf` · trap: date 2024-06-30 (end of the period of the accounts and financial report presented) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-05**: expected `2024-08-30 Board Meeting Minutes for Cal Poly Humboldt Foundation.pdf`, got `2024-08-30 MINUTES of the BOARD OF DIRECTORS for Ken Fulgham.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-06**: expected `2025-10-08 Meeting Minutes for University Senate.docx`, got `2025-10-08 Document - University Senate Meeting Minutes.docx` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

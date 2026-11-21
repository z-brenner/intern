# InternBench: live run

- **Run:** 2026-11-21T04:51:50Z · commit `a51447d321`
- **Machine:** INTEL(R) XEON(R) PLATINUM 8573C, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 736.07 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 14/45 (31.1%) |
| Document type | 35/50 (70.0%) |
| Date | 34/45 (75.6%) |
| Date role (when the reviewed date was chosen) | 6/32 (18.8%) |
| Parties | 31/50 (62.0%) |
| Relation word (when parties were named) | 27/44 (61.4%) |
| Party in the right role (when parties were named) | 20/33 (60.6%) |
| Ready / review routing | 7/30 (23.3%) |
| Description states nothing false | 50/50 (100.0%) |
| Description is specific | 30/50 (60.0%) |
| Description specificity | 54.0% (mean of 50) |
| Evidence recall (model's quotes) | 46.0% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 96.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 83.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 64.0% | 50 |
| Review rate | 96.0% | 50 |
| Evidence recall | 46.0% | 46 |
| Total latency p50 | 12.78 s | 50 |
| Total latency p95 | 28.20 s | 50 |
| Generation latency p50 | 5.91 s | 50 |
| Generation latency p95 | 7.50 s | 50 |
| Generated tokens p50 | 88 | 50 |
| Generated tokens p95 | 111 | 50 |
| Prompt tokens p50 | 1003 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 45 |
| Trap date chosen | 7 of 50 |
| Forbidden party named | 11 of 50 |
| Spurious parties named | 18 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 117 (0.0%) |
| Right name sent to review anyway | 9 of 24 |
| Sent to review for an unsupported fact | 32 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 1/2 | 2/5 | 3/3 | 100.0% | 15.85 s | 30.87 s |
| invoice | 4 | 1/4 | 2/4 | 2/4 | 0/2 | 100.0% | 10.03 s | 19.94 s |
| letter | 18 | 5/18 | 15/18 | 11/18 | 0/11 | 100.0% | 11.76 s | 22.42 s |
| memo | 5 | 2/4 | 4/4 | 2/5 | 1/3 | 80.0% | 11.77 s | 17.09 s |
| minutes | 6 | 4/6 | 6/6 | 5/6 | 1/5 | 83.3% | 19.53 s | 32.03 s |
| notice | 11 | 2/11 | 6/11 | 9/11 | 1/5 | 100.0% | 12.75 s | 19.53 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 7.55 s | 7.55 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 0/4 | 2/4 | 2/4 | 0/1 | 100.0% | 12.12 s | 12.75 s |
| native | 25 | 10/24 | 21/24 | 19/25 | 2/19 | 92.0% | 11.77 s | 23.08 s |
| office | 4 | 1/1 | 1/1 | 2/4 | 3/3 | 100.0% | 10.33 s | 19.53 s |
| scan | 17 | 3/16 | 10/16 | 8/17 | 2/7 | 100.0% | 16.26 s | 30.87 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3/18 | 8/18 | 14/22 | 5/11 | 95.5% | 10.03 s | 13.91 s |
| 2-4 | 16 | 6/15 | 15/15 | 10/16 | 1/9 | 100.0% | 17.09 s | 28.20 s |
| 5-9 | 6 | 0/6 | 5/6 | 1/6 | 0/4 | 100.0% | 13.86 s | 30.87 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 0/4 | 100.0% | 10.71 s | 23.08 s |
| 25-49 | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 18.29 s | 32.03 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 5/6 | 6/6 | 6/6 | 1/6 | 0 of 6 | 11.77 s | 32.03 s | 4.96 s | 6.97 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 0/3 | 2/3 | 1/3 | 0 |
| addressee_is_office_holder | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 1/1 | 0 |
| agency_enforcement | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_guidance | 2 | 2/2 | 2/2 | 2/2 | 0 |
| agency_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appendices | 1 | 1/1 | 1/1 | 1/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachment_with_own_date | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments | 3 | 1/3 | 2/3 | 1/3 | 2 |
| attendance_list | 3 | 3/3 | 3/3 | 3/3 | 0 |
| attendee_list | 2 | 1/2 | 2/2 | 1/2 | 0 |
| australia | 6 | 3/5 | 4/5 | 4/6 | 2 |
| bilingual_letterhead | 1 | 0/1 | 0/1 | 0/1 | 2 |
| billhead | 2 | 0/2 | 0/2 | 1/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 2 |
| board_minutes | 4 | 2/4 | 4/4 | 3/4 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 0/1 | 1 |
| born_digital | 23 | 9/22 | 20/22 | 17/23 | 5 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 0/2 | 1/2 | 0/2 | 2 |
| canada | 2 | 0/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 0/1 | 0/1 | 2 |
| certificate_of_service | 1 | 0/1 | 1/1 | 0/1 | 1 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 1/1 | 1/1 | 1/1 | 0 |
| client_vs_counsel | 1 | 0/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 38 | 12/38 | 29/38 | 23/38 | 13 |
| confirmation_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| copier_searchable_pdf | 2 | 1/2 | 1/2 | 2/2 | 1 |
| corporate_notice | 1 | 0/1 | 0/1 | 0/1 | 2 |
| council_minutes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| counsel_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 1/2 | 2/2 | 1/2 | 1 |
| customer_first | 4 | 1/4 | 2/4 | 2/4 | 1 |
| date_at_end | 5 | 1/5 | 3/5 | 4/5 | 2 |
| date_at_foot | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_at_top | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_range | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| day_first_date | 4 | 1/3 | 2/3 | 2/4 | 3 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 0 |
| deadline_in_body | 7 | 1/7 | 7/7 | 3/7 | 3 |
| dear_ceo_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| decided_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| docket_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 1/1 | 1/1 | 2/4 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 1/3 | 1/3 | 2/3 | 3 |
| enclosure | 1 | 0/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 0/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 1/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 1/1 | 1 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 1/1 | 0 |
| filled_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| first_name_salutation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 2/4 | 3/4 | 3/5 | 2 |
| footnotes | 3 | 1/3 | 3/3 | 2/3 | 0 |
| form | 6 | 1/3 | 2/3 | 3/6 | 2 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form_revision_date | 2 | 0/1 | 1/1 | 1/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 1/3 | 2/3 | 2/5 | 1 |
| government_sender | 1 | 0/1 | 0/1 | 0/1 | 2 |
| handwritten | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_annotations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 2/3 | 2/3 | 1 |
| handwritten_dates | 2 | 1/2 | 1/2 | 1/2 | 1 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 0/4 | 1/4 | 1/4 | 1 |
| historical | 2 | 1/2 | 2/2 | 2/2 | 0 |
| image | 4 | 0/4 | 2/4 | 2/4 | 0 |
| image_only | 4 | 1/3 | 1/3 | 3/4 | 2 |
| image_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| industry_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| internal_correspondence | 1 | 0/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 1/4 | 2/4 | 3/5 | 2 |
| ireland | 2 | 1/2 | 1/2 | 1/2 | 2 |
| irregular_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 1/1 | 1/1 | 1/2 | 1 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 1/1 | 1/1 | 0 |
| law_firm_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 5/18 | 15/18 | 11/18 | 4 |
| letter_format | 1 | 0/1 | 0/1 | 0/1 | 2 |
| letter_from_company | 3 | 1/3 | 2/3 | 1/3 | 0 |
| letter_from_counsel | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead | 9 | 1/9 | 7/9 | 4/9 | 3 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 1/2 | 0 |
| letterhead_logo | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 0/1 | 0/1 | 2 |
| line_item_dates | 2 | 0/2 | 2/2 | 0/2 | 0 |
| local_government | 1 | 1/1 | 1/1 | 1/1 | 0 |
| logo_issuer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| long_document | 4 | 3/4 | 4/4 | 4/4 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_dates | 8 | 4/8 | 7/8 | 6/8 | 1 |
| many_named_organisations | 3 | 2/3 | 3/3 | 2/3 | 0 |
| many_organisations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 1/4 | 3/4 | 3/4 | 1 |
| memo | 5 | 2/4 | 4/4 | 2/5 | 3 |
| memo_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| memorandum | 5 | 2/4 | 4/4 | 2/5 | 3 |
| metadata_date_trap | 14 | 5/10 | 7/10 | 9/14 | 6 |
| minutes | 6 | 4/6 | 6/6 | 5/6 | 0 |
| misprinted_year | 1 | 0/1 | 0/1 | 1/1 | 1 |
| mixed_scan_and_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 44 | 13/39 | 31/39 | 28/44 | 17 |
| month_only_date | 2 | 1/1 | 1/1 | 2/2 | 0 |
| month_only_references | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 0/2 | 1/2 | 2/2 | 1 |
| named_officials | 9 | 1/8 | 6/8 | 5/9 | 5 |
| named_person_at_org | 9 | 1/9 | 7/9 | 3/9 | 6 |
| no_addressee_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 4 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 1/3 | 4 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_signatory | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 2 | 1/2 | 1/2 | 1/2 | 1 |
| no_title | 1 | – | – | 0/1 | 2 |
| noisy_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 2 |
| nonprofit | 2 | 0/2 | 2/2 | 1/2 | 0 |
| notice | 11 | 2/11 | 6/11 | 9/11 | 6 |
| notice_of_decision | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 1/1 | 1 |
| notice_of_violation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| number_looks_like_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 2/12 | 9/12 | 5/12 | 7 |
| office | 4 | 1/1 | 1/1 | 2/4 | 2 |
| open_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| opinion_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 1/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 1/1 | 1/1 | 2/3 | 2 |
| printed_form | 1 | 0/1 | 1/1 | 0/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 0 |
| public_notice | 6 | 0/6 | 2/6 | 6/6 | 3 |
| purchase_order | 1 | 0/1 | 0/1 | 0/1 | 1 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| received_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 3 | 2/3 | 3/3 | 2/3 | 0 |
| reference_number | 11 | 1/11 | 9/11 | 6/11 | 6 |
| reference_stamp | 1 | 0/1 | 0/1 | 0/1 | 2 |
| referenced_case | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 4/6 | 6/6 | 5/6 | 0 |
| regulator | 5 | 1/5 | 5/5 | 3/5 | 1 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 0 |
| regulatory_filing | 1 | 0/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 1/1 | 0/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 0/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 2 |
| running_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 17 | 3/16 | 10/16 | 8/17 | 10 |
| scanned_signature_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| security_marking | 1 | – | – | 0/1 | 2 |
| senate_minutes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 0/2 | 2/2 | 2/3 | 1 |
| short | 8 | 1/7 | 3/7 | 6/8 | 5 |
| signatory_trap | 2 | 0/2 | 1/2 | 2/2 | 1 |
| signatory_vs_org | 3 | 3/3 | 3/3 | 3/3 | 0 |
| signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_on_later_page | 1 | 0/1 | 1/1 | 0/1 | 1 |
| signature_scan | 5 | 1/5 | 4/5 | 4/5 | 1 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 1/1 | 0/1 | 0 |
| source_title_mismatch | 3 | 1/3 | 3/3 | 2/3 | 0 |
| south_africa | 3 | 0/2 | 1/2 | 2/3 | 1 |
| staff_names | 8 | 4/8 | 8/8 | 6/8 | 0 |
| stamp_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamped_date | 3 | 0/3 | 1/3 | 0/3 | 3 |
| statement | 1 | – | – | 0/1 | 2 |
| statute_date | 1 | – | – | 1/1 | 0 |
| statutory_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| successor_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| table | 2 | 0/1 | 1/1 | 0/2 | 2 |
| table_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| table_of_contents | 2 | 2/2 | 2/2 | 2/2 | 0 |
| tables | 3 | 0/2 | 1/2 | 1/3 | 2 |
| tax_invoice | 2 | 1/2 | 2/2 | 1/2 | 0 |
| template_attachment | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 1/3 | 1 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| transmittal_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_day_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 2 | 0/1 | 1/1 | 0/2 | 2 |
| two_senders | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 1/2 | 2 |
| typed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 0/2 | 1/2 | 0/2 | 1 |
| university | 4 | 1/3 | 2/3 | 2/4 | 3 |
| university_foundation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_date_format | 4 | 2/4 | 2/4 | 3/4 | 2 |
| us_federal | 12 | 6/11 | 11/11 | 9/12 | 1 |
| us_local | 6 | 1/5 | 3/5 | 6/6 | 2 |
| us_state | 8 | 1/8 | 7/8 | 2/8 | 4 |
| warn_notice | 1 | 0/1 | 0/1 | 0/1 | 2 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| withdrawn | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 12.78 s | 28.20 s | 32.03 s |
| `extraction_wall_ms` | 50 | 317.0 ms | 3.80 s | 11.81 s |
| `worker_total_ms` | 50 | 315.8 ms | 3.80 s | 11.81 s |
| `worker_snapshot_ms` | 50 | 0.26 ms | 0.60 ms | 0.93 ms |
| `worker_parse_ms` | 50 | 5.2 ms | 51.4 ms | 310.1 ms |
| `worker_analysis_ms` | 50 | 0.70 ms | 4.8 ms | 6.8 ms |
| `worker_render_ms` | 50 | 0.00 ms | 490.2 ms | 1.23 s |
| `worker_image_decode_ms` | 50 | 0.00 ms | 27.8 ms | 51.0 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 6.08 s | 22.28 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 494.2 ms | 954.0 ms |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 5.36 s | 21.08 s |
| `worker_vision_ms` | 50 | 0.00 ms | 9.4 ms | 16.8 ms |
| `analyze_wall_ms` | 50 | 11.13 s | 22.41 s | 31.98 s |
| `distill_ms` | 50 | 2.2 ms | 26.4 ms | 39.3 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.06 ms | 0.07 ms |
| `inference_ms` | 50 | 11.13 s | 22.40 s | 31.92 s |
| `prefill_ms` | 50 | 6.21 s | 15.63 s | 26.95 s |
| `generation_ms` | 50 | 5.91 s | 7.50 s | 7.67 s |
| `validation_ms` | 50 | 5.6 ms | 19.6 ms | 32.1 ms |
| `naming_ms` | 50 | 0.23 ms | 3.3 ms | 5.1 ms |
| `index_ms` | 50 | 1.9 ms | 24.1 ms | 35.8 ms |
| `retrieval_ms` | 50 | 0.04 ms | 2.1 ms | 2.9 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 158.9 tok/s | 162.6 tok/s | 164.1 tok/s |
| `generation_tok_per_s` | 50 | 15.0 tok/s | 15.6 tok/s | 15.6 tok/s |
| `prompt_tokens` | 50 | 1003 | 2454 | 4079 |
| `cached_tokens` | 50 | 477 | 477 | 477 |
| `generated_tokens` | 50 | 88 | 111 | 114 |
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
| fast | 12 | 5.0 ms | 61.1 ms | 4.5 ms | 59.9 ms | 51.4 ms | 5.5 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 8.8 ms | 317.0 ms | 8.3 ms | 315.8 ms | 310.1 ms | 4.8 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.18 s | 1.95 s | 1.18 s | 1.95 s | 2.8 ms | 0.87 ms | 121.9 ms | 1.79 s |
| ocr | 22 | 2.20 s | 4.88 s | 2.20 s | 4.88 s | 29.8 ms | 3.5 ms | 818.7 ms | 8.96 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.18 s | 2.91 s | 1.18 s | 2.91 s | 14.8 ms | 0.87 ms | 490.2 ms | 2.88 s |
| 2-4 | 16 | 12.0 ms | 4.88 s | 11.4 ms | 4.88 s | 24.6 ms | 2.5 ms | 277.2 ms | 8.96 s |
| 5-9 | 6 | 19.8 ms | 11.81 s | 17.1 ms | 11.81 s | 310.1 ms | 4.5 ms | 1.23 s | 22.28 s |
| 10-24 | 4 | 22.2 ms | 2.28 s | 21.3 ms | 2.28 s | 51.4 ms | 6.8 ms | 421.0 ms | 3.68 s |
| 25-49 | 2 | 49.2 ms | 49.5 ms | 47.5 ms | 47.9 ms | 41.0 ms | 5.5 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 5.9 ms | 11.81 s | 5.5 ms | 11.81 s | 53.6 ms | 3.5 ms | 1.23 s | 22.28 s |
| invoice | 4 | 1.60 s | 2.20 s | 1.60 s | 2.20 s | 8.5 ms | 1.0 ms | 490.2 ms | 1.98 s |
| letter | 18 | 317.0 ms | 4.88 s | 315.8 ms | 4.88 s | 310.1 ms | 4.5 ms | 818.7 ms | 8.96 s |
| memo | 5 | 6.7 ms | 3.65 s | 6.3 ms | 3.65 s | 14.5 ms | 2.3 ms | 198.2 ms | 5.46 s |
| minutes | 6 | 49.2 ms | 2.28 s | 47.5 ms | 2.28 s | 41.0 ms | 6.8 ms | 421.0 ms | 3.68 s |
| notice | 11 | 1.05 s | 3.36 s | 1.05 s | 3.36 s | 22.3 ms | 2.0 ms | 214.5 ms | 3.18 s |
| statement | 1 | 890.7 ms | 890.7 ms | 890.4 ms | 890.4 ms | 0.44 ms | 0.15 ms | 103.9 ms | 774.1 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 428 MB | 1271 MB | real6-form-03 |
| Model server | 1576 MB | 1578 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (31)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_MISSING
- **real6-letter-05**: expected `1969-06-28 Letter to Lewis Research Center.pdf`, got `1969-06-28 Memorandum to George M. Low.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Opinion - Matthew E. Minick.pdf` · trap: party Matthew E. Minick (addressee, the county attorney as a person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice to Kalamazoo District Office.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Michelle Lujan Grisham.pdf` · trap: party Michelle Lujan Grisham (Governor who signs; the office is the sender and the office-holder is a trap, as in the earlier samples) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-09**: expected `2025-12-15 Comment Letter to City of San Juan Capistrano.pdf`, got `2025-12-15 Notice of Preparation of a Draft Environmental Impact Report to City of San Juan Capistrano.pdf` · review: LOW_CONFIDENCE
- **real6-letter-10**: expected `2025-06-23 Letter to Mike Lee.pdf`, got `2025-06-23 Letter to RE Garfield County.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-11**: expected `2023-11-14 Letter from Department of Public Expenditure NDP Delivery and Reform.pdf`, got `2023-05-19 Letter to Deirdre Malone.pdf` · trap: date 2023-05-19 (date the appointment took effect); party Deirdre Malone (the Accounting Officer appointed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter - John A. Fraser.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-14**: expected `2024-04-18 Letter from Ontario Energy Association.pdf`, got `2024-04-18 Letter to Norton Rose Fulbright Canada LLP.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document - Predman' WashingtonDe. Paly.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-17**: expected `1915-03-25 Letter from Poro College.jpg`, got `1915-03-25 Letter from A.H. PROP.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter to ACLU of Northern California.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-01**: expected `2007-12-06 Notice of Violation for Puyallup Tribe of Indians.pdf`, got `2007-12-06 Notice of Violation for Herman Dillon, Sr.pdf` · trap: party Herman Dillon, Sr. (tribal chairman and agent for service of process) · review: LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document - United States Department of the Interior.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Public Hearing Notice from Conway Board of Health.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-06**: expected `2026-06-12 Clerk's Alert from King County Superior Court Clerk's Office.pdf`, got `2026-06-19 Notice from King County Superior Court Clerk's Office.pdf` · trap: date 2026-06-19 (date the login requirement takes effect, also a court holiday) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real6-notice-08**: expected `2025-08-20 Revised Notice of Public Hearing and Participant Funding for NexGen Energy Ltd.pdf`, got `2025-08-20 Revised Notice of Public Hearing for CNSC.pdf` · review: LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `2026-06-16 Public Notice for midvaal.gov.za.jpg` · review: LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-10-10 Rapid Response - Ashley Gatlin.pdf` · trap: date 2025-10-10 (date of the layoff and the employees' last work date); party Ashley Gatlin (addressee, the department's Rapid Response program director) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting for North Carolina State University.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-02**: expected `2025-07-30 Tax Invoice from Department of Parliamentary Services.pdf`, got `2025-07-30 House Services Order Form for s 22.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document from Meade Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `2019-09-30 Federal Financial Report.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `Document - Michael S Brightwell.pdf` · trap: party Michael S Brightwell (department requestor named on the order) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Dianne Triplett.pdf` · trap: party Dianne Triplett (addressee of the attached letter, a person at the utility) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · **filed without review**
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order from Port of Vancouver USA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-05**: expected `2024-08-30 Board Meeting Minutes for Cal Poly Humboldt Foundation.pdf`, got `2024-08-30 MINUTES of the BOARD OF DIRECTORS for Robin Quigley.pdf` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

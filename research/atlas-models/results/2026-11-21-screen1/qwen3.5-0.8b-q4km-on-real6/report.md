# InternBench: live run

- **Run:** 2026-11-21T05:08:32Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 49 documents · 49 completed · gold `5d5d7ff470`
- **Wall time:** 436.15 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 6/44 (13.6%) |
| Document type | 33/49 (67.3%) |
| Date | 28/44 (63.6%) |
| Date role (when the reviewed date was chosen) | 4/27 (14.8%) |
| Parties | 21/49 (42.9%) |
| Relation word (when parties were named) | 25/43 (58.1%) |
| Party in the right role (when parties were named) | 13/33 (39.4%) |
| Ready / review routing | 6/29 (20.7%) |
| Description states nothing false | 49/49 (100.0%) |
| Description is specific | 23/49 (46.9%) |
| Description specificity | 46.3% (mean of 49) |
| Evidence recall (model's quotes) | 14.4% (mean of 45) |
| Digest recall (facts reaching the digest) | 88.9% (mean of 45) |
| Prompt recall (facts reaching the prompt sent) | 87.8% (mean of 45) |
| Sent to review | 100.0% of 49 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 0.0% | 5 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 87.8% | 49 |
| Review rate | 100.0% | 49 |
| Evidence recall | 14.4% | 45 |
| Total latency p50 | 7.29 s | 49 |
| Total latency p95 | 19.29 s | 49 |
| Generation latency p50 | 3.64 s | 49 |
| Generation latency p95 | 4.43 s | 49 |
| Generated tokens p50 | 100 | 49 |
| Generated tokens p95 | 122 | 49 |
| Prompt tokens p50 | 1003 | 49 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 44 |
| Trap date chosen | 5 of 49 |
| Forbidden party named | 13 of 49 |
| Spurious parties named | 27 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 104 (0.0%) |
| Right name sent to review anyway | 4 of 23 |
| Sent to review for an unsupported fact | 43 of 49 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 0/2 | 2/5 | 3/3 | 100.0% | 9.29 s | 21.96 s |
| invoice | 4 | 1/4 | 1/4 | 2/4 | 0/2 | 100.0% | 6.68 s | 11.83 s |
| letter | 18 | 3/18 | 13/18 | 8/18 | 0/11 | 100.0% | 7.21 s | 12.94 s |
| memo | 5 | 0/4 | 4/4 | 1/5 | 1/3 | 100.0% | 5.98 s | 10.94 s |
| minutes | 5 | 0/5 | 4/5 | 1/5 | 0/4 | 100.0% | 11.21 s | 19.71 s |
| notice | 11 | 2/11 | 6/11 | 7/11 | 1/5 | 100.0% | 6.83 s | 11.63 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 5.77 s | 5.77 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 0/4 | 1/4 | 0/4 | 0/1 | 100.0% | 6.76 s | 10.08 s |
| native | 24 | 4/23 | 18/23 | 12/24 | 1/18 | 100.0% | 6.83 s | 12.94 s |
| office | 4 | 0/1 | 0/1 | 2/4 | 3/3 | 100.0% | 5.48 s | 10.89 s |
| scan | 17 | 2/16 | 9/16 | 7/17 | 2/7 | 100.0% | 8.88 s | 21.96 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3/18 | 7/18 | 10/22 | 5/11 | 100.0% | 6.26 s | 10.00 s |
| 2-4 | 16 | 3/15 | 11/15 | 10/16 | 1/9 | 100.0% | 10.67 s | 19.29 s |
| 5-9 | 6 | 0/6 | 5/6 | 1/6 | 0/4 | 100.0% | 8.25 s | 21.96 s |
| 10-24 | 3 | 0/3 | 3/3 | 0/3 | 0/3 | 100.0% | 5.98 s | 7.21 s |
| 25-49 | 2 | 0/2 | 2/2 | 0/2 | 0/2 | 100.0% | 11.21 s | 19.71 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 5 | 0/5 | 5/5 | 0/5 | 0/5 | 0 of 5 | 7.21 s | 19.71 s | 3.78 s | 4.09 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 0/1 | 0/1 | 1/1 | 1 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 0/3 | 2/3 | 1/3 | 0 |
| addressee_is_office_holder | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 1/1 | 0 |
| agency_enforcement | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_guidance | 2 | 0/2 | 2/2 | 0/2 | 0 |
| agency_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 0/1 | 1/1 | 0/1 | 0 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 0/1 | 0/1 | 1/1 | 0 |
| attachment_with_own_date | 2 | 1/2 | 1/2 | 1/2 | 0 |
| attachments | 3 | 0/3 | 2/3 | 1/3 | 0 |
| attendance_list | 2 | 0/2 | 2/2 | 0/2 | 0 |
| attendee_list | 2 | 0/2 | 1/2 | 1/2 | 1 |
| australia | 5 | 2/4 | 2/4 | 4/5 | 2 |
| bilingual_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| billhead | 2 | 0/2 | 0/2 | 0/2 | 1 |
| blank_fields | 1 | – | – | 1/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 2 |
| board_minutes | 4 | 0/4 | 4/4 | 0/4 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 0/1 | 1 |
| born_digital | 22 | 3/21 | 17/21 | 11/22 | 6 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 0/2 | 1/2 | 1/2 | 1 |
| canada | 2 | 0/2 | 1/2 | 2/2 | 1 |
| caption_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| certificate_of_service | 1 | 0/1 | 1/1 | 0/1 | 1 |
| checkboxes | 1 | – | – | 1/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 0/1 | 0/1 | 1/1 | 1 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 37 | 5/37 | 23/37 | 16/37 | 12 |
| copier_searchable_pdf | 2 | 1/2 | 1/2 | 1/2 | 2 |
| corporate_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| counsel_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 2/2 | 0 |
| customer_first | 4 | 1/4 | 1/4 | 2/4 | 1 |
| date_at_end | 5 | 1/5 | 3/5 | 4/5 | 2 |
| date_at_foot | 1 | 0/1 | 0/1 | 0/1 | 2 |
| date_at_top | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_title | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_range | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| day_first_date | 4 | 1/3 | 1/3 | 3/4 | 3 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 0 |
| deadline_in_body | 7 | 1/7 | 6/7 | 3/7 | 3 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| docket_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 0/1 | 0/1 | 2/4 | 3 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 1/3 | 3/3 | 1/3 | 1 |
| enclosure | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 0/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 0/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 0/1 | 2 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 1/1 | 0 |
| filled_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 2/4 | 2/4 | 4/5 | 2 |
| footnotes | 3 | 1/3 | 3/3 | 2/3 | 0 |
| form | 6 | 0/3 | 0/3 | 3/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form_revision_date | 2 | 0/1 | 0/1 | 1/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 0/3 | 0/3 | 3/5 | 0 |
| government_sender | 1 | 0/1 | 0/1 | 0/1 | 1 |
| handwritten | 2 | 0/2 | 0/2 | 0/2 | 0 |
| handwritten_annotations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 2 | 0/2 | 1/2 | 1/2 | 1 |
| handwritten_dates | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 0/4 | 1/4 | 0/4 | 1 |
| historical | 2 | 1/2 | 1/2 | 2/2 | 0 |
| image | 4 | 0/4 | 1/4 | 0/4 | 1 |
| image_only | 4 | 0/3 | 1/3 | 2/4 | 2 |
| image_page | 1 | 0/1 | 0/1 | 1/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 1/4 | 1/4 | 2/5 | 2 |
| ireland | 2 | 0/2 | 1/2 | 0/2 | 1 |
| irregular_spacing | 1 | 0/1 | 0/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 1/1 | 0 |
| issuer_in_header_image | 2 | 0/1 | 0/1 | 1/2 | 2 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 0/1 | 0/1 | 1 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 0/1 | 0/1 | 1 |
| law_firm_letterhead | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letter | 18 | 3/18 | 13/18 | 8/18 | 5 |
| letter_format | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letter_from_company | 3 | 1/3 | 2/3 | 1/3 | 0 |
| letter_from_counsel | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letter_from_nonprofit | 1 | 0/1 | 1/1 | 0/1 | 0 |
| letterhead | 9 | 1/9 | 8/9 | 3/9 | 2 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 1/2 | 0 |
| letterhead_logo | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 1/1 | 0/1 | 1 |
| line_item_dates | 2 | 0/2 | 0/2 | 1/2 | 0 |
| logo_issuer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| long_document | 3 | 0/3 | 3/3 | 0/3 | 1 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 0/1 | 0/1 | 0 |
| many_dates | 7 | 0/7 | 6/7 | 2/7 | 0 |
| many_named_organisations | 3 | 1/3 | 3/3 | 1/3 | 0 |
| many_organisations | 1 | 0/1 | 0/1 | 1/1 | 1 |
| many_parties | 4 | 0/4 | 3/4 | 1/4 | 1 |
| memo | 5 | 0/4 | 4/4 | 1/5 | 2 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 0/4 | 4/4 | 1/5 | 2 |
| metadata_date_trap | 13 | 2/9 | 4/9 | 8/13 | 6 |
| minutes | 5 | 0/5 | 4/5 | 1/5 | 1 |
| misprinted_year | 1 | 0/1 | 0/1 | 0/1 | 2 |
| mixed_scan_and_digital | 1 | 0/1 | 0/1 | 0/1 | 0 |
| modern | 43 | 5/38 | 26/38 | 19/43 | 17 |
| month_only_date | 2 | 0/1 | 0/1 | 2/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 0/2 | 0/2 | 1/2 | 2 |
| named_officials | 9 | 1/8 | 7/8 | 3/9 | 4 |
| named_person_at_org | 9 | 1/9 | 7/9 | 3/9 | 5 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 4 |
| no_explicit_addressee | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 0/3 | 4 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_signatory | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_text_layer | 2 | 0/2 | 1/2 | 0/2 | 1 |
| no_title | 1 | – | – | 0/1 | 2 |
| noisy_ocr_layer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| nonprofit | 2 | 0/2 | 2/2 | 1/2 | 0 |
| notice | 11 | 2/11 | 6/11 | 7/11 | 6 |
| notice_of_decision | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 0/1 | 2 |
| notice_of_violation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 2/12 | 8/12 | 5/12 | 3 |
| office | 4 | 0/1 | 0/1 | 2/4 | 3 |
| open_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| opinion_letter | 1 | 0/1 | 0/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 0/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 0/1 | 1/1 | 0/3 | 2 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 1/1 | 0 |
| proprietor_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 0 |
| public_notice | 6 | 1/6 | 2/6 | 4/6 | 4 |
| purchase_order | 1 | 0/1 | 0/1 | 0/1 | 0 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 0 |
| received_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| redactions | 3 | 2/3 | 2/3 | 3/3 | 0 |
| reference_number | 11 | 1/11 | 8/11 | 4/11 | 6 |
| reference_stamp | 1 | 0/1 | 0/1 | 0/1 | 1 |
| referenced_case | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 1/6 | 4/6 | 3/6 | 1 |
| regulator | 5 | 0/5 | 5/5 | 2/5 | 1 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 0 |
| regulatory_filing | 1 | 0/1 | 0/1 | 1/1 | 1 |
| relative_date_reference | 1 | 0/1 | 1/1 | 0/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 0/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 2 |
| running_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 17 | 2/16 | 9/16 | 7/17 | 6 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| security_marking | 1 | – | – | 0/1 | 2 |
| senate_minutes | 1 | 0/1 | 0/1 | 1/1 | 1 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 0/2 | 0/2 | 0/3 | 2 |
| short | 8 | 1/7 | 3/7 | 5/8 | 6 |
| signatory_trap | 2 | 0/2 | 0/2 | 1/2 | 2 |
| signatory_vs_org | 3 | 1/3 | 2/3 | 2/3 | 0 |
| signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_on_later_page | 1 | 0/1 | 1/1 | 0/1 | 1 |
| signature_scan | 5 | 0/5 | 3/5 | 2/5 | 2 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 0/1 | 0/1 | 0 |
| source_title_mismatch | 3 | 1/3 | 3/3 | 1/3 | 0 |
| south_africa | 3 | 0/2 | 0/2 | 2/3 | 2 |
| staff_names | 7 | 0/7 | 4/7 | 1/7 | 2 |
| stamp_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| stamped_date | 3 | 0/3 | 1/3 | 1/3 | 1 |
| statement | 1 | – | – | 0/1 | 2 |
| statute_date | 1 | – | – | 1/1 | 0 |
| statutory_notice | 1 | 0/1 | 0/1 | 0/1 | 1 |
| successor_name | 2 | 0/2 | 0/2 | 0/2 | 1 |
| table | 2 | 0/1 | 0/1 | 0/2 | 2 |
| table_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| table_of_contents | 1 | 0/1 | 1/1 | 0/1 | 0 |
| tables | 3 | 0/2 | 1/2 | 0/3 | 1 |
| tax_invoice | 2 | 1/2 | 1/2 | 2/2 | 0 |
| template_attachment | 1 | 0/1 | 1/1 | 0/1 | 0 |
| third_party_named_in_body | 3 | 0/3 | 3/3 | 0/3 | 1 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 0/2 | 0/2 | 0 |
| transmittal_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| two_day_meeting | 1 | 0/1 | 1/1 | 0/1 | 0 |
| two_digit_year | 2 | 0/1 | 0/1 | 1/2 | 2 |
| two_senders | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_not_printed | 2 | 0/1 | 1/1 | 0/2 | 2 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 0/2 | 1/2 | 0/2 | 1 |
| university | 4 | 0/3 | 1/3 | 1/4 | 5 |
| university_foundation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_date_format | 4 | 1/4 | 2/4 | 2/4 | 0 |
| us_federal | 12 | 3/11 | 9/11 | 6/12 | 1 |
| us_local | 6 | 1/5 | 3/5 | 4/6 | 1 |
| us_state | 8 | 0/8 | 6/8 | 1/8 | 3 |
| warn_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| watermark | 1 | 0/1 | 0/1 | 1/1 | 0 |
| withdrawn | 1 | 0/1 | 0/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 136 · layout 23 · ocr 31 · ocr_regions 3
- **Documents by route class:** fast 12 · layout 13 · ocr 21 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 49 | 7.29 s | 19.29 s | 21.96 s |
| `extraction_wall_ms` | 49 | 520.6 ms | 3.63 s | 11.50 s |
| `worker_total_ms` | 49 | 520.2 ms | 3.63 s | 11.50 s |
| `worker_snapshot_ms` | 49 | 0.26 ms | 0.71 ms | 0.86 ms |
| `worker_parse_ms` | 49 | 4.8 ms | 45.2 ms | 2.53 s |
| `worker_analysis_ms` | 49 | 0.67 ms | 4.2 ms | 4.9 ms |
| `worker_render_ms` | 49 | 0.00 ms | 439.7 ms | 1.16 s |
| `worker_image_decode_ms` | 49 | 0.00 ms | 23.9 ms | 46.7 ms |
| `worker_ocr_ms` | 49 | 0.00 ms | 5.82 s | 21.76 s |
| `worker_ocr_encode_ms` | 49 | 0.00 ms | 471.6 ms | 983.4 ms |
| `worker_ocr_engine_ms` | 49 | 0.00 ms | 5.11 s | 20.53 s |
| `worker_vision_ms` | 49 | 0.00 ms | 8.7 ms | 15.4 ms |
| `analyze_wall_ms` | 49 | 6.67 s | 12.93 s | 19.66 s |
| `distill_ms` | 49 | 2.0 ms | 23.8 ms | 35.5 ms |
| `prompt_ms` | 49 | 0.03 ms | 0.06 ms | 0.06 ms |
| `inference_ms` | 49 | 6.67 s | 12.92 s | 19.60 s |
| `prefill_ms` | 49 | 3.31 s | 8.62 s | 15.51 s |
| `generation_ms` | 49 | 3.64 s | 4.43 s | 5.37 s |
| `validation_ms` | 49 | 4.1 ms | 21.2 ms | 25.3 ms |
| `naming_ms` | 49 | 0.21 ms | 2.9 ms | 4.6 ms |
| `index_ms` | 49 | 1.8 ms | 22.2 ms | 32.7 ms |
| `retrieval_ms` | 49 | 0.04 ms | 1.8 ms | 2.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 49 | 296.1 tok/s | 306.6 tok/s | 312.2 tok/s |
| `generation_tok_per_s` | 49 | 28.1 tok/s | 29.6 tok/s | 30.4 tok/s |
| `prompt_tokens` | 49 | 1003 | 2377 | 4079 |
| `cached_tokens` | 49 | 477 | 477 | 477 |
| `generated_tokens` | 49 | 100 | 122 | 133 |
| `estimated_prompt_tokens` | 49 | 1669 | 3189 | 3365 |
| `model_requests` | 49 | 1 | 1 | 1 |
| `redistillations` | 49 | 0 | 0 | 0 |
| `source_characters` | 49 | 2970 | 48744 | 68987 |
| `digest_characters` | 49 | 3190 | 7877 | 8432 |
| `prompt_characters` | 49 | 5239 | 9926 | 10487 |
| `worker_ocr_pages` | 49 | 0 | 2 | 8 |
| `worker_ocr_passes` | 49 | 0 | 2 | 8 |
| `worker_orientation_passes` | 49 | 0 | 2 | 8 |
| `worker_rendered_pixels` | 49 | 0 | 16830000 | 67320000 |
| `index_units` | 49 | 32 | 251 | 285 |
| `context_units` | 49 | 23 | 79 | 90 |
| `context_tokens` | 49 | 1081 | 2601 | 2777 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 12 | 4.3 ms | 54.3 ms | 3.9 ms | 53.2 ms | 45.2 ms | 4.9 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 8.0 ms | 2.54 s | 7.6 ms | 2.54 s | 2.53 s | 4.4 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.14 s | 1.88 s | 1.14 s | 1.88 s | 2.4 ms | 0.77 ms | 108.0 ms | 1.73 s |
| ocr | 21 | 2.10 s | 4.78 s | 2.10 s | 4.78 s | 23.0 ms | 2.7 ms | 750.1 ms | 8.72 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.14 s | 2.80 s | 1.14 s | 2.79 s | 12.4 ms | 0.85 ms | 439.7 ms | 2.77 s |
| 2-4 | 16 | 11.1 ms | 4.78 s | 10.6 ms | 4.78 s | 23.0 ms | 2.7 ms | 267.9 ms | 8.72 s |
| 5-9 | 6 | 16.0 ms | 11.50 s | 15.2 ms | 11.50 s | 2.53 s | 4.2 ms | 1.16 s | 21.76 s |
| 10-24 | 3 | 20.1 ms | 54.3 ms | 19.3 ms | 53.2 ms | 45.2 ms | 3.5 ms | 0.00 ms | 0.00 ms |
| 25-49 | 2 | 44.2 ms | 44.2 ms | 42.6 ms | 42.9 ms | 36.8 ms | 4.9 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 5.1 ms | 11.50 s | 4.7 ms | 11.50 s | 50.1 ms | 3.2 ms | 1.16 s | 21.76 s |
| invoice | 4 | 1.55 s | 2.10 s | 1.55 s | 2.10 s | 11.5 ms | 0.93 ms | 439.7 ms | 1.94 s |
| letter | 18 | 881.1 ms | 4.78 s | 880.7 ms | 4.78 s | 2.53 s | 4.2 ms | 750.1 ms | 8.72 s |
| memo | 5 | 6.0 ms | 3.57 s | 5.6 ms | 3.57 s | 9.9 ms | 1.7 ms | 181.8 ms | 5.44 s |
| minutes | 5 | 44.2 ms | 1.38 s | 42.6 ms | 1.37 s | 36.8 ms | 4.9 ms | 174.0 ms | 1.18 s |
| notice | 11 | 953.5 ms | 3.27 s | 952.8 ms | 3.27 s | 18.9 ms | 1.8 ms | 187.2 ms | 3.10 s |
| statement | 1 | 865.9 ms | 865.9 ms | 865.6 ms | 865.6 ms | 0.52 ms | 0.15 ms | 92.8 ms | 762.3 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 449 MB | 1249 MB | real6-form-03 |
| Model server | 854 MB | 856 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (38)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter.pdf` · review: TYPE_MISSING, PARTY_MISSING, LOW_CONFIDENCE
- **real6-letter-02**: expected `1980-05-12 Opinion Letter from Pension Benefit Guaranty Corporation.pdf`, got `Document from Pension Benefit Guaranty Corporation.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-01 Letter to The Members.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Letter from KEN PAXTON.pdf` · trap: party Ken Paxton (Attorney General who signs, a person) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for Matt Deh.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Michelle Lujan Grisham.pdf` · trap: party Michelle Lujan Grisham (Governor who signs; the office is the sender and the office-holder is a trap, as in the earlier samples) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-09**: expected `2025-12-15 Comment Letter to City of San Juan Capistrano.pdf`, got `2025-12-15 Letter to Laura Stokes.pdf` · trap: party Laura Stokes (addressee, a person at the City) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-10**: expected `2025-06-23 Letter to Mike Lee.pdf`, got `2025-06-23 Letter to Garfield County Colorado Board of County Commissioners.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-11**: expected `2023-11-14 Letter from Department of Public Expenditure NDP Delivery and Reform.pdf`, got `Letter to Deirdre Malone.pdf` · trap: party Deirdre Malone (the Accounting Officer appointed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to CEO.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter to John A. Fraser.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-14**: expected `2024-04-18 Letter from Ontario Energy Association.pdf`, got `2024-04-06 Letter from Norton Rose Fulbright Canada LLP.pdf` · trap: date 2024-04-06 (date of the firm's earlier letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document from Nashingon, DC.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-17**: expected `1915-03-25 Letter from Poro College.jpg`, got `1915-03-25 Letter from A.H. PROP.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter to ACLU of Northern California.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-01**: expected `2007-12-06 Notice of Violation for Puyallup Tribe of Indians.pdf`, got `2007-12-06 Notice of Violation for Herman Dillon, Sr. (Agent for Service of Process).pdf` · trap: party Herman Dillon, Sr. (tribal chairman and agent for service of process) · review: LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document - Meghan Hottel-Cox Secretary to the National Capital Planning Commission.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Public Hearing Notice from Conway Board of Health.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-07**: expected `2026-11-09 Notice of Exemption from San Bernardino County Department of Public Works.pdf`, got `Notice of Exemption from San Bernardino County.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-08**: expected `2025-08-20 Revised Notice of Public Hearing and Participant Funding for NexGen Energy Ltd.pdf`, got `2025-08-20 record for NexGen Energy Ltd.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `Public Notice for Betty Thobane.jpg` · trap: party Betty Thobane (finance official named as a contact) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-09-03 Rapid Response - Sara Welsh.pdf` · trap: party Sara Welsh (human resources contact named in the body) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting from Maggie Thompson Assistant Secretary.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces); party Maggie Thompson (Assistant Secretary who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-02**: expected `2025-07-30 Tax Invoice from Department of Parliamentary Services.pdf`, got `Document from Department of Parliamentary Services.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document from In every style known in theart, M ablalte.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `Federal Financial Report from Federal Agency and Organizational Element to Which Report is Submitted.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `Document from Fisher, Cindy.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-01**: expected `2004-06-01 Memorandum from Office of Government Ethics.pdf`, got `2004-06-01 Memorandum to Employee's Supervisor.pdf` · review: LOW_CONFIDENCE
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Office of Auditing and Performance Analysis.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 MEMORANDUM to Academic Deans.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: LOW_CONFIDENCE
- **real6-minutes-01**: expected `2026-08-18 Board Meeting Minutes for Federal Accounting Standards Advisory Board.pdf`, got `2026-08-18 Fasab Meeting Minutes for Federal GAAP Hierarchy.pdf` · review: TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order - Commissioner Burkman.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Board of Regents Meeting Record for Regent Manydeeds.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-05**: expected `2024-08-30 Board Meeting Minutes for Cal Poly Humboldt Foundation.pdf`, got `2024-08-30 MINUTES of the BOARD OF DIRECTORS for Robin Quigley.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-06**: expected `2025-10-08 Meeting Minutes for University Senate.docx`, got `2025-06-04 Document.docx` · trap: date 2025-06-04 (date of an earlier meeting whose minutes were approved) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

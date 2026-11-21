# InternBench: live run

- **Run:** 2026-11-21T05:09:42Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 778.03 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 10/45 (22.2%) |
| Document type | 36/50 (72.0%) |
| Date | 33/45 (73.3%) |
| Date role (when the reviewed date was chosen) | 7/32 (21.9%) |
| Parties | 29/50 (58.0%) |
| Relation word (when parties were named) | 24/45 (53.3%) |
| Party in the right role (when parties were named) | 19/35 (54.3%) |
| Ready / review routing | 7/30 (23.3%) |
| Description states nothing false | 50/50 (100.0%) |
| Description is specific | 28/50 (56.0%) |
| Description specificity | 54.7% (mean of 50) |
| Evidence recall (model's quotes) | 27.2% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 96.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 86.0% | 50 |
| Review rate | 96.0% | 50 |
| Evidence recall | 27.2% | 46 |
| Total latency p50 | 13.95 s | 50 |
| Total latency p95 | 29.83 s | 50 |
| Generation latency p50 | 4.87 s | 50 |
| Generation latency p95 | 6.24 s | 50 |
| Generated tokens p50 | 86 | 50 |
| Generated tokens p95 | 107 | 50 |
| Prompt tokens p50 | 918 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 45 |
| Trap date chosen | 6 of 50 |
| Forbidden party named | 12 of 50 |
| Spurious parties named | 20 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 130 (0.0%) |
| Right name sent to review anyway | 6 of 24 |
| Sent to review for an unsupported fact | 43 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 0/2 | 1/2 | 4/5 | 3/3 | 100.0% | 20.37 s | 31.24 s |
| invoice | 4 | 1/4 | 1/4 | 3/4 | 0/2 | 100.0% | 10.93 s | 18.68 s |
| letter | 18 | 4/18 | 14/18 | 11/18 | 0/11 | 100.0% | 13.13 s | 23.26 s |
| memo | 5 | 0/4 | 4/4 | 1/5 | 1/3 | 100.0% | 10.78 s | 17.12 s |
| minutes | 6 | 2/6 | 6/6 | 3/6 | 1/5 | 66.7% | 21.06 s | 36.28 s |
| notice | 11 | 3/11 | 7/11 | 7/11 | 1/5 | 100.0% | 11.36 s | 21.79 s |
| statement | 1 | – | – | 0/1 | 1/1 | 100.0% | 6.74 s | 6.74 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 0/4 | 2/4 | 2/4 | 0/1 | 100.0% | 9.64 s | 14.06 s |
| native | 25 | 7/24 | 19/24 | 15/25 | 2/19 | 96.0% | 13.22 s | 24.74 s |
| office | 4 | 0/1 | 1/1 | 1/4 | 3/3 | 75.0% | 10.50 s | 20.37 s |
| scan | 17 | 3/16 | 11/16 | 11/17 | 2/7 | 100.0% | 15.97 s | 31.24 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 5/18 | 10/18 | 13/22 | 5/11 | 100.0% | 10.00 s | 14.24 s |
| 2-4 | 16 | 1/15 | 12/15 | 9/16 | 1/9 | 93.8% | 17.13 s | 29.83 s |
| 5-9 | 6 | 1/6 | 5/6 | 3/6 | 0/4 | 100.0% | 15.74 s | 31.24 s |
| 10-24 | 4 | 1/4 | 4/4 | 2/4 | 0/4 | 100.0% | 10.78 s | 24.74 s |
| 25-49 | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 21.06 s | 36.28 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 3/6 | 6/6 | 4/6 | 1/6 | 0 of 6 | 13.22 s | 36.28 s | 4.61 s | 5.27 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 0/3 | 2/3 | 2/3 | 0 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 1/1 | 0 |
| agency_enforcement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_guidance | 2 | 0/2 | 2/2 | 0/2 | 0 |
| agency_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 0/1 | 1/1 | 0/1 | 0 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 0/1 | 0/1 | 1/1 | 0 |
| attachment_with_own_date | 2 | 0/2 | 1/2 | 2/2 | 0 |
| attachments | 3 | 0/3 | 2/3 | 2/3 | 0 |
| attendance_list | 3 | 2/3 | 3/3 | 3/3 | 0 |
| attendee_list | 2 | 0/2 | 2/2 | 0/2 | 1 |
| australia | 6 | 2/5 | 3/5 | 5/6 | 1 |
| bilingual_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 2 | 0/2 | 0/2 | 1/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 2/4 | 2 |
| board_minutes | 4 | 2/4 | 4/4 | 2/4 | 0 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 23 | 6/22 | 18/22 | 13/23 | 8 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 0/2 | 1/2 | 2/2 | 0 |
| canada | 2 | 2/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 38 | 9/38 | 28/38 | 23/38 | 13 |
| confirmation_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copier_searchable_pdf | 2 | 1/2 | 1/2 | 2/2 | 1 |
| corporate_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| council_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| counsel_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| cover_letter | 1 | 0/1 | 0/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 2/2 | 1 |
| customer_first | 4 | 1/4 | 1/4 | 3/4 | 1 |
| date_at_end | 5 | 1/5 | 3/5 | 3/5 | 2 |
| date_at_foot | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_at_top | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_in_title | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_range | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 0/1 | 1 |
| dated_line_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| day_first_date | 4 | 1/3 | 1/3 | 2/4 | 2 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 1 |
| deadline_in_body | 7 | 1/7 | 7/7 | 3/7 | 4 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| docket_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 0/1 | 1/1 | 1/4 | 3 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 0/3 | 2/3 | 2/3 | 1 |
| enclosure | 1 | 0/1 | 0/1 | 1/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 1/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 0/1 | 1 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 1/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 0/1 | 0/1 | 1/1 | 1 |
| file_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 1/1 | 0 |
| filled_form | 1 | 0/1 | 1/1 | 1/1 | 0 |
| first_name_salutation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| foi_release | 5 | 2/4 | 2/4 | 4/5 | 1 |
| footnotes | 3 | 0/3 | 3/3 | 2/3 | 1 |
| form | 6 | 1/3 | 2/3 | 5/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 0/1 | 1 |
| form_revision_date | 2 | 0/1 | 1/1 | 2/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 1/3 | 2/3 | 4/5 | 0 |
| government_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 2 | 0/2 | 0/2 | 1/2 | 0 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 2/3 | 3/3 | 0 |
| handwritten_dates | 2 | 1/2 | 1/2 | 2/2 | 0 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| hearing_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 0/4 | 1/4 | 2/4 | 1 |
| historical | 2 | 0/2 | 0/2 | 2/2 | 1 |
| image | 4 | 0/4 | 2/4 | 2/4 | 1 |
| image_only | 4 | 0/3 | 1/3 | 1/4 | 3 |
| image_page | 1 | 0/1 | 0/1 | 1/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 0/1 | 0/1 | 1/1 | 0 |
| invoice | 5 | 1/4 | 1/4 | 4/5 | 2 |
| ireland | 2 | 1/2 | 2/2 | 1/2 | 0 |
| irregular_spacing | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 0/1 | 1/1 | 0/2 | 2 |
| issuer_in_image | 1 | 0/1 | 0/1 | 0/1 | 1 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 0/1 | 1/1 | 0/1 | 1 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 4/18 | 14/18 | 11/18 | 6 |
| letter_format | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_from_company | 3 | 1/3 | 2/3 | 2/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead | 9 | 1/9 | 7/9 | 5/9 | 4 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 0/2 | 2 |
| letterhead_logo | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 0/1 | 1 |
| letterhead_vs_signer_entity | 1 | 0/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 2 | 0/2 | 1/2 | 2/2 | 0 |
| local_government | 1 | 0/1 | 1/1 | 1/1 | 0 |
| logo_issuer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| long_document | 4 | 3/4 | 4/4 | 4/4 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_dates | 8 | 3/8 | 7/8 | 6/8 | 0 |
| many_named_organisations | 3 | 1/3 | 3/3 | 2/3 | 1 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 2/4 | 3/4 | 4/4 | 0 |
| memo | 5 | 0/4 | 4/4 | 1/5 | 2 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 0/4 | 4/4 | 1/5 | 2 |
| metadata_date_trap | 14 | 4/10 | 7/10 | 8/14 | 5 |
| minutes | 6 | 2/6 | 6/6 | 3/6 | 1 |
| misprinted_year | 1 | 0/1 | 0/1 | 1/1 | 1 |
| mixed_scan_and_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 44 | 10/39 | 32/39 | 25/44 | 16 |
| month_only_date | 2 | 1/1 | 1/1 | 2/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 0/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 0/2 | 1/2 | 0/2 | 2 |
| named_officials | 9 | 2/8 | 6/8 | 4/9 | 5 |
| named_person_at_org | 9 | 4/9 | 9/9 | 6/9 | 3 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 3 |
| no_explicit_addressee | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 1/3 | 3 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_signatory | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_text_layer | 2 | 0/2 | 1/2 | 0/2 | 2 |
| no_title | 1 | – | – | 0/1 | 1 |
| noisy_ocr_layer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| nonprofit | 2 | 0/2 | 2/2 | 1/2 | 0 |
| notice | 11 | 3/11 | 7/11 | 7/11 | 6 |
| notice_of_decision | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 1/1 | 1 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 3/12 | 10/12 | 10/12 | 1 |
| office | 4 | 0/1 | 1/1 | 1/4 | 3 |
| open_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| opinion_letter | 1 | 0/1 | 0/1 | 1/1 | 1 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 1/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 0/1 | 1/1 | 1/3 | 2 |
| printed_form | 1 | 0/1 | 1/1 | 1/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| public_notice | 6 | 1/6 | 2/6 | 3/6 | 5 |
| purchase_order | 1 | 0/1 | 0/1 | 1/1 | 0 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 0 |
| received_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 1 |
| redactions | 3 | 2/3 | 2/3 | 3/3 | 0 |
| reference_number | 11 | 4/11 | 9/11 | 7/11 | 4 |
| reference_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 1/6 | 5/6 | 3/6 | 2 |
| regulator | 5 | 2/5 | 5/5 | 3/5 | 1 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 1 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 1/1 | 1/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 1/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 0/1 | 1 |
| running_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 17 | 3/16 | 11/16 | 11/17 | 5 |
| scanned_signature_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| seal_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| security_marking | 1 | – | – | 0/1 | 1 |
| senate_minutes | 1 | 0/1 | 1/1 | 0/1 | 1 |
| sender_at_end | 2 | 0/2 | 1/2 | 0/2 | 2 |
| service_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 0/2 | 2/2 | 2/3 | 2 |
| short | 8 | 1/7 | 3/7 | 3/8 | 6 |
| signatory_trap | 2 | 0/2 | 1/2 | 0/2 | 2 |
| signatory_vs_org | 3 | 0/3 | 2/3 | 2/3 | 2 |
| signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_on_later_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 1/5 | 4/5 | 3/5 | 2 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 0/1 | 1/1 | 1/1 | 0 |
| source_title_mismatch | 3 | 0/3 | 2/3 | 1/3 | 1 |
| south_africa | 3 | 0/2 | 1/2 | 0/3 | 2 |
| staff_names | 8 | 1/8 | 8/8 | 3/8 | 2 |
| stamp_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| stamped_date | 3 | 0/3 | 1/3 | 2/3 | 1 |
| statement | 1 | – | – | 0/1 | 1 |
| statute_date | 1 | – | – | 1/1 | 0 |
| statutory_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| successor_name | 2 | 0/2 | 0/2 | 1/2 | 1 |
| table | 2 | 0/1 | 1/1 | 1/2 | 1 |
| table_layout | 1 | 0/1 | 1/1 | 0/1 | 1 |
| table_of_contents | 2 | 1/2 | 2/2 | 2/2 | 0 |
| tables | 3 | 0/2 | 1/2 | 2/3 | 2 |
| tax_invoice | 2 | 1/2 | 1/2 | 2/2 | 0 |
| template_attachment | 1 | 0/1 | 1/1 | 0/1 | 0 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 1/3 | 1 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 0/2 | 2/2 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_day_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 2 | 0/1 | 0/1 | 1/2 | 1 |
| two_senders | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 0/2 | 1 |
| typed | 1 | 0/1 | 0/1 | 1/1 | 0 |
| typewritten | 2 | 0/2 | 1/2 | 1/2 | 1 |
| university | 4 | 0/3 | 2/3 | 1/4 | 4 |
| university_foundation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_date_format | 4 | 2/4 | 2/4 | 4/4 | 1 |
| us_federal | 12 | 2/11 | 9/11 | 8/12 | 3 |
| us_local | 6 | 1/5 | 3/5 | 4/6 | 3 |
| us_state | 8 | 2/8 | 7/8 | 5/8 | 3 |
| warn_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 0/1 | 0/1 | 1/1 | 1 |
| withdrawn | 1 | 0/1 | 0/1 | 1/1 | 1 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 13.95 s | 29.83 s | 36.28 s |
| `extraction_wall_ms` | 50 | 511.4 ms | 3.72 s | 11.61 s |
| `worker_total_ms` | 50 | 511.1 ms | 3.72 s | 11.61 s |
| `worker_snapshot_ms` | 50 | 0.26 ms | 0.53 ms | 0.88 ms |
| `worker_parse_ms` | 50 | 4.7 ms | 48.7 ms | 1.19 s |
| `worker_analysis_ms` | 50 | 0.69 ms | 4.5 ms | 7.2 ms |
| `worker_render_ms` | 50 | 0.00 ms | 459.2 ms | 1.19 s |
| `worker_image_decode_ms` | 50 | 0.00 ms | 19.8 ms | 46.5 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 6.07 s | 21.94 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 405.9 ms | 960.5 ms |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 5.47 s | 20.75 s |
| `worker_vision_ms` | 50 | 0.00 ms | 9.0 ms | 15.5 ms |
| `analyze_wall_ms` | 50 | 12.48 s | 24.36 s | 36.23 s |
| `distill_ms` | 50 | 1.9 ms | 24.1 ms | 35.8 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.06 ms | 0.06 ms |
| `inference_ms` | 50 | 12.47 s | 24.34 s | 36.18 s |
| `prefill_ms` | 50 | 7.76 s | 18.42 s | 32.26 s |
| `generation_ms` | 50 | 4.87 s | 6.24 s | 8.00 s |
| `validation_ms` | 50 | 4.6 ms | 18.3 ms | 25.7 ms |
| `naming_ms` | 50 | 0.21 ms | 2.9 ms | 4.5 ms |
| `index_ms` | 50 | 1.8 ms | 22.1 ms | 32.5 ms |
| `retrieval_ms` | 50 | 0.04 ms | 1.9 ms | 2.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 116.3 tok/s | 120.0 tok/s | 123.3 tok/s |
| `generation_tok_per_s` | 50 | 17.8 tok/s | 18.5 tok/s | 18.6 tok/s |
| `prompt_tokens` | 50 | 918 | 2196 | 3977 |
| `cached_tokens` | 50 | 469 | 490 | 490 |
| `generated_tokens` | 50 | 86 | 107 | 142 |
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
| fast | 12 | 4.3 ms | 58.3 ms | 3.9 ms | 57.1 ms | 48.7 ms | 4.9 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 8.1 ms | 1.20 s | 7.5 ms | 1.20 s | 1.19 s | 4.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 1.17 s | 1.90 s | 1.17 s | 1.90 s | 2.5 ms | 0.79 ms | 118.6 ms | 1.74 s |
| ocr | 22 | 2.14 s | 4.74 s | 2.14 s | 4.74 s | 26.8 ms | 3.2 ms | 751.5 ms | 8.62 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 1.17 s | 2.84 s | 1.17 s | 2.84 s | 15.1 ms | 0.83 ms | 459.2 ms | 2.82 s |
| 2-4 | 16 | 10.8 ms | 4.74 s | 10.4 ms | 4.74 s | 23.2 ms | 2.3 ms | 240.6 ms | 8.62 s |
| 5-9 | 6 | 15.9 ms | 11.61 s | 15.2 ms | 11.61 s | 1.19 s | 4.1 ms | 1.19 s | 21.94 s |
| 10-24 | 4 | 20.3 ms | 2.14 s | 19.5 ms | 2.14 s | 48.7 ms | 7.2 ms | 391.9 ms | 3.37 s |
| 25-49 | 2 | 44.4 ms | 44.6 ms | 43.1 ms | 43.1 ms | 37.1 ms | 4.9 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 5.1 ms | 11.61 s | 4.8 ms | 11.61 s | 48.7 ms | 3.2 ms | 1.19 s | 21.94 s |
| invoice | 4 | 1.55 s | 2.16 s | 1.55 s | 2.16 s | 9.5 ms | 0.91 ms | 459.2 ms | 1.94 s |
| letter | 18 | 858.4 ms | 4.74 s | 858.1 ms | 4.74 s | 1.19 s | 4.1 ms | 751.5 ms | 8.62 s |
| memo | 5 | 6.0 ms | 3.68 s | 5.6 ms | 3.68 s | 9.9 ms | 1.7 ms | 196.1 ms | 5.58 s |
| minutes | 6 | 44.4 ms | 2.14 s | 43.1 ms | 2.14 s | 37.1 ms | 7.2 ms | 391.9 ms | 3.37 s |
| notice | 11 | 1.00 s | 3.28 s | 1.00 s | 3.28 s | 19.2 ms | 1.7 ms | 187.5 ms | 3.12 s |
| statement | 1 | 854.1 ms | 854.1 ms | 853.8 ms | 853.8 ms | 0.51 ms | 0.14 ms | 93.6 ms | 749.9 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 462 MB | 1258 MB | real6-form-03 |
| Model server | 4426 MB | 4482 MB | real6-notice-08 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (35)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter from Power Systems & Supplies of Alaska.pdf` · trap: party Power Systems & Supplies of Alaska (applicant whose authorization the letter comments on; neither sender nor recipient) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-02**: expected `1980-05-12 Opinion Letter from Pension Benefit Guaranty Corporation.pdf`, got `2026-02-12 Document - PBGC.pdf` · trap: date 2026-02-12 (date PBGC withdrew the opinion letter, printed on the cover page and as a watermark) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-03**: expected `2002-11-01 Letter from Nevada Nuclear Waste Task Force.pdf`, got `2002-11-01 Letter for Judy Treichel.pdf` · trap: party Judy Treichel (Executive Director who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-04**: expected `2017-11-14 Letter to The President.pdf`, got `2017-11-14 Letter from The President.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real6-letter-05**: expected `1969-06-28 Letter to Lewis Research Center.pdf`, got `Letter to George M. Low.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Opinion from Ken Paxton.pdf` · trap: party Ken Paxton (Attorney General who signs, a person) · review: LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for Tracy Tibbitts.pdf` · trap: party Tracy Tibbitts (addressee, a person at the company) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document - Michelle Lujan Grisham.pdf` · trap: party Michelle Lujan Grisham (Governor who signs; the office is the sender and the office-holder is a trap, as in the earlier samples) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **real6-letter-10**: expected `2025-06-23 Letter to Mike Lee.pdf`, got `2025-06-23 Letter to Garfield County Colorado Board of County Commissioners.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to CEO.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter to Governor-General of the Commonwealth of Australia.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-17**: expected `1915-03-25 Letter from Poro College.jpg`, got `1915-03-25 Letter to Poro College.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter Tech Companies Must Resist Unlawful Subpoenas for User Data to ACLU of Northern California.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **real6-notice-03**: expected `2026-06-04 Notice of Exemption for Northeast Texas Railway Company.pdf`, got `2026-06-04 Document from Northeast Texas Railway Company.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document - Meghan Hottel-Cox.pdf` · trap: party Meghan Hottel-Cox (Secretary to the Commission, who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Public Hearing Notice for Kat Llamas.pdf` · trap: party Kat Llamas (Board of Health Chair who signs, a person) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-06**: expected `2026-06-12 Clerk's Alert from King County Superior Court Clerk's Office.pdf`, got `2026-06-19 Notice Regarding Changes to the KC Script Portal from King County Superior Court Clerk's Office.pdf` · trap: date 2026-06-19 (date the login requirement takes effect, also a court holiday) · review: LOW_CONFIDENCE
- **real6-notice-09**: expected `2026-06-16 Public Notice from Midvaal Local Municipality.jpg`, got `2026-06-16 Public Notice for Betty Thobane.jpg` · trap: party Betty Thobane (finance official named as a contact) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mayoral Chamber.pdf` · trap: date 2025-07-24 (date of the public hearings) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-11**: expected `2025-09-03 Notice of Mass Layoff from Perdue Foods LLC.pdf`, got `2025-09-03 Rapid Response from Perdue Foods LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-12**: expected `2024-03-13 Public Notice of Special Meeting from North Carolina State University.pdf`, got `2025-03-17 Public Notice of Special Meeting from NC State University Board of Trustees.pdf` · trap: date 2025-03-17 (date of the special meeting the notice announces) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-02**: expected `2025-07-30 Tax Invoice from Department of Parliamentary Services.pdf`, got `TAX INVOICE from Department of Parliamentary Services.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document from Meade Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-01**: expected `2019-12-23 Federal Financial Report from Secretary of the Commonwealth.pdf`, got `2019-09-30 Federal Financial Report from EAC HAVA Title 1, Section 101 Election Security Fund.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `Correct Order from West Department of Administration State of West Virginia.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-01**: expected `2004-06-01 Memorandum from Office of Government Ethics.pdf`, got `2004-06-01 Memorandum from Employee's Name, Title.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter from Office of Auditing and Performance Analysis.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order from Commissioner Burkman.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-07-16 Document from Shire of Woodanilling.pdf` · review: TYPE_UNSUPPORTED
- **real6-minutes-05**: expected `2024-08-30 Board Meeting Minutes for Cal Poly Humboldt Foundation.pdf`, got `2024-08-30 MINUTES of the BOARD OF DIRECTORS from Ken Fulgham.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-minutes-06**: expected `2025-10-08 Meeting Minutes for University Senate.docx`, got `2025-10-08 University Senate Meeting Minutes for Senate President Dyana Mason.docx` · trap: party Dyana Mason (Senate President) · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

# InternBench: live run

- **Run:** 2026-11-23T16:54:20Z · commit `70ce2164a1`
- **Machine:** AMD EPYC 9V45 96-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 50 documents · 50 completed · gold `7643340863`
- **Wall time:** 1724.81 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 23/45 (51.1%) |
| Document type | 39/50 (78.0%) |
| Date | 39/45 (86.7%) |
| Date role (when the reviewed date was chosen) | 20/38 (52.6%) |
| Parties | 35/50 (70.0%) |
| Relation word (when parties were named) | 35/45 (77.8%) |
| Party in the right role (when parties were named) | 32/39 (82.0%) |
| Ready / review routing | 8/30 (26.7%) |
| Description states nothing false | 50/50 (100.0%) |
| Description is specific | 33/50 (66.0%) |
| Description specificity | 60.7% (mean of 50) |
| Evidence recall (model's quotes) | 70.7% (mean of 46) |
| Digest recall (facts reaching the digest) | 89.1% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 88.0% (mean of 46) |
| Sent to review | 90.0% of 50 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 6 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 70.0% | 50 |
| Review rate | 90.0% | 50 |
| Evidence recall | 70.7% | 46 |
| Total latency p50 | 31.06 s | 50 |
| Total latency p95 | 56.60 s | 50 |
| Generation latency p50 | 14.40 s | 50 |
| Generation latency p95 | 17.06 s | 50 |
| Generated tokens p50 | 87 | 50 |
| Generated tokens p95 | 104 | 50 |
| Prompt tokens p50 | 1003 | 50 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 2 of 45 |
| Trap date chosen | 3 of 50 |
| Forbidden party named | 7 of 50 |
| Spurious parties named | 14 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 138 (0.0%) |
| Right name sent to review anyway | 11 of 24 |
| Sent to review for an unsupported fact | 35 of 50 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 1/2 | 2/2 | 2/5 | 3/3 | 100.0% | 37.58 s | 65.44 s |
| invoice | 4 | 2/4 | 2/4 | 2/4 | 1/2 | 75.0% | 27.77 s | 44.21 s |
| letter | 18 | 9/18 | 16/18 | 13/18 | 0/11 | 100.0% | 32.04 s | 50.52 s |
| memo | 5 | 0/4 | 4/4 | 1/5 | 1/3 | 100.0% | 28.70 s | 39.17 s |
| minutes | 6 | 3/6 | 6/6 | 6/6 | 1/5 | 83.3% | 49.23 s | 82.53 s |
| notice | 11 | 8/11 | 9/11 | 10/11 | 1/5 | 72.7% | 26.34 s | 50.81 s |
| statement | 1 | – | – | 1/1 | 1/1 | 100.0% | 13.31 s | 13.31 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 4 | 2/4 | 2/4 | 2/4 | 0/1 | 100.0% | 20.90 s | 28.30 s |
| native | 25 | 12/24 | 23/24 | 16/25 | 3/19 | 88.0% | 31.40 s | 56.60 s |
| office | 4 | 1/1 | 1/1 | 2/4 | 3/3 | 100.0% | 24.12 s | 45.25 s |
| scan | 17 | 8/16 | 13/16 | 15/17 | 2/7 | 88.2% | 33.28 s | 65.44 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 10/18 | 12/18 | 15/22 | 6/11 | 86.4% | 23.05 s | 32.34 s |
| 2-4 | 16 | 9/15 | 15/15 | 13/16 | 1/9 | 100.0% | 39.57 s | 65.44 s |
| 5-9 | 6 | 2/6 | 6/6 | 3/6 | 0/4 | 83.3% | 32.31 s | 55.78 s |
| 10-24 | 4 | 1/4 | 4/4 | 2/4 | 1/4 | 75.0% | 28.70 s | 56.60 s |
| 25-49 | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 49.23 s | 82.53 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 6 | 2/6 | 6/6 | 4/6 | 1/6 | 1 of 6 | 32.04 s | 82.53 s | 15.71 s | 16.29 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_office_holder | 1 | 0/1 | 0/1 | 1/1 | 0 |
| addressed_to_person | 3 | 2/3 | 2/3 | 2/3 | 0 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agencies_named_in_boilerplate | 1 | – | – | 0/1 | 0 |
| agency_enforcement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_guidance | 2 | 0/2 | 2/2 | 0/2 | 1 |
| agency_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| annotation_banner | 1 | 0/1 | 1/1 | 0/1 | 1 |
| appendices | 1 | 0/1 | 1/1 | 0/1 | 0 |
| applicant_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| approval_blocks_in_footer | 1 | – | – | 0/1 | 1 |
| attached_order_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachment_with_own_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| attachments | 3 | 0/3 | 3/3 | 1/3 | 1 |
| attendance_list | 3 | 1/3 | 3/3 | 3/3 | 0 |
| attendee_list | 2 | 2/2 | 2/2 | 2/2 | 0 |
| australia | 6 | 3/5 | 4/5 | 6/6 | 1 |
| bilingual_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 2 | 0/2 | 0/2 | 0/2 | 1 |
| blank_fields | 1 | – | – | 0/1 | 0 |
| blank_template | 4 | – | – | 1/4 | 2 |
| board_minutes | 4 | 2/4 | 4/4 | 4/4 | 0 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 23 | 10/22 | 21/22 | 14/23 | 6 |
| brand_names_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| brand_vs_legal_name | 2 | 2/2 | 2/2 | 2/2 | 0 |
| canada | 2 | 2/2 | 2/2 | 2/2 | 0 |
| caption_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 1 | 1/1 | 1/1 | 1/1 | 0 |
| checkboxes | 1 | – | – | 0/1 | 0 |
| class_of_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_vs_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| comment_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 38 | 21/38 | 34/38 | 30/38 | 6 |
| confirmation_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copier_searchable_pdf | 2 | 2/2 | 2/2 | 2/2 | 0 |
| corporate_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| council_minutes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| counsel_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cover_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 1/2 | 2/2 | 2/2 | 0 |
| customer_first | 4 | 2/4 | 2/4 | 2/4 | 1 |
| date_at_end | 5 | 2/5 | 3/5 | 5/5 | 1 |
| date_at_foot | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_title | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_range | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_without_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| day_first_date | 4 | 2/3 | 2/3 | 4/4 | 2 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 0/1 | 1 |
| deadline_in_body | 7 | 3/7 | 7/7 | 5/7 | 2 |
| dear_ceo_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decided_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deposit_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docket_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 4 | 1/1 | 1/1 | 2/4 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 3 | 2/3 | 3/3 | 2/3 | 0 |
| enclosure | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| encrypted_pdf | 1 | 0/1 | 1/1 | 0/1 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| engraved_portraits | 1 | 0/1 | 0/1 | 0/1 | 0 |
| event_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| event_date_prominent | 1 | 1/1 | 1/1 | 1/1 | 0 |
| file_number | 1 | 0/1 | 1/1 | 1/1 | 0 |
| fillable_form | 1 | – | – | 0/1 | 0 |
| filled_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_release | 5 | 3/4 | 3/4 | 5/5 | 1 |
| footnotes | 3 | 0/3 | 3/3 | 2/3 | 1 |
| form | 6 | 2/3 | 3/3 | 3/6 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 0/1 | 0 |
| form_revision_date | 2 | 1/1 | 1/1 | 1/2 | 0 |
| future_period | 1 | 0/1 | 1/1 | 0/1 | 1 |
| goods_for_third_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_form | 5 | 2/3 | 3/3 | 2/5 | 0 |
| government_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 2 | 0/2 | 0/2 | 0/2 | 0 |
| handwritten_annotations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 3 | 1/3 | 2/3 | 3/3 | 0 |
| handwritten_dates | 2 | 1/2 | 2/2 | 1/2 | 0 |
| handwritten_signature | 2 | 1/2 | 1/2 | 2/2 | 0 |
| hearing_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| hearing_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 4 | 1/4 | 1/4 | 1/4 | 1 |
| historical | 2 | 2/2 | 2/2 | 2/2 | 0 |
| image | 4 | 2/4 | 2/4 | 2/4 | 0 |
| image_only | 4 | 1/3 | 1/3 | 4/4 | 1 |
| image_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| industry_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| internal_correspondence | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 5 | 2/4 | 2/4 | 3/5 | 2 |
| ireland | 2 | 1/2 | 2/2 | 1/2 | 0 |
| irregular_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_header | 1 | – | – | 0/1 | 0 |
| issuer_in_header_image | 2 | 1/1 | 1/1 | 1/2 | 1 |
| issuer_in_image | 1 | 0/1 | 0/1 | 1/1 | 1 |
| issuer_in_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| issuer_in_logo_only | 1 | 0/1 | 1/1 | 0/1 | 1 |
| issuer_in_signature_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| issuer_not_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| jpeg | 1 | 1/1 | 1/1 | 1/1 | 0 |
| law_firm_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 18 | 9/18 | 16/18 | 13/18 | 3 |
| letter_format | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_company | 3 | 2/3 | 2/3 | 2/3 | 0 |
| letter_from_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter_from_nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead | 9 | 5/9 | 8/9 | 7/9 | 1 |
| letterhead_image_only | 2 | 0/2 | 2/2 | 0/2 | 1 |
| letterhead_logo | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_logo_only | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letterhead_vs_signer_entity | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| local_government | 1 | 0/1 | 1/1 | 1/1 | 0 |
| logo_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 2/4 | 4/4 | 4/4 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_dates | 8 | 3/8 | 8/8 | 6/8 | 0 |
| many_named_organisations | 3 | 1/3 | 3/3 | 2/3 | 1 |
| many_organisations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_parties | 4 | 2/4 | 4/4 | 3/4 | 0 |
| memo | 5 | 0/4 | 4/4 | 1/5 | 3 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memorandum | 5 | 0/4 | 4/4 | 1/5 | 3 |
| metadata_date_trap | 14 | 6/10 | 8/10 | 11/14 | 3 |
| minutes | 6 | 3/6 | 6/6 | 6/6 | 0 |
| misprinted_year | 1 | 1/1 | 1/1 | 1/1 | 0 |
| mixed_scan_and_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 44 | 20/39 | 36/39 | 32/44 | 9 |
| month_only_date | 2 | 1/1 | 1/1 | 1/2 | 0 |
| month_only_references | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| municipal | 2 | 1/2 | 1/2 | 2/2 | 1 |
| named_officials | 9 | 2/8 | 7/8 | 6/9 | 3 |
| named_person_at_org | 9 | 5/9 | 9/9 | 7/9 | 2 |
| no_addressee_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_document_date | 5 | – | – | 2/5 | 3 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 3 | 0/1 | 1/1 | 2/3 | 3 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_signatory | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 2 | 1/2 | 1/2 | 1/2 | 1 |
| no_title | 1 | – | – | 1/1 | 1 |
| noisy_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| nonprofit | 2 | 1/2 | 2/2 | 2/2 | 0 |
| notice | 11 | 8/11 | 9/11 | 10/11 | 1 |
| notice_of_decision | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice_of_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 12 | 7/12 | 12/12 | 11/12 | 0 |
| office | 4 | 1/1 | 1/1 | 2/4 | 2 |
| open_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| opinion_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid | 1 | 1/1 | 1/1 | 1/1 | 0 |
| partly_printed_date_line | 2 | 0/2 | 0/2 | 0/2 | 0 |
| personal_sender_on_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| placeholders | 3 | 0/1 | 1/1 | 1/3 | 3 |
| printed_form | 1 | 1/1 | 1/1 | 1/1 | 0 |
| prior_invoices_deducted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| procurement | 1 | – | – | 0/1 | 0 |
| proprietor_in_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_notice | 6 | 4/6 | 4/6 | 6/6 | 1 |
| purchase_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 1 |
| received_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 3 | 3/3 | 3/3 | 3/3 | 0 |
| reference_number | 11 | 6/11 | 11/11 | 7/11 | 3 |
| reference_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 1/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 2/6 | 6/6 | 3/6 | 2 |
| regulator | 5 | 2/5 | 5/5 | 4/5 | 0 |
| regulator_correspondence | 1 | 0/1 | 1/1 | 0/1 | 1 |
| regulatory_filing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 1/1 | 1/1 | 1/1 | 0 |
| resignation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| revised | 1 | 1/1 | 1/1 | 1/1 | 0 |
| row_dates | 1 | – | – | 1/1 | 1 |
| running_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 17 | 8/16 | 13/16 | 15/17 | 2 |
| scanned_signature_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| script_font_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| seal_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| security_marking | 1 | – | – | 1/1 | 1 |
| senate_minutes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_at_end | 2 | 0/2 | 1/2 | 1/2 | 1 |
| service_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 2/2 | 2/2 | 3/3 | 1 |
| short | 8 | 2/7 | 4/7 | 6/8 | 3 |
| signatory_trap | 2 | 1/2 | 1/2 | 2/2 | 1 |
| signatory_vs_org | 3 | 2/3 | 3/3 | 3/3 | 0 |
| signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_scan | 5 | 3/5 | 5/5 | 5/5 | 0 |
| signer_vs_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| small_print | 1 | 1/1 | 1/1 | 1/1 | 0 |
| source_title_mismatch | 3 | 3/3 | 3/3 | 3/3 | 0 |
| south_africa | 3 | 1/2 | 1/2 | 2/3 | 1 |
| staff_names | 8 | 5/8 | 8/8 | 7/8 | 0 |
| stamp_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamped_date | 3 | 0/3 | 2/3 | 1/3 | 1 |
| statement | 1 | – | – | 1/1 | 1 |
| statute_date | 1 | – | – | 0/1 | 0 |
| statutory_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| successor_name | 2 | 0/2 | 0/2 | 0/2 | 1 |
| table | 2 | 1/1 | 1/1 | 2/2 | 1 |
| table_layout | 1 | 0/1 | 1/1 | 0/1 | 0 |
| table_of_contents | 2 | 1/2 | 2/2 | 2/2 | 0 |
| tables | 3 | 0/2 | 2/2 | 2/3 | 1 |
| tax_invoice | 2 | 2/2 | 2/2 | 2/2 | 0 |
| template_attachment | 1 | 0/1 | 1/1 | 0/1 | 1 |
| third_party_named_in_body | 3 | 1/3 | 3/3 | 1/3 | 1 |
| through_line | 1 | – | – | 0/1 | 1 |
| trading_name | 2 | 0/2 | 1/2 | 0/2 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_day_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 2 | 1/1 | 1/1 | 2/2 | 1 |
| two_senders | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_in_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 2 | 0/1 | 1/1 | 2/2 | 1 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typewritten | 2 | 1/2 | 1/2 | 1/2 | 1 |
| university | 4 | 2/3 | 3/3 | 2/4 | 2 |
| university_foundation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_date_format | 4 | 2/4 | 4/4 | 3/4 | 0 |
| us_federal | 12 | 6/11 | 11/11 | 7/12 | 2 |
| us_local | 6 | 3/5 | 4/5 | 6/6 | 1 |
| us_state | 8 | 2/8 | 8/8 | 5/8 | 2 |
| warn_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| withdrawn | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 141 · layout 33 · ocr 32 · ocr_regions 4
- **Documents by route class:** fast 12 · layout 13 · ocr 22 · ocr_regions 3

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 50 | 31.06 s | 56.60 s | 82.53 s |
| `extraction_wall_ms` | 50 | 66.1 ms | 2.87 s | 7.61 s |
| `worker_total_ms` | 50 | 65.2 ms | 2.87 s | 7.61 s |
| `worker_snapshot_ms` | 50 | 0.28 ms | 0.56 ms | 0.95 ms |
| `worker_parse_ms` | 50 | 4.3 ms | 38.7 ms | 60.3 ms |
| `worker_analysis_ms` | 50 | 0.61 ms | 3.7 ms | 5.5 ms |
| `worker_render_ms` | 50 | 0.00 ms | 339.6 ms | 918.5 ms |
| `worker_image_decode_ms` | 50 | 0.00 ms | 13.3 ms | 34.2 ms |
| `worker_ocr_ms` | 50 | 0.00 ms | 4.65 s | 14.19 s |
| `worker_ocr_encode_ms` | 50 | 0.00 ms | 310.9 ms | 775.9 ms |
| `worker_ocr_engine_ms` | 50 | 0.00 ms | 4.16 s | 13.20 s |
| `worker_vision_ms` | 50 | 0.00 ms | 6.8 ms | 13.4 ms |
| `analyze_wall_ms` | 50 | 30.93 s | 54.79 s | 82.50 s |
| `distill_ms` | 50 | 1.8 ms | 19.4 ms | 27.5 ms |
| `prompt_ms` | 50 | 0.03 ms | 0.05 ms | 0.06 ms |
| `inference_ms` | 50 | 30.92 s | 54.76 s | 82.45 s |
| `prefill_ms` | 50 | 16.23 s | 40.77 s | 66.64 s |
| `generation_ms` | 50 | 14.40 s | 17.06 s | 18.12 s |
| `validation_ms` | 50 | 4.7 ms | 16.1 ms | 18.7 ms |
| `naming_ms` | 50 | 0.19 ms | 2.1 ms | 3.3 ms |
| `index_ms` | 50 | 1.5 ms | 18.1 ms | 26.2 ms |
| `retrieval_ms` | 50 | 0.03 ms | 1.6 ms | 2.1 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 50 | 60.8 tok/s | 62.9 tok/s | 63.1 tok/s |
| `generation_tok_per_s` | 50 | 6.1 tok/s | 6.3 tok/s | 6.4 tok/s |
| `prompt_tokens` | 50 | 1003 | 2454 | 4079 |
| `cached_tokens` | 50 | 477 | 477 | 477 |
| `generated_tokens` | 50 | 87 | 104 | 111 |
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
| fast | 12 | 4.0 ms | 46.2 ms | 3.7 ms | 45.3 ms | 38.7 ms | 3.9 ms | 0.00 ms | 0.00 ms |
| layout | 13 | 7.5 ms | 66.1 ms | 7.1 ms | 65.2 ms | 60.3 ms | 3.7 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 3 | 726.9 ms | 1.41 s | 726.0 ms | 1.41 s | 2.6 ms | 0.83 ms | 92.4 ms | 1.26 s |
| ocr | 22 | 1.23 s | 3.82 s | 1.23 s | 3.82 s | 25.7 ms | 2.8 ms | 583.3 ms | 6.88 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 726.9 ms | 2.18 s | 726.0 ms | 2.18 s | 21.6 ms | 0.83 ms | 339.6 ms | 2.05 s |
| 2-4 | 16 | 10.1 ms | 3.82 s | 9.7 ms | 3.82 s | 18.9 ms | 2.1 ms | 211.6 ms | 6.88 s |
| 5-9 | 6 | 13.9 ms | 7.61 s | 13.5 ms | 7.61 s | 60.3 ms | 3.7 ms | 918.5 ms | 14.19 s |
| 10-24 | 4 | 19.2 ms | 1.81 s | 18.4 ms | 1.81 s | 38.7 ms | 5.5 ms | 250.9 ms | 3.07 s |
| 25-49 | 2 | 34.8 ms | 35.0 ms | 33.7 ms | 33.9 ms | 29.0 ms | 3.9 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| form | 5 | 5.0 ms | 7.61 s | 4.6 ms | 7.61 s | 42.6 ms | 2.8 ms | 918.5 ms | 14.19 s |
| invoice | 4 | 929.3 ms | 1.23 s | 928.9 ms | 1.23 s | 14.4 ms | 0.90 ms | 339.6 ms | 1.05 s |
| letter | 18 | 66.1 ms | 3.82 s | 65.2 ms | 3.82 s | 60.3 ms | 3.7 ms | 583.3 ms | 6.88 s |
| memo | 5 | 5.5 ms | 2.71 s | 5.1 ms | 2.71 s | 8.8 ms | 1.6 ms | 141.1 ms | 3.99 s |
| minutes | 6 | 34.8 ms | 1.81 s | 33.7 ms | 1.81 s | 29.0 ms | 5.5 ms | 250.9 ms | 3.07 s |
| notice | 11 | 726.9 ms | 2.19 s | 726.0 ms | 2.19 s | 21.6 ms | 1.5 ms | 165.1 ms | 2.15 s |
| statement | 1 | 512.6 ms | 512.6 ms | 512.2 ms | 512.2 ms | 0.52 ms | 0.15 ms | 80.7 ms | 416.8 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 460 MB | 1284 MB | real6-form-03 |
| Model server | 6078 MB | 6084 MB | real6-minutes-06 |
| Benchmark (engine) | 15 MB | 18 MB | real6-minutes-06 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (22)

- **real6-letter-01**: expected `2020-03-23 Letter to National Marine Fisheries Service.pdf`, got `2020-03-23 Letter from Power Systems & Supplies of Alaska.pdf` · trap: party Power Systems & Supplies of Alaska (applicant whose authorization the letter comments on; neither sender nor recipient) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-04**: expected `2017-11-14 Letter to The President.pdf`, got `2017-11-14 Letter from The President.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real6-letter-06**: expected `2024-02-16 Attorney General Opinion to Hardin County Attorney.pdf`, got `2024-02-16 Opinion to Matthew E. Minick.pdf` · trap: party Matthew E. Minick (addressee, the county attorney as a person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-07**: expected `2019-05-02 Violation Notice to LECO Corporation.pdf`, got `2019-05-02 Violation Notice for IF Leco Corporation.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-08**: expected `2025-04-11 House Executive Message from Office of the Governor.pdf`, got `2025-04-11 Document to Javier Martínez.pdf` · trap: party Javier Martínez (Speaker of the House, addressed with the Members on the House's behalf) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-12**: expected `2024-04-24 Dear CEO Letter from Central Bank of Ireland.pdf`, got `2024-04-24 Letter to CEO.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real6-letter-13**: expected `2018-07-11 Resignation Letter from John A. Fraser.pdf`, got `Letter to His Excellency General The Hon Sir Peter Cosgrove AK MC (Retd) Governor-General.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-letter-16**: expected `1874-07-03 Letter from Freedman's Savings and Trust Company.jpg`, got `Document - Reh Docplals.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-letter-18**: expected `2026-02-05 Open Letter from ACLU of Northern California.pdf`, got `2026-02-05 Open Letter Tech Companies Must Resist Unlawful Subpoenas for User Data.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-04**: expected `2025-06-05 Commission Action for National Park Service.pdf`, got `2025-06-05 Document - Commemorative Works.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-notice-05**: expected `2025-04-23 Public Hearing Notice from Conway Board of Health.pdf`, got `Document from Conway Board of Health.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED
- **real6-notice-10**: expected `2025-07-09 Notice for Public Hearings from Mogale City Local Municipality.pdf`, got `2025-07-24 Notice for Public Hearings Uifwe Mpac from Mogale City.pdf` · trap: date 2025-07-24 (date of the public hearings) · **filed without review**
- **real6-invoice-04**: expected `1863-06-10 Bill from M. A. Meade.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real6-invoice-05**: expected `1906-03-14 Bill from Watkins-Cottrell Co.pdf`, got `Document from Fleming & Woodhouse Bros.pdf` · trap: party Fleming & Woodhouse Bros. (the customer) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real6-form-03**: expected `2026-04-06 Delivery Order to Matheny Motor Truck Co.pdf`, got `2026-04-06 Order from WEST Department of Administration.pdf` · review: LOW_CONFIDENCE
- **real6-memo-01**: expected `2004-06-01 Memorandum from Office of Government Ethics.pdf`, got `2004-06-01 Memorandum to Designated Agency Ethics Officials.pdf` · trap: party Designated Agency Ethics Officials (class of recipients named in the title) · review: PARTY_UNSUPPORTED
- **real6-memo-02**: expected `2024-10-16 Informational Memorandum from Farm Credit Administration.pdf`, got `2024-10-16 Informational Memorandum to Chair, Board of Directors.pdf` · review: LOW_CONFIDENCE
- **real6-memo-03**: expected `2024-01-08 Memorandum from Florida Public Service Commission.pdf`, got `2024-01-08 Letter to Adam J. Teitzman, Commission Clerk, Office of Commission Clerk.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real6-memo-05**: expected `2025-07-16 Memorandum from University of Florida.pdf`, got `2025-07-16 Memorandum to Academic Deans and Directors.pdf` · trap: party Academic Deans and Directors (class of addressees in the To line) · review: LOW_CONFIDENCE
- **real6-minutes-02**: expected `2025-09-23 Meeting Minutes for Port of Vancouver USA.pdf`, got `2025-09-23 Order from Port of Vancouver USA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real6-minutes-03**: expected `2025-06-05 Meeting Record for University of Wisconsin System Board of Regents.pdf`, got `2025-06-05 Record of the Regular Meeting of the from University of Wisconsin System Board of Regents.pdf` · review: LOW_CONFIDENCE
- **real6-minutes-04**: expected `2024-07-16 Ordinary Council Meeting Minutes for Shire of Woodanilling.pdf`, got `2024-07-16 Confirmed Minutes Confirmed Minutes from Shire of Woodanilling.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

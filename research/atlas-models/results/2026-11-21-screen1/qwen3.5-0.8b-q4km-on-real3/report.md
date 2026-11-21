# InternBench: live run

- **Run:** 2026-11-21T04:55:50Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 46 documents · 46 completed · gold `93cb0c4b95`
- **Wall time:** 567.80 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 12/45 (26.7%) |
| Document type | 36/46 (78.3%) |
| Date | 26/45 (57.8%) |
| Date role (when the reviewed date was chosen) | 6/26 (23.1%) |
| Parties | 23/46 (50.0%) |
| Relation word (when parties were named) | 28/45 (62.2%) |
| Party in the right role (when parties were named) | 12/21 (57.1%) |
| Ready / review routing | 4/29 (13.8%) |
| Description states nothing false | 46/46 (100.0%) |
| Description is specific | 29/46 (63.0%) |
| Description specificity | 59.4% (mean of 46) |
| Evidence recall (model's quotes) | 21.4% (mean of 46) |
| Digest recall (facts reaching the digest) | 83.7% (mean of 46) |
| Prompt recall (facts reaching the prompt sent) | 82.6% (mean of 46) |
| Sent to review | 100.0% of 46 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 100.0% | 2 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 93.5% | 46 |
| Review rate | 100.0% | 46 |
| Evidence recall | 21.4% | 46 |
| Total latency p50 | 10.56 s | 46 |
| Total latency p95 | 21.34 s | 46 |
| Generation latency p50 | 3.13 s | 46 |
| Generation latency p95 | 4.48 s | 46 |
| Generated tokens p50 | 96 | 46 |
| Generated tokens p95 | 138 | 46 |
| Prompt tokens p50 | 1077 | 46 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 45 |
| Trap date chosen | 4 of 46 |
| Forbidden party named | 14 of 46 |
| Spurious parties named | 22 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 103 (0.0%) |
| Right name sent to review anyway | 11 of 25 |
| Sent to review for an unsupported fact | 43 of 46 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 5 | 2/5 | 2/5 | 2/5 | 0/4 | 100.0% | 10.30 s | 15.37 s |
| letter | 18 | 3/18 | 12/18 | 7/18 | 0/10 | 100.0% | 10.22 s | 20.40 s |
| notice | 18 | 6/17 | 10/17 | 12/18 | 2/11 | 100.0% | 10.56 s | 34.64 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 6.60 s | 6.60 s |
| statement | 4 | 1/4 | 2/4 | 2/4 | 2/4 | 100.0% | 13.90 s | 20.56 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 2/6 | 0/6 | 0/1 | 100.0% | 6.60 s | 14.80 s |
| native | 16 | 8/15 | 13/15 | 11/16 | 1/13 | 100.0% | 11.39 s | 20.56 s |
| office | 11 | 3/11 | 5/11 | 7/11 | 1/7 | 100.0% | 8.36 s | 17.98 s |
| scan | 10 | 1/10 | 3/10 | 4/10 | 1/5 | 100.0% | 16.06 s | 34.64 s |
| text | 3 | 0/3 | 3/3 | 1/3 | 1/3 | 100.0% | 6.16 s | 17.99 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 2/19 | 9/19 | 7/19 | 3/12 | 100.0% | 6.85 s | 17.99 s |
| 2-4 | 20 | 6/19 | 11/19 | 11/20 | 1/12 | 100.0% | 11.37 s | 21.34 s |
| 5-9 | 5 | 2/5 | 4/5 | 3/5 | 0/3 | 100.0% | 19.71 s | 34.64 s |
| 10-24 | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 15.89 s | 17.43 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 0 of 2 | 15.89 s | 17.43 s | 3.58 s | 4.27 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_staff | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_billing | 4 | 3/4 | 3/4 | 4/4 | 0 |
| aging_table | 2 | 1/2 | 1/2 | 2/2 | 0 |
| all_caps_parties | 2 | 2/2 | 2/2 | 2/2 | 0 |
| attachment_with_own_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| attachments | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments_with_own_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bcc_distribution_list | 1 | 0/1 | 0/1 | 0/1 | 1 |
| billhead | 2 | 0/2 | 0/2 | 0/2 | 0 |
| blank_date | 2 | 0/2 | 0/2 | 0/2 | 1 |
| body_recounts_events | 7 | 4/6 | 5/6 | 5/7 | 2 |
| born_digital | 14 | 8/13 | 12/13 | 10/14 | 3 |
| brand_vs_legal_name | 4 | 3/4 | 3/4 | 4/4 | 0 |
| caption_court_differs_from_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_above_subject | 1 | 0/1 | 1/1 | 0/1 | 1 |
| cc_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| circular | 1 | 0/1 | 0/1 | 1/1 | 1 |
| coloured_paper | 2 | 0/2 | 0/2 | 0/2 | 1 |
| comparative_period | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 25 | 10/25 | 17/25 | 15/25 | 9 |
| consultation_response | 1 | 0/1 | 0/1 | 1/1 | 0 |
| copies_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| corrected_firm_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| corrected_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| court_filing | 3 | 3/3 | 3/3 | 3/3 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 0/2 | 2 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_first | 3 | 0/3 | 0/3 | 0/3 | 1 |
| date_at_end | 5 | 1/5 | 3/5 | 2/5 | 3 |
| date_in_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_missing_from_text_layer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_only_in_certificate | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_date | 4 | 2/4 | 2/4 | 3/4 | 1 |
| deadline_in_body | 5 | 1/4 | 2/4 | 2/5 | 4 |
| dear_colleague | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dear_sir_madam | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decision_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decision_numbers_look_like_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| direction | 1 | 0/1 | 0/1 | 1/1 | 0 |
| docket_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ecf_header_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enclosures_not_included | 1 | 0/1 | 1/1 | 1/1 | 0 |
| enforcement_notice | 1 | 0/1 | 0/1 | 0/1 | 1 |
| engraved_vignette | 1 | 0/1 | 0/1 | 0/1 | 0 |
| file_number | 1 | 0/1 | 1/1 | 0/1 | 0 |
| filing_stamps | 1 | 0/1 | 0/1 | 0/1 | 0 |
| flight_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| foi_response | 2 | 0/2 | 1/2 | 1/2 | 1 |
| footnote_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 0/2 | 0/2 | 1 |
| government_issuer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| handwritten | 3 | 0/3 | 0/3 | 0/3 | 0 |
| handwritten_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| header_fields | 1 | 0/1 | 1/1 | 1/1 | 0 |
| hearing_date_in_body | 2 | 0/2 | 1/2 | 2/2 | 1 |
| historic | 4 | 0/4 | 0/4 | 0/4 | 1 |
| historical | 3 | 0/3 | 2/3 | 0/3 | 2 |
| image | 3 | 0/3 | 2/3 | 0/3 | 2 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 0 |
| image_only_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| improvement_notice | 1 | 0/1 | 0/1 | 0/1 | 1 |
| inside_address_at_end | 1 | 0/1 | 0/1 | 0/1 | 2 |
| invoice | 5 | 2/5 | 2/5 | 2/5 | 1 |
| iso_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 3 | 0/3 | 0/3 | 0/3 | 0 |
| junk_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| legacy_doc | 2 | 0/2 | 0/2 | 2/2 | 1 |
| letter | 18 | 3/18 | 12/18 | 7/18 | 10 |
| letter_format | 2 | 0/2 | 1/2 | 1/2 | 1 |
| letterhead | 2 | 0/2 | 1/2 | 0/2 | 2 |
| letterhead_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| line_item_dates | 4 | 3/4 | 3/4 | 4/4 | 0 |
| live_date_field | 1 | 0/1 | 1/1 | 0/1 | 1 |
| local_government | 2 | 0/2 | 0/2 | 1/2 | 1 |
| lockbox | 2 | 2/2 | 2/2 | 2/2 | 0 |
| logo_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| many_dates | 3 | 2/3 | 2/3 | 3/3 | 0 |
| metadata_date_trap | 5 | 1/5 | 1/5 | 5/5 | 1 |
| metadata_title_trap | 3 | 2/3 | 2/3 | 2/3 | 1 |
| microfilm_targets | 1 | 0/1 | 0/1 | 0/1 | 1 |
| mislabelled_source | 1 | 0/1 | 0/1 | 1/1 | 0 |
| misleading_metadata | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 4 | 3/4 | 3/4 | 4/4 | 0 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_recipient | 1 | 0/1 | 0/1 | 1/1 | 0 |
| multiple_parties_side | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multiple_recipients | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_senders | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 9 | 2/9 | 6/9 | 4/9 | 6 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_title | 2 | 0/2 | 0/2 | 0/2 | 1 |
| no_issuer_in_text | 2 | 0/2 | 1/2 | 0/2 | 0 |
| no_letterhead_on_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_specific_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_title | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_title_word | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notice | 18 | 6/17 | 10/17 | 12/18 | 7 |
| notice_of_appeal | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_decision | 2 | 1/2 | 1/2 | 2/2 | 0 |
| notice_of_determination | 1 | 0/1 | 0/1 | 0/1 | 0 |
| notice_of_hearing | 2 | 0/2 | 1/2 | 2/2 | 1 |
| notice_of_removal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| notice_of_violation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_layer | 2 | 0/2 | 0/2 | 1/2 | 2 |
| ocr_text_layer | 2 | 0/2 | 1/2 | 1/2 | 1 |
| on_behalf_of_client | 3 | 1/3 | 1/3 | 2/3 | 2 |
| organization_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pgp_signed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| plain_text | 2 | 0/2 | 2/2 | 1/2 | 1 |
| posted_notice | 2 | 0/2 | 1/2 | 0/2 | 2 |
| print_field_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| printed_form | 2 | 0/2 | 0/2 | 0/2 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_notice | 4 | 0/3 | 1/3 | 1/4 | 4 |
| publication_number_year_differs | 1 | 0/1 | 1/1 | 0/1 | 1 |
| re_line_references_document | 3 | 1/3 | 2/3 | 1/3 | 2 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| receipted | 3 | 0/3 | 0/3 | 0/3 | 0 |
| redacted_addressee | 2 | 0/2 | 1/2 | 1/2 | 1 |
| redactions | 1 | 0/1 | 1/1 | 0/1 | 1 |
| reference_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_numbers_look_like_dates | 1 | 0/1 | 0/1 | 1/1 | 0 |
| relative_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| remit_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| remit_to | 5 | 3/5 | 3/5 | 4/5 | 1 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report_parameters | 1 | 1/1 | 1/1 | 1/1 | 0 |
| review | 2 | 0/2 | 1/2 | 0/2 | 0 |
| scanned | 12 | 1/12 | 4/12 | 5/12 | 8 |
| sender_block_above_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| sender_in_signoff_only | 1 | 0/1 | 0/1 | 1/1 | 0 |
| service_period | 2 | 1/2 | 1/2 | 1/2 | 0 |
| signatory_trap | 3 | 1/3 | 2/3 | 1/3 | 2 |
| signatory_vs_org | 1 | 0/1 | 0/1 | 0/1 | 0 |
| signature_on_later_page | 3 | 1/3 | 3/3 | 2/3 | 1 |
| signature_only_no_typed_name | 2 | 0/2 | 2/2 | 0/2 | 1 |
| small_print_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| spreadsheet | 1 | 0/1 | 0/1 | 0/1 | 0 |
| staff_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamp_date_only | 2 | 0/2 | 0/2 | 2/2 | 1 |
| stamp_dates | 2 | 0/2 | 1/2 | 0/2 | 2 |
| statement | 4 | 1/4 | 2/4 | 2/4 | 0 |
| subject_is_product | 1 | 0/1 | 0/1 | 1/1 | 0 |
| tables | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_subject | 1 | 0/1 | 1/1 | 0/1 | 1 |
| tiff | 1 | 0/1 | 1/1 | 0/1 | 0 |
| trading_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tribunal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| trustee_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 3 | 2/3 | 2/3 | 2/3 | 1 |
| two_letterhead_seals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_signatories | 1 | 0/1 | 1/1 | 0/1 | 1 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| typist_footer_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| typo_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| uk | 5 | 1/5 | 2/5 | 2/5 | 3 |
| undated | 1 | – | – | 0/1 | 1 |
| us_state | 5 | 0/4 | 2/4 | 2/5 | 3 |
| utility_commission | 1 | 0/1 | 1/1 | 0/1 | 1 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wide_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| windows-1252 | 1 | 0/1 | 1/1 | 0/1 | 0 |
| windows_1252_encoding | 1 | 0/1 | 1/1 | 0/1 | 1 |
| yearless_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| zero_amount | 2 | 1/2 | 1/2 | 2/2 | 0 |

## Routes

- **Pages:** fast 65 · layout 12 · ocr 38
- **Documents by route class:** fast 24 · layout 5 · ocr 17

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 46 | 10.56 s | 21.34 s | 34.64 s |
| `extraction_wall_ms` | 46 | 11.3 ms | 7.10 s | 21.37 s |
| `worker_total_ms` | 46 | 10.8 ms | 7.10 s | 21.37 s |
| `worker_snapshot_ms` | 46 | 0.26 ms | 1.3 ms | 3.4 ms |
| `worker_parse_ms` | 46 | 4.0 ms | 30.4 ms | 78.4 ms |
| `worker_analysis_ms` | 46 | 0.21 ms | 1.9 ms | 6.4 ms |
| `worker_render_ms` | 46 | 0.00 ms | 752.3 ms | 3.00 s |
| `worker_image_decode_ms` | 46 | 0.00 ms | 72.9 ms | 1.38 s |
| `worker_ocr_ms` | 46 | 0.00 ms | 12.48 s | 41.68 s |
| `worker_ocr_encode_ms` | 46 | 0.00 ms | 901.1 ms | 2.67 s |
| `worker_ocr_engine_ms` | 46 | 0.00 ms | 11.52 s | 38.63 s |
| `worker_vision_ms` | 46 | 0.00 ms | 12.5 ms | 34.6 ms |
| `analyze_wall_ms` | 46 | 9.47 s | 19.46 s | 20.56 s |
| `distill_ms` | 46 | 2.5 ms | 17.8 ms | 31.1 ms |
| `prompt_ms` | 46 | 0.04 ms | 0.08 ms | 0.09 ms |
| `inference_ms` | 46 | 9.46 s | 19.45 s | 20.54 s |
| `prefill_ms` | 46 | 6.45 s | 16.14 s | 17.40 s |
| `generation_ms` | 46 | 3.13 s | 4.48 s | 6.63 s |
| `validation_ms` | 46 | 7.6 ms | 26.4 ms | 32.0 ms |
| `naming_ms` | 46 | 0.26 ms | 1.7 ms | 3.3 ms |
| `index_ms` | 46 | 2.2 ms | 13.1 ms | 26.5 ms |
| `retrieval_ms` | 46 | 0.04 ms | 1.4 ms | 4.1 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 46 | 164.9 tok/s | 169.3 tok/s | 170.5 tok/s |
| `generation_tok_per_s` | 46 | 30.8 tok/s | 32.8 tok/s | 33.6 tok/s |
| `prompt_tokens` | 46 | 1077 | 2586 | 2778 |
| `cached_tokens` | 46 | 477 | 477 | 477 |
| `generated_tokens` | 46 | 96 | 138 | 204 |
| `estimated_prompt_tokens` | 46 | 1718 | 3006 | 3315 |
| `model_requests` | 46 | 1 | 1 | 1 |
| `redistillations` | 46 | 0 | 0 | 0 |
| `source_characters` | 46 | 2854 | 22421 | 45033 |
| `digest_characters` | 46 | 3129 | 7884 | 8158 |
| `prompt_characters` | 46 | 5178 | 9933 | 10207 |
| `worker_ocr_pages` | 46 | 0 | 4 | 8 |
| `worker_ocr_passes` | 46 | 0 | 4 | 9 |
| `worker_orientation_passes` | 46 | 0 | 4 | 8 |
| `worker_rendered_pixels` | 46 | 0 | 33660000 | 67578072 |
| `index_units` | 46 | 28 | 137 | 464 |
| `context_units` | 46 | 25 | 74 | 112 |
| `context_tokens` | 46 | 1129 | 2419 | 2727 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 24 | 3.6 ms | 27.5 ms | 3.1 ms | 25.3 ms | 25.0 ms | 1.0 ms | 0.00 ms | 0.00 ms |
| layout | 5 | 7.9 ms | 52.1 ms | 7.4 ms | 50.4 ms | 42.4 ms | 6.4 ms | 0.00 ms | 0.00 ms |
| ocr | 17 | 2.44 s | 21.37 s | 2.44 s | 21.37 s | 30.4 ms | 1.8 ms | 3.00 s | 41.68 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 7.9 ms | 5.48 s | 7.4 ms | 5.48 s | 30.4 ms | 2.3 ms | 176.2 ms | 4.03 s |
| 2-4 | 20 | 7.6 ms | 6.56 s | 6.2 ms | 6.56 s | 28.8 ms | 1.4 ms | 710.9 ms | 12.48 s |
| 5-9 | 5 | 24.2 ms | 21.37 s | 23.4 ms | 21.37 s | 20.4 ms | 1.9 ms | 3.00 s | 41.68 s |
| 10-24 | 2 | 27.5 ms | 52.1 ms | 25.3 ms | 50.4 ms | 42.4 ms | 6.4 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 5 | 458.9 ms | 2.44 s | 458.5 ms | 2.44 s | 18.2 ms | 2.3 ms | 176.2 ms | 2.41 s |
| letter | 18 | 18.7 ms | 7.10 s | 18.1 ms | 7.10 s | 78.4 ms | 1.9 ms | 1.54 s | 12.48 s |
| notice | 18 | 7.7 ms | 21.37 s | 7.4 ms | 21.37 s | 42.4 ms | 6.4 ms | 3.00 s | 41.68 s |
| receipt | 1 | 1.78 s | 1.78 s | 1.77 s | 1.77 s | 10.7 ms | 0.07 ms | 0.00 ms | 1.74 s |
| statement | 4 | 3.0 ms | 2.32 s | 2.2 ms | 2.32 s | 4.4 ms | 1.4 ms | 129.1 ms | 2.16 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 207 MB | 1276 MB | real3-notice-02 |
| Model server | 856 MB | 856 MB | real3-office-15 |
| Benchmark (engine) | 15 MB | 21 MB | real3-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (33)

- **real3-letter-02**: expected `2011-02-18 Reminder Letter to Panther Motors, Inc.pdf`, got `2011-02-18 Letter to Dominick Livoti NVS-215KJS President 09V-379 Panther Motors, Inc.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-03**: expected `2023-03-28 Decision Letter to Good Energy.pdf`, got `2023-03-28 Letter for Good Electricity Gas Limited.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-04**: expected `2010-07-23 Approval Letter to DG ENER.pdf`, got `2010-07-23 Letter to Philip Lowe.pdf` · trap: party Philip Lowe (addressee, Director General of DG ENER) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-06**: expected `1976-11-18 Letter to Los Alamos Scientific Laboratory.pdf`, got `1976-11-18 Document - D.R. Harbur.pdf` · trap: party D.R. Harbur (addressee at LASL) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-07**: expected `1966-01-04 Letter to U.S. Atomic Energy Commission.pdf`, got `Document - Paul G. Holsted.pdf` · trap: party Paul G. Holsted (addressee, AEC division director) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-08**: expected `1995-12-07 Decision Letter to Bloom & Kreten.pdf`, got `1995-12-08 Letter to Marybeth Peters.pdf` · trap: date 1995-12-08 (typist's footer date); party Marybeth Peters (Register of Copyrights who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-09**: expected `1992-12-24 Objection Letter to Pilcher and Pilcher.pdf`, got `Letter to Pilcher and Pilcher.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-10**: expected `2019-03-28 Response Letter to Transport Scotland.pdf`, got `2019-03-28 Letter to John Larkinson.pdf` · trap: party John Larkinson (ORR chief executive who signs (named above the addressee)) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-11**: expected `1907-12-26 Acknowledgment Letter to Wright Brothers.tif`, got `1907-12-26 Letter from WASHINGTON.tif` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-14**: expected `2005-12-14 Dear Colleague Letter from National Science Foundation.txt`, got `2005-12-14 Letter to Arden L. Bement, Jr.txt` · trap: party Arden L. Bement, Jr. (NSF Director who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-01**: expected `2024-02-19 Improvement Notice for The Chiltern Railway Company Limited.pdf`, got `Improvement Notice from John Giles Turner.pdf` · trap: party John Giles Turner (ORR inspector who signs and serves the notice) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-02**: expected `2007-04-24 Notice of Public Site Visit and Informational Hearing for City of Victorville.pdf`, got `2007-04-24 Application for Certification for the Victorville for Victorville 2 Hybrid Power Project.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-04**: expected `2006-02-23 Notice of Determination from City of Industry.pdf`, got `Notice of Determination for County Clerk County of Los Angeles.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-05**: expected `2008-09-22 Notice of Hearing from Mojave Desert Air Quality Management District.pdf`, got `2008-10-27 Notice of Hearing for Mojave Desert Air Quality Management District.pdf` · trap: date 2008-10-27 (date of the public hearing) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-07**: expected `2023-04-07 Notice of Violation for PacifiCorp.pdf`, got `2023-04-07 Notice of Violation for Shelley McCoy.pdf` · trap: party Shelley McCoy (PacifiCorp's Director, Regulation, to whom the letter is addressed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-11**: expected `2023-11-13 Revocation of Product Approval for Atlantis Hydrotec.pdf`, got `Letter to Water Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-12**: expected `1884-12-29 Notice for William Dew.jpg`, got `1884-11-27 Animals Order from R. F. HOU LSTON.jpg` · trap: date 1884-11-27 (date the pigstye was earlier declared infected) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_IMPLAUSIBLE, LOW_CONFIDENCE
- **real3-notice-13**: expected `1831-11-01 Notice from Parish of Clerkenwell.jpg`, got `1831-11-01 Document - T. Goode, Printer, Cross Street, Wilderness Row.jpg` · trap: party T. Goode (printer) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-15**: expected `2026-10-14 Errata Notice from The FreeBSD Project.txt`, got `2026-10-14 Errata Notice from FreeBSD Project.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-05**: expected `2025-12-05 Statement of Account from Lotus Communications Corp.pdf`, got `Statement of Account from Lotus Communications Corp.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-09**: expected `1916-11-04 Invoice from Mergenthaler Linotype Co.pdf`, got `Invoice from Panama City PUB Co.pdf` · trap: party Panama City Pub Co (customer, typed above the issuer) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-11**: expected `1850-07-11 Bill from Henry D. Gray & Son.jpg`, got `Document - Raisins, and other foreign Fruits. CIGARS, Havana and American.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-12**: expected `1848-11-14 Bill from Veranda Hotel.jpg`, got `Document from Mr. 29. Wonior.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-15**: expected `1848-07-01 Receipt from Post-Office, Duxbury, Mass.jpg`, got `Document from Mr.Dan-L. Wisov.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-office-01**: expected `2026-09-14 Consultation Response to London Borough of Islington.docx`, got `Letter to Thames Water.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-02**: expected `2018-11-21 Remit Letter to NHS Pay Review Body.docx`, got `Letter from Matt Hancock.docx` · trap: party Matt Hancock (Secretary of State who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-03**: expected `2016-03-23 Letter of Concern to Central Bedfordshire Council.docx`, got `2016-03-23 Letter to Andrew Cook, HMI.docx` · trap: party Andrew Cook (HMI Regional Director who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-04**: expected `2017-08-24 Freedom of Information Response from Office of the Advocate General for Scotland.docx`, got `2017-08-24 Letter to Information Commissioner.docx` · trap: party Information Commissioner's Office (complaints address) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-05**: expected `2011-10-01 Direction for Ofcom.doc`, got `Direction to Ofcom to Impose a Notification Condition from Secretary of State for Business, Innovation and Skills.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-06**: expected `2014-02-20 Circular from Home Office.doc`, got `2014-02-13 report of the Winsor Review for Home Secretary.doc` · trap: date 2014-02-13 (implementation date and date of the Home Secretary's announcement) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-08**: expected `2021-05-14 Freedom of Information Response from Office of Manpower Economics.odt`, got `Letter to Office of Manpower Economics.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-12**: expected `2012-09-30 Statement of Financial Position from Province of British Columbia.xlsx`, got `Document - Q2 BS table.xlsx` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-14**: expected `2017-03-31 Condensed Statement of Financial Position from Administrative Tribunals Support Service of Canada.csv`, got `2017-03-31 Document.csv` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

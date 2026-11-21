# InternBench: live run

- **Run:** 2026-11-21T04:58:41Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 45 documents · 45 completed · gold `44c28f94c5`
- **Wall time:** 1108.51 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 9/44 (20.4%) |
| Document type | 38/45 (84.4%) |
| Date | 30/44 (68.2%) |
| Date role (when the reviewed date was chosen) | 12/30 (40.0%) |
| Parties | 24/45 (53.3%) |
| Relation word (when parties were named) | 33/40 (82.5%) |
| Party in the right role (when parties were named) | 11/24 (45.8%) |
| Ready / review routing | 5/28 (17.9%) |
| Description states nothing false | 45/45 (100.0%) |
| Description is specific | 31/45 (68.9%) |
| Description specificity | 61.5% (mean of 45) |
| Evidence recall (model's quotes) | 33.7% (mean of 45) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 45) |
| Prompt recall (facts reaching the prompt sent) | 82.2% (mean of 45) |
| Sent to review | 97.8% of 45 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 2 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 91.1% | 45 |
| Review rate | 97.8% | 45 |
| Evidence recall | 33.7% | 45 |
| Total latency p50 | 23.09 s | 45 |
| Total latency p95 | 45.20 s | 45 |
| Generation latency p50 | 4.16 s | 45 |
| Generation latency p95 | 5.91 s | 45 |
| Generated tokens p50 | 88 | 45 |
| Generated tokens p95 | 122 | 45 |
| Prompt tokens p50 | 997 | 45 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 44 |
| Trap date chosen | 4 of 45 |
| Forbidden party named | 10 of 45 |
| Spurious parties named | 16 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 109 (0.0%) |
| Right name sent to review anyway | 6 of 24 |
| Sent to review for an unsupported fact | 41 of 45 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 5 | 1/5 | 1/5 | 1/5 | 1/4 | 80.0% | 15.92 s | 29.31 s |
| letter | 17 | 5/17 | 15/17 | 8/17 | 0/9 | 100.0% | 22.49 s | 38.74 s |
| notice | 18 | 2/17 | 11/17 | 14/18 | 2/11 | 100.0% | 23.55 s | 51.88 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 8.60 s | 8.60 s |
| statement | 4 | 1/4 | 3/4 | 1/4 | 2/4 | 100.0% | 24.13 s | 40.67 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 1/6 | 3/6 | 1/6 | 0/1 | 100.0% | 9.34 s | 23.09 s |
| native | 16 | 3/15 | 11/15 | 10/16 | 2/13 | 93.8% | 23.55 s | 45.20 s |
| office | 10 | 2/10 | 8/10 | 7/10 | 1/6 | 100.0% | 15.20 s | 38.74 s |
| scan | 10 | 2/10 | 5/10 | 4/10 | 1/5 | 100.0% | 28.70 s | 51.88 s |
| text | 3 | 1/3 | 3/3 | 2/3 | 1/3 | 100.0% | 12.72 s | 42.75 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4/19 | 11/19 | 10/19 | 4/12 | 94.7% | 12.72 s | 42.75 s |
| 2-4 | 19 | 3/18 | 14/18 | 9/19 | 1/11 | 100.0% | 25.37 s | 47.49 s |
| 5-9 | 5 | 1/5 | 3/5 | 3/5 | 0/3 | 100.0% | 37.82 s | 51.88 s |
| 10-24 | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 34.81 s | 38.74 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 0 of 2 | 34.81 s | 38.74 s | 3.86 s | 4.10 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_staff | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_billing | 4 | 2/4 | 3/4 | 2/4 | 2 |
| aging_table | 2 | 1/2 | 2/2 | 1/2 | 1 |
| all_caps_parties | 2 | 0/2 | 1/2 | 2/2 | 1 |
| attachment_with_own_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| attachments | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments_with_own_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| bcc_distribution_list | 1 | 0/1 | 0/1 | 0/1 | 0 |
| billhead | 2 | 0/2 | 0/2 | 0/2 | 0 |
| blank_date | 2 | 0/2 | 0/2 | 0/2 | 1 |
| body_recounts_events | 7 | 2/6 | 5/6 | 7/7 | 2 |
| born_digital | 14 | 2/13 | 10/13 | 8/14 | 7 |
| brand_vs_legal_name | 4 | 2/4 | 3/4 | 2/4 | 2 |
| caption_court_differs_from_header | 1 | 0/1 | 0/1 | 1/1 | 1 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_above_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| circular | 1 | 0/1 | 1/1 | 1/1 | 0 |
| coloured_paper | 2 | 0/2 | 0/2 | 0/2 | 1 |
| comparative_period | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 25 | 8/25 | 18/25 | 15/25 | 7 |
| copies_list | 1 | 0/1 | 1/1 | 0/1 | 0 |
| corrected_firm_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| corrected_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| court_filing | 3 | 1/3 | 2/3 | 3/3 | 1 |
| cover_sheet | 2 | 0/2 | 1/2 | 0/2 | 0 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_first | 3 | 0/3 | 0/3 | 0/3 | 1 |
| date_at_end | 5 | 2/5 | 4/5 | 3/5 | 1 |
| date_in_caption | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_missing_from_text_layer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_only_in_certificate | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_date | 4 | 0/4 | 3/4 | 3/4 | 2 |
| deadline_in_body | 5 | 0/4 | 2/4 | 3/5 | 4 |
| dear_colleague | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dear_sir_madam | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decision_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| decision_numbers_look_like_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| direction | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docket_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ecf_header_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enclosures_not_included | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enforcement_notice | 1 | 0/1 | 0/1 | 0/1 | 1 |
| engraved_vignette | 1 | 0/1 | 0/1 | 0/1 | 0 |
| file_number | 1 | 0/1 | 1/1 | 0/1 | 0 |
| filing_stamps | 1 | 0/1 | 0/1 | 0/1 | 1 |
| flight_dates | 2 | 1/2 | 1/2 | 1/2 | 1 |
| foi_response | 2 | 1/2 | 2/2 | 2/2 | 0 |
| footnote_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| form | 2 | 0/2 | 0/2 | 0/2 | 2 |
| government_issuer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| handwritten | 3 | 0/3 | 0/3 | 0/3 | 0 |
| handwritten_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| header_fields | 1 | 0/1 | 1/1 | 1/1 | 0 |
| hearing_date_in_body | 2 | 0/2 | 1/2 | 2/2 | 1 |
| historic | 4 | 0/4 | 0/4 | 0/4 | 1 |
| historical | 3 | 1/3 | 3/3 | 1/3 | 0 |
| image | 3 | 1/3 | 3/3 | 1/3 | 0 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 1 |
| image_only_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| improvement_notice | 1 | 0/1 | 0/1 | 0/1 | 1 |
| inside_address_at_end | 1 | 0/1 | 1/1 | 0/1 | 0 |
| invoice | 5 | 1/5 | 1/5 | 1/5 | 2 |
| iso_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 3 | 0/3 | 0/3 | 0/3 | 0 |
| junk_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| legacy_doc | 2 | 0/2 | 2/2 | 2/2 | 0 |
| letter | 17 | 5/17 | 15/17 | 8/17 | 4 |
| letter_format | 2 | 0/2 | 1/2 | 1/2 | 1 |
| letterhead | 2 | 0/2 | 2/2 | 0/2 | 2 |
| letterhead_logo | 1 | 0/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 4 | 2/4 | 3/4 | 2/4 | 2 |
| live_date_field | 1 | 0/1 | 1/1 | 1/1 | 0 |
| local_government | 2 | 0/2 | 0/2 | 1/2 | 2 |
| lockbox | 2 | 1/2 | 1/2 | 1/2 | 1 |
| logo_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| long_document | 2 | 1/2 | 2/2 | 2/2 | 0 |
| many_dates | 3 | 2/3 | 3/3 | 2/3 | 1 |
| metadata_date_trap | 4 | 2/4 | 4/4 | 4/4 | 0 |
| metadata_title_trap | 3 | 0/3 | 2/3 | 2/3 | 2 |
| microfilm_targets | 1 | 0/1 | 0/1 | 0/1 | 0 |
| mislabelled_source | 1 | 1/1 | 1/1 | 1/1 | 0 |
| misleading_metadata | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 4 | 2/4 | 3/4 | 2/4 | 2 |
| multi_page | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multi_recipient | 1 | 0/1 | 0/1 | 1/1 | 0 |
| multiple_parties_side | 2 | 0/2 | 1/2 | 2/2 | 1 |
| multiple_recipients | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 9 | 2/9 | 7/9 | 4/9 | 2 |
| network_logo | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_explicit_title | 2 | 0/2 | 0/2 | 0/2 | 1 |
| no_issuer_in_text | 2 | 0/2 | 1/2 | 0/2 | 0 |
| no_letterhead_on_letter | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_title_word | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 18 | 2/17 | 11/17 | 14/18 | 7 |
| notice_of_appeal | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice_of_decision | 2 | 0/2 | 1/2 | 2/2 | 0 |
| notice_of_determination | 1 | 0/1 | 0/1 | 0/1 | 1 |
| notice_of_hearing | 2 | 0/2 | 1/2 | 2/2 | 1 |
| notice_of_removal | 2 | 1/2 | 1/2 | 2/2 | 1 |
| notice_of_violation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_layer | 2 | 0/2 | 0/2 | 1/2 | 2 |
| ocr_text_layer | 2 | 1/2 | 1/2 | 2/2 | 0 |
| on_behalf_of_client | 3 | 1/3 | 2/3 | 2/3 | 0 |
| organization_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| pgp_signed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| plain_text | 2 | 1/2 | 2/2 | 2/2 | 0 |
| posted_notice | 2 | 1/2 | 2/2 | 1/2 | 0 |
| print_field_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| printed_form | 2 | 0/2 | 0/2 | 0/2 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_notice | 4 | 1/3 | 2/3 | 3/4 | 2 |
| publication_number_year_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line_references_document | 3 | 1/3 | 2/3 | 1/3 | 0 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| receipted | 3 | 0/3 | 0/3 | 0/3 | 0 |
| redacted_addressee | 2 | 1/2 | 2/2 | 2/2 | 0 |
| redactions | 1 | 0/1 | 1/1 | 0/1 | 0 |
| reference_numbers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| reference_numbers_look_like_dates | 1 | 0/1 | 0/1 | 1/1 | 0 |
| relative_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| remit_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| remit_to | 5 | 2/5 | 3/5 | 2/5 | 3 |
| remit_to_bank | 1 | 0/1 | 0/1 | 0/1 | 1 |
| report_parameters | 1 | 0/1 | 1/1 | 0/1 | 1 |
| review | 2 | 0/2 | 1/2 | 0/2 | 0 |
| scanned | 12 | 3/12 | 6/12 | 6/12 | 4 |
| sender_block_above_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 1/2 | 1/2 | 1/2 | 0 |
| signatory_trap | 3 | 1/3 | 3/3 | 1/3 | 2 |
| signatory_vs_org | 1 | 0/1 | 0/1 | 0/1 | 0 |
| signature_on_later_page | 3 | 1/3 | 3/3 | 2/3 | 1 |
| signature_only_no_typed_name | 2 | 0/2 | 2/2 | 1/2 | 0 |
| small_print_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 0/1 | 0/1 | 0 |
| staff_names | 1 | 0/1 | 0/1 | 0/1 | 1 |
| stamp_date_only | 2 | 0/2 | 0/2 | 2/2 | 1 |
| stamp_dates | 2 | 0/2 | 1/2 | 0/2 | 0 |
| statement | 4 | 1/4 | 3/4 | 1/4 | 1 |
| subject_is_product | 1 | 0/1 | 0/1 | 1/1 | 0 |
| tables | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tiff | 1 | 0/1 | 1/1 | 0/1 | 0 |
| trading_name | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tribunal | 2 | 0/2 | 1/2 | 2/2 | 1 |
| trustee_party | 1 | 0/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 3 | 1/3 | 1/3 | 1/3 | 2 |
| two_letterhead_seals | 1 | 0/1 | 1/1 | 0/1 | 1 |
| two_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| typist_footer_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| typo_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| uk | 5 | 1/5 | 3/5 | 3/5 | 1 |
| undated | 1 | – | – | 1/1 | 1 |
| us_state | 5 | 0/4 | 2/4 | 3/5 | 4 |
| utility_commission | 1 | 0/1 | 1/1 | 0/1 | 1 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wide_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| windows-1252 | 1 | 0/1 | 1/1 | 0/1 | 0 |
| windows_1252_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| yearless_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| zero_amount | 2 | 1/2 | 2/2 | 1/2 | 1 |

## Routes

- **Pages:** fast 64 · layout 12 · ocr 38
- **Documents by route class:** fast 23 · layout 5 · ocr 17

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 45 | 23.09 s | 45.20 s | 51.88 s |
| `extraction_wall_ms` | 45 | 19.7 ms | 7.38 s | 21.32 s |
| `worker_total_ms` | 45 | 18.9 ms | 7.38 s | 21.32 s |
| `worker_snapshot_ms` | 45 | 0.27 ms | 1.3 ms | 3.0 ms |
| `worker_parse_ms` | 45 | 3.8 ms | 29.5 ms | 215.4 ms |
| `worker_analysis_ms` | 45 | 0.26 ms | 1.9 ms | 6.4 ms |
| `worker_render_ms` | 45 | 0.00 ms | 730.2 ms | 3.08 s |
| `worker_image_decode_ms` | 45 | 0.00 ms | 72.9 ms | 1.38 s |
| `worker_ocr_ms` | 45 | 0.00 ms | 12.35 s | 40.90 s |
| `worker_ocr_encode_ms` | 45 | 0.00 ms | 965.0 ms | 2.92 s |
| `worker_ocr_engine_ms` | 45 | 0.00 ms | 11.02 s | 37.64 s |
| `worker_vision_ms` | 45 | 0.00 ms | 12.9 ms | 34.3 ms |
| `analyze_wall_ms` | 45 | 21.40 s | 42.75 s | 45.55 s |
| `distill_ms` | 45 | 2.5 ms | 15.1 ms | 31.2 ms |
| `prompt_ms` | 45 | 0.04 ms | 0.06 ms | 0.08 ms |
| `inference_ms` | 45 | 21.39 s | 42.73 s | 45.53 s |
| `prefill_ms` | 45 | 17.24 s | 38.99 s | 40.96 s |
| `generation_ms` | 45 | 4.16 s | 5.91 s | 6.29 s |
| `validation_ms` | 45 | 7.4 ms | 21.9 ms | 26.6 ms |
| `naming_ms` | 45 | 0.26 ms | 1.7 ms | 3.3 ms |
| `index_ms` | 45 | 2.3 ms | 13.1 ms | 27.4 ms |
| `retrieval_ms` | 45 | 0.04 ms | 1.4 ms | 4.0 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 45 | 57.6 tok/s | 58.8 tok/s | 59.2 tok/s |
| `generation_tok_per_s` | 45 | 21.3 tok/s | 22.1 tok/s | 22.1 tok/s |
| `prompt_tokens` | 45 | 997 | 2283 | 2383 |
| `cached_tokens` | 45 | 469 | 480 | 480 |
| `generated_tokens` | 45 | 88 | 122 | 133 |
| `estimated_prompt_tokens` | 45 | 1727 | 3006 | 3315 |
| `model_requests` | 45 | 1 | 1 | 1 |
| `redistillations` | 45 | 0 | 0 | 0 |
| `source_characters` | 45 | 2854 | 22421 | 45033 |
| `digest_characters` | 45 | 3129 | 7884 | 8158 |
| `prompt_characters` | 45 | 5178 | 9933 | 10207 |
| `worker_ocr_pages` | 45 | 0 | 4 | 8 |
| `worker_ocr_passes` | 45 | 0 | 4 | 9 |
| `worker_orientation_passes` | 45 | 0 | 4 | 8 |
| `worker_rendered_pixels` | 45 | 0 | 33660000 | 67578072 |
| `index_units` | 45 | 28 | 137 | 464 |
| `context_units` | 45 | 28 | 74 | 112 |
| `context_tokens` | 45 | 1139 | 2419 | 2727 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 23 | 4.5 ms | 27.7 ms | 4.0 ms | 25.5 ms | 25.1 ms | 1.0 ms | 0.00 ms | 0.00 ms |
| layout | 5 | 8.3 ms | 53.2 ms | 7.8 ms | 51.3 ms | 43.1 ms | 6.4 ms | 0.00 ms | 0.00 ms |
| ocr | 17 | 2.44 s | 21.32 s | 2.44 s | 21.32 s | 29.5 ms | 1.9 ms | 3.08 s | 40.90 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 8.3 ms | 5.59 s | 7.8 ms | 5.59 s | 29.5 ms | 2.3 ms | 188.4 ms | 4.15 s |
| 2-4 | 19 | 8.3 ms | 8.47 s | 7.9 ms | 8.47 s | 215.4 ms | 1.9 ms | 730.2 ms | 15.47 s |
| 5-9 | 5 | 24.3 ms | 21.32 s | 23.4 ms | 21.32 s | 20.5 ms | 1.9 ms | 3.08 s | 40.90 s |
| 10-24 | 2 | 27.7 ms | 53.2 ms | 25.5 ms | 51.3 ms | 43.1 ms | 6.4 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 5 | 489.9 ms | 2.44 s | 489.5 ms | 2.44 s | 18.1 ms | 2.3 ms | 188.4 ms | 2.41 s |
| letter | 17 | 24.3 ms | 7.38 s | 23.4 ms | 7.38 s | 215.4 ms | 1.9 ms | 1.69 s | 12.35 s |
| notice | 18 | 8.3 ms | 21.32 s | 7.9 ms | 21.32 s | 43.1 ms | 6.4 ms | 3.08 s | 40.90 s |
| receipt | 1 | 1.77 s | 1.77 s | 1.77 s | 1.77 s | 8.3 ms | 0.06 ms | 0.00 ms | 1.74 s |
| statement | 4 | 2.6 ms | 2.34 s | 1.8 ms | 2.34 s | 4.7 ms | 1.4 ms | 144.2 ms | 2.18 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 230 MB | 1312 MB | real3-notice-02 |
| Model server | 4428 MB | 4484 MB | real3-office-15 |
| Benchmark (engine) | 15 MB | 21 MB | real3-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (35)

- **real3-letter-02**: expected `2011-02-18 Reminder Letter to Panther Motors, Inc.pdf`, got `2011-02-18 Letter to Dominick Livoti.pdf` · trap: party Dominick Livoti (addressee, President of Panther Motors) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-03**: expected `2023-03-28 Decision Letter to Good Energy.pdf`, got `2023-03-28 Letter for Good Electricity Gas Limited.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real3-letter-04**: expected `2010-07-23 Approval Letter to DG ENER.pdf`, got `2010-07-23 Letter from ACER.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-05**: expected `2023-06-16 Letter to Target Corporation.pdf`, got `2023-06-16 Letter to Andrea Joy Campbell.pdf` · trap: party Andrea Joy Campbell (Massachusetts Attorney General who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real3-letter-06**: expected `1976-11-18 Letter to Los Alamos Scientific Laboratory.pdf`, got `1976-11-18 Letter from RR King.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-07**: expected `1966-01-04 Letter to U.S. Atomic Energy Commission.pdf`, got `Document.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID
- **real3-letter-08**: expected `1995-12-07 Decision Letter to Bloom & Kreten.pdf`, got `1995-12-07 Letter to Mr. Kreten.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-letter-09**: expected `1992-12-24 Objection Letter to Pilcher and Pilcher.pdf`, got `Letter to Pilcher and Pilcher.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-11**: expected `1907-12-26 Acknowledgment Letter to Wright Brothers.tif`, got `1907-12-26 Letter from WASHINGTON.tif` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real3-notice-01**: expected `2024-02-19 Improvement Notice for The Chiltern Railway Company Limited.pdf`, got `Improvement Notice for John Giles Turner.pdf` · trap: party John Giles Turner (ORR inspector who signs and serves the notice) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-02**: expected `2007-04-24 Notice of Public Site Visit and Informational Hearing for City of Victorville.pdf`, got `2007-04-24 Application for Certification for the Victorville for City of Victorville.pdf` · review: DATE_UNSUPPORTED
- **real3-notice-04**: expected `2006-02-23 Notice of Determination from City of Industry.pdf`, got `Notice of Determination for Mike Kissell.pdf` · trap: party Mike Kissell (Planning Director, contact person and signatory) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-05**: expected `2008-09-22 Notice of Hearing from Mojave Desert Air Quality Management District.pdf`, got `2008-10-27 Notice of Hearing from Governing Board of the Mojave Desert Air Quality Management District.pdf` · trap: date 2008-10-27 (date of the public hearing) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-06**: expected `2025-06-16 Notice of Decision to Impose a Financial Penalty for Farringdon Energy Limited.pdf`, got `2025-06-16 Notice of decision to impose a financial penalty pursuant to for Farringdon Energy Limited.pdf` · review: PARTY_UNSUPPORTED
- **real3-notice-07**: expected `2023-04-07 Notice of Violation for PacifiCorp.pdf`, got `2023-04-07 Notice of Violation for Shelley McCoy.pdf` · trap: party Shelley McCoy (PacifiCorp's Director, Regulation, to whom the letter is addressed) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-08**: expected `2018-01-26 Notice of Removal between Olymbec USA LLC and CWFS Insight LLC.pdf`, got `2018-01-22 Notice of Removal between OLYMBEC USA LLC and CWFS Insight LLC.pdf` · trap: date 2018-01-22 (date of the Second Verified Complaint and its receipt) · review: LOW_CONFIDENCE
- **real3-notice-09**: expected `2017-12-15 Notice of Appeal between NCL (Bahamas) Ltd and O.W. Bunker USA Inc.pdf`, got `2017-12-15 Notice of Appeal for O.W. BUNKER USA INC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-11**: expected `2023-11-13 Revocation of Product Approval for Atlantis Hydrotec.pdf`, got `Letter to Water Company day to day Contacts.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-13**: expected `1831-11-01 Notice from Parish of Clerkenwell.jpg`, got `1831-11-01 Document.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID
- **real3-notice-15**: expected `2026-10-14 Errata Notice from The FreeBSD Project.txt`, got `2026-10-14 Errata Notice from FreeBSD Project.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-02**: expected `2026-09-30 Invoice from KSAZ.pdf`, got `Invoice from Waterfront Strategies.pdf` · trap: party Waterfront Strategies (agency in the billing address, not the issuer) · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **real3-invoice-04**: expected `2019-06-19 Statement of Account from Riviera Broadcasting.pdf`, got `2019-06-19 Statement of Account from Katz Media Group.pdf` · trap: party Katz Media Group (agency (rep firm) addressed on the statement) · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-09**: expected `1916-11-04 Invoice from Mergenthaler Linotype Co.pdf`, got `Invoice from Panama City PUB Co.pdf` · trap: party Panama City Pub Co (customer, typed above the issuer) · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-11**: expected `1850-07-11 Bill from Henry D. Gray & Son.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-12**: expected `1848-11-14 Bill from Veranda Hotel.jpg`, got `Document from VERANDAHOTEL.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-15**: expected `1848-07-01 Receipt from Post-Office, Duxbury, Mass.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-office-02**: expected `2018-11-21 Remit Letter to NHS Pay Review Body.docx`, got `2018-11-21 Letter to Office of Manpower Economics.docx` · trap: party Office of Manpower Economics (secretariat whose address is used) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-03**: expected `2016-03-23 Letter of Concern to Central Bedfordshire Council.docx`, got `2016-03-23 Letter to Andrew Cook, HMI.docx` · trap: party Andrew Cook (HMI Regional Director who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED
- **real3-office-04**: expected `2017-08-24 Freedom of Information Response from Office of the Advocate General for Scotland.docx`, got `2017-08-24 Letter to Office of the Advocate General for Scotland.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-05**: expected `2011-10-01 Direction for Ofcom.doc`, got `2011-10-01 Direction to Ofcom to Impose a Notification Condition.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-06**: expected `2014-02-20 Circular from Home Office.doc`, got `2014-02-20 report of the Winsor Review from Home Secretary.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **real3-office-09**: expected `2018-07-25 Notice of Change of Company Name for Aston Car Group Limited.rtf`, got `2018-07-25 Notice of Change of Company Name for Company Names Tribunal.rtf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-12**: expected `2012-09-30 Statement of Financial Position from Province of British Columbia.xlsx`, got `Statement of Financial Position.xlsx` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real3-office-14**: expected `2017-03-31 Condensed Statement of Financial Position from Administrative Tribunals Support Service of Canada.csv`, got `2017-03-31 Condensed Statement of Financial Position unaudited from Departmental net debt.csv` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-15**: expected `2018-07-26 Notice of Change of Company Name for PORSCHE LTD.rtf`, got `2018-05-24 Notice of Change of Company Name for Porsche Ltd.rtf` · trap: date 2018-05-24 (date of the adjudicator's decision) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.

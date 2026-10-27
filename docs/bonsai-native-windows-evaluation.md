# Bonsai 2 Small: native Windows qualification

> **Superseded by the 77-document evaluation.** On the full InternBench corpus
> ([`bench/reports/bonsai-linux-cpu-2026-10-27`](../bench/reports/bonsai-linux-cpu-2026-10-27/README.md))
> neither Bonsai model reached Qwen3.5 2B: 37/77 correct names for Bonsai 2 2B,
> 48 for Bonsai 2 4B at 1.6 times Qwen's time, 52-54 for Qwen. The 12-document
> cohort below, where 4B led, was too small to show it. Intern now keeps Qwen3.5
> 2B as its accurate model and offers Bonsai 2 2B as an optional quick model whose
> readings always go to review; Bonsai 2 4B is not shipped. What follows is
> kept as the record of the earlier qualification.

Both Bonsai variants run through Intern's current extraction and evidence pipeline on the real Windows guest. Bonsai 2B reduces latency and memory; Bonsai 4B improves strict filename accuracy on this frozen cohort. The results do **not qualify automatic 2B-to-4B routing or a production-default replacement**: accepted summaries still contain false relationships, one 4B filename loses the document issuer, and reliable routing criteria have not been validated.

The implementation adds opt-in [native qualification and request capture](native-model-qualification.md). It does not change the app's model manifest, downloads, defaults or production prompts. The current working backend remains available. No automatic dual-model download, concurrent warm models, merge or release is part of this change.

## Verified models and runtime

The requested [Bonsai 2 Small collection](https://huggingface.co/collections/prism-ml/bonsai-2-small) contains these tested candidates:

| Candidate | Exact repository / quantization | Pinned revision | Model bytes |
| --- | --- | --- | ---: |
| Bonsai 2B | [prism-ml/Ternary-Bonsai-2-2B-gguf](https://huggingface.co/prism-ml/Ternary-Bonsai-2-2B-gguf), PQ2_0 | `683f7be2629342a50987627b40881e54761a2430` | 517,312,992 |
| Bonsai 4B | [prism-ml/Ternary-Bonsai-2-4B-gguf](https://huggingface.co/prism-ml/Ternary-Bonsai-2-4B-gguf), PQ2_0 | `8d6557168168f91ea6697a4a49ed2ef622b53adb` | 1,146,465,120 |
| Current model family | [unsloth/Qwen3.5-2B-GGUF](https://huggingface.co/unsloth/Qwen3.5-2B-GGUF), Q4_K_M | pinned full GGUF SHA-256 in the plans | 1,280,835,840 |

Bonsai cards declare Apache-2.0 and a Qwen3.5-derived architecture with a 262,144-token architectural context. The tested context is **8,192**, not that maximum. This experiment uses text input. Vision requires an additional matching projector and application integration; no projector was downloaded or tested. Changing the language model does not replace PDF extraction or OCR.

PQ2_0/PTQ1_0 require Prism kernels. The tested Windows x64 CPU runtime is Prism `prism-b10777-e3de715`, commit `e3de715222b2667c54a267441159a5729f11155e`; the Qwen runtime is upstream llama.cpp b10361. The [official runtime guide](https://docs.prismml.com/run/llamacpp), [small-model guide](https://github.com/PrismML-Eng/Bonsai-demo/blob/main/SMALL-MODELS.md) and [backend audit](https://github.com/PrismML-Eng/Bonsai-demo/blob/main/BACKEND-SUPPORT.md) distinguish quantization/backend requirements. Upstream Q2_0 with Hadamard transforms requires a sufficiently recent runtime (the guide specifies b11555+); that variant was not tested. Published CUDA/Vulkan/HIP/SYCL packages do not establish compatibility or speed on this guest. MLX is for Apple Silicon.

## Actual Windows protocol

Host: zgbrenner-pc, i5-13400F, approximately 30 GiB RAM and an RTX 3050 OEM 8 GiB. Guest: Windows 11 Enterprise LTSC Evaluation build 26100, **4 GiB RAM, two vCPUs, Microsoft Basic Display Adapter**. There is no GPU passthrough. These are real Windows CPU results; Linux cross-compilation was preparation. GPU acceleration and VRAM use remain unmeasured.

All documents are synthetic and fictional. No private legal documents or external model service were used. The unchanged worker reads native text, runs local OCR where required, and retrieves evidence from the whole document before the text model proposes facts. The engine validates and composes filenames, descriptions and review decisions.

The baseline source is `a4252761d0c4e0cbc035330990824f6ef2238b38`. Twelve IDs and 4/2/1/5 native process partitions were frozen before these heldout answers. Original/few-shot and greedy/vendor profiles were calibrated on three separate documents; Qwen and 4B each retain all 18 calibration replies. The predeclared ranking prefers complete title/date/name rows, then greedy and original on ties. It selects few-shot greedy for both. Qwen vendor calibration used the [official text non-thinking settings](https://huggingface.co/Qwen/Qwen3.5-2B) (temperature 1.0, top-p 1.0, presence penalty 2.0); Bonsai used 0.7, 0.8 and 1.5. These were fixed before observing the corresponding responses. Bonsai 2B's corresponding profile was already frozen. All three use identical effective request-body JSON values, seed 17, temperature 0, top-k1, disabled thinking and a 1,024-token reply cap. Roles/citations are independently audited because this selection score does not validate their semantics.

One fresh model per native batch; one unscored warmup per model process; a fresh benchmark/worker for every scored document. Same fixture bytes, extracted sources, source prompt hashes, grammar, context, thread count, slot and 64-MiB cache budget across models. Model/runtime is the comparison variable, so this does not isolate quantization's causal effect. No heldout prompt, gold, validator or sampler tuning followed the results.

Each guest has networking disabled and read-only input media, with Defender confirmed active. The native runner verifies plan/assets/model hashes, owns children through Windows jobs, checks its localhost listener PID, monitors Defender events and stops on security-query errors or new detections. Host controls retain a six-GiB VM cap, zero host cgroup swap, two CPUs, reserve thresholds and a 480-second watchdog. All 36 matched documents completed and every stopped-disk capture/security validation passed, with no new Defender event 1116. No owned guest/model job remains running.

## Matched results

| Frozen metric, 12 documents each | Bonsai 2B | Qwen 2B | Bonsai 4B |
| --- | ---: | ---: | ---: |
| Strict filename matches | 8 | 9 | 11 |
| Review required | 3 | 2 | 1 |
| Incorrect strict filename marked ready | 1 | 2 | 1 |
| Median document processing, seconds | 22.49 | 26.22 | 54.13 |
| Nearest-rank p95, seconds | 143.28 | 147.46 | 170.02 |
| Sum document processing, seconds | 433.64 | 482.77 | 820.28 |
| Sum inference, seconds | 235.52 | 286.12 | 625.20 |
| Sum extraction, seconds | 197.72 | 196.20 | 194.62 |
| Maximum model peak working set, decimal GB | 0.941 | 2.207 | 1.957 |

Bonsai 2B was fastest on all 12; 4B was slowest on all 12. 4B's filename gains over 2B are the remittance type, meeting issuer and amendment's last-signature date. Its median is 2.41 times 2B's and total processing 1.89 times. This is a useful accuracy candidate for further work, not evidence that all smaller-model errors can be safely escalated.

The unsafe-ready counts hide different severities. Bonsai 2B omits a meeting issuer. Qwen files an annual report for a board member and accepts an amendment's first signature instead of the required last signature. 4B uses a generic shareholders identity instead of the annual report's issuer. The addressed shareholders are present in the source; this is identity loss, not an invented person.

Correct filenames also hide description/role errors. Qwen and 4B describe a capital call as going to its issuing general partner. 4B reverses supplier/distributor into buyer/seller in raw facts; the final neutral relation hides it. Its meeting description invents a recipient relation and is correctly flagged for review. Shared engine composition drops “feet” from “square feet” and overstates effective-date roles despite conditions. Qwen additionally copied a party from the neutral example into a statement; validation blocked that invented name. Read the independent audits and raw data, rather than treating lexical `description_factual` or party-membership scores as semantic clearance.

The 25-page scan took 143.28 / 147.46 / 170.02 seconds for 2B / Qwen / 4B, with 126.10 / 125.55 / 123.24 seconds in extraction. Model replacement therefore leaves the main latency cost intact. All three produce the same truncated area unit in the description.

## Evidence, reproduction and limits

[Machine-readable comparison, profiles, frozen plans, source/raw-response JSONL and independent audits](../bench/reports/bonsai-native-windows-2026-10-18/) accompany this report. `sha256.json` binds the exported files. Frozen plan manifests include the exact executable, runtime, model and fixture hashes; binaries/weights/VM disks are not committed. Build the opt-in runner and stage reviewed local assets as described in [native qualification](native-model-qualification.md). A different build must have its assets independently reviewed and hashes regenerated; do not remove verification to make an old plan run.

The affected Rust crates passed 541 tests with zero failures and three ignored tests; Windows GNU release binaries were built and executed in the guest. Eight collector/provenance regression checks passed. The 168 reviewed Rust/Cargo source files were rehashed unchanged after reconnection. Hosted CI then identified an ambiguous Cargo default executable after adding the runner; a separately recorded one-line manifest correction restores `cargo run -p intern-bench`. Its exact 77-case recorded baseline replay passes with zero gated regressions. No measured Rust behavior or Windows/model settings changed, so the original Windows measurements and tested-build hashes remain preserved. Captures were collected read-only after shutdown, bound to complete native reports and full Defender logs; independent review reverified every export hash and source/request parity.

Single pass, fixed order and 12 observations per model: p95 is the maximum, not a population guarantee. The first four 2B cases explicitly repeat an earlier attempt. Guest splits differ to preserve the watchdog; native process partitions stay fixed. One 431 ms bounded media-copy stop overlapped the 2B 25-page scan. Qwen's first batch and 4B's first batch completed before watchdog ACPI shutdown; host timing supports termination before forced quit, but Windows Event 6006 was not inspected. Cleanup then found already-collected VM units; its bookkeeping failure is retained in the journal. Successfully validated disposable leaves were retired after independent review; all benchmark/security exports and original/backing/incident evidence remain.

RAM-backed media, OS caches and unscored warmups mean these are **warm document-processing measurements**, not cold SSD/install times. Fresh server startup is recorded separately per process in the comparison JSON. Model peak is Windows OS working set, not total system commit or VRAM. Worker OS peak is unavailable; host pre-shutdown observations do not establish full-lifetime peaks. Zero host cgroup swap does not imply zero Windows pagefile use. An unrelated Speek model server was idle during a 3-second pre-run sample and was left untouched; desktop background activity was not fully controlled. Earlier 18/24-case qualifications are separate historical cohorts and must not be used as before/after gains.

## How the retest changes the earlier recommendation

The earlier unmodified-prompt screen rejected Bonsai 2B after repeated literal instruction copying and excessive review. This retest establishes that the frozen neutral example fixes that specific failure sufficiently for controlled evaluation, while preserving the memory/speed benefit. The new 4B profile gives the best filename result in the matched set and correctly resolves the last-signature date that Qwen misses. Both are viable experimental backends, and 4B now deserves priority for further human-reviewed acceptance work.

The recommendation remains to keep production unchanged. Better filenames do not resolve false recipient relationships, generic document identity, conditional-date overstatement or unit loss; a reliable escalation policy has not been demonstrated. Earlier 18/24-case screens and this new 12-case cohort differ, so their percentages are not before/after improvements. These conclusions concern the new matched evidence rather than a retrospective repair of earlier scores.

## Practical default and next gate

Keep the current single warm backend and review flow. Keep Bonsai 4B as a reproducible experimental candidate. Do not silently download both models or keep both resident: this 4-GiB guest has not demonstrated a safe dual-model memory budget. Review flags and page count alone cannot route reliably because some observed wrong outputs were marked ready.

Prioritize source-grounded party roles and review when role certainty is unresolved; preserve units, ranges and conditional date language during composition; then measure OCR improvements on a new, predeclared fixture set. These repairs address observed errors across models and the dominant scanned-document cost. Retest a router on untouched documents before promotion. Product model selection would additionally need explicit per-model download consent, pinned runtime/weight provenance, offline operation, resource-aware unload-before-load, cancellation and incumbent fallback. Those product behaviors are not established by this benchmark and are not enabled here.

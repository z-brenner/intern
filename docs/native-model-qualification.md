# Native model qualification on Windows

`benchmark-transport` is an opt-in laboratory feature. The desktop app does not enable it, and its default model, download flow and request behavior stay unchanged.

Build the benchmark and qualification runner with the feature enabled:

```sh
cargo build --locked --release -p intern-bench --features benchmark-transport --bins
```

For a live benchmark against a local server, add both `--bench-profile PROFILE.json` and `--wire-output NEW_DIRECTORY`. Replay and extract-only modes reject these options. The profile permits bounded sampler settings and an optional system/example/prefix/suffix prompt. It cannot change the endpoint, model or grammar, and preserves the original document evidence text. Prompt wrappers can influence the interpretation of that evidence. See `crates/intern-engine/tests/fixtures/benchmark-profile.json` for the prompt format. Captures retain the original engine request, transformed request, exact transmitted bytes, raw HTTP response and hashes. Authorization headers are excluded. Captures contain document text: use synthetic fixtures or protect them as carefully as the source documents.

## Native qualification runner

```text
intern-qualify PLAN.json PLAN_SHA256 MODEL.gguf
```

The plan's root is the directory containing the plan. Its schema is defined in `crates/intern-bench/src/bin/intern-qualify.rs`: schema version, unique run ID, relative asset paths and SHA-256 hashes, server/benchmark/worker paths, corpus/gold/fixture manifest paths, model SHA-256, and ordered case IDs with profile paths. Optional `body` cases send frozen raw calibration requests. Plan and every asset/model hash are verified before execution. Relative asset paths reject parent traversal, absolute paths and symlinks; the external model path and plan parent directory are canonicalized. This is intended for previously reviewed, immutable local media, not arbitrary untrusted plans.

On Windows the runner starts one owned model process with two threads, one slot, an 8192-token context, bounded cache and zero GPU layers. Children start suspended, enter owned Windows jobs, then resume; job close terminates their descendants. A random in-memory API key protects the localhost listener, and its owning PID is checked before readiness and cases. HTTP ignores proxies and redirects and has a 150-second deadline. Native cases have a 240-second cap, including the first unscored warmup; the batch checks a 360-second deadline between cases. Each subsequent case uses a fresh benchmark/worker and the same model process without another warmup.

The runner checks Defender event1116 before, during and after the batch. An alert or failed security query stops execution. It never changes Defender settings, downloads/loads scripts or provides an export listener. A NIC-free guest and read-only input media are external lab requirements; the runner does not configure the hypervisor. It writes a new `InternNative-RUN_ID` directory in the Windows temporary directory and refuses to overwrite it.

Completion requires the expected native report with exactly one completed case; exit0 alone is insufficient. A terminal batch record must show every planned case attempted and complete. Raw calibration success means HTTP delivery only: malformed assistant JSON fails the separate structural score, while role, citation and other semantic errors require an independent audit and may pass the frozen value/shape/name score. Preserve partial results on any failure.

Read the stopped guest disk through a read-only collector. Verify asset/plan/model hashes, all capture companions and response hashes, expected case identity/completion, the complete Defender event log against each run's baseline, and extraction errors before accepting any result. Do not infer success from the console alone.

## Interpreting results

Compare the same frozen cases, extraction build, context, prompt budget, hardware and thread count. Select prompts/samplers on separate calibration documents and freeze the choice before heldouts. Retain all seeds/attempts, including interruptions and failures. Do not tune on heldout answers or compare percentages from different case cohorts as gains.

Audit the actual filename and one-sentence description against the source. Existing lexical description and party-membership scores do not establish correct units, conditions, roles or citations. Record wrong ready results separately from conservative review flags and harmless filename omissions. An OCR-bound document remains OCR-bound after a text-model replacement.

The qualification runner records Windows model/benchmark process peak working sets. The existing benchmark's sampled `/proc` memory fields are Linux-only; a zero on Windows is unavailable, not zero use. Worker OS peak is not measured by this runner. Report host VM peaks with their sampling/lifetime limits separately. RAM-backed model media and warmups do not measure cold installation or SSD-start latency.

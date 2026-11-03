# CPU model replacement: research and qualification tooling

2026-11-03, base `7996324`. **Blocked before live inference; no replacement qualified.**

[Research, licences, integration and optimization plan](../../../docs/cpu-model-research-2026-11.md).
The proposed first arm is Qwen3.5-4B Q4_K_M (2,740,937,888 bytes), against the exact shipping
Qwen3.5-2B Q4_K_M (1,280,835,840 bytes). Sizes and hashes were obtained from the pinned public
Hub metadata; the model bytes were not downloaded or validated locally.

## Completed checks

| Check | Result |
|---|---|
| Frontend typecheck, tests, production build (`npm run check`) | Passed; 754 tests / 77 files |
| Chromium browser flows | 30 passed; system Chromium through the existing local Playwright config |
| Rust tests: engine, benchmark, worker, core, intake, queue | 1,478 passed, 0 failed, 6 ignored; release profile, locked dependencies, two test threads |
| Native-feature worker and benchmark-transport build | Passed; compilation only, not a live parser/model run |
| Evidence-pipeline fixture replay | 0 regressions, 0 stale/pending fixtures; 19 completed and 2 expected errors |
| Digest-pipeline fixture replay | 0 regressions, 0 stale/pending fixtures; 19 completed and 2 expected errors |
| InternBench replay | 77 completed; baseline passed with 0 document regressions |
| Qualification tooling | 20 Python tests passed; CI now runs these |
| CI workflow contract checks after the CI edit | 10 tests passed |
| Shell syntax / diff whitespace | Passed |
| Actual model preflight | Refused with exit 2: missing/unverified pinned 2B control; no model server launched |

These are app/tooling checks and **replays of existing inference**, not measurements of a new
model. The replays preserve known errors. For example, the committed InternBench recording
still scores one forbidden party and one wrong name ready. No baseline or gold was rewritten.
No synthetic numbers from qualification-tool unit tests are presented as product results.

## Blocker and limits

The exact baseline download targeted:

`https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/f6d5376be1edb4d416d56da11e5397a961aca8ae/Qwen3.5-2B-Q4_K_M.gguf`

Hugging Face metadata succeeded. The weight redirect points to the official
`us.aws.cdn.hf.co` host, where the environment's network proxy refused the CONNECT tunnel
with **HTTP 403 Forbidden**. An authorized network escalation retried the same baseline once
and received the same failure. This was a proxy refusal, not an automatic approval-review
rejection. No alternate mirror, proxy bypass, or further weight download was attempted.
The initial independent Qwen4B/LFM2.5 download attempts also obtained no model files.

Therefore **new-model accuracy, model peak RAM, cold/warm inference latency, full native-app
inference, packaged Windows qualification and release gates remain unrun**. Source-built
llama.cpp `14e78dd` is available locally, but it is not the app's pinned b11946 runtime and
was not represented as a production control. The environment has a 16 GiB memory limit and
four CPU cores by cgroups. It is not a representative battery-powered laptop.

PR 56 was read at `052937f` and rechecked at `c08f31b`. Its six changed paths do not overlap
this branch. The later revision corrects the readiness labels in its third and first samples.
Its branch, files and reviews were left untouched. No production model/default, memory guard,
download cap, baseline, version, tag or release was changed. No PR was approved or merged.

## Reproduction of completed offline checks

```bash
npm run fixtures
npm run check
npm run test:e2e
cargo test --release --locked -p intern-engine -p intern-bench -p intern-worker \
  -p intern-core -p intern-intake -p intern-queue -- --test-threads=2
python3 -I -m unittest discover -s scripts/cpu-model-eval -p 'test_*.py'
bash -n scripts/run-internbench-candidate.sh
```

For browser reproduction in this environment the command additionally used
`--config /workspace/toolchains/playwright-local.config.ts`, selecting `/usr/bin/chromium`.
The standard browser command above is the repository/CI command.

The three replay commands are the ones in `.github/workflows/ci.yml`, using release-built
`intern-evaluate` and `intern-bench` instead of `cargo run`. Their local JSON reports are
`target/cpu-replay-evidence.json`, `target/cpu-replay-digest.json` and
`target/cpu-replay-internbench.json`; those contain existing recorded public/synthetic data
and were not duplicated in this report. See the research document for the pinned live
comparison command and the conditions required to promote a candidate and release it.

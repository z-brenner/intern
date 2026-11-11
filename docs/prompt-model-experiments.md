# Short prompts and NuExtract3

Use InternBench to compare short evidence prompts and NuExtract3 against the
shipping Qwen3.5 models. The benchmark reads real files through the parser,
uses the existing evidence validator and filename composer, and records each
model's live result. Run the arms in sequence on the same machine with the same
worker, server build, thread count, corpus, and document order.

## Models and runtime

- Use the Qwen3.5 4B and 9B Q4_K_M files pinned in
  [the model manifest](../src-tauri/resources/model-manifest.json).
- Use the [NuExtract3 Q4_K_M GGUF](https://huggingface.co/numind/NuExtract3-GGUF/resolve/28a1aae6288c5d50bdf999ca2e572240966d669b/NuExtract3-Q4_K_M.gguf)
  (2,783,445,984 bytes, SHA-256
  `7ee3c0ee9e5699a4391624ae758487f583f73b3242aa4e73dc2bb33e508d703e`).
  The pinned GGUF embeds NuExtract3's native template. Use text input with no
  vision projector.
- Build with `cargo build --locked --release -p intern-bench --features benchmark-transport`
  and use the native worker and the pinned llama.cpp runtime described in
  [InternBench](internbench.md).

## Compare retrieval without inference

This compares gold evidence recall and estimated context size using the
extraction already in the recordings. It cannot measure live filename
accuracy or latency.

```sh
mkdir -p target/prompt-model-experiments
printf '%s\n' '{}' > target/prompt-model-experiments/whole.json
printf '%s\n' '{"short_document_tokens":750}' > target/prompt-model-experiments/short.json
target/release/intern-bench retrieval \
  --recording bench/recording.json --gold bench/gold.json \
  --fixtures fixtures/corpus-recording.json --expected fixtures/expected.json \
  --config target/prompt-model-experiments/whole.json,target/prompt-model-experiments/short.json \
  --output target/prompt-model-experiments/retrieval.json \
  --markdown target/prompt-model-experiments/retrieval.md
```

`--short-document-tokens 750` considers the smaller field budgets only on
documents that would go whole under the default 1,800-token cutoff. Crowded
evidence, missing core field selections, or a prompt that would fail to get
shorter all retain the whole document. The subject and key facts keep their
normal field budgets. These estimates exclude the fixed instructions.

## Run the four live arms

Set `INTERN_RUNTIME_DIR` and `LLAMA_SERVER` to the staged native runtime and
its `llama-server`. Set `THREADS` explicitly and use the same value for all
arms. The examples assume `QWEN_9B`, `QWEN_4B`, and `NUEXTRACT_3` are paths
to the verified model files. Add the same `--only id,id` to every run for a
smoke test. Use a local, authorized corpus. Recordings and wire captures can
contain document text.

```sh
LAB=target/prompt-model-experiments
export INTERN_BENCH="$PWD/target/release/intern-bench"
export INTERN_BENCH_TOKEN_CONFIDENCE=0

MODEL="$QWEN_9B" scripts/run-internbench.sh "$LAB/qwen9"
MODEL="$QWEN_9B" scripts/run-internbench.sh "$LAB/qwen9-short" -- \
  --short-document-tokens 750
MODEL="$QWEN_4B" scripts/run-internbench.sh "$LAB/qwen4"
MODEL="$NUEXTRACT_3" scripts/run-internbench.sh "$LAB/nuextract3" -- \
  --bench-profile bench/profiles/nuextract3.json --wire-output "$LAB/nuextract3/wire"

"$INTERN_BENCH" compare --before "$LAB/qwen9/report.json" \
  --after "$LAB/qwen9-short/report.json" --markdown "$LAB/short-diff.md"
"$INTERN_BENCH" compare --before "$LAB/qwen9/report.json" \
  --after "$LAB/nuextract3/report.json" --markdown "$LAB/nuextract-vs-9b.md"
"$INTERN_BENCH" compare --before "$LAB/qwen4/report.json" \
  --after "$LAB/nuextract3/report.json" --markdown "$LAB/nuextract-vs-4b.md"
```

On Windows, run the native `llama-server.exe` and `intern-bench.exe` directly
with the same switches as [the live runner](../scripts/run-internbench.sh).
Pass `--server-pid` for memory sampling and `--model-path` for the GGUF hash.
Keep the test corpus and output on the same drive for each arm.

The model comparison uses the same evidence-only readiness rules in all arms.
For a separate prompt A/B at the shipping review threshold, add
`--min-token-confidence 0.7` to both live 9B arms. This also measures the
cost of collecting token probabilities. NuExtract's native replies are
decoded without Qwen's token-confidence gate. A benchmark Ready result
is a scoring label.

Track full filenames, date traps, subject and description quality, Ready
coverage and wrong names marked Ready, model failures, prefill and total
p50/p95, and peak memory. Check the short prompt separately on documents it
actually changed. Require live results from a 16 GB Windows laptop with its
normal apps open before changing the desktop defaults.

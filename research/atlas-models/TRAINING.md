# ATLAS training runs

Status, 2026-11-21: no model has been trained yet. The training script has been tested end to
end on a tiny random Qwen3.5 model (trains, merges, converts to GGUF with llama.cpp's converter
at the app's pin), and its speed and memory were measured on a randomly initialised model with
the real Qwen3.5-0.8B shape. The first real run is `train-runs/2026-11-21-lora1.json`.

## How a run works

Commit `research/atlas-models/train-runs/<name>.json` to an ATLAS research branch.
`.github/workflows/atlas-train.yml` then, on GitHub's free 4-vCPU, 16 GB Linux runners:

1. **dataset**: generates the train and validation splits and verifies every label through the
   engine (DATASET.md). Kept as a workflow artifact.
2. **train**: downloads the base checkpoint at its resolved commit, runs
   `training/train_lora.py`, converts both the merged student and the untouched base with
   `convert_hf_to_gguf.py` at llama.cpp `b11946` (the app's pin, `--no-mtp`), and quantizes
   both with that release's `llama-quantize`. The two GGUFs are a workflow artifact; the
   training log, pip freeze and GGUF hashes are committed.
3. **eval**: student and control, live at the app's 0.7 gate with the app's own worker,
   llama-server and prompt, on each set named in the spec.

The control is the same base converted and quantized by the same steps, so the LoRA is the only
difference between the two. It is not the unsloth Q4_K_M the screen uses (that one is
imatrix-quantized); compare the student with the control, not with the screen's numbers.

## What the script does

`training/train_lora.py`:

- builds each prompt with the model's own chat template, thinking off, as llama-server does,
  and computes the loss on the reply and its end-of-turn token only. Only the reply's positions
  get logits, which removes most of the output layer's cost over a 248k vocabulary (checked to
  match the masked-label loss to 7 digits);
- puts LoRA (rank 16, alpha 32 by default) on every linear layer of the language model,
  attention, Gated DeltaNet projections and MLP, never the vision tower, embeddings, output
  layer or MTP head;
- uses gradient checkpointing (on by default), AdamW, warm-up then cosine, gradient clipping,
  and stops cleanly at `--budget-minutes`, so a runner's job limit never loses the work;
- writes the merged weights over the base's own safetensors files: same file and tensor names,
  dtypes and config, and tensors transformers does not load (the MTP head) carried over
  untouched.

## Measured cost (this container, not a runner and not a laptop)

On 4 CPUs, a randomly initialised model with the exact Qwen3.5-0.8B shape, one ~950-token
example (prompt plus reply) per step:

| Setting | Seconds per example | Peak memory |
|---|---|---|
| Gradient checkpointing (default) | 20 | 5.8 GB |
| No checkpointing | 14 | about 14 GB |

So a 250-minute budget covers about 600-700 examples on a similar CPU. The run caps at 600 so
the learning-rate schedule finishes inside the budget. These are training costs only; a
runner's CPU will differ, and a Windows laptop's inference speed is a separate measurement
(BENCHMARK_RESULTS.md).

## What CPU training can and cannot answer

A 0.8B LoRA on 600 examples answers one question cheaply: does training on verified ATLAS
exchanges move a small model's real-set accuracy at all, measured against an identical
untrained control? It cannot answer whether a 2B-4B student can match Qwen3.5 9B. That needs
the 2B model, several thousand examples and probably teacher-labelled data, which is GPU work:
at the measured rate a 2B LoRA over 5,000 examples would take days on a free runner. That run
waits on GPU time Zack authorizes; the scripts run unchanged on a GPU machine.

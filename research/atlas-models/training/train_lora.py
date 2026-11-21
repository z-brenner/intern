#!/usr/bin/env python3
"""LoRA fine-tune of a Qwen3.5 student on verified ATLAS examples, then a merged checkpoint. Laboratory tooling.

    python3 -I train_lora.py --base DIR --train sft.jsonl --out OUT_DIR \
        [--max-examples N] [--epochs E] [--rank R] [--lr LR] [--budget-minutes M] [--val val.jsonl]

--base is a local Hugging Face snapshot (config.json, tokenizer, safetensors). Every example is
the exact system and user turn the engine sends (captured by research/atlas-models/dataset) and
the verified compact reply. The prompt is built with the model's own chat template, thinking
off, exactly as llama-server builds it, and the loss is on the reply and its end-of-turn token
only. LoRA goes on every linear layer of the language model (attention, Gated DeltaNet
projections, MLP), never the vision tower, embeddings or output layer.

Writes OUT_DIR/adapter (the LoRA), OUT_DIR/merged (the base's own safetensors files with the merged
weights written over the tensors LoRA changed: same names, dtypes, config, ready for llama.cpp's
convert_hf_to_gguf.py) and OUT_DIR/train-log.json (loss per step,
held-out loss, wall time, examples seen). Stops cleanly at --budget-minutes, so a CPU runner's
job limit never loses the work: what was trained is what is merged and reported.
"""
import argparse
import gc
import json
import math
import os
import random
import shutil
import time

import torch


def load_examples(path, limit=None, seed=0):
    with open(path, encoding="utf-8") as file:
        rows = [json.loads(line) for line in file if line.strip()]
    random.Random(seed).shuffle(rows)
    return rows[:limit] if limit else rows


def encode(tokenizer, example, max_tokens):
    messages = example["messages"]
    prompt_messages = [m for m in messages if m["role"] != "assistant"]
    reply = messages[-1]["content"]
    prompt = tokenizer.apply_chat_template(prompt_messages, tokenize=False, add_generation_prompt=True,
                                           enable_thinking=False)
    end = "<|im_end|>"
    prompt_ids = tokenizer(prompt, add_special_tokens=False)["input_ids"]
    reply_ids = tokenizer(reply + end, add_special_tokens=False)["input_ids"]
    ids = prompt_ids + reply_ids
    if len(ids) > max_tokens:
        return None
    # The reply is the tail of the sequence, so only its positions (and the one before it) need logits:
    # the output layer over a 248k vocabulary is a fifth of the compute and most of the memory otherwise.
    return torch.tensor([ids]), len(reply_ids)


def reply_loss(model, ids, reply_length):
    """Mean cross-entropy over the reply tokens and its end-of-turn token, as labels=-100 over the prompt gives."""
    ids = ids.to(DEVICE)
    logits = model(input_ids=ids, logits_to_keep=reply_length + 1).logits
    return torch.nn.functional.cross_entropy(logits[0, :-1].float(), ids[0, -reply_length:])


DEVICE = "cuda" if torch.cuda.is_available() else "cpu"


def load_model(base):
    from transformers import AutoConfig
    config = AutoConfig.from_pretrained(base)
    architectures = config.architectures or []
    # fp32 on CPU (bf16 matmuls are slow on most CPUs); bf16 on a GPU.
    kwargs = dict(dtype=torch.bfloat16 if DEVICE == "cuda" else torch.float32, low_cpu_mem_usage=True)
    if any("ConditionalGeneration" in name for name in architectures):
        from transformers import AutoModelForImageTextToText
        model = AutoModelForImageTextToText.from_pretrained(base, **kwargs)
    else:
        from transformers import AutoModelForCausalLM
        model = AutoModelForCausalLM.from_pretrained(base, **kwargs)
    return model, architectures


def lora_targets(model):
    names = []
    for name, module in model.named_modules():
        if not isinstance(module, torch.nn.Linear):
            continue
        lowered = name.lower()
        if any(part in lowered for part in ("visual", "vision", "lm_head", "embed", "mtp")):
            continue
        names.append(name)
    return names


def write_merged(base, saved_dir, tokenizer, merged_dir):
    """The base checkpoint's own files with the LoRA-merged weights written over the tensors that changed.

    Same file names, tensor names, dtypes and config as the base, so llama.cpp's converter treats the
    student exactly as it treats the base (the control), and tensors transformers does not load (for
    example multi-token-prediction heads) are carried over untouched. Falls back to transformers'
    own save when the names do not line up, and says so.
    """
    import glob
    from safetensors import safe_open
    from safetensors.torch import save_file
    from safetensors.torch import load_file

    scratch = saved_dir
    if True:
        trained = {}
        for path in glob.glob(os.path.join(scratch, "*.safetensors")):
            trained.update(load_file(path))
        base_files = sorted(glob.glob(os.path.join(base, "*.safetensors")))
        base_names = set()
        for path in base_files:
            with safe_open(path, "pt") as handle:
                base_names.update(handle.keys())
        unmatched = sorted(set(trained) - base_names)
        os.makedirs(merged_dir, exist_ok=True)
        report = {"base_files": len(base_files), "replaced": 0, "unchanged": 0, "kept_from_base": 0,
                  "unmatched_trained_tensors": unmatched[:10], "fallback": False}
        if not base_files or len(unmatched) > len(trained) // 10:
            report["fallback"] = True
            for name in os.listdir(scratch):
                shutil.copy(os.path.join(scratch, name), os.path.join(merged_dir, name))
        else:
            for path in base_files:
                tensors = {}
                with safe_open(path, "pt") as handle:
                    metadata = handle.metadata()
                    for key in handle.keys():
                        original = handle.get_tensor(key)
                        if key in trained:
                            updated = trained[key].to(original.dtype)
                            if torch.equal(updated, original):
                                report["unchanged"] += 1
                            else:
                                report["replaced"] += 1
                            tensors[key] = updated.contiguous()
                        else:
                            report["kept_from_base"] += 1
                            tensors[key] = original
                save_file(tensors, os.path.join(merged_dir, os.path.basename(path)), metadata=metadata)
                del tensors
    tokenizer.save_pretrained(merged_dir)
    # The base's own config, index and processor files win, so the converter sees the base's architecture.
    for name in os.listdir(base):
        if name.endswith((".json", ".jinja", ".txt", ".model", ".tiktoken")) and not report["fallback"] \
                or (report["fallback"] and name.endswith((".jinja", ".txt", ".model", ".tiktoken"))) \
                or (report["fallback"] and name.endswith(".json") and "safetensors.index" not in name):
            if os.path.isfile(os.path.join(base, name)):
                shutil.copy(os.path.join(base, name), os.path.join(merged_dir, name))
    return report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--train", required=True)
    parser.add_argument("--val")
    parser.add_argument("--out", required=True)
    parser.add_argument("--max-examples", type=int, default=0)
    parser.add_argument("--epochs", type=float, default=1.0)
    parser.add_argument("--rank", type=int, default=16)
    parser.add_argument("--alpha", type=int, default=32)
    parser.add_argument("--lr", type=float, default=1e-4)
    parser.add_argument("--accumulate", type=int, default=8)
    parser.add_argument("--max-tokens", type=int, default=2600)
    parser.add_argument("--budget-minutes", type=float, default=0)
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument("--no-checkpointing", action="store_true")
    parser.add_argument("--seed", type=int, default=20261121)
    args = parser.parse_args()

    started = time.time()
    torch.manual_seed(args.seed)
    if args.threads:
        torch.set_num_threads(args.threads)
    from transformers import AutoTokenizer
    from peft import LoraConfig, get_peft_model

    tokenizer = AutoTokenizer.from_pretrained(args.base)
    model, architectures = load_model(args.base)
    if not args.no_checkpointing:
        # Recomputes each layer's activations in the backward pass: about 40% slower on CPU, and peak memory
        # for the 0.8B model at ~950 tokens drops from ~14 GB to ~6 GB, inside a 16 GB runner.
        model.gradient_checkpointing_enable(gradient_checkpointing_kwargs={"use_reentrant": False})
        model.enable_input_require_grads()
    model.to(DEVICE)
    targets = lora_targets(model)
    print(f"architectures {architectures}; LoRA on {len(targets)} linear layers, e.g. {targets[:6]}", flush=True)
    config = LoraConfig(r=args.rank, lora_alpha=args.alpha, lora_dropout=0.05, target_modules=targets, bias="none")
    model = get_peft_model(model, config)
    model.print_trainable_parameters()

    rows = load_examples(args.train, args.max_examples or None, args.seed)
    data = []
    skipped = 0
    for row in rows:
        encoded = encode(tokenizer, row, args.max_tokens)
        if encoded is None:
            skipped += 1
        else:
            data.append(encoded)
    val = []
    if args.val:
        for row in load_examples(args.val, 40, args.seed):
            encoded = encode(tokenizer, row, args.max_tokens)
            if encoded is not None:
                val.append(encoded)
    tokens = [ids.shape[1] for ids, _ in data]
    print(f"{len(data)} training examples ({skipped} over {args.max_tokens} tokens), "
          f"mean {sum(tokens) / max(1, len(tokens)):.0f} tokens; {len(val)} held out", flush=True)

    trainable = [p for p in model.parameters() if p.requires_grad]
    optimizer = torch.optim.AdamW(trainable, lr=args.lr, weight_decay=0.0)
    total_steps = max(1, math.ceil(len(data) * args.epochs / args.accumulate))
    warmup = max(1, total_steps // 20)

    def lr_at(step):
        if step < warmup:
            return args.lr * (step + 1) / warmup
        progress = (step - warmup) / max(1, total_steps - warmup)
        return args.lr * 0.5 * (1 + math.cos(math.pi * min(1.0, progress)))

    def held_out_loss():
        if not val:
            return None
        model.eval()
        losses = []
        with torch.no_grad():
            for ids, reply_length in val:
                losses.append(reply_loss(model, ids, reply_length).item())
        model.train()
        return sum(losses) / len(losses)

    log = {"base": args.base, "architectures": architectures, "examples": len(data), "skipped": skipped,
           "rank": args.rank, "alpha": args.alpha, "lr": args.lr, "accumulate": args.accumulate,
           "epochs_requested": args.epochs, "steps_planned": total_steps, "steps": [],
           "held_out_before": held_out_loss(), "threads": torch.get_num_threads(),
           "gradient_checkpointing": not args.no_checkpointing, "device": DEVICE}
    print(f"held-out loss before training: {log['held_out_before']}", flush=True)
    model.train()
    step = 0
    seen = 0
    order = []
    epochs_whole = math.ceil(args.epochs)
    for epoch in range(epochs_whole):
        indices = list(range(len(data)))
        random.Random(args.seed + epoch).shuffle(indices)
        order.extend(indices)
    order = order[:int(len(data) * args.epochs)]
    stopped_for_budget = False
    running = []
    for position, index in enumerate(order):
        if args.budget_minutes and (time.time() - started) / 60 > args.budget_minutes:
            stopped_for_budget = True
            print(f"stopping at the time budget after {seen} examples", flush=True)
            break
        ids, reply_length = data[index]
        loss = reply_loss(model, ids, reply_length) / args.accumulate
        loss.backward()
        running.append(loss.item() * args.accumulate)
        seen += 1
        if seen % args.accumulate == 0 or position == len(order) - 1:
            for group in optimizer.param_groups:
                group["lr"] = lr_at(step)
            torch.nn.utils.clip_grad_norm_(trainable, 1.0)
            optimizer.step()
            optimizer.zero_grad(set_to_none=True)
            entry = {"step": step, "examples": seen, "loss": sum(running) / len(running), "lr": lr_at(step),
                     "minutes": round((time.time() - started) / 60, 2)}
            log["steps"].append(entry)
            print(json.dumps(entry), flush=True)
            running = []
            step += 1
    log["examples_seen"] = seen
    log["stopped_for_budget"] = stopped_for_budget
    log["held_out_after"] = held_out_loss()
    print(f"held-out loss after training: {log['held_out_after']}", flush=True)

    os.makedirs(args.out, exist_ok=True)
    model.save_pretrained(os.path.join(args.out, "adapter"))
    merged = model.merge_and_unload()
    saved_dir = os.path.join(args.out, "merged-transformers")
    merged.save_pretrained(saved_dir, safe_serialization=True, max_shard_size="20GB")
    del merged, model, optimizer, trainable, data
    gc.collect()
    merged_dir = os.path.join(args.out, "merged")
    log["merge"] = write_merged(args.base, saved_dir, tokenizer, merged_dir)
    shutil.rmtree(saved_dir)
    print(f"merge: {log['merge']}", flush=True)
    log["minutes"] = round((time.time() - started) / 60, 2)
    with open(os.path.join(args.out, "train-log.json"), "w", encoding="utf-8") as file:
        json.dump(log, file, indent=1)
    print(f"done in {log['minutes']} minutes; merged checkpoint in {merged_dir}", flush=True)


if __name__ == "__main__":
    main()

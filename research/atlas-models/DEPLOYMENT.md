# ATLAS deployment: how a student would ship

Status, 2026-11-24: nothing is ready to deploy. No candidate has passed the switch rule, and
nothing in the app has been changed by this work. This file is the plan for the day one does,
so that shipping it is a reviewed manifest change and not a new project.

## What a student must have passed first

In this order, each step on the result of the one before:

1. **Development screen.** Live on real1 to real6 with the app's worker, prompt, grammar,
   llama-server pin and 0.7 gate, beside a same-run 9B: at least as many names right, no more
   wrong names filed, at most 0.6 of the 9B's time on the same runner. `atlas-research.yml`
   does this from a run spec.
2. **A second run.** The repository's switch rule asks for two runs. The 9B moved by one
   document of 301 between runs; a student's margin should be clearly larger than that.
3. **Held-out answer.** real7 and real8, 110 documents never screened, run once, and only with
   Zack's say-so. This is the only clean test left; spending it on a candidate that has not
   passed steps 1 and 2 wastes it.
4. **Laptop check.** On at least one 16 GB Windows laptop, CPU only: time per document, peak
   memory with the app, a browser and the OS running, and the student's ratio to the 9B on
   the same laptop. Every speed figure in this work is from Linux runners.
5. **Readiness calibration.** The student's unsafe filings at the 0.7 gate, measured on steps 1
   to 3. The first fine-tuned 0.8B filed 2 wrong of its 4 ready real documents, so a trained
   student's gate cannot be assumed to behave like the base model's.

## The change in the app, once it has passed

The app selects models only through `src-tauri/resources/model-manifest.json`, which
`crates/intern-engine/src/manifest.rs` validates: each file must come from an immutable
revision URL with its exact size and lower-case SHA-256.

| Step | File | What changes |
|---|---|---|
| Host the GGUF | a Hugging Face model repository Zack owns | Upload the quantized student; record the commit it lands at. Nothing is created until he says so. |
| Point the tier at it | `src-tauri/resources/model-manifest.json` | The tier's `model_id`, `display_name`, file `name`, revision `url`, `size`, `sha256` |
| Update the pinned-model test | `crates/intern-engine/src/manifest.rs`, `the_pinned_models_are_qwen35_9b_and_4b` | New names and digest; the test exists so this change is deliberate |
| Memory floor | `crates/intern-engine/src/tiers.rs`, `MIN_AVAILABLE_FOR_FAST` or `_ACCURATE` | Lower it to the measured need plus the worker's peak; a 2B student needs about 1.6 GB of server memory against the 4B's 3.3 GB |
| Gate | `tiers.rs`, `REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE` | Only if calibration in step 5 says so |
| Escalation | `tiers.rs`, `ESCALATION` and `ESCALATE_BELOW_TOKEN_CONFIDENCE` | Today a placeholder 0.5 and switched off. Set only from a replay of the student's own readings with `scripts/simulate_hybrid.py`, confirmed on documents it was not fitted on |
| Download budget | `manifest.rs`, `the_whole_download_fits_a_sixteen_gigabyte_laptop` | Must still pass; a 2B in place of the 4B shrinks the total by about 1.5 GB |

A fine-tuned Qwen3.5 student keeps the architecture, tokenizer and chat template, so the
prompt, grammar, evidence handles, server flags and log-probability path need no change. A
student from another family would need all of those re-checked and is not planned.

## Which tier a student would take

- **A student that passes the switch rule outright** replaces the 9B in the accurate tier. That
  is the brief's goal and the strongest claim, so it needs every step above.
- **A student that is better than the 4B but short of the 9B** can replace the 4B in the quick
  tier. Quick readings are always held for review today, so this cannot file a wrong name
  unreviewed; it needs steps 1, 2 and 4, measured against the 4B instead of the 9B.
- **A student whose confidence separates its right readings from its wrong ones** makes the
  quick-then-accurate cascade worth switching on. The hybrid replay shows the ceiling: with
  perfect routing, the 2B plus the 9B would name 179 of 313 right against the 9B's 160 at 0.47
  of its time (MODEL_COMPARISON.md). The stock 2B's confidence gets nowhere near that.

## Release and rollback

- Ship behind an engine alpha, as the repository already does, with the qualification report
  committed under `bench/reports/` like the November 2026 one.
- Rollback is reverting the manifest commit. The app re-downloads the pinned previous file; no
  user data depends on which model produced a name.
- License: every Qwen3.5 model is Apache 2.0. A fine-tuned student is a derivative work that
  may be distributed commercially; keep the license and a notice that the weights were modified.

## What is deliberately not part of this

- No production setting, gate, model or pipeline was changed by this research.
- No weights or training data are in Git; the workflows keep them as artifacts or rebuild them.
- No document from real1 to real8 is training data, and no private document was used anywhere.

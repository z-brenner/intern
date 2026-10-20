#!/usr/bin/env bash
# Rebuilds the Indra reference goldens (spec §17) from scratch.
#
# usage: bench/tools/indra/regenerate.sh VENV MODEL_DIR INPUTS_DIR OUT_DIR
#
#   VENV        a virtualenv with requirements.txt installed
#   MODEL_DIR   the models (fetch_models.sh)
#   INPUTS_DIR  the pages (see README.md for where each comes from)
#   OUT_DIR     where the goldens go: auto/, ocr/, latin-only/, stages/,
#               intermediate/<page>/, lm_vectors.json, onnx_dump.txt
#
# Everything runs on one ONNX Runtime thread (`--cores 1`, `threads=1`): the
# runtime's results move by float ulps with the thread count.
set -euo pipefail

venv=${1:?usage: regenerate.sh VENV MODEL_DIR INPUTS_DIR OUT_DIR}
models=$(realpath "${2:?}")
inputs=$(realpath "${3:?}")
out=${4:?}
tools=$(dirname "$(realpath "$0")")
# Page paths are passed word-split below.
[[ "$inputs" != *[[:space:]]* ]] || { echo "INPUTS_DIR must not contain spaces" >&2; exit 1; }
python="$venv/bin/python"
mkdir -p "$out"
out=$(realpath "$out")
# Run the scripts from outside any source tree, isolated (-I), so nothing
# beside them is imported in place of the installed package.
cd "$out"
export INDRA_OCR_MODEL_DIR="$models"

images=(document-image.png document-image.jpg document-image.tiff
  rotated-low-resolution-scan.png scan.png multi_column.png rotated_letter.png
  table_pmc.png tilted_memo.png)
pdfs=(scanned-lease.pdf mixed-signature.pdf vendor-invoice.pdf)
paths() { for name in "$@"; do echo "$inputs/$name"; done; }

"$python" -I "$tools/dump_onnx.py" "$models"/*.onnx >onnx_dump.txt

# The CLI's own JSON lines, one file a page named after it (the CLI's --out
# would name document-image.png, .jpg and .tiff alike): default mode, and
# OCR forced for the PDFs.
mkdir -p auto ocr
for name in "${images[@]}" "${pdfs[@]}"; do
  "$venv/bin/indra-ocr" --format json --cores 1 --model-dir "$models" "$inputs/$name" >"auto/$name.jsonl"
done
for name in "${pdfs[@]}"; do
  "$venv/bin/indra-ocr" --format json --cores 1 --model-dir "$models" --mode ocr "$inputs/$name" >"ocr/$name.jsonl"
done

# The Latin-only variant Intern ports (no script routing).
"$python" -I "$tools/dump_latin_only.py" "$models" latin-only \
  $(paths "${images[@]:0:4}" multi_column.png rotated_letter.png table_pmc.png tilted_memo.png)

# Per-stage values and first-call tensors of every network.
"$python" -I "$tools/dump_stages.py" "$models" stages \
  $(paths multi_column.png tilted_memo.png table_pmc.png document-image.png rotated-low-resolution-scan.png)
for name in multi_column tilted_memo document-image rotated-low-resolution-scan; do
  "$python" -I "$tools/dump_intermediates.py" "$models" "$inputs/$name.png" "intermediate/$name" >/dev/null
done

# Beam-search cases and language-model queries.
"$python" -I "$tools/dump_lm_vectors.py" "$models" lm_vectors.json \
  $(paths tilted_memo.png rotated-low-resolution-scan.png scan.png table_pmc.png)

echo "goldens in $out"

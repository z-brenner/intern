#!/usr/bin/env bash
# Stages ONNX Runtime and the PP-OCR models into a runtime directory on
# Linux x86-64, for running the worker's PP-OCR engine and the OCR
# benchmarks locally. Windows packages get the same models, and Microsoft's
# Windows build of the same ONNX Runtime release, from
# scripts/fetch-windows-assets.ps1. See docs/ocr.md.
#
# Usage:
#   scripts/fetch-ocr-runtime.sh RUNTIME_DIR [CACHE_DIR]
#
# RUNTIME_DIR receives libonnxruntime.so and ocr-models/*.onnx, which is
# where the worker looks for them (alongside PDFium, Tesseract and tessdata,
# which this does not fetch). Point INTERN_RUNTIME_DIR at it.
#
# The models are pinned by src-tauri/resources/runtime-assets.json - the
# same URL, size and SHA-256 the Windows package uses. ONNX Runtime comes
# from the official PyPI wheel, pinned here by size and SHA-256, and the
# library taken out of it is checked again against its own digest. Every
# file is verified before anything is written to RUNTIME_DIR, and a file
# that fails is never staged. Downloads are kept in CACHE_DIR when one is
# given and verified again on reuse.
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  sed -n '2,/^set -euo/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
  exit 2
fi

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) ;;
  *) echo "fetch-ocr-runtime.sh stages the Linux x86-64 runtime; on Windows use scripts/fetch-windows-assets.ps1" >&2; exit 1 ;;
esac

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/src-tauri/resources/runtime-assets.json"
RUNTIME_DIR="$1"

# ONNX Runtime 1.30.0, the release the Windows package pins, as the CPU
# wheel publishes it for CPython 3.11 on manylinux 2.28. Only the shared
# library inside is used; no Python is needed.
ORT_WHEEL_URL="https://files.pythonhosted.org/packages/4f/2e/4c96278a99140d0307ccda6fd51e6e9d1ca7c9f6fdd1fa96ff375acfc713/onnxruntime-1.30.0-cp311-cp311-manylinux_2_28_x86_64.whl"
ORT_WHEEL_SIZE=23561046
ORT_WHEEL_SHA256="fd54b314ea385bcecac69ab431f020ba503e3878dad4ebb645fec5a24b041242"
ORT_LIBRARY_MEMBER="onnxruntime/capi/libonnxruntime.so.1.30.0"
ORT_LIBRARY_SIZE=28985152
ORT_LIBRARY_SHA256="c902c70b3003c0e99fada202f37478c515ae9bba7944c2b2abd0017bae0c82ed"

for tool in curl sha256sum unzip node; do
  command -v "$tool" >/dev/null || { echo "fetch-ocr-runtime.sh needs $tool" >&2; exit 1; }
done

if [ "$#" -eq 2 ]; then
  CACHE="$2"
  mkdir -p "$CACHE"
  OWN_CACHE=0
else
  CACHE="$(mktemp -d)"
  OWN_CACHE=1
fi
STAGE="$(mktemp -d)"
cleanup() {
  rm -rf "$STAGE"
  if [ "$OWN_CACHE" -eq 1 ]; then rm -rf "$CACHE"; fi
}
trap cleanup EXIT

# Whether FILE has exactly SIZE bytes and SHA256.
matches() {
  local file="$1" size="$2" sha256="$3"
  [ -f "$file" ] && [ "$(stat -c %s "$file")" = "$size" ] &&
    [ "$(sha256sum "$file" | cut -d' ' -f1)" = "$sha256" ]
}

# Downloads URL to CACHE/NAME unless a verified copy is there, retrying a
# failed transfer; fails unless the result has SIZE bytes and SHA256.
fetch() {
  local url="$1" name="$2" size="$3" sha256="$4"
  local target="$CACHE/$name"
  if matches "$target" "$size" "$sha256"; then
    return
  fi
  rm -f "$target"
  local attempt
  for attempt in 1 2 3 4; do
    if curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
      --output "$target.part" "$url"; then
      mv "$target.part" "$target"
      break
    fi
    rm -f "$target.part"
    if [ "$attempt" -eq 4 ]; then
      echo "failed to download $name after $attempt attempts" >&2
      exit 1
    fi
    sleep $((5 * attempt))
  done
  if ! matches "$target" "$size" "$sha256"; then
    echo "$name does not match its pinned size and SHA-256; refusing it" >&2
    rm -f "$target"
    exit 1
  fi
}

fetch "$ORT_WHEEL_URL" "$(basename "$ORT_WHEEL_URL")" "$ORT_WHEEL_SIZE" "$ORT_WHEEL_SHA256"
unzip -p "$CACHE/$(basename "$ORT_WHEEL_URL")" "$ORT_LIBRARY_MEMBER" > "$STAGE/libonnxruntime.so"
if ! matches "$STAGE/libonnxruntime.so" "$ORT_LIBRARY_SIZE" "$ORT_LIBRARY_SHA256"; then
  echo "the ONNX Runtime library in the pinned wheel does not match its pinned digest" >&2
  exit 1
fi

# The model pins, read from the manifest the Windows package is built from.
mkdir -p "$STAGE/ocr-models"
while IFS=$'\t' read -r id archive url size sha256; do
  fetch "$url" "$archive" "$size" "$sha256"
  cp "$CACHE/$archive" "$STAGE/ocr-models/$archive"
done < <(node -e '
  const manifest = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
  const wanted = ["ocr-text-detection", "ocr-text-recognition", "ocr-page-orientation"];
  for (const id of wanted) {
    const pin = manifest.downloads.find((download) => download.id === id);
    if (!pin) { console.error(`runtime-assets.json has no ${id} pin`); process.exit(1); }
    if (!/^[\w.-]+\.onnx$/.test(pin.archive)) { console.error(`unsafe archive name for ${id}`); process.exit(1); }
    console.log([id, pin.archive, pin.url, pin.size, pin.sha256].join("\t"));
  }
' "$MANIFEST")

if [ "$(find "$STAGE/ocr-models" -name '*.onnx' | wc -l)" -ne 3 ]; then
  echo "expected three OCR models to stage" >&2
  exit 1
fi

mkdir -p "$RUNTIME_DIR/ocr-models"
install -m 0644 "$STAGE/libonnxruntime.so" "$RUNTIME_DIR/libonnxruntime.so"
for model in "$STAGE"/ocr-models/*.onnx; do
  install -m 0644 "$model" "$RUNTIME_DIR/ocr-models/$(basename "$model")"
done
echo "Staged ONNX Runtime 1.30.0 and the PP-OCR models in $RUNTIME_DIR"

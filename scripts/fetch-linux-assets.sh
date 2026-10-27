#!/usr/bin/env bash
# Fetches and verifies the runtime the Linux x86-64 package carries,
# and stages it in src-tauri/resources/runtime/ with its license texts in
# src-tauri/resources/licenses/: the Linux counterpart of
# scripts/fetch-windows-assets.ps1. See docs/linux.md.
#
# Usage:
#   scripts/fetch-linux-assets.sh [CACHE_DIR]
#
# Build the parser worker first (INTERN_WORKER names a build elsewhere):
#   cargo build --locked -p intern-worker --release --features windows-native
#
# What the runtime holds, and how each part is pinned:
#   llama-server, its libraries, and one CPU backend per x86-64 generation:
#       from the PrismML llama.cpp release pinned below, Prism's prebuilt
#       Ubuntu x86-64 archive, by size and SHA-256.
#   libpdfium.so: PDFium chromium/7881 from pdfium-binaries, by size and
#       SHA-256, as the Windows package's.
#   libonnxruntime.so and ocr-models/: scripts/fetch-ocr-runtime.sh, by its
#       own pins and the models' in runtime-assets.json.
#   tessdata/: eng and osd, by the runtime-assets.json pins the Windows
#       package uses.
#   intern-worker: the build named above, recorded by digest.
# Tesseract itself is not bundled: the package depends on the distribution's
# tesseract-ocr, which reads the pinned tessdata (docs/linux.md says why).
#
# Every download is refused unless its exact byte length and SHA-256 match,
# and nothing in src-tauri/resources is touched until all of it has been
# fetched and checked. The staged set is recorded, with the size and
# SHA-256 of every file, in runtime/runtime-inventory.json, which
# scripts/stage-linux-runtime.sh checks before it copies a byte. Downloads
# are kept in CACHE_DIR when one is given, and verified again on reuse.
set -euo pipefail

if [ "$#" -gt 1 ]; then
  sed -n '2,/^set -euo/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
  exit 2
fi

# ---------------------------------------------------------------------------
# THE LLAMA.CPP PIN. The local model runs on the PrismML fork of llama.cpp,
# which reads the Bonsai 2 ternary models, as Prism's own prebuilt Ubuntu
# x86-64 release. To move the pin, change the release on the next line; it
# must be one of the releases whose archive size and SHA-256 are pinned in
# the table below it (add a row for a new one). LLAMA_CPP_RELEASE in the
# environment picks another pinned release for a trial build, and the
# inventory always records which one was staged.
LLAMA_CPP_RELEASE="${LLAMA_CPP_RELEASE:-prism-b10826-c9c34bc}"
case "$LLAMA_CPP_RELEASE" in
  prism-b10777-e3de715) LLAMA_CPP_SIZE=17882710 LLAMA_CPP_SHA256="3f97b6fbec0065fede2be92f05df531c1050636fc2613e1ba0d975249df19713" ;;
  prism-b10826-c9c34bc) LLAMA_CPP_SIZE=17912930 LLAMA_CPP_SHA256="66a250f38c3f8e2064e2f913b3a26ff5bae10955b67e8ff38c72ab48cbecf28c" ;;
  *) echo "llama.cpp $LLAMA_CPP_RELEASE has no pinned size and SHA-256 in scripts/fetch-linux-assets.sh" >&2; exit 1 ;;
esac
# ---------------------------------------------------------------------------
LLAMA_CPP_REPOSITORY="https://github.com/PrismML-Eng/llama.cpp"
LLAMA_CPP_ARCHIVE="llama-$LLAMA_CPP_RELEASE-bin-ubuntu-x64.tar.gz"
LLAMA_CPP_URL="$LLAMA_CPP_REPOSITORY/releases/download/$LLAMA_CPP_RELEASE/$LLAMA_CPP_ARCHIVE"

PDFIUM_VERSION="chromium/7881"
PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7881/pdfium-linux-x64.tgz"
PDFIUM_SIZE=3644759
PDFIUM_SHA256="1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d"

# tessdata_fast's Apache-2.0 text, from the commit the traineddata is pinned
# to. The Windows package takes the same text from its Tesseract build.
TESSDATA_LICENSE_URL="https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/87416418657359cb625c412a48b6e1d6d41c29bd/LICENSE"
TESSDATA_LICENSE_SIZE=11358
TESSDATA_LICENSE_SHA256="cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) ;;
  *) echo "fetch-linux-assets.sh stages the Linux x86-64 runtime; on Windows use scripts/fetch-windows-assets.ps1" >&2; exit 1 ;;
esac

for tool in curl sha256sum tar unzip readelf ldd node; do
  command -v "$tool" >/dev/null || { echo "fetch-linux-assets.sh needs $tool" >&2; exit 1; }
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/src-tauri/resources/runtime-assets.json"
RESOURCES="$ROOT/src-tauri/resources"
WORKER="${INTERN_WORKER:-$ROOT/target/release/intern-worker}"
INTERN_VERSION="$(node -p 'require(process.argv[1]).version' "$ROOT/src-tauri/tauri.conf.json")"
if [ ! -f "$WORKER" ] || [ ! -x "$WORKER" ]; then
  echo "the parser worker is not built at $WORKER; build it with" >&2
  echo "  cargo build --locked -p intern-worker --release --features windows-native" >&2
  exit 1
fi

if [ "$#" -eq 1 ]; then
  CACHE="$(mkdir -p "$1" && cd "$1" && pwd)"
  OWN_CACHE=0
else
  CACHE="$(mktemp -d)"
  OWN_CACHE=1
fi
WORK="$(mktemp -d)"
cleanup() {
  rm -rf "$WORK"
  if [ "$OWN_CACHE" -eq 1 ]; then rm -rf "$CACHE"; fi
}
trap cleanup EXIT
STAGE="$WORK/package"
mkdir -p "$STAGE/runtime/tessdata" "$STAGE/licenses"
PACKAGES="$WORK/packages.tsv"
: > "$PACKAGES"

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

# Records that the staged runtime file PATH (under runtime/) belongs to NAME
# at VERSION.
owned_by() {
  printf 'runtime/%s\t%s\t%s\n' "$1" "$2" "$3" >> "$PACKAGES"
}

# Refuses an archive with an entry that is absolute or climbs out of where it
# is extracted.
assert_safe_tar() {
  local entry
  while IFS= read -r entry; do
    case "/$entry/" in
      //* | */../*) echo "unsafe archive entry in $1: $entry" >&2; exit 1 ;;
    esac
  done < <(tar -tzf "$1")
}

# The ELF libraries FILE names as NEEDED.
needed() {
  readelf -d "$1" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p'
}

# --- llama.cpp ------------------------------------------------------------
fetch "$LLAMA_CPP_URL" "$LLAMA_CPP_ARCHIVE" "$LLAMA_CPP_SIZE" "$LLAMA_CPP_SHA256"
assert_safe_tar "$CACHE/$LLAMA_CPP_ARCHIVE"
mkdir -p "$WORK/llama.cpp"
tar -xzf "$CACHE/$LLAMA_CPP_ARCHIVE" -C "$WORK/llama.cpp"
LLAMA_BIN="$WORK/llama.cpp/llama-$LLAMA_CPP_RELEASE"
[ -f "$LLAMA_BIN/llama-server" ] || { echo "$LLAMA_CPP_ARCHIVE has no llama-$LLAMA_CPP_RELEASE/llama-server" >&2; exit 1; }

# The archive carries every llama.cpp tool. Only llama-server is staged, with
# every library it needs that the archive carries, each under the name it is
# needed by (its soname, which in the archive is a link), followed through the
# libraries' own needs; and every CPU backend, one per x86-64 generation from
# sse42 to zen4, which llama-server picks from at run time by the CPU it finds
# - they are loaded by name, so nothing NEEDs them. The RPC backend is left
# out: Intern's server talks to nothing but Intern. What the archive does not
# carry - libc, libstdc++, libgomp, OpenSSL - comes from the distribution,
# and the package depends on it.
declare -A STAGED=()
pending=(llama-server)
for backend in "$LLAMA_BIN"/libggml-cpu-*.so; do
  pending+=("$(basename "$backend")")
done
if [ "${#pending[@]}" -lt 2 ]; then
  echo "$LLAMA_CPP_ARCHIVE carries no CPU backends" >&2
  exit 1
fi
while [ "${#pending[@]}" -gt 0 ]; do
  name="${pending[0]}"
  pending=("${pending[@]:1}")
  if [ -n "${STAGED[$name]:-}" ] || [ ! -e "$LLAMA_BIN/$name" ]; then
    continue
  fi
  # A link in the archive must name a file in the archive.
  source="$(readlink -f "$LLAMA_BIN/$name")"
  case "$source" in
    "$LLAMA_BIN"/*) ;;
    *) echo "$name in $LLAMA_CPP_ARCHIVE points outside the archive" >&2; exit 1 ;;
  esac
  STAGED[$name]=1
  install -m 0755 "$source" "$STAGE/runtime/$name"
  owned_by "$name" llama.cpp "$LLAMA_CPP_RELEASE"
  while IFS= read -r library; do
    pending+=("$library")
  done < <(needed "$STAGE/runtime/$name")
done
# Each looks for the others beside itself, so the runtime folder works
# wherever it is put.
for name in "${!STAGED[@]}"; do
  runpath="$(readelf -d "$STAGE/runtime/$name" | sed -n 's/.*(RUNPATH).*\[\(.*\)\]/\1/p')"
  if [ "$runpath" != '$ORIGIN' ]; then
    echo "$name looks for its libraries in '$runpath', not beside itself" >&2
    exit 1
  fi
done
install -m 0644 "$LLAMA_BIN/LICENSE" "$STAGE/licenses/llama.cpp-LICENSE.txt"

# --- PDFium ---------------------------------------------------------------
fetch "$PDFIUM_URL" pdfium-linux-x64.tgz "$PDFIUM_SIZE" "$PDFIUM_SHA256"
assert_safe_tar "$CACHE/pdfium-linux-x64.tgz"
mkdir -p "$WORK/pdfium"
tar -xzf "$CACHE/pdfium-linux-x64.tgz" -C "$WORK/pdfium"
install -m 0644 "$WORK/pdfium/lib/libpdfium.so" "$STAGE/runtime/libpdfium.so"
owned_by libpdfium.so PDFium "$PDFIUM_VERSION"
install -m 0644 "$WORK/pdfium/LICENSE" "$STAGE/licenses/PDFium-LICENSE.txt"

# --- ONNX Runtime and the PP-OCR models -----------------------------------
"$ROOT/scripts/fetch-ocr-runtime.sh" "$WORK/ocr" "$CACHE" > /dev/null
install -m 0644 "$WORK/ocr/libonnxruntime.so" "$STAGE/runtime/libonnxruntime.so"
owned_by libonnxruntime.so "ONNX Runtime" 1.30.0
mkdir -p "$STAGE/runtime/ocr-models"
while IFS=$'\t' read -r archive name version; do
  install -m 0644 "$WORK/ocr/ocr-models/$archive" "$STAGE/runtime/ocr-models/$archive"
  owned_by "ocr-models/$archive" "$name" "$version"
done < <(node -e '
  const manifest = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
  for (const id of ["ocr-text-detection", "ocr-text-recognition", "ocr-page-orientation"]) {
    const pin = manifest.downloads.find((download) => download.id === id);
    const repository = new URL(pin.url).pathname.split("/").slice(1, 3).join("/");
    console.log([pin.archive, repository, pin.version].join("\t"));
  }
' "$MANIFEST")
install -m 0644 "$WORK"/ocr/licenses/*.txt "$STAGE/licenses/"

# --- tessdata -------------------------------------------------------------
while IFS=$'\t' read -r archive url size sha256 version; do
  fetch "$url" "$archive" "$size" "$sha256"
  install -m 0644 "$CACHE/$archive" "$STAGE/runtime/tessdata/$archive"
  owned_by "tessdata/$archive" tessdata_fast "$version"
done < <(node -e '
  const manifest = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
  for (const id of ["eng.traineddata", "osd.traineddata"]) {
    const pin = manifest.downloads.find((download) => download.id === id);
    if (!pin || pin.archive !== id) { console.error(`runtime-assets.json has no ${id} pin`); process.exit(1); }
    console.log([pin.archive, pin.url, pin.size, pin.sha256, pin.version].join("\t"));
  }
' "$MANIFEST")
fetch "$TESSDATA_LICENSE_URL" tessdata_fast-LICENSE.txt "$TESSDATA_LICENSE_SIZE" "$TESSDATA_LICENSE_SHA256"
install -m 0644 "$CACHE/tessdata_fast-LICENSE.txt" "$STAGE/licenses/tessdata-Apache-2.0.txt"

# --- the parser worker ----------------------------------------------------
install -m 0755 "$WORKER" "$STAGE/runtime/intern-worker"
owned_by intern-worker Intern "$INTERN_VERSION"

# --- check the staged runtime before it replaces anything -------------------
# Every library the staged executables need resolves, from beside them or
# from the system, and llama-server runs and finds its CPU backends.
for name in llama-server intern-worker "${!STAGED[@]}"; do
  if ldd "$STAGE/runtime/$name" | grep -q 'not found'; then
    echo "$name needs a library that is neither staged nor installed:" >&2
    ldd "$STAGE/runtime/$name" | grep 'not found' >&2
    exit 1
  fi
done
if ! (cd / && "$STAGE/runtime/llama-server" --version) > "$WORK/llama-server-version.txt" 2>&1; then
  cat "$WORK/llama-server-version.txt" >&2
  echo "the staged llama-server does not run" >&2
  exit 1
fi
node -e '
  const [, file, repository, release, archive, url, size, sha256] = process.argv;
  require("fs").writeFileSync(file, JSON.stringify({ repository, release, archive, url, size: Number(size), sha256 }));
' "$WORK/llama.cpp.json" "$LLAMA_CPP_REPOSITORY" "$LLAMA_CPP_RELEASE" "$LLAMA_CPP_ARCHIVE" "$LLAMA_CPP_URL" "$LLAMA_CPP_SIZE" "$LLAMA_CPP_SHA256"
node "$ROOT/scripts/linux-runtime-inventory.mjs" write "$STAGE" "$PACKAGES" "$WORK/llama.cpp.json"
node "$ROOT/scripts/linux-runtime-inventory.mjs" verify "$STAGE"

# Everything is fetched and checked; only now is the tree replaced.
rm -rf "$RESOURCES/runtime" "$RESOURCES/licenses"
cp -a "$STAGE/runtime" "$RESOURCES/runtime"
cp -a "$STAGE/licenses" "$RESOURCES/licenses"
node "$ROOT/scripts/linux-runtime-inventory.mjs" verify "$RESOURCES"
echo "Pinned Linux runtime assets are staged and verified in src-tauri/resources/runtime (llama.cpp $LLAMA_CPP_RELEASE)."

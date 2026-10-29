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
#   llama-server, its libraries, and one CPU backend per x86-64 generation,
#       twice - two llama.cpp builds, each its publisher's prebuilt Ubuntu
#       x86-64 archive, by size and SHA-256:
#         Prism: the PrismML fork's release pinned below, flat in runtime/,
#           which runs the fast tier (the Bonsai 2 ternary models need it).
#         Upstream: ggml-org's release pinned below, in
#           runtime/llama-upstream/, which runs the accurate tier.
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
# THE LLAMA.CPP PINS. The local models run on two builds of llama.cpp, each
# its publisher's own prebuilt Ubuntu x86-64 release.
#
# Prism: the PrismML fork, which reads the Bonsai 2 ternary models, and runs
# the fast tier from runtime/. To move the pin, change the release on the
# next line; it must be one of the releases whose archive size and SHA-256
# are pinned in the table below it (add a row for a new one).
# LLAMA_CPP_RELEASE in the environment picks another pinned release for a
# trial build, and the inventory always records which one was staged.
LLAMA_CPP_RELEASE="${LLAMA_CPP_RELEASE:-prism-b10826-c9c34bc}"
case "$LLAMA_CPP_RELEASE" in
  prism-b10777-e3de715) LLAMA_CPP_SIZE=17882710 LLAMA_CPP_SHA256="3f97b6fbec0065fede2be92f05df531c1050636fc2613e1ba0d975249df19713" ;;
  prism-b10826-c9c34bc) LLAMA_CPP_SIZE=17912930 LLAMA_CPP_SHA256="66a250f38c3f8e2064e2f913b3a26ff5bae10955b67e8ff38c72ab48cbecf28c" ;;
  *) echo "llama.cpp $LLAMA_CPP_RELEASE has no pinned size and SHA-256 in scripts/fetch-linux-assets.sh" >&2; exit 1 ;;
esac
#
# Upstream: ggml-org's llama.cpp, which runs the accurate tier from
# runtime/llama-upstream/. Its pin is these three values and nothing else:
# the environment cannot choose another, and moving it means changing all
# three from the release's page.
LLAMA_UPSTREAM_RELEASE="b11946"
LLAMA_UPSTREAM_SIZE=22647372
LLAMA_UPSTREAM_SHA256="9e44e1a59e971f0de3fbf74fa46355ab48fed7bb51c1c16b56a96c355947f8dd"
# ---------------------------------------------------------------------------
LLAMA_CPP_REPOSITORY="https://github.com/PrismML-Eng/llama.cpp"
LLAMA_UPSTREAM_REPOSITORY="https://github.com/ggml-org/llama.cpp"
# Where in runtime/ the upstream build goes: the app starts the accurate
# tier's server from runtime/llama-upstream/llama-server.
LLAMA_UPSTREAM_FOLDER="llama-upstream"

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

for tool in curl sha256sum tar unzip readelf ldd cmp node; do
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
# One line per staged llama.cpp build, for the inventory.
LLAMA_BUILDS="$WORK/llama.cpp.tsv"
: > "$LLAMA_BUILDS"
# Every staged llama.cpp file, and each build's llama-server, as paths under
# runtime/, and the release of each server, for the checks before anything is
# replaced.
LLAMA_STAGED=()
LLAMA_SERVERS=()
LLAMA_SERVER_RELEASES=()

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
# Stages llama.cpp RELEASE - REPOSITORY's prebuilt Ubuntu x86-64 archive,
# which must have SIZE bytes and SHA256 - into runtime/FOLDER, or flat into
# runtime/ when FOLDER is ".".
#
# The archive carries every llama.cpp tool. Only llama-server is staged, with
# every library it needs that the archive carries, each under the name it is
# needed by (its soname, which in the archive is a link), followed through the
# libraries' own needs; and every CPU backend, one per x86-64 generation from
# sse42 to zen4, which llama-server picks from at run time by the CPU it finds
# - they are loaded by name, so nothing NEEDs them. The RPC backend is left
# out: Intern's server talks to nothing but Intern. What the archive does not
# carry - libc, libstdc++, libgomp, OpenSSL - comes from the distribution,
# and the package depends on it.
#
# The two builds carry libraries of the same names (libllama.so.0,
# libggml-base.so.0, ...), and each server must load its own build's: that is
# why each build has a folder to itself and every file in it looks for the
# others beside itself, and what the checks before the tree is replaced
# confirm.
stage_llama() {
  local release="$1" repository="$2" size="$3" sha256="$4" folder="$5"
  local archive="llama-$release-bin-ubuntu-x64.tar.gz"
  local url="$repository/releases/download/$release/$archive"
  local prefix destination
  case "$folder" in
    .) prefix="" destination="$STAGE/runtime" ;;
    */* | .. | "") echo "llama.cpp $release cannot be staged in runtime/$folder" >&2; exit 1 ;;
    *) prefix="$folder/" destination="$STAGE/runtime/$folder" ;;
  esac
  if [ -e "$destination/llama-server" ]; then
    echo "runtime/${prefix}llama-server is staged already; each llama.cpp build needs a folder to itself" >&2
    exit 1
  fi
  mkdir -p "$destination"

  fetch "$url" "$archive" "$size" "$sha256"
  assert_safe_tar "$CACHE/$archive"
  mkdir -p "$WORK/llama.cpp/$release"
  tar -xzf "$CACHE/$archive" -C "$WORK/llama.cpp/$release"
  local bin="$WORK/llama.cpp/$release/llama-$release"
  [ -f "$bin/llama-server" ] || { echo "$archive has no llama-$release/llama-server" >&2; exit 1; }
  [ -f "$bin/LICENSE" ] || { echo "$archive has no llama-$release/LICENSE" >&2; exit 1; }

  local -A staged=()
  local pending=(llama-server) backend name source library runpath
  for backend in "$bin"/libggml-cpu-*.so; do
    if [ -e "$backend" ]; then pending+=("$(basename "$backend")"); fi
  done
  if [ "${#pending[@]}" -lt 2 ]; then
    echo "$archive carries no CPU backends" >&2
    exit 1
  fi
  while [ "${#pending[@]}" -gt 0 ]; do
    name="${pending[0]}"
    pending=("${pending[@]:1}")
    if [ -n "${staged[$name]:-}" ] || [ ! -e "$bin/$name" ]; then
      continue
    fi
    # A link in the archive must name a file in the archive.
    source="$(readlink -f "$bin/$name")"
    case "$source" in
      "$bin"/*) ;;
      *) echo "$name in $archive points outside the archive" >&2; exit 1 ;;
    esac
    staged[$name]=1
    install -m 0755 "$source" "$destination/$name"
    owned_by "$prefix$name" llama.cpp "$release"
    LLAMA_STAGED+=("$prefix$name")
    while IFS= read -r library; do
      pending+=("$library")
    done < <(needed "$destination/$name")
  done
  # Each looks for the others beside itself, so the build's folder works
  # wherever it is put.
  for name in "${!staged[@]}"; do
    runpath="$(readelf -d "$destination/$name" | sed -n 's/.*(RUNPATH).*\[\(.*\)\]/\1/p')"
    if [ "$runpath" != '$ORIGIN' ]; then
      echo "$name in $archive looks for its libraries in '$runpath', not beside itself" >&2
      exit 1
    fi
  done
  LLAMA_SERVERS+=("${prefix}llama-server")
  LLAMA_SERVER_RELEASES+=("$release")

  # One license text covers every llama.cpp build the package carries, so
  # each build's must be byte for byte the one already staged.
  local license="$STAGE/licenses/llama.cpp-LICENSE.txt"
  if [ ! -e "$license" ]; then
    install -m 0644 "$bin/LICENSE" "$license"
  elif ! cmp -s "$bin/LICENSE" "$license"; then
    echo "llama.cpp $release's LICENSE differs from the one licenses/llama.cpp-LICENSE.txt already holds" >&2
    echo "for another staged llama.cpp build; ship each build's own text before staging both" >&2
    exit 1
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$folder" "$repository" "$release" "$archive" "$url" "$size" "$sha256" >> "$LLAMA_BUILDS"
}

stage_llama "$LLAMA_CPP_RELEASE" "$LLAMA_CPP_REPOSITORY" "$LLAMA_CPP_SIZE" "$LLAMA_CPP_SHA256" .
stage_llama "$LLAMA_UPSTREAM_RELEASE" "$LLAMA_UPSTREAM_REPOSITORY" "$LLAMA_UPSTREAM_SIZE" "$LLAMA_UPSTREAM_SHA256" "$LLAMA_UPSTREAM_FOLDER"

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
owned_by libonnxruntime.so onnxruntime 1.30.0
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
# from the system; every library a llama.cpp file needs that its own build
# carries resolves to the copy beside it, never to the other build's of the
# same name; and each build's llama-server runs and reports its release's
# build number (both publishers name a release b<build>, Prism with a prefix
# and the commit after it).
for path in intern-worker "${LLAMA_STAGED[@]}"; do
  if ldd "$STAGE/runtime/$path" | grep -q 'not found'; then
    echo "runtime/$path needs a library that is neither staged nor installed:" >&2
    ldd "$STAGE/runtime/$path" | grep 'not found' >&2
    exit 1
  fi
done
for path in "${LLAMA_STAGED[@]}"; do
  folder="$(dirname "$STAGE/runtime/$path")"
  while read -r library arrow resolved _; do
    if [ "$arrow" = "=>" ] && [ -e "$folder/$library" ] &&
      [ "$(readlink -f "$resolved")" != "$(readlink -f "$folder/$library")" ]; then
      echo "runtime/$path loads $library from $resolved, not from beside itself" >&2
      exit 1
    fi
  done < <(ldd "$STAGE/runtime/$path")
done
for index in "${!LLAMA_SERVERS[@]}"; do
  server="${LLAMA_SERVERS[$index]}"
  release="${LLAMA_SERVER_RELEASES[$index]}"
  if ! (cd / && "$STAGE/runtime/$server" --version) > "$WORK/llama-server-version.txt" 2>&1; then
    cat "$WORK/llama-server-version.txt" >&2
    echo "the staged runtime/$server does not run" >&2
    exit 1
  fi
  build="$(sed -n 's/^\(prism-\)\{0,1\}b\([0-9]\{1,\}\)\(-.*\)\{0,1\}$/\2/p' <<< "$release")"
  if [ -z "$build" ] || ! grep -qF "(build $build," "$WORK/llama-server-version.txt"; then
    cat "$WORK/llama-server-version.txt" >&2
    echo "the staged runtime/$server does not report llama.cpp $release's build number" >&2
    exit 1
  fi
done
node -e '
  const fs = require("fs");
  const [, tsv, file] = process.argv;
  const builds = fs.readFileSync(tsv, "utf8").split("\n").filter(Boolean).map((line) => {
    const [path, repository, release, archive, url, size, sha256] = line.split("\t");
    return { repository, release, archive, url, size: Number(size), sha256, path };
  });
  fs.writeFileSync(file, JSON.stringify(builds));
' "$LLAMA_BUILDS" "$WORK/llama.cpp.json"
node "$ROOT/scripts/linux-runtime-inventory.mjs" write "$STAGE" "$PACKAGES" "$WORK/llama.cpp.json"
node "$ROOT/scripts/linux-runtime-inventory.mjs" verify "$STAGE"

# Everything is fetched and checked; only now is the tree replaced.
rm -rf "$RESOURCES/runtime" "$RESOURCES/licenses"
cp -a "$STAGE/runtime" "$RESOURCES/runtime"
cp -a "$STAGE/licenses" "$RESOURCES/licenses"
node "$ROOT/scripts/linux-runtime-inventory.mjs" verify "$RESOURCES"
echo "Pinned Linux runtime assets are staged and verified in src-tauri/resources/runtime" \
  "(llama.cpp $LLAMA_CPP_RELEASE, and $LLAMA_UPSTREAM_RELEASE in runtime/$LLAMA_UPSTREAM_FOLDER)."

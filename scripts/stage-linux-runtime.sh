#!/usr/bin/env bash
# Copies the runtime scripts/fetch-linux-assets.sh staged in
# src-tauri/resources into DESTINATION, shaped as the installed package lays
# it out (/usr/lib/Intern): DESTINATION/runtime/ holds llama-server, the
# parser worker and everything they load, and DESTINATION/licenses/ their
# license texts. The Linux counterpart of scripts/stage-windows-runtime.ps1.
#
# Usage:
#   scripts/stage-linux-runtime.sh DESTINATION
#
# Every file is checked against runtime/runtime-inventory.json - its size,
# SHA-256 and executable bit - before anything is copied, and the copy is
# checked again. Point INTERN_RUNTIME_DIR at DESTINATION/runtime to run the
# app, intern-analyze or the benchmarks against it, or hand DESTINATION to
# scripts/smoke-linux.sh --runtime.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  sed -n '2,/^set -euo/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
  exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESOURCES="$ROOT/src-tauri/resources"
DESTINATION="$1"

if [ -e "$DESTINATION" ]; then
  echo "refusing to overwrite an existing runtime stage: $DESTINATION" >&2
  exit 1
fi
if [ ! -f "$RESOURCES/runtime/runtime-inventory.json" ]; then
  echo "no Linux runtime is staged in src-tauri/resources; run scripts/fetch-linux-assets.sh first" >&2
  exit 1
fi

mkdir -p "$DESTINATION"
node "$ROOT/scripts/linux-runtime-inventory.mjs" stage "$RESOURCES" "$DESTINATION"
mkdir -p "$DESTINATION/resources"
install -m 0644 "$RESOURCES/THIRD_PARTY_NOTICES.md" "$DESTINATION/resources/THIRD_PARTY_NOTICES.md"

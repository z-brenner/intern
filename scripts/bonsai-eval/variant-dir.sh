#!/usr/bin/env bash
# Make BIN_DIR-<variant>: symlinks to every file in BIN_DIR except the other
# libggml-cpu-*.so, so the dynamic loader can only pick <variant> (to emulate a
# laptop CPU without AVX-512 on this Sapphire Rapids VM).
# Usage: variant-dir.sh BIN_DIR VARIANT OUT_DIR
set -euo pipefail
BIN=$(cd "$1" && pwd) V=$2 OUT=$3
mkdir -p "$OUT"
for f in "$BIN"/*; do
  b=$(basename "$f")
  case "$b" in
    libggml-cpu-*.so) [ "$b" = "libggml-cpu-$V.so" ] || continue ;;
  esac
  ln -sf "$f" "$OUT/$b"
done
# Binaries resolve $ORIGIN through the symlink target, so copy the executables.
for e in llama-server llama-bench; do rm -f "$OUT/$e"; cp "$BIN/$e" "$OUT/$e"; done
ls "$OUT" | grep libggml-cpu

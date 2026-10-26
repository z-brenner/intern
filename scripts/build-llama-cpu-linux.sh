#!/usr/bin/env bash
# Builds a CPU-only llama-server (and llama-bench) from a llama.cpp checkout,
# such as the PrismML fork Bonsai 2 PQ2_0/PTQ1_0 models need, the way it would
# be shipped to unknown laptops. Laboratory tooling for model evaluation: the
# app does not call it. See docs/bonsai-native-windows-evaluation.md.
#
# Usage:
#   scripts/build-llama-cpu-linux.sh SOURCE_DIR BUILD_DIR [portable|avx2|native]
#
#   portable  (default) like the official release archives: GGML_NATIVE=OFF,
#             GGML_BACKEND_DL=ON, GGML_CPU_ALL_VARIANTS=ON. One libggml-cpu-*.so
#             per x86 micro-architecture; the best one is picked at start-up.
#   avx2      a single static CPU backend for the Haswell baseline
#             (AVX2/FMA/F16C/BMI2), no runtime dispatch.
#   native    -march=native for the build machine; for comparison only, never
#             ship it.
#
# The web UI, OpenSSL/HTTPS downloads, tests and examples are left out: Intern
# runs the server with --no-webui on a local GGUF. Binaries land in
# BUILD_DIR/bin with an $ORIGIN rpath, so the directory can be moved whole.
#
# Pin the source before building, e.g. for the Prism fork:
#   git clone --depth 1 --branch prism-b10777-e3de715 \
#     https://github.com/PrismML-Eng/llama.cpp SOURCE_DIR
# and confirm `git -C SOURCE_DIR rev-parse HEAD` is
# e3de715222b2667c54a267441159a5729f11155e.
set -euo pipefail

SOURCE="${1:?source directory}"
BUILD="${2:?build directory}"
MODE="${3:-portable}"
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"

COMMON=(
  -G Ninja
  -DCMAKE_BUILD_TYPE=Release
  -DCMAKE_INSTALL_RPATH='$ORIGIN'
  -DCMAKE_BUILD_WITH_INSTALL_RPATH=ON
  -DBUILD_SHARED_LIBS=ON
  -DLLAMA_BUILD_TESTS=OFF
  -DLLAMA_BUILD_EXAMPLES=OFF
  -DLLAMA_BUILD_UI=OFF
  -DLLAMA_OPENSSL=OFF
  -DLLAMA_CURL=OFF
  -DGGML_CUDA=OFF -DGGML_VULKAN=OFF -DGGML_METAL=OFF -DGGML_OPENCL=OFF -DGGML_SYCL=OFF
)
case "$MODE" in
  portable) FLAGS=(-DGGML_NATIVE=OFF -DGGML_BACKEND_DL=ON -DGGML_CPU_ALL_VARIANTS=ON) ;;
  avx2)     FLAGS=(-DGGML_NATIVE=OFF -DGGML_BACKEND_DL=OFF -DGGML_AVX=ON -DGGML_AVX2=ON
                   -DGGML_FMA=ON -DGGML_F16C=ON -DGGML_BMI2=ON) ;;
  native)   FLAGS=(-DGGML_NATIVE=ON -DGGML_BACKEND_DL=OFF) ;;
  *) echo "unknown mode $MODE (portable|avx2|native)" >&2; exit 2 ;;
esac

cmake -S "$SOURCE" -B "$BUILD" "${COMMON[@]}" "${FLAGS[@]}"
# With GGML_BACKEND_DL the CPU variants are separate targets that nothing links
# against, so build everything the configured tree defines.
cmake --build "$BUILD" --config Release -j "$JOBS"
ls -la "$BUILD/bin"

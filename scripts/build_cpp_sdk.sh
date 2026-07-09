#!/usr/bin/env bash
# Build the clankeRS C FFI library and C++ SDK.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"

echo ">> Building clankers-ffi (release)"
cargo build -p clankers-ffi --release

FFI_LIB="${CARGO_TARGET_DIR}/release/libclankers_ffi.a"
if [[ ! -f "${FFI_LIB}" ]]; then
  echo "error: expected ${FFI_LIB}" >&2
  exit 1
fi

echo ">> Configuring CMake"
cmake -B "${ROOT}/cpp/build" -S "${ROOT}/cpp" -DCLANKERS_FFI_LIB="${FFI_LIB}"

echo ">> Building C++ SDK"
cmake --build "${ROOT}/cpp/build"

echo ">> Done. Examples:"
echo "   ${ROOT}/cpp/build/minimal_inference <model.onnx>"
echo "   ${ROOT}/cpp/build/zero_alloc_loop"

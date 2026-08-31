#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NATIVE_CORE_ROOT="${NATIVE_CORE_ROOT:-"$ROOT/../../LumioNativeCore"}"
VOXEL_ROOT="${VOXEL_ROOT:-"$ROOT/../../LumioVoxelEngine"}"
CONFIGURATION="${CONFIGURATION:-debug}"

source_digest() {
  local entries
  entries="$({
    find "$ROOT" "$NATIVE_CORE_ROOT" "$VOXEL_ROOT" -type f \
      -not -path '*/.git/*' -not -path '*/target/*' -not -path '*/bin/*' \
      -not -path '*/obj/*' -not -path '*/.build/*' -not -path '*/.run/*' \
      -print0 | sort -z | while IFS= read -r -d '' file; do
        printf '%s\0%s\n' "$file" "$(sha256sum "$file" | cut -d ' ' -f1)"
      done
  })"
  printf '%s' "$entries" | sha256sum | cut -d ' ' -f1
}

SOURCE_SHA256="$(source_digest)"
BUILD_ID="${SOURCE_SHA256:0:32}"
ABI_HASH="$(sha256sum "$ROOT/engine/abi/native-abi.json" | cut -d ' ' -f1)"
TARGET_DIR="$ROOT/.build/native-target"
export LUMIO_BUILD_ID="$BUILD_ID"
export LUMIO_ABI_HASH="$ABI_HASH"

BUILD_ARGS=(--manifest-path "$ROOT/engine/native/Cargo.toml" -p lumio-engine-native --target-dir "$TARGET_DIR")
if [[ "$CONFIGURATION" == "release" ]]; then
  BUILD_ARGS+=(--release)
fi
cargo build "${BUILD_ARGS[@]}"

NATIVE_PATH="$TARGET_DIR/$CONFIGURATION/liblumio_engine_native.so"
test -f "$NATIVE_PATH"
RUN_ROOT="$ROOT/.run/$BUILD_ID/linux-x64"
mkdir -p "$RUN_ROOT"
STAGED_PATH="$RUN_ROOT/liblumio_engine_native.so"
cp "$NATIVE_PATH" "$STAGED_PATH"
BINARY_SHA256="$(sha256sum "$STAGED_PATH" | cut -d ' ' -f1)"
cat > "$RUN_ROOT/build-info.json" <<EOF
{
  "buildId": "$BUILD_ID",
  "sourceSha256": "$SOURCE_SHA256",
  "abiHash": "$ABI_HASH",
  "binarySha256": "$BINARY_SHA256",
  "platform": "linux-x64",
  "nativePath": "$STAGED_PATH",
  "configuration": "$CONFIGURATION"
}
EOF

printf 'BUILD_ID=%s\nABI_HASH=%s\nNATIVE_PATH=%s\nBINARY_SHA256=%s\n' \
  "$BUILD_ID" "$ABI_HASH" "$STAGED_PATH" "$BINARY_SHA256"

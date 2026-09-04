#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${LumioServerRoot:-}" ]]; then
  echo 'BLOCKED: LumioServerRoot is not set. Point it at the LumioServer repo root so this script can start the rust host (lumio-entity-chat-replay or lumio-server). Do not hard-code a machine path.' >&2
  exit 2
fi
if [[ ! -d "$LumioServerRoot" ]]; then
  echo "BLOCKED: LumioServerRoot path does not exist: $LumioServerRoot" >&2
  exit 2
fi
SERVER_ROOT="$(cd "$LumioServerRoot" && pwd)"
PROCESS_TOML="$SERVER_ROOT/modules/process/Cargo.toml"
if [[ ! -f "$PROCESS_TOML" ]]; then
  echo "BLOCKED: LumioServerRoot is not a LumioServer repo (missing modules/process/Cargo.toml): $SERVER_ROOT" >&2
  exit 2
fi
if ! grep -q 'lumio-entity-chat-replay' "$PROCESS_TOML"; then
  echo 'BLOCKED: LumioServerRoot does not declare bin lumio-entity-chat-replay.' >&2
  exit 2
fi

BUILD_OUTPUT="$("$ROOT/eng/dev-build.sh")"
NATIVE_PATH="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^NATIVE_PATH=//p' | tail -n 1)"
BUILD_ID="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^BUILD_ID=//p' | tail -n 1)"
ABI_HASH="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^ABI_HASH=//p' | tail -n 1)"

find_rust_host() {
  local bin="$1"
  local exe="$bin"
  if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
    exe="${bin}.exe"
  fi
  for profile in debug release; do
    if [[ -f "$SERVER_ROOT/target/$profile/$exe" ]]; then
      printf '%s\n' "$SERVER_ROOT/target/$profile/$exe"
      return 0
    fi
  done
  return 1
}

if ! SERVER_EXE="$(find_rust_host lumio-server)"; then
  cargo build --manifest-path "$SERVER_ROOT/Cargo.toml" --bin lumio-server --bin lumio-entity-chat-replay
  SERVER_EXE="$(find_rust_host lumio-server)" || {
    echo 'BLOCKED: lumio-server binary missing after cargo build.' >&2
    exit 2
  }
fi
REPLAY_EXE="$(find_rust_host lumio-entity-chat-replay || true)"

find_hostfxr() {
  if [[ -n "${LUMIO_HOSTFXR:-}" && -f "$LUMIO_HOSTFXR" ]]; then
    printf '%s\n' "$LUMIO_HOSTFXR"
    return 0
  fi
  local name='libhostfxr.so'
  case "$(uname -s)" in
    Darwin) name='libhostfxr.dylib' ;;
    MINGW*|MSYS*|CYGWIN*) name='hostfxr.dll' ;;
  esac
  local roots=()
  [[ -n "${DOTNET_ROOT:-}" ]] && roots+=("$DOTNET_ROOT")
  if command -v dotnet >/dev/null 2>&1; then
    roots+=("$(dirname "$(command -v dotnet)")")
  fi
  [[ -n "${HOME:-}" ]] && roots+=("$HOME/.dotnet")
  local root fxr ver dll
  for root in "${roots[@]}"; do
    fxr="$root/host/fxr"
    [[ -d "$fxr" ]] || continue
    for ver in "$fxr"/*; do
      dll="$ver/$name"
      if [[ -f "$dll" ]]; then
        printf '%s\n' "$dll"
        return 0
      fi
    done
  done
  echo 'BLOCKED: hostfxr not found. Set LUMIO_HOSTFXR or DOTNET_ROOT.' >&2
  return 1
}

HOSTFXR="$(find_hostfxr)"

RUNTIME_ROOT="${LumioRuntimeRoot:-"$ROOT/../LumioGameRuntime"}"
if [[ ! -d "$RUNTIME_ROOT" ]]; then
  echo "BLOCKED: LumioGameRuntime not found at $RUNTIME_ROOT (set LumioRuntimeRoot). Needed to start the rust host." >&2
  exit 2
fi
HELLO_DIR=''
for cfg in Debug Release; do
  candidate="$RUNTIME_ROOT/modules/hello/entry/bin/$cfg/net10.0"
  if [[ -f "$candidate/Lumio.GameRuntime.HelloEntry.dll" && -f "$candidate/Lumio.GameRuntime.HelloEntry.runtimeconfig.json" ]]; then
    HELLO_DIR="$candidate"
    break
  fi
done
if [[ -z "$HELLO_DIR" ]]; then
  echo 'BLOCKED: Lumio.GameRuntime.HelloEntry.dll / runtimeconfig missing. Build Runtime modules/hello/entry or set LumioRuntimeRoot.' >&2
  exit 2
fi

WIRE_CONTRACT="$ROOT/engine/wire/hello-wire-v1.json"
secret="$(mktemp)"
server_log="$(mktemp)"
client_log="$(mktemp)"
audit_file="$(mktemp)"
ready_file="$(mktemp)"
trap 'jobs -pr | xargs -r kill 2>/dev/null || true; rm -f "$secret" "$server_log" "$client_log" "$audit_file" "$ready_file"' EXIT
printf 'lumio-dev-secret' > "$secret"

printf 'RUST_HOST exe=%s replay=%s (lumio-server is the long-running entry of lumio-server-process; lumio-entity-chat-replay is the batch suite runner in the same crate)\n' "$SERVER_EXE" "${REPLAY_EXE:-missing}"

"$SERVER_EXE" \
  --engine-native "$NATIVE_PATH" \
  --hostfxr "$HOSTFXR" \
  --runtime-config "$HELLO_DIR/Lumio.GameRuntime.HelloEntry.runtimeconfig.json" \
  --assembly "$HELLO_DIR/Lumio.GameRuntime.HelloEntry.dll" \
  --entry-type 'Lumio.GameRuntime.HelloEntry.HelloEntry, Lumio.GameRuntime.HelloEntry' \
  --entry-method 'LumioHelloEntry' \
  --wire-contract "$WIRE_CONTRACT" \
  --audit-file "$audit_file" \
  --ready-file "$ready_file" >"$server_log" 2>&1 &
server_pid=$!
dotnet run --project "$ROOT/../LumioClient/modules/bot/host/Lumio.Client.Bot.Host.csproj" --no-build -- \
  foundation --engine-native "$NATIVE_PATH" >"$client_log" 2>&1 &
client_pid=$!

deadline=$((SECONDS + 30))
while (( SECONDS < deadline )); do
  if grep -q 'SERVER_READY ' "$server_log" && grep -q 'ENGINE_NATIVE ' "$client_log"; then
    server_line="$(grep -m1 'SERVER_READY ' "$server_log")"
    client_line="$(grep -m1 'ENGINE_NATIVE ' "$client_log")"
    printf 'SERVER %s\nCLIENT %s\n' "$server_line" "$client_line"
    grep -q "buildId=$BUILD_ID" <<<"$client_line"
    grep -q "abiHash=$ABI_HASH" <<<"$client_line"
    exit 0
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    echo "BLOCKED: rust host exited." >&2
    cat "$server_log" >&2
    exit 70
  fi
  if ! kill -0 "$client_pid" 2>/dev/null; then
    cat "$client_log" >&2
    exit 71
  fi
  sleep 0.25
done

echo 'Timed out waiting for rust host SERVER_READY and client ENGINE_NATIVE proofs.' >&2
cat "$server_log" "$client_log" >&2
exit 72

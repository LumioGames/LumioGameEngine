#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_OUTPUT="$("$ROOT/eng/dev-build.sh")"
NATIVE_PATH="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^NATIVE_PATH=//p' | tail -n 1)"
BUILD_ID="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^BUILD_ID=//p' | tail -n 1)"
ABI_HASH="$(printf '%s\n' "$BUILD_OUTPUT" | sed -n 's/^ABI_HASH=//p' | tail -n 1)"

secret="$(mktemp)"
server_log="$(mktemp)"
client_log="$(mktemp)"
trap 'jobs -pr | xargs -r kill 2>/dev/null || true; rm -f "$secret" "$server_log" "$client_log"' EXIT
printf 'lumio-dev-secret' > "$secret"

dotnet run --project "$ROOT/../LumioServer/mvp-host/src/Lumio.Server.MvpHost.App/Lumio.Server.MvpHost.App.csproj" --no-build -- \
  --allow-insecure-loopback --shared-secret-file "$secret" --engine-native "$NATIVE_PATH" >"$server_log" 2>&1 &
server_pid=$!
dotnet run --project "$ROOT/../LumioClient/modules/bot/host/Lumio.Client.Bot.Host.csproj" --no-build -- \
  foundation --engine-native "$NATIVE_PATH" >"$client_log" 2>&1 &
client_pid=$!

deadline=$((SECONDS + 30))
while (( SECONDS < deadline )); do
  if grep -q 'ENGINE_NATIVE ' "$server_log" && grep -q 'ENGINE_NATIVE ' "$client_log"; then
    server_line="$(grep -m1 'ENGINE_NATIVE ' "$server_log")"
    client_line="$(grep -m1 'ENGINE_NATIVE ' "$client_log")"
    printf 'SERVER %s\nCLIENT %s\n' "$server_line" "$client_line"
    grep -q "buildId=$BUILD_ID" <<<"$server_line"
    grep -q "buildId=$BUILD_ID" <<<"$client_line"
    grep -q "abiHash=$ABI_HASH" <<<"$server_line"
    grep -q "abiHash=$ABI_HASH" <<<"$client_line"
    exit 0
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 70
  fi
  if ! kill -0 "$client_pid" 2>/dev/null; then
    cat "$client_log" >&2
    exit 71
  fi
  sleep 0.25
done

echo 'Timed out waiting for Server and Client ENGINE_NATIVE proofs.' >&2
cat "$server_log" "$client_log" >&2
exit 72

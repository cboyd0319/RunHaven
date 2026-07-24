#!/bin/bash
# Build a local Apple Development-signed RunHaven app for install and launch QA.
# This workflow never notarizes, creates a DMG, or publishes an artifact.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/macos-signing.sh"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/rust-toolchain.sh"

mode=build
case "${1:-}" in
  "") ;;
  --preflight-only) mode=preflight ;;
  *) printf 'usage: %s [--preflight-only]\n' "$0" >&2; exit 2 ;;
esac

fail() {
  printf 'local signing error: %s\n' "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

need_env() {
  local name=$1
  [ -n "${!name:-}" ] || fail "set $name before building a locally signed app"
}

for command in awk base64 node openssl plutil rustup security sed tr uname; do
  need "$command"
done
[ "$(uname -s)" = "Darwin" ] || fail "local desktop signing requires macOS"
[ "$(uname -m)" = "arm64" ] || fail "local desktop signing requires Apple silicon"
activate_pinned_rust_toolchain "$ROOT" || fail "could not activate the repository-pinned Rust toolchain"

need_env RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY
need_env RUNHAVEN_DEVELOPMENT_PROVISIONING_PROFILE
identity=$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY
profile=$RUNHAVEN_DEVELOPMENT_PROVISIONING_PROFILE
identifier=dev.runhaven.desktop.local
product_name="RunHaven Local"

case "$identity" in
  "Apple Development: "*) ;;
  *) fail "RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY must name an Apple Development identity" ;;
esac
[ -f "$profile" ] || fail "development provisioning profile not found: $profile"

tmp_parent=${TMPDIR:-/tmp}
tmp=$(mktemp -d "${tmp_parent%/}/runhaven-local-sign.XXXXXX")
tmp=$(cd "$tmp" && pwd -P)
trap 'rm -rf "$tmp"' EXIT INT TERM HUP
profile_plist="$tmp/profile.plist"
validate_macos_signing_material "$identity" "$profile" "$identifier" \
  "$profile_plist" 1 || fail "development signing material did not pass validation"

if [ "$mode" = "preflight" ]; then
  printf 'local signing preflight passed for %s\n' "$identifier"
  exit 0
fi

for command in cmp codesign cp lipo npm open rg xcrun; do
  need "$command"
done
[ -x "$ROOT/ui/node_modules/.bin/tauri" ] || fail "install the pinned root npm workspace before local signing"

entitlements="$tmp/RunHaven-Local.entitlements"
ENTITLEMENTS="$entitlements" TEAM_ID="$MACOS_SIGNING_TEAM_ID" \
  APP_ID="$MACOS_SIGNING_EXPECTED_APP_ID" node -e '
  const fs = require("fs");
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>com.apple.application-identifier</key><string>${process.env.APP_ID}</string>
<key>com.apple.developer.team-identifier</key><string>${process.env.TEAM_ID}</string>
<key>com.apple.security.get-task-allow</key><true/>
</dict></plist>\n`;
  fs.writeFileSync(process.env.ENTITLEMENTS, xml, { mode: 0o600 });
'
plutil -lint "$entitlements" >/dev/null

override="$tmp/local-sign-config.json"
CONFIG="$override" IDENTIFIER="$identifier" PRODUCT_NAME="$product_name" node -e '
  const fs = require("fs");
  fs.writeFileSync(process.env.CONFIG, JSON.stringify({
    productName: process.env.PRODUCT_NAME,
    identifier: process.env.IDENTIFIER,
    bundle: { active: true, targets: ["app"] }
  }));
'

target="$ROOT/target/local-signed"
env \
  -u APPLE_API_ISSUER \
  -u APPLE_API_KEY \
  -u APPLE_API_KEY_PATH \
  -u APPLE_ID \
  -u APPLE_PASSWORD \
  -u APPLE_TEAM_ID \
  CARGO_PROFILE_RELEASE_STRIP=none \
  CARGO_TARGET_DIR="$target" \
  npm --prefix "$ROOT" --workspace runhaven-ui run tauri -- build \
    --ci \
    --target aarch64-apple-darwin \
    --bundles app \
    --no-sign \
    --skip-stapling \
    --config "$ROOT/src-tauri/tauri.conf.json" \
    --config "$override" \
    -- --locked

app="$target/aarch64-apple-darwin/release/bundle/macos/$product_name.app"
executable="$app/Contents/MacOS/runhaven-tauri"
[ -d "$app" ] || fail "Tauri did not produce the expected local app bundle"
[ "$(lipo -archs "$executable")" = "arm64" ] || fail "the local app executable is not arm64-only"
minos=$(xcrun vtool -show-build "$executable" | awk '$1 == "minos" { print $2; exit }')
[ "$minos" = "26.0" ] || fail "the local app executable minimum macOS version is $minos, expected 26.0"

cp "$profile" "$app/Contents/embedded.provisionprofile"
codesign --force --sign "$identity" --options runtime --timestamp=none \
  --entitlements "$entitlements" "$app"
codesign --verify --deep --strict --verbose=4 "$app"

signed_details="$tmp/codesign-details.txt"
codesign -dv --verbose=4 "$app" 2> "$signed_details"
rg -F "Authority=$identity" "$signed_details" >/dev/null || fail "the local app has the wrong signing authority"
rg -F "TeamIdentifier=$MACOS_SIGNING_TEAM_ID" "$signed_details" >/dev/null ||
  fail "the local app has the wrong TeamIdentifier"
rg '^Runtime Version=' "$signed_details" >/dev/null || fail "the local app hardened runtime is not enabled"
[ "$(plutil -extract CFBundleIdentifier raw "$app/Contents/Info.plist")" = "$identifier" ] ||
  fail "the local app has the wrong bundle identifier"
cmp -s "$profile" "$app/Contents/embedded.provisionprofile" ||
  fail "the embedded development provisioning profile changed"

signed_entitlements="$tmp/signed-entitlements.plist"
extract_macos_signed_entitlements "$app" "$signed_entitlements" ||
  fail "could not read the local app's signed entitlements"
[ "$(plutil -extract 'com\.apple\.application-identifier' raw "$signed_entitlements")" = \
  "$MACOS_SIGNING_EXPECTED_APP_ID" ] || fail "the local app has the wrong application identifier entitlement"
[ "$(plutil -extract 'com\.apple\.developer\.team-identifier' raw "$signed_entitlements")" = \
  "$MACOS_SIGNING_TEAM_ID" ] || fail "the local app has the wrong team entitlement"
[ "$(plutil -extract 'com\.apple\.security\.get-task-allow' raw "$signed_entitlements")" = "true" ] ||
  fail "the local app is missing the development entitlement"
SIGNED_ENTITLEMENTS="$signed_entitlements" EXPECTED_APP_ID="$MACOS_SIGNING_EXPECTED_APP_ID" \
  TEAM_ID="$MACOS_SIGNING_TEAM_ID" node -e '
  const cp = require("child_process");
  const raw = cp.execFileSync("plutil", ["-convert", "json", "-o", "-", process.env.SIGNED_ENTITLEMENTS]);
  const entitlements = JSON.parse(raw);
  const expectedKeys = [
    "com.apple.application-identifier",
    "com.apple.developer.team-identifier",
    "com.apple.security.get-task-allow"
  ].sort();
  if (JSON.stringify(Object.keys(entitlements).sort()) !== JSON.stringify(expectedKeys)) process.exit(1);
  if (entitlements["com.apple.application-identifier"] !== process.env.EXPECTED_APP_ID) process.exit(1);
  if (entitlements["com.apple.developer.team-identifier"] !== process.env.TEAM_ID) process.exit(1);
  if (entitlements["com.apple.security.get-task-allow"] !== true) process.exit(1);
' || fail "the local app entitlements are not the exact reviewed development set"

open -n "$app"
printf 'Opened %s\n' "$app"
printf 'Use RunHaven to sign in inside each agent container. No Mac Keychain prompt is expected.\n'

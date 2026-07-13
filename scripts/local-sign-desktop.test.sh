#!/bin/bash
# Verify local signing accepts only an isolated Apple Development identity.
# Exercise development profile validation and non-publishing source controls.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
script="$ROOT/scripts/local-sign-desktop.sh"
[ -x "$script" ] || {
  printf 'local signing script is missing or not executable\n' >&2
  exit 1
}
node -e '
  const scripts = require(process.argv[1]).scripts;
  if (scripts["tauri:local-signed"] !== "cd .. && ./scripts/local-sign-desktop.sh") process.exit(1);
' "$ROOT/ui/package.json" || {
  printf 'tauri:local-signed must use the reviewed local signing wrapper\n' >&2
  exit 1
}

source_text=$(cat "$script")
helper_text=$(cat "$ROOT/scripts/lib/macos-signing.sh")
for required in \
  'dev.runhaven.desktop.local' \
  'Apple Development: ' \
  'RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY' \
  'RUNHAVEN_DEVELOPMENT_PROVISIONING_PROFILE' \
  'com.apple.security.get-task-allow' \
  'extract_macos_signed_entitlements' \
  'codesign --verify --deep --strict'; do
  printf '%s' "$source_text" | rg -F "$required" >/dev/null || {
    printf 'local signing script lost required control: %s\n' "$required" >&2
    exit 1
  }
done
printf '%s' "$helper_text" | rg -F 'codesign -d --entitlements - --xml' >/dev/null || {
  printf 'signing helper must use current XML entitlement extraction\n' >&2
  exit 1
}
for forbidden in \
  'dev.runhaven.desktop"' \
  'Developer ID Application:' \
  'notarytool' \
  'stapler' \
  'spctl' \
  'release-artifacts'; do
  if printf '%s' "$source_text" | rg -F "$forbidden" >/dev/null; then
    printf 'local signing script contains release-only behavior: %s\n' "$forbidden" >&2
    exit 1
  fi
done
if printf '%s' "$helper_text" | rg -F 'codesign -d --entitlements :-' >/dev/null; then
  printf 'signing helper uses deprecated entitlement extraction\n' >&2
  exit 1
fi

tmp=$(mktemp -d "${TMPDIR%/}/runhaven-local-sign-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT INT TERM HUP
fake_bin="$tmp/bin"
repo="$tmp/repo"
credentials="$tmp/credentials"
mkdir -p "$fake_bin" "$repo/scripts/lib" "$repo/src-tauri" "$repo/ui" "$credentials"
cp "$script" "$repo/scripts/"
cp "$ROOT/scripts/lib/macos-signing.sh" "$repo/scripts/lib/"
cp "$ROOT/scripts/lib/rust-toolchain.sh" "$repo/scripts/lib/"
printf '[toolchain]\nchannel = "1.97.0"\n' > "$repo/rust-toolchain.toml"
printf '{"version":"0.7.0"}\n' > "$repo/src-tauri/tauri.conf.json"
printf '{"name":"runhaven-ui","version":"0.7.0"}\n' > "$repo/ui/package.json"

openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=RunHaven-Development-Test \
  -keyout "$credentials/test.key" -out "$credentials/test.pem" >/dev/null 2>&1
openssl x509 -in "$credentials/test.pem" -outform DER -out "$credentials/test.der"
fingerprint=$(openssl x509 -in "$credentials/test.pem" -noout -fingerprint -sha1 |
  sed 's/.*=//' | tr -d ':')
certificate=$(base64 < "$credentials/test.der" | tr -d '\n')

write_profile() {
  local path=$1
  local app_id=$2
  local get_task_allow=$3
  local get_task_allow_xml=
  if [ -n "$get_task_allow" ]; then
    get_task_allow_xml='<key>com.apple.security.get-task-allow</key><'"$get_task_allow"'/>'
  fi
  printf '%s\n' '<?xml version="1.0" encoding="UTF-8"?>' \
    '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
    '<plist version="1.0"><dict>' \
    '<key>TeamIdentifier</key><array><string>TEAM123</string></array>' \
    '<key>ApplicationIdentifierPrefix</key><array><string>PREFIX123</string></array>' \
    '<key>ExpirationDate</key><date>2099-01-01T00:00:00Z</date>' \
    '<key>DeveloperCertificates</key><array><data>'"$certificate"'</data></array>' \
    '<key>Entitlements</key><dict>' \
    '<key>com.apple.application-identifier</key><string>'"$app_id"'</string>' \
    '<key>com.apple.developer.team-identifier</key><string>TEAM123</string>' \
    "$get_task_allow_xml" \
    '</dict></dict></plist>' > "$path"
}

profile="$credentials/RunHaven-Development.provisionprofile"
write_profile "$profile" 'PREFIX123.dev.runhaven.desktop.local' true

cat > "$fake_bin/security" <<'SH'
#!/bin/bash
case "${1:-}" in
  find-identity)
    if [ "${TEST_DUPLICATE_IDENTITIES:-0}" = "1" ]; then
      printf '  1) %s "%s"\n  2) %s "%s"\n     2 valid identities found\n' \
        "$TEST_FINGERPRINT" "$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY" \
        "$TEST_FINGERPRINT" "$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY"
    elif [ "${TEST_CONFLICTING_IDENTITY:-0}" = "1" ]; then
      printf '  1) %s "%s"\n  2) %040d "%s"\n     2 valid identities found\n' \
        "$TEST_FINGERPRINT" "$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY" \
        0 "$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY"
    else
      printf '  1) %s "%s"\n     1 valid identities found\n' \
        "$TEST_FINGERPRINT" "$RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY"
    fi
    ;;
  cms)
    cat "${!#}"
    ;;
  *) exit 1 ;;
esac
SH
cat > "$fake_bin/rustup" <<'SH'
#!/bin/bash
if [ "${1:-}" = "which" ] && [ "${2:-}" = "--toolchain" ]; then
  printf '%s/%s\n' "$TEST_FAKE_BIN" "${4:-}"
  exit 0
fi
exit 1
SH
cat > "$fake_bin/cargo" <<'SH'
#!/bin/bash
printf 'cargo 1.97.0 test\n'
SH
cat > "$fake_bin/rustc" <<'SH'
#!/bin/bash
printf 'rustc 1.97.0 test\n'
SH
ln -s "$(command -v node)" "$fake_bin/node"
chmod +x "$fake_bin"/*

identity='Apple Development: RunHaven Test (TEAM123)'
run_preflight() {
  local selected_profile=$1
  local selected_identity=${2:-$identity}
  env PATH="$fake_bin:/usr/bin:/bin" \
    TEST_FAKE_BIN="$fake_bin" \
    TEST_FINGERPRINT="$fingerprint" \
    RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY="$selected_identity" \
    RUNHAVEN_DEVELOPMENT_PROVISIONING_PROFILE="$selected_profile" \
    "$repo/scripts/local-sign-desktop.sh" --preflight-only
}

run_preflight "$profile" | rg -F 'local signing preflight passed' >/dev/null

xcode_profile="$credentials/Xcode-Development.provisionprofile"
write_profile "$xcode_profile" 'PREFIX123.dev.runhaven.desktop.local' ''
run_preflight "$xcode_profile" | rg -F 'local signing preflight passed' >/dev/null

TEST_DUPLICATE_IDENTITIES=1 run_preflight "$profile" |
  rg -F 'local signing preflight passed' >/dev/null

if TEST_CONFLICTING_IDENTITY=1 run_preflight "$profile" > "$tmp/conflicting.log" 2>&1; then
  printf 'two distinct certificates with the same identity unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'must match exactly one installed trusted identity' "$tmp/conflicting.log" >/dev/null

if env -u RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY PATH="$fake_bin:/usr/bin:/bin" \
  "$repo/scripts/local-sign-desktop.sh" --preflight-only > "$tmp/missing.log" 2>&1; then
  printf 'missing development identity unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'set RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY' "$tmp/missing.log" >/dev/null

if run_preflight "$profile" 'Developer ID Application: RunHaven Test (TEAM123)' \
  > "$tmp/release-identity.log" 2>&1; then
  printf 'release identity unexpectedly passed local signing\n' >&2
  exit 1
fi
rg -F 'must name an Apple Development identity' "$tmp/release-identity.log" >/dev/null

production_profile="$credentials/Production.provisionprofile"
write_profile "$production_profile" 'PREFIX123.dev.runhaven.desktop' true
if run_preflight "$production_profile" > "$tmp/production-profile.log" 2>&1; then
  printf 'production bundle profile unexpectedly passed local signing\n' >&2
  exit 1
fi
rg -F 'not for macOS app dev.runhaven.desktop.local' "$tmp/production-profile.log" >/dev/null

unprotected_profile="$credentials/Unprotected.provisionprofile"
write_profile "$unprotected_profile" 'PREFIX123.dev.runhaven.desktop.local' false
if run_preflight "$unprotected_profile" > "$tmp/unprotected.log" 2>&1; then
  printf 'profile without development entitlement unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'does not allow development debugging' "$tmp/unprotected.log" >/dev/null

cat > "$fake_bin/codesign" <<'SH'
#!/bin/bash
attempt=0
if [ -f "$TEST_CODESIGN_ATTEMPTS" ]; then
  attempt=$(cat "$TEST_CODESIGN_ATTEMPTS")
fi
attempt=$((attempt + 1))
printf '%s\n' "$attempt" > "$TEST_CODESIGN_ATTEMPTS"
printf '%s\n' '<?xml version="1.0"?><plist version="1.0"><dict>' \
  '<key>com.apple.application-identifier</key><string>PREFIX123.dev.runhaven.desktop.local</string>' \
  '</dict></plist>'
SH
chmod +x "$fake_bin/codesign"
attempts="$tmp/codesign-attempts"
extracted="$tmp/extracted-entitlements.plist"
PATH="$fake_bin:/usr/bin:/bin" TEST_CODESIGN_ATTEMPTS="$attempts" \
  bash -c '. "$1"; extract_macos_signed_entitlements /fake/RunHaven.app "$2"' \
  _ "$ROOT/scripts/lib/macos-signing.sh" "$extracted"
[ "$(cat "$attempts")" = "1" ] || {
  printf 'entitlement extraction unexpectedly retried a readable signed state\n' >&2
  exit 1
}
plutil -extract 'com\.apple\.application-identifier' raw "$extracted" >/dev/null

printf 'local desktop signing tests passed\n'

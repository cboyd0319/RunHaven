#!/bin/bash
# Verify the desktop release preflight accepts only an exact trusted source state.
# Exercise clean-tag, signing identity, profile, and required-environment failures.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
release_source=$(cat "$ROOT/scripts/release-desktop.sh")
signing_helper=$(cat "$ROOT/scripts/lib/macos-signing.sh")
printf '%s' "$release_source" | rg -F 'extract_macos_signed_entitlements' >/dev/null || {
  printf 'release signing must use the shared entitlement extractor\n' >&2
  exit 1
}
printf '%s' "$signing_helper" | rg -F 'codesign -d --entitlements - --xml' >/dev/null || {
  printf 'release signing must extract current XML entitlements\n' >&2
  exit 1
}
if printf '%s' "$signing_helper" | rg -F 'codesign -d --entitlements :-' >/dev/null; then
  printf 'release signing uses deprecated entitlement extraction\n' >&2
  exit 1
fi
printf '%s' "$release_source" | rg -F 'git worktree add --quiet --detach "$source_root" "$source_commit"' >/dev/null || {
  printf 'release build must use a detached worktree at the tagged source commit\n' >&2
  exit 1
}
printf '%s' "$release_source" | rg -F 'commit=$source_commit' >/dev/null || {
  printf 'release provenance must use the source commit captured before the build\n' >&2
  exit 1
}
if printf '%s' "$release_source" | rg '^commit=\$\(git rev-parse HEAD\)$' >/dev/null; then
  printf 'release provenance reads mutable HEAD after the build\n' >&2
  exit 1
fi
tmp=$(mktemp -d "${TMPDIR:-/tmp}/runhaven-release-preflight.XXXXXX")
trap 'rm -rf "$tmp"' EXIT INT TERM HUP

fake_bin="$tmp/bin"
credentials="$tmp/credentials"
base_repo="$tmp/base-repo"
mkdir -p "$fake_bin" "$credentials" "$base_repo/scripts/lib" "$base_repo/ui" "$base_repo/src-tauri"
cp "$ROOT/scripts/release-desktop.sh" "$base_repo/scripts/"
cp "$ROOT/scripts/lib/release-desktop-common.sh" "$base_repo/scripts/lib/"
cp "$ROOT/scripts/lib/macos-signing.sh" "$base_repo/scripts/lib/"
cp "$ROOT/scripts/lib/rust-toolchain.sh" "$base_repo/scripts/lib/"
chmod +x "$base_repo/scripts/release-desktop.sh"

printf '{"name":"runhaven-ui","version":"0.7.0"}\n' > "$base_repo/ui/package.json"
printf '{"version":"0.7.0"}\n' > "$base_repo/src-tauri/tauri.conf.json"
printf 'lock\n' > "$base_repo/Cargo.lock"
printf '[toolchain]\nchannel = "1.97.0"\n' > "$base_repo/rust-toolchain.toml"

git -C "$base_repo" init -q
git -C "$base_repo" config user.email test@example.com
git -C "$base_repo" config user.name test
git -C "$base_repo" add .
git -C "$base_repo" commit -qm initial
git -C "$base_repo" tag v0.7.0

openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=RunHaven-Test \
  -keyout "$credentials/test.key" -out "$credentials/test.pem" >/dev/null 2>&1
openssl x509 -in "$credentials/test.pem" -outform DER -out "$credentials/test.der"
fingerprint=$(openssl x509 -in "$credentials/test.pem" -noout -fingerprint -sha1 |
  sed 's/.*=//' | tr -d ':')
certificate=$(base64 < "$credentials/test.der" | tr -d '\n')

write_profile() {
  local path=$1
  local app_id=$2
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
    '</dict></dict></plist>' > "$path"
}

profile="$credentials/RunHaven.provisionprofile"
write_profile "$profile" "PREFIX123.dev.runhaven.desktop"

metadata="$credentials/metadata.json"
printf '%s\n' '{"packages":[' \
  '{"name":"runhaven","version":"0.7.0"},' \
  '{"name":"runhaven-cli","version":"0.7.0"},' \
  '{"name":"runhaven-core","version":"0.7.0"},' \
  '{"name":"runhaven-tauri","version":"0.7.0"}' \
  ']}' > "$metadata"

cat > "$fake_bin/cargo" <<'SH'
#!/bin/bash
if [ "${1:-}" = "--version" ]; then
  printf 'cargo 1.97.0 test\n'
  exit 0
fi
if [ "${1:-}" = "metadata" ]; then
  cat "$TEST_METADATA"
  exit 0
fi
if [ "${1:-}" = "audit" ] && [ "${2:-}" = "--version" ]; then exit 0; fi
if [ "${1:-}" = "cyclonedx" ] && [ "${2:-}" = "--version" ]; then exit 0; fi
exit 1
SH
cat > "$fake_bin/rustc" <<'SH'
#!/bin/bash
printf 'rustc 1.97.0 test\n'
SH
cat > "$fake_bin/rustup" <<'SH'
#!/bin/bash
if [ "${1:-}" = "which" ] && [ "${2:-}" = "--toolchain" ]; then
  printf '%s/%s\n' "$TEST_FAKE_BIN" "${4:-}"
  exit 0
fi
exit 1
SH
cat > "$fake_bin/security" <<'SH'
#!/bin/bash
case "${1:-}" in
  find-identity)
    printf '  1) %s "%s"\n     1 valid identities found\n' "$TEST_FINGERPRINT" "$RUNHAVEN_SIGNING_IDENTITY"
    ;;
  cms)
    cat "${!#}"
    ;;
  *) exit 1 ;;
esac
SH
cat > "$fake_bin/uname" <<'SH'
#!/bin/bash
if [ "${1:-}" = "-s" ]; then printf 'Darwin\n'; else printf 'arm64\n'; fi
SH
cat > "$fake_bin/xcode-select" <<'SH'
#!/bin/bash
printf '/Applications/TestXcode.app/Contents/Developer\n'
SH
cat > "$fake_bin/xcrun" <<'SH'
#!/bin/bash
exit 0
SH
chmod +x "$fake_bin"/*

identity='Developer ID Application: RunHaven Test (TEAM123)'

run_preflight() {
  local repo=$1
  local selected_profile=$2
  local selected_metadata=${3:-$metadata}
  env PATH="$fake_bin:$PATH" \
    TEST_FAKE_BIN="$fake_bin" \
    TEST_METADATA="$selected_metadata" \
    TEST_FINGERPRINT="$fingerprint" \
    RUNHAVEN_SIGNING_IDENTITY="$identity" \
    RUNHAVEN_PROVISIONING_PROFILE="$selected_profile" \
    RUNHAVEN_NOTARY_PROFILE=runhaven-test \
    "$repo/scripts/release-desktop.sh" --preflight-only
}

copy_repo() {
  local name=$1
  cp -R "$base_repo" "$tmp/$name"
  printf '%s\n' "$tmp/$name"
}

valid_repo=$(copy_repo valid)
run_preflight "$valid_repo" "$profile" | rg -F 'release preflight passed for v0.7.0' >/dev/null

missing_env_repo=$(copy_repo missing-env)
if env -u RUNHAVEN_SIGNING_IDENTITY PATH="$fake_bin:$PATH" TEST_METADATA="$metadata" \
  "$missing_env_repo/scripts/release-desktop.sh" --preflight-only > "$tmp/missing-env.log" 2>&1; then
  printf 'missing signing identity unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'set RUNHAVEN_SIGNING_IDENTITY' "$tmp/missing-env.log" >/dev/null

dirty_repo=$(copy_repo dirty)
printf 'dirty\n' >> "$dirty_repo/ui/package.json"
if run_preflight "$dirty_repo" "$profile" > "$tmp/dirty.log" 2>&1; then
  printf 'dirty release unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'release worktree must be clean' "$tmp/dirty.log" >/dev/null

wrong_tag_repo=$(copy_repo wrong-tag)
git -C "$wrong_tag_repo" tag -d v0.7.0 >/dev/null
git -C "$wrong_tag_repo" tag v0.6.0
if run_preflight "$wrong_tag_repo" "$profile" > "$tmp/wrong-tag.log" 2>&1; then
  printf 'wrong release tag unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'HEAD must have the exact release tag v0.7.0' "$tmp/wrong-tag.log" >/dev/null

ui_mismatch_repo=$(copy_repo ui-mismatch)
printf '{"name":"runhaven-ui","version":"0.6.0"}\n' > "$ui_mismatch_repo/ui/package.json"
git -C "$ui_mismatch_repo" add ui/package.json
git -C "$ui_mismatch_repo" commit -qm ui-mismatch
git -C "$ui_mismatch_repo" tag -f v0.7.0 >/dev/null
if run_preflight "$ui_mismatch_repo" "$profile" > "$tmp/ui-mismatch.log" 2>&1; then
  printf 'UI version mismatch unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'ui/package.json version 0.6.0 does not match 0.7.0' "$tmp/ui-mismatch.log" >/dev/null

tauri_mismatch_repo=$(copy_repo tauri-mismatch)
printf '{"version":"0.6.0"}\n' > "$tauri_mismatch_repo/src-tauri/tauri.conf.json"
git -C "$tauri_mismatch_repo" add src-tauri/tauri.conf.json
git -C "$tauri_mismatch_repo" commit -qm tauri-mismatch
git -C "$tauri_mismatch_repo" tag -f v0.7.0 >/dev/null
if run_preflight "$tauri_mismatch_repo" "$profile" > "$tmp/tauri-mismatch.log" 2>&1; then
  printf 'Tauri version mismatch unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'src-tauri/tauri.conf.json version 0.6.0 does not match 0.7.0' "$tmp/tauri-mismatch.log" >/dev/null

rust_mismatch="$credentials/metadata-mismatch.json"
sed 's/"runhaven-tauri","version":"0.7.0"/"runhaven-tauri","version":"0.6.0"/' \
  "$metadata" > "$rust_mismatch"
rust_mismatch_repo=$(copy_repo rust-mismatch)
if run_preflight "$rust_mismatch_repo" "$profile" "$rust_mismatch" > "$tmp/rust-mismatch.log" 2>&1; then
  printf 'Rust package version mismatch unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'RunHaven Rust package versions do not match' "$tmp/rust-mismatch.log" >/dev/null

rust_missing="$credentials/metadata-missing.json"
jq 'del(.packages[-1])' "$metadata" > "$rust_missing"
rust_missing_repo=$(copy_repo rust-missing)
if run_preflight "$rust_missing_repo" "$profile" "$rust_missing" > "$tmp/rust-missing.log" 2>&1; then
  printf 'missing Rust release package unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'RunHaven Rust package versions do not match' "$tmp/rust-missing.log" >/dev/null

wrong_profile="$credentials/Wrong.provisionprofile"
write_profile "$wrong_profile" "PREFIX123.example.wrong"
wrong_profile_repo=$(copy_repo wrong-profile)
if run_preflight "$wrong_profile_repo" "$wrong_profile" > "$tmp/wrong-profile.log" 2>&1; then
  printf 'wrong provisioning profile unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'profile is not for macOS app dev.runhaven.desktop' "$tmp/wrong-profile.log" >/dev/null

printf 'release desktop preflight tests passed\n'

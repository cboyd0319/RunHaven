#!/bin/bash

# Build the coworker-facing macOS artifact. This command intentionally fails
# before building unless the source, signing profile, identity, and notarization
# credentials can produce a verifiable Developer ID release.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/release-desktop-common.sh"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/macos-signing.sh"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/rust-toolchain.sh"

mode=release
case "${1:-}" in
  "") ;;
  --preflight-only) mode=preflight ;;
  *) printf 'usage: %s [--preflight-only]\n' "$0" >&2; exit 2 ;;
esac

fail() {
  printf 'release error: %s\n' "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

need_env() {
  local name=$1
  [ -n "${!name:-}" ] || fail "set $name before building a release"
}

for command in awk base64 codesign cmp date ditto find git hdiutil lipo node npm openssl plutil readlink rg rustup security sed shasum sw_vers tr uname xcode-select xcodebuild xcrun; do
  need "$command"
done
activate_pinned_rust_toolchain "$ROOT" || fail "could not activate the repository-pinned Rust toolchain"
cargo audit --version >/dev/null 2>&1 || fail "install cargo-audit before building a release"
cargo cyclonedx --version >/dev/null 2>&1 || fail "install cargo-cyclonedx before building a release"

[ "$(uname -s)" = "Darwin" ] || fail "desktop releases must be built on macOS"
[ "$(uname -m)" = "arm64" ] || fail "desktop releases must be built on Apple silicon"

xcode_path=$(xcode-select -p 2>/dev/null || true)
case "$xcode_path" in
  *.app/Contents/Developer) ;;
  *) fail "select a full Xcode installation with xcode-select; Command Line Tools are not enough" ;;
esac

need_env RUNHAVEN_SIGNING_IDENTITY
need_env RUNHAVEN_PROVISIONING_PROFILE
need_env RUNHAVEN_NOTARY_PROFILE

identity=$RUNHAVEN_SIGNING_IDENTITY
profile=$RUNHAVEN_PROVISIONING_PROFILE
notary_profile=$RUNHAVEN_NOTARY_PROFILE
identifier=dev.runhaven.desktop

case "$identity" in
  "Developer ID Application: "*) ;;
  *) fail "RUNHAVEN_SIGNING_IDENTITY must name a Developer ID Application identity" ;;
esac

[ -f "$profile" ] || fail "provisioning profile not found: $profile"
[ -z "$(git status --porcelain)" ] || fail "the release worktree must be clean"

metadata=$(cargo metadata --locked --no-deps --format-version 1)
version=$(printf '%s' "$metadata" | node -e '
  let input = "";
  process.stdin.on("data", chunk => input += chunk);
  process.stdin.on("end", () => {
    const packages = JSON.parse(input).packages.filter(pkg =>
      ["runhaven", "runhaven-cli", "runhaven-core", "runhaven-tauri"].includes(pkg.name));
    const versions = [...new Set(packages.map(pkg => pkg.version))];
    if (packages.length !== 4 || versions.length !== 1) process.exit(1);
    process.stdout.write(versions[0]);
  });
') || fail "RunHaven Rust package versions do not match"

ui_version=$(node -p 'require("./ui/package.json").version')
tauri_version=$(node -p 'require("./src-tauri/tauri.conf.json").version')
[ "$ui_version" = "$version" ] || fail "ui/package.json version $ui_version does not match $version"
[ "$tauri_version" = "$version" ] || fail "src-tauri/tauri.conf.json version $tauri_version does not match $version"

expected_tag="v$version"
actual_tag=$(git describe --tags --exact-match HEAD 2>/dev/null || true)
[ "$actual_tag" = "$expected_tag" ] || fail "HEAD must have the exact release tag $expected_tag"
source_commit=$(git rev-parse HEAD)
tag_commit=$(git rev-list -n 1 "$expected_tag")
[ "$source_commit" = "$tag_commit" ] || fail "release tag $expected_tag does not resolve to HEAD"

xcrun notarytool history --keychain-profile "$notary_profile" >/dev/null || \
  fail "the notarytool Keychain profile could not authenticate"

tmp_parent=${TMPDIR:-/tmp}
tmp=$(mktemp -d "${tmp_parent%/}/runhaven-release.XXXXXX")
tmp=$(cd "$tmp" && pwd -P)
publish_parent="$ROOT/target/release-artifacts"
publish_final="$publish_parent/v$version"
publish_tmp=""
publish_lock="$publish_parent/.v$version.lock"
publish_lock_acquired=0
source_root="$tmp/source"
source_worktree_added=0
dmg_mount=""
dmg_attached=0
sboms_generated=0
sbom_sources=(
  "$source_root/crates/runhaven/runhaven-rust.cdx.json"
  "$source_root/crates/runhaven-cli/runhaven-rust.cdx.json"
  "$source_root/crates/runhaven-core/runhaven-rust.cdx.json"
  "$source_root/src-tauri/runhaven-rust.cdx.json"
)
for sbom in "${sbom_sources[@]}"; do
  if [ -e "$sbom" ]; then
    rm -rf "$tmp"
    fail "refusing to replace existing file: $sbom"
  fi
done
cleanup() {
  if [ "$dmg_attached" = "1" ]; then
    hdiutil detach -quiet "$dmg_mount" >/dev/null 2>&1 || true
  fi
  if [ "$source_worktree_added" = "1" ]; then
    git -C "$ROOT" worktree remove --force "$source_root" >/dev/null 2>&1 || true
  fi
  rm -rf "$tmp"
  if [ -n "$publish_tmp" ]; then
    rm -rf "$publish_tmp"
  fi
  if [ "$publish_lock_acquired" = "1" ]; then
    rmdir "$publish_lock" >/dev/null 2>&1 || true
  fi
  if [ "$sboms_generated" = "1" ]; then
    rm -f "${sbom_sources[@]}"
  fi
}
trap cleanup EXIT INT TERM HUP

profile_plist="$tmp/profile.plist"
validate_macos_signing_material "$identity" "$profile" "$identifier" "$profile_plist" 0 || \
  fail "release signing material did not pass validation"
team_id=$MACOS_SIGNING_TEAM_ID
app_id_prefix=$MACOS_SIGNING_APP_ID_PREFIX
expected_app_id=$MACOS_SIGNING_EXPECTED_APP_ID
identity_fingerprint=$MACOS_SIGNING_IDENTITY_FINGERPRINT

if [ "$mode" = "preflight" ]; then
  printf 'release preflight passed for %s\n' "$expected_tag"
  exit 0
fi

publish_tmp=$(allocate_release_publish_directory "$publish_parent" "$version") || \
  fail "could not allocate private release staging"
publish_lock_acquired=1
git worktree add --quiet --detach "$source_root" "$source_commit" || \
  fail "could not create the detached release source worktree"
source_worktree_added=1

(
  cd "$source_root"
  ./init.sh
  cargo audit
  npm audit --workspace runhaven-ui --audit-level=high
  scripts/check-docs-local-paths.sh
  scripts/apple_container_smoke.sh
  scripts/gui_tauri_live_smoke.sh --skip-tauri-build
)

entitlements="$tmp/RunHaven.entitlements"
# shellcheck disable=SC2016
ENTITLEMENTS="$entitlements" TEAM_ID="$team_id" APP_ID="$expected_app_id" node -e '
  const fs = require("fs");
  const team = process.env.TEAM_ID;
  const appId = process.env.APP_ID;
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>com.apple.application-identifier</key><string>${appId}</string>
<key>com.apple.developer.team-identifier</key><string>${team}</string>
</dict></plist>\n`;
  fs.writeFileSync(process.env.ENTITLEMENTS, xml, { mode: 0o600 });
'
plutil -lint "$entitlements" >/dev/null

override="$tmp/release-config.json"
CONFIG="$override" IDENTITY="$identity" ENTITLEMENTS="$entitlements" node -e '
  const fs = require("fs");
  fs.writeFileSync(process.env.CONFIG, JSON.stringify({
    bundle: { macOS: {
      signingIdentity: process.env.IDENTITY,
      entitlements: process.env.ENTITLEMENTS,
      hardenedRuntime: true
    }}
  }));
'

[ -z "$(git -C "$source_root" status --porcelain)" ] || \
  fail "the detached release source changed during verification"

env \
  -u APPLE_API_ISSUER \
  -u APPLE_API_KEY \
  -u APPLE_API_KEY_PATH \
  -u APPLE_ID \
  -u APPLE_PASSWORD \
  -u APPLE_TEAM_ID \
  CARGO_PROFILE_RELEASE_STRIP=none \
  CARGO_TARGET_DIR="$tmp/target" \
  npm --prefix "$source_root" --workspace runhaven-ui run tauri -- build \
    --ci \
    --target aarch64-apple-darwin \
    --bundles app \
    --skip-stapling \
    --config "$source_root/src-tauri/tauri.conf.json" \
    --config "$override" \
    -- --locked

app="$tmp/target/aarch64-apple-darwin/release/bundle/macos/RunHaven.app"
[ -d "$app" ] || fail "Tauri did not produce the expected app bundle"
executable="$app/Contents/MacOS/runhaven-tauri"
[ "$(lipo -archs "$executable")" = "arm64" ] || \
  fail "the app executable is not arm64-only"
minos=$(xcrun vtool -show-build "$executable" | awk '$1 == "minos" { print $2; exit }')
[ "$minos" = "26.0" ] || fail "the app executable minimum macOS version is $minos, expected 26.0"

cp "$profile" "$app/Contents/embedded.provisionprofile"
codesign --force --sign "$identity" --options runtime --timestamp \
  --entitlements "$entitlements" "$app"

codesign --verify --deep --strict --verbose=4 "$app"
signed_details="$tmp/codesign-details.txt"
codesign -dv --verbose=4 "$app" 2> "$signed_details"
rg -F "Authority=$identity" "$signed_details" >/dev/null || fail "the app has the wrong signing authority"
rg -F "TeamIdentifier=$team_id" "$signed_details" >/dev/null || fail "the app has the wrong TeamIdentifier"
rg '^Runtime Version=' "$signed_details" >/dev/null || fail "the hardened runtime is not enabled"
rg '^Timestamp=' "$signed_details" >/dev/null || fail "the app signature has no secure timestamp"

signed_entitlements="$tmp/signed-entitlements.plist"
extract_macos_signed_entitlements "$app" "$signed_entitlements" ||
  fail "could not read the app's signed entitlements"
[ "$(plutil -extract 'com\.apple\.application-identifier' raw "$signed_entitlements")" = "$expected_app_id" ] || \
  fail "the signed app has the wrong application identifier entitlement"
[ "$(plutil -extract 'com\.apple\.developer\.team-identifier' raw "$signed_entitlements")" = "$team_id" ] || \
  fail "the signed app has the wrong team entitlement"
SIGNED_ENTITLEMENTS="$signed_entitlements" EXPECTED_APP_ID="$expected_app_id" TEAM_ID="$team_id" node -e '
  const cp = require("child_process");
  const raw = cp.execFileSync("plutil", ["-convert", "json", "-o", "-", process.env.SIGNED_ENTITLEMENTS]);
  const entitlements = JSON.parse(raw);
  const expectedKeys = [
    "com.apple.application-identifier",
    "com.apple.developer.team-identifier"
  ].sort();
  if ("com.apple.security.get-task-allow" in entitlements) process.exit(1);
  if (JSON.stringify(Object.keys(entitlements).sort()) !== JSON.stringify(expectedKeys)) process.exit(1);
  if (entitlements["com.apple.application-identifier"] !== process.env.EXPECTED_APP_ID) process.exit(1);
  if (entitlements["com.apple.developer.team-identifier"] !== process.env.TEAM_ID) process.exit(1);
' || fail "the signed app entitlements are not the exact reviewed release set"
cmp -s "$profile" "$app/Contents/embedded.provisionprofile" || fail "the embedded provisioning profile changed"

notary_upload="$tmp/RunHaven-$version-notary-upload.zip"
ditto -c -k --sequesterRsrc --keepParent "$app" "$notary_upload"
app_receipt="$tmp/app-notarization.json"
xcrun notarytool submit "$notary_upload" --keychain-profile "$notary_profile" \
  --wait --output-format json > "$app_receipt"
RECEIPT="$app_receipt" node -e '
  const receipt = require(process.env.RECEIPT);
  if (receipt.status !== "Accepted") process.exit(1);
' || fail "Apple did not accept the app for notarization"
xcrun stapler staple "$app"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=4 "$app"

app_zip="$tmp/RunHaven-$version-macos-arm64.zip"
ditto -c -k --sequesterRsrc --keepParent "$app" "$app_zip"
zip_check="$tmp/zip-check"
mkdir "$zip_check"
ditto -x -k "$app_zip" "$zip_check"
zipped_app="$zip_check/RunHaven.app"
codesign --verify --deep --strict --verbose=4 "$zipped_app"
xcrun stapler validate "$zipped_app"
spctl --assess --type execute --verbose=4 "$zipped_app"

stage="$tmp/dmg-stage"
mkdir "$stage"
ditto "$app" "$stage/RunHaven.app"
ln -s /Applications "$stage/Applications"
verify_release_dmg_layout "$stage" || fail "the installer disk image layout is invalid"
dmg="$tmp/RunHaven-$version-macos-arm64.dmg"
hdiutil create -quiet -volname "RunHaven $version" -srcfolder "$stage" -ov -format UDZO "$dmg"
codesign --force --sign "$identity" --timestamp "$dmg"
codesign --verify --strict --verbose=4 "$dmg"
dmg_receipt="$tmp/dmg-notarization.json"
xcrun notarytool submit "$dmg" --keychain-profile "$notary_profile" \
  --wait --output-format json > "$dmg_receipt"
RECEIPT="$dmg_receipt" node -e '
  const receipt = require(process.env.RECEIPT);
  if (receipt.status !== "Accepted") process.exit(1);
' || fail "Apple did not accept the disk image for notarization"
xcrun stapler staple "$dmg"
xcrun stapler validate "$dmg"
spctl --assess --type open --context context:primary-signature --verbose=4 "$dmg"

dmg_mount="$tmp/dmg-mount"
mkdir "$dmg_mount"
hdiutil attach -quiet -readonly -nobrowse -mountpoint "$dmg_mount" "$dmg"
dmg_attached=1
verify_release_dmg_layout "$dmg_mount" || fail "the mounted installer layout is invalid"
mounted_app="$dmg_mount/RunHaven.app"
[ "$(lipo -archs "$mounted_app/Contents/MacOS/runhaven-tauri")" = "arm64" ] || \
  fail "the mounted app executable is not arm64-only"
[ "$(plutil -extract CFBundleIdentifier raw "$mounted_app/Contents/Info.plist")" = "$identifier" ] || \
  fail "the mounted app has the wrong bundle identifier"
[ "$(plutil -extract CFBundleShortVersionString raw "$mounted_app/Contents/Info.plist")" = "$version" ] || \
  fail "the mounted app has the wrong version"
[ "$(plutil -extract LSMinimumSystemVersion raw "$mounted_app/Contents/Info.plist")" = "26.0" ] || \
  fail "the mounted app has the wrong minimum macOS version"
install_test="$tmp/Applications"
mkdir "$install_test"
ditto "$mounted_app" "$install_test/RunHaven.app"
codesign --verify --deep --strict --verbose=4 "$install_test/RunHaven.app"
xcrun stapler validate "$install_test/RunHaven.app"
spctl --assess --type execute --verbose=4 "$install_test/RunHaven.app"
hdiutil detach -quiet "$dmg_mount"
dmg_attached=0

cp "$app_zip" "$dmg" "$app_receipt" "$dmg_receipt" "$publish_tmp/"

sboms_generated=1
cargo cyclonedx --quiet --format json --spec-version 1.5 \
  --manifest-path "$source_root/Cargo.toml" \
  --override-filename runhaven-rust.cdx
mv "$source_root/crates/runhaven/runhaven-rust.cdx.json" \
  "$publish_tmp/runhaven-app-rust.cdx.json"
mv "$source_root/crates/runhaven-cli/runhaven-rust.cdx.json" \
  "$publish_tmp/runhaven-cli-rust.cdx.json"
mv "$source_root/crates/runhaven-core/runhaven-rust.cdx.json" \
  "$publish_tmp/runhaven-core-rust.cdx.json"
mv "$source_root/src-tauri/runhaven-rust.cdx.json" \
  "$publish_tmp/runhaven-desktop-rust.cdx.json"
npm --prefix "$source_root/ui" sbom --package-lock-only --sbom-format cyclonedx \
  --sbom-type application > "$publish_tmp/runhaven-ui.cdx.json"
[ -z "$(git -C "$source_root" status --porcelain)" ] || \
  fail "the detached release source changed while artifacts were built"

commit=$source_commit
source_epoch=$(git show -s --format=%ct "$source_commit")
build_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
macos_version=$(sw_vers -productVersion)
rust_version=$(rustc --version)
node_version=$(node --version)
npm_version=$(npm --version)
xcode_version=$(xcodebuild -version | tr '\n' ' ')
cargo_version=$(cargo --version)
tauri_cli_version=$(npm --prefix "$source_root" --workspace runhaven-ui run --silent tauri -- --version)
cargo_lock_sha256=$(shasum -a 256 "$source_root/Cargo.lock" | awk '{print $1}')
package_lock_sha256=$(shasum -a 256 "$source_root/package-lock.json" | awk '{print $1}')
PROVENANCE="$publish_tmp/provenance.json" VERSION="$version" TAG="$expected_tag" \
COMMIT="$commit" TEAM_ID="$team_id" IDENTITY="$identity" SOURCE_EPOCH="$source_epoch" \
BUILD_TIME="$build_time" MACOS_VERSION="$macos_version" RUST_VERSION="$rust_version" \
NODE_VERSION="$node_version" NPM_VERSION="$npm_version" XCODE_VERSION="$xcode_version" \
CARGO_VERSION="$cargo_version" TAURI_CLI_VERSION="$tauri_cli_version" \
CARGO_LOCK_SHA256="$cargo_lock_sha256" PACKAGE_LOCK_SHA256="$package_lock_sha256" node -e '
  const fs = require("fs");
  const data = {
    version: process.env.VERSION,
    tag: process.env.TAG,
    commit: process.env.COMMIT,
    sourceDateEpoch: Number(process.env.SOURCE_EPOCH),
    builtAt: process.env.BUILD_TIME,
    target: "aarch64-apple-darwin",
    minimumMacOS: "26.0",
    identifier: "dev.runhaven.desktop",
    signingIdentity: process.env.IDENTITY,
    teamIdentifier: process.env.TEAM_ID,
    hardenedRuntime: true,
    oauthStorage: "isolated-agent-volume",
    notarized: true,
    stapled: true,
    tooling: {
      macOS: process.env.MACOS_VERSION,
      Xcode: process.env.XCODE_VERSION.trim(),
      Rust: process.env.RUST_VERSION,
      Cargo: process.env.CARGO_VERSION,
      Node: process.env.NODE_VERSION,
      npm: process.env.NPM_VERSION,
      TauriCLI: process.env.TAURI_CLI_VERSION
    },
    lockfiles: {
      cargoLockSHA256: process.env.CARGO_LOCK_SHA256,
      packageLockSHA256: process.env.PACKAGE_LOCK_SHA256
    },
    releaseCommand: "npm run tauri:bundle"
  };
  fs.writeFileSync(process.env.PROVENANCE, JSON.stringify(data, null, 2) + "\n");
'

(
  cd "$publish_tmp"
  shasum -a 256 "RunHaven-$version-macos-arm64.zip" \
    "RunHaven-$version-macos-arm64.dmg" \
    runhaven-*.cdx.json provenance.json > SHA256SUMS
)

[ "$(git -C "$source_root" rev-parse HEAD)" = "$source_commit" ] || \
  fail "the detached release source commit changed before publication"
[ -z "$(git -C "$source_root" status --porcelain)" ] || \
  fail "the detached release source changed before publication"
mv "$publish_tmp" "$publish_final"
publish_tmp=""
rmdir "$publish_lock" || fail "could not release the publication lock"
publish_lock_acquired=0
printf 'verified release artifacts: %s\n' "$publish_final"

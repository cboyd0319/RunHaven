#!/bin/bash
# Validate Apple code-signing identities and macOS provisioning profiles.
# Keep release and isolated local-development checks on one fail-closed path.

macos_signing_error() {
  printf 'signing material error: %s\n' "$*" >&2
  return 1
}

# Extract the signature entitlements as XML and prove the dotted application-ID
# key is readable. `plutil -extract` treats unescaped dots as path separators.
extract_macos_signed_entitlements() {
  local app=$1
  local output=$2
  codesign -d --entitlements - --xml "$app" > "$output" 2>/dev/null ||
    macos_signing_error "could not extract the signed application entitlements" || return 1
  plutil -extract 'com\.apple\.application-identifier' raw "$output" >/dev/null 2>&1 ||
    macos_signing_error "the signed application has no application identifier entitlement"
}

# Validate one installed identity and one provisioning profile for an exact
# bundle identifier. Results are returned in MACOS_SIGNING_* variables.
validate_macos_signing_material() {
  local identity=$1
  local profile=$2
  local identifier=$3
  local profile_plist=$4
  local require_development=$5
  local identity_list identity_matches identity_fingerprints profile_app_id profile_team_id
  local expiration certificate_count certificate_matches profile_fingerprint
  local get_task_allow index

  identity_list=$(security find-identity -v -p codesigning 2>/dev/null || true)
  identity_matches=$(printf '%s\n' "$identity_list" | awk -v wanted="\"$identity\"" \
    'index($0, wanted) { print }')
  identity_fingerprints=$(printf '%s\n' "$identity_matches" |
    awk 'NF && !seen[toupper($2)]++ { print toupper($2) }')
  [ "$(printf '%s\n' "$identity_fingerprints" | awk 'NF { count++ } END { print count + 0 }')" = "1" ] ||
    macos_signing_error "the requested signing identity must match exactly one installed trusted identity" || return 1
  MACOS_SIGNING_IDENTITY_FINGERPRINT=$identity_fingerprints

  security cms -D -i "$profile" > "$profile_plist" ||
    macos_signing_error "could not decode the provisioning profile" || return 1
  MACOS_SIGNING_TEAM_ID=$(plutil -extract TeamIdentifier.0 raw "$profile_plist" 2>/dev/null || true)
  [ -n "$MACOS_SIGNING_TEAM_ID" ] ||
    macos_signing_error "the provisioning profile has no team identifier" || return 1
  MACOS_SIGNING_APP_ID_PREFIX=$(plutil -extract ApplicationIdentifierPrefix.0 raw \
    "$profile_plist" 2>/dev/null || true)
  [ -n "$MACOS_SIGNING_APP_ID_PREFIX" ] ||
    macos_signing_error "the provisioning profile has no application identifier prefix" || return 1
  MACOS_SIGNING_APP_ID_PREFIX=${MACOS_SIGNING_APP_ID_PREFIX%.}
  MACOS_SIGNING_EXPECTED_APP_ID="$MACOS_SIGNING_APP_ID_PREFIX.$identifier"

  profile_app_id=$(plutil -extract 'Entitlements.com\.apple\.application-identifier' raw \
    "$profile_plist" 2>/dev/null || true)
  [ "$profile_app_id" = "$MACOS_SIGNING_EXPECTED_APP_ID" ] ||
    macos_signing_error "the provisioning profile is not for macOS app $identifier and prefix $MACOS_SIGNING_APP_ID_PREFIX" || return 1
  profile_team_id=$(plutil -extract 'Entitlements.com\.apple\.developer\.team-identifier' raw \
    "$profile_plist" 2>/dev/null || true)
  [ "$profile_team_id" = "$MACOS_SIGNING_TEAM_ID" ] ||
    macos_signing_error "the provisioning profile team entitlement does not match" || return 1

  get_task_allow=$(plutil -extract 'Entitlements.com\.apple\.security\.get-task-allow' raw \
    "$profile_plist" 2>/dev/null || true)
  if [ "$require_development" = "1" ]; then
    # macOS permits get-task-allow without profile authorization. Xcode's Mac
    # Team profiles omit it and add it to the development app signature.
    [ -z "$get_task_allow" ] || [ "$get_task_allow" = "true" ] ||
      macos_signing_error "the provisioning profile does not allow development debugging" || return 1
  else
    [ "$get_task_allow" != "true" ] ||
      macos_signing_error "the release provisioning profile allows development debugging" || return 1
  fi

  expiration=$(plutil -extract ExpirationDate raw "$profile_plist" 2>/dev/null || true)
  EXPIRATION="$expiration" node -e '
    const expiry = Date.parse(process.env.EXPIRATION || "");
    if (!Number.isFinite(expiry) || expiry <= Date.now()) process.exit(1);
  ' || macos_signing_error "the provisioning profile is expired or has no valid expiration date" || return 1

  certificate_count=$(plutil -extract DeveloperCertificates raw "$profile_plist" 2>/dev/null || true)
  certificate_matches=0
  if [[ "$certificate_count" =~ ^[0-9]+$ ]]; then
    for ((index = 0; index < certificate_count; index++)); do
      profile_fingerprint=$(plutil -extract "DeveloperCertificates.$index" raw "$profile_plist" |
        base64 -D | openssl x509 -inform DER -noout -fingerprint -sha1 |
        sed 's/.*=//' | tr -d ':')
      profile_fingerprint=$(printf '%s' "$profile_fingerprint" | tr '[:lower:]' '[:upper:]')
      if [ "$profile_fingerprint" = "$MACOS_SIGNING_IDENTITY_FINGERPRINT" ]; then
        certificate_matches=1
      fi
    done
  fi
  [ "$certificate_matches" = "1" ] ||
    macos_signing_error "the provisioning profile does not contain the requested signing certificate" || return 1
}

#!/bin/bash

# Reserve one release version atomically, then allocate staging with an
# unpredictable directory. The deterministic lock directory is intentionally
# left for the caller to remove after publication or cleanup. A crashed release
# therefore fails closed until the operator inspects and removes the stale lock.
allocate_release_publish_directory() {
  local parent=$1
  local version=$2
  local final="$parent/v$version"
  local lock="$parent/.v$version.lock"
  local staging

  mkdir -p "$parent" || return 1
  if [ -e "$final" ]; then
    printf 'release output already exists: %s\n' "$final" >&2
    return 1
  fi
  if ! mkdir "$lock" 2>/dev/null; then
    printf 'release for v%s is already running or needs stale-lock recovery: %s\n' \
      "$version" "$lock" >&2
    return 1
  fi
  if ! staging=$(mktemp -d "$parent/.v$version.tmp.XXXXXX"); then
    rmdir "$lock" >/dev/null 2>&1 || true
    return 1
  fi
  printf '%s\n' "$staging"
}

# A shareable DMG has one obvious action: drag RunHaven onto the Applications
# alias. Reject missing, misleading, or extra top-level content both before the
# image is created and after it is mounted for verification.
verify_release_dmg_layout() {
  local root=$1
  local count

  [ -d "$root/RunHaven.app" ] || {
    printf 'DMG layout is missing RunHaven.app\n' >&2
    return 1
  }
  [ -L "$root/Applications" ] || {
    printf 'DMG layout is missing the Applications alias\n' >&2
    return 1
  }
  [ "$(readlink "$root/Applications")" = "/Applications" ] || {
    printf 'Applications alias must point to /Applications\n' >&2
    return 1
  }
  count=$(find "$root" -mindepth 1 -maxdepth 1 -print | wc -l | tr -d '[:space:]')
  [ "$count" = "2" ] || {
    printf 'DMG layout must contain only RunHaven.app and Applications\n' >&2
    return 1
  }
}

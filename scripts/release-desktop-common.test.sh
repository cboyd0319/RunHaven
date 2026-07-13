#!/bin/bash
# Verify desktop release staging uses collision-safe temporary directories.
# Exercise cleanup and publication without overwriting existing artifacts.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
source "$ROOT/scripts/lib/release-desktop-common.sh"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/runhaven-release-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT INT TERM HUP

parent="$tmp/artifacts"
predictable="$parent/.v0.7.0.tmp.123"
mkdir -p "$predictable"
printf 'preserve\n' > "$predictable/marker"

allocated=$(allocate_release_publish_directory "$parent" "0.7.0")
[ "$allocated" != "$predictable" ]
[ -d "$allocated" ]
[ -d "$parent/.v0.7.0.lock" ]
if allocate_release_publish_directory "$parent" "0.7.0" >/dev/null 2>&1; then
  printf 'concurrent release allocation unexpectedly passed\n' >&2
  exit 1
fi
rm -rf "$allocated"
rmdir "$parent/.v0.7.0.lock"
[ "$(cat "$predictable/marker")" = "preserve" ]

mkdir "$parent/v0.7.0"
printf 'published\n' > "$parent/v0.7.0/marker"
if allocate_release_publish_directory "$parent" "0.7.0" >/dev/null 2>&1; then
  printf 'allocation unexpectedly replaced existing release output\n' >&2
  exit 1
fi
[ "$(cat "$parent/v0.7.0/marker")" = "published" ]
[ ! -e "$parent/.v0.7.0.lock" ]

stage="$tmp/dmg-stage"
mkdir -p "$stage/RunHaven.app"
ln -s /Applications "$stage/Applications"
verify_release_dmg_layout "$stage"

rm "$stage/Applications"
ln -s /tmp "$stage/Applications"
if verify_release_dmg_layout "$stage" >"$tmp/wrong-alias.log" 2>&1; then
  printf 'wrong Applications alias unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'Applications alias must point to /Applications' "$tmp/wrong-alias.log" >/dev/null

rm "$stage/Applications"
ln -s /Applications "$stage/Applications"
touch "$stage/extra.txt"
if verify_release_dmg_layout "$stage" >"$tmp/extra.log" 2>&1; then
  printf 'extra DMG content unexpectedly passed\n' >&2
  exit 1
fi
rg -F 'must contain only RunHaven.app and Applications' "$tmp/extra.log" >/dev/null

printf 'release desktop staging tests passed\n'

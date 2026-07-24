#!/usr/bin/env bash
# Adversarial fixtures for the diagnostic-log secret scanner.
# Proves clean, secret-bearing, and source-path cases against the scanner.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT

clean="$temp/clean.log"
leak="$temp/leak.log"
secret='sk-ant-synthetic-fixture-value'
printf 'ordinary diagnostic line\n' >"$clean"
printf 'ordinary line\n%s\n' "$secret" >"$leak"

"$root/scripts/scan-dev-log.sh" "$clean" >/dev/null

set +e
output=$("$root/scripts/scan-dev-log.sh" "$leak" 2>&1)
status=$?
set -e
if [ "$status" -eq 0 ]; then
  echo "secret scanner fixture failed: synthetic leak returned success" >&2
  exit 1
fi
if [[ "$output" == *"$secret"* ]]; then
  echo "secret scanner fixture failed: output reproduced the synthetic secret" >&2
  exit 1
fi
if [[ "$output" != *"1 matching lines"* ]] || [[ "$output" != *"first lines: 2"* ]]; then
  echo "secret scanner fixture failed: bounded location metadata is missing" >&2
  exit 1
fi

link="$temp/link.log"
ln -s "$clean" "$link"
set +e
output=$("$root/scripts/scan-dev-log.sh" "$link" 2>&1)
status=$?
set -e
if [ "$status" -eq 0 ] || [[ "$output" != *"refused symlinked log"* ]]; then
  echo "secret scanner fixture failed: symlink was not rejected" >&2
  exit 1
fi

unreadable="$temp/unreadable.log"
printf 'ordinary diagnostic line\n' >"$unreadable"
chmod 000 "$unreadable"
set +e
output=$("$root/scripts/scan-dev-log.sh" "$unreadable" 2>&1)
status=$?
set -e
chmod 600 "$unreadable"
if [ "$status" -eq 0 ] || [[ "$output" != *"could not read log"* ]]; then
  echo "secret scanner fixture failed: unreadable log did not fail closed" >&2
  exit 1
fi

mkdir -p "$temp/bin" "$temp/home"
fake_node="$temp/bin/node"
printf '%s\n' '#!/usr/bin/env bash' 'exit 23' >"$fake_node"
chmod 700 "$fake_node"
set +e
output=$(HOME="$temp/home" PATH="$temp/bin:$PATH" "$root/scripts/dev-with-logs.sh" 2>&1)
status=$?
set -e
if [ "$status" -ne 23 ] || [[ "$output" != *"desktop development session failed with exit 23"* ]]; then
  echo "diagnostic-log wrapper fixture failed: application exit was not preserved" >&2
  exit 1
fi

echo "diagnostic-log security fixtures passed"

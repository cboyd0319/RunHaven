#!/usr/bin/env bash
# Verify the macOS GUI review preflight rejects stale or mismatched app processes.
# Exercise its pure checks without building or launching the desktop app.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
script="$root/scripts/review-gui.sh"

# shellcheck disable=SC1090
RUNHAVEN_GUI_REVIEW_SOURCE_ONLY=1 source "$script"

expect_failure() {
  local message="$1"
  shift
  local output
  if output="$("$@" 2>&1)"; then
    echo "expected failure: $message"
    exit 1
  fi
  if [[ "$output" != *"$message"* ]]; then
    echo "missing failure message: $message"
    echo "$output"
    exit 1
  fi
}

assert_no_running_processes ""
expect_failure \
  "Close every running RunHaven app before review" \
  assert_no_running_processes \
  "812|2026-07-13 15:37:27|/tmp/old/RunHaven.app/Contents/MacOS/runhaven-tauri"

verify_launched_process \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug" \
  100 \
  120 \
  130 \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug"

expect_failure \
  "The RunHaven process predates the reviewed binary" \
  verify_launched_process \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug" \
  100 \
  120 \
  119 \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug"

expect_failure \
  "The running executable is not the reviewed bundle" \
  verify_launched_process \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug" \
  100 \
  120 \
  130 \
  "/tmp/old/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug"

expect_failure \
  "The reviewed bundle identifier does not match" \
  verify_launched_process \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop.debug" \
  100 \
  120 \
  130 \
  "/tmp/current/RunHaven Debug.app/Contents/MacOS/runhaven-tauri" \
  "dev.runhaven.desktop"

echo "GUI review provenance tests passed"

#!/usr/bin/env bash
# Run the desktop app in dev with a full, bounded, secret-scanned log trail.
# Tees the whole tauri:dev output to a rotated dev.log, and secret-scans every
# log (dev.log + the structured runhaven.log) when the session ends, so no
# session is ever shared unscanned. Stop the app with Ctrl-C as usual.
set -uo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
teelog="$HOME/Library/Logs/RunHaven/dev.log"
commit="$(git -C "$root" rev-parse --short=8 HEAD 2>/dev/null || printf 'unknown')"
build_id="${RUNHAVEN_BUILD_ID:-${commit}-$(date -u +%Y%m%dT%H%M%SZ)}"
mkdir -p "$(dirname "$teelog")"

# Bound the tee log: keep one rotated backup, cap the live file at ~5 MiB
# (the structured runhaven.log rotates itself via the log plugin).
if [ -f "$teelog" ] && [ "$(wc -c < "$teelog" | tr -d ' ')" -gt $((5 * 1024 * 1024)) ]; then
  mv -f "$teelog" "$teelog.1"
fi

# Secret-scan on exit (including Ctrl-C), so a leak like a wire-dumped token is
# caught automatically instead of only when someone remembers to look.
scan_on_exit() {
  session_status=$?
  trap - EXIT INT TERM
  echo
  echo "===== scanning diagnostic logs for secrets ====="
  "$root/scripts/scan-dev-log.sh"
  scan_status=$?
  if [ "$scan_status" -ne 0 ]; then
    echo "SECRET SCAN FAILED: a local diagnostic log contains token-shaped text."
    echo "Do not share the log; investigate before the next run."
  fi
  if [ "$session_status" -ne 0 ]; then
    printf 'desktop development session failed with exit %s\n' "$session_status"
  fi
  if [ "$scan_status" -ne 0 ]; then
    exit "$scan_status"
  fi
  exit "$session_status"
}
trap scan_on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

{
  echo "===== tauri:dev session $(date -u +%Y-%m-%dT%H:%M:%SZ) ====="
  echo "===== build $build_id from $root/target ====="
  (
    cd "$root" &&
      RUNHAVEN_BUILD_ID="$build_id" CARGO_TARGET_DIR="$root/target" \
        node "$root/ui/scripts/tauri-dev.mjs"
  ) 2>&1
} | tee -a "$teelog"

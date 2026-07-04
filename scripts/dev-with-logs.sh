#!/usr/bin/env bash
# Run the desktop app in dev with a full, bounded, secret-scanned log trail.
# Tees the whole tauri:dev output to a rotated dev.log, and secret-scans every
# log (dev.log + the structured runhaven.log) when the session ends, so no
# session is ever shared unscanned. Stop the app with Ctrl-C as usual.
set -u

root="$(cd "$(dirname "$0")/.." && pwd)"
teelog="$HOME/Library/Logs/RunHaven/dev.log"
mkdir -p "$(dirname "$teelog")"

# Bound the tee log: keep one rotated backup, cap the live file at ~5 MiB
# (the structured runhaven.log rotates itself via the log plugin).
if [ -f "$teelog" ] && [ "$(wc -c < "$teelog" | tr -d ' ')" -gt $((5 * 1024 * 1024)) ]; then
  mv -f "$teelog" "$teelog.1"
fi

# Secret-scan on exit (including Ctrl-C), so a leak like a wire-dumped token is
# caught automatically instead of only when someone remembers to look.
scan_on_exit() {
  echo
  echo "===== scanning diagnostic logs for secrets ====="
  if "$root/scripts/scan-dev-log.sh"; then
    :
  else
    echo "!!! SECRET SCAN FAILED: the logs above contain token-shaped text."
    echo "!!! Do not share them; investigate before the next run."
  fi
}
trap scan_on_exit EXIT INT TERM

{
  echo "===== tauri:dev session $(date -u +%Y-%m-%dT%H:%M:%SZ) ====="
  (
    cd "$root" &&
      CARGO_TARGET_DIR="$root/target" \
        ./ui/node_modules/.bin/tauri dev --config src-tauri/tauri.conf.json
  ) 2>&1
} | tee -a "$teelog"

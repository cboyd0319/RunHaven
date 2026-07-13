#!/usr/bin/env bash
# Secret scan for the desktop diagnostic logs. The diagnostic log must never
# contain tokens, bearer headers, or api keys (docs/DESKTOP_APP.md promise).
# The broker's HTTP client runs in-process, so a too-verbose log level can wire-
# dump the injected Authorization header; this check fails loudly if it does.
# Run after a dev/test session (or in review). Exit non-zero on any hit.
set -u

logs=(
  "$HOME/Library/Logs/dev.runhaven.desktop.debug/runhaven.log"
  "$HOME/Library/Logs/dev.runhaven.desktop/runhaven.log"
  "$HOME/Library/Logs/dev.runhaven.desktop.local/runhaven.log"
  "$HOME/Library/Logs/RunHaven/dev.log"
)
# Rotated files too.
shopt -s nullglob
logs+=("$HOME/Library/Logs/dev.runhaven.desktop.debug/"runhaven_*.log)
logs+=("$HOME/Library/Logs/dev.runhaven.desktop/"runhaven_*.log)
logs+=("$HOME/Library/Logs/dev.runhaven.desktop.local/"runhaven_*.log)
shopt -u nullglob

# Token/secret shapes. sk-ant- is the Claude OAuth/API token prefix; the hex
# "Bearer " (4265 6172 6572) catches wire hexdumps; the rest catch env/header
# leaks. Kept narrow to avoid matching redacted "HEADERS ARE REDACTED" notices.
pattern='sk-ant-|4265 6172 6572|x-api-key: |authorization: bearer|_TOKEN=[A-Za-z0-9]|_API_KEY=[A-Za-z0-9]'

status=0
found_any=0
for log in "${logs[@]}"; do
  [ -f "$log" ] || continue
  hits=$(grep -inE "$pattern" "$log" 2>/dev/null)
  if [ -n "$hits" ]; then
    found_any=1
    status=1
    echo "LEAK in $log:"
    echo "$hits" | head -20
    echo
  fi
done

if [ "$found_any" -eq 0 ]; then
  echo "clean: no token/secret patterns in the diagnostic logs"
fi
exit "$status"

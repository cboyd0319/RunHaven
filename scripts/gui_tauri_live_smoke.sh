#!/usr/bin/env bash
# Local live smoke for the GUI/Tauri managed-session command path.
# Builds the desktop shell and runs the ignored live Tauri command test that
# launches one shell session, writes input, reads output, and stops it.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/gui_tauri_live_smoke.sh [--skip-tauri-build]

Runs an opt-in local smoke for the desktop/Tauri managed-session path.

Checks:
  - runhaven doctor
  - bundled shell image readiness
  - no-bundle Tauri debug build, unless --skip-tauri-build is set
  - one live Tauri-owned shell run on the internal network
  - selected-session Tauri command path for bounded input, bounded output, and
    confirmed stop

This does not automate WebView clicks. It proves the desktop build plus the
same typed Tauri commands the React shell calls for selected-session input,
output, and run control.
EOF
}

fail() {
  echo "error: $*" >&2
  exit 1
}

step() {
  printf '\n== %s ==\n' "$*"
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

print_file() {
  local path="$1"
  if [ -s "$path" ]; then
    sed -n '1,160p' "$path" >&2
  fi
}

SKIP_TAURI_BUILD=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-tauri-build)
      SKIP_TAURI_BUILD=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
  shift
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RUNHAVEN_BIN="${RUNHAVEN_BIN:-$REPO_ROOT/target/debug/runhaven}"

require_command cargo
require_command container
require_command grep
require_command mktemp
require_command npm
require_command sed

case "$(uname -s)" in
  Darwin) ;;
  *) fail "GUI Tauri live smoke requires macOS" ;;
esac

case "$(uname -m)" in
  arm64) ;;
  *) fail "GUI Tauri live smoke requires Apple silicon" ;;
esac

step "build CLI"
cargo build --locked --bin runhaven
[ -x "$RUNHAVEN_BIN" ] || fail "runhaven binary not found at $RUNHAVEN_BIN"

if [ "$SKIP_TAURI_BUILD" -eq 0 ]; then
  step "build Tauri debug shell"
  npm --prefix ui run tauri:build
fi

step "check container runtime and shell image"
"$RUNHAVEN_BIN" doctor
image_doctor_log="$(mktemp "${TMPDIR:-/tmp}/runhaven-gui-image-doctor.XXXXXX")"
if ! "$RUNHAVEN_BIN" image doctor shell >"$image_doctor_log" 2>&1; then
  print_file "$image_doctor_log"
  rm -f "$image_doctor_log"
  fail "bundled shell image is missing or stale; run: $RUNHAVEN_BIN image build shell"
fi
grep -F "Builder status" "$image_doctor_log" >/dev/null 2>&1 || {
  print_file "$image_doctor_log"
  rm -f "$image_doctor_log"
  fail "image doctor did not report builder status"
}
rm -f "$image_doctor_log"

step "exercise live Tauri launch input path"
cargo test -p runhaven-tauri --locked \
  managed_session_live_smoke_writes_input_and_reads_output \
  -- --ignored --show-output

echo "GUI Tauri live smoke checks passed."

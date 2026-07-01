#!/usr/bin/env bash
# Local live smoke for the GUI/Tauri managed-session command path.
# Builds the desktop shell and runs the ignored live Tauri command test that
# launches two shell sessions, writes isolated input, reads isolated output,
# and stops both sessions. Use --with-agent to additionally launch a real
# provider agent through the same Tauri-owned path.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/gui_tauri_live_smoke.sh [--skip-tauri-build] [--with-agent AGENT] [--agent-api-key-env NAME]

Runs an opt-in local smoke for the desktop/Tauri managed-session path.

Checks:
  - runhaven doctor
  - bundled shell image readiness
  - no-bundle Tauri debug build, unless --skip-tauri-build is set
  - two live Tauri-owned shell runs on the internal network
  - selected-session Tauri command path for isolated bounded input, isolated
    bounded output, and confirmed stop
  - optional brokered real-agent Tauri-owned launch with --with-agent

This does not automate WebView clicks. It proves the desktop build plus the
same typed Tauri commands the React shell calls for selected-session input,
output, run control, and multi-session isolation.
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

expected_broker_env() {
  case "$1" in
    codex) printf '%s\n' "OPENAI_API_KEY" ;;
    claude) printf '%s\n' "ANTHROPIC_API_KEY" ;;
    gemini) printf '%s\n' "GEMINI_API_KEY" ;;
    *) return 1 ;;
  esac
}

SKIP_TAURI_BUILD=0
LIVE_AGENT="${RUNHAVEN_GUI_LIVE_AGENT:-}"
LIVE_AGENT_API_KEY_ENV="${RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV:-}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-tauri-build)
      SKIP_TAURI_BUILD=1
      ;;
    --with-agent)
      shift
      [ "$#" -gt 0 ] || fail "--with-agent requires an agent name"
      LIVE_AGENT="$1"
      ;;
    --agent-api-key-env)
      shift
      [ "$#" -gt 0 ] || fail "--agent-api-key-env requires an environment variable name"
      LIVE_AGENT_API_KEY_ENV="$1"
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

if [ -n "$LIVE_AGENT" ]; then
  expected_env="$(expected_broker_env "$LIVE_AGENT")" || {
    fail "--with-agent currently supports brokered agents only: codex, claude, gemini"
  }
  if [ -z "$LIVE_AGENT_API_KEY_ENV" ]; then
    fail "--with-agent $LIVE_AGENT requires --agent-api-key-env $expected_env"
  fi
  [ "$LIVE_AGENT_API_KEY_ENV" = "$expected_env" ] || {
    fail "$LIVE_AGENT broker smoke requires --agent-api-key-env $expected_env"
  }
  step "check real-agent image ($LIVE_AGENT)"
  agent_image_doctor_log="$(mktemp "${TMPDIR:-/tmp}/runhaven-gui-agent-image-doctor.XXXXXX")"
  if ! "$RUNHAVEN_BIN" image doctor "$LIVE_AGENT" >"$agent_image_doctor_log" 2>&1; then
    print_file "$agent_image_doctor_log"
    rm -f "$agent_image_doctor_log"
    fail "bundled $LIVE_AGENT image is missing or stale; run: $RUNHAVEN_BIN image build $LIVE_AGENT"
  fi
  grep -F "Builder status" "$agent_image_doctor_log" >/dev/null 2>&1 || {
    print_file "$agent_image_doctor_log"
    rm -f "$agent_image_doctor_log"
    fail "image doctor did not report builder status for $LIVE_AGENT"
  }
  rm -f "$agent_image_doctor_log"

  if ! [[ "$LIVE_AGENT_API_KEY_ENV" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
    fail "--agent-api-key-env must be a valid environment variable name"
  fi
  if [ -z "${!LIVE_AGENT_API_KEY_ENV:-}" ]; then
    fail "$LIVE_AGENT_API_KEY_ENV is not set; export it before running the real-agent smoke"
  fi
fi

step "exercise live Tauri multi-session input path"
cargo test -p runhaven-tauri --locked \
  managed_session_live_smoke_isolates_two_sessions \
  -- --ignored --show-output

if [ -n "$LIVE_AGENT" ]; then
  step "exercise live Tauri real-agent launch path ($LIVE_AGENT)"
  RUNHAVEN_GUI_LIVE_AGENT="$LIVE_AGENT" \
  RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV="$LIVE_AGENT_API_KEY_ENV" \
    cargo test -p runhaven-tauri --locked \
      managed_session_live_smoke_launches_real_agent_profile \
      -- --ignored --show-output
fi

echo "GUI Tauri live smoke checks passed."

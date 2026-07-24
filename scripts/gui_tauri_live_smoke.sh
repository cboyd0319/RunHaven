#!/usr/bin/env bash
# Local live smoke for the GUI/Tauri managed-session command path.
# Builds the desktop shell and runs the ignored live Tauri command test that
# launches two shell sessions, writes isolated input, reads isolated output,
# and stops both sessions. Set RUNHAVEN_GUI_LIVE_REAL_AGENT=1 to opt into a
# brokered real-agent launch through the same Tauri-owned path.
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
  - optional brokered real-agent Tauri-owned launch when
    RUNHAVEN_GUI_LIVE_REAL_AGENT=1 is set

This does not automate WebView clicks. It proves the desktop build plus the
same typed Tauri commands the React shell calls for selected-session input,
output, run control, and multi-session isolation.

Real-agent broker smoke is skipped by default. To opt in, export:
  RUNHAVEN_GUI_LIVE_REAL_AGENT=1
  RUNHAVEN_GUI_LIVE_AGENT=<codex|claude> or pass --with-agent
  RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV=<fixed broker env> or pass --agent-api-key-env
  The selected broker env itself, for example OPENAI_API_KEY=...
EOF
}

fail() {
  echo "error: $*" >&2
  exit 1
}

step() {
  printf '\n== %s ==\n' "$*"
}

info() {
  printf '%s\n' "$*"
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

print_file() {
  local path="$1"
  if [ -s "$path" ]; then
    printf 'Captured output, last %s lines:\n' "$OUTPUT_LINE_LIMIT" >&2
    tail -n "$OUTPUT_LINE_LIMIT" "$path" | redact_output >&2
  fi
}

new_log() {
  local label="$1"
  local path
  path="$(mktemp "${TMPDIR:-/tmp}/runhaven-gui-${label}.XXXXXX")"
  TEMP_LOGS+=("$path")
  printf '%s\n' "$path"
}

cleanup_logs() {
  local path
  if [ "${#TEMP_LOGS[@]}" -eq 0 ]; then
    return 0
  fi
  for path in "${TEMP_LOGS[@]}"; do
    rm -f "$path"
  done
}

run_logged() {
  local label="$1"
  local log="$2"
  shift 2
  if "$@" >"$log" 2>&1; then
    return 0
  fi
  local status=$?
  printf 'error: %s failed with exit code %s\n' "$label" "$status" >&2
  print_file "$log"
  return "$status"
}

run_bounded() {
  local label="$1"
  local log
  shift
  log="$(new_log command)"
  run_logged "$label" "$log" "$@"
}

expected_broker_env() {
  case "$1" in
    codex) printf '%s\n' "OPENAI_API_KEY" ;;
    claude) printf '%s\n' "ANTHROPIC_API_KEY" ;;
    *) return 1 ;;
  esac
}

env_bool_enabled() {
  local name="$1"
  local value="${!name:-}"
  case "$value" in
    "") return 1 ;;
    1|true|TRUE|yes|YES|on|ON) return 0 ;;
    0|false|FALSE|no|NO|off|OFF) return 1 ;;
    *)
      fail "$name must be unset or one of: 1, true, yes, on, 0, false, no, off"
      ;;
  esac
}

escape_glob_pattern() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\*/\\*}"
  value="${value//\?/\\?}"
  value="${value//\[/\\[}"
  printf '%s\n' "$value"
}

set_broker_secret_redaction() {
  local value="$1"
  case "$value" in
    *$'\n'*|*$'\r'*)
      fail "$LIVE_AGENT_API_KEY_ENV must not contain newline characters"
      ;;
  esac
  BROKER_SECRET_PATTERN="$(escape_glob_pattern "$value")"
}

redact_output() {
  local line
  while IFS= read -r line || [ -n "$line" ]; do
    if [ -n "$BROKER_SECRET_PATTERN" ]; then
      line="${line//$BROKER_SECRET_PATTERN/[redacted broker secret]}"
    fi
    printf '%s\n' "$line"
  done
}

validate_real_agent_config() {
  local expected_env
  local broker_secret
  [ -n "$LIVE_AGENT" ] || {
    fail "RUNHAVEN_GUI_LIVE_REAL_AGENT=1 requires --with-agent <codex|claude> or RUNHAVEN_GUI_LIVE_AGENT"
  }
  expected_env="$(expected_broker_env "$LIVE_AGENT")" || {
    fail "--with-agent currently supports brokered agents only: codex, claude"
  }
  if [ -z "$LIVE_AGENT_API_KEY_ENV" ]; then
    fail "RUNHAVEN_GUI_LIVE_REAL_AGENT=1 with agent $LIVE_AGENT requires --agent-api-key-env $expected_env"
  fi
  [ "$LIVE_AGENT_API_KEY_ENV" = "$expected_env" ] || {
    fail "$LIVE_AGENT broker smoke requires --agent-api-key-env $expected_env"
  }
  if ! [[ "$LIVE_AGENT_API_KEY_ENV" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
    fail "--agent-api-key-env must be a valid environment variable name"
  fi
  broker_secret="${!LIVE_AGENT_API_KEY_ENV:-}"
  if [ -z "$broker_secret" ]; then
    fail "$LIVE_AGENT_API_KEY_ENV is not set; export it before running the real-agent smoke"
  fi
  set_broker_secret_redaction "$broker_secret"
}

OUTPUT_LINE_LIMIT=120
TEMP_LOGS=()
BROKER_SECRET_PATTERN=""
SKIP_TAURI_BUILD=0
LIVE_AGENT="${RUNHAVEN_GUI_LIVE_AGENT:-}"
LIVE_AGENT_API_KEY_ENV="${RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV:-}"
RUN_REAL_AGENT=0
REAL_AGENT_STATUS="skipped"

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
trap cleanup_logs EXIT

if env_bool_enabled RUNHAVEN_GUI_LIVE_REAL_AGENT; then
  RUN_REAL_AGENT=1
  validate_real_agent_config
fi

require_command container
require_command env
require_command grep
require_command mktemp
require_command npm
require_command tail

case "$(uname -s)" in
  Darwin) ;;
  *) fail "GUI Tauri live smoke requires macOS" ;;
esac

case "$(uname -m)" in
  arm64) ;;
  *) fail "GUI Tauri live smoke requires Apple silicon" ;;
esac

step "build CLI"
run_bounded "build CLI" "$REPO_ROOT/scripts/cargo.sh" build --locked --bin runhaven
[ -x "$RUNHAVEN_BIN" ] || fail "built runhaven binary not found; check RUNHAVEN_BIN or target/debug/runhaven"

if [ "$SKIP_TAURI_BUILD" -eq 0 ]; then
  step "build Tauri debug shell"
  run_bounded "build Tauri debug shell" npm run tauri:build
fi

step "check container runtime and shell image"
run_bounded "runhaven doctor" "$RUNHAVEN_BIN" doctor
image_doctor_log="$(mktemp "${TMPDIR:-/tmp}/runhaven-gui-image-doctor.XXXXXX")"
TEMP_LOGS+=("$image_doctor_log")
if ! run_logged "shell image doctor" "$image_doctor_log" "$RUNHAVEN_BIN" image doctor shell; then
  fail "bundled shell image is missing or stale; run: target/debug/runhaven image build shell"
fi
grep -F "Builder status" "$image_doctor_log" >/dev/null 2>&1 || {
  print_file "$image_doctor_log"
  fail "image doctor did not report builder status"
}

if [ "$RUN_REAL_AGENT" -eq 1 ]; then
  step "check real-agent image ($LIVE_AGENT)"
  agent_image_doctor_log="$(mktemp "${TMPDIR:-/tmp}/runhaven-gui-agent-image-doctor.XXXXXX")"
  TEMP_LOGS+=("$agent_image_doctor_log")
  if ! run_logged "$LIVE_AGENT image doctor" "$agent_image_doctor_log" "$RUNHAVEN_BIN" image doctor "$LIVE_AGENT"; then
    fail "bundled $LIVE_AGENT image is missing or stale; run: target/debug/runhaven image build $LIVE_AGENT"
  fi
  grep -F "Builder status" "$agent_image_doctor_log" >/dev/null 2>&1 || {
    print_file "$agent_image_doctor_log"
    fail "image doctor did not report builder status for $LIVE_AGENT"
  }
elif [ -n "$LIVE_AGENT" ] || [ -n "$LIVE_AGENT_API_KEY_ENV" ]; then
  step "skip real-agent broker smoke"
  info "Real-agent smoke is disabled because RUNHAVEN_GUI_LIVE_REAL_AGENT is not set to 1."
  info "Set RUNHAVEN_GUI_LIVE_REAL_AGENT=1 to opt into the live provider launch path."
else
  step "skip real-agent broker smoke"
  info "Real-agent broker smoke stays off by default. Shell-only GUI smoke will still run."
  info "Set RUNHAVEN_GUI_LIVE_REAL_AGENT=1 and choose a brokered agent to opt in."
fi

step "exercise live Tauri multi-session input path"
run_bounded "live Tauri two-session shell smoke" "$REPO_ROOT/scripts/cargo.sh" test -p runhaven-tauri --locked \
  managed_session_live_smoke_isolates_two_sessions \
  -- --ignored --test-threads=1

if [ "$RUN_REAL_AGENT" -eq 1 ]; then
  step "exercise live Tauri real-agent launch path ($LIVE_AGENT)"
  info "Using broker env name $LIVE_AGENT_API_KEY_ENV. Secret values stay host-side."
  run_bounded "live Tauri real-agent smoke" env \
      RUNHAVEN_GUI_LIVE_REAL_AGENT=1 \
      RUNHAVEN_GUI_LIVE_AGENT="$LIVE_AGENT" \
      RUNHAVEN_GUI_LIVE_AGENT_API_KEY_ENV="$LIVE_AGENT_API_KEY_ENV" \
      "$REPO_ROOT/scripts/cargo.sh" test -p runhaven-tauri --locked \
      managed_session_live_smoke_launches_real_agent_profile \
      -- --ignored --test-threads=1
  REAL_AGENT_STATUS="passed for $LIVE_AGENT"
fi

echo "GUI Tauri live smoke checks passed. Real-agent broker smoke: $REAL_AGENT_STATUS."

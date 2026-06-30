#!/usr/bin/env bash
# Local live smoke for the GUI/Tauri managed-session command path.
# Builds the desktop shell, starts one real shell run, exercises the typed
# Tauri session commands against that run, and cleans owned resources.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/gui_tauri_live_smoke.sh [--skip-tauri-build] [--keep-tmp]

Runs an opt-in local smoke for the desktop/Tauri managed-session path.

Checks:
  - runhaven doctor
  - bundled shell image readiness
  - no-bundle Tauri debug build, unless --skip-tauri-build is set
  - one live shell run on the internal network
  - selected-session Tauri command path for bounded output and confirmed stop

This does not automate WebView clicks. It proves the desktop build plus the
same typed Tauri commands the React shell calls for selected-session output and
run control.
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

safe_runhaven_network_name() {
  case "$1" in
    runhaven-*) return 0 ;;
    *) return 1 ;;
  esac
}

container_network_exists() {
  container network inspect "$1" >/dev/null 2>&1
}

delete_network_if_present() {
  local name="$1"
  [ -n "$name" ] || return 0
  safe_runhaven_network_name "$name" || fail "refusing to delete non-RunHaven network: $name"
  if container_network_exists "$name"; then
    container network delete "$name" >/dev/null 2>&1 || true
  fi
}

wait_for_run_id() {
  local stderr_path="$1"
  local run_pid="$2"
  local run_id=""

  for ((i = 0; i < 80; i++)); do
    run_id="$(awk '/^Run id: / {print $3; exit}' "$stderr_path" 2>/dev/null || true)"
    if [ -n "$run_id" ]; then
      printf '%s\n' "$run_id"
      return 0
    fi
    if ! kill -0 "$run_pid" >/dev/null 2>&1; then
      print_file "$stderr_path"
      return 1
    fi
    sleep 0.25
  done

  print_file "$stderr_path"
  return 1
}

wait_for_output() {
  local needle="$1"
  local stdout_path="$2"
  local stderr_path="$3"
  local run_pid="$4"

  for ((i = 0; i < 120; i++)); do
    if grep -F "$needle" "$stdout_path" >/dev/null 2>&1; then
      return 0
    fi
    if ! kill -0 "$run_pid" >/dev/null 2>&1; then
      print_file "$stdout_path"
      print_file "$stderr_path"
      return 1
    fi
    sleep 0.25
  done

  print_file "$stdout_path"
  print_file "$stderr_path"
  return 1
}

parse_network_from_plan() {
  local plan_path="$1"
  local name
  name="$(awk -F': ' '/^Network: / {print $2; exit}' "$plan_path")"
  [ -n "$name" ] || fail "could not parse network name from $plan_path"
  safe_runhaven_network_name "$name" || fail "parsed non-RunHaven network name: $name"
  printf '%s\n' "$name"
}

SKIP_TAURI_BUILD=0
KEEP_TMP=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-tauri-build)
      SKIP_TAURI_BUILD=1
      ;;
    --keep-tmp)
      KEEP_TMP=1
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
SESSION="gui-smoke-$(date +%Y%m%d%H%M%S)-$$"
READY_MARKER="runhaven-gui-tauri-ready"
TMP_ROOT=""
WORKSPACE=""
WORKSPACE_REAL=""
NETWORK=""
RUN_ID=""
RUN_PID=""

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  set +e

  if [ -n "${RUN_ID:-}" ]; then
    "$RUNHAVEN_BIN" runs stop "$RUN_ID" >/dev/null 2>&1 || true
  fi

  if [ -n "${RUN_PID:-}" ]; then
    wait "$RUN_PID" >/dev/null 2>&1 || true
  fi

  if [ -n "${RUN_ID:-}" ]; then
    "$RUNHAVEN_BIN" runs repair "$RUN_ID" >/dev/null 2>&1 || true
  fi

  if [ -n "${WORKSPACE:-}" ] && [ -d "${WORKSPACE:-}" ]; then
    "$RUNHAVEN_BIN" state reset shell --workspace "$WORKSPACE" --session "$SESSION" --yes >/dev/null 2>&1 || true
  fi

  delete_network_if_present "$NETWORK"

  if [ -n "${TMP_ROOT:-}" ] && [ -d "$TMP_ROOT" ] && [ "$KEEP_TMP" -eq 0 ]; then
    rm -rf "$TMP_ROOT"
  elif [ -n "${TMP_ROOT:-}" ] && [ -d "$TMP_ROOT" ]; then
    echo "kept temporary smoke files: $TMP_ROOT" >&2
  fi

  exit "$status"
}

trap cleanup EXIT INT TERM

require_command awk
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

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/runhaven-gui-tauri-smoke.XXXXXX")"
WORKSPACE="$TMP_ROOT/workspace"
mkdir -p "$WORKSPACE"
WORKSPACE_REAL="$(cd "$WORKSPACE" && pwd -P)"
printf 'runhaven gui tauri smoke\n' >"$WORKSPACE/input.txt"

plan_path="$TMP_ROOT/plan.txt"
stdout_path="$TMP_ROOT/stdout.txt"
stderr_path="$TMP_ROOT/stderr.txt"

step "plan internal read-only run"
"$RUNHAVEN_BIN" plan shell \
  --workspace "$WORKSPACE" \
  --session "$SESSION" \
  --network internal \
  --read-only-workspace \
  --no-interactive \
  --tty never \
  -- /bin/bash -lc true >"$plan_path"
grep -F "Workspace: $WORKSPACE_REAL" "$plan_path" >/dev/null || fail "plan did not use smoke workspace"
grep -F "target=/workspace,readonly" "$plan_path" >/dev/null || fail "plan did not set read-only workspace"
NETWORK="$(parse_network_from_plan "$plan_path")"

step "run live shell session"
"$RUNHAVEN_BIN" run shell \
  --workspace "$WORKSPACE" \
  --session "$SESSION" \
  --network internal \
  --read-only-workspace \
  --no-interactive \
  --tty never \
  -- /bin/bash -lc 'set -eu
test "$(pwd)" = "/workspace"
test -f input.txt
echo runhaven-gui-tauri-ready
sleep 45
' >"$stdout_path" 2>"$stderr_path" &
RUN_PID=$!

RUN_ID="$(wait_for_run_id "$stderr_path" "$RUN_PID")" || fail "run did not publish an active run id"
wait_for_output "$READY_MARKER" "$stdout_path" "$stderr_path" "$RUN_PID" || fail "live shell session did not reach ready state"

step "exercise Tauri managed-session commands"
RUNHAVEN_LIVE_MANAGED_SESSION_RUN_ID="$RUN_ID" \
RUNHAVEN_LIVE_MANAGED_SESSION_EXPECT="$READY_MARKER" \
RUNHAVEN_LIVE_MANAGED_SESSION_FORBIDDEN="$WORKSPACE_REAL" \
  cargo test -p runhaven-tauri --locked \
    managed_session_live_smoke_reads_output_and_stops_run \
    -- --ignored --show-output

set +e
wait "$RUN_PID"
run_status=$?
RUN_PID=""
set -e

case "$run_status" in
  0|130|143) ;;
  *) fail "live shell session exited with status $run_status" ;;
esac

if "$RUNHAVEN_BIN" runs active | grep -F "$RUN_ID" >/dev/null; then
  fail "stopped run still appears active: $RUN_ID"
fi

echo "GUI Tauri live smoke checks passed for run $RUN_ID."

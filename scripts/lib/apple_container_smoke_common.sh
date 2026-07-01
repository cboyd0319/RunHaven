#!/usr/bin/env bash
# Shared helpers for scripts/apple_container_smoke.sh.
# Keep this file shell-only so the smoke remains easy to run on macOS.

usage() {
  cat <<'EOF'
Usage: scripts/apple_container_smoke.sh [--with-provider] [--with-ssh] [--skip-provider] [--keep-tmp]

Checks:
  - runhaven doctor
  - bundled shell image readiness
  - internal read-only /workspace run, active status, logs, stop, and record
  - provider plan guidance plus allowlist/direct-egress denial when requested
  - SSH fail-closed guidance plus run refusal when requested

Cleanup is limited to this script's temporary workspace, state, and networks.
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

reset_state_if_possible() {
  local session="$1"
  [ -n "${WORKSPACE:-}" ] && [ -d "${WORKSPACE:-}" ] || return 0
  "$RUNHAVEN_BIN" state reset shell --workspace "$WORKSPACE" --session "$session" --yes >/dev/null 2>&1 || true
}

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  set +e

  if [ -n "${LOGS_PID:-}" ]; then
    kill "$LOGS_PID" >/dev/null 2>&1 || true
    wait "$LOGS_PID" >/dev/null 2>&1 || true
  fi

  if [ -n "${INTERNAL_RUN_ID:-}" ]; then
    "$RUNHAVEN_BIN" runs stop "$INTERNAL_RUN_ID" >/dev/null 2>&1 || true
  fi

  if [ -n "${INTERNAL_RUN_PID:-}" ]; then
    wait "$INTERNAL_RUN_PID" >/dev/null 2>&1 || true
  fi

  if [ -n "${INTERNAL_RUN_ID:-}" ]; then
    "$RUNHAVEN_BIN" runs repair "$INTERNAL_RUN_ID" >/dev/null 2>&1 || true
  fi

  reset_state_if_possible "$SESSION"
  reset_state_if_possible "$PROVIDER_SESSION"
  reset_state_if_possible "$SSH_SESSION"
  delete_network_if_present "$INTERNAL_NETWORK"
  delete_network_if_present "$PROVIDER_NETWORK"

  if [ -n "${TMP_ROOT:-}" ] && [ -d "$TMP_ROOT" ] && [ "$KEEP_TMP" -eq 0 ]; then
    rm -rf "$TMP_ROOT"
  elif [ -n "${TMP_ROOT:-}" ] && [ -d "$TMP_ROOT" ]; then
    echo "kept temporary smoke files: $TMP_ROOT" >&2
  fi

  exit "$status"
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

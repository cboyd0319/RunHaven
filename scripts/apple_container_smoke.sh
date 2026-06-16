#!/usr/bin/env bash
# Local Apple container smoke harness for RunHaven runtime boundaries.
# Builds the CLI, runs opt-in live boundary checks, and cleans owned resources.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/apple_container_smoke.sh [--with-provider] [--with-ssh] [--skip-provider] [--keep-tmp]

Runs an opt-in local Apple container smoke test against the Rust CLI.

Checks:
  - runhaven doctor
  - bundled shell image readiness
  - internal network run with a read-only /workspace mount
  - active run status, logs-follow, stop, and completed run record
  - provider plan guidance
  - provider network allowlist and direct-egress denial, when requested
  - SSH forwarding fail-closed guidance
  - SSH forwarding run refusal, when requested

The script creates a temporary workspace and RunHaven state for a unique
session. Cleanup is limited to those resources and the exact temporary
RunHaven networks parsed from runhaven plan output.
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

RUN_PROVIDER=0
RUN_SSH=0
KEEP_TMP=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --with-provider)
      RUN_PROVIDER=1
      ;;
    --with-ssh)
      RUN_SSH=1
      ;;
    --skip-provider)
      RUN_PROVIDER=0
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

if [ "${RUNHAVEN_WITH_PROVIDER_SMOKE:-0}" = "1" ]; then
  RUN_PROVIDER=1
fi

if [ "${RUNHAVEN_SKIP_PROVIDER_SMOKE:-0}" = "1" ]; then
  RUN_PROVIDER=0
fi

if [ "${RUNHAVEN_WITH_SSH_SMOKE:-0}" = "1" ]; then
  RUN_SSH=1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RUNHAVEN_BIN="${RUNHAVEN_BIN:-$REPO_ROOT/target/debug/runhaven}"
SESSION="smoke-$(date +%Y%m%d%H%M%S)-$$"
TMP_ROOT=""
WORKSPACE=""
WORKSPACE_REAL=""
INTERNAL_NETWORK=""
PROVIDER_NETWORK=""
INTERNAL_RUN_ID=""
INTERNAL_RUN_PID=""
LOGS_PID=""
PROVIDER_SESSION="${SESSION}-provider"
SSH_SESSION="${SESSION}-ssh"

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

trap cleanup EXIT INT TERM

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

require_command cargo
require_command container
require_command mktemp
require_command sed
require_command awk
case "$(uname -s)" in
  Darwin) ;;
  *) fail "Apple container smoke tests require macOS" ;;
esac

case "$(uname -m)" in
  arm64) ;;
  *) fail "Apple container smoke tests require Apple silicon" ;;
esac

step "build CLI"
cargo build --locked
[ -x "$RUNHAVEN_BIN" ] || fail "runhaven binary not found at $RUNHAVEN_BIN"

step "check container runtime and shell image"
"$RUNHAVEN_BIN" doctor
image_doctor_log="$(mktemp "${TMPDIR:-/tmp}/runhaven-image-doctor.XXXXXX")"
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
grep -F "large build resources:" "$image_doctor_log" >/dev/null 2>&1 || {
  print_file "$image_doctor_log"
  rm -f "$image_doctor_log"
  fail "image doctor did not report builder resource guidance"
}
rm -f "$image_doctor_log"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/runhaven-apple-container-smoke.XXXXXX")"
WORKSPACE="$TMP_ROOT/workspace"
mkdir -p "$WORKSPACE"
WORKSPACE_REAL="$(cd "$WORKSPACE" && pwd -P)"
printf 'runhaven apple container smoke\n' >"$WORKSPACE/input.txt"

run_internal_smoke() {
  local plan_path="$TMP_ROOT/internal-plan.txt"
  local stdout_path="$TMP_ROOT/internal-stdout.txt"
  local stderr_path="$TMP_ROOT/internal-stderr.txt"
  local status_path="$TMP_ROOT/internal-status.txt"
  local active_path="$TMP_ROOT/internal-active.txt"
  local logs_path="$TMP_ROOT/internal-logs-follow.txt"
  local logs_err_path="$TMP_ROOT/internal-logs-follow.stderr.txt"
  local show_path="$TMP_ROOT/internal-show.txt"
  local logs_status=0
  local run_status=0

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
  INTERNAL_NETWORK="$(parse_network_from_plan "$plan_path")"

  step "run internal smoke container"
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
if touch should-not-write 2>/tmp/runhaven-write-error; then
  echo "workspace unexpectedly writable" >&2
  exit 42
fi
echo runhaven-internal-ready
sleep 30
' >"$stdout_path" 2>"$stderr_path" &
  INTERNAL_RUN_PID=$!

  INTERNAL_RUN_ID="$(wait_for_run_id "$stderr_path" "$INTERNAL_RUN_PID")" || fail "run did not publish an active run id"
  wait_for_output "runhaven-internal-ready" "$stdout_path" "$stderr_path" "$INTERNAL_RUN_PID" || fail "internal smoke container did not reach ready state"

  step "check active run commands"
  "$RUNHAVEN_BIN" runs active >"$active_path"
  grep -F "$INTERNAL_RUN_ID" "$active_path" >/dev/null || fail "active run list did not include $INTERNAL_RUN_ID"
  "$RUNHAVEN_BIN" runs status "$INTERNAL_RUN_ID" >"$status_path"
  grep -F "$INTERNAL_RUN_ID" "$status_path" >/dev/null || fail "run status did not include $INTERNAL_RUN_ID"

  "$RUNHAVEN_BIN" runs logs-follow "$INTERNAL_RUN_ID" --lines 20 >"$logs_path" 2>"$logs_err_path" &
  LOGS_PID=$!
  wait_for_output "runhaven-internal-ready" "$logs_path" "$logs_err_path" "$LOGS_PID" || fail "logs-follow did not stream the ready marker"

  set +e
  kill "$LOGS_PID" >/dev/null 2>&1
  wait "$LOGS_PID"
  logs_status=$?
  LOGS_PID=""
  set -e

  case "$logs_status" in
    0|130|143) ;;
    *)
      print_file "$logs_path"
      print_file "$logs_err_path"
      fail "logs-follow exited with status $logs_status"
      ;;
  esac

  "$RUNHAVEN_BIN" runs stop "$INTERNAL_RUN_ID" >/dev/null

  set +e
  wait "$INTERNAL_RUN_PID"
  run_status=$?
  INTERNAL_RUN_PID=""
  set -e

  grep -F "runhaven-internal-ready" "$stdout_path" >/dev/null || fail "internal run output missing ready marker"
  "$RUNHAVEN_BIN" runs show "$INTERNAL_RUN_ID" >"$show_path"
  grep -F "$INTERNAL_RUN_ID" "$show_path" >/dev/null || fail "completed run record did not include $INTERNAL_RUN_ID"
  if "$RUNHAVEN_BIN" runs active | grep -F "$INTERNAL_RUN_ID" >/dev/null; then
    fail "stopped run still appears active: $INTERNAL_RUN_ID"
  fi

  printf 'internal run %s stopped after status %s\n' "$INTERNAL_RUN_ID" "$run_status"
}

expect_ssh_disabled_error() {
  local stderr_path="$1"

  grep -F "SSH forwarding is disabled" "$stderr_path" >/dev/null || {
    print_file "$stderr_path"
    fail "SSH command did not explain the disabled forwarding boundary"
  }
  grep -F "Apple container 1.0.0" "$stderr_path" >/dev/null || {
    print_file "$stderr_path"
    fail "SSH command did not name the pinned Apple container blocker"
  }
}

plan_ssh_smoke() {
  local stdout_path="$TMP_ROOT/ssh-plan.stdout.txt"
  local stderr_path="$TMP_ROOT/ssh-plan.stderr.txt"
  local status=0

  step "check SSH forwarding plan fails closed"
  set +e
  "$RUNHAVEN_BIN" plan shell \
    --workspace "$WORKSPACE" \
    --session "$SSH_SESSION" \
    --network internal \
    --read-only-workspace \
    --ssh \
    --no-interactive \
    --tty never \
    -- /bin/bash -lc true >"$stdout_path" 2>"$stderr_path"
  status=$?
  set -e

  [ "$status" -ne 0 ] || {
    print_file "$stdout_path"
    fail "SSH plan unexpectedly succeeded"
  }
  expect_ssh_disabled_error "$stderr_path"
}

run_ssh_smoke() {
  local stdout_path="$TMP_ROOT/ssh-run.stdout.txt"
  local stderr_path="$TMP_ROOT/ssh-run.stderr.txt"
  local status=0

  step "check SSH forwarding run fails closed"
  set +e
  "$RUNHAVEN_BIN" run shell \
    --workspace "$WORKSPACE" \
    --session "$SSH_SESSION" \
    --network internal \
    --read-only-workspace \
    --ssh \
    --no-interactive \
    --tty never \
    -- /bin/bash -lc true >"$stdout_path" 2>"$stderr_path"
  status=$?
  set -e

  [ "$status" -ne 0 ] || {
    print_file "$stdout_path"
    fail "SSH run unexpectedly succeeded"
  }
  expect_ssh_disabled_error "$stderr_path"
}

plan_provider_smoke() {
  local plan_path="$TMP_ROOT/provider-plan.txt"

  step "plan provider allowlist run"
  "$RUNHAVEN_BIN" plan shell \
    --workspace "$WORKSPACE" \
    --session "$PROVIDER_SESSION" \
    --network provider \
    --provider-host example.com \
    --no-interactive \
    --tty never \
    -- /bin/bash -lc true >"$plan_path"
  grep -F "Provider hosts: example.com" "$plan_path" >/dev/null || fail "plan did not include example.com provider host"
  PROVIDER_NETWORK="$(parse_network_from_plan "$plan_path")"
}

run_provider_smoke() {
  local stdout_path="$TMP_ROOT/provider-stdout.txt"
  local stderr_path="$TMP_ROOT/provider-stderr.txt"

  step "run provider allowlist smoke container"
  "$RUNHAVEN_BIN" run shell \
    --workspace "$WORKSPACE" \
    --session "$PROVIDER_SESSION" \
    --network provider \
    --provider-host example.com \
    --no-interactive \
    --tty never \
    -- /bin/bash -lc 'set -eu
curl -fsS --connect-timeout 5 --max-time 15 https://example.com >/dev/null
if curl -fsS --connect-timeout 3 --max-time 6 https://iana.org >/dev/null 2>&1; then
  echo "non-allowlisted provider host unexpectedly succeeded" >&2
  exit 43
fi
if curl -k -fsS --connect-timeout 3 --max-time 6 https://1.1.1.1 >/dev/null 2>&1; then
  echo "proxied IP literal unexpectedly succeeded" >&2
  exit 44
fi
if curl -fsS --noproxy "*" --connect-timeout 3 --max-time 6 https://example.com >/dev/null 2>&1; then
  echo "direct egress unexpectedly succeeded" >&2
  exit 45
fi
if curl -k -fsS --noproxy "*" --connect-timeout 3 --max-time 6 https://1.1.1.1 >/dev/null 2>&1; then
  echo "direct IP egress unexpectedly succeeded" >&2
  exit 46
fi
echo runhaven-provider-ready
' >"$stdout_path" 2>"$stderr_path" || {
    print_file "$stdout_path"
    print_file "$stderr_path"
    fail "provider smoke run failed"
  }

  grep -F "runhaven-provider-ready" "$stdout_path" >/dev/null || fail "provider smoke output missing ready marker"
  if container_network_exists "$PROVIDER_NETWORK"; then
    fail "provider network was not cleaned up: $PROVIDER_NETWORK"
  fi
}

run_internal_smoke
plan_ssh_smoke
if [ "$RUN_SSH" -eq 1 ]; then
  run_ssh_smoke
else
  step "skip SSH run refusal check"
  echo "Use --with-ssh to also verify that runhaven run --ssh fails closed before launch."
fi
plan_provider_smoke
if [ "$RUN_PROVIDER" -eq 1 ]; then
  run_provider_smoke
else
  step "skip live provider smoke"
  echo "Use --with-provider to run live provider allowlist and egress-denial checks."
fi

step "cleanup verification"
reset_state_if_possible "$SESSION"
reset_state_if_possible "$PROVIDER_SESSION"
reset_state_if_possible "$SSH_SESSION"
delete_network_if_present "$INTERNAL_NETWORK"
delete_network_if_present "$PROVIDER_NETWORK"

if "$RUNHAVEN_BIN" runs active | grep -F "${INTERNAL_RUN_ID:-__missing__}" >/dev/null; then
  fail "active run marker still exists after cleanup: $INTERNAL_RUN_ID"
fi

echo "Apple container smoke checks passed."

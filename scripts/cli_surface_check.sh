#!/usr/bin/env bash
# RunHaven CLI surface verification.
#
# Exercises every CLI command family live on macOS 26+ Apple silicon and prints
# PASS/FAIL per surface. This is the breadth check: it confirms that each command
# runs and produces sane output. The deep provider-egress-denial and SSH
# fail-closed boundary is verified separately by scripts/apple_container_smoke.sh.
#
# It creates a temporary git workspace and uniquely named RunHaven sessions, then
# cleans up only the resources it created: its own runs, its own session state
# volumes, and idle RunHaven-managed networks. It never prunes user agent-home
# volumes (state prune is always session-scoped here).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RUNHAVEN_BIN="${RUNHAVEN_BIN:-$REPO_ROOT/target/debug/runhaven}"
SESSION="surface-$(date +%Y%m%d%H%M%S)-$$"
WT_SESSION="${SESSION}-wt"
TMP_ROOT=""
LOG=""
PASS=0
FAIL=0
ACTIVE_RUN_PID=""
ACTIVE_RUN_ID=""

fail_hard() {
  echo "error: $*" >&2
  exit 1
}

step() { printf '\n== %s ==\n' "$*"; }

pass() {
  printf 'PASS  %s\n' "$1"
  PASS=$((PASS + 1))
}

note_fail() {
  printf 'FAIL  %s\n' "$1"
  FAIL=$((FAIL + 1))
  if [ -n "${2:-}" ] && [ -s "$2" ]; then
    sed -n '1,20p' "$2" | sed 's/^/      /' >&2
  fi
}

# ok "desc" cmd...  -> PASS when the command exits 0.
ok() {
  local desc="$1"
  shift
  if "$@" >"$LOG" 2>&1; then pass "$desc"; else note_fail "$desc" "$LOG"; fi
}

# grep_ok "desc" "needle" cmd...  -> PASS when the output contains needle
# (exit code ignored; some surfaces intentionally exit non-zero, e.g. a
# confirmation prompt without --yes).
grep_ok() {
  local desc="$1"
  local needle="$2"
  shift 2
  "$@" >"$LOG" 2>&1 || true
  if grep -qF "$needle" "$LOG"; then pass "$desc"; else note_fail "$desc" "$LOG"; fi
}

safe_runhaven_network_name() {
  case "$1" in
    runhaven-*) return 0 ;;
    *) return 1 ;;
  esac
}

cleanup() {
  local status=$?
  trap - EXIT INT TERM
  set +e
  if [ -n "$ACTIVE_RUN_ID" ]; then
    "$RUNHAVEN_BIN" runs kill "$ACTIVE_RUN_ID" >/dev/null 2>&1
  fi
  if [ -n "$ACTIVE_RUN_PID" ]; then
    wait "$ACTIVE_RUN_PID" >/dev/null 2>&1
  fi
  "$RUNHAVEN_BIN" runs repair --all >/dev/null 2>&1
  if [ -n "$TMP_ROOT" ] && [ -d "$TMP_ROOT" ]; then
    "$RUNHAVEN_BIN" state reset shell --workspace "$TMP_ROOT/work" --session "$SESSION" --yes >/dev/null 2>&1
    "$RUNHAVEN_BIN" state prune --session "$SESSION" --yes >/dev/null 2>&1
    "$RUNHAVEN_BIN" state prune --session "$WT_SESSION" --yes >/dev/null 2>&1
  fi
  # Remove only idle RunHaven-managed networks (never user volumes).
  "$RUNHAVEN_BIN" network prune --yes >/dev/null 2>&1
  if [ -n "$TMP_ROOT" ] && [ -d "$TMP_ROOT" ]; then rm -rf "$TMP_ROOT"; fi
  exit "$status"
}
trap cleanup EXIT INT TERM

wait_for_run_id() {
  local stderr_path="$1"
  local run_pid="$2"
  local run_id=""
  local i
  for ((i = 0; i < 80; i++)); do
    run_id="$(awk '/^Run id: / {print $3; exit}' "$stderr_path" 2>/dev/null || true)"
    if [ -n "$run_id" ]; then
      printf '%s\n' "$run_id"
      return 0
    fi
    kill -0 "$run_pid" >/dev/null 2>&1 || return 1
    sleep 0.25
  done
  return 1
}

wait_for_output() {
  local needle="$1" path="$2" run_pid="$3" i
  for ((i = 0; i < 120; i++)); do
    grep -qF "$needle" "$path" 2>/dev/null && return 0
    kill -0 "$run_pid" >/dev/null 2>&1 || return 1
    sleep 0.25
  done
  return 1
}

command -v container >/dev/null 2>&1 || fail_hard "missing required command: container"
case "$(uname -s)" in Darwin) ;; *) fail_hard "requires macOS" ;; esac
case "$(uname -m)" in arm64) ;; *) fail_hard "requires Apple silicon" ;; esac

step "build CLI"
cargo build --locked
[ -x "$RUNHAVEN_BIN" ] || fail_hard "runhaven binary not found at $RUNHAVEN_BIN"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/runhaven-cli-surface.XXXXXX")"
LOG="$TMP_ROOT/cmd.log"
WORK="$TMP_ROOT/work"
mkdir -p "$WORK"
printf 'runhaven cli surface check\n' >"$WORK/input.txt"

step "informational surfaces"
grep_ok "agents lists bundled profiles" "shell" "$RUNHAVEN_BIN" agents
ok "doctor reports prerequisites" "$RUNHAVEN_BIN" doctor
grep_ok "setup guides first run" "RunHaven setup" "$RUNHAVEN_BIN" setup --agent claude
grep_ok "plan claude defaults to provider" "provider allowlist" \
  "$RUNHAVEN_BIN" plan claude --workspace "$WORK" --no-interactive --tty never
grep_ok "plan shell defaults to internet" "unrestricted internet" \
  "$RUNHAVEN_BIN" plan shell --workspace "$WORK" --no-interactive --tty never

step "why diagnostics"
ok "why host" "$RUNHAVEN_BIN" why host api.anthropic.com --agent claude
ok "why workspace" "$RUNHAVEN_BIN" why workspace "$WORK"
ok "why network" "$RUNHAVEN_BIN" why network provider
ok "why state" "$RUNHAVEN_BIN" why state claude

step "image surfaces"
grep_ok "image doctor reports builder" "Builder status" "$RUNHAVEN_BIN" image doctor shell
grep_ok "image build --dry-run prints command" "container build" \
  "$RUNHAVEN_BIN" image build shell --dry-run
grep_ok "image rebuild --dry-run prints command" "container build" \
  "$RUNHAVEN_BIN" image rebuild shell --dry-run

step "inspection surfaces"
ok "network list" "$RUNHAVEN_BIN" network list
ok "state list" "$RUNHAVEN_BIN" state list
ok "runs list" "$RUNHAVEN_BIN" runs list
ok "runs active" "$RUNHAVEN_BIN" runs active
ok "egress log" "$RUNHAVEN_BIN" egress log --limit 5
ok "auth status" "$RUNHAVEN_BIN" auth status
ok "auth explain" "$RUNHAVEN_BIN" auth explain codex
ok "auth log" "$RUNHAVEN_BIN" auth log --limit 5

step "confirmation gates without --yes"
grep_ok "network prune lists before deleting" "networks" "$RUNHAVEN_BIN" network prune
grep_ok "state reset previews before deleting" "Rerun with --yes" \
  "$RUNHAVEN_BIN" state reset shell --workspace "$WORK" --session "$SESSION"

step "worktree lifecycle"
WT_REPO="$TMP_ROOT/repo"
mkdir -p "$WT_REPO"
git -C "$WT_REPO" init -q
git -C "$WT_REPO" config user.email surface@runhaven.local
git -C "$WT_REPO" config user.name "Surface Check"
printf 'base\n' >"$WT_REPO/seed.txt"
git -C "$WT_REPO" add seed.txt
git -C "$WT_REPO" commit -qm "seed"

wt_stderr="$TMP_ROOT/wt.stderr"
"$RUNHAVEN_BIN" run shell --worktree \
  --workspace "$WT_REPO" --session "$WT_SESSION" \
  --network internal --no-interactive --tty never \
  -- /bin/bash -lc 'echo surface > surface_check_marker.txt' \
  >"$TMP_ROOT/wt.stdout" 2>"$wt_stderr" || true
WT_RUN_ID="$(awk '/^Run id: / {print $3; exit}' "$wt_stderr" 2>/dev/null || true)"
if [ -n "$WT_RUN_ID" ]; then
  pass "run --worktree creates and runs a worktree ($WT_RUN_ID)"
  ok "runs show" "$RUNHAVEN_BIN" runs show "$WT_RUN_ID"
  ok "runs log" "$RUNHAVEN_BIN" runs log "$WT_RUN_ID"
  ok "runs diff" "$RUNHAVEN_BIN" runs diff "$WT_RUN_ID"
  ok "runs keep" "$RUNHAVEN_BIN" runs keep "$WT_RUN_ID"
  ok "runs recover" "$RUNHAVEN_BIN" runs recover "$WT_RUN_ID"
  ok "runs merge" "$RUNHAVEN_BIN" runs merge "$WT_RUN_ID"
else
  note_fail "run --worktree creates and runs a worktree" "$wt_stderr"
fi

# Second worktree run from a fresh clean repo to exercise discard.
# (runs merge above intentionally leaves the first repo dirty for user review.)
WT_REPO2="$TMP_ROOT/repo2"
mkdir -p "$WT_REPO2"
git -C "$WT_REPO2" init -q
git -C "$WT_REPO2" config user.email surface@runhaven.local
git -C "$WT_REPO2" config user.name "Surface Check"
printf 'base\n' >"$WT_REPO2/seed.txt"
git -C "$WT_REPO2" add seed.txt
git -C "$WT_REPO2" commit -qm "seed"

wt2_stderr="$TMP_ROOT/wt2.stderr"
"$RUNHAVEN_BIN" run shell --worktree \
  --workspace "$WT_REPO2" --session "${WT_SESSION}-2" \
  --network internal --no-interactive --tty never \
  -- /bin/bash -lc 'echo discard > discard_marker.txt' \
  >"$TMP_ROOT/wt2.stdout" 2>"$wt2_stderr" || true
WT2_RUN_ID="$(awk '/^Run id: / {print $3; exit}' "$wt2_stderr" 2>/dev/null || true)"
if [ -n "$WT2_RUN_ID" ]; then
  ok "runs discard" "$RUNHAVEN_BIN" runs discard "$WT2_RUN_ID"
else
  note_fail "runs discard (second worktree run)" "$wt2_stderr"
fi
"$RUNHAVEN_BIN" state prune --session "${WT_SESSION}-2" --yes >/dev/null 2>&1 || true

step "active run control"
active_stderr="$TMP_ROOT/active.stderr"
active_stdout="$TMP_ROOT/active.stdout"
"$RUNHAVEN_BIN" run shell \
  --workspace "$WORK" --session "$SESSION" \
  --network internal --no-interactive --tty never \
  -- /bin/bash -lc 'echo runhaven-active-ready; sleep 60' \
  >"$active_stdout" 2>"$active_stderr" &
ACTIVE_RUN_PID=$!
ACTIVE_RUN_ID="$(wait_for_run_id "$active_stderr" "$ACTIVE_RUN_PID" || true)"
if [ -n "$ACTIVE_RUN_ID" ] && wait_for_output "runhaven-active-ready" "$active_stdout" "$ACTIVE_RUN_PID"; then
  pass "run launches an active container ($ACTIVE_RUN_ID)"
  ok "runs status" "$RUNHAVEN_BIN" runs status "$ACTIVE_RUN_ID"
  grep_ok "runs attach runs a command" "/workspace" \
    "$RUNHAVEN_BIN" runs attach "$ACTIVE_RUN_ID" --tty never -- pwd
  ok "runs kill" "$RUNHAVEN_BIN" runs kill "$ACTIVE_RUN_ID"
  wait "$ACTIVE_RUN_PID" >/dev/null 2>&1 || true
  ACTIVE_RUN_PID=""
  ACTIVE_RUN_ID=""
  ok "runs repair --all" "$RUNHAVEN_BIN" runs repair --all
  ok "runs repair --all --json" "$RUNHAVEN_BIN" runs repair --all --json
else
  note_fail "run launches an active container" "$active_stderr"
fi

step "scoped cleanup surfaces"
ok "state list --session" "$RUNHAVEN_BIN" state list --session "$SESSION"
ok "state reset --yes (session-scoped)" \
  "$RUNHAVEN_BIN" state reset shell --workspace "$WORK" --session "$SESSION" --yes
ok "state prune --yes (session-scoped)" \
  "$RUNHAVEN_BIN" state prune --session "$SESSION" --yes

step "summary"
printf '%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
echo "All CLI surfaces confirmed."

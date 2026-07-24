#!/usr/bin/env bash
# Build and launch one provenance-verified Debug app for macOS design review.
# Refuse stale processes, then report exact build, executable, and icon fingerprints.
set -euo pipefail

fail() {
  echo "$1" >&2
  return 1
}

assert_no_running_processes() {
  local records="$1"
  if [[ -z "$records" ]]; then
    return 0
  fi

  echo "Running RunHaven processes:" >&2
  echo "$records" >&2
  fail "Close every running RunHaven app before review. This preflight never reuses or kills an existing process."
}

verify_launched_process() {
  local expected_executable="$1"
  local expected_bundle_id="$2"
  local build_started_epoch="$3"
  local binary_mtime_epoch="$4"
  local process_started_epoch="$5"
  local actual_executable="$6"
  local actual_bundle_id="$7"

  if [[ "$actual_executable" != "$expected_executable" ]]; then
    fail "The running executable is not the reviewed bundle."
    return 1
  fi
  if [[ "$actual_bundle_id" != "$expected_bundle_id" ]]; then
    fail "The reviewed bundle identifier does not match."
    return 1
  fi
  if (( binary_mtime_epoch < build_started_epoch )); then
    fail "The reviewed binary was not created by this review build."
    return 1
  fi
  if (( process_started_epoch < binary_mtime_epoch )); then
    fail "The RunHaven process predates the reviewed binary."
    return 1
  fi
}

process_executable() {
  local pid="$1"
  /usr/sbin/lsof -a -p "$pid" -d txt -Fn 2>/dev/null |
    sed -n 's/^n//p' |
    head -n 1
}

running_process_records() {
  local pid executable started
  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    executable="$(process_executable "$pid")"
    started="$(LC_ALL=C ps -p "$pid" -o lstart= | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    printf '%s|%s|%s\n' "$pid" "$started" "$executable"
  done < <(pgrep -x runhaven-tauri || true)
}

find_exact_process() {
  local expected_executable="$1"
  local pid
  while IFS= read -r pid; do
    [[ -n "$pid" ]] || continue
    if [[ "$(process_executable "$pid")" == "$expected_executable" ]]; then
      printf '%s\n' "$pid"
      return 0
    fi
  done < <(pgrep -x runhaven-tauri || true)
  return 1
}

sha256() {
  shasum -a 256 "$1" | awk '{print $1}'
}

main() {
  local root records commit dirty timestamp build_id build_started_epoch
  local app executable plist bundle_id binary_mtime_epoch pid actual_executable
  local process_started_text process_started_epoch binary_hash icon_hash fingerprint

  root="$(cd "$(dirname "$0")/.." && pwd)"

  if [[ "$(uname -s)" != "Darwin" ]]; then
    fail "GUI review requires macOS."
    return 1
  fi
  for command in cargo git lsof npm open pgrep plutil ps sed shasum stat; do
    command -v "$command" >/dev/null 2>&1 || {
      fail "$command was not found on PATH."
      return 1
    }
  done
  [[ -x "$root/ui/node_modules/.bin/tauri" ]] || {
    fail "Install the pinned UI dependencies before GUI review."
    return 1
  }

  records="$(running_process_records)"
  assert_no_running_processes "$records"

  commit="$(git -C "$root" rev-parse --short=8 HEAD)"
  dirty=""
  if [[ -n "$(git -C "$root" status --porcelain)" ]]; then
    dirty="-dirty"
  fi
  timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
  build_id="${commit}${dirty}-${timestamp}"
  build_started_epoch="$(date +%s)"

  (
    cd "$root"
    RUNHAVEN_BUILD_ID="$build_id" \
      CARGO_TARGET_DIR="$root/target" \
      npm --prefix "$root" --workspace runhaven-ui run tauri -- build \
      --config src-tauri/tauri.conf.json \
      --config src-tauri/tauri.dev.conf.json \
      --debug \
      --bundles app
  )

  app="$root/target/debug/bundle/macos/RunHaven Debug.app"
  executable="$app/Contents/MacOS/runhaven-tauri"
  plist="$app/Contents/Info.plist"
  [[ -x "$executable" && -f "$plist" ]] || {
    fail "The canonical RunHaven Debug bundle was not created."
    return 1
  }

  bundle_id="$(plutil -extract CFBundleIdentifier raw "$plist")"
  binary_mtime_epoch="$(stat -f %m "$executable")"
  if (( binary_mtime_epoch < build_started_epoch )); then
    fail "The reviewed binary was not created by this review build."
    return 1
  fi

  open "$app"
  pid=""
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    pid="$(find_exact_process "$executable" || true)"
    [[ -n "$pid" ]] && break
    sleep 1
  done
  [[ -n "$pid" ]] || {
    records="$(running_process_records)"
    [[ -n "$records" ]] && echo "$records" >&2
    fail "The canonical RunHaven Debug process did not start."
    return 1
  }

  actual_executable="$(process_executable "$pid")"
  process_started_text="$(LC_ALL=C ps -p "$pid" -o lstart= | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  process_started_epoch="$(LC_ALL=C date -j -f '%a %b %e %T %Y' "$process_started_text" +%s)"
  verify_launched_process \
    "$executable" \
    "dev.runhaven.desktop.debug" \
    "$build_started_epoch" \
    "$binary_mtime_epoch" \
    "$process_started_epoch" \
    "$actual_executable" \
    "$bundle_id"

  binary_hash="$(sha256 "$executable")"
  icon_hash="$(sha256 "$app/Contents/Resources/icon.icns")"
  fingerprint="$(printf '%s\n%s\n%s\n%s\n' \
    "$bundle_id" "$build_id" "$binary_hash" "$icon_hash" | shasum -a 256 | awk '{print $1}')"

  echo "RunHaven GUI review target is ready."
  echo "app=$app"
  echo "bundle_id=$bundle_id"
  echo "pid=$pid"
  echo "process_started=$process_started_text"
  echo "build_id=$build_id"
  echo "executable_sha256=$binary_hash"
  echo "icon_sha256=$icon_hash"
  echo "review_fingerprint_sha256=$fingerprint"
}

if [[ "${RUNHAVEN_GUI_REVIEW_SOURCE_ONLY:-0}" != "1" ]]; then
  main "$@"
fi

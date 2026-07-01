#!/usr/bin/env bash
# Fails when tracked files or workspace crates drift outside RunHaven's
# reviewability and ownership boundaries.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

KIB=1024
DEFAULT_BYTES=$((1024 * KIB))
CUSTOM_SOURCE_LINES=500
GENERATED_SOURCE_LINES=800
SCRIPT_LINES=500
DOC_LINES=800
STARTUP_LINES=500
AGENT_SKILL_LINES=250
HARNESS_DOC_LINES=500
PLAN_DOC_LINES=1000
JSON_LINES=600
LOCK_LINES=20000
VENDORED_SOURCE_LINES=12000
VENDORED_JSON_LINES=25000
THIRD_PARTY_LINES=5000
ASSET_JSON_LINES=1000
BINARY_ASSET_BYTES=$((2048 * KIB))
SOURCE_IMAGE_BYTES=$((6144 * KIB))
LOCK_BYTES=$((1024 * KIB))
PDF_BYTES=$((1024 * KIB))
SPLIT_HEADROOM_PERCENT=90

violations=""

append_violation() {
  violations="${violations}$1
"
}

is_binary_asset() {
  case "$1" in
    *.png|*.gif|*.webp|*.zip|*.z3|*.pdf)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

requires_split_headroom() {
  case "$1" in
    AGENTS.md|current-state.md|feature_list.json)
      return 0
      ;;
    Cargo.lock|ui/package-lock.json)
      return 1
      ;;
    crates/codex/**|third_party/**|docs/assets/**|ui/src/components/ui/**)
      return 1
      ;;
    crates/runhaven-tui/src/tui/runhaven/**)
      return 0
      ;;
    crates/runhaven-tui/src/tui/**)
      return 1
      ;;
    *.snap|*.ansi|*.record|*.errors|*.chart|*.cmp|*.zil)
      return 1
      ;;
    *)
      if is_binary_asset "$1"; then
        return 1
      fi
      return 0
      ;;
  esac
}

line_limit_for() {
  case "$1" in
    AGENTS.md)
      echo 250
      ;;
    current-state.md)
      echo "$STARTUP_LINES"
      ;;
    feature_list.json)
      echo 600
      ;;
    Cargo.lock|ui/package-lock.json)
      echo "$LOCK_LINES"
      ;;
    crates/codex/**/*.json|crates/codex/*.json)
      echo "$VENDORED_JSON_LINES"
      ;;
    crates/codex/**/*.rs|crates/codex/**/*.ts|crates/codex/**/*.tsx|crates/codex/**/*.py|crates/codex/**/*.md|crates/codex/**/*.toml|crates/codex/**/*.bazel|crates/codex/**/*.proto|crates/codex/**/*.sbpl)
      echo "$VENDORED_SOURCE_LINES"
      ;;
    crates/runhaven-tui/src/tui/runhaven/*.rs|crates/runhaven-tui/src/tui/runhaven/**/*.rs|crates/runhaven-tui/src/lib.rs)
      echo "$CUSTOM_SOURCE_LINES"
      ;;
    crates/runhaven-tui/src/tui/*.rs|crates/runhaven-tui/src/tui/**/*.rs|crates/runhaven-tui/src/tui/*.md|crates/runhaven-tui/src/tui/**/*.md|crates/runhaven-tui/src/tui/*.txt|crates/runhaven-tui/src/tui/**/*.txt)
      echo "$VENDORED_SOURCE_LINES"
      ;;
    third_party/**)
      echo "$THIRD_PARTY_LINES"
      ;;
    docs/assets/skill/**)
      echo 1000
      ;;
    docs/assets/**/*.json|docs/assets/*.json)
      echo "$ASSET_JSON_LINES"
      ;;
    docs/plans/**/*.md|docs/plans/*.md)
      echo "$PLAN_DOC_LINES"
      ;;
    docs/harness/**/*.md|docs/harness/**/*.json|docs/harness/**/*.yaml|docs/harness/**/*.yml|docs/harness/**/*.tsv)
      echo "$HARNESS_DOC_LINES"
      ;;
    .agents/**/*.md|.agents/**/*.json|.agents/**/*.txt)
      echo "$AGENT_SKILL_LINES"
      ;;
    scripts/*.sh|init.sh|init.ps1)
      echo "$SCRIPT_LINES"
      ;;
    ui/src/components/ui/*.ts|ui/src/components/ui/**/*.ts|ui/src/components/ui/*.tsx|ui/src/components/ui/**/*.tsx)
      echo "$GENERATED_SOURCE_LINES"
      ;;
    *.rs|*.ts|*.tsx|*.svelte|*.css|*.py|*.sh)
      echo "$CUSTOM_SOURCE_LINES"
      ;;
    *.md)
      echo "$DOC_LINES"
      ;;
    *.json)
      echo "$JSON_LINES"
      ;;
    *.toml|*.yaml|*.yml|*.gitattributes|*.gitignore|*.conf|*.codexpolicy|*.sources|*.manifest|*.txt)
      echo 250
      ;;
    *.snap|*.ansi|*.record|*.errors|*.chart|*.cmp|*.zil|*.html)
      echo "$THIRD_PARTY_LINES"
      ;;
    *)
      if is_binary_asset "$1"; then
        echo 0
      else
        echo "$CUSTOM_SOURCE_LINES"
      fi
      ;;
  esac
}

byte_limit_for() {
  case "$1" in
    Cargo.lock|ui/package-lock.json)
      echo "$LOCK_BYTES"
      ;;
    docs/assets/source-images/**)
      echo "$SOURCE_IMAGE_BYTES"
      ;;
    docs/assets/**/*.png|docs/assets/**/*.gif|docs/assets/**/*.webp|docs/assets/*.png|docs/assets/*.gif|docs/assets/*.webp)
      echo "$BINARY_ASSET_BYTES"
      ;;
    *.pdf|*.pdf\")
      echo "$PDF_BYTES"
      ;;
    crates/codex/**)
      echo "$DEFAULT_BYTES"
      ;;
    *)
      echo "$DEFAULT_BYTES"
      ;;
  esac
}

check_file_limits() {
  local file="$1"
  local byte_limit bytes line_limit lines headroom_limit

  byte_limit="$(byte_limit_for "$file")"
  bytes="$(wc -c < "$file" | tr -d ' ')"
  if [ "$bytes" -gt "$byte_limit" ]; then
    append_violation "file too large: ${bytes} bytes > ${byte_limit} bytes: ${file}"
  fi

  line_limit="$(line_limit_for "$file")"
  if [ "$line_limit" -eq 0 ]; then
    return
  fi
  if ! grep -Iq . "$file"; then
    return
  fi
  lines="$(wc -l < "$file" | tr -d ' ')"
  if [ "$lines" -gt "$line_limit" ]; then
    append_violation "too many lines: ${lines} > ${line_limit}: ${file}"
    return
  fi
  if requires_split_headroom "$file"; then
    headroom_limit=$((line_limit * SPLIT_HEADROOM_PERCENT / 100))
    if [ "$line_limit" -gt 100 ] && [ "$lines" -ge "$headroom_limit" ]; then
      append_violation "near line ceiling: ${lines} >= ${headroom_limit} (${SPLIT_HEADROOM_PERCENT}% of ${line_limit}); split before growth: ${file}"
    fi
  fi
}

dependency_names() {
  awk '
    /^\[(dependencies|dev-dependencies|build-dependencies)\]/ {
      in_deps = 1;
      next;
    }
    /^\[target\..*\.dependencies\]/ {
      in_deps = 1;
      next;
    }
    /^\[/ {
      in_deps = 0;
      next;
    }
    in_deps && /^[[:space:]]*[A-Za-z0-9_-]+(\.[A-Za-z0-9_-]+)?[[:space:]]*(=|\.)/ {
      name = $1;
      sub(/\..*/, "", name);
      sub(/=.*/, "", name);
      print name;
    }
  ' "$1"
}

check_runhaven_edges() {
  local manifest="$1"
  local owner="$2"
  shift 2
  local allowed=" $* "
  local dep

  for dep in $(dependency_names "$manifest" | grep -E '^runhaven-' || true); do
    case "$allowed" in
      *" ${dep} "*) ;;
      *)
        append_violation "${owner} -> ${dep} is not an allowed RunHaven dependency edge"
        ;;
    esac
  done
}

check_crate_dependencies() {
  check_runhaven_edges crates/runhaven/Cargo.toml runhaven \
    runhaven-cli runhaven-core runhaven-tui
  check_runhaven_edges crates/runhaven-core/Cargo.toml runhaven-core
  check_runhaven_edges crates/runhaven-cli/Cargo.toml runhaven-cli \
    runhaven-core
  check_runhaven_edges crates/runhaven-tui/Cargo.toml runhaven-tui \
    runhaven-core
  check_runhaven_edges src-tauri/Cargo.toml runhaven-tauri \
    runhaven-core

  for manifest in \
    crates/runhaven/Cargo.toml \
    crates/runhaven-core/Cargo.toml \
    crates/runhaven-cli/Cargo.toml \
    src-tauri/Cargo.toml
  do
    if codex_deps="$(dependency_names "$manifest" | grep -E '^codex-' || true)" \
      && [ -n "$codex_deps" ]; then
      append_violation "${manifest} must not depend on Codex crates; keep Codex-vendored dependencies in runhaven-tui"
    fi
  done

  for manifest in \
    crates/runhaven/Cargo.toml \
    crates/runhaven-core/Cargo.toml \
    crates/runhaven-cli/Cargo.toml \
    crates/runhaven-tui/Cargo.toml
  do
    if tauri_deps="$(dependency_names "$manifest" | grep -E '^tauri' || true)" \
      && [ -n "$tauri_deps" ]; then
      append_violation "${manifest} must not depend on Tauri crates; keep the desktop bridge in src-tauri"
    fi
  done
}

check_workspace_members() {
  local manifest dir

  while IFS= read -r manifest; do
    dir="${manifest%/Cargo.toml}"
    if [ "$dir" = "." ]; then
      continue
    fi
    if ! grep -Fq "\"${dir}\"" Cargo.toml; then
      append_violation "${manifest} is not listed in the root workspace members"
    fi
  done < <(git ls-files -- 'crates/**/Cargo.toml' 'src-tauri/Cargo.toml')
}

check_workspace_shape() {
  if git ls-files -- 'src/**' | grep -q .; then
    append_violation "root src/ must stay absent; crates/runhaven owns binary entrypoints"
  fi
  if git ls-files -- 'src-tauri/Cargo.lock' | grep -q .; then
    append_violation "src-tauri/Cargo.lock must stay absent; root Cargo.lock owns the workspace"
  fi
  if [ -f crates/runhaven/src/lib.rs ]; then
    append_violation "crates/runhaven must stay binary-only; do not add crates/runhaven/src/lib.rs"
  fi
  extra_root_sources="$(
    find crates/runhaven/src -type f -name '*.rs' \
      ! -path 'crates/runhaven/src/main.rs' \
      ! -path 'crates/runhaven/src/bin/*.rs' \
      -print
  )"
  if [ -n "$extra_root_sources" ]; then
    append_violation "crates/runhaven may contain main.rs and src/bin/*.rs only"
  fi
  for path in \
    crates/runhaven-cli/src/runtime \
    crates/runhaven-tui/src/runtime \
    src-tauri/src/runtime \
    ui/src/runtime
  do
    if [ -e "$path" ]; then
      append_violation "${path} must not exist; shared runtime truth belongs in runhaven-core"
    fi
  done
  if [ -f src-tauri/src/commands/tests.rs ]; then
    append_violation "src-tauri command tests must stay split under src-tauri/src/commands/tests/"
  fi
  tauri_container_calls="$(
    git grep -n 'Command::new("container")' -- src-tauri/src \
      ':!src-tauri/src/commands/tests/live.rs' || true
  )"
  if [ -n "$tauri_container_calls" ]; then
    append_violation "src-tauri must call runtime behavior through runhaven-core, not direct container commands"
  fi
}

while IFS= read -r -d '' file; do
  if [ ! -f "$file" ]; then
    continue
  fi
  check_file_limits "$file"
done < <(git ls-files -z --cached --others --exclude-standard)

check_workspace_members
check_crate_dependencies
check_workspace_shape

if [ -n "$violations" ]; then
  cat >&2 <<'EOF'
repo structure guard failed

RunHaven keeps static file-size ceilings for every tracked file category and
mechanically enforces crate/workspace ownership. Splittable repo-owned text
files also need growth headroom: split by ownership before a file gets close to
the ceiling. Raise a limit only when the file is generated, vendored, or a
binary artifact with a recorded reason.

Violations:
EOF
  printf '%s' "$violations" >&2
  exit 1
fi

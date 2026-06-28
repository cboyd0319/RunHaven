#!/usr/bin/env bash
# Compare RunHaven's vendored TUI source with the pinned upstream Codex TUI.
#
# The default source is the upstream GitHub repository at the commit that was
# used for the current vendor baseline. Override CODEX_REPO_URL, CODEX_COMMIT,
# CODEX_TUI_PATH, or RUNHAVEN_TUI_PATH only when auditing a different source.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

CODEX_REPO_URL="${CODEX_REPO_URL:-https://github.com/openai/codex.git}"
CODEX_COMMIT="${CODEX_COMMIT:-5267e805fb830891c0b23376bcd9cbd382c3473c}"
CODEX_TUI_PATH="${CODEX_TUI_PATH:-codex-rs/tui/src}"
RUNHAVEN_TUI_PATH="${RUNHAVEN_TUI_PATH:-crates/runhaven-tui/src/tui}"

LIST_MISSING=0
WRITE_MANIFESTS_DIR=""
while [ "$#" -gt 0 ]; do
  arg="$1"
  shift
  case "$arg" in
    --list-missing) LIST_MISSING=1 ;;
    --write-manifests)
      if [ "$#" -eq 0 ]; then
        echo "--write-manifests requires an output directory" >&2
        exit 2
      fi
      WRITE_MANIFESTS_DIR="$1"
      shift
      ;;
    --write-manifests=*)
      WRITE_MANIFESTS_DIR="${arg#--write-manifests=}"
      ;;
    -h|--help)
      cat <<'USAGE'
Usage: scripts/compare-codex-tui.sh [--list-missing] [--write-manifests DIR]

Compares all files under the pinned upstream Codex TUI source path with
RunHaven's vendored TUI path. The default report prints counts, RunHaven-only
files, and copied Codex files with local edits.

The comparison uses deterministic file manifests with:

  relative path, byte size, sha256

Use --list-missing to also print the full list of upstream files not currently
vendored by RunHaven. Use --write-manifests DIR to save the generated manifests
and comparison lists for audit.

Environment overrides:
  CODEX_REPO_URL       default: https://github.com/openai/codex.git
  CODEX_COMMIT         default: 5267e805fb830891c0b23376bcd9cbd382c3473c
  CODEX_TUI_PATH       default: codex-rs/tui/src
  RUNHAVEN_TUI_PATH    default: crates/runhaven-tui/src/tui
USAGE
      exit 0
      ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [ ! -d "$RUNHAVEN_TUI_PATH" ]; then
  echo "RunHaven TUI path not found: $RUNHAVEN_TUI_PATH" >&2
  exit 2
fi

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/runhaven-codex-tui.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

UPSTREAM_REPO="$TMP_ROOT/codex"
UPSTREAM_TREE="$TMP_ROOT/upstream-tree"
mkdir -p "$UPSTREAM_TREE"

git clone --quiet --filter=blob:none --sparse "$CODEX_REPO_URL" "$UPSTREAM_REPO"
git -C "$UPSTREAM_REPO" sparse-checkout set "$CODEX_TUI_PATH" >/dev/null
git -C "$UPSTREAM_REPO" rev-parse --verify "$CODEX_COMMIT^{commit}" >/dev/null
git -C "$UPSTREAM_REPO" archive "$CODEX_COMMIT" "$CODEX_TUI_PATH" | tar -xf - -C "$UPSTREAM_TREE"

UPSTREAM_TUI="$UPSTREAM_TREE/$CODEX_TUI_PATH"
if [ ! -d "$UPSTREAM_TUI" ]; then
  echo "upstream TUI path not found in commit: $CODEX_TUI_PATH" >&2
  exit 2
fi

file_size() {
  wc -c < "$1" | tr -d ' '
}

file_sha256() {
  shasum -a 256 "$1" | awk '{print $1}'
}

write_manifest() {
  local root="$1"
  local manifest="$2"
  (
    cd "$root"
    find . -type f ! -name '.DS_Store' | LC_ALL=C sort | while IFS= read -r file; do
      printf '%s\t%s\t%s\n' "$file" "$(file_size "$file")" "$(file_sha256 "$file")"
    done
  ) > "$manifest"
}

UPSTREAM_MANIFEST="$TMP_ROOT/upstream-tui.files.tsv"
RUNHAVEN_MANIFEST="$TMP_ROOT/runhaven-tui.files.tsv"
UPSTREAM_LIST="$TMP_ROOT/upstream-paths.txt"
RUNHAVEN_LIST="$TMP_ROOT/runhaven-paths.txt"
MISSING_LIST="$TMP_ROOT/missing-files.txt"
LOCAL_ONLY_LIST="$TMP_ROOT/local-only-files.txt"
COMMON_LIST="$TMP_ROOT/common-files.txt"
CHANGED_LIST="$TMP_ROOT/changed-files.txt"

write_manifest "$UPSTREAM_TUI" "$UPSTREAM_MANIFEST"
write_manifest "$RUNHAVEN_TUI_PATH" "$RUNHAVEN_MANIFEST"
cut -f1 "$UPSTREAM_MANIFEST" > "$UPSTREAM_LIST"
cut -f1 "$RUNHAVEN_MANIFEST" > "$RUNHAVEN_LIST"

comm -23 "$UPSTREAM_LIST" "$RUNHAVEN_LIST" > "$MISSING_LIST"
comm -13 "$UPSTREAM_LIST" "$RUNHAVEN_LIST" > "$LOCAL_ONLY_LIST"
comm -12 "$UPSTREAM_LIST" "$RUNHAVEN_LIST" > "$COMMON_LIST"

TAB=$'\t'
LC_ALL=C join -t "$TAB" -1 1 -2 1 "$UPSTREAM_MANIFEST" "$RUNHAVEN_MANIFEST" \
  | awk -F '\t' '$2 != $4 || $3 != $5 { print $1 }' > "$CHANGED_LIST"

if [ -n "$WRITE_MANIFESTS_DIR" ]; then
  mkdir -p "$WRITE_MANIFESTS_DIR"
  cp "$UPSTREAM_MANIFEST" "$WRITE_MANIFESTS_DIR/upstream-tui.files.tsv"
  cp "$RUNHAVEN_MANIFEST" "$WRITE_MANIFESTS_DIR/runhaven-tui.files.tsv"
  cp "$MISSING_LIST" "$WRITE_MANIFESTS_DIR/missing-files.txt"
  cp "$LOCAL_ONLY_LIST" "$WRITE_MANIFESTS_DIR/runhaven-only-files.txt"
  cp "$COMMON_LIST" "$WRITE_MANIFESTS_DIR/common-files.txt"
  cp "$CHANGED_LIST" "$WRITE_MANIFESTS_DIR/changed-files.txt"
fi

count_lines() {
  wc -l < "$1" | tr -d ' '
}

echo "Codex repo: $CODEX_REPO_URL"
echo "Codex commit: $CODEX_COMMIT"
echo "Codex path: $CODEX_TUI_PATH"
echo "RunHaven path: $RUNHAVEN_TUI_PATH"
if [ -n "$WRITE_MANIFESTS_DIR" ]; then
  echo "Manifest output: $WRITE_MANIFESTS_DIR"
fi
echo
echo "Counts"
printf '  upstream files: %s\n' "$(count_lines "$UPSTREAM_LIST")"
printf '  RunHaven files: %s\n' "$(count_lines "$RUNHAVEN_LIST")"
printf '  common files: %s\n' "$(count_lines "$COMMON_LIST")"
printf '  upstream files not vendored: %s\n' "$(count_lines "$MISSING_LIST")"
printf '  RunHaven-only files: %s\n' "$(count_lines "$LOCAL_ONLY_LIST")"
printf '  copied Codex files with local edits: %s\n' "$(count_lines "$CHANGED_LIST")"

echo
echo "Upstream files not vendored by extension"
if [ -s "$MISSING_LIST" ]; then
  awk '
    {
      name = $0
      sub(/^.*\//, "", name)
      if (name ~ /\./) {
        sub(/^.*\./, "", name)
        count[name]++
      } else {
        count["[none]"]++
      }
    }
    END {
      for (ext in count) {
        printf "  %s %d\n", ext, count[ext]
      }
    }
  ' "$MISSING_LIST" | LC_ALL=C sort
else
  echo "  none"
fi

echo
echo "RunHaven-only files"
if [ -s "$LOCAL_ONLY_LIST" ]; then
  sed 's#^\./#  #' "$LOCAL_ONLY_LIST"
else
  echo "  none"
fi

echo
echo "Copied Codex files with local edits"
if [ -s "$CHANGED_LIST" ]; then
  sed 's#^\./#  #' "$CHANGED_LIST"
else
  echo "  none"
fi

if [ "$LIST_MISSING" -eq 1 ]; then
  echo
  echo "Upstream files not vendored"
  if [ -s "$MISSING_LIST" ]; then
    sed 's#^\./#  #' "$MISSING_LIST"
  else
    echo "  none"
  fi
fi

#!/usr/bin/env bash
# Fails if docs or instruction surfaces contain machine-local references.
# Keeps committed docs portable and free of private workstation paths.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

FILES_LIST="$(mktemp "${TMPDIR:-/tmp}/runhaven-doc-paths.XXXXXX")"
trap 'rm -f "$FILES_LIST"' EXIT

git ls-files -z -- \
  AGENTS.md \
  README.md \
  current-state.md \
  feature_list.json \
  .agents \
  docs > "$FILES_LIST"

if [ ! -s "$FILES_LIST" ]; then
  exit 0
fi

patterns=(
  '(/Users/|[A-Za-z]:\\Users\\)'
  '<[A-Za-z0-9._-]*(checkout|external-draft|external-source|local-source|codex-home)[A-Za-z0-9._/-]*>'
  'local (Codex|Apple container|Learn Harness|reference|source) checkout'
)

matches=""
for pattern in "${patterns[@]}"; do
  current="$(xargs -0 rg -n --color never "$pattern" -- < "$FILES_LIST" || true)"
  if [ -n "$current" ]; then
    if [ -n "$matches" ]; then
      matches="$matches"$'\n'
    fi
    matches="$matches$current"
  fi
done

if [ -n "$matches" ]; then
  cat >&2 <<'EOF'
Found machine-local references in docs or instruction surfaces.

Do not commit local home-directory paths, download folders, sibling checkout
paths, local workspace roots, or symbolic placeholders for local external
files. Use repo-relative paths or public URLs. Cite upstream source with a
public repository plus pinned commit. If a local external document is needed,
import or summarize it into docs/ first and cite the repo copy.

Matches:
EOF
  printf '%s\n' "$matches" >&2
  exit 1
fi

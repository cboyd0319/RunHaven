#!/usr/bin/env bash
# Read-only contract check shared by bundled image contexts.
# Validates required files and hardened container-image declarations.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: test-image-contract.sh CONTEXT" >&2
  exit 2
fi
if [ -L "$1" ] || [ ! -d "$1" ]; then
  echo "image contract requires a regular context directory: $1" >&2
  exit 1
fi

common="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
images="$(cd "$common/.." && pwd -P)"
context="$(cd "$1" && pwd -P)"
case "$context" in
  "$images"/*) ;;
  *) echo "image context escapes the images root: $context" >&2; exit 1 ;;
esac

require_regular() {
  [ ! -L "$1" ] && [ -f "$1" ] || {
    echo "image contract file is missing or unsafe: $1" >&2
    exit 1
  }
}

if [ "$context" = "$common" ]; then
  require_regular "$context/manifest.json"
  require_regular "$context/create-agent-user.sh"
  python3 -m json.tool "$context/manifest.json" >/dev/null
else
  require_regular "$context/Containerfile"
  grep -F "COPY common/" "$context/Containerfile" >/dev/null || {
    echo "image context does not consume images/common: $context" >&2
    exit 1
  }
  if [ -f "$context/package.json" ]; then
    require_regular "$context/package-lock.json"
    python3 -m json.tool "$context/package.json" >/dev/null
    python3 -m json.tool "$context/package-lock.json" >/dev/null
  fi
fi

echo "image unit contract passed: ${context##*/}"

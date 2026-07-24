#!/usr/bin/env bash
# Runs repository Python tools with the exact version owned by .python-version.
# Redirects bytecode caches to ignored build output outside the source tree.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPECTED="$(tr -d '[:space:]' < "$ROOT_DIR/.python-version")"
command -v python3 >/dev/null 2>&1 || {
  echo "python3 $EXPECTED is required; python3 was not found" >&2
  exit 1
}
ACTUAL="$(python3 -c 'import platform; print(platform.python_version())')"
[ "$ACTUAL" = "$EXPECTED" ] || {
  echo "python3 $EXPECTED is required; found $ACTUAL" >&2
  exit 1
}

export PYTHONPYCACHEPREFIX="${PYTHONPYCACHEPREFIX:-$ROOT_DIR/target/python-cache}"
cd "$ROOT_DIR"
exec python3 "$@"

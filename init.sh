#!/usr/bin/env bash
set -euo pipefail

echo "== Harness verification for RunHaven =="
echo "Detected stack: python"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "RunHaven verification requires macOS 26+."
  exit 1
fi

MACOS_VERSION="$(sw_vers -productVersion)"
MACOS_MAJOR="${MACOS_VERSION%%.*}"
if [ "$MACOS_MAJOR" -lt 26 ]; then
  echo "RunHaven verification requires macOS 26+; found ${MACOS_VERSION}."
  exit 1
fi

if [ "$(uname -m)" != "arm64" ]; then
  echo "RunHaven verification requires Apple silicon."
  exit 1
fi

PYTHON_BIN="${PYTHON:-}"
if [ -z "$PYTHON_BIN" ]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
  else
    PYTHON_BIN="python"
  fi
fi

echo "== python3 -m compileall src tests scripts =="
"${PYTHON_BIN}" -m compileall src tests scripts

echo "== PYTHONPATH=src python3 -m unittest discover -s tests =="
PYTHONPATH=src "${PYTHON_BIN}" -m unittest discover -s tests

echo "== python3 scripts/check_pins.py =="
"${PYTHON_BIN}" scripts/check_pins.py

echo "== python3 -m ruff check . =="
"${PYTHON_BIN}" -m ruff check .

echo "== python3 -m mypy src =="
"${PYTHON_BIN}" -m mypy src

echo "== python3 -m build =="
"${PYTHON_BIN}" -m build

echo "== Harness verification complete =="

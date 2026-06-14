#!/usr/bin/env bash
set -euo pipefail

echo "== Harness verification for macos-container-agents =="
echo "Detected stack: python"

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

#!/usr/bin/env bash
# Enforce repository file budgets, topology, and module ownership.
# Keeps shell policy thin over fixture-tested standard-library Python cores.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

"$ROOT_DIR/scripts/python.sh" scripts/check_file_limits.py
"$ROOT_DIR/scripts/python.sh" scripts/check_repo_layout.py

echo "repository structure check passed"

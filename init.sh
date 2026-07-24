#!/usr/bin/env bash
# Thin launcher for the standard macOS initialization core.
# Delegates setup checks and the representative baseline without duplicating policy.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

exec "$ROOT_DIR/scripts/python.sh" scripts/init_repository.py "$@"

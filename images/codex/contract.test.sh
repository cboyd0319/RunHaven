#!/usr/bin/env bash
# Codex image-context contract entrypoint.
# Delegates its directory to the shared read-only image validator.
set -euo pipefail

context="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
exec bash "$context/../common/test-image-contract.sh" "$context"

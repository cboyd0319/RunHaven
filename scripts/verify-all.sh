#!/usr/bin/env bash
# Full macOS 26+ verification lane for shared contracts and release-sensitive work.
# Runs the maintained local gates in one fail-fast sequence without hosted CI.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
PYTHON="$PWD/scripts/python.sh"

run() {
  printf '== %s ==\n' "$*"
  "$@"
}

run "$PYTHON" scripts/check_environment.py --full
run "$PYTHON" scripts/test_environment.py
run bash scripts/test_scan_dev_log.sh
run "$PYTHON" scripts/check_harness_state.py
run "$PYTHON" scripts/test_harness_state.py
run "$PYTHON" scripts/test_file_limits.py
run "$PYTHON" scripts/test_repo_layout.py
run scripts/check-repo-structure.sh
run bash images/common/contract.test.sh
run bash images/base/contract.test.sh
run bash images/antigravity/contract.test.sh
run bash images/claude/contract.test.sh
run bash images/codex/contract.test.sh
run bash images/copilot/contract.test.sh

# shellcheck disable=SC1091
source scripts/lib/rust-toolchain.sh
activate_pinned_rust_toolchain "$PWD"

run cargo fmt --check
run cargo test --workspace --locked
run cargo clippy --workspace --all-targets --locked -- -D warnings
run cargo run --locked --bin runhaven-check-pins
run scripts/check-support-issue-form.sh
run scripts/release-desktop-common.test.sh
run scripts/release-desktop-preflight.test.sh
run scripts/local-sign-desktop.test.sh
run scripts/rust-toolchain.test.sh
run scripts/review-gui.test.sh
run "$PYTHON" -m json.tool feature_list.json
run scripts/check-docs-local-paths.sh
run npm ci --ignore-scripts
run npm run check
run npm test
run npm run build
run npm run test:e2e
run npm run tauri:build
run cargo build --workspace --locked
run git diff --check

printf 'full verification passed\n'

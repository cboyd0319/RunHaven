#!/usr/bin/env bash
# Runs Cargo with the exact rustup toolchain owned by rust-toolchain.toml.
# Prevents unrelated Homebrew or system Cargo binaries from bypassing the pin.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/lib/rust-toolchain.sh"
activate_pinned_rust_toolchain "$ROOT_DIR"
cd "$ROOT_DIR"

exec cargo "$@"

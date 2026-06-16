#!/usr/bin/env bash
set -euo pipefail

echo "== Harness verification for RunHaven =="
echo "Detected stack: rust"

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

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo was not found on PATH."
  exit 1
fi

echo "== cargo fmt --check =="
cargo fmt --check

echo "== cargo test --locked =="
cargo test --locked

echo "== cargo clippy --all-targets -- -D warnings =="
cargo clippy --all-targets -- -D warnings

echo "== cargo run --locked --bin runhaven-check-pins =="
cargo run --locked --bin runhaven-check-pins

echo "== cargo build --locked =="
cargo build --locked

echo "== Harness verification complete =="

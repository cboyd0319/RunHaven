#!/usr/bin/env bash
# Run the local macOS verification harness for RunHaven.
# Checks Rust formatting, TUI snapshots, workspace tests, clippy, pin policy,
# harness state, frontend, Tauri packaging, diff hygiene, and build output.
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

echo "== cargo test --workspace --locked tui =="
cargo test --workspace --locked tui

echo "== cargo test --workspace --locked =="
cargo test --workspace --locked

echo "== cargo clippy --workspace --all-targets --locked -- -D warnings =="
cargo clippy --workspace --all-targets --locked -- -D warnings

echo "== cargo run --locked --bin runhaven-check-pins =="
cargo run --locked --bin runhaven-check-pins

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 was not found on PATH."
  exit 1
fi

echo "== python3 -m json.tool feature_list.json =="
python3 -m json.tool feature_list.json >/dev/null

if [ -f "ui/package.json" ]; then
  if ! command -v npm >/dev/null 2>&1; then
    echo "npm was not found on PATH."
    exit 1
  fi

  echo "== npm --prefix ui ci --ignore-scripts =="
  npm --prefix ui ci --ignore-scripts

  echo "== npm --prefix ui run check =="
  npm --prefix ui run check

  echo "== npm --prefix ui test =="
  npm --prefix ui test

  echo "== npm --prefix ui run build =="
  npm --prefix ui run build

  echo "== npm --prefix ui run test:e2e =="
  npm --prefix ui run test:e2e

  if [ -f "src-tauri/Cargo.toml" ]; then
    echo "== npm --prefix ui run tauri:build =="
    npm --prefix ui run tauri:build
  fi
fi

echo "== cargo build --workspace --locked =="
cargo build --workspace --locked

echo "== git diff --check =="
git diff --check

echo "== Harness verification complete =="

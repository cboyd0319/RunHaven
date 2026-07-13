#!/bin/bash
# Verify repository tools ignore unrelated Rust installations already on PATH.
# Exercise rustup selection and exact pinned Cargo and Rust version checks.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/runhaven-rust-toolchain.XXXXXX")
trap 'rm -rf "$tmp"' EXIT INT TERM HUP

wrong_bin="$tmp/wrong/bin"
pinned_bin="$tmp/pinned/bin"
mkdir -p "$wrong_bin" "$pinned_bin"

write_version_command() {
  local path=$1
  local name=$2
  local version=$3
  printf '#!/bin/bash\nprintf '\''%s %s test\\n'\''\n' "$name" "$version" > "$path"
  chmod +x "$path"
}

write_version_command "$wrong_bin/cargo" cargo 1.96.1
write_version_command "$wrong_bin/rustc" rustc 1.96.1
write_version_command "$pinned_bin/cargo" cargo 1.97.0
write_version_command "$pinned_bin/rustc" rustc 1.97.0

cat > "$wrong_bin/rustup" <<'SH'
#!/bin/bash
if [ "${1:-}" = "which" ] && [ "${2:-}" = "--toolchain" ]; then
  printf '%s/%s\n' "$TEST_PINNED_BIN" "${4:-}"
  exit 0
fi
exit 1
SH
chmod +x "$wrong_bin/rustup"

PATH="$wrong_bin:/usr/bin:/bin"
export PATH TEST_PINNED_BIN="$pinned_bin"
# shellcheck disable=SC1091
source "$ROOT/scripts/lib/rust-toolchain.sh"
activate_pinned_rust_toolchain "$ROOT"

[ "$(command -v cargo)" = "$pinned_bin/cargo" ]
[ "$(command -v rustc)" = "$pinned_bin/rustc" ]
[ "$(cargo --version | awk '{print $2}')" = "1.97.0" ]
[ "$(rustc --version | awk '{print $2}')" = "1.97.0" ]

printf 'rust-toolchain selection tests passed\n'

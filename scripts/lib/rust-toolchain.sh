#!/bin/bash

# Put the repository-pinned rustup toolchain ahead of unrelated cargo/rustc
# installations already on PATH, then verify both selected versions.
activate_pinned_rust_toolchain() {
  local root=$1
  local channel cargo_path rustc_path toolchain_bin cargo_version rustc_version

  channel=$(awk -F '"' '/^[[:space:]]*channel[[:space:]]*=/{print $2; exit}' \
    "$root/rust-toolchain.toml")
  [ -n "$channel" ] || {
    printf 'could not read the pinned Rust channel from rust-toolchain.toml\n' >&2
    return 1
  }

  cargo_path=$(rustup which --toolchain "$channel" cargo) || return 1
  rustc_path=$(rustup which --toolchain "$channel" rustc) || return 1
  toolchain_bin=${cargo_path%/*}
  [ "$toolchain_bin" = "${rustc_path%/*}" ] || {
    printf 'rustup selected cargo and rustc from different toolchains\n' >&2
    return 1
  }

  PATH="$toolchain_bin:$PATH"
  export PATH
  cargo_version=$(cargo --version | awk '{print $2}')
  rustc_version=$(rustc --version | awk '{print $2}')
  [ "$cargo_version" = "$channel" ] || {
    printf 'pinned Cargo %s is required; rustup selected %s\n' \
      "$channel" "$cargo_version" >&2
    return 1
  }
  [ "$rustc_version" = "$channel" ] || {
    printf 'pinned Rust %s is required; rustup selected %s\n' \
      "$channel" "$rustc_version" >&2
    return 1
  }
}

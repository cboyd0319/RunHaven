# Pinning Policy

All package, image, and tool dependencies must be current stable and
hard-pinned. While the project has no active GitHub Actions workflows, there
are no active CI action dependencies to pin.

## Required Pins

- Rust direct dependencies use exact `=` versions in root
  `Cargo.toml` workspace dependencies.
- Workspace member crates, including `src-tauri`, consume those pinned
  dependencies through `workspace = true`.
- Rust toolchain version is pinned in `rust-toolchain.toml` and `pins.toml`.
- `Cargo.lock` is checked in for reproducible workspace builds, including the
  CLI, TUI, core library, and Tauri shell.
- Frontend direct dependencies use exact versions in `ui/package.json`, and
  `ui/package-lock.json` is checked in.
- If GitHub Actions workflows are reintroduced, actions must use immutable
  commit SHAs, with the release tag in a comment.
- Container base images use versioned tags plus `sha256` digests.
- Debian packages installed in images use exact package versions in
  `images/common/debian-packages.txt`, including the observed install closure
  for the base image.
- Debian apt sources use timestamped `snapshot.debian.org` URIs so exact
  package pins do not depend on moving mirrors.
- npm packages installed in images use exact package versions.
- npm packages used by the desktop UI use exact package versions.
- Direct binary downloads use exact versioned URLs plus checksum verification.
- Apple `container` install evidence records the release version, commit,
  installer SHA-256, signing team ID, and observed runtime helper versions.

## Disallowed

- `latest` image or package tags
- major-only GitHub Action refs such as `actions/checkout@v6`, if workflows
  are reintroduced
- loose dependency ranges such as `>=`, `~=`, or wildcard package pins
- unversioned installer scripts inside images
- unpinned `apt-get install`, `npm install`, or `cargo add`

Run the policy check:

```bash
cargo run --locked --bin runhaven-check-pins
```

The current reviewed pins are recorded in [`pins.toml`](../pins.toml).
The source record for current-version checks is
[`RESEARCH.md`](RESEARCH.md).

## Known Advisories

- GHSA-wrw7-89jp-8q8g (`glib` < 0.20.0, `VariantStrIter` iterator
  unsoundness): not applicable. `glib` is pulled only through Tauri's Linux
  GTK backend (`glib 0.18` &larr; `gtk 0.18.2` &larr;
  `webkit2gtk`/`wry`/`tao`/`muda` &larr; `tauri-runtime-wry 2.11.3`) and is
  absent from the macOS `aarch64-apple-darwin` build graph
  (`cargo tree --locked -p runhaven-tauri --target aarch64-apple-darwin -i glib`
  prints nothing on the host target), so no macOS build or shipped artifact compiles the
  vulnerable code. It is capped at 0.18.x by `gtk 0.18.2`; the patched 0.20.0
  needs a newer gtk-rs generation Tauri does not yet use. The Dependabot alert
  was dismissed as "not used" on 2026-06-24. Revisit if Tauri advances its
  Linux GTK stack or if a macOS dependency path to `glib` ever appears.

Apple `container` runtime helper images and the default Kata kernel are managed
by Apple `container`, not by this repo. Record observed values in `pins.toml`
and verify the signed installer before changing the minimum supported runtime.
`runhaven doctor` compares the installed runtime commit and structured runtime
helper properties with the recorded pins and fails closed on drift. Installer
SHA-256, signing team ID, and kernel SHA-256 remain release-evidence fields
unless a separate package or filesystem hash check is added.
Use
[`apple-container-update-playbook.md`](harness/release/apple-container-update-playbook.md)
for Apple `container` runtime, helper image, installer, and Kata kernel pin
updates.

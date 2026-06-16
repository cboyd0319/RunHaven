# Pinning Policy

All package, image, tool, and CI action dependencies must be current stable and
hard-pinned.

## Required Pins

- Rust direct dependencies use exact `=` versions in `Cargo.toml`.
- Rust toolchain version is pinned in `rust-toolchain.toml` and `pins.toml`.
- `Cargo.lock` is checked in for reproducible CLI builds.
- GitHub Actions use immutable commit SHAs, with the release tag in a comment.
- Container base images use versioned tags plus `sha256` digests.
- Debian packages installed in images use exact package versions in
  `images/common/debian-packages.txt`, including the observed install closure
  for the base image.
- Debian apt sources use timestamped `snapshot.debian.org` URIs so exact
  package pins do not depend on moving mirrors.
- npm packages installed in images use exact package versions.
- Direct binary downloads use exact versioned URLs plus checksum verification.
- Apple `container` install evidence records the release version, commit,
  installer SHA-256, signing team ID, and observed runtime helper versions.

## Disallowed

- `latest` image or package tags
- major-only GitHub Action refs such as `actions/checkout@v6`
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

Apple `container` runtime helper images and the default Kata kernel are managed
by Apple `container`, not by this repo. Record observed values in `pins.toml`
and verify the signed installer before changing the minimum supported runtime.

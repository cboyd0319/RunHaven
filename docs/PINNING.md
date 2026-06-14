# Pinning Policy

All package, image, tool, and CI action dependencies must be current stable and
hard-pinned.

## Required Pins

- Python build dependencies use exact `==` versions in `pyproject.toml`.
- Python development dependencies use exact `==` versions in `pyproject.toml`.
- Python development transitive dependencies use exact `==` versions in
  `requirements-dev.txt`.
- GitHub Actions use immutable commit SHAs, with the release tag in a comment.
- Container base images use versioned tags plus `sha256` digests.
- Debian packages installed in images use exact package versions in
  `src/macos_container_agents/images/common/debian-packages.txt`, including the
  observed install closure for the base image.
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
- unpinned `apt-get install`, `npm install`, or `pip install`

Run the policy check:

```bash
python3 scripts/check_pins.py
```

The current reviewed pins are recorded in [`pins.toml`](../pins.toml).
Python development transitive pins are recorded in
[`requirements-dev.txt`](../requirements-dev.txt).
The source record for current-version checks is
[`RESEARCH.md`](RESEARCH.md).

Apple `container` runtime helper images and the default Kata kernel are managed
by Apple `container`, not by this repo. Record observed values in `pins.toml`
and verify the signed installer before changing the minimum supported runtime.

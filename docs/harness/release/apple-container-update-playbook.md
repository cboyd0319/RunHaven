# Apple Container Update Playbook

Status: live

Use this checklist when changing RunHaven's pinned Apple `container` runtime or
helper pin surface. Runtime updates are security-sensitive because RunHaven
uses Apple `container` as the local boundary on users' personal machines.

Do not rely on chat history, local memory, or unstated host state. Record the
source, command, date, and result for every pin changed.

## Approval Boundary

Get explicit maintainer approval before:

- installing, updating, or uninstalling Apple `container`;
- restarting `container system` services when active workloads may exist;
- deleting machines, images, volumes, networks, registry credentials, or user
  data;
- publishing or yanking packages, images, or release artifacts.

Read-only inspection, local build checks, and RunHaven-owned smoke cleanup do
not need an admin action.

## Pre-Update Evidence

Collect this evidence before editing repo files:

```bash
sw_vers
uname -m
container --version
pkgutil --pkg-info com.apple.container-installer
container system status
container system version --format json
container system property list --format json
container run --help
container inspect --help
container network --help
container image list --help
container volume list --help
container logs --help
container exec --help
container kill --help
container machine --help
container registry --help
container builder status --format json
```

For the target installer package, record:

- official release version;
- release tag or source archive used for review;
- release commit SHA;
- exact installer package filename;
- installer SHA-256;
- installer signing Team ID;
- source URL or release page;
- date checked.

Receipt data from `pkgutil` can confirm what is installed, but it is not
package-signature proof. Verify the actual signed installer package for every
runtime update.

For the installed target runtime, record:

- `container` app version, build type, and commit from
  `container system version --format json`;
- `container-apiserver` version, build type, and commit from the same command;
- builder image from `container system property list --format json`;
- vminit image from `container system property list --format json`;
- Kata kernel release parsed from the kernel URL;
- kernel filename from the kernel binary path;
- kernel SHA-256 from the downloaded kernel file.

The kernel file is host-local runtime data. On the current pinned runtime it is
stored under the Apple `container` application-support kernel directory. Verify
the current location from `container system property list --format json` and
local filesystem evidence instead of hardcoding a path.

## Source Review

Review the Apple `container` release notes, local CLI help, and local source
checkout or source archive before changing pins. Focus on changes to:

- `container run`, networking, mounts, volumes, images, logs, exec, machine,
  registry, builder, and system commands;
- vmnet, DNS, Local Network privacy, XPC, launchd, Keychain, and helper-service
  boundaries;
- `--ssh` behavior for non-root guest users;
- JSON output shapes used by `runhaven doctor`, `image doctor`, provider
  runtime checks, and active-run commands;
- installer location, signing identity, helper images, vminit image, builder
  image, and Kata kernel source.

Treat a local Apple `container` source checkout as source-map evidence unless it
is exactly the reviewed release tag or source archive. Do not pin from local
branch HEAD, current-branch docs, or `Package.swift` alone.

Do not assume Docker daemon, Compose, Docker Desktop, or shared-kernel Linux
container behavior.

## Repo Updates

Update only the files needed for the changed pin surface:

- `pins.toml`: Apple `container` version, commit, installer SHA-256, installer
  Team ID, builder image, vminit image, Kata release, kernel filename, and
  kernel SHA-256.
- `crates/runhaven-core/src/doctor/runtime_pins.rs`: fixture pins and parser fixtures
  when structured Apple JSON changed.
- `crates/runhaven-core/tests/fixtures/apple_container/`: trimmed JSON fixtures
  for changed Apple output shapes.
- `docs/APPLE_CONTAINER_GAP_ANALYSIS.md`: host evidence, source evidence,
  remaining gaps, and whether `--ssh` is still fail-closed.
- `docs/RESEARCH.md`: source-review notes when the update changes runtime,
  helper, installer, or platform assumptions.
- `docs/PINNING.md`, `README.md`, `docs/INSTALLATION.md`, and other product
  docs with explicit old version text.
- `docs/harness/evidence/evidence-log.md`: compact evidence row for the update.
- `current-state.md`: latest verified work, trusted verification, blockers, and
  next step.

If the update changes default security posture, also review:

- `docs/SECURITY_MODEL.md`;
- `docs/harness/boundaries/security-boundary-map.md`;
- `docs/harness/boundaries/feature-privacy-labels.json`;
- `docs/TAURI_UI_GUARDRAILS.md`.

## Required Verification

Run the smallest focused checks first, then the release-level checks:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --locked
```

Then run the live Apple `container` checks on macOS 26+ Apple silicon:

```bash
cargo run --locked --bin runhaven -- doctor
scripts/apple_container_smoke.sh
scripts/apple_container_smoke.sh --with-provider
scripts/apple_container_smoke.sh --with-ssh
```

`--with-ssh` currently proves the fail-closed guard. Re-enable SSH forwarding
only after a no-secret smoke proves that a non-root guest can reach a
disposable empty agent and `ssh-add -l` returns the no-identities status.
Do not treat a version bump by itself as SSH evidence.

For docs and harness updates, also run:

```bash
python3 -m json.tool feature_list.json
python3 -m json.tool docs/harness/manifest.json
git diff --check
```

Run a local Markdown link check over tracked Markdown files before claiming
docs are ready.

## Cleanup Checks

After every live smoke, verify local state:

```bash
cargo run --locked --bin runhaven -- runs active
cargo run --locked --bin runhaven -- state list
cargo run --locked --bin runhaven -- network list
container list
container volume list
container network list
```

Cleanup must be targeted to RunHaven-owned resources. Prefer RunHaven commands
such as `runs stop`, `runs repair`, `network prune --yes`, and `state reset`
for owned resources. Do not use broad native prune/delete commands as a release
cleanup shortcut.

## Rollback

If the new runtime fails verification or weakens the security boundary:

1. Stop the update and keep the working tree or branch for diagnosis.
2. Restore the last reviewed pins and docs with a reviewed revert commit.
3. Re-run `cargo test --locked`, `cargo run --locked --bin runhaven-check-pins`,
   `cargo run --locked --bin runhaven -- doctor`, and the relevant smoke script.
4. Record the failed target version, observed failure, risk, and recovery action
   in `docs/harness/evidence/evidence-log.md`.
5. If any artifact was published, get maintainer approval before yanking or
   publishing a corrective release.

Do not hide a failed runtime update by relaxing `doctor`, skipping helper pin
checks, mounting raw credentials, widening workspace defaults, or switching the
default agent user to root.

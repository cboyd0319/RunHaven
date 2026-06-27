# Authoritative Facts And Docs Routing

Status: live

Repo-owned docs, tests, policy, local commands, and maintainer decisions are
authoritative for RunHaven. Optional structural reports are advisory unless a
maintainer promotes a finding into repo-owned artifacts.

## Owners

| Boundary | Owner |
| --- | --- |
| Startup harness | `AGENTS.md`, `feature_list.json`, `current-state.md` |
| Harness map | `docs/harness/README.md`, `docs/harness/manifest.json` |
| CLI/runtime behavior | `crates/runhaven-core/`, `crates/runhaven-cli/`, `crates/runhaven/tests/`, `docs/USAGE.md` |
| Desktop shell | `src-tauri/`, `ui/`, `docs/TAURI_UI_GUARDRAILS.md` |
| Security and privacy | `docs/SECURITY_MODEL.md`, `docs/harness/boundaries/security-boundary-map.md` |
| Platform contracts | macOS 26+, Apple `container` 1.0.0, Rust 1.96.0 |
| Release/package surface | `docs/V1_RELEASE_PLAN.md`, `docs/RELEASE_GAP_ANALYSIS.md`, `Cargo.toml`, `Cargo.lock`, `pins.toml`, `docs/harness/release/` |

## Fan-Out Budget

- Routine change: 0-1 state or doc updates.
- User-visible or harness-visible change: 1-3 focused updates.
- More than 3 durable updates needs an explicit product, security, platform,
  release, or startup-contract reason.

## Routing Rules

- Keep startup files compact.
- Prefer links and compact summaries over duplicated policy prose.
- Update the authoritative owner first.
- Do not commit machine-local paths, personal tool mandates, private data, or
  raw command output.

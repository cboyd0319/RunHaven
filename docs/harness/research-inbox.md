# Research Inbox

Generated: 2026-06-14

Use this file for compact source-change notes. Keep long excerpts out of the
repo and link to primary sources instead.

## Promotion checklist

- The source is current and relevant to this repo's harness.
- The finding improves security, portability, verification, restartability, or
  ease of use.
- The change can be represented as a template, audit check, verification
  command, policy file, or clear documentation.
- The proposed update preserves local-machine safety and reviewability.

## Notes

- 2026-06-15: Completed a manual source-mining pass across sibling local
  checkouts `awman`, `aspec`, and `maki`. AGY/Antigravity was not used.
- The full recommendation set is recorded in
  `docs/harness/source-mined-ideas.md`.
- High-value candidates promoted from the pass: provider endpoint matrix,
  provider proxy DNS/private-address guard, worktree isolation, workspace scope
  selection, run observability, strict workflow files, context overlays, MCP
  boundaries, generated docs drift checks, and JSON/headless output.
- Ideas explicitly not promoted as-is: host Keychain or credential extraction,
  default host home mounts, Docker fallback, Windows/Linux runtime support,
  `container machine` defaults, and unreviewed host-side dynamic provider
  scripts.
- 2026-06-15: Completed a manual external open source research pass. The full
  set is recorded in `docs/harness/external-project-ideas.md`.
- Additional external-project candidates promoted from that pass: `runhaven
  why` diagnostics, provider proxy policy logs, empty-allowlist regression
  tests, host-side provider credential brokering, agent profile investigation
  docs, devcontainer metadata import for image planning, warm reusable project
  sessions, and explicit extension/MCP boundary policy.
- 2026-06-15: Completed a UX-focused research pass around easier setup,
  clearer blocked-action explanations, recovery, and lower-friction safe
  autonomy. The full result is recorded in
  `docs/harness/ux-research-ideas.md`.
- UX candidates promoted from that pass: guided `runhaven setup`, goal-based
  network selection, `runhaven why`, provider policy logs, grouped
  blocked-host review, `runs list/show/log/diff/attach/stop`, worktree review
  flows, image/state/network repair commands, `auth status`, and task-language
  docs recipes.
- 2026-06-15: First implementation slice landed from the promoted backlog:
  provider proxy DNS/private-address rejection, provider policy decision logs,
  and `runhaven why host ...`.
- 2026-06-15: Promoted the provider endpoint matrix into
  `docs/PROVIDER_ENDPOINTS.md` and `src/runhaven/provider_endpoints.py`.
  Source-backed defaults now include Claude auth hosts, Codex ChatGPT auth, and
  Copilot-specific routing hosts. Broad path-sensitive, telemetry, update, and
  weakly sourced hosts remain explicit review items.
- 2026-06-15: Implemented grouped blocked-host review and profile-host smoke
  support. Provider run summaries now include run id, reason, count, rule, and
  suggested next action; `scripts/provider_egress_smoke.py --agent AGENT`
  validates bundled provider hosts without credentials.
- 2026-06-15: Implemented the auth broker boundary diagnostics slice.
  `runhaven auth status` and `runhaven auth explain AGENT` read static profile
  metadata only, with JSON output available for future automation. No
  credential store or environment value is read.

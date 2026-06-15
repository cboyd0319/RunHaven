# External Open Source Project Ideas

Reviewed: 2026-06-15

Scope: current open source or source-available-adjacent projects that overlap
with RunHaven's goal: running coding agents with stronger local isolation,
clearer egress boundaries, safer workspace handling, and better recovery.

This pass used manual web and source review. AGY/Antigravity was not used.

RunHaven remains macOS 26+ only and Apple `container` only. Projects that use
Docker, Linux-only primitives, Windows support, cloud sandboxes, or
`sandbox-exec` are still useful as idea sources, but they are not runtime
targets for RunHaven.

## Reviewed Projects

### Anthropic Sandbox Runtime

Source:
https://github.com/anthropic-experimental/sandbox-runtime

Relevant ideas:

- Native process sandboxing with filesystem and network policy.
- Proxy-based network filtering with an allow-only model.
- MCP server sandboxing as a first-class use case.
- Violation monitoring on macOS.
- Explicit warnings that trusted user-level configuration must not be weakened
  by project-local files.
- Security advisory history showing why empty allowlist and fail-closed tests
  matter.

RunHaven direction:

- Keep the provider proxy allow-only and test empty-allowlist behavior.
- Add policy-decision records for denied network events.
- Treat any future project-local config as unable to loosen global or
  user-approved security policy.
- If RunHaven later launches host-side helper processes or MCP servers, apply a
  separate host sandbox or move them inside the container.

### nono

Source:
https://github.com/always-further/nono

Relevant ideas:

- Agent-agnostic sandbox profiles.
- Current-directory access by default, with secrets and the rest of the disk
  hidden.
- Profile registry and profile scaffolding.
- Composable policy, credentials injection, L7 filtering, audit, rollback, and
  library bindings.

RunHaven direction:

- Add a `runhaven why` or `runhaven explain` command for blocked hosts, blocked
  mounts, rejected paths, and sensitive defaults.
- Consider signed or pinned community profile packs only after profile schema,
  source provenance, and review rules exist.
- Prefer host-side credential brokering over placing provider secrets in the
  guest environment.
- Treat rollback as a product goal tied to worktrees and run records.

### Agent Safehouse

Source:
https://github.com/eugene1g/agent-safehouse

Relevant ideas:

- macOS-focused, deny-first profile system for local AI agents.
- Practical least privilege rather than perfect-security claims.
- Agent-specific profile investigations and E2E macOS tests.

RunHaven direction:

- Keep threat-model language plain and limitation-aware.
- Add per-agent profile investigations for Codex, Claude, Gemini, Copilot, and
  future agents: required files, network hosts, update paths, auth paths, and
  known blocked operations.
- Add macOS-only E2E smoke coverage for the safest common workflows.

### SandVault

Source:
https://github.com/webcoyote/sandvault

Relevant ideas:

- Separate macOS user account plus `sandbox-exec` hardening.
- Agent shortcuts and native installation inside the isolated environment.
- Clean rebuild and uninstall flows.
- Notes about nested sandbox limitations.

RunHaven direction:

- Keep `runhaven doctor` and future repair commands oriented around exact,
  beginner-safe remediation.
- Add `runhaven image rebuild` or equivalent guided repair if agent images get
  stale or corrupted.
- Document nested sandbox expectations when users run agents that already have
  their own sandbox or approval system.

### VibeBox

Source:
https://github.com/robcholz/vibebox

Relevant ideas:

- Per-project VM sandbox optimized for fast warm re-entry.
- Explicit mount allowlists.
- Reusable sessions and multi-instance management.

RunHaven direction:

- Add warm reusable project sessions as a product goal, while preserving
  explicit workspace and mount scope. Implemented 2026-06-15 with
  `--session`, `state reset`, and session-filtered list/prune commands.
- Treat state volumes and run records as user-visible session concepts instead
  of hidden implementation detail.

### Matchlock

Source:
https://github.com/jingkaihe/matchlock

Relevant ideas:

- Ephemeral microVMs for agents.
- Network sealed by default when allowed hosts or secrets are configured.
- Host-side secret injection where real credentials never enter the sandbox.

RunHaven direction:

- Investigate a host-side credential broker for provider APIs so the container
  receives placeholders or scoped tokens instead of raw long-lived secrets.
- Tie credential brokering to provider egress policy: if secrets are brokered,
  network access must be sealed to the matching provider endpoints.

### yolo-cage

Source:
https://github.com/borenstein/yolo-cage

Relevant ideas:

- Permission fatigue is a real safety problem.
- Branch isolation and git-operation regulation.
- HTTP-layer controls for secrets, GitHub API operations, and exfiltration
  destinations.

RunHaven direction:

- Design for fewer prompts by making the outer sandbox stronger, not by
  trusting agent-level approvals.
- Add branch/worktree policy before offering high-autonomy modes.
- Consider API-operation policy for GitHub only after RunHaven has explicit
  credential and host brokering.

### yolobox

Source:
https://github.com/finbarr/yolobox

Relevant ideas:

- The agent gets a box where package installation and system mutation are cheap.
- Named volumes preserve setup across sessions.
- Security limitations are stated plainly.

RunHaven direction:

- Preserve the idea of an agent-owned Linux environment where package installs
  do not touch the host.
- Keep non-root defaults, but make package/tool cache persistence easier to
  understand and reset.
- Keep limitation language specific: mounted workspace, explicitly forwarded
  secrets, and allowed network paths remain in scope for damage.

### Container Use

Source:
https://github.com/dagger/container-use
https://container-use.com/agent-integrations

Relevant ideas:

- Containers plus git worktrees as a combined isolation model.
- MCP integration across many agents.
- Real-time visibility into command history and logs.
- Direct terminal intervention for stuck agents.
- Tool-restricted agent profiles so agents act through the environment API
  instead of touching host files directly.

RunHaven direction:

- Add worktrees and runtime isolation together, not as unrelated features.
- Add `runs log`, `runs diff`, and `runs attach` style operations.
- If RunHaven exposes MCP tools, make them the only tools a locked-down agent
  needs for environment operations.

### DevPod And Dev Containers

Sources:
https://github.com/loft-sh/devpod
https://github.com/devcontainers/cli
https://containers.dev/implementors/json_reference/

Relevant ideas:

- Declarative development environment metadata.
- Lifecycle hooks.
- Features and templates as reusable environment components.
- `read-configuration`, `up`, `exec`, `stop`, and `down` command shape.

RunHaven direction:

- Consider reading `devcontainer.json` as input to image planning, but do not
  execute host-side `initializeCommand` without explicit approval.
- Reuse the concepts of features, lifecycle hooks, and scenario tests for
  RunHaven image templates.
- Keep Apple `container` as the runtime even when importing devcontainer-like
  metadata.

### OpenHands

Source:
https://github.com/OpenHands/OpenHands
https://docs.openhands.dev/sdk/arch/overview

Relevant ideas:

- Shared SDK/source-of-truth consumed by CLI, GUI, and cloud interfaces.
- Typed boundaries around agents, workspaces, tools, conversations, events, and
  security policies.
- Clear split between core agent framework and optional workspace/server
  packages.

RunHaven direction:

- Keep planner and policy objects as the source of truth while the CLI remains
  a thin interface.
- If RunHaven later adds a GUI or API, reuse the same typed plan/run/policy
  layer rather than duplicating behavior.

### Goose

Sources:
https://github.com/aaif-goose/goose
https://goose-docs.ai/docs/getting-started/using-extensions/
https://goose-docs.ai/docs/guides/security/adversary-mode/

Relevant ideas:

- Broad provider and MCP extension ecosystem.
- Extension discovery and configuration UX.
- Adversary reviewer for tool calls before execution.
- Subagents and recipes.

RunHaven direction:

- Treat MCP as a major boundary-expansion feature, not a small config knob.
- Add extension health checks and offline/airgapped explanations if MCP support
  lands.
- Consider optional reviewer/adversary workflows inside the container, but do
  not substitute reviewer approval for technical sandbox enforcement.

### OpenCode

Source:
https://opencode.ai/docs/permissions/

Relevant ideas:

- Granular command permission rules.
- Per-agent permission overrides.
- Markdown agent definitions.

RunHaven direction:

- Generate or document recommended agent-native permission settings for each
  bundled RunHaven profile.
- Keep agent-native permissions as defense in depth only; RunHaven's container,
  mount, egress, and credential boundaries remain authoritative.

### Aider

Sources:
https://github.com/aider-ai/aider
https://aider.chat/docs/repomap.html
https://aider.chat/docs/git.html

Relevant ideas:

- Repo maps provide compact codebase context.
- Automatic commits and dirty-file separation make undo and review easier.
- Lint/test loops are part of the user workflow.

RunHaven direction:

- Add optional repo-map or context-summary generation inside the container.
- Tie worktree mode to clear git review and undo flows.
- Let profiles declare suggested test commands without granting extra host
  access.

### E2B

Source:
https://github.com/e2b-dev/e2b

Relevant ideas:

- Sandbox SDKs for creating and controlling isolated environments.
- Templates and cloud-scale sandbox lifecycle.

RunHaven direction:

- Keep cloud execution out of the default product, but borrow the API shape:
  create, start, exec, upload/download scoped files, snapshot, and destroy.
- Use local Apple `container` state and explicit mounts instead of managed
  cloud sandboxes.

### Microsandbox

Source:
https://github.com/superradcompany/microsandbox

Relevant ideas:

- Local microVMs for untrusted workloads, including AI agents, plugins, CI
  jobs, and dev environments.
- SDK-facing sandbox lifecycle.

RunHaven direction:

- Treat SDK-style lifecycle as future inspiration if RunHaven becomes an
  embeddable library.
- Keep Apple `container` as the only supported runtime unless the product goal
  changes explicitly.

## Cross-Project Patterns To Incorporate

- Worktree plus runtime isolation is becoming the standard shape for parallel
  agent work.
- Deny-by-default network policy needs host-side enforcement, blocked-event
  records, and explicit user diagnostics.
- Credential forwarding should move toward host-side brokering and scoped
  injection instead of raw environment variables inside the guest.
- Project-local configuration must never weaken a user- or organization-level
  safety policy.
- Users need recovery commands: explain, list, log, diff, attach, stop, repair,
  rebuild, discard, and clean.
- Agent-native permissions are useful, but they are not the boundary.
- Reusable sessions and cached tools reduce friction, but they need obvious
  reset and prune paths.
- Documentation should be generated or checked from source policy metadata once
  profiles and provider endpoint data become structured.

## Recommended Additions To RunHaven Backlog

1. `runhaven why`: provider-host explanation and validation guidance are
   implemented. Denied mounts, rejected workspace paths, and broader sensitive
   default explanations remain future work.
2. Provider proxy policy log: JSONL records for allowed and denied CONNECT
   decisions, with capped display in normal CLI output. Implemented
   2026-06-15.
3. Empty-allowlist regression tests for every network policy mode. Implemented
   2026-06-15.
4. Host-side provider credential broker design, tied to provider endpoint
   matrix entries. The Codex API-key broker prototype and secret-free broker
   diagnostics are implemented; broader provider brokers remain future work.
5. Worktree plus run-record feature set: create, list, log, diff, attach, stop,
   merge, keep, discard, recover. Implemented for current run-record,
   active-run, and worktree review flows on 2026-06-15.
6. Agent profile investigation docs and live macOS smoke tests per supported
   agent.
7. Devcontainer metadata import for image planning, with host lifecycle hooks
   disabled unless explicitly approved.
8. Extension/MCP boundary model: default off, explicit allowlists, scoped roots,
   timeouts, health checks, and no project-local weakening.
9. Warm reusable project sessions with explicit reset/prune UX. Implemented
   2026-06-15.
10. Source-backed docs drift checks for platform support, profiles, network
    modes, provider endpoints, and image templates.

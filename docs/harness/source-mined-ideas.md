# Source-Mined Product Ideas

Reviewed: 2026-06-15

Scope: manual review of sibling local checkouts `awman`, `aspec`, and `maki`.
AGY/Antigravity was not used for this pass.

RunHaven remains macOS 26+ only and Apple `container` only. The goal of this
review is not to keep the product small. The goal is to keep every useful idea
that can make RunHaven safer, clearer, more recoverable, or more capable,
then stage implementation by dependency and risk.

## Filters

Promote ideas when they improve:

- local-machine safety for non-technical users
- isolation and recovery around AI agent runs
- explicit network and provider boundaries
- restartability, observability, and auditability
- configuration clarity without hidden host access
- verification and documentation drift control

Reject or redesign ideas when they require:

- Windows or Linux runtime support
- Docker fallback behavior
- Apple `container machine` as the default agent boundary
- host home, cloud credentials, browser profiles, raw SSH keys, or Keychain
  access by default
- broad host mounts hidden behind convenience flags
- unreviewed host-side scripts for provider or extension behavior
- egress claims that are not enforced by code or live Apple `container`
  behavior

## High-Value Implementation Candidates

### Provider Endpoint Matrix

Sources:

- `maki:maki-providers/src/provider.rs`
- `maki:maki-providers/src/model.rs`
- `maki:site/docs/content/providers/_index.md`
- `awman:src/command/commands/status_tips.rs`

Idea: maintain provider metadata as structured data instead of scattering
provider assumptions through docs and code. For RunHaven this should track
provider, agent profile, required hosts, optional hosts, auth paths, telemetry
paths, package-install paths, source evidence, and last live smoke result.

Why it matters: provider egress friction becomes easier to explain and fix.
Non-technical users get a clearer error path when a host is blocked, while
RunHaven keeps fully qualified allowlists and avoids broad suffix mistakes.

Implementation shape:

- Add a reviewed provider endpoint ledger under `docs/` or `src/runhaven/`.
- Generate or validate profile docs from the same source.
- Require source evidence or a live smoke before changing bundled hosts.
- Keep user-added hosts explicit with `--provider-host`.

### Provider DNS And Private-Address Guard

Sources:

- `maki:plugins/webfetch/init.lua`
- `maki:maki-lua/src/api/net.rs`
- `src/runhaven/egress.py`

Idea: extend the provider CONNECT proxy so allowed hostnames are also checked
after DNS resolution. Reject loopback, link-local, private, multicast,
unspecified, and metadata-style addresses unless a future reviewed design has a
specific reason to allow them.

Why it matters: RunHaven already rejects IP literals and single-label hosts,
but DNS resolution can still point an allowed hostname at a sensitive address.
The proxy is the right host-side enforcement point.

Implementation shape:

- Resolve allowed CONNECT targets before opening the upstream socket.
- Reject unsafe address classes for IPv4, IPv6, and IPv4-mapped IPv6.
- Add focused unit tests in `tests/test_egress.py`.
- Add a live smoke variant only if it can be deterministic and low-friction.

### Worktree Isolation And Recovery

Sources:

- `awman:docs/05-workflows.md`
- `awman:src/engine/git/mod.rs`
- `awman:src/data/worktree_paths.rs`
- `awman:src/data/workflow_state_store.rs`

Idea: add an optional `runhaven run --worktree` mode that creates a temporary
branch and git worktree for an agent run, then offers merge, keep, or discard
after completion. Failed merges should leave the worktree and branch intact
with exact recovery commands.

Why it matters: this is one of the highest-value safeguards for AI coding
agents. It prevents accidental writes to the user's active checkout and gives
non-technical users a safer review point.

Implementation shape:

- Detect the repository root and dirty state before creating a worktree.
- Create RunHaven-owned worktrees under a predictable local state directory.
- Never silently broaden host mounts beyond the selected workspace.
- Persist run state so interrupted runs can resume cleanup or recovery.

### Workspace Scope Selection

Sources:

- `awman:src/command/commands/mount_scope.rs`
- `awman:src/engine/container/options.rs`
- `docs/SECURITY_MODEL.md`

Idea: make the workspace scope explicit when the current directory is inside a
git repository. Give users a clear choice between current directory, git root,
or abort.

Why it matters: AI tools often need repo root context, but silently expanding a
mount from a subdirectory to the full repository is a security and surprise
risk. RunHaven should ask, explain, and record the selected scope.

Implementation shape:

- Start with a non-interactive warning in `plan` and `run`.
- Add `--workspace-scope current|git-root`.
- Keep `current` as the safer default unless an interactive first-run flow
  intentionally asks.

### Run State, Observability, And Control

Sources:

- `awman:src/engine/container/apple.rs`
- `awman:src/engine/container/docker.rs`
- `maki:maki-storage/src/sessions.rs`
- `maki:maki-agent/src/headless.rs`

Idea: provide first-class run records and control commands: list active runs,
show run status, stop a run, inspect provider blocked-host events, and export
machine-readable results.

Why it matters: robust local agent execution needs recovery and visibility.
Users should not need to reverse-engineer Apple `container list` output or
search terminal scrollback to understand what happened.

Implementation shape:

- Persist RunHaven run metadata as append-only JSONL or per-run JSON.
- Add `runhaven runs list`, `runhaven runs show`, and `runhaven runs stop`.
- Parse Apple `container list --format json` defensively.
- Add `--json` output to `plan`, `state`, and future `runs` commands.

### Typed Run Options

Sources:

- `awman:src/engine/container/options.rs`
- `awman:src/command/dispatch/catalogue.rs`
- `maki:maki-agent/src/tools/registry.rs`

Idea: centralize RunHaven run options in typed structures with conflict,
implies, default, and documentation metadata.

Why it matters: provider mode, workspace scope, state volumes, TTY behavior,
profiles, egress policy, and future workflow flags will otherwise grow into
duplicated parser and planner logic.

Implementation shape:

- Keep `argparse`, but move validation into typed policy/planning objects.
- Add tests for invalid option combinations.
- Generate parts of help or docs from the same metadata only after the schema
  stabilizes.

### Strict Workflow Files

Sources:

- `awman:docs/05-workflows.md`
- `awman:src/data/workflow_definition.rs`
- `awman:src/data/workflow_dag.rs`
- `aspec:aspec/work-items/0000-template.md`

Idea: support project-local workflow definitions with strict schema validation,
setup and teardown steps inside the container, step dependencies, resume state,
and failure behavior.

Why it matters: a complete RunHaven should do more than wrap a one-off command.
Users will want repeatable agent tasks, preflight checks, test routing, and
post-run cleanup without granting extra host access.

Implementation shape:

- Start with strict JSON or TOML and reject unknown fields.
- Run setup, main, and teardown inside Apple `container`.
- Persist workflow hash and step state for resume.
- Keep host-side effects limited to RunHaven-owned state and selected
  workspace mounts.

### Context And Overlay Support

Sources:

- `awman:docs/08-overlays.md`
- `awman:src/engine/overlay/mod.rs`
- `maki:plugins/skill/init.lua`
- `maki:plugins/memory/init.lua`

Idea: provide explicit context overlays for read-only docs, skills, prompts, or
project memory that can be mounted into the container and referenced by the
agent.

Why it matters: richer context can improve agent output without mounting the
user's home directory or credentials.

Implementation shape:

- Only support project-local or RunHaven-managed overlay roots at first.
- Make read-only the default and let restrictive settings win on conflict.
- Treat missing named overlays as errors.
- Do not import host Keychain, SSH, browser, or cloud credential locations.

### MCP And Extension Boundaries

Sources:

- `maki:maki-agent/src/mcp/config.rs`
- `maki:maki-agent/src/mcp/http.rs`
- `maki:maki-lua/src/plugin_permissions.rs`
- `maki:site/docs/content/mcp/_index.md`

Idea: when RunHaven supports MCP or extensions, use deny-by-default manifests,
project/global scope separation, server-name validation, URL-bound auth, and
timeouts.

Why it matters: MCP can easily bypass the intended local boundary if host
tools, credentials, or network access are exposed casually.

Implementation shape:

- Default MCP off.
- Require explicit allowlists for servers, mounts, environment, and network.
- Run extension behavior inside the container where practical.
- Reject invalid or missing extension manifests instead of guessing.

### Generated Documentation Drift Checks

Sources:

- `maki:maki-docgen/src/main.rs`
- `maki:maki-docgen/src/gen_tools.rs`
- `maki:maki-docgen/src/gen_config.rs`
- `maki:maki-docgen/src/gen_commands.rs`
- `scripts/check_pins.py`

Idea: add a focused docs check that validates RunHaven docs against source
metadata for profiles, network modes, provider hosts, image tags, and supported
platform.

Why it matters: RunHaven's safety depends on docs matching actual behavior.
Provider hosts and network modes are easy to document incorrectly as the
product grows.

Implementation shape:

- Extend `scripts/check_pins.py` or add `scripts/check_docs.py`.
- Verify README and usage docs mention current profiles and network modes.
- Verify no Windows/Linux runtime support text returns.
- Verify provider docs match the structured endpoint matrix.

### Headless And Machine-Readable Output

Sources:

- `maki:maki-agent/src/headless.rs`
- `maki:site/docs/content/headless/_index.md`
- `awman:docs/09-api-mode.md`

Idea: add stable JSON output for automation and eventual headless orchestration.

Why it matters: RunHaven can stay beginner-friendly while also becoming
scriptable for advanced local workflows and CI-like checks on macOS.

Implementation shape:

- Add `--json` to read-only commands first.
- Use stable schemas with explicit version fields.
- Keep secrets and raw command output out of JSON by default.

## Ideas To Keep, But Stage Later

- Command catalogue shared by CLI, future TUI, and docs.
- API mode or local service mode with durable audit logs and workdir allowlists.
- Dynamic provider adapters, redesigned so scripts run inside the container or
  another constrained boundary.
- Code indexing support as an agent image capability, not a RunHaven host
  feature.
- Batch run orchestration with bounded output and per-child status.
- Project memory scoped to the selected workspace and RunHaven state, not host
  home directories.

## Ideas Not To Copy Directly

- Host Keychain extraction or automatic host credential injection.
- Raw SSH key, cloud credential, browser profile, or home-directory mounting by
  default.
- Docker daemon support or Docker runtime fallback.
- Windows or Linux runtime compatibility layers.
- `container machine` as the default execution boundary.
- Broad "yolo" modes unless guarded by worktree isolation, explicit workspace
  scope, and egress policy.
- Unbounded remote API mode without workdir allowlists and durable audit logs.

## Recommended Build Order

1. Harden provider proxy DNS resolution and unsafe-address rejection.
   Implemented 2026-06-15.
2. Add the provider endpoint matrix.
   Implemented 2026-06-15. Source-backed docs drift checks remain future work.
3. Add workspace scope detection and explicit `--workspace-scope`.
   Implemented 2026-06-15.
4. Add worktree isolation with merge, keep, discard, and recovery commands.
   `run --worktree` isolation and `runs merge`, `runs keep`, `runs recover`,
   `runs recover --json`, project check suggestions, and `runs discard`
   lifecycle commands implemented 2026-06-15.
5. Add run records and `runs list/show/stop`.
   Implemented 2026-06-15.
6. Add strict workflow files with container-only setup and teardown.
7. Add context overlays with read-only defaults.
8. Add MCP and extension support only after the boundary model is explicit.

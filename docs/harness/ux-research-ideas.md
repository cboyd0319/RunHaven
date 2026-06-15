# UX Research Ideas

Reviewed: 2026-06-15

Scope: research around making RunHaven easier and more understandable without
weakening its hard boundaries: macOS 26+ only, Apple `container` only,
no Windows/Linux runtime support, no Docker fallback, no default host home
mount, and no raw credential folder mounts.

This pass used manual web and source review. AGY/Antigravity was not used.

## Current RunHaven UX Baseline

RunHaven already has useful safety UX:

- `runhaven doctor` checks host prerequisites.
- `runhaven plan` shows workspace, state volume, network mode, egress status,
  preflight, and the Apple `container run` command.
- `runhaven plan` and `runhaven run` support named reusable project sessions
  with `--session NAME`.
- `runhaven run --network provider` reports blocked provider hosts after a run.
- `runhaven state list` and `runhaven state prune --yes` expose isolated agent
  home volumes.
- `runhaven state reset`, `state list --session NAME`, and
  `state prune --session NAME --yes` provide explicit cleanup paths for warm
  named sessions.
- The README explains the default boundary and warns that internet mode is
  unrestricted.

The main UX gap is that users still need to understand too much:

- which network mode to choose
- whether a blocked provider host is safe to add
- why a mount or path was rejected
- how to recover after a failed or interrupted agent run
- where agent state lives
- how to keep agent changes separate from their own work
- how to set up credentials without leaking them to the guest
- how to rebuild or repair a stale image

## Source-Backed UX Patterns

### Fewer Prompts Through Stronger Boundaries

Source:
https://www.anthropic.com/engineering/claude-code-sandboxing

Anthropic frames sandboxing as a way to reduce approval fatigue by defining
filesystem and network boundaries where the agent can work freely. The relevant
RunHaven lesson is that easier UX should come from stronger outer boundaries,
not from asking non-technical users to approve many low-context prompts.

RunHaven direction:

- Make provider egress, workspace scope, and worktree isolation good enough
  that normal runs need fewer decisions.
- Treat agent-native prompts as defense in depth, not as the main user safety
  mechanism.
- Prefer one high-quality pre-run trust checkpoint over many ambiguous
  mid-run prompts.

### Policy Logs For Debugging

Sources:
https://docs.docker.com/reference/cli/sbx/policy/log/
https://docs.docker.com/ai/sandboxes/governance/local/

Docker Sandboxes exposes policy logs that show allowed and blocked hosts, the
matching rule, proxy type, and request count, with JSON output. Docker also
uses network presets such as open, balanced, and locked down.

RunHaven direction:

- Add `runhaven egress log` or `runhaven runs log --network` for allowed and
  denied provider proxy decisions.
- Include host, port, decision, matched rule, count, profile, run id, and
  suggested next action.
- Add `--json` for automation and issue reports.
- Keep user-facing network choices goal-based: local only, provider only,
  package install, or unrestricted internet.

### Automatic Environment Suggestions

Source:
https://devpod.sh/docs/developing-in-workspaces/devcontainer-json

DevPod can use `devcontainer.json`; when no configuration exists, it attempts
language detection and provides a default configuration.

RunHaven direction:

- Add `runhaven init` or `runhaven recommend` to inspect a project and suggest
  an agent image, workspace scope, network mode, and test command.
- Detect `.devcontainer/devcontainer.json`, `.devcontainer.json`, package
  manifests, and common test commands.
- Do not run host-side devcontainer lifecycle hooks unless explicitly approved.
- Render the recommendation as a plan the user can inspect before running.

### Git Undo And Dirty-Work Separation

Sources:
https://aider.chat/docs/git.html
https://aider.chat/docs/usage/lint-test.html

Aider uses git to make AI changes easy to undo and review, separates existing
dirty work from AI edits, and can run lint/test commands after changes.

RunHaven direction:

- Add worktree mode before high-autonomy UX.
- Detect dirty work before starting and explain the choices: continue in-place,
  create a worktree, or abort.
- Add post-run review commands: `runs diff`, `runs keep`, `runs recover`,
  `runs discard`, `runs merge`.
- Let profiles or project config suggest lint/test commands that run inside the
  container after the agent finishes.

### One Command To Start A UI Or Guided Flow

Sources:
https://docs.openhands.dev/openhands/usage/run-openhands/gui-mode
https://docs.openhands.dev/openhands/usage/run-openhands/local-setup

OpenHands can launch a local GUI server from the CLI and uses an initial
settings flow for provider/model/API-key setup.

RunHaven direction:

- Keep CLI first, but add `runhaven setup` as a guided first-run path.
- Later, consider `runhaven serve` for a local-only browser UI showing doctor,
  image status, runs, blocked hosts, state volumes, and recovery actions.
- Keep the same planner and policy layer underneath any future UI.

### Zero-Setup Defaults And Profile Scaffolding

Source:
https://github.com/always-further/nono

nono emphasizes zero setup, agent profiles, registry/scaffolding, and hiding
secrets outside the current working directory.

RunHaven direction:

- Add `runhaven profile init <name>` after custom profile schema exists.
- Add `runhaven profile doctor <name>` to explain required hosts, auth state,
  image status, and likely blocked operations.
- Consider curated profile packs only with exact pins, provenance, and review.

### Practical Least Privilege

Source:
https://agent-safehouse.dev/docs/overview

Agent Safehouse emphasizes practical damage reduction: start deny-first, allow
what the agent needs, and avoid claiming perfect isolation.

RunHaven direction:

- Keep warning text practical and specific.
- Avoid scary or abstract copy; say exactly what is mounted, what can reach the
  network, and what remains risky.
- Prefer commands that answer "what can this agent touch?" over long security
  essays.

### Apple Container Machine Friction Report

Source:
https://rkiselenko.dev/blog/development-on-mac-with-acm/

This 2026-06-14 field report sets up a persistent Apple Container Machine for
Go, SSH-based Zed remote editing, and a coding agent. The useful RunHaven
signal is the friction, not the default architecture: DNS setup is cumbersome,
first-boot user creation complicates provisioning scripts, and Apple Container
Machine mounts the host user home into the Linux environment with only broad
`rw`, `ro`, or `none` mount options. The report also shows why users want this
shape: fast reproducible Linux tooling, host-service access from the guest,
remote editor access, and persistent tool state.

RunHaven direction:

- Keep `container machine` out of the default beginner-safe agent path because
  broad host-home mapping conflicts with RunHaven's project-scoped mount
  boundary.
- Add clearer setup/docs language explaining why RunHaven uses task-scoped
  `container run` instead of persistent machine defaults.
- Consider future diagnostics for host-service access from a guest, such as
  `runhaven why network host-service` or `runhaven doctor dns`, without
  claiming DNS is an egress-control boundary.
- Treat remote-editor and persistent-dev-environment workflows as explicit
  advanced modes only after workspace, SSH, and credential boundaries are
  verified.
- Support project-local bootstrap or recommendation scripts through
  inspect-before-run plans; do not run first-boot or host-mounted home scripts
  implicitly.

## UX Improvements To Add

### Guided First Run

Command shape:

```bash
runhaven setup
```

The wizard should:

- run `doctor`
- confirm Apple `container` is started or show the exact command
- ask which agent profile to use
- inspect the current project and recommend image/profile/test command
- explain workspace scope and ask current directory versus git root when needed
- ask for a network goal: local only, provider only, package install, or
  unrestricted internet
- show credential options: authenticate inside isolated state, pass one env var
  by name, or use future credential broker
- build or verify the image
- write no project config unless the user explicitly accepts
- end by printing the exact `runhaven plan` command

### Goal-Based Network Selection

Replace jargon-first UX with goal-first guidance:

- "No internet, local tests only" maps to `--network internal`.
- "Talk only to my AI provider" maps to `--network provider`.
- "Install packages for setup" maps to a future package/install profile.
- "Allow anything online" maps to `--network internet` with plain warning.

This can start as improved `plan` output and later become an interactive
selection in `runhaven setup`.

### Explain Commands

Command shapes:

```bash
runhaven why host api.example.com
runhaven why workspace PATH
runhaven why network provider
runhaven why state claude
```

The output should answer:

- what happened
- which rule made the decision
- why the default is safer
- what command would intentionally override it, if an override exists
- what risk the override creates

### Provider Blocked-Host Review

After provider runs, turn blocked hosts into an actionable review:

- grouped by host and port
- count and first/last time
- matched profile and run id
- whether the host is already known in the provider endpoint matrix
- suggested command only for fully qualified hostnames
- no suggestion for IP literals or unsafe DNS results

### Run Dashboard In The CLI

Command shapes:

```bash
runhaven runs list
runhaven runs show <id>
runhaven runs log <id>
runhaven runs diff <id>
runhaven runs attach <id>
runhaven runs stop <id>
```

This makes interrupted or background runs recoverable without asking users to
inspect Apple `container` internals.

### Worktree Review Flow

Command shape:

```bash
runhaven run claude --worktree
```

Post-run commands:

```bash
runhaven runs diff <id>
runhaven runs test <id>
runhaven runs recover <id>
runhaven runs merge <id>
runhaven runs keep <id>
runhaven runs discard <id>
```

UX requirements:

- never delete a worktree containing unmerged work without explicit
  confirmation
- keep exact recovery commands in the run record
- explain dirty-start choices before the run begins
- use normal git concepts, but do not require the user to remember git commands

Current state: `runhaven run AGENT --worktree` creates a RunHaven-owned branch
and git worktree for clean source repositories, keeps it after the run, and
records exact recovery commands. Dirty source checkouts fail before worktree
creation with choices to commit or stash, run without `--worktree`, or start
from a clean clone or git worktree. `runs keep`, `runs recover`,
`runs recover --json`, `runs merge`, and `runs discard` now provide the first
guarded review lifecycle: keep validates and prints review paths, recover
prints source and worktree status plus structured JSON without mutation, keep
and recover suggest detected project checks that run through internal-network
shell commands, merge validates the source/worktree/branch boundary before
applying changes back to the source checkout, and discard removes only the
recorded RunHaven worktree and branch.

### Image Repair And State Repair

Current command shapes:

```bash
runhaven image doctor
runhaven image doctor claude
runhaven image rebuild claude
runhaven state list
runhaven state reset claude --session review --yes
runhaven state prune --yes
runhaven network list
runhaven network prune --yes
```

Current state: `runhaven image doctor` checks local Apple `container` image
metadata for missing or stale bundled images, compares RunHaven source-digest
labels when present, uses timestamp fallback for older unlabeled images, and
prints inactive state-volume review plus rebuild, network, and state recovery
guidance without mutating resources. `runhaven image rebuild` reuses the
pinned bundled image build plan with clearer repair intent. `state` commands
list, reset, and prune RunHaven-owned agent home volumes. `network` commands
list and prune only RunHaven-managed Apple `container` networks after explicit
confirmation. Future work can make the state-volume review workspace-aware,
but the current commands avoid deleting workspace files.

### Credential UX

Current state: `runhaven auth status` and `runhaven auth explain AGENT`
provide static, secret-free broker boundary diagnostics. They describe expected
host-side key names, broker support, provider hosts, and known limitations
without reading credential stores or environment values.

Near-term:

- prefer authenticating inside isolated agent home state
- make `--env NAME` output clearer about what becomes visible inside the guest

Future:

- broaden host-side provider credential brokers beyond the Codex prototype only
  after provider-specific design and evidence
- scoped provider tokens or placeholders inside the container
- broker tied to provider endpoint matrix and proxy allowlist

### Documentation And Help Copy

Docs should answer user questions in task language:

- "I want the agent to review code without changing files."
- "I want the agent to run tests without internet."
- "I want the agent to use Claude/OpenAI/Gemini but nothing else online."
- "The agent says a host is blocked. What now?"
- "I want to undo everything the agent did."
- "I want to delete the agent's memory/state."

Add these as README or usage recipes before adding a larger UI.

## Recommended UX Build Order

1. `runhaven why` for provider-host decisions is implemented; workspace,
   network, and state explanations remain future work.
2. Provider policy logs with JSON output and grouped blocked-host review are
   implemented.
3. Goal-based network copy in `plan`, `run --help`, README, and `docs/USAGE.md`.
4. `runhaven setup` guided first run.
5. `runhaven runs list/show/log` backed by durable run records is implemented.
6. Worktree mode with diff, merge, keep, discard, and recovery commands is
   implemented.
7. Image/state/network repair commands are implemented; future work can make
   state-volume review workspace-aware.
8. Project recommendation/import from manifests and devcontainer metadata.
9. Auth status and Codex broker prototype are implemented; broader provider
   brokers remain future work.
10. Local-only browser UI after the CLI planner and run records stabilize.

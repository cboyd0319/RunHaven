# Roadmap

## Phase 1: Safe Local Baseline

- Python 3.13+ package and CLI
- agent profiles
- dry-run planning
- bundled image templates
- doctor checks
- unit tests for command boundaries

## Phase 2: Network Boundary

- provider network mode integrated into normal `runhaven run`
- live smoke harness for host allowlist proxy on an internal network
- provider-specific egress profiles
- reviewed provider endpoint matrix for auth, provider routing, telemetry,
  package, and optional feature paths
- DNS resolution and unsafe-address rejection inside the provider CONNECT proxy
- provider proxy policy logs for allowed and denied CONNECT decisions
- first `runhaven why host ...` diagnostic for provider-host decisions
- grouped blocked-host review with rule, count, run id, and suggested next
  action
- provider-profile smoke support for bundled source-backed hosts
- host-side provider credential broker design with `runhaven auth status` and
  `runhaven auth explain AGENT`
- empty-allowlist regression tests for every network policy mode
- first real host-side provider credential broker implementation
- local proxy option for model credentials
- clear offline and package-install network modes
- live provider auth-flow smokes for optional feature paths
- path-aware provider policy for broad hosts such as `github.com`

## Phase 3: Beginner Install Flow

- signed release artifacts
- one-command bootstrap for Apple `container`
- guided first-run setup
- plain-language explanations for every requested permission
- workspace scope detection with explicit current-directory versus git-root
  selection
- `runhaven why` diagnostics for blocked hosts, rejected mounts, sensitive
  defaults, and validation failures
- guided `runhaven setup` first-run flow
- goal-based network selection copy for local-only, provider-only, package
  install, and unrestricted internet use cases

## Phase 4: Agent Coverage

- custom profile file support
- per-agent policy presets
- MCP allowlists
- import/export of project profiles
- generated docs checks for profiles, network modes, provider hosts, pins, and
  macOS 26+ support text
- agent profile investigation docs for required files, network hosts, update
  paths, auth paths, and known blocked operations
- devcontainer metadata import for image planning, with host lifecycle hooks
  disabled unless explicitly approved
- task-language usage recipes for review-only, local tests, provider-only,
  undo, and state reset workflows

## Phase 5: Isolation And Recovery

- optional git worktree isolation for agent runs
- merge, keep, discard, and conflict-recovery flows
- run records with `runs list`, `runs show`, and `runs stop`
- `runs log`, `runs diff`, and `runs attach` for visibility and direct
  intervention
- structured blocked-host and cleanup event records
- stable JSON output for read-only and run-status commands
- warm reusable project sessions with explicit reset and prune UX
- image, state, and managed-network repair commands
- provider auth broker run records that never print secrets

## Phase 6: Repeatable Workflows

- strict project-local workflow files
- container-only setup, main, and teardown steps
- workflow state, resume, and failure policy
- read-only context overlays for docs, skills, prompts, and project memory
- deny-by-default extension and MCP boundaries
- source-of-truth planner and policy objects reusable by future CLI, API, or
  GUI surfaces

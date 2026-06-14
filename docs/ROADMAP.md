# Roadmap

## Phase 1: Safe Local Baseline

- Python 3.13+ package and CLI
- agent profiles
- dry-run planning
- bundled image templates
- doctor checks
- unit tests for command boundaries

## Phase 2: Network Boundary

- reserved provider network mode that fails closed until normal run integration lands
- live smoke harness for host allowlist proxy on an internal network
- provider-specific egress profiles
- local proxy option for model credentials
- clear offline, provider-only, and package-install network modes
- tests for generated firewall or proxy configuration

## Phase 3: Beginner Install Flow

- signed release artifacts
- one-command bootstrap for Apple `container`
- guided first-run setup
- plain-language explanations for every requested permission

## Phase 4: Agent Coverage

- custom profile file support
- per-agent policy presets
- MCP allowlists
- import/export of project profiles

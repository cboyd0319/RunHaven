# Auth Broker

Status: Codex API-key broker prototype. Other agent auth brokers remain design
only.

RunHaven exposes two auth inspection commands:

```bash
runhaven auth status
runhaven auth explain codex
```

These commands are intentionally safe to run. They do not inspect Keychain,
browser profiles, cloud credential files, provider login caches, or environment
values. They print broker status and profile-specific guidance only.

## Boundary

Trusted:

- the user
- the RunHaven host process
- explicit user approval for a brokered auth action

Untrusted or partially trusted:

- repository contents
- the selected agent CLI
- model output
- package install scripts
- MCP servers or extensions
- code running inside the Apple `container` guest

Sensitive:

- provider API keys and OAuth tokens
- local provider login caches
- browser profiles and cookies
- Keychain items
- Google Cloud ADC and service account JSON files
- GitHub, Copilot, OpenAI, Anthropic, Gemini, and Claude credentials

## Codex API-Key Broker

Codex can use an explicit host-side API-key broker:

```bash
runhaven run codex --network provider --codex-api-key-broker-env OPENAI_API_KEY
```

Current behavior:

- no host credential store is read
- the named host environment variable is read only for a real run, never for
  `plan`, `run --dry-run`, `auth status`, or `auth explain`
- the raw API key is not copied into the guest environment or container command
- the guest receives a placeholder `RUNHAVEN_CODEX_BROKER_TOKEN` value and
  temporary Codex custom-provider overrides pointing at the broker
- the broker binds to the RunHaven provider network, restricts clients to that
  Apple `container` subnet, and forwards only Codex Responses API create
  requests to `api.openai.com`
- after a brokered run, `runhaven auth log` records only method, sanitized path,
  decision, reason, upstream status, count, and run id
- no token is printed in plan, status, JSON, or diagnostic output
- no token value, request body, or environment variable name is stored in the
  auth broker log
- interactive login inside the isolated agent home volume remains available
  when the agent supports it

Other providers remain design-only. `--env NAME` is still available as an
explicit fallback when a user deliberately wants a token value inside the guest,
but it is not the preferred Codex headless path.

Provider egress controls are separate. `--network provider` limits CONNECT
targets by host, records policy decisions, and groups blocked-host reviews.
It does not authenticate to the provider and it does not see HTTPS URL paths.

## Smoke Coverage

`scripts/codex_broker_smoke.py` can run a real non-interactive Codex request
through the broker when a disposable OpenAI API key is available:

```bash
export RUNHAVEN_CODEX_BROKER_SMOKE_API_KEY=...
PYTHONPATH=src python3.14 scripts/codex_broker_smoke.py --require-api-key
```

If the environment variable is absent and `--require-api-key` is not passed, the
script prints `SKIP` and exits successfully. The key value is inherited by the
host process only; it is not placed on the command line.

## Why Host-Side

A plain HTTPS CONNECT proxy sees the destination host and port, not the request
path inside the TLS stream. RunHaven should not intercept provider TLS by
default just to learn paths.

For broad path-sensitive hosts such as `github.com` and `api.github.com`, the
safer long-term pattern is a provider-specific host-side broker:

- the host owns the sensitive provider credential
- the guest asks for a narrow provider action or short-lived run credential
- RunHaven audits the request and fails closed when the policy is not explicit
- the guest does not receive broad host credentials by default

## Provider Notes

Current source-backed auth surfaces:

- Codex supports ChatGPT sign-in, OpenAI API-key sign-in, trusted access tokens
  for some automation, custom model providers, command-line configuration
  overrides, and the Responses API. The RunHaven prototype uses only the
  API-key plus custom-provider path.
- Claude Code supports Claude.ai credentials, Claude API credentials, cloud
  provider auth, API key or bearer-token environment variables, and
  `apiKeyHelper`.
- Gemini CLI supports Google login, Gemini API keys, and Vertex AI auth through
  ADC, service account JSON, or Google Cloud API keys.
- Copilot CLI supports OAuth device login, environment-token auth, GitHub CLI
  fallback, and BYOK provider environment variables.
- Antigravity auth and minimal runtime endpoint sources remain incomplete, so
  no broker behavior is planned for it until official sources are reviewed.

The reviewed source links are recorded in
[`RESEARCH.md`](RESEARCH.md#agent-runtime-sources).

## Non-Goals

The broker design does not allow:

- automatic Keychain extraction
- browser profile or cookie reads
- mounting `~/.config`, cloud credential folders, SSH material, or the macOS
  home directory
- copying Google ADC or service account JSON files into the guest by default
- implicit `--env` passthrough
- printing token values or credential file contents
- TLS interception as the default provider egress model
- treating broad GitHub hosts as safe merely because Copilot uses them

## Remaining Acceptance Criteria

Before this can become a default path, or before another provider gets a real
broker, the implementation must satisfy all relevant criteria:

- explicit user opt-in for each provider account or credential source
- provider-specific policy tied to the endpoint matrix
- no real secret values in logs, plans, status output, JSON, exceptions, or
  tests
- least-privilege token or action scope
- clear expiry and revocation behavior
- run records that show what provider action was brokered without exposing the
  credential
- focused regression tests proving secret values are not printed
- live smoke coverage for the selected provider flow on macOS 26+ with Apple
  `container`
- failure mode that leaves the guest unauthenticated instead of widening the
  boundary

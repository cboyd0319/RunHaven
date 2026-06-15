# Auth Broker

Status: design only.

RunHaven exposes two auth inspection commands:

```bash
runhaven auth status
runhaven auth explain codex
```

These commands are intentionally safe to run. They do not inspect Keychain,
browser profiles, cloud credential files, provider login caches, or environment
values. They print static broker status and profile-specific guidance only.

## Boundary

Trusted:

- the user
- the RunHaven host process
- explicit user approval for a future brokered auth action

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

## Current Behavior

The auth broker is not implemented yet. Current RunHaven behavior is:

- no host credential store is read
- no environment variable value is inspected
- no provider credential is copied into the guest by RunHaven
- no token is printed in plan, status, JSON, or diagnostic output
- `--env NAME` remains an explicit fallback for deliberate headless runs
- interactive login inside the isolated agent home volume remains available
  when the agent supports it

Provider egress controls are separate. `--network provider` limits CONNECT
targets by host, records policy decisions, and groups blocked-host reviews.
It does not authenticate to the provider and it does not see HTTPS URL paths.

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

- Codex supports ChatGPT sign-in, OpenAI API-key sign-in, and trusted access
  tokens for some automation. Codex login details may be cached locally.
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

## Future Acceptance Criteria

A real broker implementation must satisfy all of these before becoming a
default path:

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

# Auth Broker

Status: host-side API-key broker for Codex, Claude, and Gemini. Copilot and
Antigravity remain design-only.

RunHaven exposes two auth inspection commands:

```bash
runhaven auth status
runhaven auth explain claude
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

## API-Key Broker

Codex, Claude, and Gemini support an explicit host-side API-key broker. The
host environment variable is named once; the real key stays on the host and the
guest is redirected at the broker:

```bash
runhaven run codex  --network provider --api-key-broker-env OPENAI_API_KEY
runhaven run claude --network provider --api-key-broker-env ANTHROPIC_API_KEY
runhaven run gemini --network provider --api-key-broker-env GEMINI_API_KEY
```

`--api-key-broker-env` resolves the broker from the selected profile. (The old
`--codex-api-key-broker-env` name still works as an alias.)

Per provider, the broker pins one upstream host, injects the real credential
host-side, and points the guest at the broker:

| Provider | Upstream host | Credential injected host-side | Guest redirect |
| --- | --- | --- | --- |
| Codex | `api.openai.com` | `Authorization: Bearer` | Codex custom-provider config (`/v1`) |
| Claude | `api.anthropic.com` | `x-api-key` + `anthropic-version` | `ANTHROPIC_BASE_URL` |
| Gemini | `generativelanguage.googleapis.com` | `x-goog-api-key` | `GOOGLE_GEMINI_BASE_URL` |

Current behavior, for every brokered provider:

- no host credential store is read
- the named host environment variable is read only for a real run, never for
  `plan`, `run --dry-run`, `auth status`, or `auth explain`
- the raw API key is not copied into the guest environment or container command
- the guest receives only a placeholder key value plus the broker base-URL
  redirect above
- the broker binds to the RunHaven provider network, restricts clients to that
  Apple `container` subnet, and forwards only the provider's allowed request
  paths to the pinned upstream host
- the broker strips any guest-sent copy of the injected credential headers
  before injecting the real value
- after a brokered run, `runhaven auth log` records only method, sanitized path,
  decision, reason, upstream status, count, and run id
- no token is printed in plan, status, JSON, or diagnostic output, and no token
  value, request body, or environment variable name is stored in the log

`--env NAME` is still available as an explicit fallback when a user deliberately
wants a token value inside the guest, but it is not the preferred headless path.

Provider egress controls are separate. `--network provider` limits CONNECT
targets by host, records policy decisions, and groups blocked-host reviews. It
does not authenticate to the provider and it does not see HTTPS URL paths.

## OAuth And Subscription Logins

The broker is for API keys only. If you sign in with OAuth or a subscription
(Claude.ai or Claude Pro/Max, ChatGPT sign-in for Codex, a Google account for
Gemini, GitHub OAuth for Copilot), the broker does not apply.

RunHaven deliberately does not mount or read your host login state:
`~/.claude.json`, the macOS Keychain, browser profiles, and cloud credential
files stay on the host. For an OAuth or subscription agent, authenticate once
inside the container's persistent state volume (`/home/agent`, per profile,
workspace, and session). The tokens live in that isolated volume and later runs
reuse it; your host login is never touched. For this to work, the provider
allowlist must permit the OAuth login and token-refresh hosts, not only the API
host.

A host-side OAuth-token broker was researched (2026-06-26) and decided against
for every provider, for three independent reasons:

- Provider terms forbid it. Anthropic, OpenAI, Google, and GitHub all prohibit
  relaying subscription or OAuth credentials through third-party tools, and
  several actively detect and block it.
- The subscription token is not a drop-in bearer. Unlike an API key, an
  OAuth/subscription token targets a different host (ChatGPT and Google login
  route to separate endpoints) or requires impersonating the official client
  (Claude's subscription token is rejected on the API path without
  client-specific headers and an identity system prompt).
- It would cross the boundary. A broker would have to read your host credential
  store or impersonate the official client, which RunHaven does not do.

So OAuth and subscription logins stay on the isolated-in-container path; the
work there is allowlisting each provider's login and token-refresh hosts and a
smooth headless login, not a broker. Tracked in
[`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md).

## Smoke Coverage

Codex broker behavior is live-verified on macOS 26+ with Apple `container` using
a disposable OpenAI API key:

```bash
export RUNHAVEN_CODEX_SMOKE_API_KEY=...
runhaven run codex --network provider \
  --api-key-broker-env RUNHAVEN_CODEX_SMOKE_API_KEY -- \
  codex --version
```

The Claude and Gemini broker paths are covered by unit tests (credential
injection, path matching, placeholder-only guest config). Their live redirect
(`ANTHROPIC_BASE_URL` over the broker, Gemini's currently undocumented
`GOOGLE_GEMINI_BASE_URL`) must be confirmed with a real provider API key on the
target CLI version before being treated as proven. The key value is inherited by
the host process only; it is never placed on the command line or inside the
guest environment.

## Why Host-Side

A plain HTTPS CONNECT proxy sees the destination host and port, not the request
path inside the TLS stream. RunHaven should not intercept provider TLS by
default just to learn paths. A host-side broker keeps the model clean:

- the host owns the sensitive provider credential
- the guest asks for a narrow provider action through a pinned host and path
- RunHaven audits the request and fails closed when the policy is not explicit
- the guest does not receive a usable credential

## Provider Notes

- Codex supports ChatGPT sign-in, OpenAI API-key sign-in, trusted access tokens,
  custom model providers, and the Responses API. The broker uses the API-key
  plus custom-provider path (`/v1/responses`).
- Claude Code supports Claude.ai or subscription OAuth, the Anthropic API key,
  cloud provider auth, and `apiKeyHelper`. The broker uses the API-key path:
  `x-api-key` to `api.anthropic.com` with an `ANTHROPIC_BASE_URL` redirect.
  OAuth and subscription logins use isolated in-container state.
- Gemini CLI supports Google login, the Gemini API key, and Vertex AI auth. The
  broker uses the Gemini API-key path: `x-goog-api-key` to
  `generativelanguage.googleapis.com` with a base-URL redirect. The redirect env
  is currently undocumented upstream and version-fragile; re-verify per CLI
  version. Vertex AI and Google account login are not brokered.
- Copilot CLI is design-only and not brokered. The Copilot API host is bounded
  (`*.githubcopilot.com`) and could be pinned, so this is not a hard technical
  block, but brokering is still the wrong call: GitHub's terms forbid proxying
  Copilot, the credential is a GitHub OAuth token exchanged at an undocumented
  endpoint, and a broker would have to read the host token store. Use isolated
  in-container login state; the GitHub device flow works headlessly.
- Antigravity auth and runtime endpoint sources remain incomplete, so no broker
  behavior is planned until official sources are reviewed.

The reviewed source links are recorded in
[`RESEARCH.md`](RESEARCH.md#agent-runtime-sources).

## Non-Goals

The broker design does not allow:

- automatic Keychain extraction
- browser profile or cookie reads
- mounting `~/.config`, cloud credential folders, SSH material, or the macOS
  home directory
- reading host login state such as `~/.claude.json` to broker OAuth tokens
- copying Google ADC or service account JSON files into the guest by default
- implicit `--env` passthrough
- printing token values or credential file contents
- TLS interception as the default provider egress model
- treating broad GitHub hosts as safe merely because Copilot uses them

## Remaining Acceptance Criteria

Codex, Claude, and Gemini brokers exist today. Before another provider gets a
real broker, or before any broker becomes a default path, the implementation
must satisfy all relevant criteria:

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

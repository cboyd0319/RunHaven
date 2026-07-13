# Provider Endpoint Matrix

Reviewed: 2026-06-26

RunHaven runs only on macOS 26+ through Apple `container`. Provider mode is a
host-level egress control for local agent runs. It is not a guarantee that a
provider host is safe for every operation, and it cannot yet restrict traffic to
individual URL paths.

This matrix is the source review behind bundled provider hosts and the
`runhaven why host` command. A bundled host is allowed for the selected profile
in `--network provider`. Candidate, optional, and build hosts are not bundled by
default; add them with `--provider-host` only when the blocked operation matches
the documented purpose. RunHaven is primarily for less-technical users, so the
goal is that the secure default just works and the user manages no hosts and
sees no hostnames in normal use.

## Default Policy

- Allow source-backed model, auth, and provider routing hosts that are narrow
  enough for host-level policy.
- Keep telemetry, reporting, release-note, update, plugin marketplace, and broad
  path-sensitive hosts explicit.
- Do not bundle hosts that are backed only by package strings or public issue
  reports without an official allowlist or a live RunHaven smoke.
- `github.com` and `api.github.com` are now bundled for Copilot to carry the
  `runhaven login copilot` device flow and token exchange. This is a deliberate
  egress widening because RunHaven cannot yet path-restrict those GitHub hosts.
- Antigravity now has a source-backed bundled host set, observed from live egress
  during `runhaven login antigravity`. It defaults to `provider` mode like the
  other bundled profiles.
- This stays an allowlist with default-deny. Data-egress hosts such as
  `storage.googleapis.com` are simply never in an allow-pattern.

## Domain-Family Patterns

The allowlist matcher accepts maintainer-curated domain-family wildcard patterns
such as `*-cloudcode-pa.googleapis.com`. The wildcard can only expand a single
subdomain label inside one registrable domain: a pattern must start with `-` or
`.` and carry a tail with at least two dots, so `*-foo.com` is rejected at
construction. A family pattern absorbs provider channel and region churn (for
example `daily-`, `us-`, `eu-` prefixes) without a re-pin, while staying inside
the registrable domain so sibling services like `storage.googleapis.com` remain
denied. Patterns are curated by maintainers, not entered by users.

## Bundled Hosts

| Profile | Bundled hosts | Purpose | Evidence |
| --- | --- | --- | --- |
| `claude` | `api.anthropic.com`, `claude.ai`, `platform.claude.com` | Claude API requests, WebFetch domain safety checks, Claude account auth, and Anthropic Console auth. | Anthropic Claude Code network configuration. |
| `codex` | `api.openai.com`, `chatgpt.com`, `auth.openai.com` | OpenAI API traffic, Codex network-policy examples, ChatGPT sign-in, Codex web surface, the standalone installer host, and Codex device-code login plus token refresh (`auth.openai.com`). | OpenAI Codex auth, CLI, approvals/security, and permissions docs. |
| `antigravity` | `oauth2.googleapis.com`, `www.googleapis.com`, `cloudcode-pa.googleapis.com`, `*-cloudcode-pa.googleapis.com` | Google OAuth token exchange and refresh, Google userinfo, and the Cloud Code backend and model-endpoint family used by the `agy` login and runtime. The `*-cloudcode-pa.googleapis.com` family covers the channel/region prefix the model call uses. | RunHaven egress ledger, observed live 2026-06-26. |
| `copilot` | `api.githubcopilot.com`, `individual.githubcopilot.com`, `business.githubcopilot.com`, `enterprise.githubcopilot.com`, `githubcopilot.com`, `copilot-proxy.githubusercontent.com`, `origin-tracker.githubusercontent.com`, `github.com`, `api.github.com` | Copilot suggestion API and subscription-based Copilot routing, plus the GitHub device-flow login (`github.com`) and token exchange (`api.github.com`) for `runhaven login copilot`. | GitHub Copilot allowlist and subscription-routing docs. |

RunHaven host rules match a listed host and its subdomains. For example,
`business.githubcopilot.com` also permits
`api.business.githubcopilot.com`. A `*`-prefixed entry is a domain-family pattern
(see [Domain-Family Patterns](#domain-family-patterns)).

The browser-side Google sign-in (`accounts.google.com`) and the Antigravity
redirect (`antigravity.google`) happen in the host browser during
`runhaven login antigravity`, not inside the guest, so neither is bundled.

## Explicit Review Hosts

| Profile | Host | Status | Purpose | Why not bundled |
| --- | --- | --- | --- | --- |
| `claude` | `downloads.claude.ai` | optional | Claude Code plugin executable downloads, native installer, and native updater. | RunHaven installs pinned npm packages into images. Runtime updater hosts should stay explicit. |
| `claude` | `raw.githubusercontent.com` | optional | Changelog feed, release notes, and plugin marketplace install counts. | GitHub raw content is broader than model/auth traffic. |
| `claude` | `bridge.claudeusercontent.com` | optional | Claude in Chrome extension bridge. | Not needed for normal CLI runs. |
| `antigravity` | `storage.googleapis.com` | build | Pinned Antigravity CLI archive download during image build. | Build-time only in the current image template; also a data-egress host kept out of the runtime allowlist. |
| `antigravity` | `accounts.google.com` | browser-side | Google account sign-in consent during `runhaven login antigravity`. | The consent runs in the host browser, not the guest, so it is not needed inside the container. |
| `antigravity` | `antigravity.google` | browser-side | OAuth redirect target during `runhaven login antigravity`. | The redirect lands in the host browser, not the guest. |
| `antigravity` | `lh3.googleusercontent.com` | optional | Profile picture for the `agy` eligibility check. | Cosmetic; a blocked fetch only prints an "Eligibility check failed" line and the agent still works. Add with `--provider-host` to silence it. |
| `antigravity` | `antigravity-unleash.goog` | optional | Feature-flag checks. | Live sign-in and the authenticated `agy models` readiness check succeeded while this host stayed denied. |
| `antigravity` | `play.googleapis.com` | optional | Client telemetry. | Telemetry is not required for sign-in or model readiness. |
| `antigravity` | `playwright*.azureedge.net` | optional | Browser binary downloads for optional browser tooling. | RunHaven does not download browser binaries during agent sign-in or the readiness check. |
| `copilot` | `collector.github.com` | optional | GitHub analytics telemetry. | Telemetry is not bundled. |
| `copilot` | `copilot-telemetry.githubusercontent.com` | optional | Copilot client telemetry. | Telemetry is not bundled. |
| `copilot` | `default.exp-tas.com` | optional | Copilot client experimentation. | Experimentation is not required for a secure default. |

## How To Review A Blocked Host

1. Run `runhaven why host HOST --agent AGENT`.
2. If the host is bundled, the run should already allow it and DNS safety will
   be checked at runtime.
3. If the host is a known candidate, add it only when the documented purpose
   matches the action you are trying to unblock.
4. If the host is unknown, find vendor documentation or run a contained smoke
   before adding it.
5. Prefer API-key or access-token flows for headless provider mode instead of
   browser sign-in flows that require broad web auth hosts.

## Direction

The design intent is to trust each agent's own provider as a few stable
domain-family patterns rather than individual hosts, and eventually ship the
per-agent policy as signed, auto-updating data so new provider endpoints need no
release and no user action. The matcher's domain-family patterns are the first
step; the signed auto-updating policy is not built yet.

## Source Notes

- Anthropic Claude Code network configuration:
  <https://code.claude.com/docs/en/corporate-proxy>
- OpenAI Codex authentication:
  <https://developers.openai.com/codex/auth>
- OpenAI Codex CLI:
  <https://developers.openai.com/codex/cli>
- OpenAI Codex approvals and network policy:
  <https://developers.openai.com/codex/agent-approvals-security>
- OpenAI Codex permissions:
  <https://developers.openai.com/codex/permissions>
- GitHub Copilot allowlist reference:
  <https://docs.github.com/en/copilot/reference/copilot-allowlist-reference>
- GitHub Copilot subscription routing:
  <https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access>
- Google Developers Blog, Gemini CLI to Antigravity CLI transition:
  <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>

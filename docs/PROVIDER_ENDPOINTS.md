# Provider Endpoint Matrix

Reviewed: 2026-06-15

RunHaven runs only on macOS 26+ through Apple `container`. Provider mode is a
host-level egress control for local agent runs. It is not a guarantee that a
provider host is safe for every operation, and it cannot yet restrict traffic to
individual URL paths.

This matrix is the source review behind bundled provider hosts and the
`runhaven why host` command. A bundled host is allowed for the selected profile
in `--network provider`. Candidate, optional, and build hosts are not bundled by
default; add them with `--provider-host` only when the blocked operation matches
the documented purpose.

## Default Policy

- Allow source-backed model, auth, and provider routing hosts that are narrow
  enough for host-level policy.
- Keep telemetry, reporting, release-note, update, plugin marketplace, and broad
  path-sensitive hosts explicit.
- Do not bundle hosts that are backed only by package strings or public issue
  reports without an official allowlist or a live RunHaven smoke.
- Do not bundle `github.com` or `api.github.com` for Copilot yet because the
  GitHub allowlist is path-specific and RunHaven currently enforces only hosts.
- Keep Antigravity with no bundled runtime hosts until a source-backed minimal
  runtime allowlist is identified.

## Bundled Hosts

| Profile | Bundled hosts | Purpose | Evidence |
| --- | --- | --- | --- |
| `claude` | `api.anthropic.com`, `claude.ai`, `platform.claude.com` | Claude API requests, WebFetch domain safety checks, Claude account auth, and Anthropic Console auth. | Anthropic Claude Code network configuration. |
| `codex` | `api.openai.com`, `chatgpt.com` | OpenAI API traffic, Codex network-policy examples, ChatGPT sign-in, Codex web surface, and standalone installer host. | OpenAI Codex auth, CLI, approvals/security, and permissions docs. |
| `gemini` | `generativelanguage.googleapis.com` | Gemini API-key model traffic. | Gemini CLI authentication docs. |
| `antigravity` | none | No stable source-backed minimal runtime host set found yet. | Antigravity docs and current pinned image template review. |
| `copilot` | `api.githubcopilot.com`, `individual.githubcopilot.com`, `business.githubcopilot.com`, `enterprise.githubcopilot.com`, `githubcopilot.com`, `copilot-proxy.githubusercontent.com`, `origin-tracker.githubusercontent.com` | Copilot suggestion API and subscription-based Copilot routing. | GitHub Copilot allowlist and subscription-routing docs. |

RunHaven host rules match a listed host and its subdomains. For example,
`business.githubcopilot.com` also permits
`api.business.githubcopilot.com`.

## Explicit Review Hosts

| Profile | Host | Status | Purpose | Why not bundled |
| --- | --- | --- | --- | --- |
| `claude` | `downloads.claude.ai` | optional | Claude Code plugin executable downloads, native installer, and native updater. | RunHaven installs pinned npm packages into images. Runtime updater hosts should stay explicit. |
| `claude` | `raw.githubusercontent.com` | optional | Changelog feed, release notes, and plugin marketplace install counts. | GitHub raw content is broader than model/auth traffic. |
| `claude` | `bridge.claudeusercontent.com` | optional | Claude in Chrome extension bridge. | Not needed for normal CLI runs. |
| `gemini` | `accounts.google.com` | candidate | Browser-based Google account sign-in. | The documented flow uses a browser and localhost callback; live container smoke is still needed. |
| `gemini` | `aiplatform.googleapis.com` | candidate | Vertex AI mode. | Vertex projects have organization-specific controls and should be explicit. |
| `gemini` | `cloudcode-pa.googleapis.com` | candidate | Gemini Code Assist path observed in public Gemini CLI error reports. | Issue evidence is weaker than a vendor allowlist. |
| `antigravity` | `storage.googleapis.com` | build | Pinned Antigravity CLI archive download during image build. | Build-time only in the current image template. |
| `copilot` | `github.com` | candidate | GitHub login and Copilot web paths. | GitHub documents path-specific URLs; RunHaven cannot yet restrict to `/login` or `/copilot`. |
| `copilot` | `api.github.com` | candidate | GitHub user and Copilot user-management API paths. | GitHub documents path-specific API URLs; RunHaven cannot yet restrict to those paths. |
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
- Gemini CLI authentication:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html>
- Gemini CLI configuration:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html>
- GitHub Copilot allowlist reference:
  <https://docs.github.com/en/copilot/reference/copilot-allowlist-reference>
- GitHub Copilot subscription routing:
  <https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access>
- Google Developers Blog, Gemini CLI to Antigravity CLI transition:
  <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>

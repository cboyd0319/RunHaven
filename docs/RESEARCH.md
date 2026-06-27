# Research And Source Ledger

Last reviewed: 2026-06-26

This file records sources used for product, security, runtime, and pinning
decisions. Update it whenever a dependency pin, runtime assumption, security
boundary, or agent behavior changes.

## Runtime Sources

- Apple `container` release 1.0.0:
  <https://github.com/apple/container/releases/tag/1.0.0>
- GitHub release API for Apple `container` 1.0.0:
  <https://api.github.com/repos/apple/container/releases/latest>
- Local Apple `container` checkout and bundled docs were consulted. The
  machine-specific checkout path is intentionally omitted from this ledger.

Observed runtime evidence on 2026-06-14:

- `container --version`: `container CLI version 1.0.0`, commit `ee848e3`.
- `container system status`: apiserver running, version 1.0.0, commit
  `ee848e3ebfd7c73b04dd419683be54fb450b8779`.
- Signed installer package:
  `container-1.0.0-installer-signed.pkg`.
- Installer SHA-256:
  `13f45f26da94c354adcbefe1e8f7631e7f126e93c5d4dd6a5a538aa66b4f479d`.
- Installer signing team ID: `UPBK2H6LZM`.
- Recommended Kata kernel URL observed from `container system property list`:
  <https://github.com/kata-containers/kata-containers/releases/download/3.28.0/kata-static-3.28.0-arm64.tar.zst>
- Installed kernel file: `vmlinux-6.18.15-186`.
- Installed kernel SHA-256:
  `2fe4a58d2885d623bcb4d705900ac8c1d4f02371152da8126b3b00c8c47fc3a1`.

Observed runtime command surface on 2026-06-15:

- Local `container --help` lists `exec`, `logs`, `stop`, `kill`, and other
  container lifecycle commands.
- Local `container exec --help` shows the supported shape:
  `container exec [<options>] <container-id> <arguments> ...`, including
  `--interactive`, `--tty`, `--user`, and `--workdir`.
- Local `container logs --help` shows the supported shape:
  `container logs [--boot] [--follow] [-n <n>] <container-id>`.
- Local `container kill --help` shows the supported shape:
  `container kill [--all] [--signal <signal>] [--debug] [<container-ids> ...]`;
  the default signal is `KILL`.
- Local `container inspect --help` shows the supported shape:
  `container inspect [--debug] <container-ids> ...`.
- Local `container inspect runhaven-nonexistent-repair-smoke` exits 1 with
  `Error: container not found: runhaven-nonexistent-repair-smoke`, which
  RunHaven uses as the narrow stale-marker repair guard.
- The pinned command reference states that `container inspect` displays detailed
  container information in JSON. Local `container inspect buildkit` confirmed
  JSON output and showed raw inspect data can include process arguments,
  environment, and mounts.
- Local `container attach --help` reports that plugin `container-attach` is not
  installed. RunHaven `runs attach` therefore uses guarded `container exec`
  against the active RunHaven-owned container name.

Important distinction:

- Runtime pins follow the installed, signed Apple `container` 1.0.0 release and
  live CLI output.
- The local Apple `container` checkout was observed at
  `c8b4fd73a1b1696a1c6533e4de85bdedfa8fc5fc` (`1.0.0-4-gc8b4fd7`) on
  2026-06-14, which is newer than the installed release commit. Do not update
  runtime pins from that local checkout unless the installed runtime is also
  updated and verified.
- The local checkout also references `containerization` `0.33.4`; the installed
  runtime reports `ghcr.io/apple/containerization/vminit:0.33.3`. The installed
  runtime is authoritative for this repo's current pins.

## Agent Runtime Sources

- Local copy of the Claude Code dev container reference was consulted. The
  machine-specific download path is intentionally omitted from this ledger.
- Claude Code memory and `CLAUDE.md` import docs:
  <https://docs.anthropic.com/en/docs/claude-code/memory>
- Claude Code sandbox environments:
  <https://code.claude.com/docs/en/sandbox-environments>
- Claude Code dev containers:
  <https://code.claude.com/docs/en/devcontainer>
- Codex sandboxing:
  <https://developers.openai.com/codex/concepts/sandboxing>
- Codex approvals and security:
  <https://developers.openai.com/codex/agent-approvals-security>
- Gemini CLI `GEMINI.md` docs:
  <https://google-gemini.github.io/gemini-cli/docs/cli/gemini-md.html>
- Gemini CLI configuration docs:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html>
- Antigravity CLI getting started:
  <https://antigravity.google/docs/cli-getting-started>
- Antigravity CLI best practices:
  <https://antigravity.google/docs/cli-best-practices>
- Antigravity rules and workflows:
  <https://antigravity.google/docs/rules-workflows>
- GitHub Copilot CLI overview:
  <https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/overview>

Auth broker source review on 2026-06-15:

- OpenAI Codex authentication:
  <https://developers.openai.com/codex/auth>.
  Reviewed for ChatGPT sign-in, API-key sign-in, access-token automation, and
  local login caching behavior.
- Claude Code authentication:
  <https://code.claude.com/docs/en/iam>.
  Reviewed for supported auth types, environment-variable precedence, and
  `apiKeyHelper`.
- Claude Code setup:
  <https://code.claude.com/docs/en/setup>.
  Reviewed for browser-login setup context.
- Gemini CLI authentication:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html>.
  Reviewed for Google login, Gemini API key, Vertex ADC, service account JSON,
  Google Cloud API-key, and headless-environment behavior.
- GitHub Copilot CLI authentication:
  <https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli>.
  Reviewed for OAuth device login, environment-token auth, GitHub CLI fallback,
  BYOK environment variables, and token precedence.
- GitHub Copilot SDK authentication:
  <https://docs.github.com/en/copilot/how-tos/copilot-sdk/auth/authenticate>.
  Reviewed for stored GitHub signed-in user credentials and system keychain
  usage.

RunHaven auth broker decision from this review:

- `runhaven auth status` and `runhaven auth explain AGENT` are static,
  secret-free diagnostics only.
- The Codex API-key broker can be a narrow first implementation because Codex
  supports custom model providers and the OpenAI Responses API.
- Other real brokers remain future work. They should be provider-specific,
  host-owned, explicit opt-in, auditable, and tied to the endpoint matrix.
- Broad path-sensitive GitHub hosts are not bundled merely because Copilot can
  use them; the broker or another verified provider-specific control must make
  the credential and path boundary explicit first.

Codex API-key broker implementation source check on 2026-06-15:

- Codex CLI command-line options:
  <https://developers.openai.com/codex/cli/reference>.
  Reviewed for repeatable `-c key=value` configuration overrides, `codex exec`,
  `--skip-git-repo-check`, `--sandbox`, `--ask-for-approval`, and
  `--output-last-message`.
- Codex advanced configuration:
  <https://developers.openai.com/codex/config-advanced>.
  Reviewed for custom model provider definitions and `model_provider`
  selection.
- Codex configuration reference:
  <https://developers.openai.com/codex/config-reference>.
  Reviewed for `model_providers.<id>.base_url`, `env_key`, and `wire_api`;
  `responses` is the supported wire API.
- OpenAI Responses create API:
  <https://developers.openai.com/api/reference/resources/responses/methods/create>.
  Reviewed for the Responses create endpoint used by the broker.

## Provider Endpoint Sources

Reviewed on 2026-06-15 for the bundled provider endpoint matrix:

- Claude Code enterprise network configuration:
  <https://code.claude.com/docs/en/corporate-proxy>.
  Source-backed bundled hosts are `api.anthropic.com`, `claude.ai`, and
  `platform.claude.com`. Update, plugin, release-note, and extension bridge
  hosts remain explicit.
- OpenAI Codex authentication:
  <https://developers.openai.com/codex/auth>.
  Codex supports ChatGPT sign-in and API-key sign-in. ChatGPT auth is the
  default path when no valid session is available.
- OpenAI Codex CLI:
  <https://developers.openai.com/codex/cli>.
  The standalone installer and Codex web surface use `chatgpt.com`.
- OpenAI Codex approvals and security:
  <https://developers.openai.com/codex/agent-approvals-security>.
  The network-policy examples use `api.openai.com` and describe allowlist-first
  host matching, local/private destination blocking, and DNS rebinding checks.
- OpenAI Codex permissions:
  <https://developers.openai.com/codex/permissions>.
  The common profile examples include `api.openai.com` as an allowed domain.
- Gemini CLI authentication:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/authentication.html>.
  API-key mode is backed by `generativelanguage.googleapis.com`; Google account
  and Vertex modes remain explicit until live smokes confirm the minimal host
  set.
- Gemini CLI configuration:
  <https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html>.
  Reviewed for settings, telemetry, and local state behavior.
- GitHub Copilot allowlist reference:
  <https://docs.github.com/en/copilot/reference/copilot-allowlist-reference>.
  Source-backed bundled hosts are limited to Copilot-specific suggestion and
  routing domains. Path-sensitive `github.com` and `api.github.com` entries,
  telemetry, experimentation, and reporting hosts remain explicit.
- GitHub Copilot subscription-based routing:
  <https://docs.github.com/en/copilot/how-tos/administer-copilot/manage-for-organization/manage-access/manage-network-access>.
  Confirms Business and Enterprise Copilot routing wildcard families.
- Google Developers Blog, Gemini CLI to Antigravity CLI transition:
  <https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/>.
  Reviewed for current Antigravity CLI lifecycle context. It does not provide a
  minimal runtime endpoint allowlist.

Pinned package scan on 2026-06-18:

- Scanned npm tarballs matching the image pins at that date:
  `@anthropic-ai/claude-code@2.1.181`, `@openai/codex@0.140.0`,
  `@google/gemini-cli@0.46.0`, and `@github/copilot@1.0.63`.
- Package strings were used only as weak supporting evidence. Hosts were not
  promoted to bundled defaults without official source evidence or a future
  live RunHaven smoke.
- Gemini package contents included many third-party and documentation domains,
  so only official docs and focused candidate hosts were recorded.
- Antigravity public docs are JavaScript-rendered through the static site. The
  current image template downloads the pinned CLI archive from
  `storage.googleapis.com/antigravity-public`, but no source-backed runtime host
  list was found.

Pinned package scan refresh on 2026-06-24:

- Current image pins are now `@anthropic-ai/claude-code@2.1.190`,
  `@openai/codex@0.142.0`, `@google/gemini-cli@0.47.0`, and
  `@github/copilot@1.0.64`. The 2026-06-18 host evidence above remains
  historical; `pins.toml` is the source of truth for current agent CLI pins.

Live agent egress observation on 2026-06-26:

- `runhaven login <agent>` was built and live-verified for Claude (host
  `claude setup-token`), Codex (`codex login --device-auth`, reaches
  `auth.openai.com`), Copilot (`copilot login`, reaches `github.com` and
  `api.github.com`), and Antigravity (agy first-run Google OAuth). The Codex and
  Copilot login/refresh hosts were flipped from candidate to bundled in the
  endpoint matrix; the Copilot widening is deliberate and documented.
- Antigravity now has an observation-backed runtime host set from real egress:
  `oauth2.googleapis.com`, `www.googleapis.com`, `cloudcode-pa.googleapis.com`,
  and `daily-cloudcode-pa.googleapis.com`. `accounts.google.com` and
  `antigravity.google` are browser-side only. This supersedes the 2026-06-18
  "no source-backed runtime host list" note for Antigravity.
- The egress matcher now supports maintainer-curated domain-family wildcard
  patterns (`*-name.domain.tld`, anchored to one registrable domain,
  default-deny preserved). Antigravity's `daily-cloudcode-pa` pin became
  `*-cloudcode-pa.googleapis.com` so region or channel prefixes need no re-pin
  while `storage.googleapis.com` and other googleapis.com services stay denied.

Local reference harness:

- A local reference harness repo was consulted for instruction, pin-check, and
  package layout patterns. Machine-specific paths are intentionally omitted
  from this ledger.

## Tauri UI Planning Sources

Reviewed on 2026-06-16 before adding the pre-implementation UI guardrails:

- Tauri v2 security overview:
  <https://v2.tauri.app/security/>.
  Reviewed for the Rust core versus WebView trust boundary and the IPC
  boundary that exposes privileged operations.
- Tauri v2 capabilities:
  <https://v2.tauri.app/security/capabilities/>.
  Reviewed for window/WebView capability scoping, capability files under
  `src-tauri/capabilities`, and the risk of broad default command exposure
  unless the app manifest narrows registered commands.
- Tauri v2 permissions:
  <https://v2.tauri.app/security/permissions/>.
  Reviewed for explicit command privileges, permission scopes, and capability
  references.

## Package And Image Sources

- Rust toolchain and crate versions reviewed on 2026-06-18 with local
  `rustc --version`, `cargo --version`, and `cargo search`:
  <https://crates.io/>
- Rust release calendar reference:
  <https://releases.rs/>
- npm registry records checked with `npm view` for:
  `@anthropic-ai/claude-code`, `@openai/codex`,
  `@google/gemini-cli`, `@github/copilot`, and `npm`.
- Debian snapshot used by bundled images:
  <http://snapshot.debian.org/archive/debian/20260614T000000Z>
- Debian security snapshot used by bundled images:
  <http://snapshot.debian.org/archive/debian-security/20260614T000000Z>
- Docker image digests checked with `docker buildx imagetools inspect`:
  `debian:trixie-slim` and `node:26.3.0-trixie-slim`.
- GitHub Actions release API (retained for reference only; no workflows
  currently exist, so no action pins are tracked, see `docs/PINNING.md`):
  <https://api.github.com/repos/actions/checkout/releases/latest>.

Current reviewed pins are recorded in [`../pins.toml`](../pins.toml). The
Rust dependency lock is recorded in [`../Cargo.lock`](../Cargo.lock). The
enforced pin policy lives in
[`../crates/runhaven-core/src/harness/pins.rs`](../crates/runhaven-core/src/harness/pins.rs)
and is run with `cargo run --locked --bin runhaven-check-pins`.

## Documentation Research Sources

README and agent-instruction guidance reviewed on 2026-06-14:

- GitHub Docs, repository README guidance:
  <https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes>
- GitHub Docs, repository best practices:
  <https://docs.github.com/en/repositories/creating-and-managing-repositories/best-practices-for-repositories>
- Open Source Guides, starting a project:
  <https://opensource.guide/starting-a-project/>
- Make a README:
  <https://www.makeareadme.com/>
- AGENTS.md open format:
  <https://agents.md/>
- OpenAI Codex `AGENTS.md` guidance:
  <https://developers.openai.com/codex/guides/agents-md>
- GitHub Blog, `AGENTS.md` lessons from open repositories:
  <https://github.blog/ai-and-ml/github-copilot/how-to-write-a-great-agents-md-lessons-from-over-2500-repositories/>

High-usage open-source README references reviewed on 2026-06-14:

- Kubernetes:
  <https://raw.githubusercontent.com/kubernetes/kubernetes/master/README.md>
- Visual Studio Code:
  <https://raw.githubusercontent.com/microsoft/vscode/main/README.md>
- React:
  <https://raw.githubusercontent.com/facebook/react/main/README.md>
- Terraform:
  <https://raw.githubusercontent.com/hashicorp/terraform/main/README.md>
- Homebrew:
  <https://raw.githubusercontent.com/Homebrew/brew/master/README.md>
- Rust:
  <https://raw.githubusercontent.com/rust-lang/rust/master/README.md>

## User-Supplied Secondary References

These sources informed threat-modeling and UX/security framing. Treat them as
secondary references and verify technical claims against primary sources before
changing code.

- <https://dev.to/wartzarbee/how-to-run-claude-code-sandboxed-containers-network-walls-and-secret-isolation-2jkn>
- <https://github.com/vitalio-sh/claude-code-secure-container>
- <https://www.truefoundry.com/blog/claude-code-sandboxing>
- <https://www.stackhawk.com/blog/developers-guide-to-writing-secure-code-with-claude-code/>

## Apple Container Egress Research

Reviewed on 2026-06-14 for Apple `container` 1.0.0:

- User supplied Apple documentation entrypoint:
  <https://apple.github.io/container/documentation/>
- Rendered Apple DocC pages with Playwright and checked generated DocC JSON:
  <https://apple.github.io/container/data/documentation/containernetworkservice/networkmode.json>,
  <https://apple.github.io/container/data/documentation/containernetworkservice/networkmode/nat.json>,
  and
  <https://apple.github.io/container/data/documentation/containernetworkservice/networkconfiguration.json>.
- Reviewed a complete local DocC snapshot supplied by the user. The snapshot
  was captured from the Apple documentation entrypoint on 2026-06-14 and
  contains 1,022 rendered Markdown pages plus raw DocC JSON with zero fetch
  failures.
- Official Apple `container` 1.0.0 command reference:
  <https://github.com/apple/container/blob/1.0.0/docs/command-reference.md>
- Local installed CLI:
  `container run --help`, `container network --help`, and
  `container network create --help`.
- Local sibling source/docs:
  `../apple-container/docs/command-reference.md`.

Observed command surface:

- `container run` supports DNS selection with `--dns`, DNS-related options,
  `--network`, and `--no-dns`.
- `container network create` supports `--internal` for host-only networks plus
  labels, plugin options, plugin selection, and subnet settings.
- Apple's generated `ContainerNetworkService` docs expose `NetworkMode.nat`,
  where host NAT lets containers reach external services, and
  `NetworkConfiguration` fields for `id`, `mode`, and `subnet`.
- The generated network plugin docs describe an XPC API for IP address
  allocation on a network; they do not describe egress filtering.
- The generated DNS docs expose nameservers, domain, search domain, options,
  and local DNS domain management. These are name-resolution controls, not
  packet or destination controls.
- No domain allowlist or egress allowlist flag was found in the pinned command
  reference, rendered DocC pages, complete generated DocC JSON/Markdown
  snapshot, local CLI help, or local docs search.
- Exact searches across the complete DocC snapshot returned no hits for
  `egress`, `allowlist`, `allow-list`, `allow list`, `denylist`, `blocklist`,
  `firewall`, `packet filter`, `proxy`, or `domain allow`.

Conclusion: RunHaven must not treat DNS selection as egress enforcement.
Provider egress needs an explicit enforcement mechanism such as a reviewed
proxy, DNS filter plus packet controls, or another Apple `container`-compatible
boundary.

Follow-up verification on 2026-06-14 proved the reviewed proxy pattern with
live Apple `container` smokes. A container on an internal network can reach a
host-side proxy through the guest's default gateway. The smoke harness starts a
CONNECT proxy with provider host allowlisting, restricts accepted clients to the
internal-network subnet when it cannot bind directly to the gateway address,
blocks IP literal CONNECT targets, and verifies allowed proxied HTTPS, denied
proxied host, denied proxied IP literal, denied direct DNS egress, and denied
direct IP egress.

Follow-up implementation on 2026-06-14 integrated the reviewed proxy lifecycle
into `runhaven run --network provider`. The CLI now creates a managed internal
network, inspects its IPv4 gateway and subnet, starts the host-side CONNECT
proxy, injects proxy environment variables, and deletes the managed provider
network after the run. A live runtime smoke verified allowed proxied HTTPS and
blocked denied proxied host, proxied IP literal, direct DNS, and direct IP
paths.

The local `container-machine.md` docs reinforce that `container machine` is not
the right beginner-safe boundary for RunHaven because it automatically maps the
host username and home directory into the Linux environment. The local
`container-system-config.md` docs expose default resource, DNS, registry,
kernel, vminit, and subnet settings, but no domain egress allowlist control.

## Supplemental Apple Container References

Reviewed on 2026-06-14 after user supplied additional sources. Treat Apple
published material as primary. Treat wiki, news, tutorial, and generated-index
sources as secondary context unless the same claim is verified in Apple docs,
the installed CLI, or the Apple source tree.

- Apple Open Source project page:
  <https://opensource.apple.com/projects/container/>
  - Primary source. Defines `container` as a Swift tool for creating and
    running Linux containers with lightweight VMs on Apple silicon.
  - States that `container` uses Apple's open source `containerization` package
    and prioritizes security, privacy, and performance.
- DeepWiki generated overview for `apple/container`:
  <https://deepwiki.com/apple/container/1-overview>
  - Secondary generated index over the Apple source tree. Useful for navigation
    to source-backed topics such as the CLI, `container-apiserver`, image
    helpers, networking, DNS, and machine runtime.
  - Do not treat DeepWiki summaries as authoritative for pins or product claims
    without checking Apple source, Apple docs, or live CLI behavior.
- Wikipedia overview:
  <https://en.wikipedia.org/wiki/Apple_container>
  - Secondary background. Summarizes the one-VM-per-container architecture and
    contrasts it with shared-VM desktop container tools.
  - Notes ecosystem limitations such as Docker Compose and DevContainer gaps;
    verify current status before making roadmap or support claims.
- The Register hands-on article:
  <https://www.theregister.com/devops/2026/06/11/apple-gives-mac-devs-a-wsl-ish-thing-to-call-their-own/5254153>
  - Secondary hands-on source. Reinforces that `container machine` is a
    persistent Linux VM workflow, closer to WSL than RunHaven's task-scoped
    agent runs.
  - Reports that `container machine` defaults can mount the macOS home directory
    read-write and that `--home-mount none` is the safer mode. This matches
    RunHaven's decision to avoid `container machine` for beginner-safe agent
    workloads.
  - Reports that memory retained by a container machine may require restarting
    the VM to release back to the host. This is noted as `container machine`
    context only, not a verified `runhaven run` behavior.
- HowToUseLinux article:
  <https://www.howtouselinux.com/post/apple-built-a-new-container-tool-for-macos-heres-why-it-actually-matters>
  - Secondary technical article. Repeats the isolated lightweight VM model,
    `vminitd`, `vmnet`, XPC, and Keychain integration themes.
  - Mentions macOS 15 limitations and macOS 26 as the full-feature target.
    RunHaven remains intentionally stricter: macOS 26+ only.
- Apidog tutorial:
  <https://apidog.com/blog/apple-container-open-source-docker-alternative/>
  - Secondary tutorial. Describes `container-apiserver`, per-container runtime
    helpers, Virtualization.framework, `vmnet`, Keychain, unified logging, and
    OCI registry workflows.
  - Useful for onboarding language, not authoritative for security boundaries.
- Suraj Deshmukh tutorial:
  <https://suraj.io/post/2026/using-osx-containerization/>
  - Secondary hands-on tutorial. Notes host-to-container networking can be
    affected by macOS Local Network privacy prompts and that both the client app
    and `container-runtime-linux` may need Local Network permission.
  - Documents embedded DNS setup and direct private-network access from the Mac.
    These are useful troubleshooting candidates if RunHaven later exposes
    port-publishing or host-to-guest workflows. They are not egress controls.

## Current Product Conclusions

- Use task-scoped `container run` instead of `container machine` for beginner
  AI-agent workloads. The wrapper must mount only the intended workspace and
  project-scoped agent home volume.
- Do not block explicit or user-managed `container machine` workflows solely
  because they are less secure. Treat them as advanced paths that need clear
  warning, user intent, and target validation before any RunHaven integration.
- Do not mount macOS home directories, cloud credential directories, browser
  profiles, or raw SSH keys by default.
- Pass secrets only by explicit environment variable name with `--env NAME`;
  never print secret values in dry-run output.
- Use non-root bundled images, read-only root filesystems, dropped Linux
  capabilities, and per-project named home volumes.
- Use `--read-only-workspace` for review-only tasks.
- Treat default internet mode as unrestricted egress.
- Use `--network provider` when model-provider traffic should be constrained
  to bundled provider hosts plus reviewed fully qualified `--provider-host`
  additions.
- Keep package, image, runtime, and CI pins current stable and exact. For apt,
  use timestamped Debian snapshots plus exact package versions.

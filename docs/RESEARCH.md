# Research And Source Ledger

Last reviewed: 2026-06-14

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

Local reference harness:

- A local reference harness repo was consulted for instruction, pin-check, and
  package layout patterns. Machine-specific paths are intentionally omitted
  from this ledger.

## Package And Image Sources

- Python source releases, current stable 3.14.6 on 2026-06-14 and latest 3.13
  maintenance release 3.13.14:
  <https://www.python.org/downloads/source/>
- PyPI JSON API for Python package versions:
  <https://pypi.org/pypi/pip/json>,
  <https://pypi.org/pypi/setuptools/json>,
  <https://pypi.org/pypi/build/json>,
  <https://pypi.org/pypi/mypy/json>,
  <https://pypi.org/pypi/ruff/json>
- npm registry records checked with `npm view` for:
  `@anthropic-ai/claude-code`, `@openai/codex`,
  `@google/gemini-cli`, `@github/copilot`, and `npm`.
- Debian snapshot used by bundled images:
  <http://snapshot.debian.org/archive/debian/20260614T000000Z>
- Debian security snapshot used by bundled images:
  <http://snapshot.debian.org/archive/debian-security/20260614T000000Z>
- Docker image digests checked with `docker buildx imagetools inspect`:
  `debian:trixie-slim` and `node:26.3.0-trixie-slim`.
- GitHub Actions release API:
  <https://api.github.com/repos/actions/checkout/releases/latest>
  and
  <https://api.github.com/repos/actions/setup-python/releases/latest>.

Current reviewed pins are recorded in [`../pins.toml`](../pins.toml). The
Python development transitive lock is recorded in
[`../requirements-dev.txt`](../requirements-dev.txt). The enforced pin policy
lives in [`../scripts/check_pins.py`](../scripts/check_pins.py).

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

## Current Product Conclusions

- Use task-scoped `container run` instead of `container machine` for beginner
  AI-agent workloads. The wrapper must mount only the intended workspace and
  project-scoped agent home volume.
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

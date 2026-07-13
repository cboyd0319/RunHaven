# Security Policy

## Supported Scope

This project targets macOS 26+ on Apple silicon with Rust 1.97.0 and Apple
`container` 1.1.0. Windows and Linux are not supported.

RunHaven remains alpha/pre-release. `v0.5.0` established the CLI contract;
`v0.7.0` is a limited coworker-test candidate for the CLI plus desktop app.
Security reports are welcome. Do not assume backward-compatible record,
provider, or desktop behavior before `v1.0.0`.

## Reporting

Open a private security advisory or contact the maintainer directly for issues
that could expose host files, credentials, network access, or command execution
outside the documented boundary.

Do not open a public issue for exploitable security bugs.

## Current Boundary

The default wrapper boundary is:

- one workspace mounted at `/workspace`
- one per-project agent home volume mounted at `/home/agent`
- read-only container root filesystem
- temporary `/tmp`
- no host home mount
- no host cloud credential mount
- no raw SSH key mount
- no host environment passthrough unless requested with `--env`
- Linux capabilities dropped with `--cap-drop ALL`
- sensitive host paths and root agent execution rejected unless explicitly
  overridden

The default `internet` network mode remains unrestricted egress and should be
treated as able to reach any destination permitted by Apple `container` and the
host network. The `internal` network mode uses Apple `container network create
--internal` for local-only work.

`--network provider` runs the agent on an internal network and injects a
host-side CONNECT proxy that permits bundled provider hosts plus their
subdomains, maintainer-curated domain-family patterns (`*-name.domain.tld`
anchored to a single registrable domain), and explicit fully qualified
`--provider-host HOST` additions. The policy stays default-deny: IP literal
proxy targets and single-label provider hosts are rejected, and direct guest
egress remains blocked by the internal network.

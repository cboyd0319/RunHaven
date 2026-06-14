# Security Policy

## Supported Scope

This project targets macOS 26+ on Apple silicon with Python 3.13+ and Apple
`container` 1.0.0. Windows and Linux are not supported.

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
subdomains, along with explicit `--provider-host HOST` additions. IP literal
proxy targets are rejected, and direct guest egress remains blocked by the
internal network.

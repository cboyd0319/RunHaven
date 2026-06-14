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

Network egress allowlisting is not fully enforced by this repo yet. The
`internal` network mode uses Apple `container network create --internal`, which
is useful for local-only work but does not let cloud AI CLIs reach their model
providers. Internet-enabled agent runs should be treated as able to reach any
destination permitted by Apple `container` and the host network.

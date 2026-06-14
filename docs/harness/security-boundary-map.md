# Security Boundary Map

Status: draft

Use this file to make agent-visible security, privacy, permission, and cost
boundaries explicit before work touches risky surfaces.

People run project commands on personal machines. The product rule is: choose
the most secure and easiest behavior for every edge case. If those conflict,
security wins and the tool must explain the safe next step.

## Access Boundaries

| Boundary | Current Owner | Rule |
| --- | --- | --- |
| Local repository files | Project maintainers | Agents must preserve user changes and avoid destructive git operations unless explicitly approved. |
| Host filesystem | `mca` CLI and Apple `container` | Default runs mount only the selected workspace and a project-scoped `/home/agent` volume. Host home directories, raw SSH keys, browser profiles, and cloud credential folders stay unmounted. |
| Agent state volume | `mca` CLI | Per-project/profile state is locked during a run so concurrent agents cannot attach the same named volume. |
| Generated paths | Project maintainers | Generated paths must remain inside the repository after symlink resolution. Unsafe path inputs are rejected. |
| Secrets and credentials | Project maintainers | Do not print, store, transform, or transmit secrets unless the task explicitly requires a reviewed secret-handling path. Prefer `--env NAME` over inline values. |
| Network calls | Project maintainers | Prefer local verification. Internet-enabled agent runs are unrestricted egress until provider allowlisting lands. |
| Cost-incurring systems | Project maintainers | Cloud, model, or paid API changes require explicit approval and rollback notes. |

## Data Boundaries

- Classify sensitive features in `feature-privacy-labels.json` when the project
  handles personal, customer, credential, financial, medical, or private
  business data.
- Default to local-only processing until an external data flow is explicit.
- Record redaction, preview, approval, and logging requirements for any
  external AI or third-party service path.

## Required Checks

Use the smallest relevant checks from `verification-matrix.md` plus human review
for authentication, authorization, secrets, payment, user data, destructive
operations, and release automation.

For runtime-boundary changes, also run `mca doctor`, `mca plan`, and a focused
Apple `container` smoke that proves the claimed mount, user, network, or
filesystem behavior.

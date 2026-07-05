# Home dashboard (variant 06, HIG foundation)

The Home destination as a daily command surface: is this Mac ready, which runs
need the user, and what happened recently. Dense, calm, native-feeling, with the
secure path visible. Built from a selected mockup (no separate design doc); this
note is the spec of record. It supersedes the earlier stacked-card Home.

## Shell

Navigation stays in the sidebar (this is not the top-nav variant). The app shell
is unchanged: full-height sidebar on the left, header plus scrolling content on
the right. `NavDestination` and the navigation model are unchanged.

### Sidebar (`RunHavenSidebar`)

- Brand header (logo, RunHaven, Agent launcher) with the NAT-1 traffic-light
  inset and drag region.
- The four destinations: Home, Agents, Diagnostics, Maintenance. A conditional
  Run terminal row appears only while an interactive run is live.
- A `Runs` summary: three count rows -- Needs review, Active, Finished --
  derived from the session list (`groupSessions` plus `sessionAttention`). The
  counts are a pipeline summary; each opens the Run history view.
- App version footer.

The sidebar no longer lists individual sessions. Any session is reachable from
the Run history view (`{ kind: "runs" }`, `views/RunsView.tsx`) -- the full list
of every run, newest first, keyboard-operable -- opened from the Runs counts or
Home's "Open full run history". Home's Needs review and Recent activity tables
reach the recent ones directly. Consequence: switching between two sessions
routes through Home or Run history rather than a persistent sidebar list.

## Home content (`HomeView`)

- Readiness line: a green check and `Ready to run` when the Mac can launch, an
  amber/red `Setup needed` line when blocked (blocking detail renders in the
  `SetupCard` below), plus an `Open diagnostics` link. Derived from backend
  setup status, never frontend optimism.
- `New run`: the one prominent action, the existing `LaunchSheet` trigger. The
  Rust-owned native confirmation stays the mutation boundary; Home never mutates.
- Two columns:
  - Left: `Needs review` (attention runs: run, agent, issue, updated, Review)
    and `Recent activity` (newest runs of any lifecycle except the ones in Needs
    review, so each run appears once; row opens the session; footer opens the
    full run history).
  - Right: `Agent setup` (a summary and a per-agent real sign-in-state row, see
    "Agent sign-in state" below; footer Open Agents) and `Secure boundary` (a
    plain-language summary of the standing safety posture -- project-folder-only
    workspace, isolated sign-in, provider allowlist, home not mounted -- footer
    View security model, which opens Diagnostics).

## Shared primitives (`app/dashboard/`)

- `status.tsx`: `runStatusTone`, `StatusDot`. One tone -> dot-color mapping onto
  the existing shadcn tokens (no parallel token set); tones include a `warning`
  amber for the needs-sign-in agent state.
- `agentVisuals.ts`: product-safe lucide glyphs tinted per provider (no vendor
  logos). `AgentCell` pairs the glyph with the agent display name.
- `agentReadiness.ts`: maps the backend-derived per-agent readiness
  (`DashboardStatus.agentReadiness`) to a dot tone and a one-line summary. The
  category and label are computed in core, not here.
- `timeAgo.ts`: compact relative times ("2m ago").
- `DashboardPanel` / `PanelFooterLink`: the shared panel chrome and footer row.

## Agent sign-in state

`Agent setup` shows each agent's REAL sign-in state, not a static "API key
broker" method label. `DashboardStatus.agentReadiness` is computed in core
(`provider/agent_sign_in_record.rs`) from what RunHaven can truthfully know
without reading an isolated credential:

- Shell (no auth needed) -> `ready` / "Ready".
- Claude -> Keychain token presence (the opt-in broker, attributes-only, no
  token read, no prompt) -> "Signed in" / "Needs sign-in".
- Other agents -> a secret-free sandbox sign-in marker
  (`cache_root/agent-sign-ins.jsonl`, fields `{agent, scope, signed_in_at}`,
  0600) written only when RunHaven's device-code sign-in exits 0.

Labels describe the AGENT'S state only. Nothing implies RunHaven holds or
manages the credential (no "via RunHaven"); the marker means the agent completed
sign-in in its isolated sandbox, which RunHaven observed -- it never sees the
credential. Security-reviewed SOUND (secret-free marker, no Keychain read/prompt,
no secret across IPC).

Honesty ceilings (documented, not worked around by reading the token):
- The marker means "signed in in its sandbox", not "the token is still valid".
- A sign-out / cleared sandbox state does not remove the marker, so it can read
  stale "Signed in" until re-evaluated. Follow-up.
- An agent signed in ONLY through a terminal (not the device-code runner, not
  Claude Keychain) leaves no marker and reads "Needs sign-in". Follow-up.

## Safety

Every panel is a read-only summary of backend-owned, already-sanitized state.
No panel renders a raw path, container name, state volume, token, environment
value, or launch argument; run titles come from `sessionDisplayTitle` and
statuses from the guarded label helpers. A regression test asserts the Home
panels never leak those shapes.

## Deferred / follow-ups

- A cross-session switcher (command palette) so switching sessions does not
  require a trip through Home or Run history.
- The New run split-button dropdown (per-agent quick launch) from the mockup.

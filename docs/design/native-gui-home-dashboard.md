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
  counts are a pipeline summary; each navigates to Home, the run overview.
- App version footer.

The sidebar no longer lists individual sessions. Sessions are reached from
Home's Needs review and Recent activity tables. Consequence: switching between
two sessions routes through Home rather than a persistent sidebar list.

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
  - Right: `Agent setup` (a readiness summary and a per-agent status row, footer
    Open Agents) and `Secure boundary` (a plain-language summary of the standing
    safety posture -- project-folder-only workspace, isolated sign-in, provider
    allowlist, home not mounted -- footer View security model, which opens
    Diagnostics).

## Shared primitives (`app/dashboard/`)

- `status.tsx`: `runStatusTone`, `StatusDot`, `RunStatusIcon`, `StatusBadge`.
  One tone/icon/badge mapping onto the existing shadcn variants (no parallel
  token set), so every panel reads the same and none invents a one-off badge.
- `agentVisuals.ts`: product-safe lucide glyphs tinted per provider (no vendor
  logos). `AgentCell` pairs the glyph with the agent display name.
- `agentReadiness.ts`: agent auth status -> truthful label (from `labels.ts`)
  plus a tone and a ready/needs-setup/unavailable category for the summary line.
- `timeAgo.ts`: compact relative times ("2m ago").
- `DashboardPanel` / `PanelFooterLink`: the shared panel chrome and footer row.

## Safety

Every panel is a read-only summary of backend-owned, already-sanitized state.
No panel renders a raw path, container name, state volume, token, environment
value, or launch argument; run titles come from `sessionDisplayTitle` and
statuses from the guarded label helpers. A regression test asserts the Home
panels never leak those shapes.

## Deferred / follow-ups

- A dedicated full run-history view. Today the sidebar counts and Recent
  activity's "Open full run history" navigate to Home; older runs beyond the
  recent list are not yet listable. Add a `runs` destination and a list view.
- A cross-session switcher so switching sessions does not require going Home.
- The New run split-button dropdown (per-agent quick launch) from the mockup.

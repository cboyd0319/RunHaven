# Native GUI Wireframe

Last updated: 2026-06-30 UTC

## Goal

Make the secure RunHaven path feel like the easy path for a nontechnical macOS
user. The first GUI should answer one question: "Can I safely start an agent in
this project now?"

This is a wireframe contract for the first Tauri prototype. It does not
authorize new backend commands, broad Tauri plugins, shell bridges, signing
work, updater work, or SwiftUI migration.

## Design Stance

RunHaven should feel like a calm macOS utility:

- Plain language, short labels, and one obvious next action.
- Dense enough for repeat use, but not a developer dashboard.
- Safe defaults visible, not hidden.
- The launch screen should use one agent picker. Do not show both a dropdown
  and duplicate agent cards for the same choice.
- Default launch should read like a short wizard: agent, project, review.
  Network, CPU, memory, and read-only project controls belong under Advanced.
- The first implementation should be a real wizard, not a form with a wizard
  header. Each step shows only the controls needed for that step, with Back and
  Continue/Review actions at the bottom.
- The default screen should not show diagnostics, raw policy detail, command
  detail, or duplicate boundary summaries. Put support details behind a
  secondary disclosure or secondary view.
- Avoid slogan-like labels. Prefer direct words such as Launch, Review access,
  Status, and Access.
- The default visual mode is dark, calm, and legible.
- Trust cues should be visible at decision points: what is shared, what is not
  shared, and what needs attention before launch.
- Expert controls present only behind secondary detail.
- No landing page, mascot-first screen, terminal-first screen, or generic
  container admin surface.

## Logo And Brand Asset

Use `docs/assets/logo.png` as the canonical GUI logo. It is a 512 x 512 RGBA
PNG with a glass cube over blue and teal stacked layers.

Usage:

- Show it at small app-identity sizes, such as `28 x 28` in the sidebar header
  or `40 x 40` on setup and empty states.
- Keep the logo near the RunHaven name, not as a large hero image.
- Let the logo inform accent colors: blue for selected navigation and primary
  links, teal or green for safe/ready states, amber for warnings, red only for
  destructive or blocked states.
- Let the logo guide the surface treatment: dark navy base, translucent
  glass-like panes, thin blue-teal borders, and clear contrast.
- Keep the asset crisp on Retina screens by using the PNG at rendered sizes
  well below 512 px.
- Provide an accessible text label through surrounding text or `alt="RunHaven"`
  if rendered as an image.

Do not use the logo as a decorative background, watermark, orb, splash screen,
or mascot replacement. The first screen should still lead with the safe launch
workflow.

## Primary Window

Target first pass: one window, about `1080 x 720`, responsive down to
`720 x 560`.

Use a simple macOS-style layout:

- Left sidebar for sections.
- Main content for the selected workflow.
- No persistent right sidebar or default three-pane layout.
- Plan, access, logs, diagnostics, and advanced details appear as the active
  main view, a review sheet, or a temporary inspector. They should not compete
  with the primary workflow by default.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ [logo] RunHaven                                   Setup Ready       Refresh │
├───────────────┬────────────────────────────────────────────────────────────┤
│ Start         │ Start an agent                                           │
│ Runs          │                                                           │
│ History       │ Project                                                   │
│ Diagnostics   │ [ <project-folder>                                Pick ]  │
│ Settings      │                                                           │
│               │ Agent                                                     │
│               │ [ Claude Code                                      v ]    │
│               │ Claude Code with isolated project state.                  │
│               │                                                           │
│               │ Access                                                    │
│               │ Shared: this project folder and isolated agent state.     │
│               │ Not shared: home folder, browser profiles, credentials,   │
│               │ and raw SSH keys.                                         │
│               │                                                           │
│               │ [ Review plan ]                                           │
└───────────────┴────────────────────────────────────────────────────────────┘
```

The wide-window layout may use generous whitespace, but it should not fill that
space with another permanent column. The app should feel focused, not like a
dashboard.

## Start Screen

The start screen has three required inputs:

1. Project folder.
2. Agent.
3. Review plan.

Everything else is secondary.

Visible by default:

- Setup health as one compact status in the title or toolbar.
- Project folder picker.
- Agent picker.
- Network posture summary, not a three-option fieldset.
- Short safety summary.
- Primary action: `Review safe plan`.

Hidden behind `Advanced`:

- Network mode override.
- Read-only project folder.
- CPU and memory.
- Custom image.
- Extra provider hosts.
- Environment variable names.
- Root user and sensitive workspace toggles.

Supported lower-security choices stay available in Advanced, but each one
should warn before launch. Unsupported choices still fail closed in the backend.

## Setup Blocked State

If setup is blocked, the app should not show the launch form first. It should
show the single next fix.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ RunHaven                                             Setup Needs Attention │
├───────────────┬────────────────────────────────────────────────────────────┤
│ Start         │ Finish setup                                               │
│ Runs          │                                                            │
│ History       │ RunHaven needs Apple container before it can start agents.  │
│ Diagnostics   │                                                            │
│ Settings      │ Apple container                                            │
│               │ Not found                                                  │
│               │                                                            │
│               │ [ Open setup instructions ]   [ Check again ]              │
│               │                                                            │
│               │ Other checks                                               │
│               │ 3 ready                                                    │
└───────────────┴────────────────────────────────────────────────────────────┘
```

Rules:

- Lead with the blocker, not a list of green checks.
- Show at most one primary fix button.
- Put full doctor details under Diagnostics.
- Do not require CLI knowledge in the main blocked state.

## Image Or Login Blocked State

Image and login problems are part of the start workflow, not separate
dashboards.

```text
Start a safe agent run

Project        <project-folder>
Agent          Claude Code

Claude is not ready yet.
You need to sign in inside RunHaven once. Your Mac credentials are not shared.

[ Sign in with Claude ]   [ Use a different agent ]

Details
Image ready. Network provider hosts ready.
```

The first implementation can use existing status and command text where no
typed sign-in GUI exists yet, but the visual shape should still be one next
action plus details.

## Plan Review

Plan review is a sheet or in-page review step, not a wall of command output.

```text
┌──────────────────────────────────────────────────────┐
│ Review safe plan                                     │
│                                                      │
│ Agent        Claude Code                             │
│ Project      <project-folder>                        │
│ Network      Provider only                           │
│ Workspace    Current folder only                     │
│ State        Isolated Claude state                   │
│                                                      │
│ RunHaven will not share:                             │
│ - your home folder                                   │
│ - browser profiles                                   │
│ - cloud credentials                                  │
│ - raw SSH keys                                       │
│                                                      │
│ Advanced details                                     │
│ > Exact container command                            │
│ > Provider hosts                                     │
│                                                      │
│ [ Back ]                         [ Start agent run ] │
└──────────────────────────────────────────────────────┘
```

If warnings exist:

- Change the primary button to disabled until all warnings are confirmed.
- Use checkboxes with plain language.
- Keep the exact command under Advanced details.
- Keep typed backend confirmation. Frontend confirmation is only UI.

## Active Run

After launch, the primary surface becomes run status. Do not make users hunt
through logs or diagnostics to see whether the run is alive.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ RunHaven                                                   Claude Running │
├───────────────┬────────────────────────────────────────────────────────────┤
│ Start         │ Claude is running                                         │
│ Runs          │ Project: <project-folder>                                │
│ History       │ Network: Provider only                                   │
│ Diagnostics   │                                                           │
│ Settings      │ [ Stop ] [ Hard stop ] [ Repair ]      [ Show details ]  │
│               │                                                           │
│               │ Recent activity                                           │
│               │ Agent started.                                            │
│               │ Provider request allowed.                                 │
│               │                                                           │
│               │ [ Show raw output ]                                       │
└───────────────┴────────────────────────────────────────────────────────────┘
```

Rules:

- `Stop`, `Hard stop`, and `Repair` stay explicit and confirmed.
- Raw output stays hidden until acknowledged.
- Logs are bounded.
- Failure states should lead with the fix, then details.
- Run details, active mounts, network policy, and approvals open from
  `Show details` as a sheet, popover, drawer, or replacement detail view. They
  are not a permanent right pane.

## Embedded Session Direction

The likely long-term active-run shape is a live agent session embedded in the
main content area. If RunHaven embeds a Codex session, the session should be
the primary surface after launch, not a third pane.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ RunHaven                                                   Claude Running │
├───────────────┬────────────────────────────────────────────────────────────┤
│ Start         │ Claude session                         [ Details ]        │
│ Runs          │ ┌────────────────────────────────────────────────────────┐ │
│ History       │ │ live terminal or richer session view                  │ │
│ Diagnostics   │ │                                                        │ │
│ Settings      │ │                                                        │ │
│               │ └────────────────────────────────────────────────────────┘ │
│               │ [ Stop ] [ Hard stop ] [ Repair ]   [ Show raw output ]  │
└───────────────┴────────────────────────────────────────────────────────────┘
```

Security rules for any embedded session:

- Rust/Tauri owns process creation and the reviewed launch plan.
- The frontend sends input and resize events only for an existing session ID.
- The frontend cannot request arbitrary host commands, shells, mounts,
  environment variables, or network policy changes.
- Session output is bounded or streamed with backpressure, and raw log access
  keeps the existing warning gate.
- This wireframe does not authorize a generic PTY bridge. A future embedded
  session slice must add focused backend contracts and guard tests first.

## History

History is secondary. It should answer "what happened last time?" without
showing a forensic interface first.

```text
History

Today
Claude Code     <project-folder>       finished       18 minutes ago
Codex           <other-project-folder>         stopped        1 hour ago

[ Review run ] opens a readable summary.
[ Show diff ] requires confirmation before loading content.
```

## Diagnostics

Diagnostics is for support and recovery. It should not be the start screen.

Sections:

- Setup checks.
- Provider network decisions.
- Auth status.
- Auth log metadata.
- Bounded raw output access.

Rules:

- Do not show secrets.
- Avoid workspace paths unless needed for the user action.
- Keep query strings and fragments scrubbed.
- Keep copy plain enough for nontechnical users.

## Settings

Settings is not part of the first code slice unless needed for launch. Park it
as a sidebar item or omit it until real settings exist.

Future settings candidates:

- Default agent.
- Default project folder.
- Theme.
- Reduced motion.
- Signing and update channel, only during a later distribution pass.

## Data And Command Mapping

The first prototype should use existing contracts:

| UI need | Existing source |
| --- | --- |
| Setup status | `get_dashboard_status`, `get_setup_status` |
| Agent choices | `DashboardStatus.agents`, `list_agents` |
| Image readiness | `get_image_status` |
| Plan review | `plan_run` |
| Launch | `launch_run` |
| Managed session list | `get_managed_sessions` |
| Active run | `get_run_status` |
| Stop, hard stop, repair | `stop_managed_session`, `kill_managed_session`, `repair_managed_session` for selected sessions; `stop_run`, `kill_run`, `repair_run` for direct run targets |
| Logs | `get_managed_session_log_snapshot`, `get_log_snapshot` |
| Diagnostics | `get_egress_log`, `get_auth_log`, `get_auth_status` |

Do not add a generic command runner, filesystem bridge, PTY bridge, HTTP bridge,
or broader plugin permission for this wireframe.

## Component Direction

Keep the first implementation small. The exact frontend framework is not fixed
by this wireframe.

- App shell: app-level state, selected section, refresh orchestration.
- Start run view: project, agent, safety summary, review action.
- Safety summary: what is shared and not shared.
- Plan review: review step or sheet.
- Setup blocked view: lead with the first blocker.
- Active run view: combine run status, safe controls, and recent
  activity.
- Existing diagnostics and output panels move behind the Diagnostics section.

Avoid nested cards. Use full-width panes, rows, sections, sheets, and temporary
inspectors. Do not use a persistent right summary rail in the default layout.

If the stack spike chooses React + shadcn/ui, map the wireframe to shadcn
primitives first:

- `sidebar` or a simple sidebar block for navigation.
- `button` for primary, secondary, icon, and destructive actions.
- `field`, `input`, `select`, and `checkbox` for project, agent, and advanced
  controls.
- `alert` for setup, image, login, warning, and success states.
- `dialog`, `alert-dialog`, `sheet`, or `drawer` for review, warning
  confirmation, and advanced details.
- `empty` for no active runs or no history.
- `table` for history and diagnostics only after those views move behind
  secondary navigation.
- `tabs` only for secondary detail views, not the start flow.

Svelte + shadcn-svelte is no longer the active local path. Treat it only as a
fallback if the React path hits a concrete blocker that is recorded with
evidence.

## Visual Notes

- Use system font and macOS-like density.
- Default to dark mode. Use a dark navy base, slightly translucent content
  panes, blue for primary actions, teal or green for ready states, amber for
  warnings, and red only for blocked or destructive action.
- Keep transparency subtle. It should support trust and depth, not become a
  decorative glass effect.
- Keep headings modest. This is a utility, not a marketing page.
- Use icons in buttons where they clarify the action, such as folder, refresh,
  play, stop, and warning.
- Keep focus states obvious.
- Ensure all text fits at `720 x 560`.

## Desktop Shell Polish

Borrow these implementation checks from the `agmmnn/tauri-ui` reference, but
adapt them to whichever Tauri frontend stack wins the stack spike:

- Prevent startup flash if it appears in baseline screenshots. The app may hide
  the main window until first paint, but only after checking Tauri behavior
  locally and keeping error states recoverable.
- Use one app-level scroll shell. Avoid page-level rubber-band scrolling and
  nested `main` scroll containers.
- Keep text selection desktop-like: disable accidental broad selection only if
  needed, and always leave inputs, code, logs, diagnostics, and copyable command
  details selectable.
- Pick deliberate default window dimensions in `tauri.conf.json` only if the
  current defaults undermine the wireframe at `1080 x 720` and `720 x 560`.
- Keep external links in the system browser only if the GUI adds external links.
  That requires a reviewed opener capability and should not be added as part of
  the first visual slice.
- Do not add a debug panel, log plugin, release workflow, or release-size
  profile in the first GUI slice.

## Prototype Acceptance

The first GUI prototype is acceptable when:

- A user can see the safe launch path without opening docs.
- The setup-blocked path has one clear next action.
- Advanced controls are available but not first-screen noise.
- Plan review plainly states what is shared and not shared.
- Launch, stop, hard stop, repair, raw output, and warnings keep explicit
  confirmation.
- Tauri capabilities stay limited to the existing typed commands and folder
  picker.
- Local dev and no-sign builds work without Apple Developer Program
  credentials.
- Screenshots at wide and compact sizes show no overlap, clipping, or dashboard
  clutter.
- Startup has no obvious blank or white flash, or the remaining flash is
  documented as a follow-up with evidence.

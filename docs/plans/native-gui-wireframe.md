# Native GUI Wireframe

Last updated: 2026-06-30 UTC

## Goal

Make the secure RunHaven path feel like the easy path for a nontechnical macOS
user. The first GUI should answer one question: "Can I safely start an agent in
this project now?"

This is a wireframe contract for the first Tauri/Svelte prototype. It does not
authorize new backend commands, broad Tauri plugins, shell bridges, signing
work, updater work, or SwiftUI migration.

## Design Stance

RunHaven should feel like a calm macOS utility:

- Plain language, short labels, and one obvious next action.
- Dense enough for repeat use, but not a developer dashboard.
- Safe defaults visible, not hidden.
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
- Center content for the selected workflow.
- Right summary rail on wide windows only.
- On narrow windows, the summary rail stacks below the workflow.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ [logo] RunHaven                                   Setup Ready       Refresh │
├───────────────┬───────────────────────────────────┬────────────────────────┤
│ Start         │ Start a safe agent run            │ Safety summary         │
│ Runs          │                                   │                        │
│ History       │ Project                           │ Shared with agent      │
│ Diagnostics   │ [ /Users/c/project          Pick ]│ - this project folder  │
│ Settings      │                                   │ - isolated agent state │
│               │ Agent                             │                        │
│               │ [ Claude Code             v ]     │ Not shared             │
│               │ Claude Code with isolated state.  │ - home folder          │
│               │                                   │ - browser profiles     │
│               │ Network                           │ - cloud credentials    │
│               │ Provider only                     │ - raw SSH keys         │
│               │ Only Anthropic hosts are allowed. │                        │
│               │                                   │ Network                │
│               │ [ Review safe plan ]              │ Provider only          │
└───────────────┴───────────────────────────────────┴────────────────────────┘
```

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

Project        /Users/c/project
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
│ Project      /Users/c/project                        │
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
├───────────────┬───────────────────────────────────┬────────────────────────┤
│ Start         │ Claude is running                 │ Run details            │
│ Runs          │ Project: /Users/c/project          │                        │
│ History       │ Network: Provider only             │ Started 2 minutes ago  │
│ Diagnostics   │ State: running                     │ Container healthy      │
│ Settings      │                                   │ Provider only network  │
│               │ [ Stop ] [ Hard stop ] [ Repair ] │                        │
│               │                                   │ Not shared             │
│               │ Recent activity                   │ home, browser profiles │
│               │ Agent started.                    │ credentials, SSH keys  │
│               │ Provider request allowed.          │                        │
│               │                                   │ [ Show raw output ]    │
└───────────────┴───────────────────────────────────┴────────────────────────┘
```

Rules:

- `Stop`, `Hard stop`, and `Repair` stay explicit and confirmed.
- Raw output stays hidden until acknowledged.
- Logs are bounded.
- Failure states should lead with the fix, then details.

## History

History is secondary. It should answer "what happened last time?" without
showing a forensic interface first.

```text
History

Today
Claude Code     /Users/c/project       finished       18 minutes ago
Codex           /Users/c/other         stopped        1 hour ago

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
| Active run | `get_run_status` |
| Stop, hard stop, repair | `stop_run`, `kill_run`, `repair_run` |
| Logs | `get_log_snapshot` |
| Diagnostics | `get_egress_log`, `get_auth_log`, `get_auth_status` |

Do not add a generic command runner, filesystem bridge, PTY bridge, HTTP bridge,
or broader plugin permission for this wireframe.

## Component Direction

Keep the first implementation small:

- `App.svelte`: app-level state, selected section, refresh orchestration.
- `StartRunView.svelte`: project, agent, safety summary, review action.
- `SafetySummary.svelte`: what is shared and not shared.
- `PlanReviewPanel.svelte`: adapt the existing component into the review step.
- `SetupBlockedView.svelte`: lead with the first blocker.
- `ActiveRunView.svelte`: combine run status, safe controls, and recent
  activity.
- Existing diagnostics and output panels move behind the Diagnostics section.

Avoid nested cards. Use full-width panes, rows, sections, and one summary rail.

## Visual Notes

- Use system font and macOS-like density.
- Use restrained colors: neutral background, white content panes, green for
  ready/safe, amber for warnings, red only for blocked or destructive action.
- Keep headings modest. This is a utility, not a marketing page.
- Use icons in buttons where they clarify the action, such as folder, refresh,
  play, stop, and warning.
- Keep focus states obvious.
- Ensure all text fits at `720 x 560`.

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

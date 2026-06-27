# 03 Runtime Wiring

## Target Runtime Wiring

The final control flow should look like this:

```text
crates/runhaven/src/main.rs
  no args + stdin/stdout are TTY
    -> runhaven_tui::run()

crates/runhaven-tui/src/lib.rs
  -> tui::run()

crates/runhaven-tui/src/tui/lib.rs or adapted entrypoint
  -> load RunHaven TUI config
  -> create RunHaven AppServerSession
  -> initialize Codex Tui terminal runtime
  -> App::run(...)

Codex-shaped App
  -> owns async event loop
  -> consumes AppEvent, active thread events, TuiEventStream, and AppServerEvent
  -> owns ChatWidget, BottomPane, overlays, transcript, status, pet
  -> calls AppServerSession for typed requests

AppServerSession
  -> RunHavenAppServerClient::request_typed(...)
  -> RunHavenTuiService methods
  -> runhaven-core planner, launcher, active runs, records, diagnostics

RunHavenTuiService
  -> emits ServerNotification and ServerRequest values
  -> never reaches around runhaven-core safety checks
```

The current temporary flow:

```text
runhaven_tui::run()
  -> tui::mod.rs run()
  -> app_shell.rs ShellState
  -> launch_wizard.rs
  -> direct runhaven-core plan build
```

should be treated as a staging path only.

The Codex `App::run` loop is a four-way router. Preserve that shape:

| Loop input | Codex source | RunHaven meaning |
| --- | --- | --- |
| `app_event_rx.recv()` | `AppEventSender` and widgets | Internal UI commands such as open picker, submit action, resolve approval, launch external editor, exit. |
| active thread channel | `thread_event_channels`, `active_thread_rx` | Events for the currently selected RunHaven run/session lane. |
| `tui_events.next()` | `TuiEventStream` | Key, paste, resize, draw, focus, suspend/resume handling. |
| `app_server.next_event()` | `AppServerSession` | Backend notifications and server requests from the RunHaven service. |

Do not collapse these into one custom RunHaven event enum unless the original
Codex event loop can still be diffed. Add RunHaven variants to the existing
lanes where needed.

## RunHaven Backend Interface

Create a RunHaven TUI backend layer that is intentionally boring:

```text
crates/runhaven-tui/src/tui/runhaven/
  app_server_client.rs   Codex-shaped client API for AppServerSession.
  protocol.rs            RunHaven request, response, notification, request types.
  service.rs             Calls runhaven-core and emits protocol events.
  mapper.rs              Converts runhaven-core data into Codex-shaped UI items.
  launch_wizard.rs       Existing RunHaven launch UI view model.
```

If this grows beyond TUI-only use, split it later:

```text
crates/runhaven-app-server-protocol/
crates/runhaven-app-server-client/
```

Do not start with extra crates unless the module becomes shared outside
`runhaven-tui`. The immediate goal is source closeness, not new workspace
ceremony.

`AppEventSender` is the internal command bus. Widgets should send app events
instead of receiving direct access to `App`, `RunHavenTuiService`, or
`runhaven-core`. This matters for source closeness because Codex already uses
`AppEvent` for pickers, persistence, approvals, thread routing, shutdown, and
background results.

`BottomPaneView` is the modal/view contract. New RunHaven views should fit this
trait before adding bespoke shell state:

- handle key events and paste
- report completion and cancellation
- consume approval, request-user-input, and elicitation-style requests
- expose time-based redraw needs
- mark terminal title action-required state when blocked on the user

## Codex Concepts Mapped To RunHaven

Use Codex vocabulary internally where it helps source reuse:

| Codex concept | RunHaven meaning |
| --- | --- |
| Thread | One RunHaven UI session or one run-history session lane. For active/history surfaces, the thread id can be the RunHaven run id or a stable UUID that maps to a run id. |
| Turn | One user-initiated RunHaven action: build plan, launch, read status, fetch logs, stop, kill, repair, show diff, show diagnostics. |
| UserInput | Composer or command-palette text plus structured text elements. RunHaven can also use app events for picker actions. |
| ThreadItem | Visible transcript item: plan summary, launch command, status snapshot, log snippet, diff summary, diagnostic result, approval request, warning. |
| CommandExecution item | A `container` command, preflight command, active run control command, or safe local diagnostic command. |
| FileChange item | Run diff or worktree diff, backed by `runhaven_core::records::run_diff_text`. |
| Approval request | Lower-security launch confirmation, stop/kill/repair confirmation, or future workspace/boundary escalation. |
| Model picker | Do not map directly to model providers unless RunHaven has a model concept. Agent profiles belong in RunHaven product cards or picker rows. |
| Status card | RunHaven status: agent, workspace, network, auth scope, active run, image readiness, diagnostics, doctor checks. |
| Goal | Optional. Leave disabled until RunHaven has a goal/task budget feature. |

Do not force every RunHaven action into Codex semantics if it damages clarity.
It is acceptable to add RunHaven-specific typed methods behind
`AppServerSession`, as long as they follow the same request/response/event
pattern.

## Minimal Method Surface

The Codex app-server protocol has many methods. RunHaven needs only a subset
for the first full TUI.

### Keep And Implement

| Codex-shaped method | RunHaven behavior |
| --- | --- |
| `thread/start` | Start a local RunHaven TUI session for the current workspace. Load profile catalog and safe defaults. Do not launch a container by itself. |
| `thread/list` | List recent RunHaven runs from `runhaven_core::records::read_run_records` plus active run markers. |
| `thread/read` | Read a run record or active run record and map it to transcript/history items. |
| `thread/resume` | Reopen a run-history view or active-run view. Do not replay a Codex model context. |
| `thread/archive`, `thread/delete` | Optional later. If implemented, affect only RunHaven-owned local records and require clear confirmation for destructive behavior. |
| `thread/name/set` | Optional run label or display alias if RunHaven adds labels. |
| `thread/settings/update` | Update selected workspace, agent profile, network mode, auth scope, display settings, or other TUI session settings. |
| `turn/start` | Execute a RunHaven action selected by the UI. Examples: build plan, start launch, refresh status, fetch logs, show diff, run diagnostics. |
| `turn/steer` | Optional. For a running RunHaven action, queue a follow-up such as "show more logs". Leave disabled until useful. |
| `turn/interrupt` | For active runs, map to a safe stop path only when a run id is known and validated. Keep kill as a distinct explicit action. |
| `review/start` | Show run diff/worktree review using RunHaven diff data. |
| `model/list` | Either return a small inert catalog for Codex UI compatibility or keep model UI disabled. Do not pretend agent profiles are LLM models unless the UI copy makes that clear. |
| `account/read` | Return local RunHaven auth/status display data, backed by `auth_status_payload` and provider sign-in metadata. |
| `config/read`, `config/batchWrite` | TUI display settings only. Do not expose arbitrary Codex config behavior. |
| `command/exec` | Optional for safe diagnostics. Prefer typed RunHaven service methods for product actions. |

### Add RunHaven-Specific Typed Methods Behind The Same Facade

These are not upstream Codex methods, but they keep the same typed facade style:

| RunHaven method | Backing code |
| --- | --- |
| `runhaven/agent/list` | `runhaven_core::runtime::profiles::profiles`, `AgentCatalogData::from_profiles`. |
| `runhaven/plan/build` | `runhaven_core::runtime::plans::build_run_plan`, `LaunchPlanData::from`. |
| `runhaven/launch/prepare` | Build and validate the final `AgentRunPlan`, then return it to the UI loop for foreground launch. |
| `runhaven/launch/start` | Reserved for future non-interactive or background launch. Foreground launch must not run inside the backend service task. |
| `runhaven/run/active/list` | `runhaven_core::runtime::active::read_active_run_records`. |
| `runhaven/run/status` | `runhaven_core::runtime::active::active_run_status_payload`. |
| `runhaven/run/logSnapshot` | `runhaven_core::runtime::active::active_run_log_snapshot_payload`. |
| `runhaven/run/stop` | `runhaven_core::runtime::active::stop_active_run`. |
| `runhaven/run/kill` | `runhaven_core::runtime::active::kill_active_run`. |
| `runhaven/run/repair` | `runhaven_core::runtime::active::repair_active_run`. |
| `runhaven/history/list` | `runhaven_core::records::read_run_records`. |
| `runhaven/history/read` | `runhaven_core::records::find_run_record`. |
| `runhaven/history/diff` | `runhaven_core::records::run_diff_text`. |
| `runhaven/diagnostics/read` | `read_egress_policy_log`, `read_auth_broker_log`, `auth_status_payload`. |
| `runhaven/doctor/read` | `runhaven_core::doctor::collect_checks`. |
| `runhaven/image/status` | Existing image readiness/status code from `runhaven-core::image`. |

Keep these extension methods inside `AppServerSession` or a typed sub-facade.
Do not call `runhaven-core` directly from widgets.

`AppServerSession` already exposes many Codex methods. Preserve the public
shape where possible even if a method returns a typed unsupported error. The
important source-close boundary is that `App` and `ChatWidget` keep talking to
`AppServerSession`, not directly to the RunHaven service.

### Leave Disabled Or Fail-Closed

| Codex method family | RunHaven default |
| --- | --- |
| `fs/*` | Disabled. Remote filesystem operations are not a RunHaven default. |
| `mcpServer/*` | Disabled unless a RunHaven MCP boundary is designed. |
| `plugin/*`, `marketplace/*`, `app/list` | Disabled unless RunHaven explicitly supports plugins/apps. |
| `hooks/*` | Disabled unless RunHaven adds a hook model. |
| `remoteControl/*` | Disabled. |
| `environment/add` | Disabled. |
| `account/login/*`, `account/logout` | Do not use Codex account login. RunHaven login is per agent through `runhaven login <agent>`. |
| `feedback/upload` | Disabled unless RunHaven has its own consent and redaction design. |
| `windowsSandbox/*` | Not relevant. RunHaven is macOS 26+ only. |
| Codex external-agent import | Disabled. RunHaven already manages external agent profiles. |

## Notification Mapping

RunHaven should emit Codex-shaped notifications so `App` and `ChatWidget` can
stay close to upstream:

| Notification | RunHaven use |
| --- | --- |
| `thread/started` | TUI session initialized, workspace/profile context loaded. |
| `thread/status/changed` | Active run state changed, launch step changed, diagnostics refreshed. |
| `turn/started` | A RunHaven action began. |
| `turn/completed` | A RunHaven action completed, failed, or was interrupted. |
| `item/started` | Start rendering a plan, command, log snapshot, diff, or diagnostic block. |
| `item/completed` | Final authoritative item for that block. |
| `item/agentMessage/delta` | Streaming text for logs, progress, or long diagnostic messages where stable/tail rendering helps. |
| `item/commandExecution/outputDelta` | Bounded command or log output chunks. |
| `turn/diff/updated` | Latest run diff text. |
| `turn/plan/updated` | Launch checklist or multi-step action progress. |
| `error` | Validation failures, image not built, active run missing, container inspect failure. |
| `serverRequest/resolved` | Approval or typed input request completed. |

Use `ServerRequest` values for interactive decisions:

- Lower-security launch confirmation.
- Stop/kill/repair confirmation.
- Future boundary escalation such as allowing a sensitive workspace.

Do not rely on ad hoc modal state outside the event model once the Codex app
loop is active.

## Launch Execution Rule

RunHaven launch is special because `launch_run_plan` starts an interactive
container command.

The TUI must not run an interactive agent inside the alternate screen while
Codex's event stream is still reading stdin.

Correct launch flow:

1. User confirms launch in Codex-shaped UI.
2. `AppServerSession` validates and builds a final `AgentRunPlan`.
3. `AppServerSession` returns that plan to the UI loop. The backend service
   does not start the foreground child process because it does not own raw
   terminal state.
4. The UI loop uses `Tui::with_restored(...)` or the same underlying
   pause/restore/resume sequence to drop the crossterm event stream, leave the
   alternate screen if active, restore terminal modes, and pause terminal
   stderr suppression.
5. TUI clears managed title/pet image before handing the terminal to the
   interactive process.
6. On the UI thread, RunHaven calls
   `runhaven_core::runtime::launch::launch_run_plan(&plan)`.
7. After the process exits, Codex terminal modes and event polling are restored
   only if the TUI is continuing. If launch exits the TUI, use Codex's exit
   cleanup path instead.
8. If launch fails before the child process takes over, restore the Codex
   terminal state and render the error as a TUI notice or popup. Do not leave
   raw mode leaked, and do not silently exit as if launch succeeded.
9. RunHaven writes the normal active/run records through existing core code.
10. A future post-run TUI can reopen from records, but the launch itself should
   use the same runtime path as the CLI.

If RunHaven later supports non-interactive background launches, that can stay
inside the TUI event loop, but it must use exact subprocess argument lists and
the existing run-record and active-marker cores.

# Tauri UI Guardrails

Status: active UI contract.

RunHaven has a first Tauri app scaffold with setup, dashboard, profile,
folder-pick, run-plan review, and an explicitly confirmed launch path. This
document defines the security, resource, and approval boundaries for every
WebView-controlled RunHaven operation.

Current release context: the desktop app remains alpha after the `v0.5.0`
CLI-only pre-release. The TUI is active in the current checkout; the desktop app
is deferred to a later first-class release phase. Each new desktop command still
needs typed Rust inputs, explicit confirmation where mutating, focused tests,
and a narrow Tauri capability.

## Source Evidence

- Tauri v2 security docs, reviewed 2026-06-16:
  <https://v2.tauri.app/security/>
- Tauri v2 capabilities docs, reviewed 2026-06-16:
  <https://v2.tauri.app/security/capabilities/>
- Tauri v2 permissions docs, reviewed 2026-06-16:
  <https://v2.tauri.app/security/permissions/>
- Current RunHaven CLI behavior, verified from `crates/runhaven-core/`,
  `crates/runhaven-cli/`,
  `docs/ARCHITECTURE.md`, `docs/SECURITY_MODEL.md`, and
  `docs/APPLE_CONTAINER_GAP_ANALYSIS.md`.

## Boundary Rules

- Treat the Tauri WebView as untrusted UI. Keep privileged behavior in typed
  Rust commands.
- Do not expose generic shell, process, filesystem, HTTP, or Apple `container`
  command bridges to JavaScript.
- Define one narrow Tauri command per RunHaven operation. Inputs must be typed
  enums, canonical paths, validated run ids, validated profile names, and
  explicit booleans for destructive confirmations.
- Keep capabilities deny-by-default. A window gets only the permissions it
  needs for its view.
- Explicitly list reviewed capability identifiers in `tauri.conf.*` and
  restrict registered commands in the app manifest so stray capability files or
  registered commands do not become broad defaults.
- Do not store provider tokens, SSH material, command lines, prompts, raw
  Apple inspect payloads, environment values, or workspace file contents in
  frontend state or browser storage.
- Keep remote content out of privileged WebViews. If remote content is ever
  needed, it must live in a separate no-privilege window.
- The secure path must be the default and easiest path. Supported advanced or
  risky paths should warn in plain language and require confirmation, but should
  not be hidden or blocked just because they are advanced.

## Initial Capability Shape

Use separate capability files as UI control surfaces are added:

| Capability | Intended window | Allowed operations |
| --- | --- | --- |
| `main-read` | main dashboard | setup status, profile list, active runs, run history summaries, launch planning |
| `folder-pick` | main dashboard | native directory picker only |
| `run-control` | run detail view | stop, kill, attach, logs-follow, repair for one validated active run id |
| `launch-run` | explicit run launch flow | plan and run with validated profile, workspace, session, network, resource, and credential options |
| `maintenance` | settings or maintenance view | state reset/prune, network prune, image rebuild, builder guidance after explicit confirmation |
| `release-admin` | disabled until release prep | update, install, uninstall, signing, notarization, updater, registry credential changes |

Do not merge these capabilities into one broad default window permission.

## Resource Guardrails

Apple `container` runs each container inside its own lightweight VM. The UI must
make resource impact visible before starting or rebuilding anything.

Current alpha launch gate:

- current active run count from `runhaven runs active --json`;
- selected run CPU and memory limits from the run plan, currently defaulting to
  `--cpus 4 --memory 4g`;
- selected network mode and egress summary from `runhaven plan`;
- selected workspace path and workspace scope;
- selected state volume and session;
- selected bundled profile image status and builder status from typed Rust
  image diagnostics;
- dynamic warnings when another RunHaven run is active and when the selected
  memory limit plus active runs could be material on the host;
- sanitized post-launch snapshot with run id, profile, workspace, state volume,
  network mode, and container name;
- typed live run-status snapshot with marker status, container state,
  resources, image, and network metadata from sanitized Rust status payloads;
- opt-in bounded raw-output snapshot for one validated active run, outside
  `main-read`, after sensitive-output acknowledgement;
- explicit confirmation of the reviewed plan;
- explicit confirmation for every warning returned by the plan;
- launch blocked when `runhaven doctor` fails.

Remaining launch-readiness gaps before this flow is complete:

- dedicated run controls. Raw logs must still not be shown automatically,
  streamed, or stored durably because agent output can contain secrets or
  workspace content.

Warnings:

- warn before full internet mode, additional provider hosts, disabled
  SSH-forwarding attempts, explicit env passthrough, sensitive workspaces, root
  user overrides, and worktree discard or merge flows;
- block launch if `runhaven doctor` fails.

The current dashboard command already returns setup, active-run, recent-run,
agent, and warning summaries. Image and builder status are available through
typed Rust commands. Launch resource warnings are computed in the Rust plan and
launch-confirmation path. Live run status is exposed through a typed read-only
Rust command that returns sanitized metadata only. Log viewing starts with a
bounded `get_log_snapshot` command that requires a sensitive-output
acknowledgement, returns capped container stdio for one validated active run,
and does not add live streaming. Add dedicated typed Rust commands for
maintenance status before parsing prose in the frontend.

## Approval Gates

Read-only by default:

- `doctor`;
- `setup`;
- `plan`;
- `agents`;
- `image doctor`;
- `runs active`;
- `runs status`;
- `runs list/show/log` for completed-run metadata, provider policy, and auth
  broker records;
- `network list`;
- `state list`;
- `auth status/explain/log`;
- `why host`;
- `egress log`.

Sensitive read acknowledgement required:

- bounded raw active-container stdio snapshots from `container logs`, exposed
  only through a typed `get_log_snapshot` command outside `main-read`.

Explicit confirmation required:

- `run`;
- `runs attach`;
- `runs stop`;
- `runs kill`;
- `runs repair`;
- `runs merge`;
- `runs discard`;
- `image build`;
- `image rebuild`;
- `state reset`;
- `state prune`;
- `network prune`;
- provider host additions;
- SSH-forwarding attempts while disabled;
- environment passthrough.

Not exposed in the desktop UI until a dedicated design exists:

- Apple `container system stop`;
- Apple `container` install, update, or uninstall;
- registry login, logout, push, or credential edits;
- managed `container machine` creation, deletion, mutation, or attachment;
- image deletion outside RunHaven-managed rebuild flows;
- volume deletion outside RunHaven-managed state commands;
- updater, signing, notarization, or release publication.

User-managed `container machine` workflows are not blocked solely because they
are less secure. They are outside the current desktop scope until RunHaven has
a warned, explicit, target-validated flow.

## Command Contract

Future Tauri commands should wrap existing Rust functions directly when
possible. If a sidecar CLI is used during the first UI pass, it must still use
fixed command shapes and validated arguments.

Each Tauri command needs:

- a typed request struct;
- a typed response struct;
- validation that matches or calls the current CLI validators;
- no raw shell strings;
- no unbounded path access;
- no raw Apple `container inspect` passthrough;
- tests for invalid input and confirmation gates.

## Acceptance Before UI Work

- This guardrail document is linked from the roadmap and Apple Container gap
  analysis.
- Tauri/UI research follows
  [`TAURI_UI_RESEARCH_PLAN.md`](TAURI_UI_RESEARCH_PLAN.md) before scaffold work.
- Tauri scaffold work starts with current official Tauri docs and pinned
  versions.
- The first `src-tauri/capabilities/` files and app manifest command list were
  reviewed against this document.
- UI command tests prove denied-by-default behavior before each new mutating
  operation is wired. The current `launch_run` command has confirmation tests
  and its own `launch-run` capability.
- Local Apple `container` smoke remains the runtime evidence source while
  hosted CI is disabled.

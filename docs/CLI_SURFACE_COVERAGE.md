# CLI Surface Coverage

Documented evidence that every RunHaven CLI command family is exercised and
confirmed live on macOS 26+ Apple silicon. Two repeatable scripts cover the full
surface; reverify with:

```bash
scripts/cli_surface_check.sh                                 # breadth: every command family
scripts/apple_container_smoke.sh --with-provider --with-ssh  # depth: provider egress denial + SSH fail-closed
```

Last full run on macOS 27.0 (build 26A5368g), Apple `container` 1.1.0:
`cli_surface_check.sh` 2026-07-13 (40/40 surfaces passed; the
session-volume reset path runs with `--auth-scope project` so it is exercised
under the `--auth-scope agent` default used for shared logins).
`apple_container_smoke.sh --with-provider --with-ssh`: 2026-07-01 passed.

## Coverage

| Command family | Confirmed by | Notes |
| --- | --- | --- |
| `agents` | surface check | lists the six bundled profiles with their support tiers (sign-in path, default network, API-key broker) |
| `doctor` | surface check + smoke | host prerequisites green on macOS 27.0 |
| `support` | surface check | creates one private text report in an isolated home; does not upload or open another app |
| `setup` | surface check | first-run guidance |
| `plan` | surface check + smoke | provider/internet defaults and security notices |
| `run` | smoke (internal and provider) | full launch and run-record lifecycle |
| `run --worktree` | surface check | create, then diff/keep/recover/merge/discard |
| `image build` / `image rebuild` | surface check (`--dry-run`) | build-command preview |
| `image doctor` | surface check + smoke | builder status |
| `network list` / `network prune` | surface check | list plus confirmation gate without `--yes` |
| `state list` / `state reset` / `state prune` | surface check + smoke | session-scoped cleanup |
| `runs list` / `show` / `log` | surface check | run records |
| `runs diff` | surface check | live worktree diff |
| `runs keep` / `recover` / `merge` / `discard` | surface check | worktree lifecycle |
| `runs active` / `status` / `attach` / `kill` / `repair` | surface check | active-run control |
| `runs logs-follow` / `stop` | smoke | streams active output, then graceful stop |
| `egress log` | surface check + smoke | provider proxy decisions |
| `auth status` / `explain` / `log` | surface check | broker boundary, no secrets |
| `login <agent>` / `login --clear` | live login smokes | per-agent isolated login (claude, codex, copilot, antigravity); needs real provider auth, so it is not in the self-cleaning breadth check |
| `why host` / `workspace` / `network` / `state` | surface check | safety explanations |

The deep provider egress-denial path (allowlist plus rejection of
non-allowlisted hosts, proxied IP literals, and direct egress) and the `--ssh`
fail-closed boundary are verified by the smoke. The surface check covers breadth
and the worktree and attach/kill/repair lifecycles the smoke does not exercise.

`runhaven login <agent>` runs a real in-sandbox provider OAuth flow, so it
cannot run in the self-cleaning breadth check. It is confirmed by live per-agent
login smokes (claude, codex, copilot, and antigravity were live-verified on
2026-06-26).

## Confirmation gates and safety

`cli_surface_check.sh` creates a temporary git workspace and uniquely named
sessions, then cleans up only the resources it created: its own runs, its own
session-scoped state volumes, and idle RunHaven-managed networks. It never prunes
user agent-home volumes (`state prune` is always session-scoped here).

# Authentication and API-key brokers

RunHaven's default sign-in path is the provider CLI's own OAuth flow inside the
agent container. RunHaven does not copy credentials from macOS, read provider
Keychain items, or store OAuth tokens on the host.

## Ordinary sign-in

In the desktop app, choose an agent and open its sign-in terminal. The same
commands are available from the CLI with `runhaven login <agent>`.

| Agent | Command inside the container | What the user does |
| --- | --- | --- |
| Claude | `claude auth login` | Open the printed URL, approve in the browser, then paste the returned code into the terminal. |
| Codex | `codex login --device-auth` | Open the printed URL, enter the one-time code, and approve. Codex polls until approval completes. |
| Antigravity | `agy` | Choose Google OAuth on first launch, approve in the browser, paste the returned code, finish first-run setup, then exit the agent. |
| Copilot | `copilot login` | Open GitHub's device page, enter the one-time code, and approve. Copilot waits for approval. |

The sign-in container uses:

- the non-root `agent` user;
- a read-only root filesystem;
- dropped Linux capabilities;
- provider-only egress by default;
- a small RunHaven-owned login workspace, not the user's project;
- the agent's isolated home volume.

A successful shared-agent sign-in remains available to later runs of that agent.
Project-scoped sign-in uses that project's separate agent state instead.

RunHaven relays terminal bytes directly to the app and does not write them to
RunHaven logs, run records, or status registries. The provider CLI may write its
own configuration, credential, or support-log files inside its isolated home
volume.

## Credential storage

OAuth credential storage is owned by each Linux provider CLI. It is protected by:

- isolation from the Mac home directory and Mac Keychain;
- an owner-only RunHaven container volume;
- no default host-home, browser-profile, SSH-key, cloud-credential, or arbitrary
  environment-variable mounts;
- no display or inspection of the stored credential by RunHaven.

The volume is not a hardware-backed secret store. Anyone with sufficient access
to the Mac account and Apple container data may be able to inspect it. Deleting
the agent's data volume removes the local credential and all other saved state
for that agent. Local deletion does not necessarily revoke the provider-side
authorization.

RunHaven writes a separate secret-free marker after a sign-in command exits
successfully. The GUI uses that marker for “Signed in” status. It means RunHaven
witnessed a successful sign-in, not that the provider token is still valid.

## Codex device authorization

Current OpenAI source confirms that `--device-auth` is a first-class login
flag. Codex requests a device code, prints the verification URL and one-time
code, then polls the token endpoint at the server-provided interval for up to 15
minutes. After approval it exchanges the authorization code and persists the
tokens under `CODEX_HOME`. In RunHaven, `CODEX_HOME` is within the isolated
agent home volume.

Source: [openai/codex device-code implementation](https://github.com/openai/codex/blob/c888e8e75a9f0e90ce7d5517f8b9540832cbbf76/codex-rs/login/src/device_code_auth.rs).

## Copilot device authorization

GitHub documents `copilot login` as an OAuth device flow. On headless Linux,
when no system credential store is available, Copilot can store its token in
`~/.copilot/config.json`. In RunHaven that path is inside Copilot's isolated
home volume.

Source: [GitHub Copilot CLI authentication](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli).

## Antigravity first-run authorization

Google documents first-run `agy` as an interactive flow: select Google OAuth,
complete browser sign-in, paste the returned code, choose a theme, accept the
terms, and confirm workspace trust. The real terminal is required for this full
sequence.

Source: [Google Antigravity CLI codelab](https://codelabs.developers.google.com/antigravity-cli-hands-on).

## Advanced API-key broker

Claude and Codex retain an optional host-side API-key broker for explicitly
configured advanced use:

```bash
runhaven run claude --network provider \
  --api-key-broker-env ANTHROPIC_API_KEY -- <agent args>

runhaven run codex --network provider \
  --api-key-broker-env OPENAI_API_KEY -- <agent args>
```

The plan stores only the environment-variable name. At launch, RunHaven reads
the value into memory, starts a narrow host broker, gives the container a
placeholder credential, and forwards only approved provider requests with the
real key. The key is never placed in the container command or guest environment.

This broker is not the ordinary sign-in path. RunHaven does not automatically
select it because an environment variable happens to exist. The user must
explicitly configure it, and provider-only network mode is required.

Copilot and Antigravity do not have a RunHaven API-key broker.

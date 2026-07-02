# Native GUI Claude OAuth-Token Broker (Design C)

Status: design only. No code. Adversarial review gate before any implementation.

This design adds a second Claude auth path to the desktop app: a macOS Keychain
OAuth-token broker (path C, the new default), alongside the existing in-sandbox
login (path A, opt-in). It reintroduces host credential custody that the
zero-custody refactor removed (commit `60e37db`), so every boundary and every
user-facing claim below is written to survive hostile review.

Label key: [S] verified source, [C] repo convention, [J] judgment.

## Decision (fixed, not relitigated here)

The Agents sign-in flow offers two Claude paths:

- Path C (default): RunHaven orchestrates `claude setup-token` on the host,
  stores the printed one-year token in a RunHaven-owned macOS Keychain item, and
  the broker proxy injects it as `Authorization: Bearer` at request time. The
  token never enters the guest.
- Path A (opt-in): the existing in-sandbox login. The user runs `claude auth
  login` inside the shared home volume; the credential persists there. No host
  custody. This is unchanged from today and stays available verbatim.

## Verified facts this design is built on

- [S] `claude setup-token` walks OAuth on the host and prints a one-year
  subscription token (inference only, no Remote Control). It saves the token
  nowhere. On a real Mac host it uses the browser plus a localhost callback, so
  there is no paste-back. Paste-back only appears in containers or over SSH where
  the callback cannot reach the local server. So the low-friction acquisition is
  running setup-token on the host, which is exactly what path C does.
- [S] Empirically verified on this machine (capture-server test): Claude Code
  routes `CLAUDE_CODE_OAUTH_TOKEN` auth through `ANTHROPIC_BASE_URL` and sends
  `Authorization: Bearer <token>` to `/v1/messages` with
  `anthropic-beta: ...oauth-2025-04-20...`. A host proxy at `ANTHROPIC_BASE_URL`
  can inject the real Bearer while the guest holds only a placeholder. This
  mirrors the existing API-key broker, which injects `x-api-key`.
- [S] The proxy core already supports Bearer injection.
  `crates/runhaven-core/src/provider/auth_broker/profiles.rs` defines
  `CredentialInjection::BearerAuth`, and
  `crates/runhaven-core/src/provider/auth_broker/headers.rs` already emits
  `Authorization: Bearer {api_key}` for it and strips any guest-sent
  `authorization` header (both as hop-by-hop and as an injected-name match). So
  the guest can never set or leak the injected header.
- [S] Prior art existed and was deleted in the zero-custody refactor:
  `runtime/token_store.rs` (a `TokenStore` trait with a macOS
  `KeychainTokenStore` using `security-framework`, service `RunHaven OAuth
  token`, account = agent) and `login.rs::login_claude`/`extract_token` (host
  `claude setup-token` capture). Both are prior art in git history at
  `60e37db^`. This design reintroduces a narrower version.

## Section 1: Keychain access mechanism

Choice: re-add the `security-framework` crate. [J]

Options weighed against the repo dependency posture and `unsafe_code = forbid`:

- `security-framework` crate: safe Rust bindings over the macOS Security
  framework. Prior art already used it and it compiled under `unsafe_code =
  forbid` (commit `2a3cb3f`). It exposes item-class, accessibility, and
  data-protection controls we need. It was already in the lock tree, so re-adding
  it is a small lock delta, not a new transitive graph. Chosen.
- macOS `security` CLI: no dependency, but it shells out and parses fragile
  human output, and passing the token as a process argument or stdin widens the
  surface (argv visibility, shell quoting). Rejected. [J]
- `keyring` crate: broader abstraction than we need, pulls extra platform
  backends we do not target, and gives less direct control over the macOS
  accessibility class and data-protection keychain. Rejected on the DRY
  build-necessity ladder. [C]

Pin: the prior art pinned `security-framework = "=3.5.1"`. [S] The exact current
stable pin is volatile; verify the latest stable at implementation time against
crates.io and the lockfile, add it to `pins.toml` exact-pinned and target-gated
to macOS, and lock the transitive tree. Do not carry `3.5.1` forward unchecked.

Item naming (load-bearing for the "we read only our item" claim): [J]

- Service: a dedicated RunHaven-owned label, for example `RunHaven Claude
  OAuth`. It must be distinct from any label Claude Code uses for its own login
  items. RunHaven creates, reads, updates, and deletes only this service.
- Account: `claude`. Room for future per-agent OAuth items without collision.
- Use the data-protection keychain (`kSecUseDataProtectionKeychain`) so the item
  is an access-controlled data-protection item, not a legacy file-keychain entry.

Accessibility class: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`. [J]

- Requires the login keychain to be unlocked when the broker reads the token,
  which matches "read only while the user is at their unlocked Mac."
- `ThisDeviceOnly` blocks iCloud Keychain sync and keychain-migration copies of a
  long-lived one-year token to other devices or backups. For a device-bound
  broker credential that is the correct, narrower default.

Access control (honest scope): the created item carries an ACL so the RunHaven
app is a trusted application for it. [J] Do not overclaim this as absolute
isolation. On macOS a same-user process can still be granted access by the user
through the standard keychain prompt, and a process running with RunHaven's own
code signature and entitlements is trusted by design. The honest boundary is:
other apps do not get silent access to this item, and RunHaven does not read
anyone else's items. Section 6 states the residual malware case plainly.

## Section 2: setup-token orchestration

Trigger: the GUI "Sign in to Claude" action (path C) calls a new Tauri command
(Section 4). The command, after a Rust-owned native confirmation, runs the host
`claude setup-token` as an exact subprocess argument list (never a shell
string), per the repo subprocess rule. [C] It reuses the deleted `login_claude`
shape (`60e37db^`) as prior art: locate the host `claude` binary, run
`setup-token`, capture stdout transiently, extract the token line.

Capture and transience: [J]

- Read `claude setup-token` stdout into a buffer that lives only for the
  extract-then-store window. Extract the token (the deleted `extract_token` took
  the last non-empty token-shaped line; reuse that logic and its test).
- Store the token to the Keychain item, then drop every transient copy.
- Zeroization, stated honestly. Rust `String`/`Vec<u8>` do not zero on drop, and
  `Command::output` hands back a `Vec<u8>` whose capacity and any prior realloc
  we do not control, plus the kernel pipe buffer is outside our reach. Best
  effort: wrap the extracted token in a `zeroize`-style `Zeroizing` guard so the
  final token buffer is overwritten on drop, and explicitly zero the stdout
  `Vec<u8>` after extraction. Residual that we must not claim to wipe: kernel
  pipe buffers, any reallocated capacity from before the zeroizing wrapper, and
  the `claude` process's own memory. Document this residual; do not say "fully
  wiped." Add `zeroize` as an exact pin if not already present. [J]

Fail-closed paths (each ends with no stored token and a plain-language error): [J]

- `claude` binary not installed on host: stop, tell the user to install Claude
  Code or use path A. No Keychain write.
- setup-token exits non-zero, or the user cancels the browser login: stop,
  report "sign-in did not finish; nothing was saved." No Keychain write.
- Output has no token-shaped line (malformed): stop, report "could not read a
  token; nothing was saved." No Keychain write.
- Keychain write fails: report the operation and agent only, never the token
  value; leave no partial state.

No secret to logs, records, or the output registry: [C] the token is never
written to a log line, the run-output registry, session records, diagnostics, or
any Tauri response. Reuse the existing redaction discipline. Error contexts name
only the operation and the agent, matching the deleted `token_store.rs` contract
("error messages must not carry the token value").

## Section 3: Broker proxy extension

The proxy machinery needs almost no change: `BearerAuth` injection and
`authorization` stripping already exist. The extension is a new Claude OAuth
profile plus a Keychain secret source. [S][J]

New profile (a sibling of `CLAUDE_BROKER` in `profiles.rs`), for example
`CLAUDE_OAUTH_BROKER`:

- `agent: "claude"`, `upstream_host: "api.anthropic.com"`,
  `path_rule: PathRule::Prefix("/v1/")` (same pinned host and path posture as the
  API-key Claude broker).
- `injection: CredentialInjection::BearerAuth` so `headers.rs` emits
  `Authorization: Bearer <token>` and strips any guest-sent `authorization`.
- `guest_redirect: GuestRedirect::EnvRedirect { base_url_env:
  "ANTHROPIC_BASE_URL" }`, matching the empirically verified routing.
- Placeholder: the guest holds `CLAUDE_CODE_OAUTH_TOKEN` set to
  `BROKER_PLACEHOLDER_VALUE` plus `ANTHROPIC_BASE_URL` pointed at the broker on
  the RunHaven provider network. The guest's placeholder Bearer is stripped
  host-side, so it is never usable. [S]

Coexistence with the x-api-key path: the two Claude brokers are mutually
exclusive per run and selected by which credential source is configured. The
API-key path continues to read the host env var through
`require_api_key_broker_secret` in `provider/runtime.rs`. The OAuth path instead
reads the token from the RunHaven Keychain item at broker-spawn time and passes
it as the proxy's `api_key` string (the field name is historical; it carries
whatever secret the injection strategy needs). The one real seam to add is a
secret source: today `require_api_key_broker_secret` reads `std::env::var`; the
OAuth path reads the Keychain item created in Section 1. Everything downstream
(`ApiKeyBrokerProxy::bind_for_profile`, `broker_request_headers`, path pinning,
client-subnet allowlist) is unchanged. [S]

Egress and provider network: `api.anthropic.com` is already the pinned upstream
for the Claude broker, and `platform.claude.com` / `claude.ai` are already
bundled provider hosts (noted in commit `60e37db`). The OAuth token exchange runs
on the host during setup-token, before any container exists, so it needs no guest
egress change. The broker's runtime egress is the same `api.anthropic.com` the
API-key path already allows. No egress widening. [S]

The token is read from the Keychain by the host broker process only, and only
the injected Bearer leaves the host, to `api.anthropic.com`. The guest holds a
placeholder and never sees the real token. [S][J]

## Section 4: Capability and command surface

New Tauri commands, isolated in their own capability, mirroring the existing
`agent-sign-in` capability posture (`windows: ["main"]`, single allow-permission,
native-dialog gated): [C]

- `start_claude_oauth_signin`: runs Section 2 on a background thread behind a
  Rust-owned native confirmation naming Claude explicitly. Returns a sign-in id
  the WebView polls. Returns no token, ever.
- `get_claude_oauth_status`: returns a sanitized enum only (for example
  `NotSignedIn`, `Signing`, `SignedIn`, `Failed(reason)`). The status must be
  derived from Keychain item presence, never from the token value, and must never
  include the token or any prefix of it.
- `clear_claude_oauth`: deletes the RunHaven Keychain item (Section 1) after a
  native confirmation. Reports cleared or already-absent. Deletes only RunHaven's
  own item.

Capability scoping: a dedicated `claude-oauth.json` capability, `windows:
["main"]` only, isolated from run-control and from `agent-sign-in` so the
credential-adjacent trigger keeps its own minimal grant. [C] Native-dialog gating
reuses `confirm_native`. No secret crosses to the WebView in any command
response: the contract types carry only ids, sanitized status, and plain error
strings. [J]

## Section 5: GUI and messaging

The Agents "Sign in to Claude" control defaults to path C and offers path A as an
explicit opt-in, so the secure, low-friction path is the default (the host
setup-token flow has no paste-back on a real Mac). [S][J]

Presentation: a primary "Sign in to Claude" button (path C) with a secondary
"Sign in inside the sandbox instead" opt-in (path A). Choosing A runs the
existing in-sandbox login unchanged; RunHaven holds no token and the credential
lives in the shared home volume.

Tooltip strings (each checked against the mechanism above): [J]

- Path C button: "Sign in on your Mac. RunHaven saves your Claude token to your
  Mac's Keychain and uses it to sign your runs. The token never goes inside the
  sandbox."
- Path A opt-in: "Sign in inside the sandbox instead. Your Claude login is saved
  in the agent's own space, not on your Mac's Keychain."
- Clear action: "Remove the Claude token RunHaven saved. This deletes only
  RunHaven's Keychain item, not your other logins."

Help/docs outline (ship in the same slice as the behavior, per the docs-as-
product rule): [C]

- What each path stores and where (Keychain item vs shared home volume).
- The exact "does and does not do" section below.
- How to remove the token, and what removal deletes.
- Update `docs/SECURITY_MODEL.md`, `docs/AUTH_BROKER.md`, `docs/DESKTOP_APP.md`,
  and `docs/USAGE.md`, which the zero-custody refactor edited, to describe path C.

## What RunHaven does and does not do with your token

Mirror of the terminal see/cannot-see section, with exact, non-overclaiming
wording. Claims validated against Sections 1 to 3:

Approved wording:

- "RunHaven saves your token to your Mac's Keychain and keeps no copy of its own
  outside the Keychain." (True: the only persistent store is the Keychain item;
  transient in-memory copies during sign-in are minimized and dropped, and the
  token is never written to a file, log, record, or app response.)
- "RunHaven reads only the one Keychain item it created. It does not read your
  other Keychain logins, including Claude's own saved login." (True: the query is
  scoped to RunHaven's own service and account.)
- "RunHaven uses your token only to sign your runs. It does not log it, display
  it, or save a copy of its own." (True per Sections 2 and 4.)
- "Your token never goes inside the sandbox. The sandbox holds only a placeholder;
  RunHaven adds the real token on your Mac, only for requests to Anthropic."
  (True per the verified routing and header injection.)

Rejected wording (would be false, flagged for review):

- "RunHaven never sends your token anywhere." REJECTED as overclaiming. The
  broker does send the token, as `Authorization: Bearer`, to `api.anthropic.com`
  on every run. That is the entire point of the broker. Correct wording:
  "RunHaven sends your token only to Anthropic, to authorize your runs, and
  nowhere else."
- "Only RunHaven can ever read this Keychain item." REJECTED as overclaiming.
  See Section 1 access-control honesty and Section 6: a same-user process with
  RunHaven's identity, or one the user grants access to at a keychain prompt, is
  a real case. Correct wording: "Other apps do not get silent access to this
  item."
- "The token is fully wiped from memory after sign-in." REJECTED. See the
  Section 2 zeroization limits. Correct wording: "RunHaven overwrites its own
  copy of the token as soon as it is saved," with the residual documented.

## Section 6: Security tradeoff analysis (feeds adversarial review)

This design reverses the zero-custody stance. State that plainly. [J]

What changes: RunHaven again holds a real, long-lived (one-year) Claude
credential on the host, in the macOS Keychain, under a RunHaven-owned item. The
zero-custody refactor's whole point was that RunHaven held no agent token; path C
reintroduces custody for Claude only, and only in the OS vault.

Attack surface: [J]

- Malware running as the same user with RunHaven's code identity, or a process
  the user grants access to at a keychain prompt, could read the item. The OS
  vault raises the bar (locked-when-locked, this-device-only, ACL) but does not
  make theft impossible.
- Blast radius if stolen: a one-year inference-only subscription token. No Remote
  Control, no account takeover, but real inference cost and quota abuse until the
  user revokes it in their Claude account. Document how to revoke and that
  `clear_claude_oauth` removes RunHaven's stored copy but does not revoke the
  token server-side.

Why it is acceptable: [J]

- The token lives in the OS Keychain, not a plaintext file, access-controlled and
  device-bound, which is strictly better than the deleted `0600` file store.
- The token never enters the guest. A compromised or malicious agent in the
  sandbox sees only a placeholder and a broker base URL.
- The credential is inference-only, bounding the blast radius.
- Sign-in acquisition runs on the host with the official Anthropic tool; RunHaven
  never handles the user's Claude password or OAuth session, only the printed
  token.

Comparison to path A: [J] path A keeps zero host custody (credential lives in the
sandbox home volume), but requires an in-sandbox paste-back flow that is more
friction, and the credential then lives in a shared volume that any run using
that volume can read. Path C trades a smaller, OS-vaulted host custody for a
low-friction default and a guest that never holds the token. Both are offered;
the user chooses.

## Section 7: Slice plan with adversarial-review gates

Headline: land the Keychain store and host capture first (custody surface), then
the proxy profile, then the GUI, with an adversarial-review gate before any code
and before the GUI default flips to path C.

- Gate 0 (before any code): adversarial review of this doc. Confirm the claim set
  and the custody reversal are acceptable. Blocking.
- Slice 1: `security-framework` re-add (verified pin, macOS-gated, `pins.toml`,
  lock), Keychain store module with the Section 1 item naming, accessibility
  class, ACL, and store/read/clear. Unit tests for selection and round-trip
  behind a live-Keychain follow-up. No GUI, no proxy wiring yet.
- Slice 2: host `claude setup-token` capture with the Section 2 transience,
  zeroization guard, and fail-closed paths. Tests reuse `extract_token`. Assert
  no token reaches logs, records, or registry.
- Slice 3: `CLAUDE_OAUTH_BROKER` profile plus the Keychain secret source seam in
  `provider/runtime.rs`. Tests: Bearer injection, placeholder stripping, path and
  host pinning, no egress widening.
- Gate 1 (before GUI default flip): adversarial and security review of Slices 1
  to 3 together, focused on the custody boundary, the claim set wording, and the
  zeroization residual. Blocking.
- Slice 4: Tauri commands and `claude-oauth` capability (Section 4), native
  confirmation, sanitized status, no secret in responses.
- Slice 5: GUI control, tooltips, and docs (Section 5), with path C as default
  and path A as opt-in. Ship docs in the same slice.
- Gate 2 (pre-merge): full verification per `AGENTS.md`, plus a live Keychain
  round-trip and a real setup-token sign-in on macOS 26+, and a final read of the
  approved claim set against the shipped strings.

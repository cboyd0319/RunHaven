# Native GUI Claude OAuth-Token Broker (Design C)

Status: Slice 1 (custody surface, Section 2 capture folded in), Slice 3
(broker proxy extension: `CLAUDE_OAUTH_BROKER` profile, `ClaudeAuthSource`
selector, Keychain secret-source seam, F5 decision), Slice 4 (the three Tauri
commands plus the isolated `claude-oauth` capability), and Slice 5 (GUI: path C
default + path A opt-in, the non-secret token-age rotation prompt, and the docs)
are implemented. Slice 2 (host capture) landed inside Slice 1. Gate 1 passed and
authorized the GUI default flip to path C, which Slice 5 made. Gate 2 (pre-merge
full verification plus the live-Keychain / real-setup-token maintainer smokes and
a final read of the approved claim set against the shipped strings) and the
double-coverage review remain before merge.

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
  login` inside the shared home volume; the credential persists there, with no host
  custody. Unchanged from today.

## Verified facts this design is built on

- [S] `claude setup-token` walks OAuth on the host and prints a one-year token
  (inference only, no Remote Control), saving it nowhere. On a real Mac it uses the
  browser plus a localhost callback, so there is no paste-back. Running it on the
  host is the low-friction acquisition, exactly what path C does.
- [S] Empirically verified on this machine (capture-server test): Claude Code
  routes `CLAUDE_CODE_OAUTH_TOKEN` auth through `ANTHROPIC_BASE_URL` and sends
  `Authorization: Bearer <token>` to `/v1/messages` with
  `anthropic-beta: ...oauth-2025-04-20...`. A host proxy at `ANTHROPIC_BASE_URL`
  injects the real Bearer while the guest holds only a placeholder, mirroring the
  existing API-key broker's `x-api-key` injection.
- [S] The proxy core already supports Bearer injection.
  `provider/auth_broker/profiles.rs` defines `CredentialInjection::BearerAuth`,
  and `provider/auth_broker/headers.rs` already emits `Authorization: Bearer
  {api_key}` for it and strips any guest-sent `authorization` header (hop-by-hop
  and injected-name match), so the guest can never set or leak the injected header.
- [S] Prior art existed and was deleted in the zero-custody refactor:
  `runtime/token_store.rs` (a `TokenStore` trait with a macOS `KeychainTokenStore`
  using `security-framework`, service `RunHaven OAuth token`, account = agent) and
  `login.rs::login_claude`/`extract_token` (host `claude setup-token` capture),
  both in git history at `60e37db^`. This design reintroduces a narrower version.

## Section 1: Keychain access mechanism

Choice: re-add the `security-framework` crate. [J] Options weighed against the
repo dependency posture and `unsafe_code = forbid`:

- `security-framework` crate: safe Rust bindings over the macOS Security
  framework. Prior art already used it and it compiled under `unsafe_code =
  forbid` (commit `2a3cb3f`). It exposes item-class, accessibility,
  data-protection, and `SecAccessControl` user-presence controls we need (its
  `access_control` module: `ProtectionMode` and
  `SecAccessControl::create_with_protection`). It was already in the lock tree,
  so re-adding it is a small lock delta, not a new transitive graph. Chosen.
- macOS `security` CLI: no dependency, but it shells out and parses fragile human
  output, and passing the token as a process argument or stdin widens the surface
  (argv visibility, shell quoting). Rejected. [J]
- `keyring` crate (3.6.x / 4.x cached, actively maintained, the popular
  cross-platform store): the right tool for a simple password store, but it
  abstracts away the control the hardened requirements need. [S] Its macOS backend
  uses the LEGACY file-based `SecKeychain`, not the data-protection keychain, and
  its API is only set/get/delete_password, with no `SecAccessControl`, biometry,
  `kSecAttrAccessible`, `kSecUseOperationPrompt`, or `LAContext` in its source. So
  it offers no user-presence gate (Hardening 1), no accessibility-class control
  (the no-iCloud-sync `AccessibleWhenUnlockedThisDeviceOnly`), no data-protection
  keychain, and no exact service + account query filter (claim 2). Since it itself
  builds on `security-framework`, using the latter directly is a
  level-of-abstraction decision, not a hand-rolled binding. Rejected. [C]

Pin: the prior art pinned `security-framework = "=3.5.1"`. [S] The exact current
stable pin is volatile; verify the latest stable at implementation time against
crates.io and the lockfile, add it to `pins.toml` exact-pinned and target-gated
to macOS, and lock the transitive tree. Do not carry `3.5.1` forward unchecked.

Item naming (load-bearing for the "we read only our item" claim): [J]

- Service: a dedicated RunHaven-owned label, e.g. `RunHaven Claude OAuth`,
  distinct from any label Claude Code uses. RunHaven creates, reads, updates, and
  deletes only this service. Account: `claude` (room for future per-agent items).
- Use the data-protection keychain (`kSecUseDataProtectionKeychain`), not a legacy
  file-keychain entry.
- Query filter (load-bearing for claim 2, "reads only the one item it created"):
  every read and delete uses an exact `service` + `account` match, never
  `kSecMatchLimitAll` or an account-only query. A Slice 1 test asserts the query
  pins both attributes and never sets `kSecMatchLimitAll`.

Protection and per-read user presence (`kSecAttrAccessControl`, Hardening 1): [J]
the item's `kSecAttrAccessControl` is a `SecAccessControl` built via
`security-framework`'s `access_control` module:
`SecAccessControl::create_with_protection(Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly), flags)`.
Two properties combine on one item:

- Protection `AccessibleWhenUnlockedThisDeviceOnly`: readable only while the login
  keychain is unlocked, and never synced to iCloud Keychain or copied by keychain
  migration or backup. The correct, narrower default for a device-bound one-year
  token.
- Flag `kSecAccessControlUserPresence` (bit 0): every read triggers a
  LocalAuthentication check (Touch ID, Apple Watch, or the device password).
  Chosen over `kSecAccessControlBiometryCurrentSet` / `BiometryAny`: user presence
  accepts the device password as a fallback, so it does not lock out Macs without
  Touch ID or with biometry off (biometry-only would exclude valid hardware,
  failing "secure path is the easy path") while still forcing a live approval. [S]
  Verify at pin time whether `security-framework-sys` exports named
  `kSecAccessControl*` constants; if not, pass the raw `CFOptionFlags` bit value.
  This stays inside `unsafe_code = forbid`: the crate owns any FFI.

Read model, one prompt per run: [J] the broker reads the token from the Keychain
once, at broker-spawn when a run launches, which triggers one user-presence
prompt. The plaintext is then held in `BrokerState` (Section 3) for the run's
lifetime, so per-request Bearer injection reuses the cached value with no
re-prompt. Prompt copy (8th grade), set via `kSecUseOperationPrompt`: "RunHaven
wants to use your Claude token to run <agent>." Optional (needs FFI check): an
`LAContext` `touchIDAuthenticationAllowableReuseDuration` can suppress a second
prompt for rapid back-to-back launches; deferrable without weakening the gate.

Security effect (plain, not overclaimed): the user-presence read means even
malware running as the same user with RunHaven's own code identity cannot SILENTLY
read the token; it gates the READ, and so each run's use. This is the strongest
single mitigation of Gate 0's residual "malware as same user" case, though not
theft-impossible for someone holding the unlocked device (Section 6).

Cross-app isolation and signing: [J] silent cross-app access is scoped by the
entitlement identity of a signed, notarized build; that isolation is a security
gate covered under Future / release hardening below and is not claimed for an
unsigned dev build. Dev-build fallback (F3, RESOLVED by a live probe): a
`SecAccessControl` is a data-protection-keychain feature (`kSecAttrAccessControl`);
the LEGACY file keychain REJECTS an item that carries one, so an unsigned dev
build (which cannot use the data-protection keychain and falls back to the legacy
keychain, `select_backend`) stores the token WITHOUT the user-presence gate, and
a signed release (data-protection keychain) stores it WITH the gate
(`access_for_backend`). Both keep `AccessibleWhenUnlockedThisDeviceOnly`. Earlier
wording that "the gate applies wherever `SecAccessControl` binds / binds in both"
was wrong: attaching a gate on the legacy keychain fails the store outright. So
the "cannot read the token silently" guarantee is scoped to signed release, as
the claim set already states; the dev build stores a usable, presence-free token.

Token rotation and bounded lifetime (Hardening 4): [J] `claude setup-token` mints
a static one-year token with no refresh (Gate 0 fact), so RunHaven owns aging it.
Store a NON-SECRET created-at timestamp in the item's own non-secret attributes
(e.g. `kSecAttrCreationDate` or a RunHaven label) or a small non-secret record;
never a secret-derived value, and never in the run marker (F1). A configurable
re-auth cadence (default 90 days: short enough to bound a leaked token, long
enough not to nag) lets RunHaven show token age and, once past the cadence, prompt
a fresh setup-token instead of using the stale one. It is a setting, not a forced
expiry, and the timestamp never reaches the marker or records.

## Section 2: setup-token orchestration

Trigger: the GUI "Sign in to Claude" action (path C) calls a new Tauri command
(Section 4). After a Rust-owned native confirmation, the command runs the host
`claude setup-token` as an exact subprocess argument list (never a shell string),
per the repo subprocess rule. [C] It reuses the deleted `login_claude` shape
(`60e37db^`): locate the host `claude` binary, run `setup-token`, capture stdout
transiently, extract the token line.

Capture and transience: [J]

- Read setup-token stdout into a buffer that lives only for the extract-then-store
  window; extract the token (the deleted `extract_token` took the last non-empty
  token-shaped line; reuse that logic and its test), store it to the Keychain item,
  then drop every transient copy.
- Zeroization plus type-enforced redaction, stated honestly (Hardening 3). Rust
  `String`/`Vec<u8>` do not zero on drop, `Command::output` hands back a `Vec<u8>`
  whose capacity and prior reallocs we do not control, and the kernel pipe buffer is
  outside our reach. Two exact-pinned, all-platform crates cover this, both composing
  with `unsafe_code = forbid` (each encapsulates its own unsafe): `zeroize` wraps the
  transient PTY read buffers (`Zeroizing<Vec<u8>>`/`Zeroizing<String>`), overwritten
  by a volatile write on drop; and `secrecy::SecretString` holds the two long-lived
  credential values, the broker's held copy (`BrokerState.api_key`, Section 3) and
  the extracted setup-token, from the Keychain/env read to injection. `SecretString`
  composes zeroize's wipe-on-drop AND adds compile-time protection: its `Debug` is
  `[REDACTED]` and it has no `Display`, so a stray `{:?}`/`{}` on the state or the
  token cannot leak it. Plaintext is reachable only through explicit, greppable
  `expose_secret()` calls: exactly one at header injection (the broker) and one at
  the Keychain store (the capture). Residual we must not claim to wipe: kernel pipe
  buffers, prior reallocated capacity, and the `claude` process memory. Not "fully
  wiped."

Piped-output prerequisite (F4): RESOLVED by a live probe on macOS 26+.
`claude setup-token` is a TTY-required Claude Code (ink) POLLING flow: with a
non-TTY stdout its UI and OAuth polling loop never run, so a plain pipe HANGS and
prints no token. Under a real controlling terminal the user authorizes in the
browser (no paste, no stdin) and setup-token polls `platform.claude.com` and
prints the one-year token. So the capture runs it under a host PTY
(`provider/claude_oauth_capture.rs`, the same `pty-process` primitive
`runtime/terminal.rs` uses), reads UNTIL the token appears (setup-token does not
exit promptly after printing it), extracts it, and reaps the child. Path C stays
one-click: authorize in the browser, no paste.

Parse-failure discipline (`extract_token`): on ANY parse failure the error emits
only the operation and the agent, never the captured buffer, because the buffer
may itself BE the token. Slice 2 test asserts the failure path logs no buffer. [J]

Fail-closed paths (each ends with no stored token, no partial state, and a
plain-language error): [J] `claude` not installed: stop, tell the user to install
Claude Code or use path A. setup-token exits non-zero or the user cancels:
"sign-in did not finish; nothing was saved." No token-shaped line: "could not read
a token; nothing was saved." Keychain write fails: report the operation and agent
only, never the token value.

No secret to logs, records, or registry: [C] the token is never written to a log,
the run-output registry, session records, diagnostics, or any Tauri response.
Error contexts name only the operation and agent, matching the deleted
`token_store.rs` contract ("error messages must not carry the token value").

## Section 3: Broker proxy extension

The proxy machinery needs almost no change: `BearerAuth` injection and
`authorization` stripping already exist. The extension is a new Claude OAuth
profile plus a Keychain secret source. [S][J]

New profile (a sibling of `CLAUDE_BROKER` in `profiles.rs`), e.g.
`CLAUDE_OAUTH_BROKER`:

- `agent: "claude"`, `upstream_host: "api.anthropic.com"`,
  `path_rule: PathRule::Prefix("/v1/")` (same pinned host and path as the API-key
  Claude broker).
- `injection: CredentialInjection::BearerAuth` so `headers.rs` emits
  `Authorization: Bearer <token>` and strips any guest-sent `authorization`.
- `guest_redirect: GuestRedirect::EnvRedirect { base_url_env:
  "ANTHROPIC_BASE_URL" }`, matching the verified routing.
- Placeholder: the guest holds `CLAUDE_CODE_OAUTH_TOKEN = BROKER_PLACEHOLDER_VALUE`
  plus `ANTHROPIC_BASE_URL` at the broker; the placeholder Bearer is stripped
  host-side, so it is never usable. [S]
- Beta header (F5): `BearerAuth` injects only `Authorization: Bearer`
  (`headers.rs:49-51`), but OAuth requests also need
  `anthropic-beta: ...oauth-2025-04-20...`. Today the guest CLI supplies it and
  the broker forwards it (not in the strip lists), so the OAuth profile depends on
  that guest header; if Anthropic requires it server-side, inject it host-side by
  extending `BearerAuth` with a fixed-header list mirroring `ApiKeyHeader.extra`.
  Not a leak either way: only the injected `Bearer` plus this non-secret beta tag
  leave the host. [S][J]

Coexistence with the x-api-key path: the two Claude brokers are mutually exclusive
per run, selected by credential source. The one real seam to add is that source:
today `require_api_key_broker_secret` in `provider/runtime.rs` reads
`std::env::var`; the OAuth path reads the Keychain item (Section 1) at broker-spawn
and passes it as the proxy's `api_key` string (historical field name; it carries
whatever secret the injection needs). Everything downstream (`bind_for_profile`,
`broker_request_headers`, path pinning, client-subnet allowlist) is unchanged.
Egress is unchanged too: `api.anthropic.com` is the pinned broker upstream, and the
OAuth exchange runs on the host before any container exists, so no guest egress
widens. [S]

### Plan selector and no-persist constraint (F1, load-bearing)

The OAuth token MUST NEVER populate `api_key_broker_env` or any other persisted
plan field. That field holds an env-var NAME today (harmless) and the run marker
persists it (`runtime/active/markers.rs:31-33`), so an OAuth token there would
leak to disk. Selection uses a distinct, non-secret selector instead.

Replace the bare `api_key_broker_env: Option<String>` on `AgentRunPlan` and
`RunOptions` (`crates/runhaven-core/src/runtime/plans/types.rs:142,166`) with a
non-secret enum:

```
enum ClaudeAuthSource {
    ApiKeyBrokerEnv(String), // env var NAME only (e.g. "ANTHROPIC_API_KEY"); non-secret; today's value
    KeychainOAuth,           // no payload; token fetched from Keychain at broker-spawn, never on the plan
}
```

Carry it as `claude_auth_source: Option<ClaudeAuthSource>`.
`require_api_key_broker_secret_with_env` (`provider/runtime.rs:211`) matches the
variant: `ApiKeyBrokerEnv(name)` keeps today's `std::env::var` read and the
`name == profile.host_env` gate; `KeychainOAuth` reads the Keychain item and no
env. One enum cannot select both, so an implementer cannot reach
`api_key_broker_env` on the OAuth path; the token flows from the Keychain read into
the proxy's `api_key` string, never onto a plan field.

Marker constraint (`runtime/active/markers.rs:31-33`): `ApiKeyBrokerEnv(name)`
persists `api_key_broker_env = name` as today (harmless env NAME; preserves the
read-model test). `KeychainOAuth` persists at most a non-secret tag
(`"claude_auth_source": "keychain-oauth"`) and NEVER the token, the Keychain
service or account, or any secret-derived value. Hard rule: `markers.rs` gains no
token-derived field. The run record path is already safe
(`records/run_history.rs:71-81` writes only count summaries), so the marker is the
only exposed persistence surface; a Slice 3 test asserts a `KeychainOAuth` marker
holds only the `keychain-oauth` tag. The token is read by the host broker only;
only the injected Bearer leaves the host, to `api.anthropic.com`. [S][J]

## Section 4: Capability and command surface

New Tauri commands, isolated in their own capability, mirroring the existing
`agent-sign-in` posture (`windows: ["main"]`, single allow-permission,
native-dialog gated): [C]

- `start_claude_oauth_signin`: runs Section 2 on a background thread behind a
  Rust-owned native confirmation naming Claude explicitly. Returns a sign-in id
  the WebView polls. Returns no token, ever.
- `get_claude_oauth_status`: returns a sanitized enum only (e.g. `NotSignedIn`,
  `Signing`, `SignedIn`, `Failed(reason)`), derived from Keychain item presence,
  never from the token value, and never including the token or any prefix.
- `clear_claude_oauth`: deletes the RunHaven Keychain item (Section 1) after a
  native confirmation. Reports cleared or already-absent. Deletes only RunHaven's
  own item.

Capability scoping: a dedicated `claude-oauth.json` capability, `windows:
["main"]` only, isolated from run-control and `agent-sign-in`, native-dialog gated
via `confirm_native`. [C] No secret crosses to the WebView: response types carry
only ids, sanitized status, and plain error strings. [J]

## Section 5: GUI and messaging

The Agents "Sign in to Claude" control defaults to path C (secure, low-friction:
host setup-token has no paste-back on a real Mac) and offers path A as an explicit
opt-in. [S][J] Path C is a primary button; path A is a secondary "Sign in inside
the sandbox instead" opt-in that runs the existing in-sandbox login unchanged, with
RunHaven holding no token and the credential in the shared home volume.

Tooltip strings (each checked against the mechanism above): [J]

- Path C button: "Sign in on your Mac. RunHaven saves your Claude token to your
  Mac's Keychain and uses it to sign your runs. The token never goes inside the
  sandbox."
- Path A opt-in: "Sign in inside the sandbox instead. Your Claude login is saved
  in the agent's own space, not on your Mac's Keychain."
- Clear action: "Remove the Claude token RunHaven saved. This deletes only
  RunHaven's Keychain item, not your other logins."

Help/docs outline (ship in the same slice, per the docs-as-product rule): [C] what
each path stores and where; the "does and does not do" section below; how to remove
the token and what removal deletes. Update `docs/SECURITY_MODEL.md`,
`docs/AUTH_BROKER.md`, `docs/DESKTOP_APP.md`, and `docs/USAGE.md` (which the
zero-custody refactor edited) to describe path C.

## What RunHaven does and does not do with your token

Mirror of the terminal see/cannot-see section, validated against Sections 1 to 3.

Approved wording (the quoted strings ship):

- "RunHaven saves no copy of its own; the only saved copy is in your Mac's
  Keychain." (PERSISTENCE only: never a file, log, record, or response; see the
  residual below.)
- "RunHaven reads only the one Keychain item it created. It does not read your
  other Keychain logins, including Claude's own." (True only under the exact
  service + account query filter, Section 1.)
- "RunHaven uses your token only to sign your runs. It does not log it, display
  it, or keep any saved copy outside your Mac's Keychain." (Persistence claim.)
- "Your token never goes inside the sandbox. The sandbox holds only a placeholder;
  RunHaven adds the real token on your Mac, only for requests to Anthropic."
- "RunHaven can use your Claude token only after you approve it with Touch ID or
  your device password. It cannot use your token silently." (The user-presence
  access control gates every read, Section 1; the broker reads once per run
  launch. It gates the READ, not encryption against someone holding your unlocked
  Mac who approves the prompt, so it is not a theft-impossible claim.)

In-memory residual (honest, F2): the claims above are PERSISTENCE claims; during a
run the broker holds the plaintext token in memory (`BrokerState.api_key`). Section
6 details the `Zeroizing` mitigation and its limits. The honest claim stays
"overwrites its own copy," not "wiped from memory."

Rejected wording (would be false). These forbidden strings are quoted here only
as design guardrails, paired with the correct alternative; the no-overclaim rule
governs shipped UI and user docs (verified clean), not this rejected-examples
list:

- "RunHaven never sends your token anywhere." REJECTED: the broker sends it as
  `Authorization: Bearer` to `api.anthropic.com` on every run. Correct: "RunHaven
  sends your token only to Anthropic, to authorize your runs, and nowhere else."
- "Only RunHaven can ever read this Keychain item." REJECTED: a user who approves
  the presence prompt, or an attacker holding the unlocked device, is a real case.
  Correct, scoped to a signed release build: "other apps do not get silent access,
  and RunHaven cannot read the token without your Touch ID or device-password
  approval." Dev-build isolation NOT claimed (Section 1).
- "The token is fully wiped from memory after sign-in." REJECTED (Section 2).
  Correct: "RunHaven overwrites its own copy of the token as soon as it is saved."

## Section 6: Security tradeoff analysis (feeds adversarial review)

This design reverses the zero-custody stance. State that plainly. [J] RunHaven
again holds a real one-year Claude credential on the host, in the macOS Keychain,
under a RunHaven-owned item. The zero-custody refactor's whole point was that
RunHaven held no agent token; path C reintroduces custody for Claude only, and
only in the OS vault.

Attack surface: [J]

- Same-user malware with RunHaven's code identity could ATTEMPT a read, but the
  user-presence access control (Section 1, the main mitigation) forces a live Touch
  ID / device-password approval on every read, so it cannot read SILENTLY. The OS
  vault (locked-when-locked, this-device-only, per-read user presence, plus
  cross-app isolation on signed builds) raises the bar substantially but does not
  make theft impossible: a user who approves the prompt, or an attacker holding the
  unlocked device, is trusted by design.
- During a run the broker holds the plaintext token in memory
  (`BrokerState.api_key`, `provider/auth_broker.rs:56-99`) because injection needs
  it. `Zeroizing` (Hardening 3) overwrites RunHaven's own copy on drop, but a host
  memory-scraper in the run window could read the live copy. Inherent to injecting
  the credential; documented, not claimed away.
- Blast radius if stolen: a one-year inference-only subscription token. No Remote
  Control or account takeover, but real inference cost and quota abuse until the
  user revokes it. Document how to revoke, and that `clear_claude_oauth` removes
  RunHaven's stored copy but does not revoke server-side.

Why it is acceptable: [J] the token lives in the OS Keychain, device-bound and
gated behind a per-read user-presence check (strictly better than the deleted
`0600` file store: no silent same-user read); it never enters the guest, which
sees only a placeholder and a broker base URL; it is inference-only, and the
rotation cadence (Section 1) bounds how long a leaked token stays usable; and
sign-in runs on the host with the official Anthropic tool, so RunHaven never
handles the user's Claude password or OAuth session, only the printed token.

Comparison to path A: [J] path A keeps zero host custody but requires a
higher-friction in-sandbox paste-back, and the credential then lives in a shared
volume any run using it can read. Path C trades a smaller, OS-vaulted,
presence-gated host custody for a low-friction default and a guest that never
holds the token. Both are offered; the user chooses.

## Future / release hardening (path 2): sign and notarize

Recorded as a SECURITY requirement, not distribution polish, and not designed
here. [J] The cross-app isolation guarantee ("other apps do not get silent access
to this item") binds only on a signed and notarized release build (Gate 0 F3): the
`keychain-access-group` / `application-identifier` entitlement identity scopes the
data-protection item, and an unsigned dev build lacks it, so notarizing the
release is a security gate for that one claim. It is already tracked as a later
release gate in `current-state.md`. Interim: the dev-build fallback (Section 1)
stands, with the isolation claim scoped to signed builds. The per-read
user-presence gate (Hardening 1) does not depend on signing and applies on both.

## Section 7: Slice plan with adversarial-review gates

Land the Keychain store and host capture first (custody surface), then the proxy
profile, then the GUI, with adversarial-review gates before any code and before the
GUI default flips to path C.

- Gate 0 (before any code): adversarial review of this doc. Confirm the claim set
  and the custody reversal are acceptable. Blocking.
- Slice 1 status: IMPLEMENTED. `crates/runhaven-core/src/provider/claude_oauth_keychain.rs`
  holds store/read/clear plus the host `claude setup-token` capture (the task
  folded Section 2's capture into Slice 1). `security-framework` is pinned
  `=3.5.1` (NOT the latest 3.7.0: the vendored `codex-network-proxy` pins
  `security-framework = "=3.5.1"` exactly, and 3.x is one semver line, so a
  3.7.0 pin forces a conflicting second copy the resolver rejects; 3.5.1 already
  exposes the full access-control API). `zeroize = "=1.9.0"`. Both are in
  `pins.toml` and the lockfile. The backend is chosen by build profile
  (`select_backend`): release uses the data-protection keychain, an unsigned dev
  build the legacy file keychain; the user-presence `SecAccessControl` binds in
  both. Hermetic tests cover the exact service+account query (claim 2), the
  user-presence-only flag, `parse_token` good/blank/garbage with a buffer-free
  failure message, and a no-persist source tripwire; a `#[ignore]` live
  round-trip on a test service passed against the real Keychain with no prompt.
  MAINTAINER LIVE SMOKES still open: F3 (dev-build vs signed-release ACL and
  data-protection behavior, and whether user presence binds on the legacy
  keychain), F4 (`claude setup-token` prints the token to a pipe like a TTY on
  macOS 26+ with the pinned CLI), and the real biometric prompt on a
  user-presence read. The custom prompt text (`kSecUseOperationPrompt`) is NOT
  set: the high-level `security-framework` `PasswordOptions` API does not expose
  that key, so the system default presence prompt is used; setting custom copy
  would need a direct `core-foundation`/`-sys` dictionary path (a new dep this
  slice avoids). The user-presence gate itself does not depend on it.
- Slice 1 (original plan): `security-framework` re-add (verified pin, macOS-gated) plus the
  `zeroize` pin (verified, all-platform), both in `pins.toml` and locked. Keychain
  store module with the Section 1 item naming, accessibility class, and the
  user-presence `SecAccessControl`
  (`create_with_protection(Some(AccessibleWhenUnlockedThisDeviceOnly), user-presence flag)`);
  store/read/clear where read exercises the LocalAuthentication path, plus the
  `Zeroizing` wrapper so Slices 2 and 3 reuse it. Run the F3 dev-build probe on an
  unsigned and a signed binary (silent, prompts per rebuild, or access-group
  fallback) and decide the dev-build fallback here, not at Gate 2. Test: the exact
  service + account query filter, never `kSecMatchLimitAll` (claim 2), and that the
  item carries the user-presence access control. No GUI or proxy wiring yet.
- Slice 2: host `claude setup-token` capture with the Section 2 transience,
  zeroization guard, and fail-closed paths. Prerequisite: F4 verification on
  macOS 26 that setup-token prints to a pipe like a TTY; if TTY-only, capture under
  a host PTY (`open_gui_attach_pty`). Tests reuse `extract_token`, assert the
  parse-failure path logs only operation + agent (never the buffer), and assert no
  token reaches logs, records, or registry.
- Slice 3 status: IMPLEMENTED. `ClaudeAuthSource` (in
  `runtime/plans/types.rs`) replaced the bare `api_key_broker_env: Option<String>`
  on `RunOptions` and `AgentRunPlan`, with `ApiKeyBrokerEnv(String)` (env NAME
  only, unchanged behavior) and payload-free `KeychainOAuth`. Every workspace
  construction/read site migrated (CLI, Tauri, core, and TUI compile-only). The
  marker (`runtime/active/markers.rs`) now writes the env NAME for the API-key
  path and only a `"claude_auth_source": "keychain-oauth"` tag for the OAuth path,
  never a token or secret-derived value; a test proves the OAuth marker carries
  neither token bytes nor an env name nor the Keychain label. `CLAUDE_OAUTH_BROKER`
  (`provider/auth_broker/profiles.rs`) reuses the existing `BearerAuth` injection
  and `ANTHROPIC_BASE_URL` redirect, pins `api.anthropic.com` + `/v1/` (no egress
  widening). The Keychain secret-source seam lives in
  `provider/runtime/broker_secret.rs` (split out of `runtime.rs` for the file-size
  ceiling): `KeychainOAuth` reads the token via `claude_oauth_keychain::read_token`
  (macOS-gated; non-macOS fails closed) once per run launch into the broker's
  in-memory secret, and fails closed with fixed path-free errors on read
  cancel/error, a missing item, or a non-text/empty token. F5 decision: the guest
  Claude Code sends `anthropic-beta` and `headers.rs` forwards it unchanged, so
  Bearer-only injection is correct today; the host-side fixed-header seam is
  documented on `CLAUDE_OAUTH_BROKER` for a future guest that stops sending it.
  Tests cover Bearer injection + guest-`authorization` stripping + beta
  forwarding, host/path pinning, the Keychain seam (fake reader), fail-closed
  paths, Claude-only enforcement, and the no-secret marker. MAINTAINER LIVE SMOKE
  still open: a real Keychain read triggering the biometric prompt and injecting a
  live token to `api.anthropic.com` (Gate 2). Original plan below.
- Slice 3 (original plan): `CLAUDE_OAUTH_BROKER` profile plus the Keychain secret
  source seam in `provider/runtime.rs`, the `ClaudeAuthSource` selector
  (Section 3, F1), and the `anthropic-beta` decision (F5). Tests: Bearer
  injection, placeholder stripping, path and host pinning, no egress widening, and
  an OAuth-run marker payload with neither token bytes nor any secret-derived
  value.
- Gate 1 (before GUI default flip): adversarial and security review of Slices 1 to
  3 together, focused on the custody boundary, the claim set wording, and the
  zeroization residual. Blocking.
- Slice 4 status: IMPLEMENTED. Three Tauri commands in
  `src-tauri/src/commands/claude_oauth.rs`, each behind the isolated
  `claude-oauth` capability (`windows: ["main"]`, three allow-permissions,
  autogenerated in `build.rs`). `start_claude_oauth_signin` runs the Section 2
  host capture on a background thread (`claude_oauth_signin.rs`, mirroring the
  agent-sign-in runner) behind a Rust-owned native confirmation naming Claude,
  returns a sign-in id, and returns no token; fail-closed on cancel (nothing
  runs) and on capture failure (polled status `failed` with a fixed path-free
  reason). `get_claude_oauth_status` returns only the sanitized
  `not_signed_in | signing | signed_in | failed` enum, derived from Keychain
  PRESENCE plus the in-flight phase via the new attributes-only
  `claude_oauth_keychain::token_present` (queries `kSecReturnAttributes`, never
  `kSecReturnData`, so no user-presence prompt and no token read; never calls
  `read_token`); while signing, presence is not even checked. `clear_claude_oauth`
  deletes only RunHaven's item after a native confirmation and reports `cleared`
  or `already_absent`. No response type carries a token, prefix, or path (a
  no-leak serialization test asserts it); the capability is main-only, isolated
  from run-control and agent-sign-in, and `capability_guard` pins it out of
  main-read and the session-monitor/read windows. Keychain-touching paths are
  macOS-gated (non-macOS fails closed / reports not-signed-in) so the workspace
  builds on the CI shapes. MAINTAINER LIVE SMOKES still open (Gate 2): the real
  `claude setup-token` sign-in, the biometric prompt on a data read, and a live
  presence check confirming attributes-only queries do not prompt. Original plan
  below.
- Slice 4 (original plan): Tauri commands and `claude-oauth` capability
  (Section 4), native confirmation, sanitized status, no secret in responses.
- Slice 5 status: IMPLEMENTED. The Agents Claude control (`ui/src/app/`
  `ClaudeSignIn.tsx`, `useClaudeOAuthSignIn.ts`) makes path C the primary "Sign
  in on your Mac" action (native-confirmed `start_claude_oauth_signin`, then
  polls `get_claude_oauth_status` through `not_signed_in|signing|signed_in|
  failed`, with a native-confirmed `clear_claude_oauth` remove) and keeps path A
  as the secondary "Sign in inside the sandbox instead" opt-in running the
  unchanged T3 terminal login. The three approved tooltip strings ship verbatim
  (exported constants, asserted character-for-character). Rotation (Hardening 4):
  `get_claude_oauth_status` now returns a non-secret `ageDays` from the item's own
  `kSecAttrCreationDate`, read through the SAME attributes-only presence query
  (no `kSecReturnData`, no prompt, no token read; parsed with the already-present
  `time` crate); past the Settings cadence (default 90 days, fixed choices) the
  card surfaces a "sign in again" prompt (a reminder, not a forced expiry). Date
  parsing was split into `provider/claude_oauth_age.rs` to keep the keychain file
  under the structure ceiling. Docs shipped in the same slice: `AUTH_BROKER.md`
  (path C mechanism + the verbatim "does and does not do" claim set + removal +
  rotation), `SECURITY_MODEL.md` (custody-reversal trade-off), `DESKTOP_APP.md`
  (GUI how-to + Settings cadence), and `USAGE.md` (brief pointer). No secret
  crosses to the WebView (Slice 4 holds; the frontend never receives a token and
  the browser preview never invokes Tauri). Original plan below.
- Slice 5 (original plan): GUI control, tooltips, and docs (Section 5), path C
  default, path A opt-in. Fold in the token-rotation cadence setting (Section 1,
  Hardening 4): show token age from the non-secret created-at timestamp and prompt
  a fresh setup-token once past the cadence. Ship docs in the same slice.
- Gate 2 (pre-merge): full verification per `AGENTS.md`, plus a live Keychain
  round-trip and a real setup-token sign-in on macOS 26+, and a final read of the
  approved claim set against the shipped strings.
- Selection wiring fix (F1 + F3, load-bearing; closes the double-coverage review
  block): before this fix, `ClaudeAuthSource::KeychainOAuth` was never constructed
  in a production launch path (the CLI and GUI mapped only the env-var broker), so
  a signed-in token was consumed by nothing and the shipped "uses it to sign your
  runs" claim was false for real runs. The precedence now lives once in
  `provider/runtime/broker_secret.rs::select_claude_auth_source_with_presence`:
  an explicit env-var broker NAME wins; otherwise, for the `claude` profile only,
  a PRESENT Keychain token (the non-prompting attributes-only
  `claude_oauth_token_present`) drives `KeychainOAuth`; otherwise no broker. Both
  the CLI (`app/run_plan.rs::make_run_plan`) and the GUI
  (`commands/launch/plan.rs::agent_run_options`) call it. The GUI selects from
  REAL Keychain presence, never from a WebView value, so the WebView cannot force
  or forge the auth source (it supplies at most the env NAME). Read timing (F3):
  plan preparation and the pre-confirm `validate_runtime_auth_broker_environment`
  now check PRESENCE only for `KeychainOAuth` (no `read_token`, no user-presence
  prompt); the single prompting read stays deferred to broker startup in
  `require_api_key_broker_secret` (`provider/runtime.rs`), after the trusted
  launch confirmation, and fires exactly once. Non-macOS presence is `false`
  (selection yields `None`); a declined Touch ID at startup still fails the run
  closed with the existing path-free error. Tests (hermetic, injected presence +
  reader; no real Keychain): core precedence + read-once/presence-only proof
  (`provider/runtime/tests/selection.rs`), CLI real run-plan build
  (`app/tests.rs`), and GUI launch-plan selection from presence
  (`commands/tests/claude_auth_source.rs`).

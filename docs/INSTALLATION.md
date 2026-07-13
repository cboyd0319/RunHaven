# Installation

RunHaven is a macOS-only app. It requires macOS 26 or newer on Apple silicon
and Apple `container` 1.1.0. Windows, Linux, and Intel Macs are not supported.

RunHaven is alpha/pre-release. `v0.5.0` was the CLI-only release; `v0.7.0` is a
limited CLI plus desktop coworker-test candidate. Coworkers should use the
signed and notarized disk image. Rust, Git, Xcode, and Terminal are not needed
for that path. The archived TUI is not installed.

## Install the coworker-test app

1. Quit an older copy of RunHaven if it is open.
2. Open `RunHaven-0.7.0-macos-arm64.dmg`.
3. Drag **RunHaven** onto the **Applications** folder in that window. Choose
   **Replace** if macOS asks.
4. Eject the **RunHaven 0.7.0** disk image.
5. Open **Applications**, then open **RunHaven**.

Do not run RunHaven from inside the disk image. Installing it in Applications
keeps the app available after the disk image is ejected.

The release owner must sign, notarize, staple, and clean-Mac test the disk image
before sharing it. If macOS says the app cannot be verified, stop and ask for a
new release artifact. Do not bypass Gatekeeper for a coworker-test build.

## First launch

RunHaven checks the Mac and leads with the first item that needs attention.

1. If Apple `container` is missing, choose **Download Apple container** and
   confirm the macOS dialog. RunHaven opens Apple's exact signed 1.1.0 installer
   download in the default browser. Open the downloaded `.pkg`, follow Apple's
   installer, then return to RunHaven and choose **Check again**. Apple asks for
   the Mac administrator password because its installer writes under
   `/usr/local`.
2. If the local container service is stopped, choose **Start container
   service** and confirm. The first start may download Apple's reviewed kernel
   and base file system. RunHaven checks the service again before continuing.
3. Open **Agents**, choose the agent you want, then choose **Install** followed
   by that agent's name, such as **Install Codex**.
   Wait for installation to finish. The first image build can take several
   minutes and needs an internet connection.
4. Complete the sign-in shown for that agent.
5. Return to **Home**, choose **New run**, select the smallest project folder
   the agent needs, review access, and start the run.

RunHaven intentionally requires Apple `container` 1.1.0. If a different
version is installed, choose **Download required version**, confirm the macOS
dialog, and review Apple's installer before replacing the existing version.
You can also get the exact signed package from
[Apple's official release](https://github.com/apple/container/releases/tag/1.1.0).
Return to RunHaven and choose **Check again** when the installer finishes.
RunHaven does not silently upgrade or downgrade a system tool.

## Update or reinstall RunHaven

Quit RunHaven, open the new signed disk image, and drag RunHaven to Applications.
Choose **Replace** when asked. Agent sign-ins, images, and run records stay in
their existing RunHaven and Apple container storage.

## Remove RunHaven

Quit RunHaven and move it from Applications to the Trash. Removing the app does
not delete agent sign-ins, images, run records, or other Apple container data.
RunHaven does not yet have a one-button erase-all action. Its Cleanup view only
removes the specific leftover resources it lists. Ask the release owner for
help if all saved RunHaven data must also be removed. Do not uninstall Apple
`container` only to remove RunHaven; other apps may use it.

## Install from source

Source installation is for contributors. It requires Rust 1.97.0, Git, and the
exact Apple `container` 1.1.0 release. Start Apple container, then install the
CLI from this checkout:

```bash
container system start
cargo install --path crates/runhaven --locked
```

Cargo installs the `runhaven` command into `~/.cargo/bin`. If your terminal
says `command not found: runhaven`, that folder is not on your PATH yet. Add
it once, then open a new terminal:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

For development without installing, run the binary through Cargo:

```bash
cargo run --locked --bin runhaven -- plan shell
```

Development tools and dependencies are exact-pinned in `rust-toolchain.toml`,
`Cargo.toml`, `Cargo.lock`, and `pins.toml`. When updating them, use the
current stable release and commit the exact new version.

Confirm the host before running an agent:

```bash
runhaven doctor
```

`doctor` checks Rust, macOS, Apple silicon, the pinned Apple `container`
version and commit, Apple container system status, and the reviewed runtime
helper surface: builder image, vminit image, and Kata kernel.

## First CLI run

Run the non-mutating guided setup before running an agent:

```bash
runhaven setup
```

`setup` runs the same prerequisite checks as `doctor`, prints exact fixes when
the Mac is not ready, and shows the image build, plan, and run commands for
the selected agent. It does not install Apple `container`, start services,
build images, run agents, write state, or mount a workspace.

Image builds use a separate Apple BuildKit builder VM with its own CPU and
memory limits. Inspect it with `runhaven image doctor AGENT` or
`container builder status`; resize it only with explicit Apple
`container builder` commands.

Build and preview a bundled image:

```bash
runhaven image build claude
runhaven plan claude
```

Run the agent from the project directory you want it to work on:

```bash
runhaven run claude
```

Use the smallest project directory the agent needs. Do not run from your home
directory, a cloud sync root, or a credential folder unless you intentionally
want that broader scope and have reviewed `runhaven plan`.

## Verification

Use focused checks for narrow changes:

```bash
cargo fmt --check
cargo test --locked
cargo run --locked --bin runhaven-check-pins
git diff --check
```

Run full local harness verification before finishing broad code, runtime,
image, security-boundary, or install-flow changes:

```bash
./init.sh
```

Use `runhaven doctor` and Apple `container` runtime smokes when changes affect
the actual macOS container boundary, image templates, agent profiles, provider
networking, or install flow.

## Troubleshooting

In the desktop app, start on **Home** and choose **Check again**. Open **System
Check** to see every check and its next step. If an agent is missing or sign-in
is incomplete, open **Agents**. Copy only the plain error message when asking
for help. Do not share raw logs unless the release owner asks for a specific,
bounded diagnostic.

Contributors using the CLI can start with the guided setup:

```bash
runhaven setup
```

If a run fails, collect these commands before opening an issue:

```bash
runhaven setup
runhaven doctor
runhaven plan <agent>
container system status
```

Do not paste secret values, API keys, SSH keys, private repository contents, or
raw Apple `container inspect` output into issues.

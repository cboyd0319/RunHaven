//! `runhaven login <agent>`: acquire a provider login once so runs reuse it
//! without re-authenticating. All four supported agents (Claude, Codex, Copilot,
//! Antigravity) log in *inside* the sandbox, on the agent's shared home volume
//! (`--auth-scope agent`), in `provider` network mode so the egress allowlist
//! permits the login and token-refresh hosts. The credential stays in that
//! isolated volume and later runs reuse it; RunHaven never holds any agent token.
//!
//! The login commands differ by what each CLI supports: Codex and Copilot have
//! device-code login subcommands; Claude runs `claude auth login` (its default
//! Claude.ai subscription flow), which prints an OAuth URL and reads a pasted
//! code back in the terminal; Antigravity (agy) has no login subcommand, so its
//! login runs the agent, which prompts a Google sign-in on first run (the user
//! types /exit afterward).

use std::io::IsTerminal;

use anyhow::{Result, bail};

use crate::provider::agent_sign_in_record::{
    AGENT_SIGN_IN_SCOPE, record_agent_sign_in, record_agent_sign_out,
};
use crate::runtime::launch::{launch_run_plan_with_completion, new_run_id};
use crate::runtime::lock::acquire_state_lock;
use crate::runtime::plans::{
    AgentRunPlan, AuthScope, RunOptions, WorkspaceScope, build_run_plan, default_network_mode,
};
use crate::runtime::profiles::get_profile;
use crate::runtime::session_state::shared_state_volume_name;
use crate::runtime::state::{VolumeDeletion, delete_volume};
use crate::support::paths::{login_workspace_dir, oauth_token_path};

pub fn login(agent: &str) -> Result<i32> {
    match sandbox_login_command(agent) {
        Some(args) => login_in_sandbox(agent, args, sandbox_login_guidance(agent)),
        None => bail!(
            "runhaven login does not yet support {agent:?}. Run the agent and log in inside the sandbox, or use --api-key-broker-env for an API key."
        ),
    }
}

/// Whether `runhaven login <agent>` can sign this agent in. All four supported
/// agents (claude, codex, copilot, antigravity) log in in-sandbox.
pub fn supports_login(agent: &str) -> bool {
    crate::provider::auth_profiles::agent_sign_in_mode(agent).supports_login()
}

/// Whether this agent has an in-sandbox login command. All four supported agents
/// (claude, codex, copilot, antigravity) do. This is a core capability, not the
/// desktop-only exception; the app uses this same capability for its
/// interactive sign-in terminal.
pub fn supports_sandbox_login(agent: &str) -> bool {
    sandbox_login_command(agent).is_some()
}

/// The command to run inside the sandbox to log in. Codex and Copilot have
/// device-code login subcommands. Claude's wrapper runs `claude auth login`
/// (its default Claude.ai subscription flow), then requires `claude auth status`
/// to confirm readiness. Antigravity (agy) has no login subcommand, so its
/// wrapper runs the agent, then confirms readiness with `agy models`.
fn sandbox_login_command(agent: &str) -> Option<&'static [&'static str]> {
    match agent {
        "claude" => Some(&["runhaven-claude-login"]),
        "codex" => Some(&["codex", "login", "--device-auth"]),
        "copilot" => Some(&["copilot", "login"]),
        "antigravity" => Some(&["runhaven-antigravity-login"]),
        _ => None,
    }
}

/// Build the in-sandbox login run plan for the desktop **terminal** login path:
/// a TTY-on plan (the host-PTY terminal is a real terminal, so paste-back works)
/// with a caller-supplied run id. This is the interactive path the app hosts in
/// an xterm terminal for every supported agent. It is still run through
/// the normal provider launch path (proxy-enforced egress); only the stdio is a
/// host PTY. Fails for an agent with no in-sandbox login command.
pub fn build_sandbox_login_terminal_plan(agent: &str, run_id: &str) -> Result<AgentRunPlan> {
    let login_args = sandbox_login_command(agent)
        .ok_or_else(|| anyhow::anyhow!("{agent} has no in-sandbox login; nothing was started"))?;
    build_sandbox_login_run_plan(agent, login_args, true, run_id)
}

fn sandbox_login_guidance(agent: &str) -> &'static str {
    match agent {
        "claude" => {
            "Logging in to Claude inside the sandbox. This runs `claude auth login`, the Claude.ai subscription flow: open the URL it prints in your browser, approve, then paste the code it gives you back into this terminal. The login persists in the agent's shared home volume, so later runs reuse it."
        }
        "codex" => {
            "Logging in to Codex inside the sandbox. Open the URL it prints and sign in; there is no code to paste back. Codex needs device-code login enabled on your OpenAI account; if it refuses, turn it on in ChatGPT under Settings then Security, and re-run. The login persists in the agent's shared home volume, so later runs reuse it."
        }
        "copilot" => {
            "Logging in to Copilot inside the sandbox. Open https://github.com/login/device, enter the code it prints, and approve. When it asks whether to store the token in a plaintext config file, answer y: in the sandbox that file lives in the isolated volume, not on your Mac. The login persists in the agent's shared home volume, so later runs reuse it."
        }
        "antigravity" => {
            "Logging in to Antigravity inside the sandbox. agy has no separate login command, so this starts agy and it prompts a Google sign-in on first run: open the URL it prints, approve in your browser, and follow any code prompt. Once you are in the agy session, type /exit to leave. The login persists in the agent's shared home volume, so later runs reuse it."
        }
        _ => "Logging in inside the sandbox. The login persists in the agent's shared home volume.",
    }
}

pub fn logout(agent: &str) -> Result<i32> {
    // All agents keep their login in the shared home volume now, so clearing it
    // means deleting that volume.
    if agent == "claude" {
        // Legacy cleanup: an older RunHaven stored a host-side claude OAuth
        // token. That custody is gone, so best-effort remove any file it left so
        // a previously stored token does not linger (ignore NotFound).
        let _ = std::fs::remove_file(oauth_token_path("claude"));
    }
    logout_shared_volume(agent)
}

fn logout_shared_volume(agent: &str) -> Result<i32> {
    logout_shared_volume_with(agent, delete_volume, || {
        record_agent_sign_out(agent, AGENT_SIGN_IN_SCOPE)
    })
}

fn logout_shared_volume_with(
    agent: &str,
    delete: impl FnOnce(&str) -> Result<VolumeDeletion>,
    sign_out: impl FnOnce() -> Result<()>,
) -> Result<i32> {
    let volume = shared_state_volume_name(agent);
    // A launch/login using this state volume holds the same lock for its whole
    // lifetime. Keep deletion and the matching secret-free sign-out marker in
    // one locked critical section so clear cannot race a live credential write.
    let _state_lock = acquire_state_lock(&volume)?;
    // Reuse the shared deletion path (existence check, in-use retry, no error on
    // a volume that was never created).
    match delete(&volume)? {
        VolumeDeletion::Deleted => {
            sign_out()?;
            eprintln!("Cleared the {agent} login (deleted the shared home volume {volume}).");
            Ok(0)
        }
        VolumeDeletion::Missing => {
            sign_out()?;
            eprintln!("No {agent} login volume to clear ({volume}).");
            Ok(0)
        }
        VolumeDeletion::Failed => Ok(1),
    }
}

/// Run an agent's own login command once inside the sandbox, on its shared home
/// volume (`--auth-scope agent`), in `provider` network mode so the egress
/// allowlist permits the login and token-refresh hosts. Stdio is inherited, so
/// the login URL (and any code prompt) reaches the terminal directly and the
/// credential lands in the isolated volume; RunHaven never sees the token.
fn login_in_sandbox(agent: &str, login_args: &[&'static str], guidance: &str) -> Result<i32> {
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    eprintln!("{guidance}");
    let run_id = new_run_id();
    let plan = build_sandbox_login_run_plan(agent, login_args, tty, &run_id)?;
    launch_run_plan_with_completion(&plan, || record_agent_sign_in(agent, AGENT_SIGN_IN_SCOPE))
}

/// Shared plan builder for an in-sandbox login. `tty` is on for the interactive
/// CLI and off for the desktop app's captured-output runner.
fn build_sandbox_login_run_plan(
    agent: &str,
    login_args: &[&'static str],
    tty: bool,
    run_id: &str,
) -> Result<AgentRunPlan> {
    let profile = get_profile(agent)?;
    let network = default_network_mode(&profile);
    build_run_plan(RunOptions {
        profile,
        workspace: login_workspace_dir()?,
        agent_args: login_args.iter().map(|s| (*s).to_string()).collect(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope: AuthScope::Agent,
        read_only_workspace: true,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: true,
        tty,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        claude_auth_source: None,
        worktree: None,
        run_id: Some(run_id.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_login_commands_invoke_each_cli_login() {
        assert_eq!(
            sandbox_login_command("claude"),
            Some(&["runhaven-claude-login"][..])
        );
        assert_eq!(
            sandbox_login_command("codex"),
            Some(&["codex", "login", "--device-auth"][..])
        );
        assert_eq!(
            sandbox_login_command("copilot"),
            Some(&["copilot", "login"][..])
        );
        assert_eq!(
            sandbox_login_command("antigravity"),
            Some(&["runhaven-antigravity-login"][..])
        );
        assert_eq!(sandbox_login_command("gemini"), None);
    }

    #[test]
    fn login_and_refresh_hosts_are_in_the_provider_allowlist() {
        use crate::provider::endpoints::bundled_provider_hosts;
        // Without these the in-sandbox login is blocked by provider egress.
        // Claude's `claude auth login` OAuth exchange hits platform.claude.com
        // and its client-metadata fetch hits claude.ai.
        assert!(bundled_provider_hosts("claude").contains(&"platform.claude.com"));
        assert!(bundled_provider_hosts("claude").contains(&"claude.ai"));
        assert!(bundled_provider_hosts("codex").contains(&"auth.openai.com"));
        assert!(bundled_provider_hosts("copilot").contains(&"github.com"));
        assert!(bundled_provider_hosts("copilot").contains(&"api.github.com"));
        // Antigravity: OAuth token exchange present, and the model family
        // pattern allows any -cloudcode-pa channel/region without opening
        // storage.googleapis.com.
        assert!(bundled_provider_hosts("antigravity").contains(&"oauth2.googleapis.com"));
        let antigravity_hosts: Vec<String> = bundled_provider_hosts("antigravity")
            .iter()
            .map(|h| (*h).to_string())
            .collect();
        let policy =
            crate::provider::egress::EgressPolicy::new(&antigravity_hosts).expect("policy");
        assert!(policy.allows("daily-cloudcode-pa.googleapis.com", 443));
        assert!(policy.allows("us-cloudcode-pa.googleapis.com", 443));
        for denied in [
            "storage.googleapis.com",
            "antigravity-unleash.goog",
            "lh3.googleusercontent.com",
            "play.googleapis.com",
            "playwright.azureedge.net",
            "playwright-akamai.azureedge.net",
            "playwright-verizon.azureedge.net",
        ] {
            assert!(!policy.allows(denied, 443), "{denied} must stay denied");
        }
    }

    #[test]
    fn supports_sandbox_login_covers_the_four_login_agents() {
        // Every supported agent has an in-sandbox login command at the core level.
        for agent in ["claude", "codex", "copilot", "antigravity"] {
            assert!(
                supports_sandbox_login(agent),
                "{agent} should support in-sandbox login"
            );
        }
        for agent in ["gemini", "shell", "unknown"] {
            assert!(
                !supports_sandbox_login(agent),
                "{agent} must not be treated as an in-sandbox login target"
            );
        }
    }

    #[test]
    fn build_sandbox_login_terminal_plan_is_exact_tty_non_root_native_login() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        for (agent, expected) in [
            ("claude", &["runhaven-claude-login"][..]),
            ("codex", &["codex", "login", "--device-auth"][..]),
            ("antigravity", &["runhaven-antigravity-login"][..]),
            ("copilot", &["copilot", "login"][..]),
        ] {
            let plan = build_sandbox_login_terminal_plan(agent, &format!("login-{agent}"))
                .unwrap_or_else(|error| panic!("{agent} plan: {error}"));
            assert!(
                plan.command
                    .windows(expected.len())
                    .any(|window| window == expected)
            );
            // A real terminal: the interactive-plus-TTY flags the captured-output
            // plan omits are present here.
            assert!(plan.command.iter().any(|arg| arg == "--interactive"));
            assert!(plan.command.iter().any(|arg| arg == "--tty"));
            // Always the non-root agent user; never root.
            let user = plan
                .command
                .windows(2)
                .find(|pair| pair[0] == "--user")
                .map(|pair| pair[1].as_str());
            assert_eq!(user, Some("agent"));
            assert!(!plan.command.iter().any(|arg| arg == "root"));
            // No token or API-key broker is ever injected.
            assert!(plan.claude_auth_source.is_none());
            // The caller's login id becomes the run id (the terminal session key).
            assert_eq!(
                plan.run_id.as_deref(),
                Some(format!("login-{agent}").as_str())
            );
        }
        // An agent with no in-sandbox login still fails closed.
        assert!(build_sandbox_login_terminal_plan("unknown", "x").is_err());
    }

    #[test]
    fn sandbox_login_resources_are_owned_by_the_supplied_run_id() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());

        let alpha =
            build_sandbox_login_terminal_plan("codex", "login-alpha").expect("alpha login plan");
        let beta =
            build_sandbox_login_terminal_plan("codex", "login-beta").expect("beta login plan");

        assert_ne!(alpha.container_name, beta.container_name);
        assert_ne!(alpha.network_name, beta.network_name);
    }

    #[test]
    fn supports_login_matches_the_four_login_agents() {
        for agent in ["claude", "codex", "copilot", "antigravity"] {
            assert!(
                supports_login(agent),
                "{agent} should support runhaven login"
            );
        }
        for agent in ["gemini", "shell", "unknown"] {
            assert!(
                !supports_login(agent),
                "{agent} should not support runhaven login"
            );
        }
    }

    #[test]
    fn login_clear_refuses_to_delete_state_while_a_run_holds_the_volume_lock() {
        use std::cell::Cell;

        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let volume = shared_state_volume_name("lock-probe");
        let _held = crate::runtime::lock::acquire_state_lock(&volume).expect("hold state lock");
        let deleted = Cell::new(false);
        let signed_out = Cell::new(false);

        let error = logout_shared_volume_with(
            "lock-probe",
            |_| {
                deleted.set(true);
                Ok(VolumeDeletion::Deleted)
            },
            || {
                signed_out.set(true);
                Ok(())
            },
        )
        .expect_err("a busy shared state volume must not be deleted");

        assert!(error.to_string().contains("already in use"));
        assert!(!deleted.get(), "delete must stay behind the state lock");
        assert!(
            !signed_out.get(),
            "sign-out must stay behind the state lock"
        );
    }

    #[test]
    fn login_clear_holds_the_lock_through_sign_out_then_releases_it() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let volume = shared_state_volume_name("lock-probe");
        let sign_out_volume = volume.clone();

        assert_eq!(
            logout_shared_volume_with(
                "lock-probe",
                |_| Ok(VolumeDeletion::Missing),
                move || {
                    assert!(
                        crate::runtime::lock::acquire_state_lock(&sign_out_volume).is_err(),
                        "sign-out must complete before the shared-volume lock is released"
                    );
                    Ok(())
                },
            )
            .expect("clear"),
            0
        );

        crate::runtime::lock::acquire_state_lock(&volume)
            .expect("clear releases the lock when complete");
    }
}

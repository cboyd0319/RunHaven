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

use crate::runtime::launch::launch_run_plan;
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
/// desktop app's in-app sign-in: the app additionally excludes claude, whose
/// login needs a code pasted back into the terminal, which the app's
/// captured-output runner cannot host (see
/// src-tauri/src/commands/agent_sign_in.rs).
pub fn supports_sandbox_login(agent: &str) -> bool {
    sandbox_login_command(agent).is_some()
}

/// Build the in-sandbox login run plan for an agent without running it, so a
/// caller (the desktop app) can run the exact same login under its own
/// output-capturing runner. The plan is capture-friendly (`tty` off) because the
/// desktop app reads stdout/stderr through pipes rather than a terminal. Fails
/// for an agent with no in-sandbox login command.
pub fn build_sandbox_login_plan(agent: &str) -> Result<AgentRunPlan> {
    let login_args = sandbox_login_command(agent)
        .ok_or_else(|| anyhow::anyhow!("{agent} has no in-sandbox login; nothing was started"))?;
    build_sandbox_login_run_plan(agent, login_args, false)
}

/// The command to run inside the sandbox to log in. Codex and Copilot have
/// device-code login subcommands; Claude runs `claude auth login` (its default
/// Claude.ai subscription flow); Antigravity (agy) has no login subcommand, so
/// it runs the agent, which prompts a Google sign-in on first run.
fn sandbox_login_command(agent: &str) -> Option<&'static [&'static str]> {
    match agent {
        "claude" => Some(&["claude", "auth", "login"]),
        "codex" => Some(&["codex", "login", "--device-auth"]),
        "copilot" => Some(&["copilot", "login"]),
        "antigravity" => Some(&["agy"]),
        _ => None,
    }
}

/// Build the in-sandbox login run plan for the desktop **terminal** login path:
/// a TTY-on plan (the host-PTY terminal is a real terminal, so paste-back works)
/// with a caller-supplied run id. Unlike [`build_sandbox_login_plan`] (TTY off,
/// for the captured-output device-code runner), this is the interactive
/// paste-back path the app hosts in an xterm terminal, so Claude's
/// `claude auth login` code can be pasted back in-app. It is still run through
/// the normal provider launch path (proxy-enforced egress); only the stdio is a
/// host PTY. Fails for an agent with no in-sandbox login command.
pub fn build_sandbox_login_terminal_plan(agent: &str, run_id: &str) -> Result<AgentRunPlan> {
    let login_args = sandbox_login_command(agent)
        .ok_or_else(|| anyhow::anyhow!("{agent} has no in-sandbox login; nothing was started"))?;
    let mut plan = build_sandbox_login_run_plan(agent, login_args, true)?;
    plan.run_id = Some(run_id.to_string());
    Ok(plan)
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
    let volume = shared_state_volume_name(agent);
    // Reuse the shared deletion path (existence check, in-use retry, no error on
    // a volume that was never created).
    match delete_volume(&volume)? {
        VolumeDeletion::Deleted => {
            eprintln!("Cleared the {agent} login (deleted the shared home volume {volume}).");
            Ok(0)
        }
        VolumeDeletion::Missing => {
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
    let plan = build_sandbox_login_run_plan(agent, login_args, tty)?;
    launch_run_plan(&plan)
}

/// Shared plan builder for an in-sandbox login. `tty` is on for the interactive
/// CLI and off for the desktop app's captured-output runner.
fn build_sandbox_login_run_plan(
    agent: &str,
    login_args: &[&'static str],
    tty: bool,
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
        run_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_login_commands_invoke_each_cli_login() {
        assert_eq!(
            sandbox_login_command("claude"),
            Some(&["claude", "auth", "login"][..])
        );
        assert_eq!(
            sandbox_login_command("codex"),
            Some(&["codex", "login", "--device-auth"][..])
        );
        assert_eq!(
            sandbox_login_command("copilot"),
            Some(&["copilot", "login"][..])
        );
        assert_eq!(sandbox_login_command("antigravity"), Some(&["agy"][..]));
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
        assert!(!policy.allows("storage.googleapis.com", 443));
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
    fn build_sandbox_login_plan_uses_the_login_command_and_refuses_unknown_agents() {
        // Keep the login-workspace directory under a temp cache root, not the
        // real user cache.
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let plan = build_sandbox_login_plan("codex").expect("codex login plan");
        // The plan runs the device-auth login inside the sandbox, never a raw
        // host login, and carries no token or API-key broker.
        assert!(plan.command.iter().any(|arg| arg == "codex"));
        assert!(plan.command.iter().any(|arg| arg == "--device-auth"));
        assert!(plan.claude_auth_source.is_none());
        // Captured-output runner: no --tty is added to the run command.
        assert!(!plan.command.iter().any(|arg| arg == "--tty"));
        // Claude now has an in-sandbox login command, so a plan builds; the
        // desktop app, not core, keeps claude out of in-app sign-in (its
        // paste-back step).
        assert!(build_sandbox_login_plan("claude").is_ok());
        // An agent with no login command still fails closed.
        assert!(build_sandbox_login_plan("unknown").is_err());
    }

    #[test]
    fn build_sandbox_login_terminal_plan_is_tty_non_root_claude_login() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = crate::support::paths::override_cache_root_for_tests(cache.path());
        let plan = build_sandbox_login_terminal_plan("claude", "login-seam")
            .expect("claude terminal login plan");
        // Runs `claude auth login` inside the sandbox, interactively, so the
        // printed OAuth code can be pasted back in the host-PTY terminal.
        assert!(plan.command.iter().any(|arg| arg == "claude"));
        assert!(
            plan.command
                .windows(2)
                .any(|pair| pair == ["auth", "login"])
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
        assert_eq!(plan.run_id.as_deref(), Some("login-seam"));
        // An agent with no in-sandbox login still fails closed.
        assert!(build_sandbox_login_terminal_plan("unknown", "x").is_err());
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
}

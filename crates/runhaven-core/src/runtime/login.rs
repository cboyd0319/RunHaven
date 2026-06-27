//! `runhaven login <agent>`: acquire a provider login once so runs reuse it
//! without re-authenticating. Two shapes, by what each CLI supports:
//!
//! - Claude: no in-container device login at the pinned version, so an
//!   in-sandbox login would need a code pasted back. Instead `runhaven login
//!   claude` runs Anthropic's official `claude setup-token` on the host (where
//!   the browser and localhost callback work), stores the resulting OAuth token
//!   `0600` in the RunHaven cache, and later runs inject it into the sandbox env
//!   at run time only. The token is a usable credential, so this is an explicit,
//!   warned opt-in: the guest then holds a token, and provider-network egress
//!   keeps it from leaving the provider's hosts.
//! - Codex, Copilot, and Antigravity: in-sandbox logins on the agent's shared
//!   home volume (`--auth-scope agent`), so the credential stays in that
//!   isolated volume and later runs reuse it; RunHaven never holds the token.
//!   Codex and Copilot have device-code login subcommands; Antigravity (agy)
//!   has none, so its login runs the agent, which prompts a Google sign-in on
//!   first run (the user types /exit afterward).

use std::io::{IsTerminal, Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::doctor::find_on_path;
use crate::runtime::launch::launch_run_plan;
use crate::runtime::plans::{
    AgentRunPlan, AuthScope, RunOptions, WorkspaceScope, build_run_plan, default_network_mode,
};
use crate::runtime::profiles::get_profile;
use crate::runtime::session_state::shared_state_volume_name;
use crate::runtime::state::{VolumeDeletion, delete_volume};
use crate::support::paths::{
    create_private_file, ensure_private_parent, login_workspace_dir, oauth_token_path,
};

/// Agents whose login RunHaven can store host-side as an OAuth token and inject
/// into the guest at run time. Maps the agent to the env var its CLI reads.
fn token_env_var(agent: &str) -> Option<&'static str> {
    match agent {
        "claude" => Some("CLAUDE_CODE_OAUTH_TOKEN"),
        _ => None,
    }
}

pub fn login(agent: &str) -> Result<i32> {
    match agent {
        "claude" => login_claude(),
        "codex" | "copilot" | "antigravity" => {
            let args = sandbox_login_command(agent).expect("agent has a sandbox login command");
            login_in_sandbox(agent, args, sandbox_login_guidance(agent))
        }
        other => bail!(
            "runhaven login does not yet support {other:?}. Run the agent and log in inside the sandbox, or use --api-key-broker-env for an API key."
        ),
    }
}

/// Whether `runhaven login <agent>` can sign this agent in: Claude via the host
/// setup-token, and Codex/Copilot/Antigravity via an in-sandbox login.
pub fn supports_login(agent: &str) -> bool {
    crate::provider::auth_profiles::agent_sign_in_mode(agent).supports_login()
}

/// The command to run inside the sandbox to log in. Codex and Copilot have
/// device-code login subcommands; Antigravity (agy) has no login subcommand, so
/// it runs the agent, which prompts a Google sign-in on first run.
fn sandbox_login_command(agent: &str) -> Option<&'static [&'static str]> {
    match agent {
        "codex" => Some(&["codex", "login", "--device-auth"]),
        "copilot" => Some(&["copilot", "login"]),
        "antigravity" => Some(&["agy"]),
        _ => None,
    }
}

fn sandbox_login_guidance(agent: &str) -> &'static str {
    match agent {
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
    match agent {
        // Codex, Copilot, and Antigravity keep their login in the shared home
        // volume, not a host token file; clearing it means removing that volume.
        "codex" | "copilot" | "antigravity" => logout_shared_volume(agent),
        _ => logout_host_token(agent),
    }
}

fn logout_host_token(agent: &str) -> Result<i32> {
    let path = oauth_token_path(agent);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("could not remove stored login: {}", path.display()))?;
        eprintln!("Cleared the stored {agent} login.");
    } else {
        eprintln!("No stored {agent} login to clear.");
    }
    Ok(0)
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
/// the device-flow URL (and any code prompt) reaches the terminal directly and
/// the credential lands in the isolated volume; RunHaven never sees the token.
fn login_in_sandbox(agent: &str, login_args: &[&'static str], guidance: &str) -> Result<i32> {
    let profile = get_profile(agent)?;
    let network = default_network_mode(&profile);
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    eprintln!("{guidance}");
    let plan = build_run_plan(RunOptions {
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
        api_key_broker_env: None,
        worktree: None,
        run_id: None,
    })?;
    launch_run_plan(&plan)
}

fn login_claude() -> Result<i32> {
    // setup-token runs on the host, where the browser and localhost callback work.
    let claude = find_on_path("claude").ok_or_else(|| {
        anyhow::anyhow!(
            "runhaven login claude needs Claude Code installed on your host to run `claude setup-token`. Install it (for example `npm install -g @anthropic-ai/claude-code`) or log in inside the sandbox with `runhaven run claude`."
        )
    })?;
    eprintln!(
        "Running `claude setup-token` on the host. Approve the login in your browser. The token is stored only for RunHaven and is injected into the sandbox at run time, never written to your `~/.claude`."
    );
    // Capture stdout (the token); inherit stderr/stdin so the browser flow and
    // any prompts work normally.
    let output = Command::new(&claude)
        .arg("setup-token")
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .output()
        .with_context(|| format!("could not run {claude} setup-token"))?;
    if !output.status.success() {
        bail!("claude setup-token did not succeed; nothing was stored");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let token = extract_token(&stdout).ok_or_else(|| {
        anyhow::anyhow!("could not find a token in `claude setup-token` output; nothing was stored")
    })?;
    store_token("claude", &token)?;
    eprintln!(
        "Stored your Claude login. `runhaven run claude` now injects it into the sandbox env (a usable token; provider-network egress keeps it from leaving Anthropic's hosts). Clear it with `runhaven login claude --clear`."
    );
    Ok(0)
}

/// Take the OAuth token from `claude setup-token` output: the last non-empty
/// line that looks like an Anthropic OAuth token.
fn extract_token(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .rfind(|line| line.starts_with("sk-ant-") && line.len() > 20)
        .map(str::to_string)
}

fn store_token(agent: &str, token: &str) -> Result<()> {
    let path = oauth_token_path(agent);
    ensure_private_parent(&path)?;
    let mut file = create_private_file(&path)
        .with_context(|| format!("could not write login store: {}", path.display()))?;
    file.write_all(token.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn read_token(agent: &str) -> Option<String> {
    let mut file = std::fs::File::open(oauth_token_path(agent)).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    let token = contents.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// The token to inject for a run, if the agent has a stored login. Returns the
/// env var name and value. The value is set on the run Command's environment
/// (never on the argv or in the printed plan) and forwarded into the guest with
/// a name-only `--env`.
pub fn run_token_injection(plan: &AgentRunPlan) -> Option<(String, String)> {
    let env = token_env_var(&plan.profile_name)?;
    let token = read_token(&plan.profile_name)?;
    Some((env.to_string(), token))
}

/// Insert a name-only `--env NAME` for the injected token before the image, so
/// the guest inherits the value from the run Command's environment.
pub fn with_token_env(command: &[String], image: &str, env_name: &str) -> Vec<String> {
    let image_index = command
        .iter()
        .position(|arg| arg == image)
        .expect("image in command");
    let mut result = command[..image_index].to_vec();
    result.extend(["--env".to_string(), env_name.to_string()]);
    result.extend(command[image_index..].to_vec());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_token_takes_the_anthropic_oauth_token_line() {
        // Obvious placeholder, not a real token; only the `sk-ant-` prefix and
        // length matter to the parser.
        let placeholder = "sk-ant-NOT-A-REAL-TOKEN-placeholder";
        let out = format!("Starting login...\n{placeholder}\n");
        assert_eq!(extract_token(&out).as_deref(), Some(placeholder));
        assert!(extract_token("no token here\n").is_none());
    }

    #[test]
    fn sandbox_login_commands_invoke_the_cli_device_flow() {
        assert_eq!(
            sandbox_login_command("codex"),
            Some(&["codex", "login", "--device-auth"][..])
        );
        assert_eq!(
            sandbox_login_command("copilot"),
            Some(&["copilot", "login"][..])
        );
        assert_eq!(sandbox_login_command("antigravity"), Some(&["agy"][..]));
        assert_eq!(sandbox_login_command("claude"), None);
    }

    #[test]
    fn login_and_refresh_hosts_are_in_the_provider_allowlist() {
        use crate::provider::endpoints::bundled_provider_hosts;
        // Without these the in-sandbox login is blocked by provider egress.
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
    fn with_token_env_inserts_name_only_env_before_image() {
        let command = vec![
            "container".to_string(),
            "run".to_string(),
            "--name".to_string(),
            "runhaven-claude-run".to_string(),
            "runhaven/claude:0.1.0".to_string(),
            "claude".to_string(),
        ];
        let result = with_token_env(&command, "runhaven/claude:0.1.0", "CLAUDE_CODE_OAUTH_TOKEN");
        let env_index = result
            .windows(2)
            .position(|w| w == ["--env", "CLAUDE_CODE_OAUTH_TOKEN"])
            .expect("env injected");
        let image_index = result
            .iter()
            .position(|arg| arg == "runhaven/claude:0.1.0")
            .expect("image present");
        // The name-only env appears before the image, and the value is never on
        // the command line.
        assert!(env_index < image_index);
        assert!(!result.iter().any(|arg| arg.contains("sk-ant-")));
    }
}
